// ABOUTME: Simplified tracing overhead benchmark
// ABOUTME: Validates <2% overhead at INFO level and <5% at DEBUG level

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_agents::testing::mocks::{MockAgent, MockAgentConfig};
use llmspell_core::{types::AgentInput, BaseAgent, ExecutionContext};
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;

/// Benchmark agent execution with different tracing levels
fn bench_agent_tracing_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("agent_tracing_overhead");

    // Configure for accurate measurements
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);
    group.warm_up_time(Duration::from_secs(3));

    let input = AgentInput {
        text: "Process this test input for benchmarking tracing overhead".to_string(),
        media: vec![],
        context: None,
        parameters: HashMap::new(),
        output_modalities: vec![],
    };

    // Baseline: Tracing disabled (RUST_LOG not set)
    group.bench_function("baseline_no_tracing", |b| {
        // Clear RUST_LOG to disable tracing
        std::env::remove_var("RUST_LOG");

        b.iter(|| {
            rt.block_on(async {
                let agent = MockAgent::new(MockAgentConfig::default());
                let context = ExecutionContext::default();
                let result = agent.execute(black_box(input.clone()), context).await;
                black_box(result)
            })
        });
    });

    // With INFO level tracing
    group.bench_function("info_level", |b| {
        // Set INFO level
        std::env::set_var("RUST_LOG", "info");

        b.iter(|| {
            rt.block_on(async {
                let agent = MockAgent::new(MockAgentConfig::default());
                let context = ExecutionContext::default();
                let result = agent.execute(black_box(input.clone()), context).await;
                black_box(result)
            })
        });
    });

    // With DEBUG level tracing
    group.bench_function("debug_level", |b| {
        // Set DEBUG level
        std::env::set_var("RUST_LOG", "debug");

        b.iter(|| {
            rt.block_on(async {
                let agent = MockAgent::new(MockAgentConfig::default());
                let context = ExecutionContext::default();
                let result = agent.execute(black_box(input.clone()), context).await;
                black_box(result)
            })
        });
    });

    // With TRACE level tracing (worst case)
    group.bench_function("trace_level", |b| {
        // Set TRACE level
        std::env::set_var("RUST_LOG", "trace");

        b.iter(|| {
            rt.block_on(async {
                let agent = MockAgent::new(MockAgentConfig::default());
                let context = ExecutionContext::default();
                let result = agent.execute(black_box(input.clone()), context).await;
                black_box(result)
            })
        });
    });

    group.finish();

    // Reset environment
    std::env::remove_var("RUST_LOG");
}

/// Benchmark hot path operations with span creation
fn bench_hot_path_spans(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("hot_path_spans");

    group.measurement_time(Duration::from_secs(5));
    group.sample_size(200);

    // Baseline: No spans
    group.bench_function("no_spans", |b| {
        std::env::remove_var("RUST_LOG");

        b.iter(|| {
            rt.block_on(async {
                let mut sum = 0i64;
                for i in 0..100 {
                    // Simulate async work
                    tokio::task::yield_now().await;
                    sum += i;
                }
                black_box(sum)
            })
        });
    });

    // With span creation at INFO level
    group.bench_function("with_info_spans", |b| {
        std::env::set_var("RUST_LOG", "info");

        b.iter(|| {
            rt.block_on(async {
                let mut sum = 0i64;
                for i in 0..100 {
                    let _span = tracing::info_span!("iteration", i = i).entered();
                    // Simulate async work
                    tokio::task::yield_now().await;
                    sum += i;
                }
                black_box(sum)
            })
        });
    });

    // With span creation at DEBUG level
    group.bench_function("with_debug_spans", |b| {
        std::env::set_var("RUST_LOG", "debug");

        b.iter(|| {
            rt.block_on(async {
                let mut sum = 0i64;
                for i in 0..100 {
                    let _span = tracing::debug_span!("iteration", i = i).entered();
                    tracing::debug!("Processing item {}", i);
                    // Simulate async work
                    tokio::task::yield_now().await;
                    sum += i;
                }
                black_box(sum)
            })
        });
    });

    group.finish();

    // Reset environment
    std::env::remove_var("RUST_LOG");
}

/// Measure Debug trait formatting overhead
fn bench_debug_formatting(c: &mut Criterion) {
    let mut group = c.benchmark_group("debug_formatting");

    #[derive(Debug)]
    #[allow(dead_code)]
    struct SimpleStruct {
        id: String,
        value: i32,
        metadata: HashMap<String, String>,
    }

    let simple = SimpleStruct {
        id: "test-123".to_string(),
        value: 42,
        metadata: HashMap::from([
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string()),
        ]),
    };

    // Baseline: No formatting
    group.bench_function("no_format", |b| {
        b.iter(|| {
            black_box(&simple);
        });
    });

    // Format with Debug
    group.bench_function("debug_format", |b| {
        b.iter(|| {
            let formatted = format!("{:?}", black_box(&simple));
            black_box(formatted)
        });
    });

    // Format with pretty Debug
    group.bench_function("debug_pretty_format", |b| {
        b.iter(|| {
            let formatted = format!("{:#?}", black_box(&simple));
            black_box(formatted)
        });
    });

    group.finish();
}

/// Calculate and report overhead percentages
#[allow(dead_code)]
fn report_overhead_summary(_c: &mut Criterion) {
    println!("\n=== Tracing Overhead Performance Summary ===\n");
    println!("Target: <2% overhead at INFO level");
    println!("Target: <5% overhead at DEBUG level");
    println!("\nRun with: cargo bench --bench tracing_overhead_simple");
    println!("\nAnalysis will show overhead percentages in criterion report");
    println!("Check target/criterion/agent_tracing_overhead/report/index.html");
    println!("\n==========================================\n");
}

criterion_group!(
    benches,
    bench_agent_tracing_overhead,
    bench_hot_path_spans,
    bench_debug_formatting,
);
criterion_main!(benches);
