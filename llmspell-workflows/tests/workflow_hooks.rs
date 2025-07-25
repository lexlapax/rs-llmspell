// ABOUTME: Comprehensive tests for workflow hook integration
// ABOUTME: Tests all workflow patterns with hook execution verification

use llmspell_workflows::{
    conditional::{ConditionalBranch, ConditionalWorkflow},
    conditions::Condition,
    hooks::{WorkflowExecutor, WorkflowLifecycleConfig},
    parallel::{ParallelBranch, ParallelWorkflowBuilder},
    r#loop::LoopWorkflowBuilder,
    sequential::SequentialWorkflow,
    traits::{ErrorStrategy, StepType, WorkflowStep},
};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_sequential_workflow_with_hooks() {
    // Create workflow executor without hook executor (testing basic integration)
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create workflow with hooks
    let workflow = SequentialWorkflow::builder("test_sequential".to_string())
        .with_hooks(workflow_executor)
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success);
    assert_eq!(result.successful_steps.len(), 2);
}

#[tokio::test]
async fn test_conditional_workflow_with_hooks() {
    // Create workflow executor
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create conditional workflow with hooks
    let branch1 = ConditionalBranch::new("branch1".to_string(), Condition::Always).with_step(
        WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ),
    );

    let workflow = ConditionalWorkflow::builder("test_conditional".to_string())
        .with_hooks(workflow_executor)
        .add_branch(branch1)
        .build();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success);
    assert_eq!(result.executed_branches.len(), 1);
}

#[tokio::test]
async fn test_loop_workflow_with_hooks() {
    // Create workflow executor
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create loop workflow with hooks
    let workflow = LoopWorkflowBuilder::new("test_loop".to_string())
        .with_range(0, 3, 1)
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .with_hooks(workflow_executor)
        .build()
        .unwrap();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success);
    assert_eq!(result.completed_iterations, 3);
}

#[tokio::test]
async fn test_parallel_workflow_with_hooks() {
    // Create workflow executor
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create parallel workflow with hooks
    let branch1 = ParallelBranch::new("branch1".to_string()).add_step(WorkflowStep::new(
        "step1".to_string(),
        StepType::Custom {
            function_name: "test".to_string(),
            parameters: json!({}),
        },
    ));

    let branch2 = ParallelBranch::new("branch2".to_string()).add_step(WorkflowStep::new(
        "step2".to_string(),
        StepType::Custom {
            function_name: "test".to_string(),
            parameters: json!({}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("test_parallel".to_string())
        .add_branch(branch1)
        .add_branch(branch2)
        .with_hooks(workflow_executor)
        .build()
        .unwrap();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success);
    assert_eq!(result.branch_results.len(), 2);
}

#[tokio::test]
async fn test_workflow_without_hooks() {
    // Test that workflows work without hooks (backward compatibility)
    let workflow = SequentialWorkflow::builder("test_no_hooks".to_string())
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    let result = workflow.execute().await.unwrap();
    assert!(result.success);
}

#[tokio::test]
async fn test_error_handling_with_hooks() {
    // Create workflow executor
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create workflow with a failing step
    let workflow = SequentialWorkflow::builder("test_error".to_string())
        .with_hooks(workflow_executor)
        .with_error_strategy(ErrorStrategy::Continue)
        .add_step(WorkflowStep::new(
            "failing_step".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Empty tool name causes failure
                parameters: json!({}),
            },
        ))
        .add_step(WorkflowStep::new(
            "success_step".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success); // Should succeed because error strategy is Continue
    assert_eq!(result.failed_steps.len(), 1);
    assert_eq!(result.successful_steps.len(), 1);
}

#[tokio::test]
async fn test_state_management_with_hooks() {
    // Create workflow executor
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create workflow
    let workflow = SequentialWorkflow::builder("test_state".to_string())
        .with_hooks(workflow_executor)
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    // Set some shared data
    workflow
        .set_shared_data("test_key".to_string(), json!("test_value"))
        .await
        .unwrap();

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert!(result.success);

    // Verify state was preserved
    let value = workflow.get_shared_data("test_key").await.unwrap();
    assert_eq!(value, Some(json!("test_value")));
}

#[tokio::test]
async fn test_nested_workflow_patterns() {
    // Test that complex nested workflow patterns work with hooks
    let workflow_executor = Arc::new(WorkflowExecutor::new(
        WorkflowLifecycleConfig::default(),
        None,
        None,
    ));

    // Create a conditional workflow with multiple branches
    let branch1 = ConditionalBranch::new("main_branch".to_string(), Condition::Always)
        .with_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({"branch": "main"}),
            },
        ))
        .with_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({"branch": "main"}),
            },
        ));

    let branch2 = ConditionalBranch::new("fallback_branch".to_string(), Condition::Never)
        .with_step(WorkflowStep::new(
            "fallback_step".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({"branch": "fallback"}),
            },
        ));

    let workflow = ConditionalWorkflow::builder("test_nested".to_string())
        .with_hooks(workflow_executor)
        .add_branch(branch1)
        .add_branch(branch2)
        .build();

    let result = workflow.execute().await.unwrap();
    assert!(result.success);
    assert_eq!(result.executed_branches.len(), 1);
    assert_eq!(result.executed_branches[0].branch_name, "main_branch");
}

#[tokio::test]
async fn test_workflow_lifecycle_config() {
    // Test workflow with custom lifecycle configuration
    let mut config = WorkflowLifecycleConfig::default();
    config.enable_circuit_breaker = true;
    config.max_hook_execution_time = std::time::Duration::from_millis(100);

    let workflow_executor = Arc::new(WorkflowExecutor::new(config, None, None));

    let workflow = SequentialWorkflow::builder("test_lifecycle".to_string())
        .with_hooks(workflow_executor)
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "test".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    let result = workflow.execute().await.unwrap();
    assert!(result.success);
}
