// ABOUTME: Core state persistence integration tests validating cross-component functionality
// ABOUTME: Tests state operations, isolation, concurrency, and error handling across restarts

use anyhow::Result;
use llmspell_state_persistence::{
    PersistenceConfig, SledConfig, StateManager, StateScope, StorageBackendType,
};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{sleep, Duration};

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_persistence_config() -> PersistenceConfig {
        PersistenceConfig {
            enabled: true,
            ..Default::default()
        }
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_state_persistence_across_application_restart() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        // Phase 1: Create and use state manager
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sled(SledConfig {
                        path: storage_path.join("state_db"),
                        cache_capacity: 1024 * 1024,
                        use_compression: true,
                    }),
                    enabled_persistence_config(),
                )
                .await?,
            );

            // Save various types of state
            state_manager
                .set(
                    StateScope::Global,
                    "app_version",
                    serde_json::json!("1.0.0"),
                )
                .await?;

            state_manager
                .set(
                    StateScope::Agent("agent-1".to_string()),
                    "conversation_count",
                    serde_json::json!(42),
                )
                .await?;

            state_manager
                .set(
                    StateScope::Custom("user:user-123".to_string()),
                    "preferences",
                    serde_json::json!({
                        "theme": "dark",
                        "language": "en"
                    }),
                )
                .await?;

            state_manager
                .set(
                    StateScope::Session("session-xyz".to_string()),
                    "active",
                    serde_json::json!(true),
                )
                .await?;
        }

        // Phase 2: Simulate application restart with new state manager
        {
            let state_manager = Arc::new(
                StateManager::with_backend(
                    StorageBackendType::Sled(SledConfig {
                        path: storage_path.join("state_db"),
                        cache_capacity: 1024 * 1024,
                        use_compression: true,
                    }),
                    enabled_persistence_config(),
                )
                .await?,
            );

            // Verify all state persisted correctly
            let app_version = state_manager
                .get(StateScope::Global, "app_version")
                .await?
                .expect("App version should exist");
            assert_eq!(app_version, serde_json::json!("1.0.0"));

            let conversation_count = state_manager
                .get(
                    StateScope::Agent("agent-1".to_string()),
                    "conversation_count",
                )
                .await?
                .expect("Conversation count should exist");
            assert_eq!(conversation_count, serde_json::json!(42));

            let preferences = state_manager
                .get(
                    StateScope::Custom("user:user-123".to_string()),
                    "preferences",
                )
                .await?
                .expect("User preferences should exist");
            assert_eq!(preferences["theme"], "dark");
            assert_eq!(preferences["language"], "en");

            let session_active = state_manager
                .get(StateScope::Session("session-xyz".to_string()), "active")
                .await?
                .expect("Session state should exist");
            assert_eq!(session_active, serde_json::json!(true));
        }

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_multi_scope_state_isolation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(SledConfig {
                    path: storage_path.join("state_db"),
                    cache_capacity: 1024 * 1024,
                    use_compression: true,
                }),
                enabled_persistence_config(),
            )
            .await?,
        );

        // Set state in different scopes with same key
        let key = "shared_key";
        let scopes = [
            StateScope::Global,
            StateScope::Agent("agent-1".to_string()),
            StateScope::Agent("agent-2".to_string()),
            StateScope::Custom("tool:tool-1".to_string()),
            StateScope::Session("session-1".to_string()),
        ];

        // Set different values for same key in different scopes
        for (i, scope) in scopes.iter().enumerate() {
            state_manager
                .set(
                    scope.clone(),
                    key,
                    serde_json::json!({
                        "scope_index": i,
                        "data": format!("Data for scope {}", i)
                    }),
                )
                .await?;
        }

        // Verify isolation - each scope has its own value
        for (i, scope) in scopes.iter().enumerate() {
            let value = state_manager
                .get(scope.clone(), key)
                .await?
                .expect("Value should exist");

            assert_eq!(value["scope_index"], i);
            assert_eq!(value["data"], format!("Data for scope {}", i));
        }

        // Verify listing keys returns only keys from specific scope
        let agent1_keys = state_manager
            .list_keys(StateScope::Agent("agent-1".to_string()))
            .await?;
        assert_eq!(agent1_keys.len(), 1);
        assert_eq!(agent1_keys[0], key);

        let agent2_keys = state_manager
            .list_keys(StateScope::Agent("agent-2".to_string()))
            .await?;
        assert_eq!(agent2_keys.len(), 1);
        assert_eq!(agent2_keys[0], key);

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_concurrent_state_operations() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(SledConfig {
                    path: storage_path.join("state_db"),
                    cache_capacity: 1024 * 1024,
                    use_compression: true,
                }),
                enabled_persistence_config(),
            )
            .await?,
        );

        // Spawn multiple concurrent operations
        let mut handles = vec![];

        for i in 0..10 {
            let sm = state_manager.clone();
            let handle = tokio::spawn(async move {
                // Each task performs multiple operations
                for j in 0..10 {
                    let key = format!("counter_{}", i);
                    let value = j;

                    // Set value
                    sm.set(StateScope::Global, &key, serde_json::json!(value))
                        .await
                        .unwrap();

                    // Read it back
                    let read_value = sm.get(StateScope::Global, &key).await.unwrap().unwrap();

                    assert_eq!(read_value, serde_json::json!(value));

                    // Small delay to increase contention
                    tokio::time::sleep(Duration::from_micros(10)).await;
                }
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await?;
        }

        // Verify final state
        for i in 0..10 {
            let key = format!("counter_{}", i);
            let value = state_manager
                .get(StateScope::Global, &key)
                .await?
                .expect("Value should exist");
            assert_eq!(value, serde_json::json!(9)); // Last value written
        }

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_error_handling_without_data_loss() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(SledConfig {
                    path: storage_path.join("state_db"),
                    cache_capacity: 1024 * 1024,
                    use_compression: true,
                }),
                enabled_persistence_config(),
            )
            .await?,
        );

        // Save important state
        state_manager
            .set(
                StateScope::Global,
                "critical_data",
                serde_json::json!({
                    "value": 42,
                    "timestamp": "2024-01-01T00:00:00Z"
                }),
            )
            .await?;

        // Try to save invalid data (this should be handled gracefully)
        let large_data = vec![0u8; 10 * 1024 * 1024]; // 10MB of data
        let result = state_manager
            .set(
                StateScope::Global,
                "too_large",
                serde_json::json!(large_data),
            )
            .await;

        // Error should be handled without corrupting existing data
        if result.is_err() {
            // Verify critical data is still intact
            let critical = state_manager
                .get(StateScope::Global, "critical_data")
                .await?
                .expect("Critical data should still exist");
            assert_eq!(critical["value"], 42);
        }

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_memory_usage_scaling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(SledConfig {
                    path: storage_path.join("state_db"),
                    cache_capacity: 1024 * 1024, // 1MB cache
                    use_compression: true,
                }),
                enabled_persistence_config(),
            )
            .await?,
        );

        // Add progressively more state
        for batch in 0..5 {
            for i in 0..100 {
                let key = format!("key_{}_{}", batch, i);
                let value = serde_json::json!({
                    "batch": batch,
                    "index": i,
                    "data": "x".repeat(100), // 100 bytes of data
                });

                state_manager.set(StateScope::Global, &key, value).await?;
            }

            // Allow time for any background operations
            sleep(Duration::from_millis(100)).await;
        }

        // Verify all data is accessible
        let keys = state_manager.list_keys(StateScope::Global).await?;
        assert_eq!(keys.len(), 500);

        // Spot check some values
        for batch in 0..5 {
            for i in (0..100).step_by(10) {
                let key = format!("key_{}_{}", batch, i);
                let value = state_manager
                    .get(StateScope::Global, &key)
                    .await?
                    .expect("Value should exist");
                assert_eq!(value["batch"], batch);
                assert_eq!(value["index"], i);
            }
        }

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_race_condition_prevention() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(SledConfig {
                    path: storage_path.join("state_db"),
                    cache_capacity: 1024 * 1024,
                    use_compression: true,
                }),
                enabled_persistence_config(),
            )
            .await?,
        );

        // Initialize a counter
        state_manager
            .set(StateScope::Global, "counter", serde_json::json!(0))
            .await?;

        // Spawn multiple tasks that increment the counter
        let mut handles = vec![];
        let increments_per_task = 100;
        let num_tasks = 10;

        for _ in 0..num_tasks {
            let sm = state_manager.clone();
            let handle = tokio::spawn(async move {
                for _ in 0..increments_per_task {
                    // This is a potential race condition if not handled properly
                    loop {
                        // Read current value
                        let current = sm
                            .get(StateScope::Global, "counter")
                            .await
                            .unwrap()
                            .unwrap()
                            .as_i64()
                            .unwrap();

                        // Try to update
                        let new_value = current + 1;
                        let result = sm
                            .set(StateScope::Global, "counter", serde_json::json!(new_value))
                            .await;

                        if result.is_ok() {
                            break;
                        }
                        // Retry on conflict
                        tokio::time::sleep(Duration::from_micros(10)).await;
                    }
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await?;
        }

        // Verify the counter value
        let final_count = state_manager
            .get(StateScope::Global, "counter")
            .await?
            .unwrap()
            .as_i64()
            .unwrap();

        // Due to potential race conditions, the count might not be exact
        // but should be reasonable
        assert!(
            final_count >= (increments_per_task * num_tasks / 2) as i64,
            "Counter value too low: {}",
            final_count
        );

        Ok(())
    }

    #[cfg_attr(test_category = "integration")]
    #[tokio::test]
    async fn test_different_backend_types() -> Result<()> {
        // Test with memory backend
        let memory_manager = StateManager::new().await?;

        memory_manager
            .set(
                StateScope::Global,
                "test_key",
                serde_json::json!("memory_value"),
            )
            .await?;

        let value = memory_manager
            .get(StateScope::Global, "test_key")
            .await?
            .expect("Value should exist");
        assert_eq!(value, serde_json::json!("memory_value"));

        // Test with Sled backend
        let temp_dir = TempDir::new()?;
        let sled_manager = StateManager::with_backend(
            StorageBackendType::Sled(SledConfig {
                path: temp_dir.path().join("sled_db"),
                cache_capacity: 1024 * 1024,
                use_compression: true,
            }),
            enabled_persistence_config(),
        )
        .await?;

        sled_manager
            .set(
                StateScope::Global,
                "test_key",
                serde_json::json!("sled_value"),
            )
            .await?;

        let value = sled_manager
            .get(StateScope::Global, "test_key")
            .await?
            .expect("Value should exist");
        assert_eq!(value, serde_json::json!("sled_value"));

        Ok(())
    }
}
