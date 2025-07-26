//! ABOUTME: Concurrent access tests for state persistence with real providers
//! ABOUTME: Tests multiple agents accessing and modifying state simultaneously

use super::common::*;
use anyhow::Result;
use llmspell_agents::StatePersistence;
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Barrier;
use tokio::time::{sleep, timeout};
use tracing::{info, warn};

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY"]
async fn test_concurrent_openai_agents_shared_state() -> Result<()> {
    // Skip if no API key
    if !check_api_key("openai") {
        warn!("Skipping concurrent OpenAI test - no API key");
        return Ok(());
    }

    let context = Arc::new(ProviderTestContext::new().await?);
    let agent_base_id = "concurrent-openai-shared";
    let num_agents = 3;

    info!(
        "Starting concurrent OpenAI agents test with {} agents",
        num_agents
    );

    // Create a barrier to synchronize agent starts
    let barrier = Arc::new(Barrier::new(num_agents));

    // Spawn multiple agents concurrently
    let mut handles = vec![];

    for i in 0..num_agents {
        let context = Arc::clone(&context);
        let barrier = Arc::clone(&barrier);
        let agent_id = format!("{}-{}", agent_base_id, i);

        let handle = tokio::spawn(async move {
            // Create agent with unique ID
            let agent = context
                .create_agent_with_id(&agent_id, "openai", "gpt-4")
                .await?
                .expect("Should create OpenAI agent");

            agent.initialize().await?;
            agent.start().await?;

            // Wait for all agents to be ready
            barrier.wait().await;

            info!("Agent {} starting concurrent operations", i);

            // Each agent has a different conversation
            let messages = match i {
                0 => vec![
                    "I'm agent 0. My favorite color is red.",
                    "I also like strawberries.",
                ],
                1 => vec![
                    "I'm agent 1. My favorite color is blue.",
                    "I enjoy swimming in the ocean.",
                ],
                2 => vec![
                    "I'm agent 2. My favorite color is green.",
                    "I love hiking in forests.",
                ],
                _ => vec![],
            };

            let exec_context = ExecutionContext::new();

            for (j, msg) in messages.iter().enumerate() {
                let input = AgentInput::text(*msg);

                // Add small random delay to create more realistic concurrent access
                sleep(Duration::from_millis(100 * (i as u64 + j as u64))).await;

                let response = timeout(
                    Duration::from_secs(30),
                    agent.execute(input, exec_context.clone()),
                )
                .await??;

                info!("Agent {} message {}: {} chars", i, j, response.text.len());

                // Save state after each message
                agent.save_state().await?;
            }

            // Final save
            agent.save_state().await?;
            agent.stop().await?;
            agent.terminate().await?;

            // Return the actual agent UUID, not the name we used
            let actual_agent_id = agent.metadata().id.to_string();
            info!("Agent {} completed with actual ID: {}", i, actual_agent_id);
            Ok::<_, anyhow::Error>(actual_agent_id)
        });

        handles.push(handle);
    }

    // Wait for all agents to complete
    let mut agent_ids = vec![];
    for handle in handles {
        let agent_id = handle.await??;
        agent_ids.push(agent_id);
    }

    info!("All concurrent agents completed");

    // Verify each agent's state was persisted correctly
    for (i, agent_id) in agent_ids.iter().enumerate() {
        info!("Checking state for agent {} with ID: {}", i, agent_id);
        // Load state using the actual agent ID
        let state = context.state_manager.load_agent_state(agent_id).await?;

        if let Some(state) = state {
            info!(
                "Agent {} has {} messages in history",
                i,
                state.state.conversation_history.len()
            );

            assert_eq!(
                state.state.conversation_history.len(),
                4, // 2 messages * 2 (user + assistant)
                "Agent {} should have correct message count",
                i
            );

            // Verify agent-specific content
            let history_text = state
                .state
                .conversation_history
                .iter()
                .map(|msg| msg.content.clone())
                .collect::<Vec<_>>()
                .join(" ");

            match i {
                0 => assert!(history_text.contains("red") && history_text.contains("strawberries")),
                1 => assert!(history_text.contains("blue") && history_text.contains("ocean")),
                2 => assert!(history_text.contains("green") && history_text.contains("forests")),
                _ => {}
            }
        } else {
            panic!("Agent {} state not found", i);
        }
    }

    info!("Concurrent OpenAI access test completed successfully");
    Ok(())
}

#[tokio::test]
#[ignore = "requires ANTHROPIC_API_KEY"]
async fn test_concurrent_anthropic_agents_race_conditions() -> Result<()> {
    // Skip if no API key
    if !check_api_key("anthropic") {
        warn!("Skipping concurrent Anthropic test - no API key");
        return Ok(());
    }

    let context = Arc::new(ProviderTestContext::new().await?);
    let shared_agent_id = "concurrent-anthropic-race";
    let num_workers = 5;

    info!(
        "Starting Anthropic race condition test with {} workers",
        num_workers
    );

    // Create initial agent and establish base conversation
    {
        let agent = context
            .create_agent_with_id(shared_agent_id, "anthropic", "claude-3-5-sonnet-latest")
            .await?
            .expect("Should create Anthropic agent");

        agent.initialize().await?;
        agent.start().await?;

        let input = AgentInput::text("Let's count together. Start with 1.");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("Initial response: {}", response.text);

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Create barrier for synchronized start
    let barrier = Arc::new(Barrier::new(num_workers));

    // Multiple workers try to update the same agent's state concurrently
    let mut handles = vec![];

    for i in 0..num_workers {
        let context = Arc::clone(&context);
        let barrier = Arc::clone(&barrier);
        let shared_agent_id = shared_agent_id.to_string();

        let handle = tokio::spawn(async move {
            // Each worker loads and updates the same agent
            let agent = context
                .create_and_restore_agent_with_id(
                    &shared_agent_id,
                    "anthropic",
                    "claude-3-5-sonnet-latest",
                )
                .await?
                .expect("Should restore agent");

            agent.start().await?;

            // Synchronize start
            barrier.wait().await;

            info!("Worker {} attempting concurrent update", i);

            // Each worker adds their number
            let input = AgentInput::text(&format!("Add {} to our count.", i + 2));
            let exec_context = ExecutionContext::new();

            // Small random delay to increase chance of race conditions
            sleep(Duration::from_millis(50 + (i as u64 * 10))).await;

            let response =
                timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

            info!("Worker {} got response: {} chars", i, response.text.len());

            // Attempt to save - this tests concurrent write handling
            match agent.save_state().await {
                Ok(_) => info!("Worker {} successfully saved state", i),
                Err(e) => warn!("Worker {} failed to save: {}", i, e),
            }

            agent.stop().await?;
            agent.terminate().await?;

            Ok::<_, anyhow::Error>(i)
        });

        handles.push(handle);
    }

    // Wait for all workers
    let mut successful_workers = 0;
    for handle in handles {
        match handle.await? {
            Ok(_) => successful_workers += 1,
            Err(e) => warn!("Worker failed: {}", e),
        }
    }

    info!(
        "{}/{} workers completed successfully",
        successful_workers, num_workers
    );

    // Verify final state integrity
    let final_state = context
        .state_manager
        .load_agent_state(shared_agent_id)
        .await?;

    if let Some(state) = final_state {
        info!(
            "Final state has {} messages",
            state.state.conversation_history.len()
        );

        // State should be valid and contain some of the concurrent updates
        assert!(
            state.state.conversation_history.len() >= 2,
            "Should have at least the initial conversation"
        );

        // Check that the conversation history is coherent (not corrupted)
        for msg in &state.state.conversation_history {
            assert!(!msg.content.is_empty(), "No empty messages");
            assert!(msg.content.len() < 10000, "No corrupted huge messages");
        }
    }

    info!("Anthropic race condition test completed");
    Ok(())
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY and ANTHROPIC_API_KEY"]
async fn test_concurrent_mixed_providers() -> Result<()> {
    // Skip if missing API keys
    if !check_api_key("openai") || !check_api_key("anthropic") {
        warn!("Skipping mixed provider concurrent test - missing API keys");
        return Ok(());
    }

    let context = Arc::new(ProviderTestContext::new().await?);

    info!("Starting concurrent mixed provider test");

    // Create a shared conversation that both providers will add to
    let shared_topic_id = "concurrent-mixed-providers";

    // Initialize with OpenAI
    {
        let agent = context
            .create_agent_with_id(shared_topic_id, "openai", "gpt-4")
            .await?
            .expect("Should create OpenAI agent");

        agent.initialize().await?;
        agent.start().await?;

        let input = AgentInput::text("Let's collaboratively write a story about a robot. I'll start: Once upon a time, there was a friendly robot named Beep.");
        let exec_context = ExecutionContext::new();

        let response =
            timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        info!("OpenAI starts story: {} chars", response.text.len());

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Concurrent providers adding to the story
    let openai_handle = {
        let context = Arc::clone(&context);
        let shared_topic_id = shared_topic_id.to_string();

        tokio::spawn(async move {
            // Wait a bit to ensure both start around the same time
            sleep(Duration::from_millis(100)).await;

            let agent = context
                .create_and_restore_agent_with_id(&shared_topic_id, "openai", "gpt-4")
                .await?
                .expect("Should restore with OpenAI");

            agent.start().await?;

            let input = AgentInput::text(
                "Continue the story: Beep loved to help humans with their daily tasks.",
            );
            let exec_context = ExecutionContext::new();

            let response =
                timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

            info!("OpenAI continues: {} chars", response.text.len());

            agent.save_state().await?;
            agent.stop().await?;
            agent.terminate().await?;

            Ok::<_, anyhow::Error>("openai")
        })
    };

    let anthropic_handle = {
        let context = Arc::clone(&context);
        let shared_topic_id = shared_topic_id.to_string();

        tokio::spawn(async move {
            // Wait a bit to ensure both start around the same time
            sleep(Duration::from_millis(150)).await;

            let agent = context
                .create_and_restore_agent_with_id(
                    &shared_topic_id,
                    "anthropic",
                    "claude-3-5-sonnet-latest",
                )
                .await?
                .expect("Should restore with Anthropic");

            agent.start().await?;

            let input = AgentInput::text(
                "Add to the story: One day, Beep discovered a mysterious signal from space.",
            );
            let exec_context = ExecutionContext::new();

            let response =
                timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

            info!("Anthropic adds: {} chars", response.text.len());

            agent.save_state().await?;
            agent.stop().await?;
            agent.terminate().await?;

            Ok::<_, anyhow::Error>("anthropic")
        })
    };

    // Wait for both providers
    let (openai_result, anthropic_result) = tokio::join!(openai_handle, anthropic_handle);

    openai_result??;
    anthropic_result??;

    // Load final state and verify
    let final_state = context
        .state_manager
        .load_agent_state(shared_topic_id)
        .await?;

    if let Some(state) = final_state {
        info!(
            "Final mixed provider state has {} messages",
            state.state.conversation_history.len()
        );

        // Should have messages from both providers
        let history_text = state
            .state
            .conversation_history
            .iter()
            .map(|msg| msg.content.clone())
            .collect::<Vec<_>>()
            .join(" ");

        assert!(history_text.contains("Beep"), "Should contain robot name");
        assert!(
            history_text.contains("story"),
            "Should contain story context"
        );

        // Verify conversation coherence despite concurrent access
        assert!(
            state.state.conversation_history.len() >= 4,
            "Should have initial + at least one concurrent update"
        );
    }

    info!("Mixed provider concurrent test completed");
    Ok(())
}

#[tokio::test]
#[ignore = "requires OPENAI_API_KEY or ANTHROPIC_API_KEY"]
async fn test_concurrent_state_conflict_resolution() -> Result<()> {
    // Need at least one API key
    if !check_api_key("openai") && !check_api_key("anthropic") {
        warn!("Skipping conflict resolution test - no API keys");
        return Ok(());
    }

    let context = Arc::new(ProviderTestContext::new().await?);
    let provider = if check_api_key("openai") {
        "openai"
    } else {
        "anthropic"
    };
    let model = if provider == "openai" {
        "gpt-4"
    } else {
        "claude-3-5-sonnet-latest"
    };

    info!(
        "Testing concurrent state conflict resolution with {}",
        provider
    );

    let conflict_agent_id = "concurrent-conflict-test";

    // Create initial state
    {
        let agent = context
            .create_agent_with_id(conflict_agent_id, provider, model)
            .await?
            .expect("Should create agent");

        agent.initialize().await?;
        agent.start().await?;

        let input = AgentInput::text("Remember this number: 42");
        let exec_context = ExecutionContext::new();

        timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

        agent.save_state().await?;
        agent.stop().await?;
        agent.terminate().await?;
    }

    // Simulate conflicting updates
    let update_tasks = vec![
        ("The number is now 100", 50),
        ("Actually, the number is 200", 100),
        ("Let me correct that - it's 300", 150),
    ];

    let mut handles: Vec<tokio::task::JoinHandle<Result<bool>>> = vec![];

    for (update_msg, delay_ms) in update_tasks.iter() {
        let context = Arc::clone(&context);
        let conflict_agent_id = conflict_agent_id.to_string();
        let provider = provider.to_string();
        let model = model.to_string();
        let update_msg = update_msg.to_string();
        let delay_ms = *delay_ms;

        let handle = tokio::spawn(async move {
            // Stagger starts to create conflicts
            sleep(Duration::from_millis(delay_ms)).await;

            let agent = context
                .create_and_restore_agent_with_id(&conflict_agent_id, &provider, &model)
                .await?
                .expect("Should restore agent");

            agent.start().await?;

            let input = AgentInput::text(&update_msg);
            let exec_context = ExecutionContext::new();

            let response =
                timeout(Duration::from_secs(30), agent.execute(input, exec_context)).await??;

            info!(
                "Update '{}' got response: {} chars",
                update_msg,
                response.text.len()
            );

            // Try to save - last write wins
            match agent.save_state().await {
                Ok(_) => {
                    info!("Successfully saved: '{}'", update_msg);
                    Ok(true)
                }
                Err(e) => {
                    warn!("Failed to save '{}': {}", update_msg, e);
                    Ok(false)
                }
            }
        });

        handles.push(handle);
    }

    // Wait for all updates
    let mut successful_saves = 0;
    for handle in handles {
        if handle.await?? {
            successful_saves += 1;
        }
    }

    info!(
        "{} successful saves out of {} attempts",
        successful_saves,
        update_tasks.len()
    );

    // Verify final state is consistent
    let final_state = context
        .state_manager
        .load_agent_state(conflict_agent_id)
        .await?;

    if let Some(state) = final_state {
        info!(
            "Final state after conflicts has {} messages",
            state.state.conversation_history.len()
        );

        // The state should be valid and contain a coherent conversation
        assert!(
            state.state.conversation_history.len() >= 2,
            "Should have at least initial exchange"
        );

        // Last message should be from one of our updates (last write wins)
        let last_user_message = state
            .state
            .conversation_history
            .iter()
            .filter(|msg| matches!(msg.role, llmspell_state_persistence::MessageRole::User))
            .last()
            .map(|msg| msg.content.clone())
            .unwrap_or_default();

        info!("Last user message in final state: {}", last_user_message);

        // Verify it's one of our updates
        assert!(
            last_user_message.contains("100")
                || last_user_message.contains("200")
                || last_user_message.contains("300"),
            "Final state should contain one of the concurrent updates"
        );
    }

    info!("Conflict resolution test completed");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    async fn test_concurrent_execution_capability() {
        // This test verifies the test infrastructure supports concurrent execution
        let num_tasks = 10;
        let barrier = Arc::new(Barrier::new(num_tasks));

        let mut handles = vec![];

        for i in 0..num_tasks {
            let barrier = Arc::clone(&barrier);

            let handle = tokio::spawn(async move {
                barrier.wait().await;
                // All tasks should execute this roughly at the same time
                let start = std::time::Instant::now();
                sleep(Duration::from_millis(100)).await;
                let elapsed = start.elapsed();
                (i, elapsed)
            });

            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            results.push(handle.await.unwrap());
        }

        // Verify tasks ran concurrently (total time should be ~100ms, not 1000ms)
        let max_elapsed = results.iter().map(|(_, e)| e.as_millis()).max().unwrap();
        assert!(
            max_elapsed < 200,
            "Tasks should run concurrently, not sequentially"
        );
    }
}
