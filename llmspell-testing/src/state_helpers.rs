//! ABOUTME: Test helpers for state management and persistence testing
//! ABOUTME: Provides common setup functions and data generators for state-related tests

//! State management test helpers.
//!
//! This module provides common test utilities for testing state management,
//! persistence, and backup functionality across the llmspell framework.
//!
//! # Examples
//!
//! ```rust,no_run
//! use llmspell_testing::state_helpers::{
//!     create_test_state_manager,
//!     create_test_state_manager_with_backup,
//!     populate_test_state_data,
//! };
//! use llmspell_kernel::state::StateScope;
//!
//! # async fn test_example() {
//! // Create a test state manager
//! let state_manager = create_test_state_manager().await;
//!
//! // Populate with test data
//! populate_test_state_data(&state_manager).await.unwrap();
//!
//! // Use in tests
//! let value = state_manager.get(StateScope::Global, "user_settings").await.unwrap();
//! assert!(value.is_some());
//! # }
//! ```

use llmspell_kernel::state::{
    backup::{BackupConfig, BackupManager, CompressionType},
    config::{PersistenceConfig, StorageBackendType},
    manager::StateManager,
    StateScope,
};
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

/// Creates a test StateManager with in-memory backend
pub async fn create_test_state_manager() -> Arc<StateManager> {
    let config = PersistenceConfig {
        enabled: true,
        ..Default::default()
    };

    Arc::new(
        StateManager::with_backend(StorageBackendType::Memory, config)
            .await
            .expect("Failed to create test state manager"),
    )
}

/// Creates a test StateManager with backup capabilities
pub async fn create_test_state_manager_with_backup(
) -> (Arc<StateManager>, Arc<BackupManager>, TempDir) {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let backup_dir = temp_dir.path().join("backups");
    std::fs::create_dir_all(&backup_dir).expect("Failed to create backup dir");

    let persistence_config = PersistenceConfig {
        enabled: true,
        ..Default::default()
    };

    let backup_config = BackupConfig {
        backup_dir: backup_dir.clone(),
        compression_enabled: true,
        compression_type: CompressionType::Zstd,
        compression_level: 3,
        encryption_enabled: false,
        max_backups: Some(5),
        incremental_enabled: true,
        max_backup_age: Some(std::time::Duration::from_secs(7 * 24 * 3600)),
        ..Default::default()
    };

    let state_manager = Arc::new(
        StateManager::with_backend(StorageBackendType::Memory, persistence_config)
            .await
            .expect("Failed to create state manager"),
    );

    let backup_manager = Arc::new(
        BackupManager::new(backup_config, state_manager.clone())
            .expect("Failed to create backup manager"),
    );

    (state_manager, backup_manager, temp_dir)
}

/// Populates a StateManager with common test data
pub async fn populate_test_state_data(
    state_manager: &StateManager,
) -> Result<(), Box<dyn std::error::Error>> {
    // Global settings
    state_manager
        .set(
            StateScope::Global,
            "user_settings",
            json!({
                "theme": "dark",
                "language": "en",
                "notifications": true
            }),
        )
        .await?;

    state_manager
        .set(
            StateScope::Global,
            "session_data",
            json!({
                "user_id": 12345,
                "login_time": "2025-01-27T10:00:00Z",
                "permissions": ["read", "write"]
            }),
        )
        .await?;

    // Agent-specific data
    state_manager
        .set(
            StateScope::Custom("agent_1".to_string()),
            "config",
            json!({
                "model": "gpt-4",
                "temperature": 0.7,
                "max_tokens": 2000
            }),
        )
        .await?;

    state_manager
        .set(
            StateScope::Custom("agent_2".to_string()),
            "history",
            json!({
                "conversations": [
                    {"role": "user", "content": "Hello"},
                    {"role": "assistant", "content": "Hi there!"}
                ],
                "total_tokens": 15
            }),
        )
        .await?;

    Ok(())
}

/// Creates test data for various state scopes
pub fn create_scope_test_data() -> Vec<(StateScope, &'static str, serde_json::Value)> {
    vec![
        (
            StateScope::Global,
            "app_config",
            json!({
                "version": "1.0.0",
                "features": ["chat", "tools", "workflows"]
            }),
        ),
        (
            StateScope::User("test_user".to_string()),
            "preferences",
            json!({
                "theme": "light",
                "timezone": "UTC"
            }),
        ),
        (
            StateScope::Session("session_123".to_string()),
            "context",
            json!({
                "start_time": "2025-01-27T10:00:00Z",
                "active": true
            }),
        ),
        (
            StateScope::Agent("assistant".to_string()),
            "state",
            json!({
                "conversation_count": 5,
                "last_active": "2025-01-27T10:30:00Z"
            }),
        ),
        (
            StateScope::Tool("calculator".to_string()),
            "usage",
            json!({
                "call_count": 42,
                "success_rate": 0.98
            }),
        ),
    ]
}

/// Creates test data for performance testing with specified size
pub fn create_large_test_dataset(num_entries: usize) -> Vec<(String, serde_json::Value)> {
    (0..num_entries)
        .map(|i| {
            (
                format!("item_{}", i),
                json!({
                    "id": i,
                    "data": format!("Large data string for item {} with lots of text content to make the backup more realistic in size and complexity", i),
                    "metadata": {
                        "created_at": "2025-01-27T10:00:00Z",
                        "tags": ["tag1", "tag2", "tag3"],
                        "properties": {
                            "nested": true,
                            "level": i % 10
                        }
                    }
                }),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_state_manager() {
        let state_manager = create_test_state_manager().await;

        // Should be able to set and get values
        state_manager
            .set(StateScope::Global, "test", json!("value"))
            .await
            .unwrap();

        let value = state_manager.get(StateScope::Global, "test").await.unwrap();
        assert_eq!(value, Some(json!("value")));
    }

    #[tokio::test]
    async fn test_populate_test_data() {
        let state_manager = create_test_state_manager().await;
        populate_test_state_data(&state_manager).await.unwrap();

        // Check that data was populated
        let settings = state_manager
            .get(StateScope::Global, "user_settings")
            .await
            .unwrap();
        assert!(settings.is_some());

        let agent_config = state_manager
            .get(StateScope::Custom("agent_1".to_string()), "config")
            .await
            .unwrap();
        assert!(agent_config.is_some());
    }

    #[test]
    fn test_create_scope_test_data() {
        let data = create_scope_test_data();
        assert!(!data.is_empty());

        // Should have various scope types
        let has_global = data.iter().any(|(scope, _, _)| scope.is_global());
        let has_user = data.iter().any(|(scope, _, _)| scope.is_user_scope());
        assert!(has_global);
        assert!(has_user);
    }

    #[test]
    fn test_create_large_dataset() {
        let dataset = create_large_test_dataset(100);
        assert_eq!(dataset.len(), 100);

        // Check that each item is unique
        let keys: Vec<_> = dataset.iter().map(|(k, _)| k).collect();
        let unique_keys: std::collections::HashSet<_> = keys.iter().collect();
        assert_eq!(keys.len(), unique_keys.len());
    }
}
