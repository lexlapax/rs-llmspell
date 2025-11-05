//! Integration tests for PostgreSQL API Keys Storage (Phase 13b.13.2)

#![cfg(feature = "postgres")]

use chrono::{Duration, Utc};
use llmspell_storage::backends::postgres::{
    ApiKeyMetadata, ApiKeyStats, PostgresApiKeyStorage, PostgresBackend, PostgresConfig,
};
use std::sync::Arc;
use tokio::sync::OnceCell;
use uuid::Uuid;

const ADMIN_CONNECTION_STRING: &str =
    "postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev";

const APP_CONNECTION_STRING: &str =
    "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

static MIGRATION_INIT: OnceCell<()> = OnceCell::const_new();

/// Ensure migrations run once before all tests
async fn ensure_migrations_run_once() {
    MIGRATION_INIT
        .get_or_init(|| async {
            let config = PostgresConfig::new(ADMIN_CONNECTION_STRING);
            let backend = PostgresBackend::new(config)
                .await
                .expect("Failed to create backend for migration init");

            backend
                .run_migrations()
                .await
                .expect("Failed to run migrations during test initialization");
        })
        .await;
}

#[tokio::test]
async fn test_api_key_store_and_get() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_store_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let key_id = format!("key_{}", Uuid::new_v4());
    let plaintext_key = "sk-1234567890abcdef";

    let metadata = ApiKeyMetadata {
        key_id: key_id.clone(),
        service: "openai".to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: None,
        is_active: true,
        usage_count: 0,
    };

    // Store key
    storage
        .store(&key_id, plaintext_key, &metadata)
        .await
        .unwrap();

    // Retrieve key
    let retrieved = storage.get(&key_id).await.unwrap();
    assert_eq!(retrieved, Some(plaintext_key.to_string()));

    // Get metadata
    let meta = storage.get_metadata(&key_id).await.unwrap().unwrap();
    assert_eq!(meta.service, "openai");
    assert!(meta.is_active);

    // Cleanup
    storage.delete(&key_id).await.unwrap();
}

#[tokio::test]
async fn test_api_key_update_metadata() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_update_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let key_id = format!("key_{}", Uuid::new_v4());
    let metadata = ApiKeyMetadata {
        key_id: key_id.clone(),
        service: "anthropic".to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: None,
        is_active: true,
        usage_count: 0,
    };

    storage
        .store(&key_id, "sk-ant-test", &metadata)
        .await
        .unwrap();

    // Update metadata
    let mut updated_meta = metadata.clone();
    updated_meta.usage_count = 42;
    updated_meta.last_used = Some(Utc::now());

    storage
        .update_metadata(&key_id, &updated_meta)
        .await
        .unwrap();

    // Verify update
    let meta = storage.get_metadata(&key_id).await.unwrap().unwrap();
    assert_eq!(meta.usage_count, 42);
    assert!(meta.last_used.is_some());

    // Cleanup
    storage.delete(&key_id).await.unwrap();
}

#[tokio::test]
async fn test_api_key_list_keys() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_list_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let mut key_ids = Vec::new();

    // Store 3 keys
    for i in 0..3 {
        let key_id = format!("key_{}_{}", i, Uuid::new_v4());
        key_ids.push(key_id.clone());

        let metadata = ApiKeyMetadata {
            key_id: key_id.clone(),
            service: format!("service_{}", i),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            usage_count: 0,
        };

        storage.store(&key_id, "test_key", &metadata).await.unwrap();
    }

    // List keys
    let keys = storage.list_keys().await.unwrap();
    assert_eq!(keys.len(), 3);

    // Cleanup
    for key_id in key_ids {
        storage.delete(&key_id).await.unwrap();
    }
}

#[tokio::test]
async fn test_api_key_rotation() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_rotate_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let old_key_id = format!("old_key_{}", Uuid::new_v4());
    let metadata = ApiKeyMetadata {
        key_id: old_key_id.clone(),
        service: "google_search".to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: None,
        is_active: true,
        usage_count: 10,
    };

    storage
        .store(&old_key_id, "old_secret", &metadata)
        .await
        .unwrap();

    // Rotate key
    let new_key_id = storage.rotate_key(&old_key_id, "new_secret").await.unwrap();

    // Old key should be inactive
    let old_meta = storage.get_metadata(&old_key_id).await.unwrap().unwrap();
    assert!(!old_meta.is_active);

    // New key should be active
    let new_meta = storage.get_metadata(&new_key_id).await.unwrap().unwrap();
    assert!(new_meta.is_active);
    assert_eq!(new_meta.service, "google_search");
    assert_eq!(new_meta.usage_count, 0);

    // New key should decrypt correctly
    let new_key = storage.get(&new_key_id).await.unwrap().unwrap();
    assert_eq!(new_key, "new_secret");

    // Cleanup
    storage.delete(&old_key_id).await.unwrap();
    storage.delete(&new_key_id).await.unwrap();
}

#[tokio::test]
async fn test_api_key_cleanup_expired() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_cleanup_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let expired_key_id = format!("expired_{}", Uuid::new_v4());
    let expired_metadata = ApiKeyMetadata {
        key_id: expired_key_id.clone(),
        service: "test_service".to_string(),
        created_at: Utc::now(),
        last_used: None,
        expires_at: Some(Utc::now() - Duration::days(1)),
        is_active: true,
        usage_count: 0,
    };

    storage
        .store(&expired_key_id, "expired_key", &expired_metadata)
        .await
        .unwrap();

    // Cleanup expired keys
    let deleted_count = storage.cleanup_expired_keys().await.unwrap();
    assert_eq!(deleted_count, 1);

    // Key should be deleted
    let result = storage.get(&expired_key_id).await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_api_key_statistics() {
    ensure_migrations_run_once().await;

    let config = PostgresConfig::new(APP_CONNECTION_STRING);
    let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
    let storage = PostgresApiKeyStorage::new(backend.clone(), "test_passphrase".to_string());

    let tenant_id = format!("test_stats_{}", Uuid::new_v4());
    backend.set_tenant_context(&tenant_id).await.unwrap();

    let mut key_ids = Vec::new();

    // Store keys with different services
    for (i, service) in ["openai", "anthropic", "openai"].iter().enumerate() {
        let key_id = format!("key_stats_{}_{}", i, Uuid::new_v4());
        key_ids.push(key_id.clone());

        let metadata = ApiKeyMetadata {
            key_id: key_id.clone(),
            service: service.to_string(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: i < 2, // First 2 active, last one inactive
            usage_count: (i * 10) as u64,
        };

        storage.store(&key_id, "test_key", &metadata).await.unwrap();
    }

    // Get statistics
    let stats: ApiKeyStats = storage.get_statistics().await.unwrap();

    assert_eq!(stats.total_keys, 3);
    assert_eq!(stats.active_keys, 2);
    assert_eq!(stats.keys_by_service.get("openai"), Some(&2));
    assert_eq!(stats.keys_by_service.get("anthropic"), Some(&1));
    assert_eq!(stats.total_usage, 30); // 0 + 10 + 20
    assert_eq!(stats.avg_usage_per_key, 10.0);

    // Cleanup
    for key_id in key_ids {
        storage.delete(&key_id).await.unwrap();
    }
}
