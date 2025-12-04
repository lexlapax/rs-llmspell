//! ABOUTME: Performance benchmarks for RAG bridge functionality
//! ABOUTME: Measures vector search, ingestion, and filtering performance

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_bridge::rag_bridge::{ChunkingConfig, RAGBridge, RAGDocument};
use llmspell_bridge::ProviderManager;
use llmspell_config::providers::ProviderManagerConfig;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_kernel::sessions::{SessionManager, SessionManagerConfig};
use llmspell_kernel::state::StateManager;
use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_storage::MemoryBackend;
use llmspell_storage::VectorStorage;
use llmspell_tenancy::MultiTenantVectorManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

// Setup a test bridge instance for benchmarking
async fn setup_bridge() -> Arc<RAGBridge> {
    // Setup state manager
    let state_manager = Arc::new(StateManager::new(None).await.unwrap());

    // Setup session manager
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());
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

    // Setup SQLite vector storage (in-memory for benchmarking)
    let config = SqliteConfig::in_memory();
    let backend = Arc::new(SqliteBackend::new(config).await.unwrap());
    let vector_storage = Arc::new(SqliteVectorStorage::new(backend, 384).await.unwrap());

    // Setup multi-tenant infrastructure
    let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage.clone()));
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
        Some(vector_storage as Arc<dyn VectorStorage>),
    ))
}

fn generate_documents(count: usize) -> Vec<RAGDocument> {
    (0..count)
        .map(|i| {
            let mut metadata = HashMap::new();
            metadata.insert("index".to_string(), serde_json::json!(i));
            metadata.insert(
                "category".to_string(),
                serde_json::json!(format!("cat_{}", i % 10)),
            );
            RAGDocument {
                id: format!("doc_{i}"),
                text: format!("This is document number {i} containing information about various topics including category {}", i % 10),
                metadata: Some(metadata),
            }
        })
        .collect()
}

fn bench_vector_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = rt.block_on(setup_bridge());

    // Pre-populate with documents
    rt.block_on(async {
        let docs = generate_documents(1000);
        bridge
            .ingest(
                docs,
                Some("bench".to_string()),
                Some("search_bench".to_string()),
                None,
                None,
                None,
            )
            .await
            .unwrap();
    });

    let mut group = c.benchmark_group("vector_search");

    // Benchmark different k values
    for k in &[1, 5, 10, 50] {
        group.bench_with_input(BenchmarkId::new("k", k), k, |b, &k| {
            b.to_async(&rt).iter(|| async {
                black_box(
                    bridge
                        .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                            query: "document information category".to_string(),
                            k: Some(k),
                            scope: Some("bench".to_string()),
                            scope_id: Some("search_bench".to_string()),
                            filters: None,
                            threshold: None,
                            context: None,
                        })
                        .await
                        .unwrap(),
                )
            });
        });
    }

    group.finish();
}

fn bench_document_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("document_ingestion");

    for doc_count in &[10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("docs", doc_count),
            doc_count,
            |b, &count| {
                let rt = Runtime::new().unwrap();
                let bridge = rt.block_on(setup_bridge());
                let docs = generate_documents(count);

                b.to_async(&rt).iter(|| async {
                    black_box(
                        bridge
                            .ingest(
                                docs.clone(),
                                Some("bench".to_string()),
                                Some(format!("ingest_bench_{}", uuid::Uuid::new_v4())),
                                None,
                                None,
                                None,
                            )
                            .await
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_filtered_search(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = rt.block_on(setup_bridge());

    // Pre-populate with documents
    rt.block_on(async {
        let docs = generate_documents(5000);
        bridge
            .ingest(
                docs,
                Some("bench".to_string()),
                Some("filter_bench".to_string()),
                None,
                None,
                None,
            )
            .await
            .unwrap();
    });

    let mut group = c.benchmark_group("filtered_search");

    // Benchmark with no filters
    group.bench_function("no_filters", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                bridge
                    .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                        query: "document information".to_string(),
                        k: Some(10),
                        scope: Some("bench".to_string()),
                        scope_id: Some("filter_bench".to_string()),
                        filters: None,
                        threshold: None,
                        context: None,
                    })
                    .await
                    .unwrap(),
            )
        });
    });

    // Benchmark with filters
    group.bench_function("with_filters", |b| {
        b.to_async(&rt).iter(|| async {
            let mut filters = HashMap::new();
            filters.insert("category".to_string(), serde_json::json!("cat_5"));

            black_box(
                bridge
                    .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                        query: "document information".to_string(),
                        k: Some(10),
                        scope: Some("bench".to_string()),
                        scope_id: Some("filter_bench".to_string()),
                        filters: Some(filters),
                        threshold: None,
                        context: None,
                    })
                    .await
                    .unwrap(),
            )
        });
    });

    // Benchmark with threshold
    group.bench_function("with_threshold", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(
                bridge
                    .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                        query: "document information".to_string(),
                        k: Some(10),
                        scope: Some("bench".to_string()),
                        scope_id: Some("filter_bench".to_string()),
                        filters: None,
                        threshold: Some(0.7),
                        context: None,
                    })
                    .await
                    .unwrap(),
            )
        });
    });

    group.finish();
}

fn bench_chunking_strategies(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("chunking_strategies");

    // Generate a large document for chunking
    let long_doc = RAGDocument {
        id: "long_doc".to_string(),
        text: "This is a very long document. ".repeat(500),
        metadata: None,
    };

    for chunk_size in &[100, 256, 512] {
        group.bench_with_input(
            BenchmarkId::new("chunk_size", chunk_size),
            chunk_size,
            |b, &chunk_size| {
                let bridge = rt.block_on(setup_bridge());
                let doc = long_doc.clone();

                b.to_async(&rt).iter(|| async {
                    let chunking = ChunkingConfig {
                        chunk_size: Some(chunk_size),
                        overlap: Some(50),
                        strategy: Some("sliding_window".to_string()),
                    };

                    black_box(
                        bridge
                            .ingest(
                                vec![doc.clone()],
                                Some("bench".to_string()),
                                Some(format!("chunk_bench_{}", uuid::Uuid::new_v4())),
                                None,
                                Some(chunking),
                                None,
                            )
                            .await
                            .unwrap(),
                    )
                });
            },
        );
    }

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let bridge = rt.block_on(setup_bridge());

    // Pre-populate with documents
    rt.block_on(async {
        let docs = generate_documents(1000);
        bridge
            .ingest(
                docs,
                Some("bench".to_string()),
                Some("concurrent_bench".to_string()),
                None,
                None,
                None,
            )
            .await
            .unwrap();
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
                                bridge
                                    .search(llmspell_bridge::rag_bridge::RAGSearchParams {
                                        query: format!("document information {i}"),
                                        k: Some(10),
                                        scope: Some("bench".to_string()),
                                        scope_id: Some("concurrent_bench".to_string()),
                                        filters: None,
                                        threshold: None,
                                        context: None,
                                    })
                                    .await
                                    .unwrap()
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

fn bench_memory_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_impact");

    for doc_count in &[100, 1000, 5000] {
        group.bench_with_input(
            BenchmarkId::new("vector_count", doc_count),
            doc_count,
            |b, &count| {
                let rt = Runtime::new().unwrap();
                let bridge = rt.block_on(setup_bridge());

                b.to_async(&rt).iter(|| async {
                    let docs = generate_documents(count);

                    let result = bridge
                        .ingest(
                            docs,
                            Some("bench".to_string()),
                            Some(format!("memory_bench_{}", uuid::Uuid::new_v4())),
                            None,
                            None,
                            None,
                        )
                        .await
                        .unwrap();

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
    bench_memory_impact
);
criterion_main!(benches);
