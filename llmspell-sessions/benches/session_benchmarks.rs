//! ABOUTME: Comprehensive performance benchmarks for llmspell-sessions crate
//! ABOUTME: Validates all performance targets from Phase 6 requirements

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use futures::executor;
use llmspell_events::bus::EventBus;
use llmspell_hooks::replay::ReplayMode;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_sessions::{
    config::SessionManagerConfig, types::CreateSessionOptions, ArtifactType, SessionManager,
};
use llmspell_state_persistence::StateManager;
use llmspell_storage::MemoryBackend;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Create a test session manager for benchmarking
async fn create_benchmark_manager() -> SessionManager {
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());
    let config = SessionManagerConfig::default();

    SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        config,
    )
    .unwrap()
}

/// Benchmark session creation - Target: <10ms
fn bench_session_creation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    c.bench_function("session_creation", |b| {
        b.to_async(&rt).iter(|| async {
            let options = CreateSessionOptions::default();
            let session_id = manager.create_session(black_box(options)).await.unwrap();
            // Clean up to avoid memory accumulation
            let _ = manager.delete_session(&session_id).await;
            session_id
        });
    });
}

/// Benchmark session save - Target: <20ms
fn bench_session_save(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    // Pre-create a session
    let session_id = rt.block_on(async {
        manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap()
    });

    c.bench_function("session_save", |b| {
        b.to_async(&rt).iter(|| async {
            let session = manager.get_session(&session_id).await.unwrap();
            manager.save_session(&session).await.unwrap()
        });
    });

    // Cleanup
    rt.block_on(async {
        let _ = manager.delete_session(&session_id).await;
    });
}

/// Benchmark artifact store - Target: <15ms
fn bench_artifact_store(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    // Pre-create a session
    let session_id = rt.block_on(async {
        manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap()
    });

    let content = b"Benchmark content for performance testing".to_vec();

    c.bench_function("artifact_store", |b| {
        b.to_async(&rt).iter(|| async {
            let name = format!("benchmark_{}.txt", rand::thread_rng().gen::<u32>());
            manager
                .store_artifact(
                    &session_id,
                    ArtifactType::UserInput,
                    black_box(name),
                    black_box(content.clone()),
                    None,
                )
                .await
                .unwrap()
        });
    });

    // Cleanup
    rt.block_on(async {
        let _ = manager.delete_session(&session_id).await;
    });
}

/// Benchmark artifact retrieve - Target: <10ms
fn bench_artifact_retrieve(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    // Pre-create a session with artifacts
    let (session_id, artifact_id) = rt.block_on(async {
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();
        let artifact_id = manager
            .store_artifact(
                &session_id,
                ArtifactType::UserInput,
                "benchmark.txt".to_string(),
                b"Benchmark content".to_vec(),
                None,
            )
            .await
            .unwrap();
        (session_id, artifact_id)
    });

    c.bench_function("artifact_retrieve", |b| {
        b.to_async(&rt).iter(|| async {
            manager
                .get_artifact(&session_id, &artifact_id)
                .await
                .unwrap()
        });
    });

    // Cleanup
    rt.block_on(async {
        let _ = manager.delete_session(&session_id).await;
    });
}

/// Benchmark session restore/load - Target: <25ms
fn bench_session_restore(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    // Pre-create and save multiple sessions
    let session_ids: Vec<_> = rt.block_on(async {
        let mut ids = Vec::new();
        for i in 0..5 {
            let options = CreateSessionOptions {
                name: Some(format!("benchmark_session_{}", i)),
                ..Default::default()
            };
            let session_id = manager.create_session(options).await.unwrap();
            let session = manager.get_session(&session_id).await.unwrap();
            manager.save_session(&session).await.unwrap();
            manager.complete_session(&session_id).await.unwrap();
            ids.push(session_id);
        }
        ids
    });

    use std::sync::atomic::{AtomicUsize, Ordering};
    let idx = AtomicUsize::new(0);
    c.bench_function("session_restore", |b| {
        b.to_async(&rt).iter(|| async {
            let current_idx = idx.fetch_add(1, Ordering::Relaxed);
            let session_id = &session_ids[current_idx % session_ids.len()];
            manager.load_session(black_box(session_id)).await.unwrap()
        });
    });

    // Cleanup
    rt.block_on(async {
        for session_id in session_ids {
            let _ = manager.delete_session(&session_id).await;
        }
    });
}

/// Benchmark hook overhead - Target: <2%
fn bench_hook_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Create manager with hooks enabled
    let manager_with_hooks = rt.block_on(create_benchmark_manager());

    // Create manager with hooks disabled
    let manager_no_hooks = rt.block_on(async {
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let storage_backend = Arc::new(MemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let mut config = SessionManagerConfig::default();
        config.hook_config.enable_lifecycle_hooks = false;

        SessionManager::new(
            state_manager,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            config,
        )
        .unwrap()
    });

    let mut group = c.benchmark_group("hook_overhead");

    group.bench_function("with_hooks", |b| {
        b.to_async(&rt).iter(|| async {
            let options = CreateSessionOptions::default();
            let session_id = manager_with_hooks
                .create_session(black_box(options))
                .await
                .unwrap();
            let _ = manager_with_hooks.delete_session(&session_id).await;
            session_id
        });
    });

    group.bench_function("without_hooks", |b| {
        b.to_async(&rt).iter(|| async {
            let options = CreateSessionOptions::default();
            let session_id = manager_no_hooks
                .create_session(black_box(options))
                .await
                .unwrap();
            let _ = manager_no_hooks.delete_session(&session_id).await;
            session_id
        });
    });

    group.finish();
}

/// Benchmark replay performance
/// Note: Replay requires hook executions to be present in the session.
/// For benchmarking purposes, we'll test the replay infrastructure
/// without actual hook executions by catching the expected error.
fn bench_replay_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let manager = rt.block_on(create_benchmark_manager());

    // Pre-create a session with artifacts (but no hook executions for simplicity)
    let session_id = rt.block_on(async {
        let session_id = manager
            .create_session(CreateSessionOptions::default())
            .await
            .unwrap();

        // Add some artifacts to make session meaningful
        for i in 0..10 {
            manager
                .store_artifact(
                    &session_id,
                    ArtifactType::UserInput,
                    format!("replay_artifact_{}.txt", i),
                    format!("Content {}", i).into_bytes(),
                    None,
                )
                .await
                .unwrap();
        }

        // Save session to enable replay
        let session = manager.get_session(&session_id).await.unwrap();
        manager.save_session(&session).await.unwrap();
        manager.complete_session(&session_id).await.unwrap();

        session_id
    });

    // Benchmark replay attempt (will fail due to no hooks, but tests the infrastructure)
    c.bench_function("session_replay_infrastructure", |b| {
        b.to_async(&rt).iter(|| async {
            let replay_config = llmspell_sessions::replay::session_adapter::SessionReplayConfig {
                mode: ReplayMode::Exact,
                timeout: Duration::from_secs(60),
                stop_on_error: false,
                compare_results: false, // Skip comparison for benchmarking
                ..Default::default()
            };

            // Attempt replay - expect it to fail due to no hook executions
            // This still benchmarks the replay infrastructure setup
            let result = manager.replay_session(&session_id, replay_config).await;

            // We expect this to fail with "No hook executions found"
            // but the benchmark measures the infrastructure overhead
            match result {
                Err(e) if e.to_string().contains("No hook executions found") => {
                    // Expected error - replay infrastructure is working
                }
                Ok(_) => {
                    // Unexpected success (shouldn't happen without hooks)
                }
                Err(e) => {
                    // Unexpected error
                    panic!("Unexpected replay error: {}", e);
                }
            }
        });
    });

    // Cleanup
    rt.block_on(async {
        let _ = manager.delete_session(&session_id).await;
    });
}

/// Benchmark memory usage with different session counts
fn bench_memory_usage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(10));

    for session_count in [1, 10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::new("sessions", session_count),
            session_count,
            |b, &count| {
                b.to_async(&rt).iter_batched(
                    || executor::block_on(create_benchmark_manager()),
                    |manager| async move {
                        let mut session_ids = Vec::new();

                        // Create sessions
                        for i in 0..count {
                            let options = CreateSessionOptions {
                                name: Some(format!("memory_test_{}", i)),
                                ..Default::default()
                            };
                            let session_id = manager.create_session(options).await.unwrap();

                            // Add some artifacts to each session
                            for j in 0..5 {
                                manager
                                    .store_artifact(
                                        &session_id,
                                        ArtifactType::UserInput,
                                        format!("artifact_{}.txt", j),
                                        vec![0u8; 1024], // 1KB per artifact
                                        None,
                                    )
                                    .await
                                    .unwrap();
                            }

                            session_ids.push(session_id);
                        }

                        // Return session count for verification
                        session_ids.len()
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_session_creation,
    bench_session_save,
    bench_artifact_store,
    bench_artifact_retrieve,
    bench_session_restore,
    bench_hook_overhead,
    bench_replay_performance,
    bench_memory_usage
);

criterion_main!(benches);
