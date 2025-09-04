//! Unit tests for StateFactory
//! Verifies that StateFactory correctly creates StateManager instances from LLMSpellConfig

use llmspell_config::{GlobalRuntimeConfig, LLMSpellConfig, StatePersistenceConfig};
use llmspell_state_persistence::factory::StateFactory;
use std::sync::Arc;

/// Test that StateFactory creates correct backend from config - Memory backend
#[tokio::test]
async fn test_state_factory_creates_memory_backend() {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config).await.unwrap();
    assert!(
        state_manager.is_some(),
        "StateFactory should create StateManager when persistence is enabled"
    );

    // Verify we can write and read from the created StateManager
    let sm = state_manager.unwrap();
    let test_value = serde_json::json!({"test": "data"});
    sm.set(
        llmspell_state_persistence::StateScope::Global,
        "test_key",
        test_value.clone(),
    )
    .await
    .unwrap();

    let retrieved = sm
        .get(llmspell_state_persistence::StateScope::Global, "test_key")
        .await
        .unwrap();

    assert_eq!(
        retrieved,
        Some(test_value),
        "Memory backend should store and retrieve values correctly"
    );
}

/// Test that StateFactory creates correct backend from config - Sled backend
#[tokio::test]
async fn test_state_factory_creates_sled_backend() {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "sled".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config).await.unwrap();
    assert!(
        state_manager.is_some(),
        "StateFactory should create StateManager with sled backend"
    );

    // Verify the sled backend works
    let sm = state_manager.unwrap();
    let test_value = serde_json::json!({"sled": "test"});
    sm.set(
        llmspell_state_persistence::StateScope::Global,
        "sled_key",
        test_value.clone(),
    )
    .await
    .unwrap();

    let retrieved = sm
        .get(llmspell_state_persistence::StateScope::Global, "sled_key")
        .await
        .unwrap();

    assert_eq!(
        retrieved,
        Some(test_value),
        "Sled backend should persist data correctly"
    );
}

/// Test that StateFactory returns None when persistence is disabled
#[tokio::test]
async fn test_state_factory_returns_none_when_disabled() {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let state_manager = StateFactory::create_from_config(&config).await.unwrap();
    assert!(
        state_manager.is_none(),
        "StateFactory should return None when persistence is disabled"
    );
}

/// Test that StateFactory handles invalid backend types gracefully
#[tokio::test]
async fn test_state_factory_handles_invalid_backend() {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "invalid_backend".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    // Should default to memory backend for unknown types
    let state_manager = StateFactory::create_from_config(&config).await.unwrap();
    assert!(
        state_manager.is_some(),
        "StateFactory should fall back to memory backend for invalid types"
    );
}

/// Test that multiple StateFactory calls with same config create independent instances
#[tokio::test]
async fn test_state_factory_creates_independent_instances() {
    let config = Arc::new(
        LLMSpellConfig::builder()
            .runtime(
                GlobalRuntimeConfig::builder()
                    .state_persistence(StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let sm1 = StateFactory::create_from_config(&config)
        .await
        .unwrap()
        .unwrap();
    let sm2 = StateFactory::create_from_config(&config)
        .await
        .unwrap()
        .unwrap();

    // Verify they are different instances (different pointers)
    let ptr1 = Arc::as_ptr(&sm1);
    let ptr2 = Arc::as_ptr(&sm2);
    assert_ne!(
        ptr1, ptr2,
        "StateFactory should create new instances on each call"
    );

    // Verify they don't share state (memory backend creates independent stores)
    sm1.set(
        llmspell_state_persistence::StateScope::Global,
        "unique_key",
        serde_json::json!("value1"),
    )
    .await
    .unwrap();

    let sm2_value = sm2
        .get(llmspell_state_persistence::StateScope::Global, "unique_key")
        .await
        .unwrap();

    assert_eq!(
        sm2_value, None,
        "Independent StateManager instances should not share state"
    );
}
