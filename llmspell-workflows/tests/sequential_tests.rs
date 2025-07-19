//! Comprehensive integration tests for SequentialWorkflow
//!
//! Tests workflow functionality end-to-end with various scenarios,
//! error conditions, and performance requirements.

use llmspell_workflows::{
    ErrorStrategy, SequentialWorkflow, StepType, WorkflowConfig, WorkflowStatus, WorkflowStep,
};
use std::time::Duration;

/// Test basic workflow creation and step management
#[tokio::test]
async fn test_workflow_creation_and_setup() {
    let config = WorkflowConfig::default();
    let workflow = SequentialWorkflow::new("test_workflow".to_string(), config);

    assert_eq!(workflow.name(), "test_workflow");
    assert_eq!(workflow.step_count(), 0);
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Pending
    );
}

/// Test workflow execution with multiple tool types
#[tokio::test]
async fn test_mixed_tool_workflow_execution() {
    let config = WorkflowConfig::default();

    // Step 1: Tool execution
    let tool_step = WorkflowStep::new(
        "calculator_step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({"expression": "5 * 8"}),
        },
    );

    // Step 2: Custom function
    let custom_step = WorkflowStep::new(
        "validation_step".to_string(),
        StepType::Custom {
            function_name: "validation".to_string(),
            parameters: serde_json::json!({"strict": true}),
        },
    );

    // Step 3: Agent execution
    let agent_step = WorkflowStep::new(
        "agent_step".to_string(),
        StepType::Agent {
            agent_id: llmspell_core::ComponentId::new(),
            input: "Process data with agent".to_string(),
        },
    );

    let workflow = SequentialWorkflow::builder("mixed_tools_workflow".to_string())
        .with_config(config)
        .add_step(tool_step)
        .add_step(custom_step)
        .add_step(agent_step)
        .build();

    assert_eq!(workflow.step_count(), 3);

    // Execute workflow
    let result = workflow.execute().await.unwrap();
    assert_eq!(result.total_steps(), 3);
    assert!(result.success);
    assert_eq!(result.successful_steps.len(), 3);
    assert_eq!(result.failed_steps.len(), 0);
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Completed
    );

    // Verify outputs are different for each step type
    assert!(result.successful_steps[0].output.contains("Calculator"));
    assert!(result.successful_steps[1].output.contains("Validation"));
    assert!(result.successful_steps[2].output.contains("Agent"));
}

/// Test workflow with shared data between steps
#[tokio::test]
async fn test_workflow_shared_data() {
    let config = WorkflowConfig::default();

    let step = WorkflowStep::new(
        "data_processor".to_string(),
        StepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({"operation": "transform"}),
        },
    );

    let workflow = SequentialWorkflow::builder("shared_data_workflow".to_string())
        .with_config(config)
        .add_step(step)
        .build();

    // Set initial shared data
    workflow
        .set_shared_data("user_id".to_string(), serde_json::json!("user_123"))
        .await
        .unwrap();
    workflow
        .set_shared_data("session_id".to_string(), serde_json::json!("session_456"))
        .await
        .unwrap();

    // Execute and verify shared data persists
    let _result = workflow.execute().await.unwrap();

    let user_id = workflow.get_shared_data("user_id").await.unwrap();
    let session_id = workflow.get_shared_data("session_id").await.unwrap();

    assert_eq!(user_id, Some(serde_json::json!("user_123")));
    assert_eq!(session_id, Some(serde_json::json!("session_456")));
}

/// Test workflow continue-on-error strategy
#[tokio::test]
async fn test_workflow_continue_strategy() {
    let mut config = WorkflowConfig::default();
    config.continue_on_error = true; // Continue on error
    let error_strategy = ErrorStrategy::Continue;

    // Add multiple steps - even if one fails, others should continue
    let steps = vec![
        ("step1", "calculator"),
        ("step2", "json_processor"),
        ("step3", "file_operations"),
    ];

    let mut workflow_builder = SequentialWorkflow::builder("continue_workflow".to_string())
        .with_config(config)
        .with_error_strategy(error_strategy);

    for (name, tool) in steps {
        let step = WorkflowStep::new(
            name.to_string(),
            StepType::Tool {
                tool_name: tool.to_string(),
                parameters: serde_json::json!({}),
            },
        );
        workflow_builder = workflow_builder.add_step(step);
    }

    let workflow = workflow_builder.build();
    let result = workflow.execute().await.unwrap();

    assert_eq!(result.total_steps(), 3);
    assert!(result.success); // All should succeed with mock execution
    assert_eq!(result.successful_steps.len(), 3);
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Completed
    );
}

/// Test workflow step timeout functionality
#[tokio::test]
async fn test_workflow_step_timeout() {
    let config = WorkflowConfig::default();

    // Add a step with very short timeout
    let timeout_step = WorkflowStep::new(
        "timeout_step".to_string(),
        StepType::Custom {
            function_name: "slow_function".to_string(),
            parameters: serde_json::json!({}),
        },
    )
    .with_timeout(Duration::from_millis(1)); // Very short timeout

    let workflow = SequentialWorkflow::builder("timeout_workflow".to_string())
        .with_config(config)
        .add_step(timeout_step)
        .build();

    let result = workflow.execute().await.unwrap();
    assert_eq!(result.total_steps(), 1);
    assert!(!result.success); // Should fail due to timeout
    assert_eq!(result.failed_steps.len(), 1);
    assert!(result.failed_steps[0]
        .error
        .as_ref()
        .unwrap()
        .contains("timed out"));
}

/// Test workflow execution statistics
#[tokio::test]
async fn test_workflow_execution_statistics() {
    let config = WorkflowConfig::default();

    let mut workflow_builder =
        SequentialWorkflow::builder("stats_workflow".to_string()).with_config(config);

    // Add several steps
    for i in 1..=5 {
        let step = WorkflowStep::new(
            format!("step_{}", i),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": format!("{} + {}", i, i)}),
            },
        );
        workflow_builder = workflow_builder.add_step(step);
    }

    let workflow = workflow_builder.build();
    let _result = workflow.execute().await.unwrap();
    let stats = workflow.get_execution_stats().await.unwrap();

    assert_eq!(stats.total_steps, 5);
    assert_eq!(stats.successful_steps, 5);
    assert_eq!(stats.failed_steps, 0);
    assert_eq!(stats.success_rate(), 100.0);
    assert!(stats.total_duration > Duration::ZERO);
    assert!(stats.average_step_duration > Duration::ZERO);
}

/// Test workflow reset functionality
#[tokio::test]
async fn test_workflow_reset() {
    let config = WorkflowConfig::default();

    let step = WorkflowStep::new(
        "test_step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({}),
        },
    );

    let workflow = SequentialWorkflow::builder("reset_workflow".to_string())
        .with_config(config)
        .add_step(step)
        .build();

    // Set shared data and execute
    workflow
        .set_shared_data("test_key".to_string(), serde_json::json!("test_value"))
        .await
        .unwrap();

    let _result = workflow.execute().await.unwrap();
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Completed
    );

    // Reset workflow
    workflow.reset().await.unwrap();

    // Verify everything is reset
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Pending
    );
    let shared_data = workflow.get_shared_data("test_key").await.unwrap();
    assert_eq!(shared_data, None);
}

/// Test workflow performance requirements (<50ms creation)
#[tokio::test]
async fn test_workflow_performance_creation() {
    let start = std::time::Instant::now();

    let config = WorkflowConfig::default();
    let _workflow = SequentialWorkflow::new("performance_test".to_string(), config);

    let creation_time = start.elapsed();

    // Should create workflow in under 50ms as per requirements
    assert!(
        creation_time < Duration::from_millis(50),
        "Workflow creation took {:?}, should be < 50ms",
        creation_time
    );
}

/// Test workflow with retry strategy
#[tokio::test]
async fn test_workflow_retry_strategy() {
    let config = WorkflowConfig::default();
    let retry_strategy = ErrorStrategy::Retry {
        max_attempts: 3,
        backoff_ms: 10, // Short delay for test
    };

    // Add step with retry configuration
    let retry_step = WorkflowStep::new(
        "retry_step".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(), // This should succeed
            parameters: serde_json::json!({"expression": "2 + 2"}),
        },
    )
    .with_retry(2);

    let workflow = SequentialWorkflow::builder("retry_workflow".to_string())
        .with_config(config)
        .with_error_strategy(retry_strategy)
        .add_step(retry_step)
        .build();

    let result = workflow.execute().await.unwrap();
    assert_eq!(result.total_steps(), 1);
    assert!(result.success);
    assert_eq!(result.successful_steps[0].retry_count, 0); // Should succeed on first try
}

/// Integration test: Complete data processing pipeline
#[tokio::test]
async fn test_complete_data_pipeline() {
    let config = WorkflowConfig::default();

    // Build a realistic data processing pipeline
    let steps = vec![
        (
            "extract_data",
            "file_operations",
            serde_json::json!({"operation": "read", "path": "data.csv"}),
        ),
        (
            "validate_data",
            "data_validation",
            serde_json::json!({"schema": "strict"}),
        ),
        (
            "transform_data",
            "json_processor",
            serde_json::json!({"operation": "normalize"}),
        ),
        (
            "enrich_data",
            "api_enrichment",
            serde_json::json!({"service": "geocoding"}),
        ),
        (
            "aggregate_data",
            "aggregation",
            serde_json::json!({"type": "sum", "group_by": "region"}),
        ),
        (
            "store_results",
            "file_operations",
            serde_json::json!({"operation": "write", "path": "results.json"}),
        ),
    ];

    let mut workflow_builder =
        SequentialWorkflow::builder("data_pipeline".to_string()).with_config(config);

    for (name, tool, params) in steps {
        let step = WorkflowStep::new(
            name.to_string(),
            StepType::Tool {
                tool_name: tool.to_string(),
                parameters: params,
            },
        );
        workflow_builder = workflow_builder.add_step(step);
    }

    let workflow = workflow_builder.build();

    // Set pipeline metadata
    workflow
        .set_shared_data("pipeline_id".to_string(), serde_json::json!("pipeline_001"))
        .await
        .unwrap();
    workflow
        .set_shared_data(
            "start_time".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        )
        .await
        .unwrap();

    // Execute the complete pipeline
    let start_time = std::time::Instant::now();
    let result = workflow.execute().await.unwrap();
    let execution_time = start_time.elapsed();

    // Verify pipeline execution
    assert_eq!(result.total_steps(), 6);
    assert!(result.success);
    assert_eq!(result.successful_steps.len(), 6);
    assert_eq!(
        workflow.get_status().await.unwrap(),
        WorkflowStatus::Completed
    );

    // Verify pipeline metadata persisted
    let pipeline_id = workflow.get_shared_data("pipeline_id").await.unwrap();
    assert_eq!(pipeline_id, Some(serde_json::json!("pipeline_001")));

    // Performance check - should complete reasonably quickly
    assert!(
        execution_time < Duration::from_secs(1),
        "Pipeline took {:?}, expected < 1s",
        execution_time
    );

    // Check statistics
    let stats = workflow.get_execution_stats().await.unwrap();
    assert_eq!(stats.success_rate(), 100.0);
    assert_eq!(stats.total_retries, 0);

    println!("✅ Data pipeline completed successfully:");
    println!("   Duration: {:?}", execution_time);
    println!("   Steps: {}", stats.total_steps);
    println!("   Success Rate: {:.1}%", stats.success_rate());
}
