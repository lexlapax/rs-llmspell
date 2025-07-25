//! ABOUTME: Integration tests for agent state persistence functionality
//! ABOUTME: Tests the complete lifecycle of saving and loading agent states

use anyhow::Result;
use llmspell_agents::{
    agents::{basic::BasicAgent, llm::LLMAgent},
    builder::AgentBuilder,
    state::StatePersistence,
};
use llmspell_core::{
    traits::{agent::Agent, base_agent::BaseAgent},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_providers::ProviderManager;
use llmspell_state_persistence::{PersistenceConfig, StateManager, StorageBackendType};
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
    let mut agent1 = BasicAgent::new(config.clone())?;
    let mut agent2 = BasicAgent::new(config)?;

    // Create state manager with persistent storage
    let state_manager = Arc::new(
        StateManager::with_backend(
            StorageBackendType::Sled(llmspell_state_persistence::SledConfig {
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
                performance: Default::default(),
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
async fn test_llm_agent_state_persistence() -> Result<()> {
    // Skip if no provider is configured
    if std::env::var("OPENAI_API_KEY").is_err() {
        println!("Skipping LLM agent test - no OPENAI_API_KEY set");
        return Ok(());
    }

    // Create provider manager
    let provider_manager = Arc::new(ProviderManager::new());

    // Create agent configuration
    let config = AgentBuilder::new("test-llm", "llm")
        .description("Test LLM agent")
        .with_model("openai", "gpt-3.5-turbo")
        .temperature(0.7)
        .max_tokens(100)
        .build()?;

    // Try to create the agent, skip test if provider not available
    let mut agent = match LLMAgent::new(config, provider_manager).await {
        Ok(agent) => agent,
        Err(e) => {
            println!("Skipping LLM agent test - provider error: {}", e);
            return Ok(());
        }
    };

    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);
    agent.set_state_manager(state_manager.clone());

    // Initialize and start
    agent.initialize().await?;
    agent.start().await?;

    // Have a conversation
    let context = ExecutionContext::new();
    let input1 = AgentInput::text("Remember this number: 42");
    let _response1 = agent.execute(input1, context.clone()).await?;

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
    assert!(restored_history.len() > 0);
    assert!(restored_history[0].content.contains("42"));

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

    let mut agent1 = BasicAgent::new(config1)?;
    let mut agent2 = BasicAgent::new(config2)?;

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
    let mut new_agent1 = BasicAgent::new(
        AgentBuilder::basic("agent-1")
            .description("First test agent")
            .build()?,
    )?;
    let mut new_agent2 = BasicAgent::new(
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
    let mut agent = BasicAgent::new(config)?;

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
