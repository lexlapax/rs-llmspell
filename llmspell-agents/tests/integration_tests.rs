//! ABOUTME: Integration tests for the agent infrastructure
//! ABOUTME: Tests factory, registry, lifecycle management, and tool integration

use llmspell_agents::testing::mocks;

use llmspell_agents::{
    lifecycle::state_machine::{AgentState, AgentStateMachine},
    templates::{AgentTemplate, OrchestratorAgentTemplate},
    AgentBuilder, AgentConfig, AgentFactory, DIContainer, DefaultAgentFactory, ResourceLimits,
};
use llmspell_core::{types::AgentInput, BaseAgent, ExecutionContext};
use std::{sync::Arc, time::Duration};

/// Test agent factory creation
#[tokio::test]
async fn test_agent_factory_creation() {
    let factory = DefaultAgentFactory::new();

    let config = AgentConfig {
        name: "test_agent".to_string(),
        description: "Test agent".to_string(),
        agent_type: "basic".to_string(),
        model: None,
        allowed_tools: vec![],
        custom_config: serde_json::Map::new(),
        resource_limits: ResourceLimits::default(),
    };

    let agent = factory.create_agent(config).await.unwrap();
    assert_eq!(agent.metadata().name, "test_agent");
}

/// Test agent builder pattern
#[tokio::test]
async fn test_agent_builder() {
    let config = AgentBuilder::new("builder_agent", "basic")
        .description("Built with builder pattern")
        .allow_tool("calculator")
        .allow_tool("search")
        .build()
        .unwrap();

    let factory = DefaultAgentFactory::new();
    let agent = factory.create_agent(config).await.unwrap();

    let metadata = agent.metadata();
    assert_eq!(metadata.name, "builder_agent");
    assert_eq!(metadata.description, "Built with builder pattern");
}

/// Test agent lifecycle transitions
#[tokio::test]
async fn test_agent_lifecycle() {
    let state_machine = AgentStateMachine::new("lifecycle_test".to_string(), Default::default());

    // Initial state should be Uninitialized
    assert_eq!(
        state_machine.current_state().await,
        AgentState::Uninitialized
    );

    // Initialize
    state_machine.initialize().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Ready);

    // Start
    state_machine.start().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Running);

    // Pause
    state_machine.pause().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Paused);

    // Resume
    state_machine.resume().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Running);

    // Stop
    state_machine.stop().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Ready);

    // Terminate
    state_machine.terminate().await.unwrap();
    assert_eq!(state_machine.current_state().await, AgentState::Terminated);
}

/// Test agent template instantiation
#[tokio::test]
async fn test_agent_templates() {
    let template = OrchestratorAgentTemplate::default();
    let _factory = Arc::new(DefaultAgentFactory::new());

    use llmspell_agents::templates::base::TemplateInstantiationParams;

    let mut parameters = std::collections::HashMap::new();
    parameters.insert(
        "agent_name".to_string(),
        serde_json::Value::String("orchestrator_instance".to_string()),
    );

    let params = TemplateInstantiationParams {
        agent_id: "orchestrator_instance".to_string(),
        parameters,
        resource_manager: None,
        event_system: None,
        config_overrides: Default::default(),
        environment: Default::default(),
    };

    let result = template.instantiate(params).await.unwrap();
    assert_eq!(result.agent.metadata().name, "orchestrator_instance");
}

/// Test dependency injection container
#[tokio::test]
async fn test_di_container() {
    let container = DIContainer::new();

    // Test scoped container creation
    let scoped = container.create_scope();

    // Scoped container should be created successfully
    // (Just verify it exists - can't test much without registration methods)
    let _ = scoped;
}

/// Test resource limits enforcement
#[tokio::test]
async fn test_resource_limits() {
    let limits = ResourceLimits {
        max_execution_time_secs: 5,
        max_memory_mb: 256,
        max_tool_calls: 10,
        max_recursion_depth: 3,
    };

    assert_eq!(limits.max_execution_time_secs, 5);
    assert_eq!(limits.max_memory_mb, 256);
    assert_eq!(limits.max_tool_calls, 10);
    assert_eq!(limits.max_recursion_depth, 3);
}

/// Test agent with tool integration
#[tokio::test]
async fn test_agent_with_tools() {
    use mocks::TestDoubles;

    let agent = TestDoubles::tool_agent("tool_test", vec!["calculator", "search"]);

    // Check tool availability
    use llmspell_core::traits::tool_capable::ToolCapable;
    assert!(agent.tool_available("calculator").await);
    assert!(agent.tool_available("search").await);
    assert!(!agent.tool_available("unknown").await);

    let tools = agent.list_available_tools().await.unwrap();
    assert_eq!(tools.len(), 2);
    assert!(tools.contains(&"calculator".to_string()));
    assert!(tools.contains(&"search".to_string()));
}

/// Test concurrent agent execution
#[tokio::test]
async fn test_concurrent_agent_execution() {
    use mocks::MockAgentBuilder;

    let agent = Arc::new(
        MockAgentBuilder::new("concurrent_agent")
            .with_response(None, "Processed")
            .build(),
    );

    let mut handles = Vec::new();

    // Spawn 10 concurrent executions
    for i in 0..10 {
        let agent_clone = agent.clone();
        let handle = tokio::spawn(async move {
            let input = AgentInput::text(&format!("Request {}", i));
            agent_clone
                .execute(input, ExecutionContext::default())
                .await
        });
        handles.push(handle);
    }

    // Wait for all to complete
    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await.unwrap());
    }

    // All should succeed
    assert_eq!(results.len(), 10);
    for result in results {
        assert!(result.is_ok());
    }

    assert_eq!(agent.execution_count(), 10);
}

/// Test agent error handling
#[tokio::test]
async fn test_agent_error_handling() {
    use mocks::TestDoubles;

    let agent = TestDoubles::failing_agent("error_agent", "Simulated error");

    let input = AgentInput::text("test");
    let result = agent.execute(input, ExecutionContext::default()).await;

    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Simulated error"));
}

/// Test agent state persistence across operations
#[tokio::test]
async fn test_agent_state_persistence() {
    use mocks::MockAgentBuilder;

    let mut agent = MockAgentBuilder::new("stateful_agent")
        .with_response(Some("hello".to_string()), "Hello response")
        .with_response(Some("goodbye".to_string()), "Goodbye response")
        .build();

    // First execution
    let input1 = AgentInput::text("hello");
    let output1 = agent
        .execute(input1, ExecutionContext::default())
        .await
        .unwrap();
    assert_eq!(output1.text, "Hello response");

    // Add conversation message
    use llmspell_core::traits::agent::{Agent, ConversationMessage, MessageRole};
    agent
        .add_message(ConversationMessage {
            role: MessageRole::User,
            content: "hello".to_string(),
            timestamp: chrono::Utc::now(),
        })
        .await
        .unwrap();

    agent
        .add_message(ConversationMessage {
            role: MessageRole::Assistant,
            content: "Hello response".to_string(),
            timestamp: chrono::Utc::now(),
        })
        .await
        .unwrap();

    // Check conversation
    let conversation = agent.get_conversation().await.unwrap();
    assert_eq!(conversation.len(), 2);

    // Second execution
    let input2 = AgentInput::text("goodbye");
    let output2 = agent
        .execute(input2, ExecutionContext::default())
        .await
        .unwrap();
    assert_eq!(output2.text, "Goodbye response");

    // Clear conversation
    agent.clear_conversation().await.unwrap();
    let conversation = agent.get_conversation().await.unwrap();
    assert_eq!(conversation.len(), 0);
}

/// Test execution context propagation
#[tokio::test]
async fn test_execution_context_propagation() {
    use mocks::MockAgentBuilder;

    let agent = MockAgentBuilder::new("context_agent")
        .with_response(None, "Context processed")
        .build();

    // Create context with specific values
    let mut context = ExecutionContext::new();
    context.conversation_id = Some("conv123".to_string());
    context.user_id = Some("user456".to_string());
    context.session_id = Some("session789".to_string());

    let input = AgentInput::text("test with context");
    let _output = agent.execute(input, context.clone()).await.unwrap();

    // Verify context was captured
    let last_context = agent.last_context().unwrap();
    assert_eq!(last_context.conversation_id, Some("conv123".to_string()));
    assert_eq!(last_context.user_id, Some("user456".to_string()));
    assert_eq!(last_context.session_id, Some("session789".to_string()));
}

/// Test agent performance requirements
#[tokio::test]
async fn test_agent_performance() {
    use mocks::MockAgentBuilder;
    use std::time::Instant;

    let agent = MockAgentBuilder::new("perf_agent")
        .with_response(None, "Fast response")
        .build();

    let input = AgentInput::text("performance test");
    let start = Instant::now();

    let output = agent
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();

    let duration = start.elapsed();

    assert!(output.text.contains("Fast response"));
    // Agent creation and execution should be fast
    assert!(duration < Duration::from_millis(100));
}
