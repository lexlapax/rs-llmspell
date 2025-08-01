//! Comprehensive tests for parallel workflow functionality

use llmspell_workflows::{
    traits::{ErrorStrategy, StepType, WorkflowStep},
    ParallelBranch, ParallelWorkflowBuilder, WorkflowConfig,
};
use serde_json::json;
use std::time::Duration;

/// Test basic parallel execution
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_basic_parallel() {
    let branch1 = ParallelBranch::new("branch1".to_string()).add_step(WorkflowStep::new(
        "step1".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: json!({"expression": "1 + 1"}),
        },
    ));

    let branch2 = ParallelBranch::new("branch2".to_string()).add_step(WorkflowStep::new(
        "step2".to_string(),
        StepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: json!({"input": {"value": 42}}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("test_parallel")
        .add_branch(branch1)
        .add_branch(branch2)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.branch_results.len(), 2);
    assert_eq!(result.successful_branches, 2);
    assert_eq!(result.failed_branches, 0);
    assert!(!result.stopped_early);
}

/// Test optional branches
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_optional_branches() {
    let required = ParallelBranch::new("required".to_string()).add_step(WorkflowStep::new(
        "required_step".to_string(),
        StepType::Custom {
            function_name: "always_success".to_string(),
            parameters: json!({}),
        },
    ));

    let optional = ParallelBranch::new("optional".to_string())
        .optional()
        .add_step(WorkflowStep::new(
            "optional_step".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Will fail
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("test_optional")
        .add_branch(required)
        .add_branch(optional)
        .continue_on_optional_failure(true)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success); // Should succeed because required branch succeeded
    assert_eq!(result.successful_branches, 1);
    assert_eq!(result.failed_branches, 1);
}

/// Test fail-fast behavior
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_fail_fast() {
    let fast_fail = ParallelBranch::new("fast_fail".to_string()).add_step(WorkflowStep::new(
        "fail_step".to_string(),
        StepType::Tool {
            tool_name: "".to_string(), // Empty tool name causes immediate failure
            parameters: json!({}),
        },
    ));

    let slow_branch = ParallelBranch::new("slow_branch".to_string()).add_step(WorkflowStep::new(
        "slow_step".to_string(),
        StepType::Custom {
            function_name: "sleep".to_string(),
            parameters: json!({"ms": 5000}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("test_fail_fast")
        .add_branch(fast_fail)
        .add_branch(slow_branch)
        .fail_fast(true)
        .build()
        .unwrap();

    let start = std::time::Instant::now();
    let result = workflow.execute().await.unwrap();
    let elapsed = start.elapsed();

    println!("Fail-fast test result:");
    println!("  Success: {}", result.success);
    println!("  Stopped early: {}", result.stopped_early);
    println!("  Elapsed: {:?}", elapsed);
    println!("  Branch results: {}", result.branch_results.len());
    for br in &result.branch_results {
        println!(
            "    {}: success={}, required={}",
            br.branch_name, br.success, br.required
        );
    }

    assert!(!result.success);
    assert!(result.stopped_early || elapsed < Duration::from_secs(2)); // Should stop early
}

/// Test concurrency limits
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_concurrency_limits() {
    let mut branches = Vec::new();

    // Create 6 branches that each take ~100ms
    for i in 0..6 {
        let branch = ParallelBranch::new(format!("branch_{}", i)).add_step(WorkflowStep::new(
            format!("step_{}", i),
            StepType::Custom {
                function_name: "delay".to_string(),
                parameters: json!({"ms": 100}),
            },
        ));
        branches.push(branch);
    }

    let mut builder = ParallelWorkflowBuilder::new("test_concurrency").with_max_concurrency(2); // Only 2 at a time

    for branch in branches {
        builder = builder.add_branch(branch);
    }

    let workflow = builder.build().unwrap();

    let start = std::time::Instant::now();
    let result = workflow.execute().await.unwrap();
    let elapsed = start.elapsed();

    assert!(result.success);
    assert_eq!(result.branch_results.len(), 6);
    // With concurrency 2 and 100ms per task: should take at least 300ms (3 batches)
    assert!(elapsed >= Duration::from_millis(250)); // Allow some tolerance
}

/// Test branch timeout
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_branch_timeout() {
    let timeout_branch = ParallelBranch::new("timeout_branch".to_string())
        .with_timeout(Duration::from_millis(100))
        .add_step(WorkflowStep::new(
            "slow_step".to_string(),
            StepType::Custom {
                function_name: "sleep".to_string(),
                parameters: json!({"ms": 5000}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("test_timeout")
        .add_branch(timeout_branch)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(!result.success);
    assert_eq!(result.failed_branches, 1);
    let branch_result = &result.branch_results[0];
    assert!(branch_result.error.as_ref().unwrap().contains("timed out"));
}

/// Test workflow timeout
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_workflow_timeout() {
    let mut branches = Vec::new();

    // Create 3 slow branches
    for i in 0..3 {
        let branch = ParallelBranch::new(format!("slow_{}", i)).add_step(WorkflowStep::new(
            format!("slow_step_{}", i),
            StepType::Custom {
                function_name: "sleep".to_string(),
                parameters: json!({"ms": 5000}),
            },
        ));
        branches.push(branch);
    }

    let mut builder = ParallelWorkflowBuilder::new("test_workflow_timeout")
        .with_timeout(Duration::from_millis(200));

    for branch in branches {
        builder = builder.add_branch(branch);
    }

    let workflow = builder.build().unwrap();

    let start = std::time::Instant::now();
    let result = workflow.execute().await.unwrap();
    let elapsed = start.elapsed();

    assert!(elapsed < Duration::from_secs(1)); // Should timeout quickly
    assert!(result.stopped_early);
}

/// Test empty branch validation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_empty_branches() {
    let result = ParallelWorkflowBuilder::new("test_empty").build();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("at least one branch"));
    }
}

/// Test zero concurrency validation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_zero_concurrency() {
    let branch = ParallelBranch::new("branch".to_string()).add_step(WorkflowStep::new(
        "step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: json!({}),
        },
    ));

    let result = ParallelWorkflowBuilder::new("test_zero_concurrency")
        .add_branch(branch)
        .with_max_concurrency(0)
        .build();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("at least 1"));
    }
}

/// Test multiple steps per branch
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_multiple_steps_per_branch() {
    let branch = ParallelBranch::new("multi_step".to_string())
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": "1 + 1"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: json!({"input": {"result": "$step1_output"}}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step3".to_string(),
            StepType::Custom {
                function_name: "finalize".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("test_multi_step")
        .add_branch(branch)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.branch_results.len(), 1);
    let branch_result = &result.branch_results[0];
    assert_eq!(branch_result.step_results.len(), 3);
}

/// Test error propagation in branches
#[cfg_attr(test_category = "integration")]
#[tokio::test]
#[tracing::instrument]
async fn test_error_propagation() {
    // Initialize tracing for this test
    let _ = tracing_subscriber::fmt()
        .with_env_filter("llmspell_workflows=debug")
        .try_init();
    let branch = ParallelBranch::new("error_branch".to_string())
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "success".to_string(),
                parameters: json!({}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Will fail
                parameters: json!({}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step3".to_string(),
            StepType::Custom {
                function_name: "should_not_run".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("test_error_prop")
        .add_branch(branch)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    // Debug output
    println!("Error propagation test result:");
    println!("  Success: {}", result.success);
    println!("  Branch results count: {}", result.branch_results.len());
    println!("  Stopped early: {}", result.stopped_early);
    if let Some(error) = &result.error {
        println!("  Error: {}", error);
    }

    assert!(!result.success);
    assert!(
        !result.branch_results.is_empty(),
        "Expected at least one branch result"
    );
    let branch_result = &result.branch_results[0];
    assert!(!branch_result.success);
    // Only first step should have succeeded
    assert!(!branch_result.step_results.is_empty());
    assert!(branch_result.step_results[0].success);
}

/// Test workflow configuration integration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_workflow_config() {
    let config = WorkflowConfig {
        max_execution_time: Some(Duration::from_secs(30)),
        default_error_strategy: ErrorStrategy::Retry {
            max_attempts: 2,
            backoff_ms: 100,
        },
        ..Default::default()
    };

    let branch = ParallelBranch::new("config_test".to_string()).add_step(WorkflowStep::new(
        "step".to_string(),
        StepType::Custom {
            function_name: "test".to_string(),
            parameters: json!({}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("test_config")
        .add_branch(branch)
        .with_workflow_config(config)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
}

/// Test result report generation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_result_report() {
    let branch1 = ParallelBranch::new("success_branch".to_string()).add_step(WorkflowStep::new(
        "step".to_string(),
        StepType::Custom {
            function_name: "test".to_string(),
            parameters: json!({}),
        },
    ));

    let branch2 = ParallelBranch::new("fail_branch".to_string())
        .optional()
        .add_step(WorkflowStep::new(
            "step".to_string(),
            StepType::Tool {
                tool_name: "".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("test_report")
        .add_branch(branch1)
        .add_branch(branch2)
        .fail_fast(false)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();
    let report = result.generate_report();

    assert!(report.contains("test_report"));
    assert!(report.contains("success_branch"));
    assert!(report.contains("fail_branch"));
    assert!(report.contains("required"));
    assert!(report.contains("optional"));
}

/// Test mixed success and failure handling
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_mixed_results() {
    let mut branches = Vec::new();

    // Success branches
    for i in 0..3 {
        let branch = ParallelBranch::new(format!("success_{}", i)).add_step(WorkflowStep::new(
            format!("step_{}", i),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": format!("{} + {}", i, i)}),
            },
        ));
        branches.push(branch);
    }

    // Optional failure branches
    for i in 0..2 {
        let branch = ParallelBranch::new(format!("optional_fail_{}", i))
            .optional()
            .add_step(WorkflowStep::new(
                format!("fail_step_{}", i),
                StepType::Tool {
                    tool_name: "".to_string(),
                    parameters: json!({}),
                },
            ));
        branches.push(branch);
    }

    let mut builder = ParallelWorkflowBuilder::new("test_mixed")
        .fail_fast(false)
        .continue_on_optional_failure(true);

    for branch in branches {
        builder = builder.add_branch(branch);
    }

    let workflow = builder.build().unwrap();
    let result = workflow.execute().await.unwrap();

    assert!(result.success); // Should succeed because all required branches succeeded
    assert_eq!(result.successful_branches, 3);
    assert_eq!(result.failed_branches, 2);
    assert_eq!(result.branch_results.len(), 5);
}

/// Test branch builder methods
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_branch_builder() {
    let branch = ParallelBranch::new("test_branch".to_string())
        .with_description("Test branch description".to_string())
        .optional()
        .with_timeout(Duration::from_secs(10))
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
                function_name: "test2".to_string(),
                parameters: json!({}),
            },
        ));

    assert_eq!(branch.name, "test_branch");
    assert_eq!(branch.description, "Test branch description");
    assert!(!branch.required);
    assert_eq!(branch.timeout, Some(Duration::from_secs(10)));
    assert_eq!(branch.steps.len(), 2);
}
