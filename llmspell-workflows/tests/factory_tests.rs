// ABOUTME: Integration tests for workflow factory functionality
// ABOUTME: Tests factory patterns for creating workflow instances

use llmspell_core::{execution_context::ExecutionContext, types::AgentInput};
use llmspell_workflows::{
    adapters::WorkflowOutputAdapter,
    factory::{
        DefaultWorkflowFactory, TemplateWorkflowFactory, WorkflowFactory, WorkflowParams,
        WorkflowType,
    },
    types::{WorkflowConfig, WorkflowOutput},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

#[tokio::test]
async fn test_factory_sequential_workflow_creation() {
    let factory = DefaultWorkflowFactory::new();

    let params = WorkflowParams {
        name: "test_sequential".to_string(),
        workflow_type: WorkflowType::Sequential,
        config: WorkflowConfig {
            max_execution_time: Some(Duration::from_secs(60)),
            ..Default::default()
        },
        type_config: json!({}),
    };

    let workflow = factory.create_workflow(params).await.unwrap();
    assert_eq!(workflow.metadata().name, "test_sequential");

    // Test execution through BaseAgent interface
    let input = AgentInput::text("test input");
    let context = ExecutionContext::with_conversation("test-conv".to_string());
    let result = workflow.execute(input, context).await;

    // Workflow without steps should fail to execute
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("without steps") || err.to_string().contains("no steps"));
    }
}

#[tokio::test]
async fn test_factory_parallel_workflow_creation() {
    let factory = DefaultWorkflowFactory::new();

    let params = WorkflowParams {
        name: "test_parallel".to_string(),
        workflow_type: WorkflowType::Parallel,
        config: WorkflowConfig::default(),
        type_config: json!({
            "max_concurrency": 4,
            "fail_fast": false,
            "continue_on_optional_failure": true,
        }),
    };

    let workflow = factory.create_workflow(params).await.unwrap();
    assert_eq!(workflow.metadata().name, "test_parallel");

    // Execute the workflow
    let input = AgentInput::text("test parallel");
    let context = ExecutionContext::with_conversation("test-conv".to_string());
    let result = workflow.execute(input, context).await;

    // Workflow without steps should fail to execute
    assert!(result.is_err());
    if let Err(err) = result {
        let err_msg = err.to_string();
        assert!(
            err_msg.contains("without steps")
                || err_msg.contains("no steps")
                || err_msg.contains("branches")
                || err_msg.contains("empty"),
            "Unexpected error message: {}",
            err_msg
        );
    }
}

#[tokio::test]
async fn test_factory_conditional_workflow_creation() {
    let factory = DefaultWorkflowFactory::new();

    let params = WorkflowParams {
        name: "test_conditional".to_string(),
        workflow_type: WorkflowType::Conditional,
        config: WorkflowConfig::default(),
        type_config: json!({
            "execute_all_matching": false,
            "execute_default_on_no_match": true,
        }),
    };

    let workflow = factory.create_workflow(params).await.unwrap();
    assert_eq!(workflow.metadata().name, "test_conditional");
}

#[tokio::test]
async fn test_factory_loop_workflow_creation() {
    let factory = DefaultWorkflowFactory::new();

    let params = WorkflowParams {
        name: "test_loop".to_string(),
        workflow_type: WorkflowType::Loop,
        config: WorkflowConfig::default(),
        type_config: json!({
            "iterator": {
                "type": "range",
                "start": 0,
                "end": 5,
                "step": 1
            },
            "body": [],
            "break_conditions": [],
            "aggregation": "collect_all",
            "continue_on_error": false,
        }),
    };

    let workflow = factory.create_workflow(params).await.unwrap();
    assert_eq!(workflow.metadata().name, "test_loop");
}

#[tokio::test]
async fn test_factory_invalid_config() {
    let factory = DefaultWorkflowFactory::new();

    // Test with invalid parallel config
    let params = WorkflowParams {
        name: "test_invalid".to_string(),
        workflow_type: WorkflowType::Parallel,
        config: WorkflowConfig::default(),
        type_config: json!({
            "invalid_field": "value",
        }),
    };

    // Should fail due to missing required fields
    let result = factory.create_workflow(params).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_template_factory_default_templates() {
    let factory = TemplateWorkflowFactory::new();

    // Verify default templates are available
    let templates = factory.list_templates();
    assert!(templates.contains(&"data_pipeline".to_string()));
    assert!(templates.contains(&"parallel_analysis".to_string()));
    assert!(templates.contains(&"retry_with_backoff".to_string()));
    assert!(templates.contains(&"conditional_router".to_string()));
}

#[tokio::test]
async fn test_template_factory_create_from_template() {
    let factory = TemplateWorkflowFactory::new();

    // Create workflow from data_pipeline template
    let workflow = factory
        .create_from_template("data_pipeline", "my_etl_pipeline".to_string())
        .await
        .unwrap();

    assert_eq!(workflow.metadata().name, "my_etl_pipeline");

    // Verify it uses the template configuration
    let input = AgentInput::text("process data");
    let context = ExecutionContext::with_conversation("test-conv".to_string());
    let result = workflow.execute(input, context).await;

    // Workflow without steps should fail to execute
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("without steps") || err.to_string().contains("no steps"));
    }
}

#[tokio::test]
async fn test_template_factory_invalid_template() {
    let factory = TemplateWorkflowFactory::new();

    // Try to create from non-existent template
    let result = factory
        .create_from_template("non_existent_template", "workflow".to_string())
        .await;

    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("not found"));
    }
}

#[tokio::test]
async fn test_factory_workflow_execution_with_adapters() {
    let factory = DefaultWorkflowFactory::new();

    // Create a sequential workflow
    let params = WorkflowParams {
        name: "test_adapters".to_string(),
        workflow_type: WorkflowType::Sequential,
        config: WorkflowConfig::default(),
        type_config: json!({}),
    };

    let workflow = factory.create_workflow(params).await.unwrap();

    // Execute with agent input
    let mut agent_input = AgentInput::text("Process this data");
    agent_input
        .parameters
        .insert("timeout_ms".to_string(), json!(5000));

    let context = ExecutionContext::with_conversation("conv-123".to_string());
    let result = workflow.execute(agent_input, context).await;

    // Workflow without steps should fail to execute
    assert!(result.is_err());
    if let Err(err) = result {
        assert!(err.to_string().contains("without steps") || err.to_string().contains("no steps"));
    }
}

#[test]
fn test_workflow_type_serialization() {
    // Test that workflow types serialize correctly
    let seq_type = WorkflowType::Sequential;
    let serialized = serde_json::to_string(&seq_type).unwrap();
    assert_eq!(serialized, r#""sequential""#);

    let par_type = WorkflowType::Parallel;
    let serialized = serde_json::to_string(&par_type).unwrap();
    assert_eq!(serialized, r#""parallel""#);

    // Test deserialization
    let deserialized: WorkflowType = serde_json::from_str(r#""conditional""#).unwrap();
    assert_eq!(deserialized, WorkflowType::Conditional);
}

#[test]
fn test_default_configs_for_types() {
    let factory = DefaultWorkflowFactory::new();

    // Sequential should have default config
    let seq_config = factory.default_config(&WorkflowType::Sequential);
    assert!(!seq_config.continue_on_error);
    assert_eq!(seq_config.max_retry_attempts, 3);

    // Parallel should continue on error
    let par_config = factory.default_config(&WorkflowType::Parallel);
    assert!(par_config.continue_on_error);

    // Loop should have reduced retries
    let loop_config = factory.default_config(&WorkflowType::Loop);
    assert_eq!(loop_config.max_retry_attempts, 1);
}

#[tokio::test]
async fn test_workflow_output_adapter_integration() {
    // Test adapter conversion with real workflow output
    let mut context = HashMap::new();
    context.insert("processed_items".to_string(), json!(42));
    context.insert("status".to_string(), json!("complete"));

    let workflow_output = WorkflowOutput {
        output: json!({
            "result": "Data processed successfully",
            "items": ["item1", "item2", "item3"]
        }),
        success: true,
        duration: Duration::from_millis(1500),
        steps_executed: 5,
        steps_failed: 0,
        final_context: context,
        error: None,
    };

    let agent_output = WorkflowOutputAdapter::to_agent_output(workflow_output.clone());

    // Verify conversion
    assert_eq!(agent_output.text, "Data processed successfully");
    assert_eq!(agent_output.metadata.execution_time_ms, Some(1500));
    assert_eq!(
        agent_output.metadata.extra.get("workflow_success").unwrap(),
        &json!(true)
    );
    assert_eq!(
        agent_output.metadata.extra.get("steps_executed").unwrap(),
        &json!(5)
    );
    assert_eq!(
        agent_output
            .metadata
            .extra
            .get("context_processed_items")
            .unwrap(),
        &json!(42)
    );
    assert_eq!(
        agent_output.metadata.extra.get("context_status").unwrap(),
        &json!("complete")
    );

    // Verify structured output is preserved
    let structured = agent_output
        .metadata
        .extra
        .get("structured_output")
        .unwrap();
    assert!(structured.get("items").is_some());
}
