//! ABOUTME: Performance benchmarks for RAG operations
//! ABOUTME: Measures vector search, embedding, and ingestion performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_bridge::rag_bridge::{RAGBridge, RAGDocument, RAGIngestRequest, RAGSearchRequest};
use llmspell_bridge::ProviderManager;
use llmspell_config::providers::ProviderManagerConfig;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_sessions::{SessionManager, SessionManagerConfig};
use llmspell_state_persistence::StateManager;
use llmspell_storage::backends::vector::hnsw::HNSWVectorStorage;
use llmspell_storage::vector_storage::HNSWConfig;
use llmspell_storage::MemoryBackend;
use llmspell_tenancy::manager::MultiTenantVectorManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn setup_rag_bridge(rt: &Runtime) -> Arc<RAGBridge> {
    rt.block_on(async {
        // Setup state manager
        let state_manager = Arc::new(StateManager::new().await.unwrap());

        // Setup session manager
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(llmspell_hooks::HookRegistry::new());
        let hook_executor = Arc::new(llmspell_hooks::HookExecutor::new());
        let event_bus = Arc::new(llmspell_events::bus::EventBus::new());
        let session_config = SessionManagerConfig::default();

        let session_manager = Arc::new(
            SessionManager::new(
                state_manager.clone(),
                storage_backend,
                hook_registry,
                hook_executor,
                &event_bus,
                session_config,
            )
            .unwrap(),
        );

        // Setup HNSW vector storage with optimized parameters
        let hnsw_config = HNSWConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            max_elements: 1_000_000, // 1M vectors
            seed: None,
            metric: llmspell_storage::vector_storage::DistanceMetric::Cosine,
            allow_replace_deleted: true,
            num_threads: None,
        };
        let vector_storage = Arc::new(HNSWVectorStorage::new(384, hnsw_config));

        // Setup multi-tenant infrastructure
        let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage));
        let multi_tenant_rag = Arc::new(MultiTenantRAG::new(tenant_manager));

        // Setup provider manager
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());
        let core_providers = providers.create_core_manager_arc().await.unwrap();

        Arc::new(RAGBridge::new(
            state_manager,
            session_manager,
            multi_tenant_rag,
            core_providers,
        ))
    })
}

fn generate_documents(count: usize) -> Vec<RAGDocument> {
    (0..count)
        .map(|i| {
            let mut metadata = HashMap::new();
            metadata.insert("index".to_string(), serde_json::json!(i));
            metadata.insert("category".to_string(), serde_json::json!(format!("cat_{}", i % 10)));

            RAGDocument {
                id: format!("doc_{i}"),
                text: format!("This is document number {i}. It contains various information about topic {i} and relates to category {}.", i % 10),
                metadata: Some(metadata),
            }
        })
        .collect()
}

fn bench_vector_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    // Pre-populate with test data
    rt.block_on(async {
        let docs = generate_documents(1000);
        let request = RAGIngestRequest {
            documents: docs,
            scope: Some("bench".to_string()),
            scope_id: Some("search_bench".to_string()),
            provider: None,
            chunking: None,
        };
        bridge.ingest(request, None).await.unwrap();
    });

    let mut group = c.benchmark_group("vector_search");

    // Benchmark different k values
    for k in &[1, 5, 10, 50] {
        group.bench_with_input(BenchmarkId::new("k", k), k, |b, &k| {
            b.to_async(&rt).iter(|| async {
                let request = RAGSearchRequest {
                    query: "document information category".to_string(),
                    k: Some(k),
                    scope: Some("bench".to_string()),
                    scope_id: Some("search_bench".to_string()),
                    filters: None,
                    threshold: None,
                };
                black_box(bridge.search(request, None).await.unwrap())
            });
        });
    }

    group.finish();
}

fn bench_document_ingestion(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    let mut group = c.benchmark_group("document_ingestion");

    // Benchmark different document counts
    for count in &[1, 10, 100, 500] {
        let docs = generate_documents(*count);

        group.bench_with_input(BenchmarkId::new("docs", count), &docs, |b, docs| {
            b.to_async(&rt).iter(|| async {
                let request = RAGIngestRequest {
                    documents: docs.clone(),
                    scope: Some("bench".to_string()),
                    scope_id: Some(format!("ingest_bench_{}", uuid::Uuid::new_v4())),
                    provider: None,
                    chunking: None,
                };
                black_box(bridge.ingest(request, None).await.unwrap())
            });
        });
    }

    group.finish();
}

fn bench_filtered_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    // Pre-populate with test data
    rt.block_on(async {
        let docs = generate_documents(5000);
        let request = RAGIngestRequest {
            documents: docs,
            scope: Some("bench".to_string()),
            scope_id: Some("filter_bench".to_string()),
            provider: None,
            chunking: None,
        };
        bridge.ingest(request, None).await.unwrap();
    });

    let mut group = c.benchmark_group("filtered_search");

    // Benchmark with no filters
    group.bench_function("no_filters", |b| {
        b.to_async(&rt).iter(|| async {
            let request = RAGSearchRequest {
                query: "document information".to_string(),
                k: Some(10),
                scope: Some("bench".to_string()),
                scope_id: Some("filter_bench".to_string()),
                filters: None,
                threshold: None,
            };
            black_box(bridge.search(request, None).await.unwrap())
        });
    });

    // Benchmark with filters
    group.bench_function("with_filters", |b| {
        b.to_async(&rt).iter(|| async {
            let mut filters = HashMap::new();
            filters.insert("category".to_string(), serde_json::json!("cat_5"));

            let request = RAGSearchRequest {
                query: "document information".to_string(),
                k: Some(10),
                scope: Some("bench".to_string()),
                scope_id: Some("filter_bench".to_string()),
                filters: Some(filters),
                threshold: None,
            };
            black_box(bridge.search(request, None).await.unwrap())
        });
    });

    // Benchmark with threshold
    group.bench_function("with_threshold", |b| {
        b.to_async(&rt).iter(|| async {
            let request = RAGSearchRequest {
                query: "document information".to_string(),
                k: Some(10),
                scope: Some("bench".to_string()),
                scope_id: Some("filter_bench".to_string()),
                filters: None,
                threshold: Some(0.7),
            };
            black_box(bridge.search(request, None).await.unwrap())
        });
    });

    group.finish();
}

fn bench_chunking_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
    let docs = vec![RAGDocument {
        id: "long_doc".to_string(),
        text: long_text,
        metadata: None,
    }];

    let mut group = c.benchmark_group("chunking_strategies");

    // Benchmark different chunk sizes
    for chunk_size in &[100, 250, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            chunk_size,
            |b, &chunk_size| {
                b.to_async(&rt).iter(|| async {
                    let request = RAGIngestRequest {
                        documents: docs.clone(),
                        scope: Some("bench".to_string()),
                        scope_id: Some(format!("chunk_bench_{}", uuid::Uuid::new_v4())),
                        provider: None,
                        chunking: Some(llmspell_bridge::rag_bridge::ChunkingConfig {
                            chunk_size: Some(chunk_size),
                            overlap: Some(50),
                            strategy: Some("sliding_window".to_string()),
                        }),
                    };
                    black_box(bridge.ingest(request, None).await.unwrap())
                });
            },
        );
    }

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    // Pre-populate
    rt.block_on(async {
        let docs = generate_documents(1000);
        let request = RAGIngestRequest {
            documents: docs,
            scope: Some("bench".to_string()),
            scope_id: Some("concurrent_bench".to_string()),
            provider: None,
            chunking: None,
        };
        bridge.ingest(request, None).await.unwrap();
    });

    let mut group = c.benchmark_group("concurrent_operations");

    // Benchmark concurrent searches
    for concurrency in &[1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("concurrent_searches", concurrency),
            concurrency,
            |b, &concurrency| {
                b.to_async(&rt).iter(|| async {
                    let handles: Vec<_> = (0..concurrency)
                        .map(|i| {
                            let bridge = bridge.clone();
                            tokio::spawn(async move {
                                let request = RAGSearchRequest {
                                    query: format!("document information {i}"),
                                    k: Some(10),
                                    scope: Some("bench".to_string()),
                                    scope_id: Some("concurrent_bench".to_string()),
                                    filters: None,
                                    threshold: None,
                                };
                                bridge.search(request, None).await.unwrap()
                            })
                        })
                        .collect();

                    for handle in handles {
                        black_box(handle.await.unwrap());
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = setup_rag_bridge(&rt);

    let mut group = c.benchmark_group("memory_usage");
    group.sample_size(10); // Smaller sample for memory benchmarks

    // Benchmark memory per vector
    for vector_count in &[100, 1000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("vectors", vector_count),
            vector_count,
            |b, &vector_count| {
                b.to_async(&rt).iter(|| async {
                    let docs = generate_documents(vector_count);
                    let request = RAGIngestRequest {
                        documents: docs,
                        scope: Some("bench".to_string()),
                        scope_id: Some(format!("memory_bench_{}", uuid::Uuid::new_v4())),
                        provider: None,
                        chunking: None,
                    };

                    let result = bridge.ingest(request, None).await.unwrap();

                    // Get stats to measure memory impact
                    let stats = bridge
                        .get_stats(
                            "bench",
                            Some(&format!("memory_bench_{}", result.documents_processed)),
                        )
                        .await
                        .unwrap();

                    black_box(stats)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_vector_search,
    bench_document_ingestion,
    bench_filtered_search,
    bench_chunking_strategies,
    bench_concurrent_operations,
    bench_memory_usage
);
criterion_main!(benches);
