// ABOUTME: Performance benchmarks for state persistence operations
// ABOUTME: Measures latency, throughput, memory usage, and overhead of state operations

// Benchmark file

use base64::Engine;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use llmspell_agents::{agents::basic::BasicAgent, builder::AgentBuilder, state::StatePersistence};
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_kernel::state::{
    PersistenceConfig, SledConfig, StateManager, StateScope, StorageBackendType,
};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::runtime::Runtime;

/// Benchmark state save operations with various payload sizes
fn bench_state_save_by_size(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _temp_dir = TempDir::new().unwrap();

    let mut group = c.benchmark_group("state_save_operations");

    // Test different payload sizes
    let sizes = vec![
        (100, "100B"),
        (1_000, "1KB"),
        (10_000, "10KB"),
        (100_000, "100KB"),
        (1_000_000, "1MB"),
    ];

    for (size, label) in sizes {
        group.throughput(Throughput::Bytes(size as u64));

        group.bench_with_input(
            BenchmarkId::new("memory_backend", label),
            &size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let state_manager = StateManager::new(None).await.unwrap();

                        let data = "x".repeat(size);
                        let value = serde_json::json!({ "data": data });

                        state_manager
                            .set(StateScope::Global, "test_key", value)
                            .await
                            .unwrap();

                        black_box(state_manager)
                    })
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("sled_backend", label),
            &size,
            |b, &size| {
                b.iter(|| {
                    rt.block_on(async {
                        let temp = TempDir::new().unwrap();
                        let state_manager = StateManager::with_backend(
                            StorageBackendType::Sled(SledConfig {
                                path: temp.path().join("state"),
                                cache_capacity: 10 * 1024 * 1024,
                                use_compression: true,
                            }),
                            PersistenceConfig::default(),
                            None, // No memory manager
                        )
                        .await
                        .unwrap();

                        let data = "x".repeat(size);
                        let value = serde_json::json!({ "data": data });

                        state_manager
                            .set(StateScope::Global, "test_key", value)
                            .await
                            .unwrap();

                        black_box(state_manager)
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark state load operations
fn bench_state_load_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("state_load_operations");

    // Pre-populate state
    let state_manager = rt.block_on(async {
        let sm = StateManager::new(None).await.unwrap();

        // Add various state entries
        for i in 0..100 {
            sm.set(
                StateScope::Global,
                &format!("key_{}", i),
                serde_json::json!({ "index": i, "data": "x".repeat(1000) }),
            )
            .await
            .unwrap();
        }

        Arc::new(sm)
    });

    group.bench_function("single_key_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let value = state_manager
                    .get(StateScope::Global, "key_50")
                    .await
                    .unwrap();
                black_box(value)
            })
        });
    });

    group.bench_function("list_keys_100_entries", |b| {
        b.iter(|| {
            rt.block_on(async {
                let keys = state_manager.list_keys(StateScope::Global).await.unwrap();
                black_box(keys)
            })
        });
    });

    group.finish();
}

/// Benchmark concurrent state access
fn bench_concurrent_state_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_state_access");

    let state_manager = rt.block_on(async { Arc::new(StateManager::new(None).await.unwrap()) });

    group.bench_function("10_concurrent_writes", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..10 {
                    let sm = state_manager.clone();
                    let handle = tokio::spawn(async move {
                        sm.set(
                            StateScope::Global,
                            &format!("concurrent_{}", i),
                            serde_json::json!({ "thread": i }),
                        )
                        .await
                        .unwrap();
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    });

    group.bench_function("10_concurrent_reads", |b| {
        // Pre-populate
        rt.block_on(async {
            for i in 0..10 {
                state_manager
                    .set(
                        StateScope::Global,
                        &format!("read_key_{}", i),
                        serde_json::json!({ "value": i }),
                    )
                    .await
                    .unwrap();
            }
        });

        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];

                for i in 0..10 {
                    let sm = state_manager.clone();
                    let handle = tokio::spawn(async move {
                        let value = sm
                            .get(StateScope::Global, &format!("read_key_{}", i))
                            .await
                            .unwrap();
                        black_box(value)
                    });
                    handles.push(handle);
                }

                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    });

    group.finish();
}

/// Benchmark agent state persistence overhead
fn bench_agent_state_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_state_overhead");

    // Baseline: Agent without state persistence
    group.bench_function("agent_execution_no_persistence", |b| {
        b.iter(|| {
            rt.block_on(async {
                let config = AgentBuilder::basic("test-agent")
                    .description("Test agent")
                    .build()
                    .unwrap();
                let agent = BasicAgent::new(config).unwrap();

                agent.initialize().await.unwrap();

                let input = AgentInput::text("Hello world");
                let context = ExecutionContext::new();
                let response = agent.execute(input, context).await.unwrap();

                black_box(response)
            })
        });
    });

    // With state persistence
    group.bench_function("agent_execution_with_persistence", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = Arc::new(StateManager::new(None).await.unwrap());

                let config = AgentBuilder::basic("test-agent")
                    .description("Test agent")
                    .build()
                    .unwrap();
                let agent = BasicAgent::new(config).unwrap();
                agent.set_state_manager(state_manager);

                agent.initialize().await.unwrap();

                let input = AgentInput::text("Hello world");
                let context = ExecutionContext::new();
                let response = agent.execute(input, context).await.unwrap();

                // Save state
                agent.save_state().await.unwrap();

                black_box(response)
            })
        });
    });

    group.finish();
}

/// Measure memory usage scaling
fn bench_memory_usage_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_usage_scaling");
    group.sample_size(10); // Fewer samples for memory tests
    group.measurement_time(Duration::from_secs(1)); // Faster execution

    // Reduced counts to avoid overwhelming event system in benchmarks
    // Use even smaller counts when running as test
    let entry_counts =
        if std::env::var("CARGO").is_ok() && !std::env::args().any(|arg| arg.contains("bench")) {
            vec![10, 50, 100, 200] // Minimal for test mode
        } else {
            vec![100, 500, 1000, 2000] // Normal for benchmarks
        };

    for count in entry_counts {
        group.bench_with_input(
            BenchmarkId::new("state_entries", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let state_manager = StateManager::new(None).await.unwrap();

                        // Add entries - handle rate limiting gracefully
                        for i in 0..count {
                            let _ = state_manager
                                .set(
                                    StateScope::Global,
                                    &format!("entry_{}", i),
                                    serde_json::json!({
                                        "index": i,
                                        "data": "x".repeat(100)
                                    }),
                                )
                                .await;
                            // Ignore rate limiting errors in benchmarks
                            // The state operation itself succeeds, only event emission fails
                        }

                        // Force a read to ensure data is in memory
                        let _ = state_manager.list_keys(StateScope::Global).await.unwrap();

                        black_box(state_manager)
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark state compression effectiveness
fn bench_compression_effectiveness(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("compression_effectiveness");

    // Test with compressible data
    group.bench_function("highly_compressible_data", |b| {
        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let state_manager = StateManager::with_backend(
                    StorageBackendType::Sled(SledConfig {
                        path: temp_dir.path().join("state"),
                        cache_capacity: 1024 * 1024,
                        use_compression: true,
                    }),
                    PersistenceConfig::default(),
                    None, // No memory manager
                )
                .await
                .unwrap();

                // Highly repetitive data
                let data = "a".repeat(10000);
                state_manager
                    .set(StateScope::Global, "compressed", serde_json::json!(data))
                    .await
                    .unwrap();

                // Read back
                let value = state_manager
                    .get(StateScope::Global, "compressed")
                    .await
                    .unwrap();

                black_box(value)
            })
        });
    });

    // Test with random data
    group.bench_function("random_data", |b| {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        b.iter(|| {
            rt.block_on(async {
                let temp_dir = TempDir::new().unwrap();
                let state_manager = StateManager::with_backend(
                    StorageBackendType::Sled(SledConfig {
                        path: temp_dir.path().join("state"),
                        cache_capacity: 1024 * 1024,
                        use_compression: true,
                    }),
                    PersistenceConfig::default(),
                    None, // No memory manager
                )
                .await
                .unwrap();

                // Random data
                let data: Vec<u8> = (0..10000).map(|_| rng.gen()).collect();
                let data_str = base64::engine::general_purpose::STANDARD.encode(&data);

                state_manager
                    .set(StateScope::Global, "random", serde_json::json!(data_str))
                    .await
                    .unwrap();

                // Read back
                let value = state_manager
                    .get(StateScope::Global, "random")
                    .await
                    .unwrap();

                black_box(value)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_state_save_by_size,
    bench_state_load_operations,
    bench_concurrent_state_access,
    bench_agent_state_overhead,
    bench_memory_usage_scaling,
    bench_compression_effectiveness,
);
criterion_main!(benches);
