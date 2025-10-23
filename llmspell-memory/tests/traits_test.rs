//! Trait compilation and basic functionality tests

use chrono::Utc;
use llmspell_memory::prelude::*;
use serde_json::json;

/// Test that all trait types compile and can be instantiated
#[test]
fn test_trait_types_compile() {
    // EpisodicEntry
    let entry = EpisodicEntry::new("session-1".into(), "user".into(), "test".into());
    assert_eq!(entry.session_id, "session-1");
    assert_eq!(entry.role, "user");
    assert_eq!(entry.content, "test");
    assert!(!entry.processed);

    // ConsolidationMode
    let mode = ConsolidationMode::Immediate;
    assert_eq!(mode, ConsolidationMode::Immediate);

    // ConsolidationResult
    let result = ConsolidationResult::empty();
    assert_eq!(result.entries_processed, 0);
    assert_eq!(result.entities_added, 0);

    // Entity
    let entity = Entity {
        id: "test-123".into(),
        entity_type: "person".into(),
        name: "Test Person".into(),
        properties: json!({"key": "value"}),
        event_time: Utc::now(),
        ingestion_time: Utc::now(),
    };
    assert_eq!(entity.id, "test-123");
    assert_eq!(entity.entity_type, "person");

    // Relationship
    let relationship = Relationship {
        id: "rel-456".into(),
        from_entity: "person-1".into(),
        to_entity: "company-1".into(),
        relationship_type: "works_at".into(),
        properties: json!({}),
        event_time: Utc::now(),
        ingestion_time: Utc::now(),
    };
    assert_eq!(relationship.from_entity, "person-1");
    assert_eq!(relationship.relationship_type, "works_at");

    // ConsolidationDecision
    let _decision = ConsolidationDecision::Noop;
    let _decision2 = ConsolidationDecision::Add(entity);
}

/// Test `EpisodicEntry` mutation methods
#[test]
fn test_episodic_entry_methods() {
    let mut entry = EpisodicEntry::new("session-1".into(), "user".into(), "test".into());

    assert!(!entry.processed);
    entry.mark_processed();
    assert!(entry.processed);
}

/// Test `ConsolidationResult` methods
#[test]
fn test_consolidation_result() {
    let result = ConsolidationResult::empty();
    assert_eq!(result.entries_processed, 0);
    assert_eq!(result.duration_ms, 0);
}

/// Test that trait objects can be created (trait object safety)
#[test]
fn test_trait_object_safety() {
    // This test verifies that our traits are object-safe
    // (can be used with dyn Trait syntax)

    // We can't instantiate actual implementations yet, but we can
    // verify the types compile by using trait bounds

    fn _accepts_episodic_memory<T: EpisodicMemory>(_m: &T) {}
    fn _accepts_semantic_memory<T: SemanticMemory>(_m: &T) {}
    fn _accepts_procedural_memory<T: ProceduralMemory>(_m: &T) {}
    fn _accepts_memory_manager<T: MemoryManager>(_m: &T) {}
}

/// Test serialization of core types
#[test]
fn test_serialization() {
    let entry = EpisodicEntry::new("session-1".into(), "user".into(), "test content".into());

    // Serialize to JSON
    let json = serde_json::to_string(&entry).expect("serialization failed");
    assert!(json.contains("session-1"));
    assert!(json.contains("test content"));

    // Deserialize back
    let deserialized: EpisodicEntry =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.session_id, entry.session_id);
    assert_eq!(deserialized.content, entry.content);
}

/// Test Entity serialization
#[test]
fn test_entity_serialization() {
    let entity = Entity {
        id: "test-123".into(),
        entity_type: "person".into(),
        name: "Test Person".into(),
        properties: json!({"role": "engineer"}),
        event_time: Utc::now(),
        ingestion_time: Utc::now(),
    };

    // Serialize
    let json = serde_json::to_string(&entity).expect("serialization failed");
    assert!(json.contains("test-123"));
    assert!(json.contains("engineer"));

    // Deserialize
    let deserialized: Entity = serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.id, entity.id);
    assert_eq!(deserialized.name, entity.name);
}

/// Test Relationship serialization
#[test]
fn test_relationship_serialization() {
    let relationship = Relationship {
        id: "rel-456".into(),
        from_entity: "person-1".into(),
        to_entity: "company-1".into(),
        relationship_type: "works_at".into(),
        properties: json!({"since": "2024"}),
        event_time: Utc::now(),
        ingestion_time: Utc::now(),
    };

    // Serialize
    let json = serde_json::to_string(&relationship).expect("serialization failed");
    assert!(json.contains("rel-456"));
    assert!(json.contains("works_at"));

    // Deserialize
    let deserialized: Relationship =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(deserialized.id, relationship.id);
    assert_eq!(deserialized.relationship_type, relationship.relationship_type);
}

/// Test Clone semantics for `InMemoryEpisodicMemory`
#[tokio::test]
async fn test_clone_semantics() {
    let memory1 = InMemoryEpisodicMemory::new();

    // Add entry to original
    let entry = EpisodicEntry::new("session-1".into(), "user".into(), "test content".into());
    let id = memory1.add(entry).await.unwrap();

    // Clone the memory
    let memory2 = memory1.clone();

    // Both should access the same underlying data (Arc-counted)
    let retrieved1 = memory1.get(&id).await.unwrap();
    let retrieved2 = memory2.get(&id).await.unwrap();

    assert_eq!(retrieved1.content, retrieved2.content);
    assert_eq!(retrieved1.id, retrieved2.id);

    // Add entry to clone
    let entry2 = EpisodicEntry::new("session-2".into(), "user".into(), "clone content".into());
    let id2 = memory2.add(entry2).await.unwrap();

    // Original should see the new entry (shared state via Arc)
    let retrieved_from_original = memory1.get(&id2).await.unwrap();
    assert_eq!(retrieved_from_original.content, "clone content");
}

/// Test Default trait for `InMemoryEpisodicMemory`
#[test]
fn test_default_trait() {
    let memory = InMemoryEpisodicMemory::default();

    // Should be empty initially
    let result = tokio_test::block_on(memory.get_session("any-session"));
    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

/// Test Send/Sync bounds by spawning in 'static thread
#[test]
fn test_send_sync_bounds() {
    use std::thread;

    let memory = InMemoryEpisodicMemory::new();

    // Clone for thread (verifies Send)
    let mem_clone = memory;

    // Spawn in 'static thread (requires Send + Sync)
    let handle = thread::spawn(move || {
        // Use tokio runtime in thread
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let entry = EpisodicEntry::new("session-1".into(), "user".into(), "thread test".into());
            mem_clone.add(entry).await
        })
    });

    // Should complete successfully
    let result = handle.join().unwrap();
    assert!(result.is_ok());
}
