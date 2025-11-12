//! ABOUTME: Benchmarks for context assembly operations

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_bridge::ContextBridge;
use llmspell_memory::{DefaultMemoryManager, MemoryManager};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tracing::info;

fn context_assemble_benchmark(c: &mut Criterion) {
    info!("Starting context_assemble benchmark");

    let rt = Runtime::new().unwrap();
    let context_bridge = rt.block_on(async {
        let mm = DefaultMemoryManager::new_in_memory()
            .expect("Failed to create memory manager");

        // Preload memory for realistic context assembly
        for i in 0..500 {
            let entry = llmspell_memory::EpisodicEntry::new(
                "bench-session".to_string(),
                if i % 2 == 0 { "user" } else { "assistant" }.to_string(),
                format!("Context assembly test message {i} about Rust"),
            );
            mm.episodic().add(entry).await.unwrap();
        }

        Arc::new(ContextBridge::new(Arc::new(mm)))
    });

    let mut group = c.benchmark_group("context_assemble");

    for strategy in &["episodic", "hybrid"] {
        for budget in &[1000, 2000, 4000] {
            group.bench_with_input(
                BenchmarkId::new(*strategy, budget),
                &(strategy, budget),
                |b, &(strategy, budget)| {
                    let cb = context_bridge.clone();
                    b.to_async(&rt).iter(|| {
                        let cb = cb.clone();
                        async move {
                            cb.assemble(
                                black_box("Rust ownership model"),
                                black_box(strategy),
                                black_box(*budget),
                                Some(black_box("bench-session")),
                            )
                            .unwrap();
                        }
                    });
                },
            );
        }
    }

    group.finish();
}

fn context_parallel_retrieval_benchmark(c: &mut Criterion) {
    info!("Starting context_parallel_retrieval benchmark");

    let rt = Runtime::new().unwrap();
    let context_bridge = rt.block_on(async {
        let mm = DefaultMemoryManager::new_in_memory()
            .expect("Failed to create memory manager");
        Arc::new(ContextBridge::new(Arc::new(mm)))
    });

    let mut group = c.benchmark_group("context_parallel_retrieval");
    group.throughput(Throughput::Elements(4)); // 4 parallel queries

    group.bench_function("4_parallel_queries", |b| {
        let cb = context_bridge.clone();
        b.to_async(&rt).iter(|| {
            let cb = cb.clone();
            async move {
                // Simulate parallel retrieval from multiple sources
                let futures = vec![
                    cb.assemble("query1", "episodic", 500, None),
                    cb.assemble("query2", "episodic", 500, None),
                    cb.assemble("query3", "episodic", 500, None),
                    cb.assemble("query4", "episodic", 500, None),
                ];

                let _results = futures::future::join_all(black_box(futures)).await;
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    context_assemble_benchmark,
    context_parallel_retrieval_benchmark
);
criterion_main!(benches);
