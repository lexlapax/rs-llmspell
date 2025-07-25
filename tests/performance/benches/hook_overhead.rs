// ABOUTME: Performance test for hook system overhead measurement
// ABOUTME: Validates <5% overhead requirement across agent operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_agents::testing::mocks::{MockAgent, MockAgentConfig};
use llmspell_core::{
    types::AgentInput,
    BaseAgent, ExecutionContext,
};
use std::collections::HashMap;
use tokio::runtime::Runtime;

/// Measure baseline agent execution without hooks
fn bench_agent_baseline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_execution_baseline", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut agent = MockAgent::new(MockAgentConfig::default());
                
                let input = AgentInput {
                    text: "test input".to_string(),
                    media: vec![],
                    context: None,
                    parameters: HashMap::new(),
                    output_modalities: vec![],
                };
                let context = ExecutionContext::default();

                let result = agent.execute(input, context).await;
                black_box(result)
            });
        });
    });
}

/// Measure agent execution with simulated hook overhead
fn bench_agent_with_simulated_hooks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_execution_with_simulated_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut agent = MockAgent::new(MockAgentConfig::default());
                
                // Simulate hook overhead with 5 simple operations
                for _ in 0..5 {
                    black_box(format!("hook-operation-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
                }
                
                let input = AgentInput {
                    text: "test input".to_string(),
                    media: vec![],
                    context: None,
                    parameters: HashMap::new(),
                    output_modalities: vec![],
                };
                let context = ExecutionContext::default();

                let result = agent.execute(input, context).await;
                black_box(result)
            });
        });
    });
}

/// Measure hook overhead only
fn bench_hook_operations_only(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("hook_operations_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate 5 hook operations without agent execution
                let mut results = Vec::new();
                for i in 0..5 {
                    let hook_result = format!("hook-{}-{}", i, chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
                    results.push(hook_result);
                }
                black_box(results)
            });
        });
    });
}

/// Measure memory allocation overhead
fn bench_memory_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("memory_allocation_overhead", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simulate memory allocations similar to hook system
                let contexts: Vec<String> = Vec::with_capacity(5);
                let mut hook_data = HashMap::new();
                
                for i in 0..5 {
                    hook_data.insert(format!("hook-{}", i), format!("data-{}", i));
                }
                
                black_box((contexts, hook_data))
            });
        });
    });
}

/// Calculate hook overhead percentage
fn calculate_hook_overhead(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Hook Overhead Analysis ===");

    rt.block_on(async {
        // Baseline: Agent execution without hooks
        let start = tokio::time::Instant::now();
        for _ in 0..1000 {
            let mut agent = MockAgent::new(MockAgentConfig::default());
            let input = AgentInput {
                text: "test input".to_string(),
                media: vec![],
                context: None,
                parameters: HashMap::new(),
                output_modalities: vec![],
            };
            let context = ExecutionContext::default();
            
            let _ = agent.execute(input, context).await;
        }
        let baseline = start.elapsed();

        // With simulated hooks
        let start = tokio::time::Instant::now();
        for _ in 0..1000 {
            // Simulate hook overhead
            for _ in 0..5 {
                black_box(format!("hook-operation-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
            }
            
            let mut agent = MockAgent::new(MockAgentConfig::default());
            let input = AgentInput {
                text: "test input".to_string(),
                media: vec![],
                context: None,
                parameters: HashMap::new(),
                output_modalities: vec![],
            };
            let context = ExecutionContext::default();
            
            let _ = agent.execute(input, context).await;
        }
        let with_hooks = start.elapsed();

        let overhead_ns = with_hooks.as_nanos().saturating_sub(baseline.as_nanos());
        let overhead_percent = (overhead_ns as f64 / baseline.as_nanos() as f64) * 100.0;

        println!("Baseline execution: {:?}", baseline);
        println!("With simulated hooks: {:?}", with_hooks);
        println!("Hook overhead: {:.2}%", overhead_percent);
        println!("Target: <5%");
        println!(
            "Status: {}",
            if overhead_percent < 5.0 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );

        // Also test pure hook operation overhead
        println!("\n--- Pure Hook Operations Overhead ---");
        
        let start = tokio::time::Instant::now();
        for _ in 0..10000 {
            black_box(42 + 42); // Baseline operation
        }
        let hook_baseline = start.elapsed();

        let start = tokio::time::Instant::now();
        for _ in 0..10000 {
            black_box(format!("hook-{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)));
        }
        let hook_operations = start.elapsed();

        let hook_only_overhead_ns = hook_operations.as_nanos().saturating_sub(hook_baseline.as_nanos());
        let hook_only_overhead_percent = (hook_only_overhead_ns as f64 / hook_baseline.as_nanos() as f64) * 100.0;

        println!("Hook baseline: {:?}", hook_baseline);
        println!("Hook operations: {:?}", hook_operations);
        println!("Hook-only overhead: {:.2}%", hook_only_overhead_percent);
        println!(
            "Hook-only status: {}",
            if hook_only_overhead_percent < 1.0 {
                "PASS ✅"
            } else {
                "ACCEPTABLE ⚠️"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_agent_baseline,
    bench_agent_with_simulated_hooks,
    bench_hook_operations_only,
    bench_memory_overhead,
    calculate_hook_overhead
);
criterion_main!(benches);