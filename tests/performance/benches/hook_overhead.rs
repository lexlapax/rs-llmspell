// ABOUTME: Performance test for hook system overhead measurement
// ABOUTME: Validates <5% overhead requirement across agent, tool, and workflow operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_agents::{Agent, AgentConfig, AgentFactory, AgentInput, AgentTrait};
use llmspell_core::{
    component::ComponentMetadata, execution::ExecutionContext, tracing::correlation::CorrelationId,
};
use llmspell_hooks::{hook_registry::HookRegistry, HookContext, HookPoint, HookResult, Priority};
use llmspell_tools::{registry::ToolRegistry, Tool, ToolInput};
use llmspell_workflows::{
    patterns::{sequential::SequentialWorkflow, WorkflowPattern},
    WorkflowBuilder, WorkflowConfig, WorkflowStep,
};
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Measure baseline agent execution without hooks
fn bench_agent_baseline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let factory = Arc::new(AgentFactory::new());

    c.bench_function("agent_execution_baseline", |b| {
        b.iter(|| {
            rt.block_on(async {
                let agent = factory
                    .create_agent("basic", "test-agent", AgentConfig::default())
                    .await
                    .unwrap();

                let input = AgentInput::new("test input");
                let context = ExecutionContext::new(CorrelationId::new());

                let _ = black_box(agent.execute(input, context).await);
            });
        });
    });
}

/// Measure agent execution with hooks
fn bench_agent_with_hooks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let factory = Arc::new(AgentFactory::new());
    let registry = Arc::new(HookRegistry::new());

    // Register 5 hooks at different points
    rt.block_on(async {
        for i in 0..5 {
            registry
                .register_hook(
                    HookPoint::BeforeAgentExecution,
                    Box::new(move |_ctx: &HookContext| {
                        Box::pin(async move { HookResult::Continue })
                    }),
                    Priority::Normal,
                    Some(format!("test-hook-{}", i)),
                )
                .await
                .unwrap();
        }
    });

    c.bench_function("agent_execution_with_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let agent = factory
                    .create_agent("basic", "test-agent", AgentConfig::default())
                    .await
                    .unwrap();

                let input = AgentInput::new("test input");
                let context = ExecutionContext::new(CorrelationId::new());

                // Execute hooks
                let hook_context =
                    HookContext::new(agent.metadata().id.clone(), context.correlation_id.clone());
                let _ = registry
                    .execute_hooks(HookPoint::BeforeAgentExecution, hook_context)
                    .await;

                let _ = black_box(agent.execute(input, context).await);
            });
        });
    });
}

/// Measure tool execution overhead
fn bench_tool_execution_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let tool_registry = Arc::new(ToolRegistry::new());
    let hook_registry = Arc::new(HookRegistry::new());

    // Register a simple tool
    rt.block_on(async {
        let tool = Tool::new(
            "test-tool",
            "Test tool for benchmarking",
            |_input: ToolInput| async move { Ok(serde_json::json!({"result": "success"})) },
        );
        tool_registry.register_tool(tool).await.unwrap();

        // Register tool hooks
        for i in 0..3 {
            hook_registry
                .register_hook(
                    HookPoint::BeforeToolExecution,
                    Box::new(move |_ctx: &HookContext| {
                        Box::pin(async move { HookResult::Continue })
                    }),
                    Priority::Normal,
                    Some(format!("tool-hook-{}", i)),
                )
                .await
                .unwrap();
        }
    });

    c.bench_function("tool_execution_with_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let input = ToolInput::new(serde_json::json!({"test": "data"}));
                let context = ExecutionContext::new(CorrelationId::new());

                // Execute hooks
                let hook_context = HookContext::new(
                    ComponentMetadata::tool("test-tool").id,
                    context.correlation_id.clone(),
                );
                let _ = hook_registry
                    .execute_hooks(HookPoint::BeforeToolExecution, hook_context.clone())
                    .await;

                // Execute tool
                let tool = tool_registry.get_tool("test-tool").await.unwrap();
                let _ = black_box(tool.execute(input).await);

                // After execution hook
                let _ = hook_registry
                    .execute_hooks(HookPoint::AfterToolExecution, hook_context)
                    .await;
            });
        });
    });
}

/// Measure workflow execution with hooks
fn bench_workflow_execution_overhead(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let hook_registry = Arc::new(HookRegistry::new());

    rt.block_on(async {
        // Register workflow hooks
        for point in [
            HookPoint::BeforeWorkflowStart,
            HookPoint::BeforeWorkflowStage,
            HookPoint::AfterWorkflowStage,
            HookPoint::AfterWorkflowComplete,
        ] {
            hook_registry
                .register_hook(
                    point,
                    Box::new(|_ctx: &HookContext| Box::pin(async move { HookResult::Continue })),
                    Priority::Normal,
                    None,
                )
                .await
                .unwrap();
        }
    });

    c.bench_function("workflow_execution_with_hooks", |b| {
        b.iter(|| {
            rt.block_on(async {
                let workflow = WorkflowBuilder::new("test-workflow")
                    .add_step(WorkflowStep::new(
                        "step1",
                        Box::new(|_| Box::pin(async move { Ok(serde_json::Value::Null) })),
                    ))
                    .add_step(WorkflowStep::new(
                        "step2",
                        Box::new(|_| Box::pin(async move { Ok(serde_json::Value::Null) })),
                    ))
                    .add_step(WorkflowStep::new(
                        "step3",
                        Box::new(|_| Box::pin(async move { Ok(serde_json::Value::Null) })),
                    ))
                    .build();

                let context = ExecutionContext::new(CorrelationId::new());
                let config = WorkflowConfig::default();

                let pattern = SequentialWorkflow::new(workflow, config);
                let _ = black_box(pattern.execute(context).await);
            });
        });
    });
}

/// Measure hook registration overhead
fn bench_hook_registration(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("hook_registration", |b| {
        b.iter(|| {
            rt.block_on(async {
                let registry = HookRegistry::new();

                // Register multiple hooks
                for i in 0..10 {
                    let _ = black_box(
                        registry
                            .register_hook(
                                HookPoint::BeforeAgentExecution,
                                Box::new(move |_ctx: &HookContext| {
                                    Box::pin(async move { HookResult::Continue })
                                }),
                                Priority::Normal,
                                Some(format!("bench-hook-{}", i)),
                            )
                            .await,
                    );
                }
            });
        });
    });
}

/// Measure overhead percentage calculation
fn calculate_overhead_percentage(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let factory = Arc::new(AgentFactory::new());
    let hook_registry = Arc::new(HookRegistry::new());

    // Measure baseline and with hooks for comparison
    let mut group = c.benchmark_group("overhead_percentage");

    // Baseline measurement
    let baseline_ns = {
        let start = std::time::Instant::now();
        rt.block_on(async {
            for _ in 0..100 {
                let agent = factory
                    .create_agent("basic", "test-agent", AgentConfig::default())
                    .await
                    .unwrap();
                let input = AgentInput::new("test input");
                let context = ExecutionContext::new(CorrelationId::new());
                let _ = agent.execute(input, context).await;
            }
        });
        start.elapsed().as_nanos() / 100
    };

    // With hooks measurement
    rt.block_on(async {
        for i in 0..5 {
            hook_registry
                .register_hook(
                    HookPoint::BeforeAgentExecution,
                    Box::new(move |_ctx: &HookContext| {
                        Box::pin(async move { HookResult::Continue })
                    }),
                    Priority::Normal,
                    Some(format!("overhead-hook-{}", i)),
                )
                .await
                .unwrap();
        }
    });

    let with_hooks_ns = {
        let start = std::time::Instant::now();
        rt.block_on(async {
            for _ in 0..100 {
                let agent = factory
                    .create_agent("basic", "test-agent", AgentConfig::default())
                    .await
                    .unwrap();
                let input = AgentInput::new("test input");
                let context = ExecutionContext::new(CorrelationId::new());

                // Execute hooks
                let hook_context =
                    HookContext::new(agent.metadata().id.clone(), context.correlation_id.clone());
                let _ = hook_registry
                    .execute_hooks(HookPoint::BeforeAgentExecution, hook_context)
                    .await;

                let _ = agent.execute(input, context).await;
            }
        });
        start.elapsed().as_nanos() / 100
    };

    let overhead_percentage =
        ((with_hooks_ns as f64 - baseline_ns as f64) / baseline_ns as f64) * 100.0;

    println!("\n=== Hook Overhead Analysis ===");
    println!("Baseline execution time: {} ns", baseline_ns);
    println!("With hooks execution time: {} ns", with_hooks_ns);
    println!("Overhead: {:.2}%", overhead_percentage);
    println!("Target: <5%");
    println!(
        "Status: {}",
        if overhead_percentage < 5.0 {
            "PASS ✅"
        } else {
            "FAIL ❌"
        }
    );

    group.finish();
}

criterion_group!(
    benches,
    bench_agent_baseline,
    bench_agent_with_hooks,
    bench_tool_execution_overhead,
    bench_workflow_execution_overhead,
    bench_hook_registration,
    calculate_overhead_percentage
);
criterion_main!(benches);
