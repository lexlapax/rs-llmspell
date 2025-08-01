// ABOUTME: Test to verify StateScope::Session functionality
// ABOUTME: Ensures session scope isolation and basic operations work correctly

#[cfg(test)]
#[cfg_attr(test_category = "state")]
mod session_tests {
    use crate::{StateManager, PersistenceConfig, SerializableState};
    use llmspell_state_traits::StateScope;
    use serde_json::json;
    use tempfile::TempDir;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_scope_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            backend_type: crate::StorageBackendType::Disk,
            path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();

        // Test saving to session scope
        let session_id = "test-session-123";
        let session_scope = StateScope::Session(session_id.to_string());
        
        state_manager
            .set(session_scope.clone(), "user_id", json!("user-456"))
            .await
            .unwrap();
        
        state_manager
            .set(session_scope.clone(), "preferences", json!({"theme": "dark"}))
            .await
            .unwrap();

        // Test loading from session scope
        let user_id = state_manager
            .get(session_scope.clone(), "user_id")
            .await
            .unwrap();
        assert_eq!(user_id, Some(json!("user-456")));

        let prefs = state_manager
            .get(session_scope.clone(), "preferences")
            .await
            .unwrap();
        assert_eq!(prefs, Some(json!({"theme": "dark"})));

        // Test session isolation - different session shouldn't see the data
        let other_session = StateScope::Session("other-session-789".to_string());
        let other_user = state_manager
            .get(other_session, "user_id")
            .await
            .unwrap();
        assert_eq!(other_user, None);

        // Test listing keys in session scope
        let session_keys = state_manager
            .list_keys(session_scope.clone())
            .await
            .unwrap();
        assert_eq!(session_keys.len(), 2);
        assert!(session_keys.contains(&"user_id".to_string()));
        assert!(session_keys.contains(&"preferences".to_string()));

        // Test deleting from session scope
        state_manager
            .delete(session_scope.clone(), "user_id")
            .await
            .unwrap();

        let deleted_user = state_manager
            .get(session_scope.clone(), "user_id")
            .await
            .unwrap();
        assert_eq!(deleted_user, None);

        // Verify preferences still exists
        let prefs_after = state_manager
            .get(session_scope, "preferences")
            .await
            .unwrap();
        assert_eq!(prefs_after, Some(json!({"theme": "dark"})));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_scope_isolation_from_global() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            backend_type: crate::StorageBackendType::Disk,
            path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();

        // Save same key in different scopes
        state_manager
            .set(StateScope::Global, "config", json!("global-config"))
            .await
            .unwrap();

        state_manager
            .set(
                StateScope::Session("session-1".to_string()),
                "config",
                json!("session-config")
            )
            .await
            .unwrap();

        // Verify they don't interfere
        let global_config = state_manager
            .get(StateScope::Global, "config")
            .await
            .unwrap();
        assert_eq!(global_config, Some(json!("global-config")));

        let session_config = state_manager
            .get(StateScope::Session("session-1".to_string()), "config")
            .await
            .unwrap();
        assert_eq!(session_config, Some(json!("session-config")));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_clear_session_scope() {
        let temp_dir = TempDir::new().unwrap();
        let config = PersistenceConfig {
            enabled: true,
            backend_type: crate::StorageBackendType::Disk,
            path: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let state_manager = StateManager::new(config).await.unwrap();
        let session_scope = StateScope::Session("cleanup-test".to_string());

        // Add multiple keys to session
        for i in 0..5 {
            state_manager
                .set(
                    session_scope.clone(),
                    &format!("key-{}", i),
                    json!(format!("value-{}", i))
                )
                .await
                .unwrap();
        }

        // Verify all keys exist
        let keys = state_manager.list_keys(session_scope.clone()).await.unwrap();
        assert_eq!(keys.len(), 5);

        // Clear the session scope
        state_manager.clear_scope(session_scope.clone()).await.unwrap();

        // Verify all keys are gone
        let keys_after = state_manager.list_keys(session_scope).await.unwrap();
        assert_eq!(keys_after.len(), 0);
    }
}