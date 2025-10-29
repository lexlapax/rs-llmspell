//! Integration tests for `QueryPatternTracker` + `MemoryManager` consolidation priority
//!
//! Tests the full flow:
//! 1. `HybridRetriever` retrieves episodic entries
//! 2. `QueryPatternTracker` records retrieval frequency
//! 3. `get_consolidation_candidates()` returns high-frequency entries
//! 4. `MemoryManager.consolidate()` prioritizes those entries

use llmspell_context::retrieval::{HybridRetriever, QueryPatternTracker, RetrievalWeights};
use llmspell_memory::{
    types::{ConsolidationMode, EpisodicEntry},
    DefaultMemoryManager, MemoryManager,
};
use std::sync::Arc;

/// Create in-memory memory manager with test data
async fn setup_memory_with_entries() -> Arc<DefaultMemoryManager> {
    let memory = Arc::new(
        DefaultMemoryManager::new_in_memory()
            .await
            .expect("Failed to create memory manager"),
    );

    // Add 5 episodic entries for testing
    let entries = vec![
        EpisodicEntry::new(
            "test-session".to_string(),
            "user".to_string(),
            "Entry 1: Rust programming".to_string(),
        ),
        EpisodicEntry::new(
            "test-session".to_string(),
            "user".to_string(),
            "Entry 2: Python data science".to_string(),
        ),
        EpisodicEntry::new(
            "test-session".to_string(),
            "user".to_string(),
            "Entry 3: JavaScript frameworks".to_string(),
        ),
        EpisodicEntry::new(
            "test-session".to_string(),
            "user".to_string(),
            "Entry 4: Go concurrency".to_string(),
        ),
        EpisodicEntry::new(
            "test-session".to_string(),
            "user".to_string(),
            "Entry 5: TypeScript types".to_string(),
        ),
    ];

    for entry in entries {
        memory
            .episodic()
            .add(entry)
            .await
            .expect("Failed to add entry");
    }

    memory
}

#[tokio::test]
async fn test_query_pattern_tracker_records_retrievals() {
    let memory = setup_memory_with_entries().await;
    let tracker = Arc::new(QueryPatternTracker::new());

    // Create HybridRetriever with tracker (no RAG, memory-only)
    let retriever = HybridRetriever::new(None, memory.clone(), RetrievalWeights::default())
        .with_query_tracker(tracker.clone());

    // Perform multiple retrievals - some entries retrieved more frequently
    for _ in 0..5 {
        let _ = retriever
            .retrieve_hybrid("Rust programming", "test-session", 1000)
            .await;
    }

    for _ in 0..3 {
        let _ = retriever
            .retrieve_hybrid("Python data", "test-session", 1000)
            .await;
    }

    for _ in 0..1 {
        let _ = retriever
            .retrieve_hybrid("JavaScript", "test-session", 1000)
            .await;
    }

    // Verify tracker recorded retrievals
    assert!(
        tracker.tracked_count() > 0,
        "Should have tracked some entries"
    );

    // Get candidates with min 3 retrievals
    let candidates = tracker.get_consolidation_candidates(3);
    assert!(
        !candidates.is_empty(),
        "Should have at least one high-frequency entry"
    );

    // Candidates should be entries that were retrieved 3+ times
    // (Entry 1: Rust - 5 times, Entry 2: Python - 3 times qualify)
    assert!(
        candidates.len() >= 2,
        "Should have at least 2 candidates (Rust, Python)"
    );
}

#[tokio::test]
async fn test_consolidation_priority_integration() {
    let memory = setup_memory_with_entries().await;
    let tracker = Arc::new(QueryPatternTracker::new());

    // Create HybridRetriever with tracker
    let retriever = HybridRetriever::new(None, memory.clone(), RetrievalWeights::default())
        .with_query_tracker(tracker.clone());

    // Simulate frequent retrievals of specific entries
    for _ in 0..7 {
        let _ = retriever
            .retrieve_hybrid("Rust programming", "test-session", 1000)
            .await;
    }

    for _ in 0..5 {
        let _ = retriever
            .retrieve_hybrid("Python data", "test-session", 1000)
            .await;
    }

    for _ in 0..2 {
        let _ = retriever
            .retrieve_hybrid("JavaScript", "test-session", 1000)
            .await;
    }

    // Get high-frequency candidates (min 5 retrievals)
    let priority_candidates = tracker.get_consolidation_candidates(5);
    assert!(
        !priority_candidates.is_empty(),
        "Should have priority candidates"
    );
    assert!(
        priority_candidates.len() >= 2,
        "Should have Rust (7x) and Python (5x) as priorities"
    );

    // Consolidate with priority hints
    let result = memory
        .consolidate(
            "test-session",
            ConsolidationMode::Manual,
            Some(&priority_candidates),
        )
        .await
        .expect("Consolidation should succeed");

    // Note: With NoopConsolidationEngine (default for in-memory), entries_processed = 0
    // The important part is that consolidation succeeds and accepts priority hints.
    // In real usage with ManualConsolidationEngine or LLMConsolidationEngine,
    // priority entries would be processed first and entities extracted.
    assert_eq!(
        result.entries_processed, 0,
        "NoopConsolidationEngine returns 0 (no actual processing)"
    );
    assert_eq!(result.entities_added, 0, "No entities with noop engine");
}

#[tokio::test]
async fn test_consolidation_without_priority() {
    let memory = setup_memory_with_entries().await;

    // Consolidate without priority hints (baseline)
    let result = memory
        .consolidate("test-session", ConsolidationMode::Manual, None)
        .await
        .expect("Consolidation should succeed");

    // With NoopConsolidationEngine, no entries are actually processed
    // But consolidation should succeed without priority hints
    assert_eq!(
        result.entries_processed, 0,
        "NoopConsolidationEngine returns 0 (baseline without priorities)"
    );
}

#[tokio::test]
async fn test_consolidation_with_nonexistent_priority() {
    let memory = setup_memory_with_entries().await;

    // Try to prioritize entries that don't exist
    let fake_priorities = vec!["nonexistent-1".to_string(), "nonexistent-2".to_string()];

    let result = memory
        .consolidate(
            "test-session",
            ConsolidationMode::Manual,
            Some(&fake_priorities),
        )
        .await
        .expect("Consolidation should succeed");

    // Consolidation should succeed even with non-matching priorities
    // With NoopConsolidationEngine, still returns 0
    assert_eq!(
        result.entries_processed, 0,
        "NoopConsolidationEngine returns 0 (even with non-matching priorities)"
    );
}

#[tokio::test]
async fn test_tracker_clear() {
    let tracker = QueryPatternTracker::new();

    // Record some retrievals
    tracker.record_retrieval(&["entry-1".to_string(), "entry-2".to_string()]);
    assert_eq!(tracker.tracked_count(), 2);

    // Clear tracker
    tracker.clear();
    assert_eq!(tracker.tracked_count(), 0, "Should clear all tracking data");

    // Verify candidates are empty after clear
    let candidates = tracker.get_consolidation_candidates(1);
    assert!(candidates.is_empty(), "Should have no candidates after clear");
}

#[tokio::test]
async fn test_tracker_get_count() {
    let tracker = QueryPatternTracker::new();

    // Record retrievals with different frequencies
    tracker.record_retrieval(&["entry-1".to_string()]);
    tracker.record_retrieval(&["entry-1".to_string()]);
    tracker.record_retrieval(&["entry-2".to_string()]);

    assert_eq!(tracker.get_count("entry-1"), 2);
    assert_eq!(tracker.get_count("entry-2"), 1);
    assert_eq!(tracker.get_count("nonexistent"), 0);
}

#[tokio::test]
async fn test_hybrid_retriever_without_tracker() {
    let memory = setup_memory_with_entries().await;

    // Create HybridRetriever without tracker (should work fine)
    let retriever = HybridRetriever::new(None, memory.clone(), RetrievalWeights::default());

    // Should still retrieve normally
    let results = retriever
        .retrieve_hybrid("Rust", "test-session", 1000)
        .await
        .expect("Retrieval should succeed");

    assert!(
        !results.is_empty(),
        "Should retrieve results without tracker"
    );
}

#[tokio::test]
async fn test_consolidation_candidates_sorting() {
    let tracker = QueryPatternTracker::new();

    // Record entries with different frequencies
    for _ in 0..10 {
        tracker.record_retrieval(&["entry-high".to_string()]);
    }
    for _ in 0..5 {
        tracker.record_retrieval(&["entry-medium".to_string()]);
    }
    for _ in 0..2 {
        tracker.record_retrieval(&["entry-low".to_string()]);
    }

    // Get all candidates (min 1 retrieval)
    let candidates = tracker.get_consolidation_candidates(1);

    // Should be sorted by frequency descending
    assert_eq!(candidates.len(), 3);
    assert_eq!(candidates[0], "entry-high", "Highest frequency first");
    assert_eq!(candidates[1], "entry-medium", "Medium frequency second");
    assert_eq!(candidates[2], "entry-low", "Lowest frequency last");
}
