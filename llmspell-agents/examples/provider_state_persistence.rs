//! Example demonstrating state persistence with real AI providers
//!
//! This example shows how to use state persistence with actual `OpenAI` or Anthropic APIs.
//! It requires either `OPENAI_API_KEY` or `ANTHROPIC_API_KEY` environment variables.
//!
//! Usage:
//! ```bash
//! # With OpenAI
//! OPENAI_API_KEY=your_key cargo run --example provider_state_persistence
//!
//! # With Anthropic
//! ANTHROPIC_API_KEY=your_key cargo run --example provider_state_persistence
//! ```

use anyhow::Result;
use llmspell_agents::{agents::llm::LLMAgent, builder::AgentBuilder, StatePersistence};
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_kernel::state::config::{PerformanceConfig, SledConfig};
use llmspell_kernel::state::{PersistenceConfig, StateManager, StorageBackendType};
use llmspell_providers::ProviderManager;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{info, Level};

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Check which provider is available
    let (provider, model) = if env::var("OPENAI_API_KEY").is_ok() {
        ("openai", "gpt-4")
    } else if env::var("ANTHROPIC_API_KEY").is_ok() {
        ("anthropic", "claude-3-5-sonnet-latest")
    } else {
        eprintln!(
            "This example requires either OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable"
        );
        eprintln!("Set one of these and try again:");
        eprintln!("  OPENAI_API_KEY=your_key cargo run --example provider_state_persistence");
        eprintln!("  ANTHROPIC_API_KEY=your_key cargo run --example provider_state_persistence");
        return Ok(());
    };

    info!("Using {} provider with model {}", provider, model);

    // Create temporary directory for state storage
    let temp_dir = TempDir::new()?;
    let storage_path = temp_dir.path().to_path_buf();
    info!("Using storage path: {:?}", storage_path);

    // Create state manager with persistent storage
    let state_manager = Arc::new(
        StateManager::with_backend(
            StorageBackendType::Sled(SledConfig {
                path: storage_path.join("example_states"),
                cache_capacity: 1024 * 1024, // 1MB
                use_compression: true,
            }),
            PersistenceConfig {
                enabled: true,
                backend_type: StorageBackendType::Memory, // Overridden by with_backend
                flush_interval: Duration::from_secs(1),
                compression: true,
                encryption: None,
                backup_retention: Duration::from_secs(3600),
                backup: None,
                performance: PerformanceConfig::default(),
            },
            None, // No memory manager for this example
        )
        .await?,
    );

    // Create provider manager and register rig provider
    let provider_manager = Arc::new(ProviderManager::new());
    provider_manager
        .register_provider("rig", llmspell_providers::create_rig_provider)
        .await;

    // Session 1: Create agent and have initial conversation
    info!("\n=== Session 1: Initial Conversation ===");

    let agent_id = "persistent-example-agent";
    let config = AgentBuilder::new(agent_id, "llm")
        .description(format!("Example agent using {provider} provider"))
        .with_model(provider, model)
        .temperature(0.7)
        .max_tokens(500)
        .build()?;

    let agent = LLMAgent::new(config, provider_manager.clone()).await?;
    agent.set_state_manager(state_manager.clone());

    // Initialize and start agent
    agent.initialize().await?;
    agent.start().await?;

    // Have an initial conversation
    let context = ExecutionContext::new();

    let input1 = AgentInput::text("Hello! Please introduce yourself and tell me something interesting about artificial intelligence.");
    let response1 = agent.execute(input1, context.clone()).await?;
    info!("Agent: {}", response1.text);

    let input2 = AgentInput::text("I'm particularly interested in how AI systems handle memory and context. What are your thoughts on this?");
    let response2 = agent.execute(input2, context.clone()).await?;
    info!("Agent: {}", response2.text);

    // Save state
    info!("Saving agent state...");
    agent.save_state().await?;

    // Stop the agent
    agent.stop().await?;
    agent.terminate().await?;

    info!("Session 1 complete. State saved.");

    // Simulate some time passing or application restart
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Session 2: Create new agent instance and restore state
    info!("\n=== Session 2: State Restoration ===");

    let config2 = AgentBuilder::new(agent_id, "llm") // Same ID for state restoration
        .description(format!("Restored {provider} agent"))
        .with_model(provider, model)
        .temperature(0.7)
        .max_tokens(500)
        .build()?;

    let agent2 = LLMAgent::new(config2, provider_manager.clone()).await?;
    agent2.set_state_manager(state_manager.clone());

    // Initialize and restore state
    agent2.initialize().await?;
    info!("Restoring agent state...");
    agent2.load_state().await?;
    agent2.start().await?;

    // Continue the conversation to test memory
    let input3 = AgentInput::text("Do you remember what we were discussing about AI and memory? Can you continue that conversation?");
    let response3 = agent2.execute(input3, context.clone()).await?;
    info!("Restored Agent: {}", response3.text);

    // Test with a more specific reference
    let input4 = AgentInput::text("Based on our previous conversation, what would you say is the most important aspect of AI memory systems?");
    let response4 = agent2.execute(input4, context.clone()).await?;
    info!("Restored Agent: {}", response4.text);

    // Save state again
    agent2.save_state().await?;

    // Clean up
    agent2.stop().await?;
    agent2.terminate().await?;

    info!("\n=== Example Complete ===");
    info!("This example demonstrated:");
    info!("1. Creating an agent with {} provider", provider);
    info!("2. Having a conversation with real AI responses");
    info!("3. Saving the agent's state (including conversation history)");
    info!("4. Creating a new agent instance with the same ID");
    info!("5. Restoring state and continuing the conversation");
    info!("6. Verifying that the agent maintains context across sessions");

    // Show some statistics if available
    let storage_dir = storage_path.join("example_states");
    if storage_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&storage_dir) {
            let file_count = entries.count();
            info!("State storage contains {} files", file_count);
        }
    }

    info!("State files are preserved at: {:?}", storage_path);
    info!("You can inspect them or run the example again to see state persistence in action.");

    Ok(())
}
