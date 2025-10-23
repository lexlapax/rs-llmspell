//! Integration tests for memory consolidation
//!
//! Tests the end-to-end flow: Episodic → Consolidation → Semantic

use llmspell_graph::extraction::RegexExtractor;
use llmspell_graph::storage::surrealdb::SurrealDBBackend;
use llmspell_memory::consolidation::ManualConsolidationEngine;
use llmspell_memory::episodic::InMemoryEpisodicMemory;
use llmspell_memory::manager::DefaultMemoryManager;
use llmspell_memory::semantic::GraphSemanticMemory;
use llmspell_memory::traits::MemoryManager;
use llmspell_memory::types::{ConsolidationMode, EpisodicEntry};
use std::sync::Arc;
use tempfile::TempDir;

/// Create test manager with manual consolidation engine
async fn create_test_manager() -> (DefaultMemoryManager, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    // Create backends
    let episodic = Arc::new(InMemoryEpisodicMemory::new());
    let graph = Arc::new(
        SurrealDBBackend::new(temp_dir.path().to_path_buf())
            .await
            .unwrap(),
    );
    let semantic = Arc::new(GraphSemanticMemory::new(graph.clone()));
    let procedural = Arc::new(llmspell_memory::procedural::NoopProceduralMemory);

    // Create consolidation engine
    let extractor = Arc::new(RegexExtractor::new());
    let consolidation = Arc::new(ManualConsolidationEngine::new(extractor, graph));

    let manager =
        DefaultMemoryManager::with_consolidation(episodic, semantic, procedural, consolidation);

    (manager, temp_dir)
}

#[tokio::test]
async fn test_episodic_to_semantic_flow() {
    let (manager, _temp) = create_test_manager().await;

    // Add episodic entries about Rust
    let entry1 = EpisodicEntry::new(
        "session-rust".to_string(),
        "user".to_string(),
        "Tell me about Rust programming language.".to_string(),
    );

    let entry2 = EpisodicEntry::new(
        "session-rust".to_string(),
        "assistant".to_string(),
        "Rust is a systems programming language. Rust has memory safety.".to_string(),
    );

    manager.episodic().add(entry1).await.unwrap();
    manager.episodic().add(entry2).await.unwrap();

    // Trigger consolidation
    let result = manager
        .consolidate("session-rust", ConsolidationMode::Manual)
        .await
        .unwrap();

    // Verify consolidation result
    assert_eq!(result.entries_processed, 2, "Should process both entries");
    assert!(
        result.entities_added > 0,
        "Should extract at least one entity (Rust)"
    );
    assert!(result.duration_ms > 0, "Should track duration");

    // Verify entities were added to semantic memory
    // (We can't easily query by name without implementing a search method,
    // but we can verify that some entities exist by checking the result)
}

#[tokio::test]
async fn test_consolidation_marks_entries_processed() {
    let (manager, _temp) = create_test_manager().await;

    // Add episodic entry
    let entry = EpisodicEntry::new(
        "session-1".to_string(),
        "user".to_string(),
        "Python is a high-level programming language.".to_string(),
    );

    let entry_id = entry.id.clone();
    manager.episodic().add(entry).await.unwrap();

    // Consolidate
    let result = manager
        .consolidate("session-1", ConsolidationMode::Manual)
        .await
        .unwrap();

    assert_eq!(result.entries_processed, 1);

    // Verify entry is marked as processed
    let retrieved = manager.episodic().get(&entry_id).await.unwrap();
    assert!(
        retrieved.processed,
        "Entry should be marked as processed after consolidation"
    );
}

#[tokio::test]
async fn test_consolidation_skips_processed_entries() {
    let (manager, _temp) = create_test_manager().await;

    // Add and process an entry
    let entry = EpisodicEntry::new(
        "session-1".to_string(),
        "user".to_string(),
        "JavaScript is a scripting language.".to_string(),
    );

    manager.episodic().add(entry).await.unwrap();

    // First consolidation
    let result1 = manager
        .consolidate("session-1", ConsolidationMode::Manual)
        .await
        .unwrap();
    assert_eq!(result1.entries_processed, 1);

    // Second consolidation should skip already-processed entries
    let result2 = manager
        .consolidate("session-1", ConsolidationMode::Manual)
        .await
        .unwrap();
    assert_eq!(result2.entries_processed, 0, "Should not reprocess entries");
    assert_eq!(result2.entities_added, 0);
}

#[tokio::test]
async fn test_consolidation_session_isolation() {
    let (manager, _temp) = create_test_manager().await;

    // Add entries to different sessions
    let entry1 = EpisodicEntry::new(
        "session-A".to_string(),
        "user".to_string(),
        "Rust is fast.".to_string(),
    );

    let entry2 = EpisodicEntry::new(
        "session-B".to_string(),
        "user".to_string(),
        "Python is easy.".to_string(),
    );

    manager.episodic().add(entry1).await.unwrap();
    manager.episodic().add(entry2).await.unwrap();

    // Consolidate only session-A
    let result = manager
        .consolidate("session-A", ConsolidationMode::Manual)
        .await
        .unwrap();

    assert_eq!(
        result.entries_processed, 1,
        "Should only process session-A entries"
    );
}

#[tokio::test]
async fn test_empty_session_consolidation() {
    let (manager, _temp) = create_test_manager().await;

    // Consolidate non-existent session
    let result = manager
        .consolidate("nonexistent-session", ConsolidationMode::Manual)
        .await
        .unwrap();

    assert_eq!(result.entries_processed, 0);
    assert_eq!(result.entities_added, 0);
}

#[tokio::test]
async fn test_multiple_relationship_extraction() {
    let (manager, _temp) = create_test_manager().await;

    // Add entry with multiple relationships
    let entry = EpisodicEntry::new(
        "session-1".to_string(),
        "assistant".to_string(),
        "Rust is a systems programming language. \
         Rust has memory safety. \
         Rust has zero-cost abstractions. \
         Cargo is a tool for Rust."
            .to_string(),
    );

    manager.episodic().add(entry).await.unwrap();

    // Consolidate
    let result = manager
        .consolidate("session-1", ConsolidationMode::Manual)
        .await
        .unwrap();

    assert_eq!(result.entries_processed, 1);
    assert!(
        result.entities_added >= 2,
        "Should extract at least Rust and Cargo"
    );
}

#[tokio::test]
async fn test_consolidation_with_no_op_engine() {
    // Test that default manager (with no-op engine) returns empty result
    let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

    let entry = EpisodicEntry::new(
        "session-1".to_string(),
        "user".to_string(),
        "Test content.".to_string(),
    );

    manager.episodic().add(entry).await.unwrap();

    // Consolidation with no-op engine should return 0
    let result = manager
        .consolidate("session-1", ConsolidationMode::Manual)
        .await
        .unwrap();

    assert_eq!(
        result.entries_processed, 0,
        "No-op engine should not process entries"
    );
}

#[tokio::test]
async fn test_consolidation_immediate_mode() {
    let (manager, _temp) = create_test_manager().await;

    let entry = EpisodicEntry::new(
        "session-1".to_string(),
        "user".to_string(),
        "Go is a language designed at Google.".to_string(),
    );

    manager.episodic().add(entry).await.unwrap();

    // Test immediate mode (should behave same as manual for now)
    let result = manager
        .consolidate("session-1", ConsolidationMode::Immediate)
        .await
        .unwrap();

    assert_eq!(result.entries_processed, 1);
    assert!(result.entities_added > 0);
}
