//! ABOUTME: OpenAI provider hook integration tests for persistence and replay functionality  
//! ABOUTME: Tests hook execution during real OpenAI API calls and validates persistence

use super::common::*;
use anyhow::Result;
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
use tracing_subscriber;

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_hook_execution_and_persistence() -> Result<()> {
    // Initialize tracing for debug output
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();

    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI hook test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    // Create OpenAI agent
    let mut agent = context
        .create_openai_agent_with_hooks()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Starting OpenAI hook execution test");

    // Run agent with hooks
    let response = timeout(
        Duration::from_secs(30),
        context.run_agent_with_hooks(&mut agent, "Say 'Hello, Hooks!' and nothing else."),
    )
    .await??;

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    info!("Agent response: {}", response);

    // Verify hooks were executed - get the correlation ID from the first context created
    let stored = context.stored_executions.read().await;
    info!("Stored executions: {} entries", stored.len());
    for (id, execs) in stored.iter() {
        info!("Correlation {}: {} executions", id, execs.len());
        for exec in execs {
            info!("  - Hook: {}", exec.hook_id);
        }
    }
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

    info!("✅ OpenAI hook integration verified successfully");

    Ok(())
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_cost_tracking_hook() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI cost tracking test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    // Create OpenAI agent
    let mut agent = context
        .create_openai_agent_with_hooks()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Testing cost tracking hook with OpenAI");

    // Run agent with hooks
    let response = timeout(
        Duration::from_secs(30),
        context.run_agent_with_hooks(&mut agent, "Count to 5 please."),
    )
    .await??;

    // Verify we got a response
    assert!(!response.is_empty(), "Response should not be empty");
    info!("Agent response: {}", response);

    // Verify hooks were executed and cost tracking hook was among them
    let stored = context.stored_executions.read().await;
    let correlation_id = stored.keys().next().unwrap();

    assert!(
        context
            .verify_hook_executed(correlation_id, "CostTrackingHook")
            .await?,
        "CostTrackingHook should have executed"
    );

    info!("✅ Cost tracking hook verified successfully");

    Ok(())
}

#[cfg_attr(test_category = "external")]
#[cfg_attr(test_category = "hook")]
#[cfg_attr(test_category = "performance")]
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_rate_limiting_hook() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI rate limiting test - no API key");
        return Ok(());
    }

    let context = HookTestContext::new().await?;

    info!("Testing rate limiting hook with OpenAI");

    // Run multiple requests to test rate limiting
    for i in 1..=3 {
        // Create a fresh agent for each request to avoid state issues
        let mut fresh_agent = context
            .create_openai_agent_with_hooks()
            .await?
            .expect("Should create OpenAI agent when API key is available");

        let response = timeout(
            Duration::from_secs(30),
            context.run_agent_with_hooks(&mut fresh_agent, &format!("Say 'Request {}' please.", i)),
        )
        .await??;

        assert!(!response.is_empty(), "Response {} should not be empty", i);
        info!("Request {} response: {}", i, response);

        // Small delay between requests
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    // Verify rate limiting hook was executed
    let stored = context.stored_executions.read().await;
    let has_rate_limit = stored.values().any(|executions| {
        executions
            .iter()
            .any(|exec| exec.hook_id.contains("RateLimitHook"))
    });

    assert!(has_rate_limit, "RateLimitHook should have executed");

    info!("✅ Rate limiting hook verified successfully");

    Ok(())
}
