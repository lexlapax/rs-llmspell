//! RAG Retriever trait - session-agnostic retrieval interface
//!
//! Provides abstract interface for RAG retrieval without direct `SessionManager` dependency.
//! Session context is encoded in `StateScope` when needed.
//!
//! Note: Named `RAGRetriever` (not `RAGPipeline`) to avoid confusion with the concrete
//! `RAGPipeline` struct which is a full-featured pipeline with ingestion, storage, etc.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::state::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Result from RAG retrieval
///
/// Bridge format between RAG vector search and context assembly.
/// Simpler than `SessionVectorResult` - no session-specific fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGResult {
    /// Unique result ID
    pub id: String,
    /// Content text
    pub content: String,
    /// Similarity/relevance score (0.0-1.0)
    pub score: f32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp (when document was ingested or result created)
    pub timestamp: DateTime<Utc>,
}

impl RAGResult {
    /// Create a new RAG result
    #[must_use]
    pub fn new(id: String, content: String, score: f32) -> Self {
        Self {
            id,
            content,
            score,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        }
    }

    /// Add metadata field
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set timestamp
    #[must_use]
    pub const fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = timestamp;
        self
    }
}

/// Abstract RAG retriever interface
///
/// Session-agnostic retrieval interface. Session context encoded in `StateScope`
/// when needed (e.g., `StateScope::Custom("session:abc123")`).
///
/// This trait allows:
/// - Context crate to depend on RAG without `SessionManager`
/// - Testing with mock implementations
/// - Multiple RAG backend implementations
#[async_trait]
pub trait RAGRetriever: Send + Sync {
    /// Retrieve relevant documents
    ///
    /// # Arguments
    /// * `query` - Search query text
    /// * `k` - Number of results to return
    /// * `scope` - Optional scope for filtering (session, tenant, etc.)
    ///
    /// # Returns
    /// Vector of RAG results sorted by relevance score (highest first)
    ///
    /// # Errors
    /// Returns error if retrieval fails
    async fn retrieve(
        &self,
        query: &str,
        k: usize,
        scope: Option<StateScope>,
    ) -> Result<Vec<RAGResult>>;
}
