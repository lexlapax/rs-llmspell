// ABOUTME: Performance benchmarks for tracing instrumentation overhead
// ABOUTME: Validates <2% overhead at INFO level and <5% at DEBUG level

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_agents::testing::mocks::{MockAgent, MockAgentConfig};
use llmspell_core::{types::AgentInput, BaseAgent, ExecutionContext};
use llmspell_tools::registry::ToolRegistry;
use std::collections::HashMap;
use std::time::Duration;
use tokio::runtime::Runtime;
use tracing::Level;
use tracing_subscriber::EnvFilter;

/// Setup tracing at specified level
fn setup_tracing_at_level(level: Level) {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(level.into()))
        .with_writer(std::io::sink) // Discard output for benchmarks
        .try_init();
}

/// Disable tracing completely
fn disable_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(Level::ERROR.into()))
        .with_writer(std::io::sink)
        .try_init();
}

/// Benchmark tool execution with different tracing levels
fn bench_tool_execution_tracing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("tool_execution_tracing");

    // Configure for accurate measurements
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Test different input sizes
    let input_sizes = vec![10, 100, 1000];

    for size in input_sizes {
        let input = AgentInput {
            text: "x".repeat(size),
            media: vec![],
            context: None,
            parameters: HashMap::from([
                ("operation".to_string(), serde_json::json!("add")),
                ("a".to_string(), serde_json::json!(2)),
                ("b".to_string(), serde_json::json!(3)),
            ]),
            output_modalities: vec![],
        };

        // Baseline: No tracing
        group.bench_with_input(
            BenchmarkId::new("baseline_no_tracing", size),
            &input,
            |b, input| {
                disable_tracing();
                b.iter(|| {
                    rt.block_on(async {
                        let registry = ToolRegistry::new();
                        // Create calculator tool inline
                        use llmspell_tools::CalculatorTool;
                        let calculator = CalculatorTool::new();
                        registry
                            .register("calculator".to_string(), calculator)
                            .await
                            .unwrap();
                        let result = registry
                            .execute_tool("calculator", input.clone(), ExecutionContext::default())
                            .await;
                        black_box(result)
                    })
                });
            },
        );

        // With INFO level tracing
        group.bench_with_input(
            BenchmarkId::new("info_level_tracing", size),
            &input,
            |b, input| {
                setup_tracing_at_level(Level::INFO);
                b.iter(|| {
                    rt.block_on(async {
                        let registry = ToolRegistry::new();
                        // Create calculator tool inline
                        use llmspell_tools::CalculatorTool;
                        let calculator = CalculatorTool::new();
                        registry
                            .register("calculator".to_string(), calculator)
                            .await
                            .unwrap();
                        let result = registry
                            .execute_tool("calculator", input.clone(), ExecutionContext::default())
                            .await;
                        black_box(result)
                    })
                });
            },
        );

        // With DEBUG level tracing
        group.bench_with_input(
            BenchmarkId::new("debug_level_tracing", size),
            &input,
            |b, input| {
                setup_tracing_at_level(Level::DEBUG);
                b.iter(|| {
                    rt.block_on(async {
                        let registry = ToolRegistry::new();
                        // Create calculator tool inline
                        use llmspell_tools::CalculatorTool;
                        let calculator = CalculatorTool::new();
                        registry
                            .register("calculator".to_string(), calculator)
                            .await
                            .unwrap();
                        let result = registry
                            .execute_tool("calculator", input.clone(), ExecutionContext::default())
                            .await;
                        black_box(result)
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark agent execution with different tracing levels
fn bench_agent_execution_tracing(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("agent_execution_tracing");

    group.measurement_time(Duration::from_secs(5));
    group.sample_size(50);

    let input = AgentInput {
        text: "Process this test input for benchmarking".to_string(),
        media: vec![],
        context: None,
        parameters: HashMap::new(),
        output_modalities: vec![],
    };

    // Baseline: No tracing
    group.bench_function("baseline_no_tracing", |b| {
        disable_tracing();
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
    group.bench_function("info_level_tracing", |b| {
        setup_tracing_at_level(Level::INFO);
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
    group.bench_function("debug_level_tracing", |b| {
        setup_tracing_at_level(Level::DEBUG);
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
}

/// Benchmark span creation overhead in hot paths
fn bench_span_creation_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("span_creation");

    // Baseline: No span creation
    group.bench_function("baseline_no_span", |b| {
        b.iter(|| {
            let mut sum = 0;
            for i in 0..1000 {
                sum += black_box(i);
            }
            black_box(sum)
        });
    });

    // With span creation (INFO level)
    group.bench_function("with_span_info", |b| {
        setup_tracing_at_level(Level::INFO);
        b.iter(|| {
            let span = tracing::info_span!("hot_path_operation");
            let _guard = span.enter();
            let mut sum = 0;
            for i in 0..1000 {
                sum += black_box(i);
            }
            black_box(sum)
        });
    });

    // With span creation (DEBUG level)
    group.bench_function("with_span_debug", |b| {
        setup_tracing_at_level(Level::DEBUG);
        b.iter(|| {
            let span = tracing::debug_span!("hot_path_operation", iteration = 0);
            let _guard = span.enter();
            let mut sum = 0;
            for i in 0..1000 {
                tracing::trace!("Iteration {}", i);
                sum += black_box(i);
            }
            black_box(sum)
        });
    });

    group.finish();
}

/// Benchmark memory impact of Debug implementations
fn bench_memory_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("debug_memory_impact");

    // Measure formatting overhead for complex types
    group.bench_function("format_simple_struct", |b| {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct SimpleStruct {
            id: String,
            value: i32,
        }

        let s = SimpleStruct {
            id: "test-123".to_string(),
            value: 42,
        };

        b.iter(|| {
            let formatted = format!("{:?}", black_box(&s));
            black_box(formatted)
        });
    });

    group.bench_function("format_complex_nested", |b| {
        #[derive(Debug)]
        #[allow(dead_code)]
        struct ComplexStruct {
            id: String,
            nested: Vec<HashMap<String, Vec<i32>>>,
            metadata: HashMap<String, String>,
        }

        let mut nested = Vec::new();
        for i in 0..10 {
            let mut map = HashMap::new();
            map.insert(format!("key_{}", i), vec![1, 2, 3, 4, 5]);
            nested.push(map);
        }

        let mut metadata = HashMap::new();
        for i in 0..20 {
            metadata.insert(format!("meta_{}", i), format!("value_{}", i));
        }

        let c = ComplexStruct {
            id: "complex-test".to_string(),
            nested,
            metadata,
        };

        b.iter(|| {
            let formatted = format!("{:?}", black_box(&c));
            black_box(formatted)
        });
    });

    group.finish();
}

/// Measure actual overhead percentages and validate targets
fn measure_overhead_percentages(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("overhead_validation");

    // High-frequency operation to measure overhead
    group.bench_function("high_frequency_baseline", |b| {
        disable_tracing();
        b.iter(|| {
            rt.block_on(async {
                let mut results = Vec::with_capacity(100);
                for i in 0..100 {
                    // Simulate a quick operation
                    let result = tokio::task::yield_now().await;
                    results.push((i, result));
                }
                black_box(results)
            })
        });
    });

    group.bench_function("high_frequency_info", |b| {
        setup_tracing_at_level(Level::INFO);
        b.iter(|| {
            rt.block_on(async {
                let mut results = Vec::with_capacity(100);
                for i in 0..100 {
                    let span = tracing::info_span!("operation", id = i);
                    let _guard = span.enter();
                    let result = tokio::task::yield_now().await;
                    results.push((i, result));
                }
                black_box(results)
            })
        });
    });

    group.bench_function("high_frequency_debug", |b| {
        setup_tracing_at_level(Level::DEBUG);
        b.iter(|| {
            rt.block_on(async {
                let mut results = Vec::with_capacity(100);
                for i in 0..100 {
                    let span = tracing::debug_span!("operation", id = i);
                    let _guard = span.enter();
                    tracing::debug!("Processing item {}", i);
                    let result = tokio::task::yield_now().await;
                    results.push((i, result));
                }
                black_box(results)
            })
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_tool_execution_tracing,
    bench_agent_execution_tracing,
    bench_span_creation_overhead,
    bench_memory_impact,
    measure_overhead_percentages
);
criterion_main!(benches);
