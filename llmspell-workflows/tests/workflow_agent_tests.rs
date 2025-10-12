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
    let result = workflow.execute(input, context).await;

    // Should return an error when workflow fails
    assert!(result.is_err());
    if let Err(e) = result {
        let error_string = e.to_string();
        // Check that error message contains the step name that failed
        assert!(error_string.contains("invalid_step") || error_string.contains("failed"));
        // Check that it mentions the actual error (empty tool name)
        assert!(
            error_string.contains("Tool name cannot be empty") || error_string.contains("empty")
        );
    }
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

#[tokio::test]
async fn test_sequential_workflow_collects_agent_outputs() {
    use llmspell_workflows::test_utils::MockStateAccess;
    use std::sync::Arc;

    // Create a sequential workflow with 2 agent steps
    let workflow = SequentialWorkflowBuilder::new("agent_pipeline".to_string())
        .add_step(WorkflowStep::new(
            "requirements_agent".to_string(),
            StepType::Agent {
                agent_id: "requirements_analyst".to_string(),
                input: "Gather requirements for the project".to_string(),
            },
        ))
        .add_step(WorkflowStep::new(
            "design_agent".to_string(),
            StepType::Agent {
                agent_id: "system_designer".to_string(),
                input: "Design the system architecture".to_string(),
            },
        ))
        .build();

    // Create context with mock state
    let mock_state = Arc::new(MockStateAccess::new());
    let context = ExecutionContext::default().with_state(mock_state.clone());

    // Execute workflow
    let input = AgentInput::text("Build a web application");
    let result = workflow.execute(input, context).await.unwrap();

    // Verify agent_outputs exists in metadata
    assert!(result.metadata.extra.contains_key("agent_outputs"));

    // Get agent outputs map
    let agent_outputs = result.metadata.extra.get("agent_outputs").unwrap();
    assert!(agent_outputs.is_object());

    let outputs_map = agent_outputs.as_object().unwrap();

    // Verify both agents' outputs are collected
    assert_eq!(outputs_map.len(), 2, "Should collect outputs from 2 agents");
    assert!(
        outputs_map.contains_key("requirements_analyst"),
        "Should contain requirements_analyst output"
    );
    assert!(
        outputs_map.contains_key("system_designer"),
        "Should contain system_designer output"
    );

    // Verify output format (should be JSON strings from mock agent execution)
    let req_output = outputs_map.get("requirements_analyst").unwrap();
    assert!(req_output.is_string(), "Agent output should be a string");
    let req_text = req_output.as_str().unwrap();
    assert!(
        req_text.contains("Agent") && req_text.contains("processed"),
        "Mock agent output should contain expected text"
    );

    // Verify state was used (check that outputs were written to state)
    let state_keys = mock_state.get_all_keys();
    assert!(
        !state_keys.is_empty(),
        "State should contain agent output keys"
    );

    // Verify convenience method works
    let outputs_via_method = llmspell_workflows::result::WorkflowResult::success(
        result
            .metadata
            .extra
            .get("execution_id")
            .unwrap()
            .as_str()
            .unwrap()
            .to_string(),
        llmspell_workflows::result::WorkflowType::Sequential,
        "agent_pipeline".to_string(),
        vec![],
        2,
        std::time::Duration::from_secs(1),
    )
    .with_metadata("agent_outputs".to_string(), agent_outputs.clone());

    assert!(outputs_via_method.agent_outputs().is_some());
    assert_eq!(outputs_via_method.agent_outputs().unwrap().len(), 2);
}

#[tokio::test]
async fn test_sequential_workflow_no_agents_no_outputs() {
    use llmspell_workflows::test_utils::MockStateAccess;
    use std::sync::Arc;

    // Create a sequential workflow with only tool steps (no agents)
    let workflow = SequentialWorkflowBuilder::new("tool_pipeline".to_string())
        .add_step(WorkflowStep::new(
            "calculator".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": "2 + 2"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "processor".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: json!({"operation": "validate"}),
            },
        ))
        .build();

    // Create context with mock state
    let mock_state = Arc::new(MockStateAccess::new());
    let context = ExecutionContext::default().with_state(mock_state);

    // Execute workflow
    let input = AgentInput::text("Process data without agents");
    let result = workflow.execute(input, context).await.unwrap();

    // Verify agent_outputs does NOT exist in metadata
    assert!(
        !result.metadata.extra.contains_key("agent_outputs"),
        "Should not have agent_outputs key when no agents are executed"
    );
}

#[tokio::test]
async fn test_workflow_result_convenience_methods() {
    use llmspell_workflows::result::{WorkflowResult, WorkflowType};
    use std::time::Duration;

    // Test 1: WorkflowResult with agent_outputs in metadata
    let mut result_with_agents = WorkflowResult::success(
        "exec-123".to_string(),
        WorkflowType::Sequential,
        "test-workflow".to_string(),
        vec![],
        3,
        Duration::from_secs(5),
    );

    // Add agent outputs to metadata
    let mut agent_outputs = serde_json::Map::new();
    agent_outputs.insert("agent1".to_string(), json!({"text": "Agent 1 output"}));
    agent_outputs.insert("agent2".to_string(), json!({"text": "Agent 2 output"}));
    result_with_agents.metadata.insert(
        "agent_outputs".to_string(),
        serde_json::Value::Object(agent_outputs),
    );

    // Test agent_outputs() method
    let outputs = result_with_agents.agent_outputs();
    assert!(outputs.is_some(), "agent_outputs() should return Some");
    assert_eq!(outputs.unwrap().len(), 2);

    // Test get_agent_output() method
    let agent1_output = result_with_agents.get_agent_output("agent1");
    assert!(
        agent1_output.is_some(),
        "get_agent_output() should return Some for existing agent"
    );
    assert_eq!(
        agent1_output
            .unwrap()
            .get("text")
            .unwrap()
            .as_str()
            .unwrap(),
        "Agent 1 output"
    );

    let missing_agent = result_with_agents.get_agent_output("nonexistent");
    assert!(
        missing_agent.is_none(),
        "get_agent_output() should return None for missing agent"
    );

    // Test 2: WorkflowResult without agent_outputs
    let result_without_agents = WorkflowResult::success(
        "exec-456".to_string(),
        WorkflowType::Sequential,
        "tool-only-workflow".to_string(),
        vec![],
        2,
        Duration::from_secs(3),
    );

    // Both methods should return None
    assert!(result_without_agents.agent_outputs().is_none());
    assert!(result_without_agents
        .get_agent_output("any_agent")
        .is_none());
}

#[tokio::test]
async fn test_all_workflow_types_collect_agent_outputs() {
    use llmspell_workflows::test_utils::MockStateAccess;
    use std::sync::Arc;

    // Helper to extract agent_outputs from result
    let get_agent_outputs = |result: &llmspell_core::types::AgentOutput| {
        result
            .metadata
            .extra
            .get("agent_outputs")
            .and_then(|v| v.as_object())
            .map(|m| m.len())
    };

    // Test Sequential workflow (validated working)
    let sequential = SequentialWorkflowBuilder::new("seq".to_string())
        .add_step(WorkflowStep::new(
            "agent1".to_string(),
            StepType::Agent {
                agent_id: "test_agent".to_string(),
                input: "test input".to_string(),
            },
        ))
        .build();

    let mock_state = Arc::new(MockStateAccess::new());
    let context = ExecutionContext::default().with_state(mock_state);
    let seq_result = sequential
        .execute(AgentInput::text("test"), context)
        .await
        .unwrap();
    assert_eq!(
        get_agent_outputs(&seq_result),
        Some(1),
        "Sequential should collect 1 agent output"
    );

    // Note: Parallel, Loop, and Conditional workflows have the collection code implemented
    // (parallel.rs:997-1018, loop.rs:1489-1505, conditional.rs:1324-1343) but require
    // registry-based agent execution for proper state propagation. Mock-based testing for
    // these workflow types is more complex and should be done in dedicated integration tests
    // with proper registry setup.
}
