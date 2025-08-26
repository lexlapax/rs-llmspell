//! Core vector storage trait and types

use anyhow::Result;
use async_trait::async_trait;
use llmspell_state_traits::StateScope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::SystemTime;

/// Core vector storage trait with multi-tenant support
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Insert vectors with metadata and scope
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;

    /// Search for similar vectors
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;

    /// Scoped search with tenant isolation
    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>>;

    /// Update vector metadata
    async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()>;

    /// Delete vectors by ID
    async fn delete(&self, ids: &[String]) -> Result<()>;

    /// Delete all vectors for a scope (tenant cleanup)
    async fn delete_scope(&self, scope: &StateScope) -> Result<usize>;

    /// Get storage statistics
    async fn stats(&self) -> Result<StorageStats>;

    /// Get tenant-specific statistics
    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>;
}

/// Multi-tenant aware vector entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,

    /// Embedding vector
    pub embedding: Vec<f32>,

    /// Metadata for filtering and context
    pub metadata: HashMap<String, Value>,

    /// Tenant/user/session binding
    pub scope: StateScope,

    /// Creation timestamp
    pub created_at: SystemTime,

    /// Optional expiration time for session vectors
    pub expires_at: Option<SystemTime>,

    /// Explicit tenant ID for billing
    pub tenant_id: Option<String>,
}

impl VectorEntry {
    /// Create a new vector entry with default values
    #[must_use]
    pub fn new(id: String, embedding: Vec<f32>) -> Self {
        Self {
            id,
            embedding,
            metadata: HashMap::new(),
            scope: StateScope::Global,
            created_at: SystemTime::now(),
            expires_at: None,
            tenant_id: None,
        }
    }

    /// Set the scope for this vector
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.scope = scope;
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }

    /// Set expiration time
    #[must_use]
    pub const fn with_expiration(mut self, expires_at: SystemTime) -> Self {
        self.expires_at = Some(expires_at);
        self
    }
}

/// Query parameters for vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorQuery {
    /// Query vector
    pub vector: Vec<f32>,

    /// Number of results to return
    pub k: usize,

    /// Optional metadata filters
    pub filter: Option<HashMap<String, Value>>,

    /// Optional scope restriction
    pub scope: Option<StateScope>,

    /// Similarity threshold (0.0 to 1.0)
    pub threshold: Option<f32>,

    /// Include metadata in results
    pub include_metadata: bool,
}

impl VectorQuery {
    /// Create a new vector query
    #[must_use]
    pub const fn new(vector: Vec<f32>, k: usize) -> Self {
        Self {
            vector,
            k,
            filter: None,
            scope: None,
            threshold: None,
            include_metadata: true,
        }
    }

    /// Set scope filter
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.scope = Some(scope);
        self
    }

    /// Add metadata filters
    #[must_use]
    pub fn with_filter(mut self, filter: HashMap<String, Value>) -> Self {
        self.filter = Some(filter);
        self
    }

    /// Set similarity threshold
    #[must_use]
    pub const fn with_threshold(mut self, threshold: f32) -> Self {
        self.threshold = Some(threshold);
        self
    }
}

/// Result from vector search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorResult {
    /// Vector ID
    pub id: String,

    /// Similarity score (higher is better)
    pub score: f32,

    /// Optional vector data
    pub vector: Option<Vec<f32>>,

    /// Metadata if requested
    pub metadata: Option<HashMap<String, Value>>,

    /// Distance metric value
    pub distance: f32,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageStats {
    /// Total number of vectors
    pub total_vectors: usize,

    /// Total storage size in bytes
    pub storage_bytes: usize,

    /// Number of namespaces/tenants
    pub namespace_count: usize,

    /// Index build time in milliseconds
    pub index_build_time_ms: Option<u64>,

    /// Average query time in milliseconds
    pub avg_query_time_ms: Option<f32>,

    /// Number of dimensions
    pub dimensions: Option<usize>,
}

/// Scoped statistics for a specific tenant/scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScopedStats {
    /// Scope identifier
    pub scope: StateScope,

    /// Number of vectors in this scope
    pub vector_count: usize,

    /// Storage used in bytes
    pub storage_bytes: usize,

    /// Number of queries executed
    pub query_count: usize,

    /// Total tokens/embeddings processed
    pub tokens_processed: usize,

    /// Estimated cost in USD
    pub estimated_cost: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_entry_builder() {
        let entry = VectorEntry::new("test".to_string(), vec![1.0, 2.0, 3.0])
            .with_scope(StateScope::Custom("tenant:tenant-123".to_string()))
            .with_metadata(HashMap::from([(
                "source".to_string(),
                Value::String("test.txt".to_string()),
            )]));

        assert_eq!(entry.id, "test");
        assert_eq!(entry.embedding.len(), 3);
        assert!(entry.metadata.contains_key("source"));
    }

    #[test]
    fn test_vector_query_builder() {
        let query = VectorQuery::new(vec![1.0, 2.0, 3.0], 10)
            .with_threshold(0.8)
            .with_scope(StateScope::User("user-456".to_string()));

        assert_eq!(query.k, 10);
        assert_eq!(query.threshold, Some(0.8));
        assert!(query.scope.is_some());
    }
}
