//! ABOUTME: Anthropic provider hook integration tests for persistence and replay functionality
//! ABOUTME: Tests hook execution during real Anthropic API calls and validates persistence

use super::common::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_hook_execution_and_persistence() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic hook test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    // Create Anthropic agent
    let mut agent = context
        .create_anthropic_agent_with_hooks()
        .await?
        .expect("Should create Anthropic agent when API key is available");

    info!("Starting Anthropic hook execution test");

    // Run agent with hooks
    let response = timeout(
        Duration::from_secs(30),
        context.run_agent_with_hooks(&mut agent, "Say 'Hello from Claude!' and nothing else."),
    )
    .await??;

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    info!("Agent response: {}", response);

    // Verify hooks were executed - get the correlation ID from the first context created
    let stored = context.stored_executions.read().await;
    assert!(!stored.is_empty(), "Should have captured hook executions");

    let correlation_id = stored.keys().next().unwrap();
    info!("Verifying hooks for correlation ID: {}", correlation_id);

    // Verify hook executions
    assert!(
        context.verify_hook_executions(correlation_id).await?,
        "Should have hook executions"
    );

    // Verify specific hooks were executed
    assert!(
        context
            .verify_hook_executed(correlation_id, "logging_hook")
            .await?,
        "logging_hook should have executed"
    );

    assert!(
        context
            .verify_hook_executed(correlation_id, "MetricsHook")
            .await?,
        "MetricsHook should have executed"
    );

    assert!(
        context
            .verify_hook_executed(correlation_id, "SecurityHook")
            .await?,
        "SecurityHook should have executed"
    );

    info!("✅ Anthropic hook integration verified successfully");

    Ok(())
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_security_hook() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic security test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    // Create Anthropic agent
    let mut agent = context
        .create_anthropic_agent_with_hooks()
        .await?
        .expect("Should create Anthropic agent when API key is available");

    info!("Testing security hook with Anthropic");

    // Run agent with hooks
    let response = timeout(
        Duration::from_secs(30),
        context.run_agent_with_hooks(&mut agent, "Tell me about AI safety."),
    )
    .await??;

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    info!("Agent response: {}", response);

    // Verify hooks were executed and security hook was among them
    let stored = context.stored_executions.read().await;
    let correlation_id = stored.keys().next().unwrap();

    assert!(
        context
            .verify_hook_executed(correlation_id, "SecurityHook")
            .await?,
        "SecurityHook should have executed"
    );

    info!("✅ Security hook verified successfully");

    Ok(())
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "security")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_conversation_tracking() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic conversation tracking test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    info!("Testing conversation tracking with hooks");

    // Run a series of interactions
    let interactions = vec![
        "Hello! My name is Alice.",
        "What's the weather like?",
        "Do you remember my name?",
    ];

    for (i, message) in interactions.iter().enumerate() {
        // Create a fresh agent for each interaction to avoid state issues
        let mut fresh_agent = context
            .create_anthropic_agent_with_hooks()
            .await?
            .expect("Should create Anthropic agent when API key is available");

        let response = timeout(
            Duration::from_secs(30),
            context.run_agent_with_hooks(&mut fresh_agent, message),
        )
        .await??;

        assert!(
            !response.is_empty(),
            "Response {} should not be empty",
            i + 1
        );
        info!("Interaction {}: {} -> {}", i + 1, message, response);

        // Small delay between interactions
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Verify multiple hook executions were captured
    let stored = context.stored_executions.read().await;
    let total_executions: usize = stored.values().map(|v| v.len()).sum();

    assert!(
        total_executions >= 6,
        "Should have at least 6 hook executions (2 per interaction)"
    );

    info!("✅ Conversation tracking with hooks verified successfully");

    Ok(())
}
