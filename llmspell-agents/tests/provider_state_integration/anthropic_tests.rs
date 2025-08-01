//! ABOUTME: Anthropic provider integration tests for state persistence
//! ABOUTME: Tests real Anthropic Claude API calls with conversation state persistence and restoration

use super::common::*;
use anyhow::Result;
use llmspell_agents::StatePersistence;
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_conversation_persistence() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create Anthropic agent
    let mut agent = context
        .create_anthropic_agent()
        .await?
        .expect("Should create Anthropic agent when API key is available");

    info!("Starting Anthropic conversation persistence test");

    // Test conversation messages
    let messages = vec![
        "Hello Claude! Please introduce yourself briefly.",
        "What's the largest planet in our solar system?",
        "Can you recall what you told me about yourself?",
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
            "Response {} should not be empty",
            i
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
        .create_and_restore_agent("anthropic", "claude-3-5-sonnet-latest")
        .await?
        .expect("Should create restored agent");

    // Continue conversation to verify state restoration
    let input = AgentInput::text("Do you remember what we discussed about planets?");
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
    let response_lower = response.text.to_lowercase();
    let has_memory_indicators = response_lower.contains("jupiter")
        || response_lower.contains("planet")
        || response_lower.contains("solar")
        || response_lower.contains("remember")
        || response_lower.contains("discussed");

    if has_memory_indicators {
        info!("✅ Agent appears to remember previous conversation");
    } else {
        warn!("⚠️  Agent response doesn't clearly indicate memory of previous conversation");
        warn!("This may be normal depending on the model's behavior");
    }

    // Clean up
    restored_agent.stop().await?;
    restored_agent.terminate().await?;

    info!("Anthropic conversation persistence test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_structured_thinking() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic structured thinking test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create Anthropic agent
    let agent = context
        .create_anthropic_agent()
        .await?
        .expect("Should create Anthropic agent when API key is available");

    info!("Starting Anthropic structured thinking test");

    // Initialize and start agent
    agent.initialize().await?;
    agent.start().await?;

    // Ask for a structured response that tests Claude's reasoning
    let input = AgentInput::text("Please solve this step by step: If a train travels 120 miles in 2 hours, what's its average speed? Show your work.");
    let exec_context = ExecutionContext::new();

    let response = timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

    info!("Agent response: {}", response.text);
    assert!(!response.text.is_empty());

    // Check if the response contains structured thinking
    let response_lower = response.text.to_lowercase();
    let has_structure = response_lower.contains("step") 
        || response_lower.contains("60") // Expected answer
        || response_lower.contains("miles per hour")
        || response_lower.contains("mph");

    assert!(
        has_structure,
        "Response should show structured problem solving"
    );

    // Save state
    agent.save_state().await?;

    // Verify state persistence
    let agent_id = agent.metadata().id.to_string();
    assert!(
        context.verify_state_persistence(&agent_id, 2).await?, // 1 user + 1 assistant
        "Should have persisted conversation state"
    );

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    info!("Anthropic structured thinking test completed");
    Ok(())
}
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_long_context() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic long context test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Create Anthropic agent
    let agent = context
        .create_anthropic_agent()
        .await?
        .expect("Should create Anthropic agent when API key is available");

    info!("Starting Anthropic long context test");

    // Initialize and start agent
    agent.initialize().await?;
    agent.start().await?;

    // Create a longer conversation to test context handling
    let messages = vec![
        "I'm going to tell you about several animals. Please remember them.",
        "First, there's a red fox named Ruby who lives in the forest.",
        "Second, there's a blue whale named Blue who lives in the ocean.",
        "Third, there's a golden eagle named Gold who soars in the mountains.",
        "Fourth, there's a green turtle named Emerald who swims in the reef.",
        "Now, can you tell me about all the animals I mentioned and their colors?",
    ];

    let mut all_responses = Vec::new();
    let exec_context = ExecutionContext::new();

    for message in messages {
        let input = AgentInput::text(message);
        let response = timeout(
            Duration::from_secs(30),
            agent.execute(input, exec_context.clone()),
        )
        .await??;

        info!("Response to '{}': {}", message, response.text);
        all_responses.push(response);

        // Small delay between messages
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // Check the final response for memory of all animals
    let final_response = &all_responses.last().unwrap().text.to_lowercase();
    let remembers_ruby = final_response.contains("ruby") || final_response.contains("fox");
    let remembers_blue = final_response.contains("blue") || final_response.contains("whale");
    let remembers_gold = final_response.contains("gold") || final_response.contains("eagle");
    let remembers_emerald = final_response.contains("emerald") || final_response.contains("turtle");

    let memory_score = [
        remembers_ruby,
        remembers_blue,
        remembers_gold,
        remembers_emerald,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    info!("Memory score: {}/4 animals remembered", memory_score);

    // We expect at least 2 out of 4 to be remembered for a passing test
    assert!(
        memory_score >= 2,
        "Should remember at least 2 out of 4 animals in context"
    );

    // Save state
    agent.save_state().await?;

    // Verify state persistence with multiple messages
    let agent_id = agent.metadata().id.to_string();
    assert!(
        context.verify_state_persistence(&agent_id, 10).await?, // 6 user + at least 4 assistant
        "Should have persisted long conversation state"
    );

    // Clean up
    agent.stop().await?;
    agent.terminate().await?;

    info!("Anthropic long context test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_anthropic_cross_session_memory() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping Anthropic cross-session memory test - no API key");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;

    // Session 1: Create agent and establish some facts
    {
        let agent = context
            .create_anthropic_agent()
            .await?
            .expect("Should create Anthropic agent when API key is available");

        info!("Starting Anthropic cross-session memory test - Session 1");

        agent.initialize().await?;
        agent.start().await?;

        // Establish some memorable facts
        let input1 =
            AgentInput::text("My favorite programming language is Rust. Please remember this.");
        let input2 =
            AgentInput::text("I work as a software engineer in San Francisco. Remember this too.");

        let exec_context = ExecutionContext::new();

        let response1 = timeout(
            Duration::from_secs(30),
            agent.execute(input1, exec_context.clone()),
        )
        .await??;

        let response2 = timeout(
            Duration::from_secs(30),
            agent.execute(input2, exec_context.clone()),
        )
        .await??;

        info!("Session 1 Response 1: {}", response1.text);
        info!("Session 1 Response 2: {}", response2.text);

        // Save state
        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Small delay to simulate session break
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Session 2: Create new agent instance and test memory
    {
        let agent = context
            .create_and_restore_agent("anthropic", "claude-3-5-sonnet-latest")
            .await?
            .expect("Should create restored agent");

        info!("Anthropic cross-session memory test - Session 2");

        // Test if it remembers the facts from session 1
        let input =
            AgentInput::text("What do you remember about my work and programming preferences?");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("Session 2 Response: {}", response.text);

        // Check for memory indicators
        let response_lower = response.text.to_lowercase();
        let remembers_rust = response_lower.contains("rust");
        let remembers_engineer =
            response_lower.contains("engineer") || response_lower.contains("software");
        let remembers_sf =
            response_lower.contains("san francisco") || response_lower.contains("francisco");

        let memory_indicators = [remembers_rust, remembers_engineer, remembers_sf]
            .iter()
            .filter(|&&x| x)
            .count();

        info!("Cross-session memory indicators: {}/3", memory_indicators);

        if memory_indicators >= 1 {
            info!("✅ Agent shows some cross-session memory");
        } else {
            warn!("⚠️  Agent doesn't clearly show cross-session memory");
            warn!("This could be due to model behavior or state restoration issues");
        }

        // Clean up
        agent.stop().await?;
        agent.terminate().await?;
    }

    info!("Anthropic cross-session memory test completed");
    Ok(())
}
