//! Comprehensive tests for loop workflow functionality

use llmspell_workflows::{
    traits::{ErrorStrategy, StepType, WorkflowStep},
    LoopWorkflowBuilder, ResultAggregation, WorkflowConfig,
};
use serde_json::json;
use std::time::Duration;

/// Test basic range iteration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_range_iteration() {
    let workflow = LoopWorkflowBuilder::new("test_range")
        .with_range(0, 5, 1)
        .add_step(WorkflowStep::new(
            "count".to_string(),
            StepType::Custom {
                function_name: "counter".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.total_iterations, 5);
    assert_eq!(result.completed_iterations, 5);
    assert!(result.break_reason.is_none());
}

/// Test range with negative step
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_range_negative_step() {
    let workflow = LoopWorkflowBuilder::new("test_negative_range")
        .with_range(10, 5, -1)
        .add_step(WorkflowStep::new(
            "countdown".to_string(),
            StepType::Custom {
                function_name: "counter".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.total_iterations, 5);
    assert_eq!(result.completed_iterations, 5);
}

/// Test invalid range configuration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_invalid_range() {
    // Zero step
    let result = LoopWorkflowBuilder::new("test_zero_step")
        .with_range(0, 10, 0)
        .add_step(WorkflowStep::new(
            "step".to_string(),
            StepType::Custom {
                function_name: "noop".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("zero"));
    }
}

/// Test collection iteration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_collection_iteration() {
    let items = vec![json!("apple"), json!("banana"), json!("cherry")];

    let workflow = LoopWorkflowBuilder::new("test_collection")
        .with_collection(items)
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Custom {
                function_name: "process_item".to_string(),
                parameters: json!({"item": "$loop_value"}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.total_iterations, 3);
    assert_eq!(result.completed_iterations, 3);
}

/// Test empty collection
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_empty_collection() {
    let workflow = LoopWorkflowBuilder::new("test_empty")
        .with_collection::<serde_json::Value>(vec![])
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Custom {
                function_name: "process_item".to_string(),
                parameters: json!({}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.total_iterations, 0);
    assert_eq!(result.completed_iterations, 0);
}

/// Test while condition
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_while_condition() {
    let workflow = LoopWorkflowBuilder::new("test_while")
        .with_while_condition("$iteration < 3", 10)
        .add_step(WorkflowStep::new(
            "increment".to_string(),
            StepType::Custom {
                function_name: "increment".to_string(),
                parameters: json!({"counter": "$loop_index"}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.completed_iterations <= 3);
}

/// Test break conditions
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_break_conditions() {
    let workflow = LoopWorkflowBuilder::new("test_break")
        .with_range(0, 100, 1)
        .add_step(WorkflowStep::new(
            "check".to_string(),
            StepType::Custom {
                function_name: "check_value".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .add_break_condition("$iteration > 5", Some("Limit reached".to_string()))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.completed_iterations <= 6); // 0-5 inclusive
    assert_eq!(result.break_reason, Some("Limit reached".to_string()));
}

/// Test multiple break conditions
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_multiple_break_conditions() {
    let workflow = LoopWorkflowBuilder::new("test_multi_break")
        .with_range(0, 100, 1)
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Custom {
                function_name: "process".to_string(),
                parameters: json!({}),
            },
        ))
        .add_break_condition("$iteration == 3", Some("Hit 3".to_string()))
        .add_break_condition("$iteration == 10", Some("Hit 10".to_string()))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.completed_iterations, 3);
    assert_eq!(result.break_reason, Some("Hit 3".to_string()));
}

/// Test continue on error
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_continue_on_error() {
    let workflow = LoopWorkflowBuilder::new("test_continue_on_error")
        .with_range(0, 5, 1)
        .add_step(WorkflowStep::new(
            "may_fail".to_string(),
            StepType::Custom {
                function_name: "fail_on_even".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .continue_on_error(true)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.completed_iterations, 5); // All iterations complete despite errors
}

/// Test fail fast on error
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_fail_fast() {
    // Use empty tool name to trigger failure
    let workflow = LoopWorkflowBuilder::new("test_fail_fast")
        .with_range(0, 10, 1)
        .add_step(WorkflowStep::new(
            "fail_step".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Empty tool name will cause failure
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .continue_on_error(false)
        .with_error_strategy(ErrorStrategy::FailFast)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(!result.success);
    assert!(result.completed_iterations < 10);
    assert!(result.error.is_some());
}

/// Test timeout
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeout() {
    let workflow = LoopWorkflowBuilder::new("test_timeout")
        .with_range(0, 1000, 1)
        .add_step(WorkflowStep::new(
            "slow_op".to_string(),
            StepType::Custom {
                function_name: "sleep".to_string(),
                parameters: json!({"ms": 10}),
            },
        ))
        .with_timeout(Duration::from_millis(50))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.completed_iterations < 1000);
    assert!(result.break_reason.is_some());
    assert!(result.break_reason.unwrap().contains("timeout"));
}

/// Test iteration delay
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_iteration_delay() {
    let start = std::time::Instant::now();

    let workflow = LoopWorkflowBuilder::new("test_delay")
        .with_range(0, 3, 1)
        .add_step(WorkflowStep::new(
            "quick_op".to_string(),
            StepType::Custom {
                function_name: "noop".to_string(),
                parameters: json!({}),
            },
        ))
        .with_iteration_delay(Duration::from_millis(100))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();
    let elapsed = start.elapsed();

    assert!(result.success);
    assert!(elapsed >= Duration::from_millis(200)); // At least 2 delays between 3 iterations
}

/// Test result aggregation - CollectAll
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_aggregation_collect_all() {
    let workflow = LoopWorkflowBuilder::new("test_collect_all")
        .with_range(0, 3, 1)
        .add_step(WorkflowStep::new(
            "generate".to_string(),
            StepType::Custom {
                function_name: "generate".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .with_aggregation(ResultAggregation::CollectAll)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.aggregated_results.contains_key("all_iterations"));

    if let Some(all_iterations) = result.aggregated_results.get("all_iterations") {
        assert!(all_iterations.is_array());
        assert_eq!(all_iterations.as_array().unwrap().len(), 3);
    }
}

/// Test result aggregation - LastOnly
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_aggregation_last_only() {
    let workflow = LoopWorkflowBuilder::new("test_last_only")
        .with_range(0, 5, 1)
        .add_step(WorkflowStep::new(
            "generate".to_string(),
            StepType::Custom {
                function_name: "generate".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .with_aggregation(ResultAggregation::LastOnly)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.aggregated_results.contains_key("last_iteration"));
    assert!(!result.aggregated_results.contains_key("all_iterations"));
}

/// Test result aggregation - FirstN
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_aggregation_first_n() {
    let workflow = LoopWorkflowBuilder::new("test_first_n")
        .with_range(0, 10, 1)
        .add_step(WorkflowStep::new(
            "generate".to_string(),
            StepType::Custom {
                function_name: "generate".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .with_aggregation(ResultAggregation::FirstN(3))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.aggregated_results.contains_key("iterations"));

    if let Some(iterations) = result.aggregated_results.get("iterations") {
        assert!(iterations.is_array());
        assert_eq!(iterations.as_array().unwrap().len(), 3);
    }
}

/// Test result aggregation - LastN
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_aggregation_last_n() {
    let workflow = LoopWorkflowBuilder::new("test_last_n")
        .with_range(0, 10, 1)
        .add_step(WorkflowStep::new(
            "generate".to_string(),
            StepType::Custom {
                function_name: "generate".to_string(),
                parameters: json!({"value": "$loop_value"}),
            },
        ))
        .with_aggregation(ResultAggregation::LastN(3))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert!(result.aggregated_results.contains_key("iterations"));

    if let Some(iterations) = result.aggregated_results.get("iterations") {
        assert!(iterations.is_array());
        assert_eq!(iterations.as_array().unwrap().len(), 3);
    }
}

/// Test complex workflow with multiple steps
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_multiple_steps() {
    let workflow = LoopWorkflowBuilder::new("test_multi_step")
        .with_range(0, 3, 1)
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Custom {
                function_name: "process1".to_string(),
                parameters: json!({"input": "$loop_value"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Custom {
                function_name: "process2".to_string(),
                parameters: json!({"input": "$loop_value", "index": "$loop_index"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step3".to_string(),
            StepType::Custom {
                function_name: "finalize".to_string(),
                parameters: json!({"iteration": "$iteration"}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
    assert_eq!(result.completed_iterations, 3);
}

/// Test workflow configuration integration
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_workflow_config() {
    let config = WorkflowConfig {
        max_execution_time: Some(Duration::from_secs(30)),
        default_error_strategy: ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 100,
        },
        ..Default::default()
    };

    let workflow = LoopWorkflowBuilder::new("test_config")
        .with_range(0, 5, 1)
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Custom {
                function_name: "process".to_string(),
                parameters: json!({}),
            },
        ))
        .with_workflow_config(config)
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.success);
}

/// Test empty body validation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_empty_body_validation() {
    let result = LoopWorkflowBuilder::new("test_empty_body")
        .with_range(0, 10, 1)
        .build();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("empty"));
    }
}

/// Test missing iterator validation
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_missing_iterator_validation() {
    let result = LoopWorkflowBuilder::new("test_no_iterator")
        .add_step(WorkflowStep::new(
            "step".to_string(),
            StepType::Custom {
                function_name: "noop".to_string(),
                parameters: json!({}),
            },
        ))
        .build();

    assert!(result.is_err());
    if let Err(e) = result {
        assert!(e.to_string().contains("iterator"));
    }
}

/// Test loop metadata in results
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_loop_metadata() {
    let workflow = LoopWorkflowBuilder::new("test_metadata")
        .with_range(0, 5, 1)
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Custom {
                function_name: "process".to_string(),
                parameters: json!({}),
            },
        ))
        .build()
        .unwrap();

    let result = workflow.execute().await.unwrap();

    assert!(result.aggregated_results.contains_key("loop_metadata"));

    if let Some(metadata) = result.aggregated_results.get("loop_metadata") {
        assert!(metadata.get("total_iterations").is_some());
        assert!(metadata.get("completed_iterations").is_some());
        assert!(metadata.get("duration_ms").is_some());
        assert!(metadata.get("break_reason").is_some());
    }
}
