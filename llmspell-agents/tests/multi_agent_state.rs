// ABOUTME: Multi-agent state integration tests validating cross-agent interactions
// ABOUTME: Tests agent coordination, state sharing, and isolation in multi-agent scenarios

use anyhow::Result;
use llmspell_agents::{agents::basic::BasicAgent, builder::AgentBuilder, state::StatePersistence};
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_kernel::state::config::SqliteConfig;
use llmspell_kernel::state::{PersistenceConfig, StateManager, StateScope, StorageBackendType};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::sync::RwLock;

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_persistence_config() -> PersistenceConfig {
        PersistenceConfig {
            enabled: true,
            ..PersistenceConfig::default()
        }
    }
    #[tokio::test]
    async fn test_multi_agent_state_isolation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sqlite(SqliteConfig {
                    path: storage_path.join("state_db.db"),
                }),
                PersistenceConfig::default(),
                None, // No memory manager for this test
            )
            .await?,
        );

        // Create multiple agents with same state manager
        let config1 = AgentBuilder::basic("agent-1")
            .description("First test agent")
            .build()?;
        let agent1 = BasicAgent::new(config1)?;
        agent1.set_state_manager(state_manager.clone());

        let config2 = AgentBuilder::basic("agent-2")
            .description("Second test agent")
            .build()?;
        let agent2 = BasicAgent::new(config2)?;
        agent2.set_state_manager(state_manager.clone());

        let config3 = AgentBuilder::basic("agent-3")
            .description("Third test agent")
            .build()?;
        let agent3 = BasicAgent::new(config3)?;
        agent3.set_state_manager(state_manager.clone());

        // Initialize all agents
        agent1.initialize().await?;
        agent2.initialize().await?;
        agent3.initialize().await?;

        // Agent 1 saves state
        state_manager
            .set(
                StateScope::Agent("agent-1".to_string()),
                "preference",
                serde_json::json!("blue"),
            )
            .await?;
        agent1.save_state().await?;

        // Agent 2 saves different state
        state_manager
            .set(
                StateScope::Agent("agent-2".to_string()),
                "preference",
                serde_json::json!("red"),
            )
            .await?;
        agent2.save_state().await?;

        // Agent 3 saves yet another state
        state_manager
            .set(
                StateScope::Agent("agent-3".to_string()),
                "preference",
                serde_json::json!("green"),
            )
            .await?;
        agent3.save_state().await?;

        // Verify state isolation
        let agent1_pref = state_manager
            .get(StateScope::Agent("agent-1".to_string()), "preference")
            .await?
            .expect("Agent 1 preference should exist");
        assert_eq!(agent1_pref, serde_json::json!("blue"));

        let agent2_pref = state_manager
            .get(StateScope::Agent("agent-2".to_string()), "preference")
            .await?
            .expect("Agent 2 preference should exist");
        assert_eq!(agent2_pref, serde_json::json!("red"));

        let agent3_pref = state_manager
            .get(StateScope::Agent("agent-3".to_string()), "preference")
            .await?
            .expect("Agent 3 preference should exist");
        assert_eq!(agent3_pref, serde_json::json!("green"));

        // Verify each agent has its own state space
        let agent1_keys = state_manager
            .list_keys(StateScope::Agent("agent-1".to_string()))
            .await?;
        let agent2_keys = state_manager
            .list_keys(StateScope::Agent("agent-2".to_string()))
            .await?;
        let agent3_keys = state_manager
            .list_keys(StateScope::Agent("agent-3".to_string()))
            .await?;

        // Each agent should have its own keys
        assert!(!agent1_keys.is_empty());
        assert!(!agent2_keys.is_empty());
        assert!(!agent3_keys.is_empty());

        Ok(())
    }
    #[tokio::test]
    async fn test_agent_state_sharing_patterns() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sqlite(SqliteConfig {
                    path: storage_path.join("state_db.db"),
                }),
                PersistenceConfig::default(),
                None, // No memory manager for this test
            )
            .await?,
        );

        // Create shared context in global scope
        state_manager
            .set(
                StateScope::Global,
                "shared_context",
                serde_json::json!({
                    "topic": "collaborative task",
                    "participants": []
                }),
            )
            .await?;

        // Create multiple agents that can read/write shared state
        let mut agents = vec![];
        for i in 0..3 {
            let config = AgentBuilder::basic(format!("collaborator-{i}"))
                .description(format!("Collaborative agent {i}"))
                .build()?;
            let agent = BasicAgent::new(config)?;
            agent.set_state_manager(state_manager.clone());
            agent.initialize().await?;
            agents.push(agent);
        }

        // Each agent updates the shared context
        for (i, agent) in agents.iter_mut().enumerate() {
            // Read current shared state
            let mut shared = state_manager
                .get(StateScope::Global, "shared_context")
                .await?
                .unwrap();

            // Update participants list
            if let Some(participants) = shared
                .get_mut("participants")
                .and_then(|v| v.as_array_mut())
            {
                participants.push(serde_json::json!(format!("collaborator-{}", i)));
            }

            // Write back
            state_manager
                .set(StateScope::Global, "shared_context", shared)
                .await?;

            // Agent also maintains its own state
            agent.save_state().await?;
        }

        // Verify shared state has all participants
        let final_shared = state_manager
            .get(StateScope::Global, "shared_context")
            .await?
            .unwrap();
        let participants = final_shared["participants"].as_array().unwrap();
        assert_eq!(participants.len(), 3);

        Ok(())
    }
    #[tokio::test]
    async fn test_agent_state_persistence_across_restart() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        // Phase 1: Create agent and save state
        let agent_id = "persistent-agent";
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sqlite(SqliteConfig {
                        path: storage_path.join("state_db.db"),
                    }),
                    enabled_persistence_config(),
                    None, // No memory manager for this test
                )
                .await?,
            );

            let config = AgentBuilder::basic(agent_id)
                .description("Agent with persistent state")
                .build()?;
            let agent = BasicAgent::new(config)?;
            agent.set_state_manager(state_manager.clone());
            agent.initialize().await?;
            agent.start().await?;

            // Execute some operations
            let context = ExecutionContext::new();
            let input1 = AgentInput::text("Remember my name is Alice");
            let _ = agent.execute(input1, context.clone()).await?;

            let input2 = AgentInput::text("My favorite color is purple");
            let _ = agent.execute(input2, context).await?;

            // Save state
            agent.save_state().await?;

            // Also save custom state
            state_manager
                .set(
                    StateScope::Agent(agent_id.to_string()),
                    "custom_memory",
                    serde_json::json!({
                        "user_name": "Alice",
                        "favorite_color": "purple",
                        "interaction_count": 2
                    }),
                )
                .await?;
        }

        // Phase 2: Create new agent instance and restore state
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sqlite(SqliteConfig {
                        path: storage_path.join("state_db.db"),
                    }),
                    enabled_persistence_config(),
                    None, // No memory manager for this test
                )
                .await?,
            );

            let config = AgentBuilder::basic(agent_id)
                .description("Agent with persistent state")
                .build()?;
            let agent = BasicAgent::new(config)?;
            agent.set_state_manager(state_manager.clone());
            agent.initialize().await?;

            // Load previous state
            agent.load_state().await?;

            // Verify custom memory was persisted
            let custom_memory = state_manager
                .get(StateScope::Agent(agent_id.to_string()), "custom_memory")
                .await?
                .expect("Custom memory should exist");

            assert_eq!(custom_memory["user_name"], "Alice");
            assert_eq!(custom_memory["favorite_color"], "purple");
            assert_eq!(custom_memory["interaction_count"], 2);

            // Verify agent can continue from previous state
            let context = ExecutionContext::new();
            let input3 = AgentInput::text("What is my name?");
            let response = agent.execute(input3, context).await?;

            // Agent should have context from previous session
            assert!(!response.text.is_empty());
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_concurrent_multi_agent_state_access() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sqlite(SqliteConfig {
                    path: storage_path.join("state_db.db"),
                }),
                PersistenceConfig::default(),
                None, // No memory manager for this test
            )
            .await?,
        );

        // Shared task queue
        let task_queue = Arc::new(RwLock::new(vec![
            "task1", "task2", "task3", "task4", "task5", "task6", "task7", "task8", "task9",
            "task10",
        ]));

        let mut handles = vec![];

        // Spawn multiple agent workers
        for worker_id in 0..5 {
            let sm = state_manager.clone();
            let queue = task_queue.clone();

            let handle = tokio::spawn(async move {
                let agent_id = format!("worker-{worker_id}");

                loop {
                    // Try to claim a task
                    let task = {
                        let mut queue_guard = queue.write().await;
                        queue_guard.pop()
                    };

                    let Some(task) = task else { break };

                    // Process task
                    sm.set(
                        StateScope::Agent(agent_id.clone()),
                        &format!("processed_{task}"),
                        serde_json::json!({
                            "task": task,
                            "processed_at": "2024-01-01T00:00:00Z",
                            "worker": agent_id.clone()
                        }),
                    )
                    .await
                    .unwrap();

                    // Update global processing stats
                    let stats_key = "processing_stats";
                    let current_stats = sm
                        .get(StateScope::Global, stats_key)
                        .await
                        .unwrap()
                        .unwrap_or_else(|| serde_json::json!({"total": 0}));

                    let new_total = current_stats["total"].as_i64().unwrap_or(0) + 1;
                    sm.set(
                        StateScope::Global,
                        stats_key,
                        serde_json::json!({"total": new_total}),
                    )
                    .await
                    .unwrap();

                    // Small delay to simulate processing
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            });

            handles.push(handle);
        }

        // Wait for all workers
        for handle in handles {
            handle.await?;
        }

        // Verify all tasks were processed
        let stats = state_manager
            .get(StateScope::Global, "processing_stats")
            .await?
            .unwrap();
        assert_eq!(stats["total"], 10);

        // Verify each worker has processed some tasks
        for worker_id in 0..5 {
            let worker_keys = state_manager
                .list_keys(StateScope::Agent(format!("worker-{worker_id}")))
                .await?;
            assert!(
                !worker_keys.is_empty(),
                "Worker {worker_id} should have processed tasks"
            );
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_agent_state_migration_scenario() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        // Phase 1: Old agent version
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sqlite(SqliteConfig {
                        path: storage_path.join("state_db.db"),
                    }),
                    enabled_persistence_config(),
                    None, // No memory manager for this test
                )
                .await?,
            );

            // Save state in old format
            state_manager
                .set(
                    StateScope::Agent("legacy-agent".to_string()),
                    "state_v1",
                    serde_json::json!({
                        "version": 1,
                        "data": {
                            "conversations": ["Hello", "Hi there"],
                            "context": {"user": "test"}
                        }
                    }),
                )
                .await?;
        }

        // Phase 2: New agent version with migration
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sqlite(SqliteConfig {
                        path: storage_path.join("state_db.db"),
                    }),
                    enabled_persistence_config(),
                    None, // No memory manager for this test
                )
                .await?,
            );

            // Read old state
            let old_state = state_manager
                .get(StateScope::Agent("legacy-agent".to_string()), "state_v1")
                .await?
                .expect("Old state should exist");

            // Migrate to new format
            let migrated_state = serde_json::json!({
                "version": 2,
                "data": {
                    "conversation_history": old_state["data"]["conversations"],
                    "user_context": old_state["data"]["context"],
                    "metadata": {
                        "migrated_from": "v1",
                        "migration_date": "2024-01-01"
                    }
                }
            });

            // Save migrated state
            state_manager
                .set(
                    StateScope::Agent("legacy-agent".to_string()),
                    "state_v2",
                    migrated_state,
                )
                .await?;

            // Verify migration
            let new_state = state_manager
                .get(StateScope::Agent("legacy-agent".to_string()), "state_v2")
                .await?
                .unwrap();

            assert_eq!(new_state["version"], 2);
            assert_eq!(new_state["data"]["conversation_history"][0], "Hello");
            assert_eq!(new_state["data"]["metadata"]["migrated_from"], "v1");
        }

        Ok(())
    }
}
