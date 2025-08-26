// ABOUTME: Performance test for circuit breaker effectiveness
// ABOUTME: Validates circuit breaker triggers correctly under load and protects system performance

// Benchmark file

use anyhow::anyhow;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_hooks::{
    circuit_breaker::{BreakerConfig, CircuitBreakerManager},
    BreakerState, CircuitBreaker,
};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::time::sleep;

/// Test circuit breaker activation under normal load
fn bench_circuit_breaker_normal_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_normal_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let circuit_breaker = CircuitBreaker::new("test-breaker".to_string());

                // Execute operations that succeed - should not trip circuit breaker
                for _i in 0..100 {
                    // Record successful operation
                    circuit_breaker.record_success(Duration::from_micros(100));

                    // Check state remains closed
                    let state = circuit_breaker.state();
                    assert_eq!(state, BreakerState::Closed);
                }

                black_box(circuit_breaker)
            })
        });
    });
}

/// Test circuit breaker activation under failure conditions
fn bench_circuit_breaker_failure_activation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_failure_activation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let breaker_config = BreakerConfig {
                    failure_threshold: 3, // Trip after 3 failures
                    ..Default::default()
                };

                let circuit_breaker =
                    CircuitBreaker::with_config("failing-breaker".to_string(), breaker_config);

                // Record failures to trip the breaker
                for _ in 0..3 {
                    circuit_breaker.record_failure(&anyhow!("Test failure"));
                }

                // Check state is now open
                let state = circuit_breaker.state();
                assert_eq!(state, BreakerState::Open);

                // Check if operation is allowed (should be false)
                let allowed = circuit_breaker.can_execute();
                assert!(!allowed);

                black_box(circuit_breaker)
            })
        });
    });
}

/// Test circuit breaker recovery process
fn bench_circuit_breaker_recovery(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_recovery", |b| {
        b.iter(|| {
            rt.block_on(async {
                let breaker_config = BreakerConfig {
                    failure_threshold: 3,
                    success_threshold: 2,
                    open_duration: Duration::from_millis(10), // Short for testing
                    ..Default::default()
                };

                let circuit_breaker =
                    CircuitBreaker::with_config("recovery-breaker".to_string(), breaker_config);

                // Trip the breaker
                for _ in 0..3 {
                    circuit_breaker.record_failure(&anyhow!("Test failure"));
                }

                // Wait for half-open state
                sleep(Duration::from_millis(15)).await;

                // Trigger transition to half-open by checking if we can execute
                assert!(
                    circuit_breaker.can_execute(),
                    "Should transition to half-open"
                );

                // Record successes to recover
                for _ in 0..2 {
                    circuit_breaker.record_success(Duration::from_micros(100));
                }

                // Check state is closed again
                let state = circuit_breaker.state();
                assert_eq!(state, BreakerState::Closed);

                black_box(circuit_breaker)
            })
        });
    });
}

/// Benchmark circuit breaker overhead on hook execution
fn bench_circuit_breaker_hook_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_hook_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                let circuit_breaker = CircuitBreaker::new("overhead-test".to_string());

                // Measure overhead of checking circuit breaker state
                let start = tokio::time::Instant::now();

                for _ in 0..1000 {
                    let allowed = circuit_breaker.can_execute();
                    if allowed {
                        // Simulate hook execution
                        circuit_breaker.record_success(Duration::from_micros(10));
                    }
                }

                let duration = start.elapsed();
                let overhead_per_check = duration.as_nanos() / 1000;

                black_box(overhead_per_check)
            })
        });
    });
}

/// Test circuit breaker manager with multiple components
fn bench_circuit_breaker_manager(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("circuit_breaker_manager_multi_component", |b| {
        b.iter(|| {
            rt.block_on(async {
                let manager = CircuitBreakerManager::new();

                // Create multiple components
                let component_names: Vec<_> = (0..10).map(|i| format!("agent-{}", i)).collect();

                // Simulate mixed success/failure patterns
                for i in 0..100 {
                    for (idx, name) in component_names.iter().enumerate() {
                        let breaker = manager.get_or_create(name);

                        if i % (idx + 2) == 0 {
                            // Failure pattern varies by component
                            breaker.record_failure(&anyhow!("Test failure"));
                        } else {
                            breaker.record_success(Duration::from_micros(100));
                        }
                    }
                }

                // Check states
                let mut open_count = 0;
                for name in &component_names {
                    let breaker = manager.get_or_create(name);
                    let state = breaker.state();
                    if state == BreakerState::Open {
                        open_count += 1;
                    }
                }

                black_box(open_count)
            })
        });
    });
}

/// Calculate overhead percentage for circuit breaker
fn calculate_circuit_breaker_overhead(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Circuit Breaker Overhead Analysis ===");

    rt.block_on(async {
        let circuit_breaker = CircuitBreaker::new("overhead-analysis".to_string());

        // Baseline: Direct execution without circuit breaker
        let start = tokio::time::Instant::now();
        for _ in 0..10_000 {
            // Simulate some work
            black_box(42 + 42);
        }
        let baseline = start.elapsed();

        // With circuit breaker
        let start = tokio::time::Instant::now();
        for _ in 0..10_000 {
            if circuit_breaker.can_execute() {
                // Simulate same work
                black_box(42 + 42);
                circuit_breaker.record_success(Duration::from_micros(1));
            }
        }
        let with_breaker = start.elapsed();

        let overhead_ns = with_breaker.as_nanos().saturating_sub(baseline.as_nanos());
        #[allow(clippy::cast_precision_loss)]
        let overhead_ns_f64 = overhead_ns as f64;
        #[allow(clippy::cast_precision_loss)]
        let baseline_ns_f64 = baseline.as_nanos() as f64;
        let overhead_percent = (overhead_ns_f64 / baseline_ns_f64) * 100.0;

        println!("Baseline execution: {:?}", baseline);
        println!("With circuit breaker: {:?}", with_breaker);
        println!("Overhead: {:.2}%", overhead_percent);
        println!("Target: <5%");
        println!(
            "Status: {}",
            if overhead_percent < 5.0 {
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
    bench_circuit_breaker_failure_activation,
    bench_circuit_breaker_recovery,
    bench_circuit_breaker_hook_overhead,
    bench_circuit_breaker_manager,
    calculate_circuit_breaker_overhead
);
criterion_main!(benches);
