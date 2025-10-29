//! Integration tests for hybrid RAG + Memory retrieval
//!
//! Tests RetrievalWeights validation, RAG adapter conversion, and budget allocation.

use chrono::Utc;
use llmspell_context::retrieval::{rag_result_to_ranked_chunk, rag_results_to_ranked_chunks, RetrievalWeights};
use llmspell_rag::pipeline::RAGResult;
use std::collections::HashMap;

#[test]
fn test_retrieval_weights_validation_valid() {
    // Valid weights that sum to 1.0
    assert!(RetrievalWeights::new(0.4, 0.6).is_ok());
    assert!(RetrievalWeights::new(0.5, 0.5).is_ok());
    assert!(RetrievalWeights::new(0.3, 0.7).is_ok());
    assert!(RetrievalWeights::new(0.0, 1.0).is_ok());
    assert!(RetrievalWeights::new(1.0, 0.0).is_ok());

    // Within tolerance (±0.01)
    assert!(RetrievalWeights::new(0.401, 0.599).is_ok());
}

#[test]
fn test_retrieval_weights_validation_invalid() {
    // Invalid weights that don't sum to 1.0
    assert!(RetrievalWeights::new(0.3, 0.5).is_err()); // Sum = 0.8
    assert!(RetrievalWeights::new(0.6, 0.6).is_err()); // Sum = 1.2
    assert!(RetrievalWeights::new(0.0, 0.0).is_err()); // Sum = 0.0
    assert!(RetrievalWeights::new(0.5, 0.4).is_err()); // Sum = 0.9

    // Outside tolerance
    assert!(RetrievalWeights::new(0.45, 0.53).is_err()); // Sum = 0.98 (just outside)
}

#[test]
fn test_retrieval_weights_presets() {
    let balanced = RetrievalWeights::balanced();
    assert_eq!(balanced.rag_weight, 0.5);
    assert_eq!(balanced.memory_weight, 0.5);

    let rag_focused = RetrievalWeights::rag_focused();
    assert_eq!(rag_focused.rag_weight, 0.7);
    assert_eq!(rag_focused.memory_weight, 0.3);

    let memory_focused = RetrievalWeights::memory_focused();
    assert_eq!(memory_focused.rag_weight, 0.4);
    assert_eq!(memory_focused.memory_weight, 0.6);

    // Verify default matches memory_focused
    let default = RetrievalWeights::default();
    assert_eq!(default.rag_weight, memory_focused.rag_weight);
    assert_eq!(default.memory_weight, memory_focused.memory_weight);
}

#[test]
fn test_token_budget_allocation() {
    // Test budget allocation based on weights

    // 40/60 split (memory-focused)
    let total_budget = 2000_usize;
    let weights = RetrievalWeights::new(0.4, 0.6).unwrap();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    let rag_budget = (total_budget as f32 * weights.rag_weight) as usize;
    let memory_budget = total_budget - rag_budget;

    assert_eq!(rag_budget, 800);
    assert_eq!(memory_budget, 1200);

    // 70/30 split (rag-focused)
    let weights_rag = RetrievalWeights::new(0.7, 0.3).unwrap();

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
    let rag_budget_2 = (total_budget as f32 * weights_rag.rag_weight) as usize;
    let memory_budget_2 = total_budget - rag_budget_2;

    assert_eq!(rag_budget_2, 1400);
    assert_eq!(memory_budget_2, 600);
}

#[test]
fn test_weighted_merge_calculation() {
    // Test weighted score calculation

    // RAG result: score 0.8, weight 0.4 → weighted score ≈0.32
    let rag_score = 0.8_f32;
    let rag_weight = 0.4_f32;
    let weighted_rag = rag_score * rag_weight;
    assert!((weighted_rag - 0.32).abs() < 0.001);

    // Memory result: score 0.6, weight 0.6 → weighted score ≈0.36
    let memory_score = 0.6_f32;
    let memory_weight = 0.6_f32;
    let weighted_memory = memory_score * memory_weight;
    assert!((weighted_memory - 0.36).abs() < 0.001);

    // Memory should rank higher after weighting
    assert!(weighted_memory > weighted_rag);
}

#[test]
fn test_rag_adapter_format_conversion_preserves_scores() {
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("test.txt"));

    let rag_result = RAGResult {
        id: "test-1".to_string(),
        content: "test content".to_string(),
        score: 0.92,
        metadata,
        timestamp: Utc::now(),
    };

    let ranked_chunk = rag_result_to_ranked_chunk(rag_result.clone());

    // Verify score preserved
    assert_eq!(ranked_chunk.score, 0.92);

    // Verify ID and content
    assert_eq!(ranked_chunk.chunk.id, "test-1");
    assert_eq!(ranked_chunk.chunk.content, "test content");

    // Verify ranker marked as RAG
    assert_eq!(ranked_chunk.ranker, "rag_vector_search");

    // Verify metadata
    assert!(ranked_chunk.chunk.metadata.is_some());
}

#[test]
fn test_rag_adapter_batch_conversion() {
    let results = vec![
        RAGResult {
            id: "r1".to_string(),
            content: "content 1".to_string(),
            score: 0.9,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RAGResult {
            id: "r2".to_string(),
            content: "content 2".to_string(),
            score: 0.8,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RAGResult {
            id: "r3".to_string(),
            content: "content 3".to_string(),
            score: 0.7,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
    ];

    let chunks = rag_results_to_ranked_chunks(results);

    assert_eq!(chunks.len(), 3);

    // Verify scores preserved in order
    assert_eq!(chunks[0].score, 0.9);
    assert_eq!(chunks[1].score, 0.8);
    assert_eq!(chunks[2].score, 0.7);

    // Verify IDs
    assert_eq!(chunks[0].chunk.id, "r1");
    assert_eq!(chunks[1].chunk.id, "r2");
    assert_eq!(chunks[2].chunk.id, "r3");
}

#[test]
fn test_rag_adapter_handles_empty_metadata() {
    let rag_result = RAGResult {
        id: "test-empty".to_string(),
        content: "content without metadata".to_string(),
        score: 0.5,
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    let ranked_chunk = rag_result_to_ranked_chunk(rag_result);

    // Empty metadata should result in None
    assert!(ranked_chunk.chunk.metadata.is_none());
}

#[test]
fn test_rag_adapter_source_extraction() {
    // Test with custom source in metadata
    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("custom_doc.pdf"));

    let rag_result = RAGResult {
        id: "test-source".to_string(),
        content: "content".to_string(),
        score: 0.7,
        metadata,
        timestamp: Utc::now(),
    };

    let ranked_chunk = rag_result_to_ranked_chunk(rag_result);

    // Source should be extracted from metadata
    assert_eq!(ranked_chunk.chunk.source, "custom_doc.pdf");

    // Test without source metadata (uses default)
    let rag_result_no_source = RAGResult {
        id: "test-default".to_string(),
        content: "content".to_string(),
        score: 0.7,
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    };

    let ranked_chunk_default = rag_result_to_ranked_chunk(rag_result_no_source);

    // Should use default source
    assert_eq!(ranked_chunk_default.chunk.source, "rag");
}

#[test]
fn test_retrieval_weights_error_messages() {
    // Test that error messages are informative
    let result = RetrievalWeights::new(0.3, 0.5);
    assert!(result.is_err());

    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("sum to 1.0"));
    assert!(err_msg.contains("0.8")); // The actual sum
}
