// ABOUTME: Simple migration API integration test for bridge functionality
// ABOUTME: Tests that migration methods are available when migration support is enabled

use llmspell_bridge::globals::{state_global::StateGlobal, GlobalObject};
use llmspell_state_persistence::StateManager;
use std::sync::Arc;

#[tokio::test]
async fn test_state_global_creation() {
    // Test StateGlobal without migration support
    let state_global = StateGlobal::new();

    // Verify components are absent by default
    assert!(state_global.migration_engine.is_none());
    assert!(state_global.schema_registry.is_none());
    assert!(state_global.state_manager.is_none());

    // Test StateGlobal with StateManager
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let state_global_with_sm = StateGlobal::with_state_manager(state_manager);

    assert!(state_global_with_sm.state_manager.is_some());
    assert!(state_global_with_sm.migration_engine.is_none());
    assert!(state_global_with_sm.schema_registry.is_none());
}

#[test]
fn test_state_global_api_structure() {
    // Test that the API methods exist and can be called
    let state_global = StateGlobal::new();

    // Test metadata
    let metadata = state_global.metadata();
    assert_eq!(metadata.name, "State");
    assert!(metadata.description.contains("State management"));

    // Test scope parsing utility
    let global_scope = StateGlobal::parse_scope("global");
    let agent_scope = StateGlobal::parse_scope("agent:test");
    let workflow_scope = StateGlobal::parse_scope("workflow:test");
    let session_scope = StateGlobal::parse_scope("session:test");
    let custom_scope = StateGlobal::parse_scope("custom:test");

    // Verify scope parsing works
    assert!(matches!(
        global_scope,
        llmspell_state_persistence::StateScope::Global
    ));
    assert!(matches!(
        agent_scope,
        llmspell_state_persistence::StateScope::Agent(_)
    ));
    assert!(matches!(
        workflow_scope,
        llmspell_state_persistence::StateScope::Workflow(_)
    ));
    assert!(matches!(
        session_scope,
        llmspell_state_persistence::StateScope::Session(_)
    ));
    assert!(matches!(
        custom_scope,
        llmspell_state_persistence::StateScope::Custom(_)
    ));
}
