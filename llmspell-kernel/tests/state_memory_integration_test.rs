//! State-Memory Integration Tests (Phase 13.7.4d)
//!
//! Verifies StateMemoryHook integration with StateManager:
//! - State transitions create procedural memory patterns
//! - Pattern frequency threshold detection (≥3 occurrences)
//! - Opt-in design (StateManager works without memory_manager)

use llmspell_kernel::state::{
    config::{PersistenceConfig, StorageBackendType},
    StateManager, StateScope,
};
use llmspell_memory::{DefaultMemoryManager, MemoryManager};
use serde_json::json;
use std::sync::Arc;

#[tokio::test]
async fn test_state_transitions_create_patterns() {
    // Create memory manager and state manager with memory integration
    let memory_manager =
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
    let memory_arc = Arc::new(memory_manager);

    // Use memory backend with persistence disabled to force synchronous hook execution
    let config = PersistenceConfig {
        enabled: false,
        ..Default::default()
    };

    let state_manager =
        StateManager::with_backend(StorageBackendType::Memory, config, Some(memory_arc.clone()))
            .expect("Failed to create state manager");

    // Perform same state transition 3 times
    for _ in 0..3 {
        state_manager
            .set_with_hooks(StateScope::Global, "config.theme", json!("dark"))
            .await
            .expect("Failed to set state");
    }

    // Verify pattern was recorded in procedural memory
    let frequency = memory_arc
        .procedural()
        .get_pattern_frequency("global", "config.theme", "dark")
        .await
        .expect("Failed to get pattern frequency");

    assert_eq!(frequency, 3, "Expected 3 transitions to be recorded");
}

#[tokio::test]
async fn test_pattern_threshold_detection() {
    // Create memory manager and state manager
    let memory_manager =
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
    let memory_arc = Arc::new(memory_manager);

    let config = PersistenceConfig {
        enabled: false,
        ..Default::default()
    };

    let state_manager =
        StateManager::with_backend(StorageBackendType::Memory, config, Some(memory_arc.clone()))
            .await
            .expect("Failed to create state manager");

    // Transition 1: Should record but not yet a pattern
    state_manager
        .set_with_hooks(
            StateScope::Session("test-session".to_string()),
            "user.lang",
            json!("rust"),
        )
        .await
        .expect("Failed to set state");

    let freq1 = memory_arc
        .procedural()
        .get_pattern_frequency("session:test-session", "user.lang", "rust")
        .await
        .expect("Failed to get frequency");
    assert_eq!(freq1, 1);

    // Transition 2: Still not a pattern
    state_manager
        .set_with_hooks(
            StateScope::Session("test-session".to_string()),
            "user.lang",
            json!("rust"),
        )
        .await
        .expect("Failed to set state");

    let freq2 = memory_arc
        .procedural()
        .get_pattern_frequency("session:test-session", "user.lang", "rust")
        .await
        .expect("Failed to get frequency");
    assert_eq!(freq2, 2);

    // Transition 3: NOW it becomes a pattern (≥3 threshold)
    state_manager
        .set_with_hooks(
            StateScope::Session("test-session".to_string()),
            "user.lang",
            json!("rust"),
        )
        .await
        .expect("Failed to set state");

    let freq3 = memory_arc
        .procedural()
        .get_pattern_frequency("session:test-session", "user.lang", "rust")
        .await
        .expect("Failed to get frequency");
    assert_eq!(freq3, 3);

    // Verify it shows up in learned patterns
    let patterns = memory_arc
        .procedural()
        .get_learned_patterns(3)
        .await
        .expect("Failed to get learned patterns");

    assert_eq!(patterns.len(), 1, "Expected exactly 1 learned pattern");
    assert_eq!(patterns[0].scope, "session:test-session");
    assert_eq!(patterns[0].key, "user.lang");
    assert_eq!(patterns[0].value, "rust");
    assert_eq!(patterns[0].frequency, 3);
}

#[tokio::test]
async fn test_state_without_memory_manager() {
    // Create state manager WITHOUT memory manager (opt-in design)
    let state_manager = StateManager::new(None)
        .await
        .expect("Failed to create state manager without memory");

    // State operations should work normally
    state_manager
        .set_with_hooks(StateScope::Global, "config.debug", json!(true))
        .await
        .expect("State set should work without memory manager");

    let value = state_manager
        .get(StateScope::Global, "config.debug")
        .await
        .expect("State get should work without memory manager");

    assert_eq!(value, Some(json!(true)));

    // Multiple transitions should not cause errors
    for i in 0..5 {
        state_manager
            .set_with_hooks(StateScope::Global, "counter", json!(i))
            .await
            .expect("Repeated state sets should work without memory manager");
    }
}

#[tokio::test]
async fn test_multiple_scopes_and_keys() {
    // Test that different scopes and keys create separate patterns
    let memory_manager =
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
    let memory_arc = Arc::new(memory_manager);

    let config = PersistenceConfig {
        enabled: false,
        ..Default::default()
    };

    let state_manager =
        StateManager::with_backend(StorageBackendType::Memory, config, Some(memory_arc.clone()))
            .await
            .expect("Failed to create state manager");

    // Set different keys in different scopes
    for _ in 0..3 {
        state_manager
            .set_with_hooks(StateScope::Global, "theme", json!("dark"))
            .await
            .unwrap();
    }

    for _ in 0..4 {
        state_manager
            .set_with_hooks(
                StateScope::Agent("agent-1".to_string()),
                "mode",
                json!("active"),
            )
            .await
            .unwrap();
    }

    for _ in 0..2 {
        state_manager
            .set_with_hooks(StateScope::Global, "language", json!("en"))
            .await
            .unwrap();
    }

    // Verify each pattern is tracked separately
    let global_theme_freq = memory_arc
        .procedural()
        .get_pattern_frequency("global", "theme", "dark")
        .await
        .unwrap();
    assert_eq!(global_theme_freq, 3);

    let agent_mode_freq = memory_arc
        .procedural()
        .get_pattern_frequency("agent:agent-1", "mode", "active")
        .await
        .unwrap();
    assert_eq!(agent_mode_freq, 4);

    let global_lang_freq = memory_arc
        .procedural()
        .get_pattern_frequency("global", "language", "en")
        .await
        .unwrap();
    assert_eq!(global_lang_freq, 2);

    // Get learned patterns (≥3 threshold)
    let patterns = memory_arc
        .procedural()
        .get_learned_patterns(3)
        .await
        .unwrap();

    assert_eq!(patterns.len(), 2, "Expected 2 patterns above threshold");
    // Patterns should be sorted by frequency descending
    assert_eq!(patterns[0].frequency, 4); // agent mode
    assert_eq!(patterns[1].frequency, 3); // global theme
}

#[tokio::test]
async fn test_state_value_changes_tracked() {
    // Verify that changing from one value to another is tracked
    let memory_manager =
        DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
    let memory_arc = Arc::new(memory_manager);

    let config = PersistenceConfig {
        enabled: false,
        ..Default::default()
    };

    let state_manager =
        StateManager::with_backend(StorageBackendType::Memory, config, Some(memory_arc.clone()))
            .await
            .expect("Failed to create state manager");

    // Initial state: light mode
    state_manager
        .set_with_hooks(StateScope::Global, "theme", json!("light"))
        .await
        .unwrap();

    // Transition to dark mode 3 times
    for _ in 0..3 {
        state_manager
            .set_with_hooks(StateScope::Global, "theme", json!("dark"))
            .await
            .unwrap();
    }

    // Both values should be tracked
    let light_freq = memory_arc
        .procedural()
        .get_pattern_frequency("global", "theme", "light")
        .await
        .unwrap();
    assert_eq!(light_freq, 1);

    let dark_freq = memory_arc
        .procedural()
        .get_pattern_frequency("global", "theme", "dark")
        .await
        .unwrap();
    assert_eq!(dark_freq, 3);
}
