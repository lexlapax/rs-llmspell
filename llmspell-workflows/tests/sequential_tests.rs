//! Comprehensive integration tests for BasicSequentialWorkflow
//!
//! Tests workflow functionality end-to-end with various scenarios,
//! error conditions, and performance requirements.

use llmspell_workflows::{
    BasicErrorStrategy, BasicSequentialWorkflow, BasicStepType, BasicWorkflow, BasicWorkflowConfig,
    BasicWorkflowStatus, BasicWorkflowStep,
};
use std::time::Duration;

/// Test basic workflow creation and step management
#[tokio::test]
async fn test_workflow_creation_and_setup() {
    let config = BasicWorkflowConfig::default();
    let workflow = BasicSequentialWorkflow::new("test_workflow".to_string(), config);

    assert_eq!(workflow.name(), "test_workflow");
    assert_eq!(workflow.step_count(), 0);
    assert!(workflow.is_empty());
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Pending
    );
}

/// Test workflow execution with multiple tool types
#[tokio::test]
async fn test_mixed_tool_workflow_execution() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("mixed_tools_workflow".to_string(), config);

    // Step 1: Tool execution
    let tool_step = BasicWorkflowStep::new(
        "calculator_step".to_string(),
        BasicStepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({"expression": "5 * 8"}),
        },
    );

    // Step 2: Custom function
    let custom_step = BasicWorkflowStep::new(
        "validation_step".to_string(),
        BasicStepType::Custom {
            function_name: "validation".to_string(),
            parameters: serde_json::json!({"strict": true}),
        },
    );

    // Step 3: Agent execution
    let agent_step = BasicWorkflowStep::new(
        "agent_step".to_string(),
        BasicStepType::Agent {
            agent_id: llmspell_core::ComponentId::new(),
            input: "Process data with agent".to_string(),
        },
    );

    workflow.add_step(tool_step).await.unwrap();
    workflow.add_step(custom_step).await.unwrap();
    workflow.add_step(agent_step).await.unwrap();

    assert_eq!(workflow.step_count(), 3);

    // Execute workflow
    let results = workflow.execute().await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success));
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Completed
    );

    // Verify outputs are different for each step type
    assert!(results[0].output.contains("Calculator"));
    assert!(results[1].output.contains("Validation"));
    assert!(results[2].output.contains("Agent"));
}

/// Test workflow with shared data between steps
#[tokio::test]
async fn test_workflow_shared_data() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("shared_data_workflow".to_string(), config);

    // Set initial shared data
    workflow
        .set_shared_data("user_id".to_string(), serde_json::json!("user_123"))
        .await
        .unwrap();
    workflow
        .set_shared_data("session_id".to_string(), serde_json::json!("session_456"))
        .await
        .unwrap();

    let step = BasicWorkflowStep::new(
        "data_processor".to_string(),
        BasicStepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({"operation": "transform"}),
        },
    );

    workflow.add_step(step).await.unwrap();

    // Execute and verify shared data persists
    let _results = workflow.execute().await.unwrap();

    let user_id = workflow.get_shared_data("user_id").await.unwrap();
    let session_id = workflow.get_shared_data("session_id").await.unwrap();

    assert_eq!(user_id, Some(serde_json::json!("user_123")));
    assert_eq!(session_id, Some(serde_json::json!("session_456")));
}

/// Test workflow failure with fail-fast strategy
#[tokio::test]
async fn test_workflow_fail_fast_strategy() {
    let mut config = BasicWorkflowConfig::default();
    config.continue_on_error = false; // Fail fast
    let mut workflow = BasicSequentialWorkflow::new("fail_fast_workflow".to_string(), config);

    // Add a successful step
    let success_step = BasicWorkflowStep::new(
        "success_step".to_string(),
        BasicStepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({"expression": "1 + 1"}),
        },
    );

    // Add a failing step (empty tool name will be caught in validation)
    let failing_step = BasicWorkflowStep::new(
        "failing_step".to_string(),
        BasicStepType::Tool {
            tool_name: "".to_string(), // This will cause validation failure
            parameters: serde_json::json!({}),
        },
    );

    workflow.add_step(success_step).await.unwrap();
    workflow.add_step(failing_step).await.unwrap();

    // Validation should fail
    let validation_result = workflow.validate().await;
    assert!(validation_result.is_err());
    assert!(validation_result
        .unwrap_err()
        .to_string()
        .contains("empty tool name"));
}

/// Test workflow continue-on-error strategy
#[tokio::test]
async fn test_workflow_continue_strategy() {
    let mut config = BasicWorkflowConfig::default();
    config.continue_on_error = true; // Continue on error
    let error_strategy = BasicErrorStrategy::Continue;

    let mut workflow = BasicSequentialWorkflow::with_error_strategy(
        "continue_workflow".to_string(),
        config,
        error_strategy,
    );

    // Add multiple steps - even if one fails, others should continue
    let steps = vec![
        ("step1", "calculator"),
        ("step2", "json_processor"),
        ("step3", "file_operations"),
    ];

    for (name, tool) in steps {
        let step = BasicWorkflowStep::new(
            name.to_string(),
            BasicStepType::Tool {
                tool_name: tool.to_string(),
                parameters: serde_json::json!({}),
            },
        );
        workflow.add_step(step).await.unwrap();
    }

    let results = workflow.execute().await.unwrap();
    assert_eq!(results.len(), 3);
    assert!(results.iter().all(|r| r.success)); // All should succeed with mock execution
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Completed
    );
}

/// Test workflow step timeout functionality
#[tokio::test]
async fn test_workflow_step_timeout() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("timeout_workflow".to_string(), config);

    // Add a step with very short timeout
    let timeout_step = BasicWorkflowStep::new(
        "timeout_step".to_string(),
        BasicStepType::Custom {
            function_name: "slow_function".to_string(),
            parameters: serde_json::json!({}),
        },
    )
    .with_timeout(Duration::from_millis(1)); // Very short timeout

    workflow.add_step(timeout_step).await.unwrap();

    let results = workflow.execute().await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(!results[0].success); // Should fail due to timeout
    assert!(results[0].error.as_ref().unwrap().contains("timed out"));
}

/// Test workflow execution statistics
#[tokio::test]
async fn test_workflow_execution_statistics() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("stats_workflow".to_string(), config);

    // Add several steps
    for i in 1..=5 {
        let step = BasicWorkflowStep::new(
            format!("step_{}", i),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": format!("{} + {}", i, i)}),
            },
        );
        workflow.add_step(step).await.unwrap();
    }

    let _results = workflow.execute().await.unwrap();
    let stats = workflow.get_stats().await.unwrap();

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
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("reset_workflow".to_string(), config);

    // Set shared data and execute
    workflow
        .set_shared_data("test_key".to_string(), serde_json::json!("test_value"))
        .await
        .unwrap();

    let step = BasicWorkflowStep::new(
        "test_step".to_string(),
        BasicStepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({}),
        },
    );
    workflow.add_step(step).await.unwrap();

    let _results = workflow.execute().await.unwrap();
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Completed
    );

    // Reset workflow
    workflow.reset().await.unwrap();

    // Verify everything is reset
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Pending
    );
    let shared_data = workflow.get_shared_data("test_key").await.unwrap();
    assert_eq!(shared_data, None);
    let results = workflow.get_results().await.unwrap();
    assert!(results.is_empty());
}

/// Test workflow step removal
#[tokio::test]
async fn test_workflow_step_removal() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("removal_workflow".to_string(), config);

    let step1 = BasicWorkflowStep::new(
        "step1".to_string(),
        BasicStepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: serde_json::json!({}),
        },
    );
    let step2 = BasicWorkflowStep::new(
        "step2".to_string(),
        BasicStepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({}),
        },
    );

    let step1_id = step1.id;
    let step2_id = step2.id;

    workflow.add_step(step1).await.unwrap();
    workflow.add_step(step2).await.unwrap();
    assert_eq!(workflow.step_count(), 2);

    // Remove first step
    workflow.remove_step(step1_id).await.unwrap();
    assert_eq!(workflow.step_count(), 1);

    // Verify remaining step is step2
    let steps = workflow.get_steps().await.unwrap();
    assert_eq!(steps.len(), 1);
    assert_eq!(steps[0].id, step2_id);

    // Try to remove non-existent step
    let result = workflow.remove_step(step1_id).await;
    assert!(result.is_err());
}

/// Test workflow performance requirements (<50ms creation)
#[tokio::test]
async fn test_workflow_performance_creation() {
    let start = std::time::Instant::now();

    let config = BasicWorkflowConfig::default();
    let _workflow = BasicSequentialWorkflow::new("performance_test".to_string(), config);

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
    let config = BasicWorkflowConfig::default();
    let retry_strategy = BasicErrorStrategy::Retry {
        max_attempts: 3,
        backoff_ms: 10, // Short delay for test
    };

    let mut workflow = BasicSequentialWorkflow::with_error_strategy(
        "retry_workflow".to_string(),
        config,
        retry_strategy,
    );

    // Add step with retry configuration
    let retry_step = BasicWorkflowStep::new(
        "retry_step".to_string(),
        BasicStepType::Tool {
            tool_name: "calculator".to_string(), // This should succeed
            parameters: serde_json::json!({"expression": "2 + 2"}),
        },
    )
    .with_retry(2);

    workflow.add_step(retry_step).await.unwrap();

    let results = workflow.execute().await.unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].success);
    assert_eq!(results[0].retry_count, 0); // Should succeed on first try
}

/// Test workflow execution timeout (global)
#[tokio::test]
async fn test_workflow_global_timeout() {
    let mut config = BasicWorkflowConfig::default();
    config.max_execution_time = Some(Duration::from_millis(1)); // Very short timeout to test timeout behavior

    let mut workflow = BasicSequentialWorkflow::new("timeout_workflow".to_string(), config);

    // Add steps that might exceed global timeout
    for i in 1..=3 {
        let step = BasicWorkflowStep::new(
            format!("step_{}", i),
            BasicStepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({}),
            },
        );
        workflow.add_step(step).await.unwrap();
    }

    // Execute - should timeout due to very short global timeout
    let execution_result = workflow.execute().await;

    // Should fail due to timeout
    assert!(execution_result.is_err());
    let error = execution_result.unwrap_err();
    assert!(error.to_string().contains("exceeded maximum time limit"));
}

/// Test empty workflow validation
#[tokio::test]
async fn test_empty_workflow_validation() {
    let config = BasicWorkflowConfig::default();
    let workflow = BasicSequentialWorkflow::new("empty_workflow".to_string(), config);

    let validation_result = workflow.validate().await;
    assert!(validation_result.is_err());
    assert!(validation_result
        .unwrap_err()
        .to_string()
        .contains("no steps"));
}

/// Test workflow state snapshot functionality
#[tokio::test]
async fn test_workflow_state_snapshot() {
    let config = BasicWorkflowConfig::default();
    let workflow = BasicSequentialWorkflow::new("snapshot_workflow".to_string(), config);

    // Set some shared data
    workflow
        .set_shared_data("test_data".to_string(), serde_json::json!({"value": 42}))
        .await
        .unwrap();

    let snapshot = workflow.get_state_snapshot().await.unwrap();
    assert_eq!(snapshot.current_step, 0);
    assert!(snapshot.shared_data.contains_key("test_data"));
    assert_eq!(
        snapshot.shared_data["test_data"],
        serde_json::json!({"value": 42})
    );
}

/// Integration test: Complete data processing pipeline
#[tokio::test]
async fn test_complete_data_pipeline() {
    let config = BasicWorkflowConfig::default();
    let mut workflow = BasicSequentialWorkflow::new("data_pipeline".to_string(), config);

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

    for (name, tool, params) in steps {
        let step = BasicWorkflowStep::new(
            name.to_string(),
            BasicStepType::Tool {
                tool_name: tool.to_string(),
                parameters: params,
            },
        );
        workflow.add_step(step).await.unwrap();
    }

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
    let results = workflow.execute().await.unwrap();
    let execution_time = start_time.elapsed();

    // Verify pipeline execution
    assert_eq!(results.len(), 6);
    assert!(results.iter().all(|r| r.success));
    assert_eq!(
        workflow.status().await.unwrap(),
        BasicWorkflowStatus::Completed
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
    let stats = workflow.get_stats().await.unwrap();
    assert_eq!(stats.success_rate(), 100.0);
    assert_eq!(stats.total_retries, 0);

    println!("✅ Data pipeline completed successfully:");
    println!("   Duration: {:?}", execution_time);
    println!("   Steps: {}", stats.total_steps);
    println!("   Success Rate: {:.1}%", stats.success_rate());
}
