//! Integration tests for context engineering pipeline
//!
//! Tests end-to-end flows: Query → Understanding → Strategy → Retrieval → Reranking → Assembly

use chrono::Utc;
use llmspell_context::assembly::ContextAssembler;
use llmspell_context::query::RegexQueryAnalyzer;
use llmspell_context::reranking::BM25Reranker;
use llmspell_context::retrieval::{BM25Retriever, StrategySelector};
use llmspell_context::traits::{QueryAnalyzer, Reranker};
use llmspell_context::types::{Chunk, QueryIntent, RetrievalStrategy};

/// Create test chunks for integration testing
fn create_test_corpus() -> Vec<Chunk> {
    let now = Utc::now();
    vec![
        Chunk {
            id: "1".to_string(),
            content: "Rust ownership system prevents data races through compile-time checks"
                .to_string(),
            source: "conversation".to_string(),
            timestamp: now - chrono::Duration::hours(1),
            metadata: None,
        },
        Chunk {
            id: "2".to_string(),
            content: "HashMap is a hash map implementation in Rust std collections".to_string(),
            source: "conversation".to_string(),
            timestamp: now - chrono::Duration::hours(2),
            metadata: None,
        },
        Chunk {
            id: "3".to_string(),
            content: "Python uses garbage collection for memory management".to_string(),
            source: "conversation".to_string(),
            timestamp: now - chrono::Duration::days(1),
            metadata: None,
        },
        Chunk {
            id: "4".to_string(),
            content: "Rust's borrow checker enforces memory safety without runtime overhead"
                .to_string(),
            source: "conversation".to_string(),
            timestamp: now - chrono::Duration::minutes(30),
            metadata: None,
        },
        Chunk {
            id: "5".to_string(),
            content: "VecDeque provides efficient double-ended queue operations".to_string(),
            source: "conversation".to_string(),
            timestamp: now - chrono::Duration::hours(3),
            metadata: None,
        },
    ]
}

#[tokio::test]
async fn test_end_to_end_pipeline_howto_query() {
    // Setup pipeline components
    let analyzer = RegexQueryAnalyzer::new();
    let selector = StrategySelector::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    let query = "How do I use HashMap in Rust?";

    // Step 1: Query understanding
    let understanding = analyzer.understand(query).await.unwrap();
    assert_eq!(understanding.intent, QueryIntent::HowTo);
    assert!(understanding.entities.contains(&"HashMap".to_string()));
    assert!(understanding.keywords.contains(&"rust".to_string()));

    // Step 2: Strategy selection
    let strategy = selector.select(&understanding);
    assert_eq!(strategy, RetrievalStrategy::Episodic); // HowTo → Episodic

    // Step 3: Retrieval
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    assert!(!retrieved.is_empty());

    // Step 4: Reranking
    let reranked = reranker.rerank(retrieved, query, 5).await.unwrap();
    assert!(!reranked.is_empty());
    assert!(reranked.len() <= 5);

    // Step 5: Assembly
    let context = assembler.assemble(reranked, &understanding);
    assert!(!context.chunks.is_empty());
    assert!(context.total_confidence > 0.0);
    assert!(context.token_count > 0);
    assert!(!context.formatted.is_empty());
}

#[tokio::test]
async fn test_end_to_end_pipeline_whatis_query() {
    let analyzer = RegexQueryAnalyzer::new();
    let selector = StrategySelector::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    let query = "What is the borrow checker in Rust?";

    // Query understanding
    let understanding = analyzer.understand(query).await.unwrap();
    assert_eq!(understanding.intent, QueryIntent::WhatIs);
    assert!(understanding.keywords.contains(&"borrow".to_string()));

    // Strategy selection - WhatIs with entities → Semantic (fallback to BM25 here)
    let strategy = selector.select(&understanding);
    assert!(strategy == RetrievalStrategy::Semantic || strategy == RetrievalStrategy::BM25);

    // Retrieval
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);

    // Reranking
    let reranked = reranker.rerank(retrieved, query, 3).await.unwrap();

    // Assembly
    let context = assembler.assemble(reranked, &understanding);
    assert!(!context.chunks.is_empty());

    // Verify most relevant chunk is about borrow checker (chunk 4)
    let top_chunk = &context.chunks[0];
    assert!(top_chunk.chunk.content.contains("borrow checker"));
}

#[tokio::test]
async fn test_end_to_end_pipeline_debug_query() {
    let analyzer = RegexQueryAnalyzer::new();
    let selector = StrategySelector::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    let query = "I have an error with memory management in Rust";

    // Query understanding
    let understanding = analyzer.understand(query).await.unwrap();
    assert_eq!(understanding.intent, QueryIntent::Debug);

    // Strategy selection - Debug → Hybrid
    let strategy = selector.select(&understanding);
    assert_eq!(strategy, RetrievalStrategy::Hybrid);

    // Full pipeline
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 5).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    assert!(!context.chunks.is_empty());
}

#[tokio::test]
async fn test_pipeline_with_confidence_filtering() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();

    // High confidence threshold
    let assembler = ContextAssembler::with_config(8000, 0.8);

    let corpus = create_test_corpus();
    let query = "Rust ownership";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 10).await.unwrap();
    let reranked_len = reranked.len();
    let context = assembler.assemble(reranked, &understanding);

    // High threshold should filter out low-confidence chunks
    assert!(context.chunks.is_empty() || context.chunks.len() < reranked_len);
}

#[tokio::test]
async fn test_pipeline_with_token_budget() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();

    // Very small token budget
    let assembler = ContextAssembler::with_config(100, 0.0);

    let corpus = create_test_corpus();
    let query = "Rust memory safety";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 10).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    // Token budget should be respected
    assert!(context.token_count <= 100);
}

#[tokio::test]
async fn test_pipeline_with_empty_corpus() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let query = "How do I use Rust?";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &[], 10);
    let reranked = reranker.rerank(retrieved, query, 10).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    // Empty corpus should produce empty context
    assert!(context.chunks.is_empty());
    assert_eq!(context.token_count, 0);
    assert!(context.total_confidence.abs() < f32::EPSILON);
}

#[tokio::test]
async fn test_pipeline_with_no_matching_chunks() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    // Query with completely unrelated terms
    let query = "quantum physics black holes";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 5).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    // Should return something (BM25 fallback) but with low confidence
    if !context.chunks.is_empty() {
        assert!(context.total_confidence < 0.5);
    }
}

#[tokio::test]
async fn test_temporal_ordering_in_pipeline() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    let query = "Rust memory";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 10).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    // Verify temporal ordering (recent first)
    if context.chunks.len() >= 2 {
        for i in 0..context.chunks.len() - 1 {
            assert!(context.chunks[i].chunk.timestamp >= context.chunks[i + 1].chunk.timestamp);
        }
    }
}

#[tokio::test]
async fn test_metadata_preservation() {
    let analyzer = RegexQueryAnalyzer::new();
    let retriever = BM25Retriever::new();
    let reranker = BM25Reranker::new();
    let assembler = ContextAssembler::new();

    let corpus = create_test_corpus();
    let query = "Rust ownership";

    let understanding = analyzer.understand(query).await.unwrap();
    let retrieved = retriever.retrieve_from_chunks(query, &corpus, 10);
    let reranked = reranker.rerank(retrieved, query, 5).await.unwrap();
    let context = assembler.assemble(reranked, &understanding);

    // Verify metadata preserved
    for ranked_chunk in &context.chunks {
        assert!(!ranked_chunk.chunk.id.is_empty());
        assert!(!ranked_chunk.chunk.source.is_empty());
        assert!(!ranked_chunk.ranker.is_empty());
        assert!(ranked_chunk.score >= 0.0 && ranked_chunk.score <= 1.0);
    }

    // Verify temporal span
    let (oldest, newest) = context.temporal_span;
    assert!(newest >= oldest);
}
