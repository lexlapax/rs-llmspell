//! Tests for lifecycle hooks with automatic state persistence

use anyhow::Result;
use llmspell_agents::{
    agents::basic::BasicAgent,
    builder::AgentBuilder,
    config::{presets, PersistenceConfigBuilder},
    hooks::StatePersistenceHook,
    lifecycle::events::{
        EventSystemConfig, LifecycleEvent, LifecycleEventData, LifecycleEventSystem,
        LifecycleEventType,
    },
    state::StatePersistence,
};
use llmspell_core::{
    traits::{agent::Agent, base_agent::BaseAgent},
    types::AgentInput,
    ExecutionContext,
};
use llmspell_state_persistence::StateManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
#[tokio::test]
async fn test_save_on_pause() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create persistence config
    let config = PersistenceConfigBuilder::new()
        .save_on_pause(true)
        .save_on_stop(false)
        .restore_on_resume(false)
        .non_blocking(false) // Synchronous for testing
        .build();

    // Create persistence hook
    let hook = Arc::new(StatePersistenceHook::new(state_manager.clone(), config));

    // Create event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Subscribe hook to events
    let _subscription = event_system
        .subscribe_filtered(
            "test_hook",
            {
                let hook = hook.clone();
                move |event: &LifecycleEvent| {
                    let hook = hook.clone();
                    let event = event.clone();
                    tokio::spawn(async move {
                        let _ = hook.handle_event(&event).await;
                    });
                }
            },
            vec![LifecycleEventType::AgentPaused],
        )
        .await;

    // Create agent
    let config = AgentBuilder::basic("test-agent").build()?;
    let agent = BasicAgent::new(config)?;
    let agent_id = agent.metadata().id.to_string();
    agent.set_state_manager(state_manager.clone());

    // Execute to create some state
    let context = ExecutionContext::new();
    let input = AgentInput::text("Test message");
    agent.execute(input, context).await?;

    // Register agent with hook
    hook.register_agent(
        agent_id.clone(),
        Arc::new(tokio::sync::Mutex::new(
            Box::new(agent) as Box<dyn Agent + Send + Sync>
        )),
    )
    .await;

    // Emit pause event
    let _ = event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::AgentPaused,
            agent_id,
            LifecycleEventData::Generic {
                message: "Test pause".to_string(),
                details: Default::default(),
            },
            "test".to_string(),
        ))
        .await;

    // Wait for save to complete
    sleep(Duration::from_millis(100)).await;

    // Check metrics
    let metrics = hook.metrics();
    assert_eq!(
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    Ok(())
}
#[tokio::test]
async fn test_save_on_stop() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create persistence config
    let config = PersistenceConfigBuilder::new()
        .save_on_pause(false)
        .save_on_stop(true)
        .restore_on_resume(false)
        .non_blocking(false)
        .build();

    // Create persistence hook
    let hook = Arc::new(StatePersistenceHook::new(state_manager.clone(), config));

    // Create event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Subscribe hook to events
    let _subscription = event_system
        .subscribe_filtered(
            "test_hook",
            {
                let hook = hook.clone();
                move |event: &LifecycleEvent| {
                    let hook = hook.clone();
                    let event = event.clone();
                    tokio::spawn(async move {
                        let _ = hook.handle_event(&event).await;
                    });
                }
            },
            vec![LifecycleEventType::TerminationStarted],
        )
        .await;

    // Create agent
    let config = AgentBuilder::basic("test-agent-stop").build()?;
    let agent = BasicAgent::new(config)?;
    let agent_id = agent.metadata().id.to_string();
    agent.set_state_manager(state_manager.clone());

    // Execute to create some state
    let context = ExecutionContext::new();
    let input = AgentInput::text("Test message for stop");
    agent.execute(input, context).await?;

    // Register agent with hook
    hook.register_agent(
        agent_id.clone(),
        Arc::new(tokio::sync::Mutex::new(
            Box::new(agent) as Box<dyn Agent + Send + Sync>
        )),
    )
    .await;

    // Emit termination started event
    let _ = event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::TerminationStarted,
            agent_id,
            LifecycleEventData::Generic {
                message: "Test stop".to_string(),
                details: Default::default(),
            },
            "test".to_string(),
        ))
        .await;

    // Wait for save to complete
    sleep(Duration::from_millis(100)).await;

    // Check metrics
    let metrics = hook.metrics();
    assert_eq!(
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    Ok(())
}
#[tokio::test]
async fn test_auto_save() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create persistence config with fast auto-save
    let config = PersistenceConfigBuilder::new()
        .with_auto_save(Duration::from_millis(100)) // Fast for testing
        .save_on_pause(false)
        .save_on_stop(false)
        .non_blocking(false)
        .build();

    // Create persistence hook
    let hook = Arc::new(StatePersistenceHook::new(state_manager.clone(), config));

    // Create agent
    let config = AgentBuilder::basic("test-agent-auto").build()?;
    let agent = BasicAgent::new(config)?;
    let agent_id = agent.metadata().id.to_string();
    agent.set_state_manager(state_manager.clone());

    // Execute to create some state
    let context = ExecutionContext::new();
    let input = AgentInput::text("Test auto-save");
    agent.execute(input, context).await?;

    // Register agent with hook
    hook.register_agent(
        agent_id.clone(),
        Arc::new(tokio::sync::Mutex::new(
            Box::new(agent) as Box<dyn Agent + Send + Sync>
        )),
    )
    .await;

    // Trigger auto-save check multiple times
    for _ in 0..3 {
        hook.check_auto_save().await?;
        sleep(Duration::from_millis(150)).await;
    }

    // Check metrics
    let metrics = hook.metrics();
    assert!(
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 2
    );

    Ok(())
}
#[tokio::test]
async fn test_circuit_breaker() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create persistence config with low failure threshold
    let config = PersistenceConfigBuilder::new()
        .with_failure_threshold(2)
        .save_on_pause(true)
        .non_blocking(false)
        .build();

    // Create persistence hook
    let hook = Arc::new(StatePersistenceHook::new(state_manager.clone(), config));

    // Create event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Subscribe hook to events
    let _subscription = event_system
        .subscribe_filtered(
            "test_hook",
            {
                let hook = hook.clone();
                move |event: &LifecycleEvent| {
                    let hook = hook.clone();
                    let event = event.clone();
                    tokio::spawn(async move {
                        let _ = hook.handle_event(&event).await;
                    });
                }
            },
            vec![LifecycleEventType::AgentPaused],
        )
        .await;

    // Create agent but DON'T register it (will cause saves to fail)
    let config = AgentBuilder::basic("test-agent-breaker").build()?;
    let agent = BasicAgent::new(config)?;
    let agent_id = agent.metadata().id.to_string();

    // Emit pause events to trigger failures
    for i in 0..3 {
        let _ = event_system
            .emit(LifecycleEvent::new(
                LifecycleEventType::AgentPaused,
                agent_id.clone(),
                LifecycleEventData::Generic {
                    message: format!("Test pause {}", i),
                    details: Default::default(),
                },
                "test".to_string(),
            ))
            .await;
        sleep(Duration::from_millis(50)).await;
    }

    // Check metrics - should show failures
    let metrics = hook.metrics();
    assert!(
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed)
            >= 2
    );

    Ok(())
}
#[tokio::test]
async fn test_non_blocking_saves() -> Result<()> {
    // Create state manager
    let state_manager = Arc::new(StateManager::new().await?);

    // Create persistence config with non-blocking saves
    let config = PersistenceConfigBuilder::new()
        .save_on_pause(true)
        .non_blocking(true)
        .build();

    // Create persistence hook
    let hook = Arc::new(StatePersistenceHook::new(state_manager.clone(), config));

    // Create event system
    let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));

    // Subscribe hook to events
    let _subscription = event_system
        .subscribe_filtered(
            "test_hook",
            {
                let hook = hook.clone();
                move |event: &LifecycleEvent| {
                    let hook = hook.clone();
                    let event = event.clone();
                    tokio::spawn(async move {
                        let _ = hook.handle_event(&event).await;
                    });
                }
            },
            vec![LifecycleEventType::AgentPaused],
        )
        .await;

    // Create agent
    let config = AgentBuilder::basic("test-agent-async").build()?;
    let agent = BasicAgent::new(config)?;
    let agent_id = agent.metadata().id.to_string();
    agent.set_state_manager(state_manager.clone());

    // Execute to create some state
    let context = ExecutionContext::new();
    let input = AgentInput::text("Test non-blocking");
    agent.execute(input, context).await?;

    // Register agent with hook
    hook.register_agent(
        agent_id.clone(),
        Arc::new(tokio::sync::Mutex::new(
            Box::new(agent) as Box<dyn Agent + Send + Sync>
        )),
    )
    .await;

    // Emit pause event
    let start = std::time::Instant::now();
    let _ = event_system
        .emit(LifecycleEvent::new(
            LifecycleEventType::AgentPaused,
            agent_id,
            LifecycleEventData::Generic {
                message: "Test pause".to_string(),
                details: Default::default(),
            },
            "test".to_string(),
        ))
        .await;

    // Non-blocking save should return quickly
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(50),
        "Non-blocking save took too long"
    );

    // Wait a bit for background save to complete
    sleep(Duration::from_millis(200)).await;

    // Check metrics
    let metrics = hook.metrics();
    assert_eq!(
        metrics
            .saves_attempted
            .load(std::sync::atomic::Ordering::Relaxed),
        1
    );

    Ok(())
}
#[tokio::test]
async fn test_presets() -> Result<()> {
    // Test different preset configurations
    let dev_config = presets::development();
    assert_eq!(dev_config.auto_save_interval, Some(Duration::from_secs(60)));
    assert!(!dev_config.non_blocking);

    let prod_config = presets::production();
    assert_eq!(
        prod_config.auto_save_interval,
        Some(Duration::from_secs(300))
    );
    assert!(prod_config.non_blocking);

    let test_config = presets::testing();
    assert_eq!(
        test_config.auto_save_interval,
        Some(Duration::from_millis(100))
    );
    assert_eq!(test_config.max_retries, 0);

    let minimal_config = presets::minimal();
    assert!(minimal_config.auto_save_interval.is_none());
    assert!(!minimal_config.save_on_pause);
    assert!(minimal_config.save_on_stop);

    Ok(())
}
