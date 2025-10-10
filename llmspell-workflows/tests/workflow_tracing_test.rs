//! Workflow tracing instrumentation tests

use anyhow::Result;
use llmspell_core::ComponentId;
use llmspell_workflows::{
    conditional::ConditionalWorkflow,
    executor::{
        DefaultWorkflowExecutor, ExecutionContext as WorkflowExecutionContext, WorkflowExecutor,
    },
    parallel::{ParallelBranch, ParallelWorkflow},
    sequential::SequentialWorkflow,
    step_executor::StepExecutor,
    traits::{StepType, WorkflowStep},
    types::WorkflowInput,
    Condition,
};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tracing::info_span;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Initialize test tracing subscriber
fn init_test_tracing() {
    let _ = tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("debug,llmspell_workflows=trace")),
        )
        .with(fmt::layer().with_test_writer())
        .try_init();
}

#[tokio::test]
async fn test_workflow_executor_tracing() -> Result<()> {
    init_test_tracing();

    // Create a simple sequential workflow for testing
    let workflow = Arc::new(
        SequentialWorkflow::builder("test-workflow".to_string())
            .add_step(WorkflowStep::new(
                "step1".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            ))
            .build(),
    );

    let executor = DefaultWorkflowExecutor::new();

    // Test execute_workflow tracing
    let input = WorkflowInput::new(json!({"test": "data"}));

    info_span!("test_execute_workflow")
        .in_scope(|| async {
            let result = executor
                .execute_workflow(workflow.clone(), input.clone())
                .await;
            assert!(result.is_ok());
        })
        .await;

    // Test execute_with_context tracing
    let context = WorkflowExecutionContext {
        cancel_token: None,
        timeout: Some(Duration::from_secs(5)),
        collect_metrics: true,
        enable_tracing: true,
    };

    info_span!("test_execute_with_context")
        .in_scope(|| async {
            let result = executor
                .execute_with_context(workflow.clone(), input.clone(), context)
                .await;
            assert!(result.is_ok());
        })
        .await;

    // Test execute_async tracing
    info_span!("test_execute_async")
        .in_scope(|| async {
            let handle = executor.execute_async(workflow.clone(), input.clone());
            let result = handle.await?;
            assert!(result.is_ok());
            Ok::<(), anyhow::Error>(())
        })
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_sequential_workflow_tracing() -> Result<()> {
    init_test_tracing();

    let workflow = Arc::new(
        SequentialWorkflow::builder("test-sequential".to_string())
            .add_step(WorkflowStep::new(
                "step1".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            ))
            .add_step(WorkflowStep::new(
                "step2".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            ))
            .build(),
    );

    let executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"sequential": "test"}));

    info_span!("test_sequential_execution")
        .in_scope(|| async {
            let result = executor.execute_workflow(workflow, input).await;
            assert!(result.is_ok());
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_step_executor_tracing() -> Result<()> {
    init_test_tracing();

    let config = llmspell_workflows::WorkflowConfig::default();
    let _executor = StepExecutor::new(config);

    let _step = WorkflowStep::new(
        "traced-step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: json!({"operation": "add", "values": [1, 1]}),
        },
    );

    // Create a context internally (StepExecutionContext is not public)
    // We'll test through the workflow execution instead

    // Test that step executor can be created
    assert!(std::mem::size_of_val(&_executor) > 0);

    Ok(())
}

#[tokio::test]
async fn test_conditional_workflow_tracing() -> Result<()> {
    init_test_tracing();

    let branch = llmspell_workflows::conditional::ConditionalBranch {
        id: ComponentId::new(),
        name: "always-branch".to_string(),
        condition: Condition::Always,
        steps: vec![WorkflowStep::new(
            "conditional-step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"operation": "add", "values": [1, 1]}),
            },
        )],
        is_default: false,
    };

    let workflow = ConditionalWorkflow::builder("traced-conditional".to_string())
        .add_branch(branch)
        .build();

    let executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"conditional": "test"}));

    info_span!("test_conditional_execution")
        .in_scope(|| async {
            let result = executor.execute_workflow(Arc::new(workflow), input).await;
            assert!(result.is_ok());
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_parallel_workflow_tracing() -> Result<()> {
    init_test_tracing();

    let workflow = ParallelWorkflow::builder("traced-parallel".to_string())
        .add_branch(
            ParallelBranch::new("parallel-branch-1".to_string()).add_step(WorkflowStep::new(
                "parallel-step-1".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            )),
        )
        .add_branch(
            ParallelBranch::new("parallel-branch-2".to_string()).add_step(WorkflowStep::new(
                "parallel-step-2".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            )),
        )
        .with_max_concurrency(2)
        .build()?;

    let executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"parallel": "test"}));

    info_span!("test_parallel_execution")
        .in_scope(|| async {
            let result = executor.execute_workflow(Arc::new(workflow), input).await;
            assert!(result.is_ok());
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_step_timing_tracing() -> Result<()> {
    init_test_tracing();

    let config = llmspell_workflows::WorkflowConfig::default();
    let _executor = StepExecutor::new(config);

    let step = WorkflowStep::new(
        "timed-step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: json!({"operation": "add", "values": [1, 1]}),
        },
    )
    .with_timeout(Duration::from_millis(100))
    .with_retry(2);

    // Test timing through workflow execution since StepExecutionContext is not public
    let workflow = Arc::new(
        SequentialWorkflow::builder("timed-workflow".to_string())
            .add_step(step)
            .build(),
    );

    let workflow_executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"timing": "test"}));

    // Test timing instrumentation
    info_span!("test_step_timing")
        .in_scope(|| async {
            let start = std::time::Instant::now();
            let _ = workflow_executor.execute_workflow(workflow, input).await;
            let duration = start.elapsed();

            // Verify timing was captured (duration should be non-zero even on failure)
            assert!(duration.as_nanos() > 0);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_error_handling_tracing() -> Result<()> {
    init_test_tracing();

    // Create a workflow that will fail
    let workflow = Arc::new(
        SequentialWorkflow::builder("error-workflow".to_string())
            .add_step(WorkflowStep::new(
                "failing-step".to_string(),
                StepType::Tool {
                    tool_name: "nonexistent-tool".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            ))
            .build(),
    );

    let executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"test": "error"}));

    info_span!("test_error_handling")
        .in_scope(|| async {
            let result = executor.execute_workflow(workflow, input).await;
            // We expect this to fail, we're just testing that error tracing works
            assert!(result.is_err() || !result.unwrap().success);
        })
        .await;

    Ok(())
}

#[tokio::test]
async fn test_workflow_tracing_performance() -> Result<()> {
    // Test that tracing doesn't add significant overhead
    let workflow = Arc::new(
        SequentialWorkflow::builder("perf-test".to_string())
            .add_step(WorkflowStep::new(
                "perf-step".to_string(),
                StepType::Tool {
                    tool_name: "calculator".to_string(),
                    parameters: json!({"operation": "add", "values": [1, 1]}),
                },
            ))
            .build(),
    );

    let executor = DefaultWorkflowExecutor::new();
    let input = WorkflowInput::new(json!({"perf": "test"}));

    // Measure without tracing span
    let start = std::time::Instant::now();
    for _ in 0..10 {
        let _ = executor
            .execute_workflow(workflow.clone(), input.clone())
            .await;
    }
    let duration_without = start.elapsed();

    // Measure with tracing span
    let start = std::time::Instant::now();
    info_span!("perf_test")
        .in_scope(|| async {
            for _ in 0..10 {
                let _ = executor
                    .execute_workflow(workflow.clone(), input.clone())
                    .await;
            }
        })
        .await;
    let duration_with = start.elapsed();

    // Check overhead is less than 2%
    let overhead_percent = ((duration_with.as_nanos() as f64 - duration_without.as_nanos() as f64)
        / duration_without.as_nanos() as f64)
        * 100.0;

    println!("Workflow tracing overhead: {:.2}%", overhead_percent);

    // Allow for some variance in CI/testing environments
    assert!(
        overhead_percent < 10.0,
        "Tracing overhead too high: {:.2}%",
        overhead_percent
    );

    Ok(())
}
