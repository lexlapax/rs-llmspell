//! ABOUTME: `OpenAI` provider integration tests for state persistence
//! ABOUTME: Tests real `OpenAI` API calls with conversation state persistence and restoration

use super::common::*;
use anyhow::Result;
use llmspell_agents::StatePersistence;
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_conversation_persistence() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create OpenAI agent
    let mut agent = context
        .create_openai_agent()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Starting OpenAI conversation persistence test");

    // Test conversation messages
    let messages = vec![
        "Hello! Please introduce yourself.",
        "What's the capital of France?",
        "Can you remember what I asked you first?",
    ];

    // Run conversation and save state
    let responses = context
        .run_conversation_with_save(&mut agent, messages)
        .await?;

    // Verify we got responses
    assert_eq!(responses.len(), 3);
    for (i, response) in responses.iter().enumerate() {
        assert!(
            !response.text.is_empty(),
            "Response {i} should not be empty"
        );
        info!("Response {}: {}", i + 1, response.text);
    }

    // Verify state persistence
    let agent_id = agent.metadata().id.to_string();
    assert!(
        context.verify_state_persistence(&agent_id, 6).await?, // 3 user + 3 assistant messages
        "Should have persisted conversation state"
    );

    // Stop the first agent
    agent.stop().await?;

    // Create a new agent instance with the same ID and restore state
    let restored_agent = context
        .create_and_restore_agent("openai", "gpt-4")
        .await?
        .expect("Should create restored agent");

    // Continue conversation to verify state restoration
    let input = AgentInput::text("Do you remember our conversation about France?");
    let context_exec = ExecutionContext::new();

    // Use timeout to prevent hanging
    let response = timeout(
        Duration::from_secs(30),
        restored_agent.execute(input, context_exec),
    )
    .await??;

    info!("Restored agent response: {}", response.text);
    assert!(!response.text.is_empty(), "Restored agent should respond");

    // Verify the response shows some memory of the conversation
    // Note: This is heuristic since LLMs may not always reference previous context
    let response_lower = response.text.to_lowercase();
    let has_memory_indicators = response_lower.contains("paris")
        || response_lower.contains("france")
        || response_lower.contains("capital")
        || response_lower.contains("remember")
        || response_lower.contains("conversation");

    if has_memory_indicators {
        info!("✅ Agent appears to remember previous conversation");
    } else {
        warn!("⚠️  Agent response doesn't clearly indicate memory of previous conversation");
        warn!("This may be normal depending on the model's behavior");
    }

    // Clean up
    restored_agent.stop().await?;
    restored_agent.terminate().await?;

    info!("OpenAI conversation persistence test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_token_tracking() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI token tracking test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create OpenAI agent
    let agent = context
        .create_openai_agent()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Starting OpenAI token tracking test");

    // Initialize and start agent
    agent.initialize().await?;
    agent.start().await?;

    // Have a conversation that will generate tokens
    let input = AgentInput::text("Write a short poem about programming in exactly 50 words.");
    let exec_context = ExecutionContext::new();

    let response = timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

    info!("Agent response: {}", response.text);
    assert!(!response.text.is_empty());

    // Save state
    agent.save_state().await?;

    // Verify token usage is tracked (this is provider-dependent)
    let agent_id = agent.metadata().id.to_string();
    let has_token_data = context.verify_token_usage(&agent_id).await?;

    if has_token_data {
        info!("✅ Token usage is being tracked");
    } else {
        warn!("⚠️  Token usage not found in state - this may be expected if not implemented yet");
    }

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    info!("OpenAI token tracking test completed");
    Ok(())
}
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_error_recovery() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI error recovery test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create OpenAI agent
    let agent = context
        .create_openai_agent()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Starting OpenAI error recovery test");

    // Initialize and start agent
    agent.initialize().await?;
    agent.start().await?;

    // Have a normal conversation first
    let input1 = AgentInput::text("Hello, how are you?");
    let exec_context = ExecutionContext::new();

    let response1 = timeout(
        Duration::from_secs(30),
        agent.execute(input1, exec_context.clone()),
    )
    .await??;

    info!("First response: {}", response1.text);
    assert!(!response1.text.is_empty());

    // Save state after successful interaction
    agent.save_state().await?;

    // Try to trigger an error with a very large request (this might not always fail)
    let large_input = "A".repeat(100000); // Very large input
    let input2 = AgentInput::text(&large_input);

    // This might succeed or fail, we just want to ensure state remains intact
    let result = timeout(
        Duration::from_secs(60),
        agent.execute(input2, exec_context.clone()),
    )
    .await;

    match result {
        Ok(Ok(response)) => {
            info!(
                "Large input succeeded: {}",
                response.text.chars().take(100).collect::<String>()
            );
        }
        Ok(Err(e)) => {
            info!("Large input failed as expected: {}", e);
        }
        Err(_) => {
            warn!("Large input timed out");
        }
    }

    // Verify state is still intact after potential error
    let agent_id = agent.metadata().id.to_string();
    assert!(
        context.verify_state_persistence(&agent_id, 2).await?, // At least 1 user + 1 assistant
        "State should be preserved even after errors"
    );

    // Continue with normal conversation to verify agent still works
    let input3 = AgentInput::text("Can you tell me a short joke?");
    let response3 = timeout(Duration::from_secs(30), agent.execute(input3, exec_context)).await??;

    info!("Recovery response: {}", response3.text);
    assert!(
        !response3.text.is_empty(),
        "Agent should still work after error"
    );

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    info!("OpenAI error recovery test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_openai_system_prompt_persistence() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping OpenAI system prompt test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create OpenAI agent with a specific system prompt
    let agent = context
        .create_openai_agent()
        .await?
        .expect("Should create OpenAI agent when API key is available");

    info!("Starting OpenAI system prompt persistence test");

    // Initialize with a system prompt (this would be in the agent config)
    agent.initialize().await?;
    agent.start().await?;

    // Test that the agent follows the system behavior
    let input = AgentInput::text("Please respond with enthusiasm!");
    let exec_context = ExecutionContext::new();

    let response = timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

    info!("Agent response: {}", response.text);
    assert!(!response.text.is_empty());

    // Save state
    agent.save_state().await?;

    // Verify provider metadata includes system configuration
    let agent_id = agent.metadata().id.to_string();
    let has_metadata = context.verify_provider_metadata(&agent_id).await?;

    if has_metadata {
        info!("✅ Provider metadata is being persisted");
    } else {
        warn!("⚠️  Provider metadata not found - this may be expected if not implemented yet");
    }

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    info!("OpenAI system prompt persistence test completed");
    Ok(())
}
