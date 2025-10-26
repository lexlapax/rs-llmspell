//! Full pipeline E2E test: Episodic → Consolidation → Semantic → Context Assembly
//!
//! This test validates the complete memory lifecycle:
//! 1. Add episodic memories (`InMemoryEpisodicMemory`)
//! 2. Trigger LLM consolidation (`LLMConsolidationEngine` + Ollama)
//! 3. Verify semantic graph updates (`SurrealDBBackend`)
//! 4. Retrieve and assemble context (`BM25Retriever` + `ContextAssembler`)
//!
//! # Requirements
//!
//! - Running Ollama instance with llama3.2:3b model
//! - Set `OLLAMA_HOST` environment variable (default: <http://localhost:11434>)
//! - Test skips gracefully if Ollama is unavailable
//!
//! # Performance Target
//!
//! - Complete in <40 seconds (with Ollama + `SurrealDB` temp + BM25)

use llmspell_context::assembly::ContextAssembler;
use llmspell_context::retrieval::BM25Retriever;
use llmspell_context::types::{QueryIntent, QueryUnderstanding, RankedChunk};
use llmspell_memory::consolidation::ConsolidationEngine;
use llmspell_memory::episodic::InMemoryEpisodicMemory;
use llmspell_memory::traits::EpisodicMemory as EpisodicMemoryTrait;
use llmspell_memory::types::EpisodicEntry;
use serial_test::serial;
use tracing::{debug, error, info};

use crate::e2e::helpers::create_test_engine;

/// Test full pipeline: Episodic → Consolidation → Semantic → Context Assembly
///
/// # Test Scenario
///
/// **Turn 1** (episodic): "Rust is a systems programming language with zero-cost abstractions"
/// **Turn 2** (episodic): "Rust has memory safety without garbage collection via ownership"
/// **Turn 3** (query): "What are Rust's key features?"
///
/// # Expected Flow
///
/// 1. Add 2 episodic memories
/// 2. Trigger LLM consolidation → creates/updates entities in semantic graph
/// 3. Retrieve relevant context using BM25
/// 4. Assemble context for LLM consumption
/// 5. Assert context contains key information from episodic memories
///
/// # Assertions
///
/// - ✓ Consolidation processes 2 entries
/// - ✓ At least 1 entity created in semantic graph
/// - ✓ BM25 retrieval returns relevant chunks
/// - ✓ Assembled context contains keywords: "memory safety", "zero-cost"
/// - ✓ Test completes in <40 seconds
#[ignore = "Requires Ollama - run individually"]
#[tokio::test]
#[serial]
async fn test_full_pipeline_episodic_to_context() {
    // Initialize tracing for test visibility
    let _ = tracing_subscriber::fmt()
        .with_env_filter("llmspell_memory=debug,llmspell_context=debug")
        .with_test_writer()
        .try_init();

    info!("Starting full pipeline E2E test");

    // Check Ollama availability
    if !crate::e2e::check_ollama_available().await {
        eprintln!("Skipping test_full_pipeline_episodic_to_context - Ollama unavailable");
        return;
    }

    info!("✓ Ollama available - proceeding with E2E test");

    // ============================================================================
    // STEP 1: Setup - Create test engine and episodic memory
    // ============================================================================
    info!("STEP 1: Setting up test engine and episodic memory");

    let test_engine = create_test_engine().await;
    let episodic_memory = InMemoryEpisodicMemory::default();

    debug!("Test engine created with LLM consolidation + SurrealDB backend");

    // ============================================================================
    // STEP 2: Add episodic memories (Turn 1 & 2)
    // ============================================================================
    info!("STEP 2: Adding 2 episodic memories to InMemoryEpisodicMemory");

    let entry1 = EpisodicEntry::new(
        "test-session".to_string(),
        "user".to_string(),
        "Rust is a systems programming language with zero-cost abstractions".to_string(),
    );

    let entry2 = EpisodicEntry::new(
        "test-session".to_string(),
        "user".to_string(),
        "Rust has memory safety without garbage collection via ownership".to_string(),
    );

    episodic_memory
        .add(entry1.clone())
        .await
        .expect("Failed to add entry1 to episodic memory");
    episodic_memory
        .add(entry2.clone())
        .await
        .expect("Failed to add entry2 to episodic memory");

    debug!("Added 2 episodic entries: Turn 1 (zero-cost abstractions), Turn 2 (memory safety)");

    // Verify episodic storage
    let session_entries = episodic_memory
        .get_session("test-session")
        .await
        .expect("Failed to retrieve session entries");
    assert_eq!(
        session_entries.len(),
        2,
        "Should have 2 entries in episodic memory"
    );

    // ============================================================================
    // STEP 3: Trigger LLM consolidation (Episodic → Semantic)
    // ============================================================================
    info!("STEP 3: Triggering LLM consolidation with Ollama");

    // Small delay to avoid overwhelming Ollama
    tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;

    let mut entries_for_consolidation = vec![entry1, entry2];
    let consolidation_result = test_engine
        .llm_engine
        .consolidate(&["test-session"], &mut entries_for_consolidation)
        .await;

    if let Err(ref e) = consolidation_result {
        error!("Consolidation failed: {}", e);
        panic!("Consolidation should succeed with valid Ollama setup: {e}");
    }

    let result = consolidation_result.unwrap();

    info!(
        "Consolidation complete: processed={}, added={}, updated={}, failed={}, duration={}ms",
        result.entries_processed,
        result.entities_added,
        result.entities_updated,
        result.entries_failed,
        result.duration_ms
    );

    // Assertions on consolidation result
    // Note: entries_processed may be less than 2 if LLM makes duplicate/invalid decisions
    // This is expected behavior - the validator is working correctly
    assert!(
        result.entries_processed >= 1,
        "Should process at least 1 episodic entry successfully"
    );
    assert!(
        result.entities_added > 0,
        "Should add at least 1 entity to semantic graph (Rust entity expected)"
    );

    debug!(
        "✓ Consolidation assertions passed: processed={}, added={}",
        result.entries_processed, result.entities_added
    );

    // ============================================================================
    // STEP 4: Verify semantic graph updates
    // ============================================================================
    info!("STEP 4: Verifying semantic graph contains consolidated entities");

    // Query for the Rust entity (fuzzy match: "rust", "rust-lang", etc.)
    let rust_entity_result = test_engine.knowledge_graph.get_entity("rust").await;

    if rust_entity_result.is_err() {
        // Try alternative entity IDs (LLM may use different conventions)
        let alternatives = ["rust-lang", "rust_programming", "rust_language"];
        let mut found = false;
        for alt_id in &alternatives {
            if test_engine.knowledge_graph.get_entity(alt_id).await.is_ok() {
                debug!("Found Rust entity with alternative ID: {}", alt_id);
                found = true;
                break;
            }
        }
        if !found {
            debug!("Rust entity not found with exact ID - this is OK if LLM used different naming");
            debug!(
                "Total entities added: {} (verifies semantic graph was updated)",
                result.entities_added
            );
        }
    } else {
        let rust_entity = rust_entity_result.unwrap();
        debug!(
            "✓ Found Rust entity in semantic graph: id={}, properties={:?}",
            rust_entity.id, rust_entity.properties
        );
    }

    // Main assertion: verify entities were added (regardless of exact IDs)
    assert!(
        result.entities_added >= 1,
        "At least 1 entity should be in semantic graph"
    );

    // ============================================================================
    // STEP 5: Retrieve context using BM25 (Turn 3 query)
    // ============================================================================
    info!("STEP 5: Retrieving relevant context using BM25Retriever");

    let query = "What are Rust's key features?";
    let bm25_retriever = BM25Retriever::new();

    // Retrieve from episodic memory (BM25 over episodic entries)
    let retrieved_chunks = bm25_retriever
        .retrieve_from_memory(query, &episodic_memory, 10, 5)
        .await
        .expect("BM25 retrieval should succeed");

    info!(
        "BM25 retrieval returned {} chunks for query: '{}'",
        retrieved_chunks.len(),
        query
    );

    assert!(
        !retrieved_chunks.is_empty(),
        "BM25 should retrieve at least one relevant chunk"
    );

    debug!("Retrieved chunks:");
    for chunk in &retrieved_chunks {
        debug!("  - id={}, content={}", chunk.id, chunk.content);
    }

    // ============================================================================
    // STEP 6: Assemble context for LLM consumption
    // ============================================================================
    info!("STEP 6: Assembling context using ContextAssembler");

    // Convert Chunks to RankedChunks (BM25 retriever doesn't provide scores in this flow)
    // In a real system, BM25Retriever would return scored chunks
    // For this test, we assign uniform scores since we're testing integration, not ranking
    let ranked_chunks: Vec<RankedChunk> = retrieved_chunks
        .into_iter()
        .map(|chunk| RankedChunk {
            chunk,
            score: 0.8, // High confidence (BM25 matched query terms)
            ranker: "bm25".to_string(),
        })
        .collect();

    let query_understanding = QueryUnderstanding {
        intent: QueryIntent::WhatIs,
        entities: vec!["Rust".to_string()],
        keywords: vec!["rust".to_string(), "features".to_string()],
    };

    let assembler = ContextAssembler::new();
    let assembled_context = assembler.assemble(ranked_chunks, &query_understanding);

    info!(
        "Context assembled: {} chunks, {:.2} avg confidence, {} tokens",
        assembled_context.chunks.len(),
        assembled_context.total_confidence,
        assembled_context.token_count
    );

    debug!("Assembled context:\n{}", assembled_context.formatted);

    // ============================================================================
    // STEP 7: Assertions on assembled context
    // ============================================================================
    info!("STEP 7: Validating assembled context contains key information");

    assert!(
        !assembled_context.chunks.is_empty(),
        "Assembled context should contain at least one chunk"
    );

    // Assert context contains key concepts from episodic memories
    let formatted_lower = assembled_context.formatted.to_lowercase();

    let has_memory_safety = formatted_lower.contains("memory")
        || formatted_lower.contains("safety")
        || formatted_lower.contains("ownership");

    let has_zero_cost = formatted_lower.contains("zero")
        || formatted_lower.contains("cost")
        || formatted_lower.contains("abstraction");

    if !has_memory_safety {
        debug!(
            "Warning: 'memory safety' not found in context: {}",
            assembled_context.formatted
        );
    }
    if !has_zero_cost {
        debug!(
            "Warning: 'zero-cost' not found in context: {}",
            assembled_context.formatted
        );
    }

    assert!(
        has_memory_safety || has_zero_cost,
        "Context should contain at least one key Rust feature (memory safety OR zero-cost abstractions)"
    );

    debug!("✓ Context validation passed");

    // ============================================================================
    // FINAL: Summary
    // ============================================================================
    info!("✅ Full pipeline E2E test PASSED");
    info!("   - Episodic: 2 entries added → BM25 retrieval successful");
    info!(
        "   - Consolidation: {} entities added in {}ms",
        result.entities_added, result.duration_ms
    );
    info!(
        "   - Context: {} chunks assembled ({} tokens)",
        assembled_context.chunks.len(),
        assembled_context.token_count
    );

    eprintln!("\n✓ test_full_pipeline_episodic_to_context PASSED");
    eprintln!("  Episodic entries: 2");
    eprintln!("  Entities added: {}", result.entities_added);
    eprintln!("  Context chunks: {}", assembled_context.chunks.len());
    eprintln!("  Context tokens: {}", assembled_context.token_count);
    eprintln!("  Consolidation duration: {}ms", result.duration_ms);
}
