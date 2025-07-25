//! Example demonstrating automatic state persistence with lifecycle hooks

use anyhow::Result;
use llmspell_agents::{
    agents::basic::BasicAgent,
    builder::AgentBuilder,
    hooks::StatePersistenceHook,
    lifecycle::events::{
        EventSystemConfig, LifecycleEvent, LifecycleEventSystem, LifecycleEventType,
    },
    state::StatePersistence,
    PersistenceConfigBuilder,
};
use llmspell_core::{
    traits::{agent::Agent, base_agent::BaseAgent},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_state_persistence::{PersistenceConfig, StateManager, StorageBackendType};
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::time::sleep;
use tracing::{info, Level};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    // Create a temporary directory for storage
    let temp_dir = TempDir::new()?;
    let storage_path = temp_dir.path().to_path_buf();
    info!("Using storage path: {:?}", storage_path);

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
                flush_interval: Duration::from_secs(1),
                compression: true,
                encryption: None,
                backup_retention: Duration::from_secs(3600),
                performance: Default::default(),
            },
        )
        .await?,
    );

    // Configure auto-save behavior
    let persistence_config = PersistenceConfigBuilder::new()
        .with_auto_save(Duration::from_secs(5)) // Auto-save every 5 seconds
        .with_max_retries(2)
        .save_on_pause(true)
        .save_on_stop(true)
        .restore_on_resume(true)
        .non_blocking(true) // Don't block agent operations
        .build();

    // Or use a preset configuration
    // let persistence_config = presets::development(); // Auto-save every minute

    // Create the persistence hook
    let persistence_hook = Arc::new(StatePersistenceHook::new(
        state_manager.clone(),
        persistence_config,
    ));

    // Create event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Subscribe the persistence hook to lifecycle events
    let _subscription = event_system
        .subscribe_filtered(
            "persistence_hook",
            {
                let hook = persistence_hook.clone();
                move |event: &LifecycleEvent| {
                    let hook = hook.clone();
                    let event = event.clone();
                    tokio::spawn(async move {
                        if let Err(e) = hook.handle_event(&event).await {
                            tracing::error!("Persistence hook error: {}", e);
                        }
                    });
                }
            },
            vec![
                LifecycleEventType::AgentPaused,
                LifecycleEventType::TerminationStarted,
                LifecycleEventType::AgentResumed,
            ],
        )
        .await;

    // Create agent configuration
    let config = AgentBuilder::basic("auto-save-agent")
        .description("Agent with automatic state persistence")
        .build()?;

    // Create first agent instance
    let mut agent1 = BasicAgent::new(config.clone())?;
    agent1.set_state_manager(state_manager.clone());
    let agent1_id = agent1.metadata().id.to_string();

    // Initialize and start agent
    agent1.initialize().await?;
    agent1.start().await?;

    info!("Agent started, having a conversation...");

    // Have a conversation
    let context = ExecutionContext::new();

    let input1 = AgentInput::text("Hello! I'm testing auto-save functionality.");
    let response1 = agent1.execute(input1, context.clone()).await?;
    info!("Agent response: {}", response1.text);

    let input2 = AgentInput::text("Remember that my favorite color is blue.");
    let response2 = agent1.execute(input2, context.clone()).await?;
    info!("Agent response: {}", response2.text);

    // Register agent with persistence hook after some conversation
    persistence_hook
        .register_agent(
            agent1_id.clone(),
            Arc::new(tokio::sync::Mutex::new(
                Box::new(agent1) as Box<dyn Agent + Send + Sync>
            )),
        )
        .await;

    // Start auto-save monitoring
    let auto_save_handle = {
        let hook = persistence_hook.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(5)).await;
                if let Err(e) = hook.check_auto_save().await {
                    tracing::error!("Auto-save check failed: {}", e);
                }
            }
        })
    };

    info!("Waiting for auto-save to trigger (5 seconds)...");
    sleep(Duration::from_secs(6)).await;

    // Add more conversation (using another agent instance since first is moved)
    let mut agent1_cont = BasicAgent::new(config.clone())?;
    agent1_cont.set_state_manager(state_manager.clone());
    agent1_cont.initialize().await?;
    agent1_cont.start().await?;

    let input3 = AgentInput::text("What else should I tell you?");
    let response3 = agent1_cont.execute(input3, context.clone()).await?;
    info!("Agent response: {}", response3.text);

    // Pause the agent (should trigger save)
    info!("Pausing agent (should trigger save)...");
    agent1_cont.pause().await?;

    // Emit pause event
    event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::AgentPaused,
            agent1_id.clone(),
            llmspell_agents::lifecycle::events::LifecycleEventData::Generic {
                message: "Agent paused by user".to_string(),
                details: Default::default(),
            },
            "example".to_string(),
        ))
        .await?;

    sleep(Duration::from_secs(1)).await;

    // Create a new agent instance with same ID
    info!("Creating new agent instance to test state restoration...");
    let config2 = AgentBuilder::basic("auto-save-agent")
        .description("Agent with automatic state persistence")
        .build()?;
    let mut agent2 = BasicAgent::new(config2)?;
    agent2.set_state_manager(state_manager.clone());
    let agent2_id = agent2.metadata().id.to_string();

    // Register new agent with persistence hook
    persistence_hook
        .register_agent(
            agent2_id.clone(),
            Arc::new(tokio::sync::Mutex::new(
                Box::new(agent2) as Box<dyn Agent + Send + Sync>
            )),
        )
        .await;

    // Resume the agent (should trigger restore) - create another new agent
    info!("Resuming agent (should trigger restore)...");
    let config3 = AgentBuilder::basic("auto-save-agent")
        .description("Agent with automatic state persistence")
        .build()?;
    let mut agent3 = BasicAgent::new(config3)?;
    agent3.set_state_manager(state_manager.clone());
    agent3.initialize().await?;
    agent3.resume().await?;

    // Emit resume event
    event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::AgentResumed,
            agent2_id.clone(),
            llmspell_agents::lifecycle::events::LifecycleEventData::Generic {
                message: "Agent resumed by user".to_string(),
                details: Default::default(),
            },
            "example".to_string(),
        ))
        .await?;

    sleep(Duration::from_secs(1)).await;

    // Verify conversation was restored
    let conversation = agent3.get_conversation().await?;
    info!("Restored conversation has {} messages", conversation.len());

    for (i, msg) in conversation.iter().enumerate() {
        info!("  [{}] {}: {}", i + 1, msg.role, msg.content);
    }

    // Continue conversation
    let input4 = AgentInput::text("Do you remember what my favorite color is?");
    let response4 = agent3.execute(input4, context).await?;
    info!("Agent response: {}", response4.text);

    // Stop the agent (should trigger final save)
    info!("Stopping agent (should trigger final save)...");
    agent3.stop().await?;

    // Emit stop event
    event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::TerminationStarted,
            agent2_id,
            llmspell_agents::lifecycle::events::LifecycleEventData::Generic {
                message: "Agent stopped by user".to_string(),
                details: Default::default(),
            },
            "example".to_string(),
        ))
        .await?;

    agent3.terminate().await?;

    // Cancel auto-save task
    auto_save_handle.abort();

    // Get persistence metrics
    let metrics = persistence_hook.metrics();
    info!("\nPersistence Metrics:");
    info!(
        "  Saves attempted: {}",
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    info!(
        "  Saves succeeded: {}",
        metrics
            .saves_succeeded
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    info!(
        "  Saves failed: {}",
        metrics
            .saves_failed
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    info!(
        "  Restores attempted: {}",
        metrics
            .restores_attempted
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    info!(
        "  Restores succeeded: {}",
        metrics
            .restores_succeeded
            .load(std::sync::atomic::Ordering::Relaxed)
    );
    info!(
        "  Restores failed: {}",
        metrics
            .restores_failed
            .load(std::sync::atomic::Ordering::Relaxed)
    );

    info!("Example completed successfully!");
    Ok(())
}
