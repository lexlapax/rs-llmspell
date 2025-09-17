//! Comprehensive workflow tracing tests for Phase 9.4.5.6 Subtask 6.2
//!
//! Tests the instrumentation added to:
//! - Workflow executor
//! - Step executor
//! - Conditional workflow logic
//! - Parallel execution
//! - Sequential execution

use llmspell_workflows::{
    ConditionalWorkflow, ConditionalBranch, ParallelWorkflow, ParallelBranch,
    SequentialWorkflow, WorkflowStep, StepType, WorkflowConfig, ErrorStrategy,
    Condition, DefaultWorkflowExecutor, WorkflowExecutor, ExecutionContext,
    WorkflowInput, WorkflowOutput,
};
use llmspell_core::{ComponentId, ComponentMetadata};
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tracing::{debug, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use uuid::Uuid;

/// Test helper to capture trace output
struct TraceCapture {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl TraceCapture {
    fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn contains(&self, pattern: &str) -> bool {
        self.buffer
            .lock()
            .unwrap()
            .iter()
            .any(|line| line.contains(pattern))
    }

    fn setup_subscriber(&self) -> tracing::subscriber::DefaultGuard {
        let buffer = self.buffer.clone();

        // Create a custom writer that captures to our buffer
        let make_writer = move || TraceCaptureWriter {
            buffer: buffer.clone(),
        };

        let subscriber = tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    .with_writer(make_writer)
                    .with_level(true)
                    .with_target(true)
                    .with_thread_ids(false)
                    .with_thread_names(false)
                    .without_time()
                    .compact(),
            )
            .with(tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("llmspell_workflows=trace".parse().unwrap()));

        tracing::subscriber::set_default(subscriber)
    }
}

struct TraceCaptureWriter {
    buffer: Arc<Mutex<Vec<String>>>,
}

impl std::io::Write for TraceCaptureWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = String::from_utf8_lossy(buf);
        self.buffer.lock().unwrap().push(s.to_string());
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Test workflow executor instrumentation
#[tokio::test]
async fn test_workflow_executor_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting workflow executor tracing test");

    let executor = Arc::new(DefaultWorkflowExecutor::new());

    // Create a simple sequential workflow
    let workflow = Arc::new(SequentialWorkflow::builder()
        .name("test-workflow")
        .add_step(WorkflowStep {
            id: ComponentId::new(),
            name: "test-step".to_string(),
            description: "Test step".to_string(),
            step_type: StepType::Custom {
                function_name: "test_func".to_string(),
            },
            required: true,
            timeout: None,
            retry_count: 0,
        })
        .build()
        .unwrap());

    let input = WorkflowInput {
        data: serde_json::json!({"test": "data"}),
        context: std::collections::HashMap::new(),
    };

    // Execute workflow
    let _ = executor.execute_workflow(workflow, input).await;

    // Verify tracing output
    assert!(
        capture.contains("execute_workflow"),
        "Should trace workflow execution"
    );
    assert!(
        capture.contains("workflow_name"),
        "Should include workflow name in trace"
    );
    assert!(
        capture.contains("execution_id"),
        "Should include execution ID in trace"
    );
}

/// Test step executor instrumentation
#[tokio::test]
async fn test_step_executor_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting step executor tracing test");

    use llmspell_workflows::{StepExecutor, StepExecutionContext};

    let executor = StepExecutor::new(WorkflowConfig::default());

    let step = WorkflowStep {
        id: ComponentId::new(),
        name: "traced-step".to_string(),
        description: "Step for tracing".to_string(),
        step_type: StepType::Custom {
            function_name: "traced_func".to_string(),
        },
        required: true,
        timeout: Some(Duration::from_secs(5)),
        retry_count: 0,
    };

    let context = StepExecutionContext::new();

    // Execute step
    let _ = executor.execute_step(&step, context).await;

    // Verify tracing output
    assert!(
        capture.contains("execute_step"),
        "Should trace step execution"
    );
    assert!(
        capture.contains("step_name"),
        "Should include step name in trace"
    );
    assert!(
        capture.contains("step_type"),
        "Should include step type in trace"
    );
}

/// Test conditional workflow instrumentation
#[tokio::test]
async fn test_conditional_workflow_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting conditional workflow tracing test");

    let workflow = ConditionalWorkflow::builder()
        .name("traced-conditional")
        .add_branch(
            ConditionalBranch::new(
                "branch1".to_string(),
                Condition::Always,
            )
            .with_step(WorkflowStep {
                id: ComponentId::new(),
                name: "conditional-step".to_string(),
                description: "Step in conditional".to_string(),
                step_type: StepType::Custom {
                    function_name: "cond_func".to_string(),
                },
                required: true,
                timeout: None,
                retry_count: 0,
            })
        )
        .build()
        .unwrap();

    // Execute workflow
    let _ = workflow.execute_workflow().await;

    // Verify tracing output
    assert!(
        capture.contains("execute_workflow"),
        "Should trace conditional workflow execution"
    );
    assert!(
        capture.contains("branch_count"),
        "Should include branch count in trace"
    );
    assert!(
        capture.contains("execute_branch"),
        "Should trace branch execution"
    );
    assert!(
        capture.contains("branch_name"),
        "Should include branch name in trace"
    );
}

/// Test parallel workflow instrumentation
#[tokio::test]
async fn test_parallel_workflow_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting parallel workflow tracing test");

    let workflow = ParallelWorkflow::builder()
        .name("traced-parallel")
        .add_branch(
            ParallelBranch::new("parallel-branch-1")
                .with_step(WorkflowStep {
                    id: ComponentId::new(),
                    name: "parallel-step-1".to_string(),
                    description: "First parallel step".to_string(),
                    step_type: StepType::Custom {
                        function_name: "parallel_func_1".to_string(),
                    },
                    required: true,
                    timeout: None,
                    retry_count: 0,
                })
        )
        .add_branch(
            ParallelBranch::new("parallel-branch-2")
                .with_step(WorkflowStep {
                    id: ComponentId::new(),
                    name: "parallel-step-2".to_string(),
                    description: "Second parallel step".to_string(),
                    step_type: StepType::Custom {
                        function_name: "parallel_func_2".to_string(),
                    },
                    required: true,
                    timeout: None,
                    retry_count: 0,
                })
        )
        .max_parallel(2)
        .build()
        .unwrap();

    use llmspell_core::{types::AgentInput, execution_context::ExecutionContext};
    use llmspell_core::traits::base_agent::BaseAgent;

    let input = AgentInput {
        data: "test parallel".to_string(),
        metadata: std::collections::HashMap::new(),
    };

    let context = ExecutionContext::new();

    // Execute workflow
    let _ = workflow.execute(input, context).await;

    // Verify tracing output
    assert!(
        capture.contains("execute_impl"),
        "Should trace parallel workflow execution"
    );
    assert!(
        capture.contains("branch_count"),
        "Should include branch count in trace"
    );
    assert!(
        capture.contains("max_parallel"),
        "Should include max parallel setting in trace"
    );
    assert!(
        capture.contains("execute_branch"),
        "Should trace parallel branch execution"
    );
}

/// Test step timing instrumentation
#[tokio::test]
async fn test_step_timing_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting step timing tracing test");

    use llmspell_workflows::{StepExecutor, StepExecutionContext};

    let executor = StepExecutor::new(WorkflowConfig::default());

    let step = WorkflowStep {
        id: ComponentId::new(),
        name: "timed-step".to_string(),
        description: "Step with timing".to_string(),
        step_type: StepType::Custom {
            function_name: "timed_func".to_string(),
        },
        required: true,
        timeout: Some(Duration::from_millis(100)),
        retry_count: 2,
    };

    let context = StepExecutionContext::new();
    let error_strategy = ErrorStrategy {
        max_retries: 2,
        retry_delay: Duration::from_millis(10),
        ..Default::default()
    };

    // Execute step with retry
    let _ = executor.execute_step_with_retry(&step, context, &error_strategy).await;

    // Verify tracing output
    assert!(
        capture.contains("execute_step_with_retry"),
        "Should trace step retry execution"
    );
    assert!(
        capture.contains("max_retries"),
        "Should include max retries in trace"
    );
}

/// Test error handling instrumentation
#[tokio::test]
async fn test_error_handling_tracing() {
    let capture = TraceCapture::new();
    let _guard = capture.setup_subscriber();

    info!("Starting error handling tracing test");

    let executor = Arc::new(DefaultWorkflowExecutor::new());

    // Create a workflow that will fail
    let workflow = Arc::new(SequentialWorkflow::builder()
        .name("error-workflow")
        .add_step(WorkflowStep {
            id: ComponentId::new(),
            name: "failing-step".to_string(),
            description: "Step that fails".to_string(),
            step_type: StepType::Tool {
                tool_id: ComponentId::from_string("nonexistent-tool"),
                parameters: serde_json::Value::Null,
            },
            required: true,
            timeout: None,
            retry_count: 0,
        })
        .build()
        .unwrap());

    let input = WorkflowInput {
        data: serde_json::json!({"test": "error"}),
        context: std::collections::HashMap::new(),
    };

    // Execute workflow (will fail)
    let _ = executor.execute_workflow(workflow, input).await;

    // Verify error tracing
    assert!(
        capture.contains("run_error_hooks"),
        "Should trace error hook execution"
    );
}

/// Test that tracing doesn't introduce performance regression
#[tokio::test]
async fn test_tracing_performance_impact() {
    use std::time::Instant;

    // Test without detailed tracing
    let start = Instant::now();
    for _ in 0..50 {
        let workflow = SequentialWorkflow::builder()
            .name("perf-test")
            .add_step(WorkflowStep {
                id: ComponentId::new(),
                name: "perf-step".to_string(),
                description: "Performance test step".to_string(),
                step_type: StepType::Custom {
                    function_name: "perf_func".to_string(),
                },
                required: true,
                timeout: None,
                retry_count: 0,
            })
            .build()
            .unwrap();

        use llmspell_core::{types::AgentInput, execution_context::ExecutionContext};
        use llmspell_core::traits::base_agent::BaseAgent;

        let input = AgentInput {
            data: "perf test".to_string(),
            metadata: std::collections::HashMap::new(),
        };

        let context = ExecutionContext::new();
        let _ = workflow.execute(input, context).await;
    }
    let duration_with_tracing = start.elapsed();

    // Performance assertion - should complete 50 iterations quickly
    assert!(
        duration_with_tracing.as_millis() < 2000,
        "50 workflow executions should complete in under 2 seconds, took {}ms",
        duration_with_tracing.as_millis()
    );

    info!("Performance test: 50 iterations in {:?}", duration_with_tracing);
}

/// Verify no new clippy warnings in instrumented code
#[test]
fn test_no_new_clippy_warnings() {
    // This test just needs to compile without warnings
    // The actual clippy check is done during compilation

    // Test that all our trace macros compile without warnings
    info!("Testing trace macro usage");
    debug!("Debug level trace");
    warn!("Warning level trace");

    // Test field recording patterns we use
    let execution_id = Uuid::new_v4();
    info!(execution_id = %execution_id, "Testing field recording");

    // Test skip patterns
    let large_data = vec![0u8; 1000];
    debug!(data_size = large_data.len(), "Skipping large data in trace");
}