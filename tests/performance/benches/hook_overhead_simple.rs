// ABOUTME: Simplified performance test for hook system overhead measurement
// ABOUTME: Tests the overhead of hooks on agent, tool, and workflow operations

use async_trait::async_trait;
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_hooks::{
    ComponentId, ComponentType, Hook, HookContext, HookPoint, HookRegistry, HookResult, Priority,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Simple test hook that does minimal work
struct TestHook {
    name: String,
}

#[async_trait]
impl Hook for TestHook {
    async fn execute(&self, _context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Minimal processing
        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> llmspell_hooks::HookMetadata {
        llmspell_hooks::HookMetadata {
            name: self.name.clone(),
            priority: Priority::NORMAL,
            language: llmspell_hooks::Language::Native,
            description: None,
            tags: vec![],
            version: "1.0.0".to_string(),
        }
    }
}

/// Benchmark hook registration performance
fn bench_hook_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("hook_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = HookRegistry::new();

                // Register 100 hooks
                for i in 0..100 {
                    let hook = TestHook {
                        name: format!("test-hook-{}", i),
                    };
                    registry
                        .register(HookPoint::BeforeAgentExecution, hook)
                        .unwrap();
                }

                black_box(registry)
            })
        });
    });
}

/// Benchmark hook execution overhead
fn bench_hook_execution(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up registry with hooks
    let registry = Arc::new(HookRegistry::new());
    rt.block_on(async {
        for i in 0..10 {
            let hook = TestHook {
                name: format!("test-hook-{}", i),
            };
            registry
                .register(HookPoint::BeforeAgentExecution, hook)
                .unwrap();
        }
    });

    c.bench_function("hook_execution_with_10_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
                let context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

                // Get hooks and execute them
                let hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);

                for hook in hooks {
                    let mut ctx = context.clone();
                    let _ = hook.execute(&mut ctx).await;
                }

                black_box(context)
            })
        });
    });
}

/// Benchmark baseline operation without hooks
fn bench_baseline_operation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("baseline_operation_no_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate some work without hooks
                let mut sum = 0u64;
                for i in 0..1000 {
                    sum = sum.wrapping_add(i);
                    tokio::task::yield_now().await;
                }
                black_box(sum)
            })
        });
    });
}

/// Benchmark operation with hooks
fn bench_operation_with_hooks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    // Set up registry with hooks
    let registry = Arc::new(HookRegistry::new());
    rt.block_on(async {
        for i in 0..5 {
            let hook = TestHook {
                name: format!("test-hook-{}", i),
            };
            registry
                .register(HookPoint::BeforeAgentExecution, hook)
                .unwrap();
        }
    });

    c.bench_function("operation_with_5_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
                let context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

                // Execute hooks before operation
                let hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);
                for hook in hooks {
                    let mut ctx = context.clone();
                    let _ = hook.execute(&mut ctx).await;
                }

                // Simulate some work
                let mut sum = 0u64;
                for i in 0..1000 {
                    sum = sum.wrapping_add(i);
                    tokio::task::yield_now().await;
                }

                black_box(sum)
            })
        });
    });
}

/// Calculate and print hook overhead percentage
fn calculate_hook_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Hook Overhead Analysis ===");

    rt.block_on(async {
        use tokio::time::Instant;

        // Measure baseline
        let baseline_start = Instant::now();
        for _ in 0..1000 {
            let mut sum = 0u64;
            for i in 0..100 {
                sum = sum.wrapping_add(i);
            }
            black_box(sum);
        }
        let baseline_duration = baseline_start.elapsed();

        // Measure with hooks
        let registry = Arc::new(HookRegistry::new());
        for i in 0..5 {
            let hook = TestHook {
                name: format!("test-hook-{}", i),
            };
            registry
                .register(HookPoint::BeforeAgentExecution, hook)
                .unwrap();
        }

        let with_hooks_start = Instant::now();
        for _ in 0..1000 {
            let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
            let context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

            // Execute hooks
            let hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);
            for hook in hooks {
                let mut ctx = context.clone();
                let _ = hook.execute(&mut ctx).await;
            }

            let mut sum = 0u64;
            for i in 0..100 {
                sum = sum.wrapping_add(i);
            }
            black_box(sum);
        }
        let with_hooks_duration = with_hooks_start.elapsed();

        let overhead_ms =
            (with_hooks_duration.as_secs_f64() - baseline_duration.as_secs_f64()) * 1000.0;
        let overhead_percent = (overhead_ms / (baseline_duration.as_secs_f64() * 1000.0)) * 100.0;

        println!("Baseline: {:?}", baseline_duration);
        println!("With 5 hooks: {:?}", with_hooks_duration);
        println!("Overhead: {:.2}ms ({:.2}%)", overhead_ms, overhead_percent);
        println!("Target: <5% overhead");
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
    bench_hook_registration,
    bench_hook_execution,
    bench_baseline_operation,
    bench_operation_with_hooks,
    calculate_hook_overhead
);
criterion_main!(benches);
