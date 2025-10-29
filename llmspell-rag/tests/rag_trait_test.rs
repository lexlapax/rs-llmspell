//! Integration tests for RAG Retriever trait
//!
//! Tests `RAGRetriever` trait behavior and implementation.

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use llmspell_core::state::StateScope;
use llmspell_kernel::sessions::SessionId;
use llmspell_rag::pipeline::{RAGResult, RAGRetriever};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Query tracking enum - distinguishes never-called, called-with-None, called-with-Some(scope)
#[derive(Debug, Clone)]
enum ScopeQuery {
    NeverCalled,
    CalledWithNone,
    CalledWithScope(StateScope),
}

/// Mock `RAGRetriever` implementation for testing
struct MockRAGRetriever {
    /// Track which scope was queried (None = never called, Some(None) = called with None, Some(Some(scope)) = called with scope)
    last_scope_queried: Arc<Mutex<ScopeQuery>>,
    /// Track query parameters
    last_query: Arc<Mutex<Option<(String, usize)>>>,
    /// Mock results to return
    mock_results: Vec<RAGResult>,
}

impl MockRAGRetriever {
    fn new(mock_results: Vec<RAGResult>) -> Self {
        Self {
            last_scope_queried: Arc::new(Mutex::new(ScopeQuery::NeverCalled)),
            last_query: Arc::new(Mutex::new(None)),
            mock_results,
        }
    }

    fn get_last_scope(&self) -> ScopeQuery {
        self.last_scope_queried.lock().unwrap().clone()
    }

    fn get_last_query(&self) -> Option<(String, usize)> {
        self.last_query.lock().unwrap().clone()
    }
}

#[async_trait]
impl RAGRetriever for MockRAGRetriever {
    async fn retrieve(
        &self,
        query: &str,
        k: usize,
        scope: Option<StateScope>,
    ) -> Result<Vec<RAGResult>> {
        // Record query parameters
        *self.last_scope_queried.lock().unwrap() =
            scope.map_or(ScopeQuery::CalledWithNone, ScopeQuery::CalledWithScope);
        *self.last_query.lock().unwrap() = Some((query.to_string(), k));

        // Return mock results
        Ok(self.mock_results.clone())
    }
}

#[tokio::test]
async fn test_rag_retriever_with_session_scope() {
    const EPSILON: f32 = 0.001;

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), serde_json::json!("test_doc.txt"));

    let mock_results = vec![RAGResult {
        id: "result-1".to_string(),
        content: "test content".to_string(),
        score: 0.85,
        metadata,
        timestamp: Utc::now(),
    }];

    let retriever = Arc::new(MockRAGRetriever::new(mock_results));
    let test_session = SessionId::new();

    // Create scope with session
    let scope_value = StateScope::Custom(format!("session:{test_session}"));
    let scope = Some(scope_value.clone());

    // Query retriever
    let results = retriever.retrieve("test query", 5, scope).await.unwrap();

    // Verify scope was passed correctly
    match retriever.get_last_scope() {
        ScopeQuery::CalledWithScope(s) => assert_eq!(s, scope_value),
        _ => panic!("Expected CalledWithScope"),
    }

    // Verify query params
    let (query, k) = retriever.get_last_query().unwrap();
    assert_eq!(query, "test query");
    assert_eq!(k, 5);

    // Verify results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "result-1");
    assert_eq!(results[0].content, "test content");
    assert!((results[0].score - 0.85).abs() < EPSILON);
}

#[tokio::test]
async fn test_rag_retriever_with_no_scope() {
    let mock_results = vec![RAGResult {
        id: "result-2".to_string(),
        content: "default content".to_string(),
        score: 0.75,
        metadata: HashMap::new(),
        timestamp: Utc::now(),
    }];

    let retriever = Arc::new(MockRAGRetriever::new(mock_results));

    // Query without scope
    let results = retriever.retrieve("test query", 10, None).await.unwrap();

    // Verify None scope was passed
    match retriever.get_last_scope() {
        ScopeQuery::CalledWithNone => (),
        _ => panic!("Expected CalledWithNone"),
    }

    // Verify results
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "result-2");
}

#[tokio::test]
async fn test_rag_retriever_with_global_scope() {
    let mock_results = vec![];
    let retriever = Arc::new(MockRAGRetriever::new(mock_results));

    // Query with Global scope
    let scope_value = StateScope::Global;
    let scope = Some(scope_value.clone());
    let _results = retriever.retrieve("test query", 5, scope).await.unwrap();

    // Verify Global scope was passed
    match retriever.get_last_scope() {
        ScopeQuery::CalledWithScope(s) => assert_eq!(s, scope_value),
        _ => panic!("Expected CalledWithScope"),
    }
}

#[tokio::test]
async fn test_rag_result_preserves_metadata() {
    const EPSILON: f32 = 0.001;

    let mut metadata = HashMap::new();
    metadata.insert("doc_type".to_string(), serde_json::json!("technical"));
    metadata.insert("author".to_string(), serde_json::json!("test_user"));
    metadata.insert("tags".to_string(), serde_json::json!(["rust", "testing"]));

    let timestamp = Utc::now();
    let mock_results = vec![RAGResult {
        id: "detailed-result".to_string(),
        content: "detailed content with metadata".to_string(),
        score: 0.92,
        metadata: metadata.clone(),
        timestamp,
    }];

    let retriever = Arc::new(MockRAGRetriever::new(mock_results));
    let results = retriever.retrieve("test query", 5, None).await.unwrap();

    // Verify all fields preserved
    assert_eq!(results.len(), 1);
    let result = &results[0];

    assert_eq!(result.id, "detailed-result");
    assert_eq!(result.content, "detailed content with metadata");
    assert!((result.score - 0.92).abs() < EPSILON);
    assert_eq!(result.timestamp, timestamp);

    // Verify metadata preserved
    assert_eq!(
        result.metadata.get("doc_type"),
        Some(&serde_json::json!("technical"))
    );
    assert_eq!(
        result.metadata.get("author"),
        Some(&serde_json::json!("test_user"))
    );
    assert_eq!(
        result.metadata.get("tags"),
        Some(&serde_json::json!(["rust", "testing"]))
    );
}

#[tokio::test]
async fn test_rag_retriever_handles_empty_results() {
    let retriever = Arc::new(MockRAGRetriever::new(vec![]));
    let results = retriever.retrieve("test query", 5, None).await.unwrap();

    assert_eq!(results.len(), 0);
}

#[tokio::test]
async fn test_rag_retriever_handles_multiple_results() {
    const EPSILON: f32 = 0.001;

    let mock_results = vec![
        RAGResult {
            id: "result-a".to_string(),
            content: "content a".to_string(),
            score: 0.9,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RAGResult {
            id: "result-b".to_string(),
            content: "content b".to_string(),
            score: 0.8,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
        RAGResult {
            id: "result-c".to_string(),
            content: "content c".to_string(),
            score: 0.7,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        },
    ];

    let retriever = Arc::new(MockRAGRetriever::new(mock_results));
    let results = retriever.retrieve("test query", 5, None).await.unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].id, "result-a");
    assert_eq!(results[1].id, "result-b");
    assert_eq!(results[2].id, "result-c");

    // Verify scores preserved
    assert!((results[0].score - 0.9).abs() < EPSILON);
    assert!((results[1].score - 0.8).abs() < EPSILON);
    assert!((results[2].score - 0.7).abs() < EPSILON);
}

#[tokio::test]
async fn test_rag_result_builder_pattern() {
    const EPSILON: f32 = 0.001;

    // Test RAGResult builder methods
    let result = RAGResult::new("test-id".to_string(), "test content".to_string(), 0.95)
        .with_metadata("key1".to_string(), serde_json::json!("value1"))
        .with_metadata("key2".to_string(), serde_json::json!(42))
        .with_timestamp(Utc::now());

    assert_eq!(result.id, "test-id");
    assert_eq!(result.content, "test content");
    assert!((result.score - 0.95).abs() < EPSILON);
    assert_eq!(
        result.metadata.get("key1"),
        Some(&serde_json::json!("value1"))
    );
    assert_eq!(result.metadata.get("key2"), Some(&serde_json::json!(42)));
}
