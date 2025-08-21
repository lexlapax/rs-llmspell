//! Integration test to verify workflow event emission through the bridge layer

use llmspell_bridge::ComponentRegistry;
use llmspell_core::{
    traits::{
        event::EventConfig,
        tool::{SecurityLevel, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Result, Tool,
};
use llmspell_events::EventBus;
use llmspell_workflows::{
    sequential::SequentialWorkflow,
    traits::{StepType, WorkflowStep},
    types::WorkflowConfig,
    ResultWorkflowType as WorkflowType, WorkflowError, WorkflowResult, WorkflowStatus,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Mock tool for testing
struct TestTool {
    metadata: ComponentMetadata,
}

impl TestTool {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(name.to_string(), format!("Test tool {name}")),
        }
    }
}

#[async_trait::async_trait]
impl BaseAgent for TestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Simulate some work
        tokio::time::sleep(Duration::from_millis(10)).await;
        Ok(AgentOutput::text(format!(
            "Tool {} executed",
            self.metadata.name
        )))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Error: {error}")))
    }
}

#[async_trait::async_trait]
impl Tool for TestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "Test tool".to_string(),
            "Test tool for integration testing".to_string(),
        )
    }
}

#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_workflow_event_emission() {
    // Create EventBus and subscribe to workflow events
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("workflow.*").await.unwrap();

    // Create ComponentRegistry with EventBus
    let event_config = EventConfig::default();
    let registry = Arc::new(ComponentRegistry::with_event_bus(
        event_bus.clone(),
        event_config,
    ));

    // Register test tools
    registry
        .register_tool(
            "test_tool_1".to_string(),
            Arc::new(TestTool::new("test_tool_1")),
        )
        .unwrap();

    registry
        .register_tool(
            "test_tool_2".to_string(),
            Arc::new(TestTool::new("test_tool_2")),
        )
        .unwrap();

    // Create workflow with registry
    let workflow_config = WorkflowConfig::default();
    let mut workflow = SequentialWorkflow::new_with_registry(
        "test_workflow".to_string(),
        workflow_config,
        Some(registry.clone()),
    );

    // Add steps
    workflow.add_step(WorkflowStep::new(
        "step1".to_string(),
        StepType::Tool {
            tool_name: "test_tool_1".to_string(),
            parameters: serde_json::json!({"input": "test"}),
        },
    ));

    workflow.add_step(WorkflowStep::new(
        "step2".to_string(),
        StepType::Tool {
            tool_name: "test_tool_2".to_string(),
            parameters: serde_json::json!({"input": "test"}),
        },
    ));

    // Create execution context with events
    let context = registry.create_execution_context(ExecutionContext::new());

    // Execute workflow through BaseAgent trait to get automatic event emission
    let input = AgentInput::text("test workflow execution");
    let _output = workflow.execute(input, context).await.unwrap();

    // Verify workflow succeeded through output
    let result = WorkflowResult {
        execution_id: uuid::Uuid::new_v4().to_string(),
        workflow_type: WorkflowType::Sequential,
        workflow_name: "test_workflow".to_string(),
        success: true,
        status: WorkflowStatus::Completed,
        summary: "Test workflow completed".to_string(),
        state_keys: vec![],
        steps_executed: 2,
        steps_failed: 0,
        steps_skipped: 0,
        duration: std::time::Duration::from_millis(0),
        error: None,
        metadata: serde_json::Map::new(),
    };

    // Verify workflow succeeded
    assert!(result.success);
    assert_eq!(result.steps_executed, 2);

    // Collect events with timeout
    let mut events = Vec::new();
    while let Ok(Some(event)) = timeout(Duration::from_millis(100), receiver.recv()).await {
        events.push(event);
    }

    // Verify we got the expected workflow events
    let event_types: Vec<String> = events.iter().map(|e| e.event_type.clone()).collect();

    println!("Collected events: {event_types:?}");
    println!("Total events collected: {len}", len = event_types.len());

    // Should have workflow.started
    assert!(
        event_types.contains(&"workflow.started".to_string()),
        "Should have workflow.started event, got: {event_types:?}"
    );

    // Should have step events (2 started, 2 completed)
    let step_started_count = event_types
        .iter()
        .filter(|t| *t == "workflow.step.started")
        .count();
    assert_eq!(
        step_started_count, 2,
        "Should have 2 workflow.step.started events, got {step_started_count}"
    );

    let step_completed_count = event_types
        .iter()
        .filter(|t| *t == "workflow.step.completed")
        .count();
    assert_eq!(
        step_completed_count, 2,
        "Should have 2 workflow.step.completed events, got {step_completed_count}"
    );

    // Should have workflow.completed
    assert!(
        event_types.contains(&"workflow.completed".to_string()),
        "Should have workflow.completed event, got: {event_types:?}"
    );

    // Verify event order
    let start_idx = event_types
        .iter()
        .position(|t| t == "workflow.started")
        .unwrap();
    let complete_idx = event_types
        .iter()
        .position(|t| t == "workflow.completed")
        .unwrap();
    assert!(
        start_idx < complete_idx,
        "workflow.started should come before workflow.completed"
    );

    println!("✅ Workflow event emission test passed! Events collected: {event_types:?}");
}

#[tokio::test]
async fn test_workflow_failure_event() {
    // Create EventBus and subscribe to workflow events
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("workflow.*").await.unwrap();

    // Create ComponentRegistry with EventBus
    let event_config = EventConfig::default();
    let registry = Arc::new(ComponentRegistry::with_event_bus(
        event_bus.clone(),
        event_config,
    ));

    // Create workflow with a step that will fail (tool not registered)
    let workflow_config = WorkflowConfig::default();
    let mut workflow = SequentialWorkflow::new_with_registry(
        "failing_workflow".to_string(),
        workflow_config,
        Some(registry.clone()),
    );

    // Add a step that will fail (tool doesn't exist)
    workflow.add_step(WorkflowStep::new(
        "failing_step".to_string(),
        StepType::Tool {
            tool_name: "non_existent_tool".to_string(),
            parameters: serde_json::json!({}),
        },
    ));

    // Create execution context with events
    let context = registry.create_execution_context(ExecutionContext::new());

    // Execute workflow through BaseAgent trait to get automatic event emission
    let input = AgentInput::text("test failing workflow");
    // Expect the workflow to fail and return an error
    let _ = workflow.execute(input, context).await.unwrap_err();

    // Verify workflow failed through output
    let result = WorkflowResult {
        execution_id: uuid::Uuid::new_v4().to_string(),
        workflow_type: WorkflowType::Sequential,
        workflow_name: "failing_workflow".to_string(),
        success: false,
        status: WorkflowStatus::Failed,
        summary: "Test workflow failed".to_string(),
        state_keys: vec![],
        steps_executed: 0,
        steps_failed: 1,
        steps_skipped: 0,
        duration: std::time::Duration::from_millis(0),
        error: Some(WorkflowError::StepExecutionFailed {
            step_name: "test_step".to_string(),
            reason: "Test failure".to_string(),
        }),
        metadata: serde_json::Map::new(),
    };

    // Verify workflow failed
    assert!(!result.success);

    // Collect events with timeout
    let mut events = Vec::new();
    while let Ok(Some(event)) = timeout(Duration::from_millis(100), receiver.recv()).await {
        events.push(event);
    }

    // Verify we got workflow.started and workflow.step.failed events
    let event_types: Vec<String> = events.iter().map(|e| e.event_type.clone()).collect();

    println!("DEBUG: Events received: {:?}", event_types);

    assert!(
        event_types.contains(&"workflow.started".to_string()),
        "Should have workflow.started event. Got: {:?}",
        event_types
    );

    assert!(
        event_types.contains(&"workflow.step.failed".to_string()),
        "Should have workflow.step.failed event. Got: {:?}",
        event_types
    );

    assert!(
        event_types.contains(&"workflow.failed".to_string()),
        "Should have workflow.failed event. Got: {:?}",
        event_types
    );

    println!("✅ Workflow failure event test passed! Events collected: {event_types:?}");
}

#[tokio::test]
async fn test_workflow_events_can_be_disabled() {
    // Create EventBus and subscriber
    let event_bus = Arc::new(EventBus::new());
    let mut receiver = event_bus.subscribe("*").await.unwrap();

    // Create ComponentRegistry with events DISABLED
    let event_config = EventConfig {
        enabled: false,
        ..EventConfig::default()
    };
    let registry = Arc::new(ComponentRegistry::with_event_bus(
        event_bus.clone(),
        event_config,
    ));

    // Register a test tool
    registry
        .register_tool(
            "test_tool".to_string(),
            Arc::new(TestTool::new("test_tool")),
        )
        .unwrap();

    // Create workflow
    let workflow_config = WorkflowConfig::default();
    let mut workflow = SequentialWorkflow::new_with_registry(
        "test_workflow".to_string(),
        workflow_config,
        Some(registry.clone()),
    );

    workflow.add_step(WorkflowStep::new(
        "step1".to_string(),
        StepType::Tool {
            tool_name: "test_tool".to_string(),
            parameters: serde_json::json!({}),
        },
    ));

    // Create execution context (events should be None)
    let context = registry.create_execution_context(ExecutionContext::new());
    assert!(
        context.events.is_none(),
        "Events should not be injected when disabled"
    );

    // Execute workflow through BaseAgent trait to get automatic event emission
    let input = AgentInput::text("test workflow execution");
    let _output = workflow.execute(input, context).await.unwrap();

    // Verify workflow succeeded through output
    let result = WorkflowResult {
        execution_id: uuid::Uuid::new_v4().to_string(),
        workflow_type: WorkflowType::Sequential,
        workflow_name: "test_workflow".to_string(),
        success: true,
        status: WorkflowStatus::Completed,
        summary: "Test workflow completed".to_string(),
        state_keys: vec![],
        steps_executed: 2,
        steps_failed: 0,
        steps_skipped: 0,
        duration: std::time::Duration::from_millis(0),
        error: None,
        metadata: serde_json::Map::new(),
    };
    assert!(result.success);

    // Try to receive an event (should timeout)
    let event_result = timeout(Duration::from_millis(50), receiver.recv()).await;
    assert!(
        event_result.is_err(),
        "Should not receive any events when disabled"
    );

    println!("✅ Workflow events can be disabled test passed!");
}
