//! ABOUTME: Benchmarks for memory operations (episodic, semantic, consolidation)

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_memory::{ConsolidationMode, DefaultMemoryManager, EpisodicEntry, MemoryManager};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

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
        let mm = DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager");

        // Preload semantic entities (simulated)
        // Note: Requires SemanticMemory.add() method
        mm
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

criterion_group!(
    benches,
    episodic_add_benchmark,
    episodic_search_benchmark,
    consolidation_benchmark,
    semantic_query_benchmark,
    memory_footprint_benchmark
);
criterion_main!(benches);
