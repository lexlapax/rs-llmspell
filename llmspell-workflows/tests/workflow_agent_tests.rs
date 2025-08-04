//! ABOUTME: Integration tests for workflows as agents
//! ABOUTME: Tests BaseAgent interface implementation for all workflow types

use llmspell_core::{
    execution_context::ExecutionContext, traits::base_agent::BaseAgent, types::agent_io::AgentInput,
};
use llmspell_workflows::{
    conditional::{ConditionalBranch, ConditionalWorkflowBuilder},
    conditions::Condition,
    r#loop::LoopWorkflowBuilder,
    traits::{StepType, WorkflowStep},
    ParallelBranch, ParallelWorkflowBuilder, SequentialWorkflowBuilder,
};
use serde_json::json;

#[tokio::test]
async fn test_sequential_workflow_as_agent() {
    // Create a simple sequential workflow
    let workflow = SequentialWorkflowBuilder::new("test_pipeline".to_string())
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": "2 + 2"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "step2".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: json!({"operation": "validate"}),
            },
        ))
        .build();

    // Execute as agent
    let input = AgentInput::text("Process data through pipeline");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Verify AgentOutput
    assert!(result.text.contains("Sequential workflow"));
    assert!(result.text.contains("completed"));
    assert_eq!(
        result.metadata.extra.get("workflow_type").unwrap(),
        &json!("sequential")
    );
    assert_eq!(result.metadata.extra.get("total_steps").unwrap(), &json!(2));
}

#[tokio::test]
async fn test_parallel_workflow_as_agent() {
    // Create parallel branches
    let branch1 = ParallelBranch::new("branch1".to_string()).add_step(WorkflowStep::new(
        "api_call".to_string(),
        StepType::Tool {
            tool_name: "http_request".to_string(),
            parameters: json!({"url": "https://api.example.com"}),
        },
    ));

    let branch2 = ParallelBranch::new("branch2".to_string()).add_step(WorkflowStep::new(
        "process".to_string(),
        StepType::Tool {
            tool_name: "data_processor".to_string(),
            parameters: json!({"operation": "transform"}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("parallel_test")
        .add_branch(branch1)
        .add_branch(branch2)
        .build()
        .unwrap();

    // Execute as agent
    let input = AgentInput::text("Run parallel operations");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Verify AgentOutput
    assert!(result.text.contains("Parallel workflow"));
    assert!(result.metadata.execution_time_ms.is_some());
    assert_eq!(
        result.metadata.extra.get("workflow_type").unwrap(),
        &json!("parallel")
    );
}

#[tokio::test]
async fn test_conditional_workflow_as_agent() {
    // Create conditional branches
    let branch1 = ConditionalBranch::new(
        "process_csv".to_string(),
        Condition::SharedDataEquals {
            key: "file_type".to_string(),
            expected_value: json!("csv"),
        },
    )
    .with_step(WorkflowStep::new(
        "csv_parse".to_string(),
        StepType::Tool {
            tool_name: "csv_parser".to_string(),
            parameters: json!({"delimiter": ","}),
        },
    ));

    let default_branch =
        ConditionalBranch::default("default".to_string()).with_step(WorkflowStep::new(
            "text_parse".to_string(),
            StepType::Tool {
                tool_name: "text_parser".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ConditionalWorkflowBuilder::new("conditional_test".to_string())
        .add_branch(branch1)
        .add_branch(default_branch)
        .build();

    // Execute as agent
    let input = AgentInput::text("Process file based on type");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Verify AgentOutput
    assert!(result.text.contains("Conditional workflow"));
    assert_eq!(
        result.metadata.extra.get("workflow_type").unwrap(),
        &json!("conditional")
    );
}

#[tokio::test]
async fn test_loop_workflow_as_agent() {
    // Create a loop workflow
    let workflow = LoopWorkflowBuilder::new("loop_test")
        .with_range(0, 3, 1) // Loop 3 times
        .add_step(WorkflowStep::new(
            "process_item".to_string(),
            StepType::Tool {
                tool_name: "item_processor".to_string(),
                parameters: json!({"operation": "increment"}),
            },
        ))
        .build()
        .unwrap();

    // Execute as agent
    let input = AgentInput::text("Process items in loop");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Verify AgentOutput
    assert!(result.text.contains("Loop workflow"));
    assert!(result.metadata.execution_time_ms.is_some());
    assert_eq!(
        result.metadata.extra.get("workflow_type").unwrap(),
        &json!("loop")
    );
}

#[tokio::test]
async fn test_workflow_with_parameters() {
    // Create workflow
    let workflow = SequentialWorkflowBuilder::new("param_test".to_string())
        .add_step(WorkflowStep::new(
            "process".to_string(),
            StepType::Tool {
                tool_name: "data_processor".to_string(),
                parameters: json!({"format": "json"}),
            },
        ))
        .build();

    // Execute with parameters
    let mut input = AgentInput::text("Process with parameters");
    input
        .parameters
        .insert("timeout_ms".to_string(), json!(5000));
    input.parameters.insert("max_retries".to_string(), json!(3));

    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Verify parameters were handled
    assert!(result.text.contains("completed"));
    assert!(result.metadata.execution_time_ms.unwrap() < 5000);
}

#[tokio::test]
async fn test_workflow_error_handling() {
    // Create workflow with invalid step
    let workflow = SequentialWorkflowBuilder::new("error_test".to_string())
        .add_step(WorkflowStep::new(
            "invalid_step".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // Invalid tool name
                parameters: json!({}),
            },
        ))
        .build();

    // Execute as agent
    let input = AgentInput::text("Test error handling");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await.unwrap();

    // Should handle error gracefully
    assert!(result.text.contains("failed"));
    assert_eq!(
        result.metadata.extra.get("workflow_type").unwrap(),
        &json!("sequential")
    );
}

#[tokio::test]
async fn test_workflow_metadata_preservation() {
    // Create workflow
    let workflow = SequentialWorkflowBuilder::new("metadata_test".to_string())
        .add_step(WorkflowStep::new(
            "step1".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": "10 / 2"}),
            },
        ))
        .build();

    // Execute with context
    let input = AgentInput::text("Test metadata");
    let context = ExecutionContext::with_conversation("conv-123".to_string());
    let result = workflow.execute(input, context).await.unwrap();

    // Verify metadata
    assert!(result.metadata.execution_time_ms.is_some());
    assert!(result.metadata.extra.contains_key("workflow_name"));
    assert!(result.metadata.extra.contains_key("total_steps"));
    assert!(result.metadata.extra.contains_key("successful_steps"));
    assert!(result.metadata.extra.contains_key("failed_steps"));
    assert!(result.metadata.extra.contains_key("success_rate"));
}

#[tokio::test]
async fn test_workflow_validate_input() {
    // Create workflow
    let workflow = SequentialWorkflowBuilder::new("validation_test".to_string()).build();

    // Test with empty input
    let input = AgentInput::text("");
    let context = ExecutionContext::default();
    let result = workflow.execute(input, context).await;

    // Should fail validation
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("empty"));
}
