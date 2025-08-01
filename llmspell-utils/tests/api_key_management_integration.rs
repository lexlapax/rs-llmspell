//! ABOUTME: Integration tests for API key management system
//! ABOUTME: Tests persistent storage, key rotation, and audit logging

use chrono::Utc;
use llmspell_utils::{
    ApiKeyAction, ApiKeyManager, ApiKeyMetadata, ApiKeyStorage, PersistentApiKeyStorage,
};
use std::collections::HashMap;
use tempfile::TempDir;

fn create_test_metadata(service: &str) -> ApiKeyMetadata {
    ApiKeyMetadata {
        key_id: format!("{}_key", service),
        service: service.to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: None,
        is_active: true,
        usage_count: 0,
    }
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_api_key_manager_basic_operations() {
    let manager = ApiKeyManager::new();

    // Add a key
    let metadata = create_test_metadata("test_service");
    manager
        .add_key("test_service_key", "test_api_key_123", metadata)
        .expect("Failed to add key");

    // Get the key
    let key = manager.get_key("test_service").expect("Failed to get key");
    assert_eq!(key, Some("test_api_key_123".to_string()));

    // Check audit log
    let audit_log = manager.get_audit_log(Some(10));
    assert!(!audit_log.is_empty());

    // The audit log should have both Create and Read actions
    // Find the Create action (it might not be the first if log is in reverse order)
    let create_entry = audit_log
        .iter()
        .find(|entry| entry.action == ApiKeyAction::Create && entry.service == "test_service")
        .expect("Create action not found in audit log");

    assert_eq!(create_entry.service, "test_service");
    assert_eq!(create_entry.action, ApiKeyAction::Create);
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_api_key_rotation() {
    let manager = ApiKeyManager::new();

    // Add initial key
    let metadata = create_test_metadata("rotation_service");
    manager
        .add_key("rotation_service_key", "old_key_123", metadata)
        .expect("Failed to add key");

    // Rotate the key
    manager
        .rotate_key("rotation_service", "new_key_456")
        .expect("Failed to rotate key");

    // Verify new key is active
    let key = manager
        .get_key("rotation_service")
        .expect("Failed to get key");
    assert_eq!(key, Some("new_key_456".to_string()));

    // Check audit log for rotation
    let audit_log = manager.get_audit_log(Some(10));
    let rotation_entry = audit_log
        .iter()
        .find(|e| e.action == ApiKeyAction::Rotate)
        .expect("Rotation entry not found");
    assert_eq!(rotation_entry.service, "rotation_service");
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_environment_variable_loading() {
    let mut manager = ApiKeyManager::new();
    manager.set_env_prefix("LLMSPELL_API_KEY_");

    // Set test environment variables
    std::env::set_var("LLMSPELL_API_KEY_OPENAI", "test_openai_key");
    std::env::set_var("LLMSPELL_API_KEY_ANTHROPIC", "test_anthropic_key");

    // Load from environment
    manager.load_from_env().expect("Failed to load from env");

    // Verify keys were loaded
    assert_eq!(
        manager.get_key("openai").expect("Failed to get key"),
        Some("test_openai_key".to_string())
    );
    assert_eq!(
        manager.get_key("anthropic").expect("Failed to get key"),
        Some("test_anthropic_key".to_string())
    );

    // Clean up
    std::env::remove_var("LLMSPELL_API_KEY_OPENAI");
    std::env::remove_var("LLMSPELL_API_KEY_ANTHROPIC");
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_configuration_file_loading() {
    let manager = ApiKeyManager::new();

    // Create test config
    let mut config = HashMap::new();
    config.insert("github".to_string(), "github_token_123".to_string());
    config.insert("slack".to_string(), "slack_webhook_456".to_string());

    // Load from config
    manager
        .load_from_config(config)
        .expect("Failed to load from config");

    // Verify keys were loaded
    assert_eq!(
        manager.get_key("github").expect("Failed to get key"),
        Some("github_token_123".to_string())
    );
    assert_eq!(
        manager.get_key("slack").expect("Failed to get key"),
        Some("slack_webhook_456".to_string())
    );
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_persistent_storage_integration() {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let encryption_key = [77u8; 32]; // Test encryption key

    // Create manager with persistent storage
    {
        let storage: Box<dyn ApiKeyStorage> = Box::new(
            PersistentApiKeyStorage::new(temp_dir.path(), encryption_key)
                .expect("Failed to create persistent storage"),
        );
        let manager = ApiKeyManager::with_storage(storage);

        // Store some keys
        let metadata1 = create_test_metadata("persistent_service1");
        manager
            .add_key("persistent_service1_key", "key1_value", metadata1)
            .expect("Failed to add key1");

        let metadata2 = create_test_metadata("persistent_service2");
        manager
            .add_key("persistent_service2_key", "key2_value", metadata2)
            .expect("Failed to add key2");
    }

    // Create new manager instance and verify keys persist
    {
        let storage: Box<dyn ApiKeyStorage> = Box::new(
            PersistentApiKeyStorage::new(temp_dir.path(), encryption_key)
                .expect("Failed to create persistent storage"),
        );
        let manager = ApiKeyManager::with_storage(storage);

        // Verify keys still exist
        assert_eq!(
            manager
                .get_key("persistent_service1")
                .expect("Failed to get key"),
            Some("key1_value".to_string())
        );
        assert_eq!(
            manager
                .get_key("persistent_service2")
                .expect("Failed to get key"),
            Some("key2_value".to_string())
        );
    }
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_key_expiration() {
    let manager = ApiKeyManager::new();

    // Set a key with expiration
    let expires_at = Utc::now() + chrono::Duration::hours(1);
    let mut metadata = create_test_metadata("expiring_service");
    metadata.expires_at = Some(expires_at);

    manager
        .add_key("expiring_service_key", "temp_key", metadata)
        .expect("Failed to add key with expiry");

    // Key should be available now
    assert_eq!(
        manager
            .get_key("expiring_service")
            .expect("Failed to get key"),
        Some("temp_key".to_string())
    );

    // Verify metadata has expiration
    let metadata = manager
        .get_metadata("expiring_service_key")
        .expect("Failed to get metadata")
        .expect("Metadata not found");
    assert!(metadata.expires_at.is_some());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_audit_log_limits() {
    let manager = ApiKeyManager::new();

    // Create many audit entries
    for i in 0..20 {
        let metadata = create_test_metadata(&format!("service_{}", i));
        manager
            .add_key(
                &format!("service_{}_key", i),
                &format!("key_{}", i),
                metadata,
            )
            .expect("Failed to add key");
    }

    // Get limited audit log
    let audit_log = manager.get_audit_log(Some(5));
    assert_eq!(audit_log.len(), 5);

    // Get all entries
    let full_log = manager.get_audit_log(None);
    assert!(full_log.len() >= 20);
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_key_usage_tracking() {
    let manager = ApiKeyManager::new();

    // Add a key
    let metadata = create_test_metadata("tracked_service");
    manager
        .add_key("tracked_service_key", "tracked_key", metadata)
        .expect("Failed to add key");

    // Use the key multiple times
    for _ in 0..5 {
        let _ = manager.get_key("tracked_service");
    }

    // Check usage count
    let metadata = manager
        .get_metadata("tracked_service_key")
        .expect("Failed to get metadata")
        .expect("Metadata not found");
    assert_eq!(metadata.usage_count, 5);
    assert!(metadata.last_used.is_some());
}

#[cfg_attr(test_category = "external")]
#[ignore]
#[test]
fn test_invalid_operations() {
    let manager = ApiKeyManager::new();

    // Try to rotate non-existent key
    let result = manager.rotate_key("nonexistent", "new_key");
    assert!(result.is_err());

    // Try to get non-existent key
    let key = manager.get_key("nonexistent").expect("Failed to get key");
    assert!(key.is_none());

    // Try to deactivate non-existent key
    let result = manager.deactivate_key("nonexistent");
    assert!(result.is_err());
}
