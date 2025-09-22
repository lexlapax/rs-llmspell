//! ABOUTME: Integration tests for agent state persistence functionality
//! ABOUTME: Tests the complete lifecycle of saving and loading agent states

use anyhow::Result;
use llmspell_agents::{
    agents::basic::BasicAgent,
    builder::AgentBuilder,
    state::StatePersistence,
    testing::mocks::{MockAgent, MockAgentConfig, MockResponse},
};
use llmspell_core::types::OutputMetadata;
use llmspell_core::{
    traits::{agent::Agent, base_agent::BaseAgent},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_kernel::state::config::{PerformanceConfig, SledConfig};
use llmspell_kernel::state::{PersistenceConfig, StateManager, StorageBackendType};
use std::sync::Arc;
use tempfile::TempDir;
#[tokio::test]
async fn test_basic_agent_state_persistence() -> Result<()> {
    // Create a temporary directory for storage
    let temp_dir = TempDir::new()?;
    let storage_path = temp_dir.path().to_path_buf();

    // Create agent configuration
    let config = AgentBuilder::basic("test-agent")
        .description("Test agent for state persistence")
        .build()?;

    // Create two agents with same ID to test save/load
    let agent1 = BasicAgent::new(config.clone())?;
    let agent2 = BasicAgent::new(config)?;

    // Create state manager with persistent storage
    let state_manager = Arc::new(
        StateManager::with_backend(
            StorageBackendType::Sled(SledConfig {
                path: storage_path.join("agent_states"),
                cache_capacity: 1024 * 1024, // 1MB
                use_compression: true,
            }),
            PersistenceConfig {
                enabled: true,
                backend_type: StorageBackendType::Memory, // Overridden by with_backend
                flush_interval: std::time::Duration::from_secs(1),
                compression: true,
                encryption: None,
                backup_retention: std::time::Duration::from_secs(3600),
                backup: None,
                performance: PerformanceConfig::default(),
            },
        )
        .await?,
    );

    // Set the state manager on both agents
    agent1.set_state_manager(state_manager.clone());
    agent2.set_state_manager(state_manager.clone());

    // Initialize and start agent1
    agent1.initialize().await?;
    agent1.start().await?;

    // Execute some interactions with agent1
    let context = ExecutionContext::new();
    let input1 = AgentInput::text("Hello, my name is Bob");
    let response1 = agent1.execute(input1, context.clone()).await?;
    assert!(!response1.text.is_empty());

    let input2 = AgentInput::text("I like pizza");
    let response2 = agent1.execute(input2, context.clone()).await?;
    assert!(!response2.text.is_empty());

    // Save agent1's state
    agent1.save_state().await?;

    // Verify agent1 has conversation history
    let agent1_history = agent1.get_conversation().await?;
    assert_eq!(agent1_history.len(), 4); // 2 user + 2 assistant messages

    // Verify agent2 has no conversation history yet
    let agent2_history_before = agent2.get_conversation().await?;
    assert_eq!(agent2_history_before.len(), 0);

    // Load state into agent2
    let loaded = agent2.load_state().await?;
    assert!(loaded);

    // Verify agent2 now has the same conversation history
    let agent2_history_after = agent2.get_conversation().await?;
    assert_eq!(agent2_history_after.len(), 4);

    // Verify the content matches
    assert_eq!(agent2_history_after[0].content, "Hello, my name is Bob");
    assert_eq!(agent2_history_after[2].content, "I like pizza");

    // Continue conversation with agent2
    let input3 = AgentInput::text("What's my name?");
    let response3 = agent2.execute(input3, context).await?;
    assert!(!response3.text.is_empty());

    // Clean up
    agent1.stop().await?;
    agent1.terminate().await?;

    Ok(())
}
#[tokio::test]
async fn test_mock_agent_state_persistence() -> Result<()> {
    // Create mock agent configuration with pre-programmed responses
    let mut mock_config = MockAgentConfig::default();
    mock_config.agent_config.name = "test-mock-agent".to_string();
    mock_config.agent_config.description = "Mock agent for state persistence testing".to_string();

    // Add mock responses that simulate an LLM conversation
    mock_config.responses = vec![
        MockResponse {
            input_pattern: Some("Remember this number".to_string()),
            text: "I'll remember the number 42 for you.".to_string(),
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
        },
        MockResponse {
            input_pattern: Some("What number".to_string()),
            text: "The number you asked me to remember is 42.".to_string(),
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
        },
    ];

    // Create the mock agent
    let mut agent = MockAgent::new(mock_config);

    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);
    agent.set_state_manager(state_manager.clone());

    // Initialize and start
    agent.initialize().await?;
    agent.start().await?;

    // Have a conversation
    let context = ExecutionContext::new();
    let input1 = AgentInput::text("Remember this number: 42");
    let response1 = agent.execute(input1, context.clone()).await?;
    assert!(response1.text.contains("42"));

    // Save state
    agent.save_state().await?;

    // Clear conversation
    agent.clear_conversation().await?;
    let cleared_history = agent.get_conversation().await?;
    assert_eq!(cleared_history.len(), 0);

    // Reload state
    let loaded = agent.load_state().await?;
    assert!(loaded);

    // Verify conversation was restored
    let restored_history = agent.get_conversation().await?;
    assert_eq!(restored_history.len(), 2); // User message + assistant response
    assert!(restored_history[0].content.contains("Remember this number"));
    assert!(restored_history[1].content.contains("42"));

    // Test that the mock agent can still respond correctly after reload
    let input2 = AgentInput::text("What number did I ask you to remember?");
    let response2 = agent.execute(input2, context).await?;
    assert!(response2.text.contains("42"));

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    Ok(())
}
#[tokio::test]
async fn test_state_persistence_with_multiple_agents() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create multiple agents
    let config1 = AgentBuilder::basic("agent-1")
        .description("First test agent")
        .build()?;
    let config2 = AgentBuilder::basic("agent-2")
        .description("Second test agent")
        .build()?;

    let agent1 = BasicAgent::new(config1)?;
    let agent2 = BasicAgent::new(config2)?;

    // Set state manager on both
    agent1.set_state_manager(state_manager.clone());
    agent2.set_state_manager(state_manager.clone());

    // Initialize both agents
    agent1.initialize().await?;
    agent2.initialize().await?;

    // Execute different conversations
    let context = ExecutionContext::new();

    // Agent 1 conversation
    agent1
        .execute(AgentInput::text("Agent 1 message"), context.clone())
        .await?;

    // Agent 2 conversation
    agent2
        .execute(AgentInput::text("Agent 2 message"), context.clone())
        .await?;

    // Save both states
    agent1.save_state().await?;
    agent2.save_state().await?;

    // Create new instances with same IDs
    let new_agent1 = BasicAgent::new(
        AgentBuilder::basic("agent-1")
            .description("First test agent")
            .build()?,
    )?;
    let new_agent2 = BasicAgent::new(
        AgentBuilder::basic("agent-2")
            .description("Second test agent")
            .build()?,
    )?;

    new_agent1.set_state_manager(state_manager.clone());
    new_agent2.set_state_manager(state_manager.clone());

    // Load states
    let loaded1 = new_agent1.load_state().await?;
    let loaded2 = new_agent2.load_state().await?;
    assert!(loaded1);
    assert!(loaded2);

    // Verify each agent has its own conversation
    let history1 = new_agent1.get_conversation().await?;
    let history2 = new_agent2.get_conversation().await?;

    assert!(history1.iter().any(|m| m.content.contains("Agent 1")));
    assert!(history2.iter().any(|m| m.content.contains("Agent 2")));
    assert!(!history1.iter().any(|m| m.content.contains("Agent 2")));
    assert!(!history2.iter().any(|m| m.content.contains("Agent 1")));

    Ok(())
}
#[tokio::test]
async fn test_state_persistence_error_handling() -> Result<()> {
    // Create agent without state manager
    let config = AgentBuilder::basic("test-agent")
        .description("Test agent")
        .build()?;
    let agent = BasicAgent::new(config)?;

    // Try to save state without state manager
    let save_result = agent.save_state().await;
    assert!(save_result.is_err());
    assert!(save_result
        .unwrap_err()
        .to_string()
        .contains("No state manager configured"));

    // Try to load state without state manager
    let load_result = agent.load_state().await;
    assert!(load_result.is_err());
    assert!(load_result
        .unwrap_err()
        .to_string()
        .contains("No state manager configured"));

    Ok(())
}

// Placeholder for Task 5.2.7: Real Provider Integration Tests
#[tokio::test]
#[ignore = "Task 5.2.7: Requires real LLM provider configuration"]
async fn test_llm_agent_state_persistence_with_real_provider() -> Result<()> {
    // This test will be implemented in Task 5.2.7
    // It will test state persistence with real LLM providers like OpenAI, Anthropic, etc.
    // Configuration will come from environment variables or test configuration

    // TODO: Implementation in Task 5.2.7 will include:
    // 1. Create LLM agent with real provider
    // 2. Have a conversation with context that can be verified
    // 3. Save state
    // 4. Create new agent instance
    // 5. Load state
    // 6. Verify conversation context is maintained
    // 7. Continue conversation to verify context understanding

    Ok(())
}
