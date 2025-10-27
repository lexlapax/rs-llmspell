//! Example of creating and using a stateful agent with persistence

use anyhow::Result;
use llmspell_agents::agents::basic::BasicAgent;
use llmspell_agents::builder::AgentBuilder;
use llmspell_agents::state::persistence::StatePersistence;
use llmspell_core::traits::agent::Agent;
use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use llmspell_kernel::state::config::PerformanceConfig;
use llmspell_kernel::state::{PersistenceConfig, StateManager, StorageBackendType};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Stateful Agent Example ===\n");

    // Create agent configuration
    let config = AgentBuilder::basic("memory-agent")
        .description("An agent that remembers conversations")
        .build()?;

    // Create the agent
    let agent = BasicAgent::new(config)?;
    println!("Created agent: {}", agent.metadata().name);

    // Create state manager with persistent storage
    let state_manager = Arc::new(
        StateManager::with_backend(
            StorageBackendType::Memory, // Use Memory for example, could be Sled or RocksDB
            PersistenceConfig {
                enabled: true,
                backend_type: StorageBackendType::Memory,
                flush_interval: std::time::Duration::from_secs(1),
                compression: false,
                encryption: None,
                backup_retention: std::time::Duration::from_secs(7 * 24 * 60 * 60),
                backup: None,
                performance: PerformanceConfig::default(),
            },
            None, // No memory manager for this example
        )
        .await?,
    );

    // Set the state manager on the agent
    agent.set_state_manager(state_manager.clone());
    println!("Configured state persistence");

    // Initialize the agent
    agent.initialize().await?;
    agent.start().await?;

    // First conversation
    println!("\n--- First Conversation ---");

    let input1 = AgentInput::text("Hello! My name is Alice.");
    let context = ExecutionContext::new();
    let response1 = agent.execute(input1, context.clone()).await?;
    println!("User: Hello! My name is Alice.");
    println!("Agent: {}", response1.text);

    let input2 = AgentInput::text("What's the weather like?");
    let response2 = agent.execute(input2, context.clone()).await?;
    println!("User: What's the weather like?");
    println!("Agent: {}", response2.text);

    // Save the agent's state
    println!("\n--- Saving State ---");
    agent.save_state().await?;
    println!("State saved successfully!");

    // Clear the conversation to simulate a new session
    agent.clear_conversation().await?;
    println!("Cleared conversation history");

    // Verify conversation is empty
    let history = agent.get_conversation().await?;
    println!("Current conversation length: {}", history.len());

    // Load the saved state
    println!("\n--- Loading State ---");
    let loaded = agent.load_state().await?;
    println!("State loaded: {loaded}");

    // Check restored conversation
    let restored_history = agent.get_conversation().await?;
    println!("Restored conversation length: {}", restored_history.len());

    if !restored_history.is_empty() {
        println!("\nRestored conversation:");
        for msg in &restored_history {
            println!("  {}: {}", msg.role, msg.content);
        }
    }

    // Continue the conversation
    println!("\n--- Continuing Conversation ---");
    let input3 = AgentInput::text("Do you remember my name?");
    let response3 = agent.execute(input3, context).await?;
    println!("User: Do you remember my name?");
    println!("Agent: {}", response3.text);

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    println!("\n=== Example Complete ===");
    Ok(())
}
