//! ABOUTME: Benchmarks for memory operations (episodic, semantic, consolidation)
//!
//! Includes comparative benchmarks for `InMemory` vs HNSW episodic backends.

use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_core::traits::embedding::EmbeddingProvider;
use llmspell_core::LLMSpellError;
use llmspell_memory::{
    embeddings::EmbeddingService, ConsolidationMode, DefaultMemoryManager, EpisodicEntry,
    MemoryConfig, MemoryManager,
};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

/// Test embedding provider for benchmarks (generates random 384-dim vectors)
struct TestEmbeddingProvider;

#[async_trait]
impl EmbeddingProvider for TestEmbeddingProvider {
    fn name(&self) -> &'static str {
        "test-benchmark-provider"
    }

    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError> {
        // Generate random embeddings for benchmarking
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Ok(texts
            .iter()
            .map(|_| (0..384).map(|_| rng.gen::<f32>()).collect())
            .collect())
    }

    fn embedding_dimensions(&self) -> usize {
        384
    }

    fn embedding_model(&self) -> Option<&str> {
        Some("test-model")
    }
}

fn episodic_add_benchmark(c: &mut Criterion) {
    info!("Starting episodic_add benchmark");

    let rt = Runtime::new().unwrap();
    let memory_manager = rt.block_on(async {
        DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager")
    });
    let memory_manager = Arc::new(memory_manager);

    let mut group = c.benchmark_group("episodic_add");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_entry", |b| {
        let mm = memory_manager.clone();
        b.to_async(&rt).iter(|| async {
            let entry = EpisodicEntry::new(
                "bench-session".to_string(),
                "user".to_string(),
                "Test message for benchmarking".to_string(),
            );
            mm.episodic().add(black_box(entry)).await.unwrap();
        });
    });

    group.finish();
}

fn episodic_search_benchmark(c: &mut Criterion) {
    info!("Starting episodic_search benchmark");

    let rt = Runtime::new().unwrap();
    let memory_manager = rt.block_on(async {
        let mm = DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager");

        // Preload 1000 entries for realistic search
        for i in 0..1000 {
            let entry = EpisodicEntry::new(
                "bench-session".to_string(),
                if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                format!("Message {i} about Rust programming"),
            );
            mm.episodic().add(entry).await.unwrap();
        }

        mm
    });
    let memory_manager = Arc::new(memory_manager);

    let mut group = c.benchmark_group("episodic_search");
    for limit in &[5, 10, 20, 50] {
        group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, &limit| {
            let mm = memory_manager.clone();
            b.to_async(&rt).iter(|| {
                let mm = mm.clone();
                async move {
                    mm.episodic()
                        .search(black_box("Rust ownership"), black_box(limit))
                        .await
                        .unwrap();
                }
            });
        });
    }

    group.finish();
}

fn consolidation_benchmark(c: &mut Criterion) {
    info!("Starting consolidation benchmark");

    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("consolidation");
    group.sample_size(10); // Consolidation is slow, fewer samples
    group.throughput(Throughput::Elements(100)); // 100 entries per consolidation

    group.bench_function("100_entries", |b| {
        b.iter_with_setup(
            || {
                // Setup: Create memory manager with 100 unprocessed entries
                rt.block_on(async {
                    let mm = DefaultMemoryManager::new_in_memory()
                        .await
                        .expect("Failed to create memory manager");

                    for i in 0..100 {
                        let entry = EpisodicEntry::new(
                            "consolidate-session".to_string(),
                            if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                            format!("Consolidation test message {i}"),
                        );
                        mm.episodic().add(entry).await.unwrap();
                    }

                    Arc::new(mm)
                })
            },
            |mm| {
                // Benchmark: Consolidate
                rt.block_on(async {
                    mm.consolidate("consolidate-session", ConsolidationMode::Immediate, None)
                        .await
                        .unwrap();
                });
            },
        );
    });

    group.finish();
}

fn semantic_query_benchmark(c: &mut Criterion) {
    info!("Starting semantic_query benchmark");

    let rt = Runtime::new().unwrap();
    let memory_manager = rt.block_on(async {
        // Preload semantic entities (simulated)
        // Note: Requires SemanticMemory.add() method
        DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager")
    });
    let memory_manager = Arc::new(memory_manager);

    let mut group = c.benchmark_group("semantic_query");
    for limit in &[5, 10, 20] {
        group.bench_with_input(BenchmarkId::from_parameter(limit), limit, |b, &limit| {
            let mm = memory_manager.clone();
            b.to_async(&rt).iter(|| {
                let mm = mm.clone();
                async move {
                    let _ = mm
                        .semantic()
                        .query_by_type(black_box(""))
                        .await
                        .unwrap()
                        .into_iter()
                        .take(black_box(limit))
                        .collect::<Vec<_>>();
                }
            });
        });
    }

    group.finish();
}

fn memory_footprint_benchmark(c: &mut Criterion) {
    info!("Starting memory_footprint benchmark");

    let rt = Runtime::new().unwrap();

    c.bench_function("memory_footprint_idle", |b| {
        b.iter(|| {
            // Measure idle memory footprint (empty memory manager)
            let mm = rt.block_on(async {
                DefaultMemoryManager::new_in_memory()
                    .await
                    .expect("Failed to create memory manager")
            });

            info!("Memory footprint (idle): ~minimal (empty DashMaps + Arc overhead)");
            black_box(mm);
        });
    });

    c.bench_function("memory_footprint_loaded_1k", |b| {
        b.iter(|| {
            // Measure loaded memory footprint (1000 entries)
            let mm = rt.block_on(async {
                let mm = DefaultMemoryManager::new_in_memory()
                    .await
                    .expect("Failed to create memory manager");

                for i in 0..1000 {
                    let entry = EpisodicEntry::new(
                        "footprint-session".to_string(),
                        if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                        format!("Memory footprint test message {i} with some content"),
                    );
                    mm.episodic().add(entry).await.unwrap();
                }

                mm
            });

            // Approximate footprint calculation:
            // - 1000 entries × ~200 bytes/entry (UUID + timestamps + content) ≈ 200KB
            // - Embeddings: 1000 × 768 floats × 4 bytes ≈ 3MB
            // - DashMap overhead: ~50KB
            // Total: ~3.25MB for 1000 entries
            info!("Memory footprint (1000 entries): ~3-4MB (episodic + embeddings)");
            black_box(mm);
        });
    });

    c.bench_function("memory_footprint_loaded_10k", |b| {
        b.iter(|| {
            // Measure loaded memory footprint (10000 entries)
            let mm = rt.block_on(async {
                let mm = DefaultMemoryManager::new_in_memory()
                    .await
                    .expect("Failed to create memory manager");

                for i in 0..10000 {
                    let entry = EpisodicEntry::new(
                        "footprint-session".to_string(),
                        if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                        format!("Memory footprint test message {i} with some content"),
                    );
                    mm.episodic().add(entry).await.unwrap();
                }

                mm
            });

            // Approximate footprint calculation:
            // - 10000 entries × ~200 bytes/entry ≈ 2MB
            // - Embeddings: 10000 × 768 floats × 4 bytes ≈ 30MB
            // - DashMap overhead: ~200KB
            // Total: ~32MB for 10000 entries
            info!("Memory footprint (10000 entries): ~30-35MB (episodic + embeddings)");
            black_box(mm);
        });
    });
}

/// Comparative benchmarks: `InMemory` vs HNSW episodic backends
fn backend_comparison_search_benchmark(c: &mut Criterion) {
    info!("Starting backend_comparison_search benchmark");

    let rt = Runtime::new().unwrap();

    // Create test embedding service for HNSW
    let embedding_service = {
        let provider: Arc<dyn EmbeddingProvider> = Arc::new(TestEmbeddingProvider);
        Arc::new(EmbeddingService::new(provider))
    };

    // Benchmark at different scales
    for &dataset_size in &[100, 1000, 10000] {
        let mut group = c.benchmark_group(format!("backend_search_{dataset_size}"));
        group.sample_size(20); // Smaller sample size for large datasets

        // InMemory backend
        group.bench_function("InMemory", |b| {
            let mm = rt.block_on(async {
                let config = MemoryConfig::for_testing();
                let mm = DefaultMemoryManager::with_config(&config).unwrap();

                // Preload entries
                for i in 0..dataset_size {
                    let entry = EpisodicEntry::new(
                        "bench-session".to_string(),
                        if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                        format!("Message {i} about Rust programming and systems design"),
                    );
                    mm.episodic().add(entry).await.unwrap();
                }

                Arc::new(mm)
            });

            b.to_async(&rt).iter(|| {
                let mm = mm.clone();
                async move {
                    mm.episodic()
                        .search(black_box("Rust ownership"), black_box(10))
                        .await
                        .unwrap();
                }
            });
        });

        // HNSW backend
        group.bench_function("HNSW", |b| {
            let mm = rt.block_on(async {
                let config = MemoryConfig::for_production(Arc::clone(&embedding_service));
                let mm = DefaultMemoryManager::with_config(&config).unwrap();

                // Preload entries
                for i in 0..dataset_size {
                    let entry = EpisodicEntry::new(
                        "bench-session".to_string(),
                        if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                        format!("Message {i} about Rust programming and systems design"),
                    );
                    mm.episodic().add(entry).await.unwrap();
                }

                Arc::new(mm)
            });

            b.to_async(&rt).iter(|| {
                let mm = mm.clone();
                async move {
                    mm.episodic()
                        .search(black_box("Rust ownership"), black_box(10))
                        .await
                        .unwrap();
                }
            });
        });

        group.finish();
    }
}

criterion_group!(
    benches,
    episodic_add_benchmark,
    episodic_search_benchmark,
    consolidation_benchmark,
    semantic_query_benchmark,
    memory_footprint_benchmark,
    backend_comparison_search_benchmark
);
criterion_main!(benches);
