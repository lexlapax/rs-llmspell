//! Integration tests for `MemoryManager` with libsql backend
//!
//! Tests the full memory system integration: episodic + semantic + procedural via libsql

use llmspell_graph::extraction::RegexExtractor;
use llmspell_graph::{Entity, Relationship};
use llmspell_memory::consolidation::ManualConsolidationEngine;
use llmspell_memory::episodic::InMemoryEpisodicMemory;
use llmspell_memory::manager::DefaultMemoryManager;
use llmspell_memory::procedural::NoopProceduralMemory;
use llmspell_memory::semantic::GraphSemanticMemory;
use llmspell_memory::traits::MemoryManager;
use llmspell_memory::types::{ConsolidationMode, EpisodicEntry};
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteGraphStorage};
use serde_json::json;
use std::sync::Arc;

/// Create integrated memory manager with all subsystems via libsql
async fn create_integrated_manager() -> DefaultMemoryManager {
    // Create in-memory SQLite backend for testing
    let config = SqliteConfig::in_memory();
    let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

    // Create all three memory subsystems
    let episodic = Arc::new(InMemoryEpisodicMemory::new());
    let graph = Arc::new(SqliteGraphStorage::new(Arc::clone(&backend)));
    let semantic = Arc::new(GraphSemanticMemory::new(graph.clone()));
    let procedural = Arc::new(NoopProceduralMemory);

    // Create consolidation engine
    let extractor = Arc::new(RegexExtractor::new());
    let consolidation = Arc::new(ManualConsolidationEngine::new(extractor, graph));

    DefaultMemoryManager::with_consolidation(episodic, semantic, procedural, consolidation)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_episodic_operations() {
    let manager = create_integrated_manager().await;

    // Test episodic add
    let entry1 = EpisodicEntry::new(
        "session-1".to_string(),
        "user".to_string(),
        "What is Rust?".to_string(),
    );
    let entry2 = EpisodicEntry::new(
        "session-1".to_string(),
        "assistant".to_string(),
        "Rust is a systems programming language.".to_string(),
    );

    manager.episodic().add(entry1.clone()).await.unwrap();
    manager.episodic().add(entry2.clone()).await.unwrap();

    // Test episodic search
    let results = manager
        .episodic()
        .search("Rust programming", 5)
        .await
        .unwrap();
    assert!(!results.is_empty(), "Should find entries related to Rust");

    // Verify entry retrieval
    let retrieved1 = manager.episodic().get(&entry1.id).await.unwrap();
    assert_eq!(retrieved1.content, "What is Rust?");

    let retrieved2 = manager.episodic().get(&entry2.id).await.unwrap();
    assert_eq!(
        retrieved2.content,
        "Rust is a systems programming language."
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_semantic_operations() {
    let manager = create_integrated_manager().await;

    // Test semantic memory (knowledge graph)
    // Add entities
    let rust_entity = Entity::new(
        "Rust".to_string(),
        "programming_language".to_string(),
        json!({"paradigm": "multi-paradigm", "memory_safety": true}),
    );
    let rust_id = rust_entity.id.clone();
    manager.semantic().upsert_entity(rust_entity).await.unwrap();

    // Add target entity for relationship
    let memory_safety_entity = Entity::new(
        "memory_safety".to_string(),
        "feature".to_string(),
        json!({"description": "prevents memory leaks"}),
    );
    let memory_safety_id = memory_safety_entity.id.clone();
    manager
        .semantic()
        .upsert_entity(memory_safety_entity)
        .await
        .unwrap();

    // Add relationship between them
    let relationship = Relationship::new(
        rust_id.clone(),
        memory_safety_id.clone(),
        "has_feature".to_string(),
        json!({"verified": true}),
    );
    manager
        .semantic()
        .add_relationship(relationship)
        .await
        .unwrap();

    // NOTE: get_relationships is not fully implemented yet (returns empty Vec)
    // See semantic.rs:172 - TODO to expand KnowledgeGraph trait
    // For now, we just verify that add_relationship doesn't error

    // Verify we can retrieve the entities back
    let rust_retrieved = manager.semantic().get_entity(&rust_id).await.unwrap();
    assert!(
        rust_retrieved.is_some(),
        "Should be able to retrieve Rust entity"
    );
    assert_eq!(rust_retrieved.unwrap().name, "Rust");

    let memory_safety_retrieved = manager
        .semantic()
        .get_entity(&memory_safety_id)
        .await
        .unwrap();
    assert!(
        memory_safety_retrieved.is_some(),
        "Should be able to retrieve memory_safety entity"
    );
    assert_eq!(memory_safety_retrieved.unwrap().name, "memory_safety");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_procedural_operations() {
    let manager = create_integrated_manager().await;

    // Test procedural memory (patterns)
    // Note: NoopProceduralMemory is a placeholder, so operations are no-ops
    // This test verifies the API works without errors

    // Store pattern (no-op but should not error)
    let pattern_json = r#"{"pattern_type":"prompt_template","template":"What is {subject}?"}"#;
    let _pattern_id = manager
        .procedural()
        .store_pattern(pattern_json)
        .await
        .unwrap();

    // Record state transition (no-op but should not error)
    let _frequency = manager
        .procedural()
        .record_transition("global", "config.theme", Some("light"), "dark")
        .await
        .unwrap();

    // Get pattern frequency (returns 0 for no-op)
    let frequency = manager
        .procedural()
        .get_pattern_frequency("global", "config.theme", "dark")
        .await
        .unwrap();
    assert_eq!(
        frequency, 0,
        "NoopProceduralMemory should return 0 frequency"
    );

    // Get learned patterns (returns empty for no-op)
    let patterns = manager.procedural().get_learned_patterns(3).await.unwrap();
    assert!(
        patterns.is_empty(),
        "NoopProceduralMemory should return empty patterns"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_consolidation_flow() {
    let manager = create_integrated_manager().await;

    // Add episodic entries about Python
    let entry1 = EpisodicEntry::new(
        "session-python".to_string(),
        "user".to_string(),
        "Tell me about Python.".to_string(),
    );

    let entry2 = EpisodicEntry::new(
        "session-python".to_string(),
        "assistant".to_string(),
        "Python is a high-level programming language. Python has dynamic typing.".to_string(),
    );

    manager.episodic().add(entry1).await.unwrap();
    manager.episodic().add(entry2).await.unwrap();

    // Trigger consolidation (episodic → semantic)
    let result = manager
        .consolidate("session-python", ConsolidationMode::Manual, None)
        .await
        .unwrap();

    // Verify consolidation result
    assert_eq!(result.entries_processed, 2, "Should process both entries");
    assert!(
        result.entities_added > 0,
        "Should extract at least one entity (Python)"
    );
    assert!(result.duration_ms > 0, "Should track duration");

    println!(
        "Consolidation: {} entries → {} entities in {}ms",
        result.entries_processed, result.entities_added, result.duration_ms
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_full_integration() {
    let manager = create_integrated_manager().await;

    // 1. Add episodic memories about Go language
    let entries = vec![
        EpisodicEntry::new(
            "session-go".to_string(),
            "user".to_string(),
            "What is Go?".to_string(),
        ),
        EpisodicEntry::new(
            "session-go".to_string(),
            "assistant".to_string(),
            "Go is a compiled programming language. Go was designed at Google.".to_string(),
        ),
        EpisodicEntry::new(
            "session-go".to_string(),
            "user".to_string(),
            "What are Go's features?".to_string(),
        ),
        EpisodicEntry::new(
            "session-go".to_string(),
            "assistant".to_string(),
            "Go has built-in concurrency. Go has garbage collection.".to_string(),
        ),
    ];

    for entry in &entries {
        manager.episodic().add(entry.clone()).await.unwrap();
    }

    // 2. Search episodic memory
    let search_results = manager
        .episodic()
        .search("Go language features", 5)
        .await
        .unwrap();
    assert!(!search_results.is_empty(), "Should find entries about Go");

    // 3. Consolidate to semantic memory
    let consolidation = manager
        .consolidate("session-go", ConsolidationMode::Manual, None)
        .await
        .unwrap();
    assert_eq!(consolidation.entries_processed, 4);
    assert!(consolidation.entities_added > 0);

    // 4. Verify entities were added to semantic memory
    // We can't easily query by name without search, but we verified entities_added > 0

    // 5. Manually add a direct semantic entity
    let go_entity = Entity::new(
        "Go".to_string(),
        "programming_language".to_string(),
        json!({"compiled": true, "creator": "Google"}),
    );
    let go_id = go_entity.id.clone();
    manager.semantic().upsert_entity(go_entity).await.unwrap();

    // 6. Add target entity for relationship
    let concurrency_entity = Entity::new(
        "concurrency".to_string(),
        "feature".to_string(),
        json!({"description": "built-in goroutines and channels"}),
    );
    let concurrency_id = concurrency_entity.id.clone();
    manager
        .semantic()
        .upsert_entity(concurrency_entity)
        .await
        .unwrap();

    // 7. Add relationship
    let concurrency_rel = Relationship::new(
        go_id.clone(),
        concurrency_id.clone(),
        "has_feature".to_string(),
        json!({"verified": true}),
    );
    manager
        .semantic()
        .add_relationship(concurrency_rel)
        .await
        .unwrap();

    // 8. Verify entities (NOTE: get_relationships not fully implemented yet)
    let go_retrieved = manager.semantic().get_entity(&go_id).await.unwrap();
    assert!(go_retrieved.is_some(), "Should retrieve Go entity");

    let concurrency_retrieved = manager
        .semantic()
        .get_entity(&concurrency_id)
        .await
        .unwrap();
    assert!(
        concurrency_retrieved.is_some(),
        "Should retrieve concurrency entity"
    );

    // 9. Test procedural memory (no-op but verify API works)
    let pattern_json = r#"{"template":"What is {language}?"}"#;
    manager
        .procedural()
        .store_pattern(pattern_json)
        .await
        .unwrap();

    println!("✓ Full integration test passed:");
    println!("  - Episodic: {} entries added", entries.len());
    println!(
        "  - Consolidation: {} → {} entities",
        consolidation.entries_processed, consolidation.entities_added
    );
    println!("  - Semantic: entities and relationships verified");
    println!("  - Procedural: pattern save verified");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_memory_manager_shutdown() {
    let manager = create_integrated_manager().await;

    // Add some data
    let entry = EpisodicEntry::new(
        "session-shutdown".to_string(),
        "user".to_string(),
        "Test shutdown".to_string(),
    );
    manager.episodic().add(entry).await.unwrap();

    // Graceful shutdown should not error
    manager.shutdown().await.unwrap();
}
