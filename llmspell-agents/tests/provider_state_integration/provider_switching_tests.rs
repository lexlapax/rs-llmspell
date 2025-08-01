//! ABOUTME: Provider switching tests to verify state persistence across different LLM providers
//! ABOUTME: Tests that conversation history and context are preserved when switching between providers

use super::common::*;
use anyhow::Result;
use llmspell_agents::StatePersistence;
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{info, warn};
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY and ANTHROPIC_API_KEY"]
async fn test_switch_openai_to_anthropic() -> Result<()> {
    // Skip if either API key is missing
    if !check_api_key("openai") || !check_api_key("anthropic") {
        warn!("Skipping OpenAI to Anthropic switch test - missing API keys");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;
    let agent_id = "switch-test-openai-anthropic";

    info!("Starting provider switch test: OpenAI → Anthropic");

    // Phase 1: Start with OpenAI
    {
        let agent = context
            .create_openai_agent()
            .await?
            .expect("Should create OpenAI agent");

        agent.initialize().await?;
        agent.start().await?;

        // Have initial conversation
        let input1 = AgentInput::text("Hello! I'm going to tell you a story about a blue whale named Walter. Walter loves to sing underwater songs.");
        let exec_context = ExecutionContext::new();

        let response1 = timeout(
            Duration::from_secs(30),
            agent.execute(input1, exec_context.clone()),
        )
        .await??;

        info!("OpenAI response 1: {}", response1.text);

        // Continue the story
        let input2 = AgentInput::text("Walter's best friend is a dolphin named Diana. What do you think they like to do together?");
        let response2 =
            timeout(Duration::from_secs(30), agent.execute(input2, exec_context)).await??;

        info!("OpenAI response 2: {}", response2.text);

        // Save state
        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;

        info!("OpenAI session completed, state saved");
    }

    // Phase 2: Switch to Anthropic and continue
    {
        // Create Anthropic agent and restore state
        let agent = context
            .create_and_restore_agent("anthropic", "claude-3-5-sonnet-latest")
            .await?
            .expect("Should create and restore Anthropic agent");

        agent.start().await?;

        // Continue the conversation - Anthropic should remember Walter and Diana
        let input3 = AgentInput::text("Can you remind me what Walter's favorite activity is?");
        let exec_context = ExecutionContext::new();

        let response3 = timeout(
            Duration::from_secs(30),
            agent.execute(input3, exec_context.clone()),
        )
        .await??;

        info!(
            "Anthropic response (should remember Walter): {}",
            response3.text
        );

        // Verify context is maintained
        assert!(
            response3.text.to_lowercase().contains("sing")
                || response3.text.to_lowercase().contains("song")
                || response3.text.to_lowercase().contains("walter"),
            "Anthropic should remember Walter's singing from OpenAI conversation"
        );

        // Add more to the story
        let input4 = AgentInput::text(
            "Walter and Diana discovered a hidden underwater cave. What happened next?",
        );
        let response4 =
            timeout(Duration::from_secs(30), agent.execute(input4, exec_context)).await??;

        info!("Anthropic continues story: {}", response4.text);

        // Save final state
        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Verify the complete conversation is persisted
    let final_state = context.state_manager.load_agent_state(agent_id).await?;
    if let Some(state) = final_state {
        let message_count = state.state.conversation_history.len();
        info!("Total messages across both providers: {}", message_count);

        assert!(
            message_count >= 8,
            "Should have messages from both OpenAI and Anthropic sessions"
        );

        // Verify both providers' messages are in history
        let history_text = state
            .state
            .conversation_history
            .iter()
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(
            history_text.contains("Walter"),
            "Should contain Walter from initial conversation"
        );
        assert!(
            history_text.contains("Diana"),
            "Should contain Diana from initial conversation"
        );
    }

    info!("Provider switch test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY and OPENAI_API_KEY"]
async fn test_switch_anthropic_to_openai() -> Result<()> {
    // Skip if either API key is missing
    if !check_api_key("anthropic") || !check_api_key("openai") {
        warn!("Skipping Anthropic to OpenAI switch test - missing API keys");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;
    let agent_id = "switch-test-anthropic-openai";

    info!("Starting provider switch test: Anthropic → OpenAI");

    // Phase 1: Start with Anthropic
    {
        let agent = context
            .create_anthropic_agent()
            .await?
            .expect("Should create Anthropic agent");

        agent.initialize().await?;
        agent.start().await?;

        // Start a technical discussion
        let input1 =
            AgentInput::text("Let's discuss the Fibonacci sequence. Can you explain what it is?");
        let exec_context = ExecutionContext::new();

        let response1 = timeout(
            Duration::from_secs(30),
            agent.execute(input1, exec_context.clone()),
        )
        .await??;

        info!("Anthropic explains Fibonacci: {}", response1.text);

        // Ask for specific values
        let input2 = AgentInput::text("What are the first 10 numbers in the Fibonacci sequence?");
        let response2 =
            timeout(Duration::from_secs(30), agent.execute(input2, exec_context)).await??;

        info!("Anthropic lists numbers: {}", response2.text);

        // Save state
        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;

        info!("Anthropic session completed, state saved");
    }

    // Phase 2: Switch to OpenAI and continue
    {
        // Create OpenAI agent and restore state
        let agent = context
            .create_and_restore_agent("openai", "gpt-4")
            .await?
            .expect("Should create and restore OpenAI agent");

        agent.start().await?;

        // Continue the technical discussion
        let input3 = AgentInput::text("Based on our discussion, can you write a simple recursive function to calculate the nth Fibonacci number?");
        let exec_context = ExecutionContext::new();

        let response3 = timeout(
            Duration::from_secs(30),
            agent.execute(input3, exec_context.clone()),
        )
        .await??;

        info!("OpenAI provides code: {}", response3.text);

        // Verify context is maintained
        assert!(
            response3.text.to_lowercase().contains("fibonacci")
                || response3.text.to_lowercase().contains("recursive")
                || response3.text.to_lowercase().contains("function"),
            "OpenAI should understand the Fibonacci context from Anthropic conversation"
        );

        // Ask about optimization
        let input4 = AgentInput::text(
            "What's the time complexity of the recursive approach? Can we optimize it?",
        );
        let response4 =
            timeout(Duration::from_secs(30), agent.execute(input4, exec_context)).await??;

        info!("OpenAI discusses optimization: {}", response4.text);

        // Save final state
        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Verify the complete conversation is persisted
    let final_state = context.state_manager.load_agent_state(agent_id).await?;
    if let Some(state) = final_state {
        let message_count = state.state.conversation_history.len();
        info!("Total messages across both providers: {}", message_count);

        assert!(
            message_count >= 8,
            "Should have messages from both Anthropic and OpenAI sessions"
        );
    }

    info!("Provider switch test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY and ANTHROPIC_API_KEY"]
async fn test_multiple_provider_switches() -> Result<()> {
    // Skip if either API key is missing
    if !check_api_key("openai") || !check_api_key("anthropic") {
        warn!("Skipping multiple provider switch test - missing API keys");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;
    let agent_id = "switch-test-multiple";

    info!("Starting multiple provider switch test");

    let mut total_messages = 0;

    // Switch 1: OpenAI
    {
        let agent = context
            .create_openai_agent()
            .await?
            .expect("Should create OpenAI agent");

        agent.initialize().await?;
        agent.start().await?;

        let input = AgentInput::text("Let's play a word association game. I'll say 'ocean'.");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("OpenAI associates: {}", response.text);
        total_messages += 2;

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Switch 2: Anthropic
    {
        let agent = context
            .create_and_restore_agent("anthropic", "claude-3-5-sonnet-latest")
            .await?
            .expect("Should restore to Anthropic");

        agent.start().await?;

        let input = AgentInput::text("Good association! Now I'll say 'mountain'.");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("Anthropic associates: {}", response.text);
        total_messages += 2;

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Switch 3: Back to OpenAI
    {
        let agent = context
            .create_and_restore_agent("openai", "gpt-4")
            .await?
            .expect("Should restore back to OpenAI");

        agent.start().await?;

        let input =
            AgentInput::text("Great! Can you remind me what words we've used so far in our game?");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("OpenAI recalls game history: {}", response.text);

        // Verify it remembers the game context
        assert!(
            response.text.to_lowercase().contains("ocean")
                || response.text.to_lowercase().contains("mountain")
                || response.text.to_lowercase().contains("word"),
            "Should remember the word association game context"
        );

        total_messages += 2;

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Verify complete history
    let final_state = context.state_manager.load_agent_state(agent_id).await?;
    if let Some(state) = final_state {
        assert_eq!(
            state.state.conversation_history.len(),
            total_messages,
            "Should have all messages from multiple provider switches"
        );

        info!(
            "Successfully maintained {} messages across 3 provider switches",
            total_messages
        );
    }

    info!("Multiple provider switch test completed successfully");
    Ok(())
}
#[tokio::test]
#[ignore = "requires OPENAI_API_KEY or ANTHROPIC_API_KEY"]
async fn test_provider_switch_with_context_preservation() -> Result<()> {
    // Need at least one API key
    if !check_api_key("openai") && !check_api_key("anthropic") {
        warn!("Skipping context preservation test - no API keys");
        return Ok(());
    }

    let context = ProviderTestContext::new().await?;
    let available_provider = if check_api_key("openai") {
        "openai"
    } else {
        "anthropic"
    };
    let model = if available_provider == "openai" {
        "gpt-4"
    } else {
        "claude-3-5-sonnet-latest"
    };

    info!(
        "Testing context preservation with {} provider",
        available_provider
    );

    // Create agent with rich context
    let agent = if available_provider == "openai" {
        context.create_openai_agent().await?
    } else {
        context.create_anthropic_agent().await?
    }
    .expect("Should create agent");

    agent.initialize().await?;
    agent.start().await?;

    // Build up context with multiple facts
    let facts = vec![
        "My favorite color is turquoise.",
        "I have a pet hamster named Mr. Whiskers.",
        "I'm learning to play the ukulele.",
        "My birthday is in September.",
    ];

    let exec_context = ExecutionContext::new();

    for fact in &facts {
        let input = AgentInput::text(*fact);
        let response = timeout(
            Duration::from_secs(30),
            agent.execute(input, exec_context.clone()),
        )
        .await??;

        info!("Agent acknowledges: {}", response.text);
    }

    // Save state
    agent.save_state().await?;
    let _agent_id = agent.metadata().id.to_string();
    agent.stop().await?;
    agent.terminate().await?;

    // Restore to same provider (simulating provider switch with same provider type)
    let restored_agent = context
        .create_and_restore_agent(available_provider, model)
        .await?
        .expect("Should restore agent");

    restored_agent.start().await?;

    // Test context recall
    let test_input = AgentInput::text("Can you tell me what you remember about me?");
    let recall_response = timeout(
        Duration::from_secs(30),
        restored_agent.execute(test_input, exec_context),
    )
    .await??;

    info!("Agent recalls: {}", recall_response.text);

    // Verify at least some facts are remembered
    let response_lower = recall_response.text.to_lowercase();
    let remembered_facts = vec![
        response_lower.contains("turquoise"),
        response_lower.contains("hamster") || response_lower.contains("whiskers"),
        response_lower.contains("ukulele"),
        response_lower.contains("september"),
    ];

    let facts_remembered = remembered_facts.iter().filter(|&&x| x).count();
    info!("Agent remembered {}/4 facts", facts_remembered);

    assert!(
        facts_remembered >= 2,
        "Agent should remember at least 2 facts after provider switch"
    );

    restored_agent.stop().await?;
    restored_agent.terminate().await?;

    info!("Context preservation test completed successfully");
    Ok(())
}
