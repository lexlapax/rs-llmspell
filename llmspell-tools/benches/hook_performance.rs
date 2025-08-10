//! Performance benchmarks for tool hook integration
//! Measures overhead of hook system across various tools and scenarios

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_core::{traits::tool::Tool, types::AgentInput, ExecutionContext};
use llmspell_tools::{
    data::json_processor::{JsonProcessorConfig, JsonProcessorTool},
    fs::file_operations::FileOperationsTool,
    lifecycle::hook_integration::{HookFeatures, ToolExecutor, ToolLifecycleConfig},
    util::{
        calculator::CalculatorTool,
        hash_calculator::HashCalculatorTool,
        text_manipulator::{TextManipulatorConfig, TextManipulatorTool},
        uuid_generator::UuidGeneratorTool,
    },
};
use serde_json::json;
use std::time::Duration;

fn benchmark_calculator_without_hooks(c: &mut Criterion) {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            circuit_breaker_enabled: false,
            security_validation_enabled: false,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let inputs = vec![
        ("simple", json!({"operation": "evaluate", "input": "2 + 2"})),
        (
            "complex",
            json!({"operation": "evaluate", "input": "sin(pi()/2) + cos(0) * log(10, 100)"}),
        ),
        (
            "variables",
            json!({"operation": "evaluate", "input": "x^2 + y^2", "variables": {"x": 3, "y": 4}}),
        ),
    ];

    let mut group = c.benchmark_group("calculator_no_hooks");
    for (name, params) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &params, |b, params| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                runtime.block_on(async {
                    let input =
                        AgentInput::text("benchmark").with_parameter("parameters", params.clone());
                    let result = executor
                        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                        .await
                        .unwrap();
                    black_box(result);
                });
            });
        });
    }
    group.finish();
}

fn benchmark_calculator_with_hooks(c: &mut Criterion) {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            security_validation_enabled: true,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let inputs = vec![
        ("simple", json!({"operation": "evaluate", "input": "2 + 2"})),
        (
            "complex",
            json!({"operation": "evaluate", "input": "sin(pi()/2) + cos(0) * log(10, 100)"}),
        ),
        (
            "variables",
            json!({"operation": "evaluate", "input": "x^2 + y^2", "variables": {"x": 3, "y": 4}}),
        ),
    ];

    let mut group = c.benchmark_group("calculator_with_hooks");
    for (name, params) in inputs {
        group.bench_with_input(BenchmarkId::from_parameter(name), &params, |b, params| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                runtime.block_on(async {
                    let input =
                        AgentInput::text("benchmark").with_parameter("parameters", params.clone());
                    let result = executor
                        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                        .await
                        .unwrap();
                    black_box(result);
                });
            });
        });
    }
    group.finish();
}

fn benchmark_json_processor_hooks(c: &mut Criterion) {
    let config_no_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            circuit_breaker_enabled: false,
            security_validation_enabled: false,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor_no_hooks = ToolExecutor::new(config_no_hooks, None, None);

    let config_with_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            security_validation_enabled: true,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor_with_hooks = ToolExecutor::new(config_with_hooks, None, None);

    let json_tool = JsonProcessorTool::default();

    let test_data = json!({
        "users": [
            {"name": "Alice", "age": 30, "city": "NYC"},
            {"name": "Bob", "age": 25, "city": "LA"},
            {"name": "Charlie", "age": 35, "city": "Chicago"}
        ],
        "products": [
            {"id": 1, "name": "Laptop", "price": 999.99},
            {"id": 2, "name": "Phone", "price": 699.99}
        ]
    });

    let queries = vec![
        ("select_all", "."),
        ("filter_users", ".users | map(select(.age > 26))"),
        (
            "complex_transform",
            ".users | map({name, older: (.age > 30)}) | sort_by(.name)",
        ),
    ];

    let mut group = c.benchmark_group("json_processor");

    for (query_name, query) in queries {
        // Benchmark without hooks
        group.bench_with_input(
            BenchmarkId::new("no_hooks", query_name),
            &query,
            |b, query| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    runtime.block_on(async {
                        let input = AgentInput::text("benchmark").with_parameter(
                            "parameters",
                            json!({
                                "operation": "query",
                                "input": test_data.to_string(),
                                "query": query
                            }),
                        );
                        let result = executor_no_hooks
                            .execute_tool_with_hooks(&json_tool, input, ExecutionContext::default())
                            .await
                            .unwrap();
                        black_box(result);
                    });
                });
            },
        );

        // Benchmark with hooks
        group.bench_with_input(
            BenchmarkId::new("with_hooks", query_name),
            &query,
            |b, query| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    runtime.block_on(async {
                        let input = AgentInput::text("benchmark").with_parameter(
                            "parameters",
                            json!({
                                "operation": "query",
                                "input": test_data.to_string(),
                                "query": query
                            }),
                        );
                        let result = executor_with_hooks
                            .execute_tool_with_hooks(&json_tool, input, ExecutionContext::default())
                            .await
                            .unwrap();
                        black_box(result);
                    });
                });
            },
        );
    }
    group.finish();
}

fn benchmark_circuit_breaker(c: &mut Criterion) {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            security_validation_enabled: true,
        },
        circuit_breaker_failure_threshold: 3,
        circuit_breaker_recovery_time: Duration::from_millis(100),
        ..ToolLifecycleConfig::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    c.bench_function("circuit_breaker_healthy", |b| {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        b.iter(|| {
            runtime.block_on(async {
                // All valid expressions - circuit breaker should remain closed
                for i in 0..10 {
                    let input = AgentInput::text("benchmark").with_parameter(
                        "parameters",
                        json!({"operation": "evaluate", "input": format!("{} + {}", i, i)}),
                    );
                    let result = executor
                        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                        .await
                        .unwrap();
                    black_box(result);
                }
            });
        });
    });
}

fn benchmark_hook_overhead_comparison(c: &mut Criterion) {
    let tools: Vec<(&str, Box<dyn Tool>)> = vec![
        ("calculator", Box::new(CalculatorTool::new())),
        ("json_processor", Box::new(JsonProcessorTool::default())),
        ("file_operations", Box::new(FileOperationsTool::default())),
    ];

    let mut group = c.benchmark_group("hook_overhead_percentage");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    for (tool_name, tool) in tools {
        group.bench_function(BenchmarkId::new("overhead", tool_name), |b| {
            let runtime = tokio::runtime::Runtime::new().unwrap();

            // Prepare executors
            let config_no_hooks = ToolLifecycleConfig {
                features: HookFeatures {
                    hooks_enabled: false,
                    circuit_breaker_enabled: false,
                    security_validation_enabled: false,
                },
                ..ToolLifecycleConfig::default()
            };
            let executor_no_hooks = ToolExecutor::new(config_no_hooks, None, None);

            let config_with_hooks = ToolLifecycleConfig {
                features: HookFeatures {
                    hooks_enabled: true,
                    circuit_breaker_enabled: true,
                    security_validation_enabled: true,
                },
                ..ToolLifecycleConfig::default()
            };
            let executor_with_hooks = ToolExecutor::new(config_with_hooks, None, None);

            // Prepare appropriate input for each tool
            let input = match tool_name {
                "calculator" => AgentInput::text("benchmark").with_parameter(
                    "parameters",
                    json!({"operation": "evaluate", "input": "100 * 200 + 300"}),
                ),
                "json_processor" => AgentInput::text("benchmark").with_parameter(
                    "parameters",
                    json!({
                        "operation": "query",
                        "input": r#"{"value": 42}"#,
                        "query": ".value"
                    }),
                ),
                "file_operations" => AgentInput::text("benchmark").with_parameter(
                    "parameters",
                    json!({
                        "operation": "list",
                        "path": "/tmp"
                    }),
                ),
                _ => unreachable!(),
            };

            b.iter(|| {
                runtime.block_on(async {
                    // Measure without hooks
                    let start_no_hooks = std::time::Instant::now();
                    let _ = executor_no_hooks
                        .execute_tool_with_hooks(
                            tool.as_ref(),
                            input.clone(),
                            ExecutionContext::default(),
                        )
                        .await;
                    let duration_no_hooks = start_no_hooks.elapsed();

                    // Measure with hooks
                    let start_with_hooks = std::time::Instant::now();
                    let _ = executor_with_hooks
                        .execute_tool_with_hooks(
                            tool.as_ref(),
                            input.clone(),
                            ExecutionContext::default(),
                        )
                        .await;
                    let duration_with_hooks = start_with_hooks.elapsed();

                    // Calculate overhead
                    let with_hooks_micros =
                        u64::try_from(duration_with_hooks.as_micros()).unwrap_or(u64::MAX) as f64;
                    let no_hooks_micros =
                        u64::try_from(duration_no_hooks.as_micros()).unwrap_or(u64::MAX) as f64;
                    let overhead_micros = with_hooks_micros - no_hooks_micros;
                    let overhead_percent = (overhead_micros / no_hooks_micros) * 100.0;

                    black_box(overhead_percent);
                });
            });
        });
    }
    group.finish();
}

fn benchmark_hook_phases(c: &mut Criterion) {
    // This benchmark focuses on individual hook phase overhead
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            security_validation_enabled: true,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let mut group = c.benchmark_group("hook_phases");

    // Test different complexity levels to see how hooks scale
    let test_cases = vec![
        ("minimal", json!({"operation": "evaluate", "input": "1"})),
        ("simple", json!({"operation": "evaluate", "input": "2 + 2"})),
        (
            "medium",
            json!({"operation": "evaluate", "input": "x^2 + y^2", "variables": {"x": 3, "y": 4}}),
        ),
        (
            "complex",
            json!({
                "operation": "evaluate",
                "input": "sin(x) * cos(y) + log(10, z)",
                "variables": {"x": 1.57, "y": 0, "z": 100}
            }),
        ),
    ];

    for (name, params) in test_cases {
        group.bench_with_input(BenchmarkId::from_parameter(name), &params, |b, params| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.iter(|| {
                runtime.block_on(async {
                    let input =
                        AgentInput::text("benchmark").with_parameter("parameters", params.clone());
                    let result = executor
                        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                        .await
                        .unwrap();
                    black_box(result);
                });
            });
        });
    }
    group.finish();
}

fn benchmark_concurrent_hook_execution(c: &mut Criterion) {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            security_validation_enabled: true,
        },
        ..ToolLifecycleConfig::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let mut group = c.benchmark_group("concurrent_hooks");

    for num_concurrent in [1, 2, 4, 8, 16] {
        group.bench_with_input(
            BenchmarkId::from_parameter(num_concurrent),
            &num_concurrent,
            |b, &num_concurrent| {
                let runtime = tokio::runtime::Runtime::new().unwrap();
                b.iter(|| {
                    runtime.block_on(async {
                        let mut handles = vec![];

                        for i in 0..num_concurrent {
                            let executor_clone = executor.clone();
                            let calculator_clone = calculator.clone();

                            let handle = tokio::spawn(async move {
                                let input = AgentInput::text("concurrent").with_parameter(
                                    "parameters",
                                    json!({"operation": "evaluate", "input": format!("{} * {}", i, i)}),
                                );
                                executor_clone
                                    .execute_tool_with_hooks(&calculator_clone, input, ExecutionContext::default())
                                    .await
                                    .unwrap()
                            });
                            handles.push(handle);
                        }

                        for handle in handles {
                            black_box(handle.await.unwrap());
                        }
                    });
                });
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    benchmark_calculator_without_hooks,
    benchmark_calculator_with_hooks,
    benchmark_json_processor_hooks,
    benchmark_circuit_breaker,
    benchmark_hook_overhead_comparison,
    benchmark_hook_phases,
    benchmark_concurrent_hook_execution
);
criterion_main!(benches);
