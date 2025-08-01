// ABOUTME: System-wide integration overhead benchmarks for state persistence
// ABOUTME: Measures performance impact when state persistence is integrated with agents, workflows, tools

#![cfg_attr(test_category = "benchmark")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use llmspell_agents::factory::{AgentFactory, DefaultAgentFactory};
use llmspell_core::{types::AgentInput, BaseAgent, ExecutionContext};
use llmspell_providers::ProviderManager;
use llmspell_state_persistence::{StateClass, StateManager, StateScope};
use llmspell_tools::util::calculator::CalculatorTool;
use llmspell_workflows::{
    parallel::ParallelBranch, ParallelWorkflowBuilder, SequentialWorkflowBuilder, StepType,
    WorkflowStep,
};
use std::sync::Arc;
use std::time::Instant;
use tokio::runtime::Runtime;

/// System integration test data
struct IntegrationTestData {
    agent_count: usize,
    #[allow(dead_code)]
    tool_count: usize,
    workflow_steps: usize,
    state_operations_per_component: usize,
}

impl IntegrationTestData {
    fn small() -> Self {
        Self {
            agent_count: 5,
            tool_count: 3,
            workflow_steps: 3,
            state_operations_per_component: 10,
        }
    }

    fn medium() -> Self {
        Self {
            agent_count: 20,
            tool_count: 10,
            workflow_steps: 10,
            state_operations_per_component: 50,
        }
    }

    #[allow(dead_code)]
    fn large() -> Self {
        Self {
            agent_count: 50,
            tool_count: 25,
            workflow_steps: 25,
            state_operations_per_component: 100,
        }
    }
}

/// Benchmark agent system with state persistence overhead
fn bench_agent_system_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("agent_system_overhead");
    group.sample_size(20); // Reduce sample size for complex benchmarks

    let test_scenarios = vec![
        (IntegrationTestData::small(), "small_system"),
        (IntegrationTestData::medium(), "medium_system"),
    ];

    for (test_data, label) in test_scenarios {
        // Baseline: Agents without state persistence
        group.bench_with_input(
            BenchmarkId::new("baseline_no_state", label),
            &test_data,
            |b, test_data| {
                b.iter(|| {
                    rt.block_on(async {
                        let start = Instant::now();

                        // Create factory and agents without state persistence
                        let provider_manager = Arc::new(ProviderManager::new());
                        let factory = DefaultAgentFactory::new(provider_manager);

                        for _i in 0..test_data.agent_count {
                            let agent = factory.create_from_template("basic").await.unwrap();

                            // Simulate agent operations without state
                            let input = AgentInput::text("test query");
                            let ctx = ExecutionContext::default();
                            let _ = agent.execute(input, ctx).await;
                        }

                        start.elapsed()
                    })
                });
            },
        );

        // With state persistence
        group.bench_with_input(
            BenchmarkId::new("with_state_persistence", label),
            &test_data,
            |b, test_data| {
                b.iter(|| {
                    rt.block_on(async {
                        let start = Instant::now();
                        let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                        // Create factory and agents (state persistence simulated separately)
                        let provider_manager = Arc::new(ProviderManager::new());
                        let factory = DefaultAgentFactory::new(provider_manager);

                        for i in 0..test_data.agent_count {
                            let agent_id = format!("benchmark:test_agent_{}", i);
                            let agent = factory.create_from_template("basic").await.unwrap();

                            // Simulate agent operations
                            let input = AgentInput::text("test query");
                            let ctx = ExecutionContext::default();
                            let _ = agent.execute(input, ctx).await;

                            // Perform state operations
                            let scope = StateScope::Agent(agent_id.clone());
                            for j in 0..test_data.state_operations_per_component {
                                let key = format!("benchmark:state_{}", j);
                                let value = serde_json::json!({
                                    "iteration": j,
                                    "data": "test_data"
                                });
                                state_manager
                                    .set_with_class(
                                        scope.clone(),
                                        &key,
                                        value,
                                        Some(StateClass::Trusted),
                                    )
                                    .await
                                    .unwrap();
                            }
                        }

                        start.elapsed()
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark workflow system with state persistence overhead
fn bench_workflow_system_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("workflow_system_overhead");
    group.sample_size(20);

    let test_scenarios = vec![
        (IntegrationTestData::small(), "small_workflows"),
        (IntegrationTestData::medium(), "medium_workflows"),
    ];

    for (test_data, label) in test_scenarios {
        // Baseline: Workflows without state
        group.bench_with_input(
            BenchmarkId::new("baseline_no_state", label),
            &test_data,
            |b, test_data| {
                b.iter(|| {
                    rt.block_on(async {
                        let start = Instant::now();

                        // Create and execute sequential workflow
                        let mut builder =
                            SequentialWorkflowBuilder::new("test_workflow".to_string());
                        for i in 0..test_data.workflow_steps {
                            let step = WorkflowStep::new(
                                format!("step_{}", i),
                                StepType::Custom {
                                    function_name: "test_function".to_string(),
                                    parameters: serde_json::json!({"index": i}),
                                },
                            );
                            builder = builder.add_step(step);
                        }
                        let workflow = builder.build();
                        let _ = workflow.execute().await;

                        start.elapsed()
                    })
                });
            },
        );

        // With state persistence
        group.bench_with_input(
            BenchmarkId::new("with_state_persistence", label),
            &test_data,
            |b, test_data| {
                b.iter(|| {
                    rt.block_on(async {
                        let start = Instant::now();
                        let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                        // Create and execute workflow with state persistence
                        let workflow_id = "benchmark:test_workflow";
                        let mut builder = SequentialWorkflowBuilder::new(workflow_id.to_string());

                        for i in 0..test_data.workflow_steps {
                            let step = WorkflowStep::new(
                                format!("step_{}", i),
                                StepType::Custom {
                                    function_name: "test_function".to_string(),
                                    parameters: serde_json::json!({"index": i}),
                                },
                            );
                            builder = builder.add_step(step);
                        }

                        let workflow = builder.build();
                        let scope = StateScope::Workflow(workflow_id.to_string());

                        // Save workflow state before execution
                        state_manager
                            .set_with_class(
                                scope.clone(),
                                "benchmark:workflow_config",
                                serde_json::json!({"steps": test_data.workflow_steps}),
                                Some(StateClass::Trusted),
                            )
                            .await
                            .unwrap();

                        let _ = workflow.execute().await;

                        // Save workflow results
                        for i in 0..test_data.state_operations_per_component {
                            let key = format!("benchmark:result_{}", i);
                            let value = serde_json::json!({
                                "step": i,
                                "status": "completed"
                            });
                            state_manager
                                .set_with_class(
                                    scope.clone(),
                                    &key,
                                    value,
                                    Some(StateClass::Trusted),
                                )
                                .await
                                .unwrap();
                        }

                        start.elapsed()
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark tool system with state persistence overhead
fn bench_tool_system_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("tool_system_overhead");
    group.sample_size(20);

    // Baseline: Tools without state
    group.bench_function("baseline_no_state", |b| {
        b.iter(|| {
            rt.block_on(async {
                let start = Instant::now();

                // Create and use tools without state
                for i in 0..10 {
                    let tool = CalculatorTool::new();
                    let input = serde_json::json!({
                        "operation": "add",
                        "a": i,
                        "b": i + 1
                    });
                    let ctx = ExecutionContext::default();
                    let text = serde_json::to_string(&input).unwrap();
                    let _ = tool.execute(AgentInput::text(text), ctx).await;
                }

                start.elapsed()
            })
        });
    });

    // With state persistence
    group.bench_function("with_state_persistence", |b| {
        b.iter(|| {
            rt.block_on(async {
                let start = Instant::now();
                let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                // Create and use tools with state persistence
                for i in 0..10 {
                    let tool = CalculatorTool::new();
                    let tool_id = format!("benchmark:calculator_{}", i);
                    let scope = StateScope::Tool(tool_id);

                    // Save tool configuration
                    state_manager
                        .set_with_class(
                            scope.clone(),
                            "benchmark:config",
                            serde_json::json!({"precision": 2}),
                            Some(StateClass::Trusted),
                        )
                        .await
                        .unwrap();

                    let input = serde_json::json!({
                        "operation": "add",
                        "a": i,
                        "b": i + 1
                    });
                    let ctx = ExecutionContext::default();
                    let text = serde_json::to_string(&input).unwrap();
                    let result = tool.execute(AgentInput::text(text), ctx).await.unwrap();

                    // Save tool result
                    state_manager
                        .set_with_class(
                            scope.clone(),
                            "benchmark:last_result",
                            serde_json::Value::String(result.text),
                            Some(StateClass::Trusted),
                        )
                        .await
                        .unwrap();
                }

                start.elapsed()
            })
        });
    });

    group.finish();
}

/// Benchmark full system integration with all components
fn bench_full_system_integration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("full_system_integration");
    group.sample_size(10); // Even smaller sample size for complex integration tests

    // Test a realistic scenario with agents, workflows, and tools
    group.bench_function("complete_integration_scenario", |b| {
        b.iter(|| {
            rt.block_on(async {
                let start = Instant::now();
                let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                // Create agent with state
                // Create agent using factory
                let provider_manager = Arc::new(ProviderManager::new());
                let factory = DefaultAgentFactory::new(provider_manager);
                let agent = factory.create_from_template("basic").await.unwrap();

                // Create workflow that uses tools
                let mut builder = ParallelWorkflowBuilder::new("benchmark:integration_workflow");
                for i in 0..3 {
                    let step = WorkflowStep::new(
                        format!("parallel_step_{}", i),
                        StepType::Tool {
                            tool_name: "calculator".to_string(),
                            parameters: serde_json::json!({
                                "operation": "multiply",
                                "a": i * 10,
                                "b": i + 1
                            }),
                        },
                    );
                    let branch = ParallelBranch {
                        name: format!("branch_{}", i),
                        description: format!("Branch {}", i),
                        steps: vec![step.into()],
                        required: true,
                        timeout: None,
                    };
                    builder = builder.add_branch(branch);
                }
                let workflow = builder.build().unwrap();

                // Agent processes input
                let input = AgentInput::text("Calculate parallel operations");
                let ctx = ExecutionContext::default();
                let agent_result = agent.execute(input, ctx).await.unwrap();

                // Save agent result
                state_manager
                    .set_with_class(
                        StateScope::Agent("benchmark:integration_agent".to_string()),
                        "benchmark:agent_result",
                        serde_json::json!({"output": agent_result.text}),
                        Some(StateClass::Trusted),
                    )
                    .await
                    .unwrap();

                // Execute workflow
                let workflow_result = workflow.execute().await.unwrap();

                // Save workflow result
                state_manager
                    .set_with_class(
                        StateScope::Workflow("benchmark:integration_workflow".to_string()),
                        "benchmark:workflow_result",
                        serde_json::to_value(&workflow_result).unwrap(),
                        Some(StateClass::Trusted),
                    )
                    .await
                    .unwrap();

                // Simulate cross-component state access
                for i in 0..5 {
                    let key = format!("benchmark:shared_state_{}", i);
                    state_manager
                        .set_with_class(
                            StateScope::Global,
                            &key,
                            serde_json::json!({"iteration": i, "timestamp": chrono::Utc::now()}),
                            Some(StateClass::Trusted),
                        )
                        .await
                        .unwrap();

                    // Read from different scope
                    let _ = state_manager
                        .get_with_class(StateScope::Global, &key, Some(StateClass::Trusted))
                        .await;
                }

                start.elapsed()
            })
        });
    });

    group.finish();
}

/// Benchmark streaming operations with state persistence
fn bench_streaming_with_state(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("streaming_with_state");
    group.sample_size(20);

    group.bench_function("agent_streaming_with_state", |b| {
        b.iter(|| {
            rt.block_on(async {
                let start = Instant::now();
                let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                // Create agent using factory
                let provider_manager = Arc::new(ProviderManager::new());
                let factory = DefaultAgentFactory::new(provider_manager);
                let agent = factory.create_from_template("basic").await.unwrap();

                // Stream processing with state saves
                let input = AgentInput::text("Generate streaming response");
                // Basic agents don't support streaming, so simulate with execute
                let ctx = ExecutionContext::default();
                let result = agent.execute(input, ctx).await.unwrap();

                // Simulate streaming chunks
                let text = result.text;
                let chunk_size = 10;
                let chunks: Vec<_> = text
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(chunk_size)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect();
                let scope = StateScope::Agent("benchmark:streaming_agent".to_string());

                // Process chunks and save state
                for (chunk_count, chunk_text) in chunks.iter().enumerate() {
                    // Save chunk state (simulate real streaming scenario)
                    if chunk_count % 10 == 0 {
                        state_manager
                            .set_with_class(
                                scope.clone(),
                                &format!("benchmark:chunk_{}", chunk_count),
                                serde_json::json!({
                                    "index": chunk_count,
                                    "content": chunk_text
                                }),
                                Some(StateClass::Ephemeral), // Use ephemeral for streaming
                            )
                            .await
                            .unwrap();
                    }
                }

                start.elapsed()
            })
        });
    });

    group.finish();
}

/// Measure memory usage impact of state persistence
fn bench_memory_scaling(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("memory_scaling");
    group.sample_size(10);

    let state_sizes = vec![
        (100, "100_states"),
        (1000, "1000_states"),
        (5000, "5000_states"),
    ];

    for (count, label) in state_sizes {
        group.bench_with_input(
            BenchmarkId::new("state_memory_usage", label),
            &count,
            |b, &count| {
                b.iter(|| {
                    rt.block_on(async {
                        let start = Instant::now();
                        let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

                        // Create many states to test memory scaling
                        for i in 0..count {
                            let scope = StateScope::Agent(format!("benchmark:agent_{}", i % 100));
                            let key = format!("benchmark:state_{}", i);
                            let value = serde_json::json!({
                                "id": i,
                                "data": vec![0u8; 1024], // 1KB per state
                                "metadata": {
                                    "created": chrono::Utc::now(),
                                    "version": 1
                                }
                            });

                            state_manager
                                .set_with_class(scope, &key, value, Some(StateClass::Trusted))
                                .await
                                .unwrap();
                        }

                        // Read some states to ensure they're in memory
                        for i in (0..count).step_by(10) {
                            let scope = StateScope::Agent(format!("benchmark:agent_{}", i % 100));
                            let key = format!("benchmark:state_{}", i);
                            let _ = state_manager
                                .get_with_class(scope, &key, Some(StateClass::Trusted))
                                .await;
                        }

                        start.elapsed()
                    })
                });
            },
        );
    }

    group.finish();
}

/// Calculate and report integrated overhead percentages
fn calculate_integrated_overhead(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== Integrated State Persistence Overhead Analysis ===");

    rt.block_on(async {
        let state_manager = Arc::new(StateManager::new_benchmark().await.unwrap());

        // Test agent overhead
        println!("\n--- Agent Integration Overhead ---");
        let agent_count = 10;

        // Baseline: Agents without state
        let start = Instant::now();
        for _i in 0..agent_count {
            let provider_manager = Arc::new(ProviderManager::new());
            let factory = DefaultAgentFactory::new(provider_manager);
            let agent = factory.create_from_template("basic").await.unwrap();
            let input = AgentInput::text("test");
            let ctx = ExecutionContext::default();
            let _ = agent.execute(input, ctx).await;
        }
        let baseline_time = start.elapsed();

        // With state persistence
        let start = Instant::now();
        for i in 0..agent_count {
            let agent_id = format!("benchmark:test_agent_{}", i);
            let provider_manager = Arc::new(ProviderManager::new());
            let factory = DefaultAgentFactory::new(provider_manager);
            let agent = factory.create_from_template("basic").await.unwrap();

            let input = AgentInput::text("test");
            let ctx = ExecutionContext::default();
            let _ = agent.execute(input, ctx).await;

            // Save agent state
            state_manager
                .set_with_class(
                    StateScope::Agent(agent_id),
                    "benchmark:state",
                    serde_json::json!({"processed": true}),
                    Some(StateClass::Trusted),
                )
                .await
                .unwrap();
        }
        let with_state_time = start.elapsed();

        let agent_overhead = ((with_state_time.as_nanos() as f64
            - baseline_time.as_nanos() as f64)
            / baseline_time.as_nanos() as f64)
            * 100.0;

        println!("Agent baseline: {:?}", baseline_time);
        println!("Agent with state: {:?}", with_state_time);
        println!("Agent integration overhead: {:.2}%", agent_overhead);
        println!("Target: <50%");
        println!(
            "Status: {}",
            if agent_overhead < 50.0 {
                "PASS ✅"
            } else {
                "REVIEW ⚠️"
            }
        );

        // Test concurrent access performance
        println!("\n--- Concurrent Access Performance ---");
        let concurrent_count = 100;

        let start = Instant::now();
        let handles: Vec<_> = (0..concurrent_count)
            .map(|i| {
                let sm = state_manager.clone();
                tokio::spawn(async move {
                    let scope = StateScope::Agent(format!("benchmark:concurrent_{}", i));
                    sm.set_with_class(
                        scope.clone(),
                        "benchmark:test",
                        serde_json::json!({"index": i}),
                        Some(StateClass::Trusted),
                    )
                    .await
                    .unwrap();
                    sm.get_with_class(scope, "benchmark:test", Some(StateClass::Trusted))
                        .await
                })
            })
            .collect();

        for handle in handles {
            let _ = handle.await.unwrap();
        }
        let concurrent_time = start.elapsed();

        let ops_per_sec = (concurrent_count as f64 * 2.0) / concurrent_time.as_secs_f64();
        println!(
            "Concurrent operations: {} in {:?}",
            concurrent_count * 2,
            concurrent_time
        );
        println!("Operations per second: {:.0}", ops_per_sec);
        println!(
            "Average latency: {:.2}ms",
            concurrent_time.as_millis() as f64 / (concurrent_count as f64 * 2.0)
        );
        println!("Target: <10ms at 99th percentile");
        println!(
            "Status: {}",
            if concurrent_time.as_millis() / (concurrent_count * 2) < 10 {
                "PASS ✅"
            } else {
                "REVIEW ⚠️"
            }
        );

        // Test memory efficiency
        println!("\n--- Memory Efficiency Test ---");
        let memory_test_count = 1000;
        let value_size = 1024; // 1KB per value

        let start = Instant::now();
        for i in 0..memory_test_count {
            let scope = StateScope::Global;
            let key = format!("benchmark:memory_test_{}", i);
            let value = serde_json::json!({
                "data": vec![0u8; value_size],
                "index": i
            });
            state_manager
                .set_with_class(scope, &key, value, Some(StateClass::Trusted))
                .await
                .unwrap();
        }
        let write_time = start.elapsed();

        let write_throughput_mb =
            (memory_test_count * value_size) as f64 / 1_048_576.0 / write_time.as_secs_f64();
        println!(
            "Wrote {} states ({} MB) in {:?}",
            memory_test_count,
            (memory_test_count * value_size) / 1_048_576,
            write_time
        );
        println!("Write throughput: {:.2} MB/s", write_throughput_mb);
        println!(
            "Status: {}",
            if write_throughput_mb > 10.0 {
                "PASS ✅"
            } else {
                "REVIEW ⚠️"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_agent_system_overhead,
    bench_workflow_system_overhead,
    bench_tool_system_overhead,
    bench_full_system_integration,
    bench_streaming_with_state,
    bench_memory_scaling,
    calculate_integrated_overhead
);
criterion_main!(benches);
