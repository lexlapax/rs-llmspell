// ABOUTME: Performance test for circuit breaker effectiveness
// ABOUTME: Validates circuit breaker triggers correctly under load and protects system performance

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_core::{
    component::ComponentMetadata, execution::ExecutionContext, tracing::correlation::CorrelationId,
};
use llmspell_hooks::{
    circuit_breaker::{CircuitBreaker, CircuitBreakerConfig},
    performance::{PerformanceMonitor, PerformanceThreshold},
    HookContext, HookPoint, HookResult, HookSystem, Priority,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

/// Test circuit breaker activation under normal load
fn bench_circuit_breaker_normal_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_normal_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let hook_system = Arc::new(HookSystem::new());
                let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout: Duration::from_secs(30),
                    half_open_max_calls: 3,
                });

                // Register a normal hook that succeeds
                hook_system
                    .register_hook(
                        HookPoint::BeforeAgentExecution,
                        Box::new(|_ctx: &HookContext| {
                            Box::pin(async move {
                                // Simulate normal processing
                                sleep(Duration::from_micros(100)).await;
                                HookResult::Continue
                            })
                        }),
                        Priority::Normal,
                        Some("normal-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Execute hooks 100 times - should not trip circuit breaker
                for i in 0..100 {
                    let context = HookContext::new(
                        ComponentMetadata::agent("test-agent").id,
                        CorrelationId::new(),
                    );

                    let result = circuit_breaker
                        .execute(|| {
                            hook_system.execute_hooks(HookPoint::BeforeAgentExecution, context)
                        })
                        .await;

                    black_box(result);
                }

                // Circuit breaker should still be closed
                assert!(circuit_breaker.is_closed().await);
            });
        });
    });
}

/// Test circuit breaker activation under failing conditions
fn bench_circuit_breaker_failure_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_failure_detection", |b| {
        b.iter(|| {
            rt.block_on(async {
                let hook_system = Arc::new(HookSystem::new());
                let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 5,
                    success_threshold: 3,
                    timeout: Duration::from_millis(100),
                    half_open_max_calls: 3,
                });

                // Register a failing hook
                let failure_count = Arc::new(tokio::sync::Mutex::new(0));
                let fc = failure_count.clone();

                hook_system
                    .register_hook(
                        HookPoint::BeforeToolExecution,
                        Box::new(move |_ctx: &HookContext| {
                            let fc = fc.clone();
                            Box::pin(async move {
                                let mut count = fc.lock().await;
                                *count += 1;

                                // Fail first 10 calls
                                if *count <= 10 {
                                    Err(llmspell_core::LLMSpellError::Hook {
                                        message: "Simulated failure".to_string(),
                                        hook_point: "BeforeToolExecution".to_string(),
                                        source: None,
                                    })
                                } else {
                                    Ok(HookResult::Continue)
                                }
                            })
                        }),
                        Priority::High,
                        Some("failing-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Execute until circuit breaker opens
                let mut open_at = None;
                for i in 0..20 {
                    let context = HookContext::new(
                        ComponentMetadata::tool("test-tool").id,
                        CorrelationId::new(),
                    );

                    let result = circuit_breaker
                        .execute(|| {
                            hook_system.execute_hooks(HookPoint::BeforeToolExecution, context)
                        })
                        .await;

                    if circuit_breaker.is_open().await && open_at.is_none() {
                        open_at = Some(i);
                    }

                    black_box(result);
                }

                // Circuit breaker should have opened after 5 failures
                assert!(open_at.is_some());
                assert!(open_at.unwrap() >= 5);
            });
        });
    });
}

/// Test circuit breaker recovery (half-open to closed transition)
fn bench_circuit_breaker_recovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                let hook_system = Arc::new(HookSystem::new());
                let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 3,
                    success_threshold: 3,
                    timeout: Duration::from_millis(50), // Short timeout for testing
                    half_open_max_calls: 3,
                });

                // Register a hook that fails initially then succeeds
                let call_count = Arc::new(tokio::sync::Mutex::new(0));
                let cc = call_count.clone();

                hook_system
                    .register_hook(
                        HookPoint::BeforeWorkflowStart,
                        Box::new(move |_ctx: &HookContext| {
                            let cc = cc.clone();
                            Box::pin(async move {
                                let mut count = cc.lock().await;
                                *count += 1;

                                // Fail first 5 calls, then succeed
                                if *count <= 5 {
                                    Err(llmspell_core::LLMSpellError::Hook {
                                        message: "Temporary failure".to_string(),
                                        hook_point: "BeforeWorkflowStart".to_string(),
                                        source: None,
                                    })
                                } else {
                                    Ok(HookResult::Continue)
                                }
                            })
                        }),
                        Priority::Normal,
                        Some("recovering-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Trip the circuit breaker
                for _ in 0..5 {
                    let context = HookContext::new(
                        ComponentMetadata::workflow("test-workflow").id,
                        CorrelationId::new(),
                    );

                    let _ = circuit_breaker
                        .execute(|| {
                            hook_system.execute_hooks(HookPoint::BeforeWorkflowStart, context)
                        })
                        .await;
                }

                assert!(circuit_breaker.is_open().await);

                // Wait for timeout
                sleep(Duration::from_millis(60)).await;

                // Should be half-open now, execute successful calls
                for _ in 0..5 {
                    let context = HookContext::new(
                        ComponentMetadata::workflow("test-workflow").id,
                        CorrelationId::new(),
                    );

                    let _ = circuit_breaker
                        .execute(|| {
                            hook_system.execute_hooks(HookPoint::BeforeWorkflowStart, context)
                        })
                        .await;
                }

                // Should be closed again
                assert!(circuit_breaker.is_closed().await);
            });
        });
    });
}

/// Test performance monitor integration with circuit breaker
fn bench_performance_monitor_integration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("performance_monitor_circuit_breaker", |b| {
        b.iter(|| {
            rt.block_on(async {
                let hook_system = Arc::new(HookSystem::new());
                let performance_monitor = Arc::new(PerformanceMonitor::new());
                let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

                // Set performance thresholds
                performance_monitor
                    .set_threshold(
                        HookPoint::BeforeAgentExecution,
                        PerformanceThreshold {
                            max_duration: Duration::from_millis(10),
                            max_overhead_percentage: 5.0,
                        },
                    )
                    .await;

                // Register a slow hook
                hook_system
                    .register_hook(
                        HookPoint::BeforeAgentExecution,
                        Box::new(|_ctx: &HookContext| {
                            Box::pin(async move {
                                // Simulate slow processing
                                sleep(Duration::from_millis(20)).await;
                                HookResult::Continue
                            })
                        }),
                        Priority::Normal,
                        Some("slow-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Execute with monitoring
                let mut threshold_violations = 0;

                for _ in 0..10 {
                    let context = HookContext::new(
                        ComponentMetadata::agent("test-agent").id,
                        CorrelationId::new(),
                    );

                    let start = tokio::time::Instant::now();

                    let result = circuit_breaker
                        .execute(|| {
                            hook_system
                                .execute_hooks(HookPoint::BeforeAgentExecution, context.clone())
                        })
                        .await;

                    let duration = start.elapsed();

                    // Record performance
                    performance_monitor
                        .record_execution(HookPoint::BeforeAgentExecution, duration)
                        .await;

                    // Check if threshold was violated
                    if duration > Duration::from_millis(10) {
                        threshold_violations += 1;
                        performance_monitor
                            .record_threshold_violation(HookPoint::BeforeAgentExecution, context)
                            .await;
                    }

                    black_box(result);
                }

                // Should have detected threshold violations
                assert!(threshold_violations > 0);

                let stats = performance_monitor
                    .get_statistics(HookPoint::BeforeAgentExecution)
                    .await;

                assert!(stats.is_some());
                let stats = stats.unwrap();
                assert!(stats.avg_duration > Duration::from_millis(10));
            });
        });
    });
}

/// Test circuit breaker under concurrent load
fn bench_circuit_breaker_concurrent_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_concurrent_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let hook_system = Arc::new(HookSystem::new());
                let circuit_breaker = Arc::new(CircuitBreaker::new(CircuitBreakerConfig {
                    failure_threshold: 10,
                    success_threshold: 5,
                    timeout: Duration::from_secs(1),
                    half_open_max_calls: 5,
                }));

                // Register a hook that fails randomly
                hook_system
                    .register_hook(
                        HookPoint::BeforeToolExecution,
                        Box::new(|_ctx: &HookContext| {
                            Box::pin(async move {
                                // 30% failure rate
                                if rand::random::<f32>() < 0.3 {
                                    Err(llmspell_core::LLMSpellError::Hook {
                                        message: "Random failure".to_string(),
                                        hook_point: "BeforeToolExecution".to_string(),
                                        source: None,
                                    })
                                } else {
                                    sleep(Duration::from_micros(100)).await;
                                    Ok(HookResult::Continue)
                                }
                            })
                        }),
                        Priority::Normal,
                        Some("random-hook".to_string()),
                    )
                    .await
                    .unwrap();

                // Spawn concurrent tasks
                let mut handles = vec![];

                for task_id in 0..100 {
                    let hs = hook_system.clone();
                    let cb = circuit_breaker.clone();

                    let handle = tokio::spawn(async move {
                        let mut successes = 0;
                        let mut failures = 0;

                        for _ in 0..10 {
                            let context = HookContext::new(
                                ComponentMetadata::tool(&format!("tool-{}", task_id)).id,
                                CorrelationId::new(),
                            );

                            let result = cb
                                .execute(|| {
                                    hs.execute_hooks(HookPoint::BeforeToolExecution, context)
                                })
                                .await;

                            match result {
                                Ok(_) => successes += 1,
                                Err(_) => failures += 1,
                            }
                        }

                        (successes, failures)
                    });

                    handles.push(handle);
                }

                // Wait for all tasks
                let mut total_successes = 0;
                let mut total_failures = 0;

                for handle in handles {
                    let (s, f) = handle.await.unwrap();
                    total_successes += s;
                    total_failures += f;
                }

                println!("\nConcurrent load test results:");
                println!("Successes: {}", total_successes);
                println!("Failures: {}", total_failures);
                println!(
                    "Circuit breaker state: {:?}",
                    circuit_breaker.get_state().await
                );

                black_box((total_successes, total_failures));
            });
        });
    });
}

/// Verify circuit breaker effectiveness metrics
fn verify_circuit_breaker_effectiveness(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Circuit Breaker Effectiveness Analysis ===");

    rt.block_on(async {
        let hook_system = Arc::new(HookSystem::new());
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_millis(100),
            half_open_max_calls: 3,
        });

        // Register a hook that always fails
        hook_system
            .register_hook(
                HookPoint::OnError,
                Box::new(|_ctx: &HookContext| {
                    Box::pin(async move {
                        Err(llmspell_core::LLMSpellError::Hook {
                            message: "Always fails".to_string(),
                            hook_point: "OnError".to_string(),
                            source: None,
                        })
                    })
                }),
                Priority::Highest,
                Some("always-fail-hook".to_string()),
            )
            .await
            .unwrap();

        // Measure time to circuit breaker opening
        let start = tokio::time::Instant::now();
        let mut calls_before_open = 0;

        while !circuit_breaker.is_open().await && calls_before_open < 20 {
            let context =
                HookContext::new(ComponentMetadata::agent("test").id, CorrelationId::new());

            let _ = circuit_breaker
                .execute(|| hook_system.execute_hooks(HookPoint::OnError, context))
                .await;

            calls_before_open += 1;
        }

        let time_to_open = start.elapsed();

        println!("Circuit breaker opened after {} calls", calls_before_open);
        println!("Time to open: {:?}", time_to_open);
        println!("Expected: 5 calls (failure threshold)");
        println!(
            "Status: {}",
            if calls_before_open <= 6 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );

        // Test rejection rate when open
        let mut rejected = 0;
        for _ in 0..100 {
            let context =
                HookContext::new(ComponentMetadata::agent("test").id, CorrelationId::new());

            match circuit_breaker
                .execute(|| hook_system.execute_hooks(HookPoint::OnError, context))
                .await
            {
                Err(e) if e.to_string().contains("circuit breaker is open") => rejected += 1,
                _ => {}
            }
        }

        println!("\nRejection rate when open: {}%", rejected);
        println!("Expected: 100% (all calls rejected)");
        println!(
            "Status: {}",
            if rejected >= 95 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_circuit_breaker_normal_load,
    bench_circuit_breaker_failure_detection,
    bench_circuit_breaker_recovery,
    bench_performance_monitor_integration,
    bench_circuit_breaker_concurrent_load,
    verify_circuit_breaker_effectiveness
);
criterion_main!(benches);
