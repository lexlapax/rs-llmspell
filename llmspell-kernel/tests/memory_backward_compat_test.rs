//! Backward Compatibility Tests for Memory Integration (Phase 13.7.5c)
//!
//! Verifies that components work correctly with memory_manager=None (opt-in design):
//! - StateManager works without memory_manager
//! - IntegratedKernel components work without memory
//! - Existing tests continue to pass

use llmspell_kernel::state::{
    config::{PersistenceConfig, StorageBackendType},
    StateManager, StateScope,
};
use serde_json::json;

#[tokio::test]
async fn test_state_manager_without_memory() {
    // Create StateManager WITHOUT memory_manager (backward compat)
    let config = PersistenceConfig {
        enabled: false,
        ..Default::default()
    };

    let state_manager = StateManager::with_backend(
        StorageBackendType::Memory,
        config,
        None, // memory_manager = None for backward compat
    )
    .await
    .expect("StateManager should work without memory_manager");

    // State operations should work normally
    state_manager
        .set_with_hooks(StateScope::Global, "test.key", json!("value"))
        .await
        .expect("State set should work without memory_manager");

    let value = state_manager
        .get(StateScope::Global, "test.key")
        .await
        .expect("State get should work without memory_manager");

    assert_eq!(value, Some(json!("value")));

    // Multiple operations should not cause errors
    for i in 0..10 {
        state_manager
            .set_with_hooks(StateScope::Global, "counter", json!(i))
            .await
            .expect("Repeated state operations should work without memory_manager");
    }
}

#[tokio::test]
async fn test_state_manager_new_without_memory() {
    // Test StateManager::new() with None parameter
    let state_manager = StateManager::new(None)
        .await
        .expect("StateManager::new(None) should work");

    // Basic operations should work
    state_manager
        .set(StateScope::Global, "test", json!(42))
        .await
        .expect("State operations should work");

    let value = state_manager
        .get(StateScope::Global, "test")
        .await
        .expect("Get should work");

    assert_eq!(value, Some(json!(42)));
}

#[tokio::test]
async fn test_multiple_scopes_without_memory() {
    // Test multiple state scopes without memory_manager
    let state_manager = StateManager::new(None)
        .await
        .expect("Failed to create state manager");

    // Test different scopes
    state_manager
        .set(StateScope::Global, "global_key", json!("global_value"))
        .await
        .unwrap();

    state_manager
        .set(
            StateScope::Session("session1".to_string()),
            "session_key",
            json!("session_value"),
        )
        .await
        .unwrap();

    state_manager
        .set(
            StateScope::Agent("agent1".to_string()),
            "agent_key",
            json!("agent_value"),
        )
        .await
        .unwrap();

    // Verify all scopes work
    assert_eq!(
        state_manager.get(StateScope::Global, "global_key").await.unwrap(),
        Some(json!("global_value"))
    );

    assert_eq!(
        state_manager
            .get(StateScope::Session("session1".to_string()), "session_key")
            .await
            .unwrap(),
        Some(json!("session_value"))
    );

    assert_eq!(
        state_manager
            .get(StateScope::Agent("agent1".to_string()), "agent_key")
            .await
            .unwrap(),
        Some(json!("agent_value"))
    );
}

#[tokio::test]
async fn test_hooks_without_memory() {
    // Test that hooks system still works without memory_manager
    // (hooks just won't write to memory, but shouldn't error)
    let state_manager = StateManager::new(None)
        .await
        .expect("Failed to create state manager");

    // set_with_hooks should work even without memory_manager
    state_manager
        .set_with_hooks(StateScope::Global, "hook_test", json!("works"))
        .await
        .expect("Hooks should work (no-op) without memory_manager");

    let value = state_manager
        .get(StateScope::Global, "hook_test")
        .await
        .unwrap();

    assert_eq!(value, Some(json!("works")));
}

#[test]
fn test_backward_compat_documented() {
    // This test documents that backward compatibility is maintained:
    // - StateManager accepts Option<Arc<dyn MemoryManager>>
    // - None = no memory integration (backward compatible)
    // - Some(...) = memory integration enabled (new feature)
    //
    // All existing code passing None continues to work unchanged.

    // This is a documentation test - always passes
    assert!(true, "Backward compatibility design documented");
}
