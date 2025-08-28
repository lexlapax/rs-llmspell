//! Vector storage traits and types
//!
//! This module defines the core abstractions for vector storage operations,
//! including storage backends, query interfaces, and configuration types.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_state_traits::StateScope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::SystemTime;

// ============================================================================
// Core Storage Trait and Types
// ============================================================================

/// Core vector storage trait with multi-tenant support
#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Insert vectors with metadata and scope into storage.
    ///
    /// Stores a batch of vectors with their embeddings, metadata, and tenant scope
    /// information. Each vector is assigned a unique ID and can be retrieved later
    /// for similarity search.
    ///
    /// # Arguments
    ///
    /// * `vectors` - A vector of `VectorEntry` structs containing embeddings,
    ///   metadata, and scope information
    ///
    /// # Returns
    ///
    /// A vector of unique IDs assigned to the inserted vectors, in the same order
    /// as the input vectors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::{VectorEntry, VectorStorage};
    /// use llmspell_state_traits::StateScope;
    /// use std::collections::HashMap;
    /// use serde_json::Value;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let vectors = vec![
    ///     VectorEntry::new("doc1".to_string(), vec![0.1, 0.2, 0.3])
    ///         .with_scope(StateScope::User("user-123".to_string()))
    ///         .with_metadata(HashMap::from([
    ///             ("source".to_string(), Value::String("document.txt".to_string())),
    ///             ("timestamp".to_string(), Value::Number(1234567890.into())),
    ///         ])),
    /// ];
    ///
    /// let ids = storage.insert(vectors).await?;
    /// println!("Inserted vectors with IDs: {:?}", ids);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The storage backend is unavailable
    /// - Vector dimensions don't match the storage configuration
    /// - Duplicate IDs are provided
    /// - Storage capacity is exceeded
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;

    /// Search for vectors similar to the query vector.
    ///
    /// Performs approximate nearest neighbor search to find the most similar vectors
    /// to the provided query vector. Results are ordered by similarity score
    /// (descending - higher scores indicate greater similarity).
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters including the search vector, number of results,
    ///   and optional filters
    ///
    /// # Returns
    ///
    /// A vector of `VectorResult` structs containing matching vectors with similarity
    /// scores, ordered by relevance (highest similarity first).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::{VectorQuery, VectorStorage};
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 5)
    ///     .with_threshold(0.7);  // Only return results with >70% similarity
    ///
    /// let results = storage.search(&query).await?;
    /// for result in results {
    ///     println!("Found vector {} with similarity {:.3}", result.id, result.score);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;

    /// Search for vectors with tenant isolation and scope filtering.
    ///
    /// Like `search()`, but restricts results to vectors within the specified scope.
    /// This ensures tenant isolation in multi-tenant systems by preventing users
    /// from accessing vectors outside their authorized scope.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters (scope in query is ignored in favor of explicit scope)
    /// * `scope` - The scope to restrict search results to
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::{VectorQuery, VectorStorage};
    /// use llmspell_state_traits::StateScope;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10);
    /// let user_scope = StateScope::User("user-123".to_string());
    ///
    /// // Only returns vectors belonging to this user
    /// let results = storage.search_scoped(&query, &user_scope).await?;
    /// assert!(results.iter().all(|r| r.metadata.is_some()));
    /// # Ok(())
    /// # }
    /// ```
    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>>;

    /// Update metadata for an existing vector without changing its embedding.
    ///
    /// Allows modification of vector metadata while preserving the embedding and
    /// its position in the search index. Useful for updating document properties,
    /// tags, or other contextual information.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier of the vector to update
    /// * `metadata` - New metadata to replace the existing metadata
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::VectorStorage;
    /// use std::collections::HashMap;
    /// use serde_json::Value;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let new_metadata = HashMap::from([
    ///     ("status".to_string(), Value::String("processed".to_string())),
    ///     ("updated_at".to_string(), Value::Number(1234567890.into())),
    /// ]);
    ///
    /// storage.update_metadata("vector-123", new_metadata).await?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the vector ID is not found or the storage is read-only.
    async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()>;

    /// Delete vectors by their unique identifiers.
    ///
    /// Removes vectors from both the search index and persistent storage.
    /// This operation cannot be undone, so use with caution.
    ///
    /// # Arguments
    ///
    /// * `ids` - Slice of vector IDs to delete
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::VectorStorage;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let ids_to_delete = vec!["vector-1".to_string(), "vector-2".to_string()];
    /// storage.delete(&ids_to_delete).await?;
    /// println!("Deleted {} vectors", ids_to_delete.len());
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if some IDs are not found or storage is read-only.
    /// The operation may partially succeed, deleting some but not all vectors.
    async fn delete(&self, ids: &[String]) -> Result<()>;

    /// Delete all vectors belonging to a specific scope.
    ///
    /// Performs bulk deletion of all vectors within a tenant or user scope.
    /// This is typically used for tenant cleanup, account deletion, or
    /// session expiration scenarios.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope for which all vectors should be deleted
    ///
    /// # Returns
    ///
    /// The number of vectors that were deleted.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::VectorStorage;
    /// use llmspell_state_traits::StateScope;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let user_scope = StateScope::User("user-to-delete".to_string());
    /// let deleted_count = storage.delete_scope(&user_scope).await?;
    /// println!("Deleted {} vectors for user", deleted_count);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Performance
    ///
    /// This operation may be expensive for large scopes. Consider implementing
    /// batch deletion strategies for better performance.
    async fn delete_scope(&self, scope: &StateScope) -> Result<usize>;

    /// Get overall storage statistics and performance metrics.
    ///
    /// Provides insights into storage usage, performance characteristics,
    /// and system health. Useful for monitoring, capacity planning, and
    /// optimization decisions.
    ///
    /// # Returns
    ///
    /// A `StorageStats` struct containing various metrics including vector counts,
    /// storage size, query performance, and index statistics.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::VectorStorage;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let stats = storage.stats().await?;
    /// println!("Storage contains {} vectors using {} bytes",
    ///          stats.total_vectors, stats.storage_bytes);
    ///
    /// if let Some(query_time) = stats.avg_query_time_ms {
    ///     println!("Average query time: {:.2}ms", query_time);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn stats(&self) -> Result<StorageStats>;

    /// Get statistics for a specific scope (tenant, user, or session).
    ///
    /// Provides detailed metrics for vectors within a particular scope,
    /// enabling per-tenant billing, usage tracking, and performance analysis.
    ///
    /// # Arguments
    ///
    /// * `scope` - The scope to get statistics for
    ///
    /// # Returns
    ///
    /// A `ScopedStats` struct containing scope-specific metrics including
    /// vector counts, storage usage, query patterns, and cost estimates.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_storage::VectorStorage;
    /// use llmspell_state_traits::StateScope;
    ///
    /// # async fn example(storage: &dyn VectorStorage) -> anyhow::Result<()> {
    /// let tenant_scope = StateScope::Custom("tenant:acme-corp".to_string());
    /// let stats = storage.stats_for_scope(&tenant_scope).await?;
    ///
    /// println!("Tenant has {} vectors, estimated cost: ${:.2}",
    ///          stats.vector_count, stats.estimated_cost);
    /// # Ok(())
    /// # }
    /// ```
    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>;

    /// Save vectors to persistent storage (if supported)
    async fn save(&self) -> Result<()> {
        // Default implementation for storages that don't support persistence
        Ok(())
    }

    /// Load vectors from persistent storage (if supported)
    async fn load(&self) -> Result<()> {
        // Default implementation for storages that don't support persistence
        Ok(())
    }
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

// ============================================================================
// HNSW-Specific Types
// ============================================================================

/// HNSW-specific storage trait with namespace support
#[async_trait]
pub trait HNSWStorage: VectorStorage {
    /// Configure HNSW parameters
    fn configure_hnsw(&mut self, config: HNSWConfig);

    /// Build or rebuild the HNSW index
    async fn build_index(&self) -> Result<()>;

    /// Create tenant-specific namespace/index
    async fn create_namespace(&self, namespace: &str) -> Result<()>;

    /// Delete a namespace and all its vectors
    async fn delete_namespace(&self, namespace: &str) -> Result<()>;

    /// Get current HNSW parameters
    fn hnsw_params(&self) -> &HNSWConfig;

    /// Optimize index for better performance
    async fn optimize_index(&self) -> Result<()>;

    /// Get namespace statistics
    async fn namespace_stats(&self, namespace: &str) -> Result<NamespaceStats>;

    /// Save index to persistent storage
    async fn save(&self) -> Result<()>;
}

/// HNSW algorithm configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HNSWConfig {
    /// Number of bi-directional links created for each node (16-64 typical)
    /// Higher values give better recall but use more memory
    pub m: usize,

    /// Size of the dynamic candidate list during construction (200 typical)
    /// Higher values give better recall but slower construction
    pub ef_construction: usize,

    /// Size of the dynamic candidate list during search (50-200 typical)
    /// Higher values give better recall but slower search
    pub ef_search: usize,

    /// Maximum number of elements that can be stored
    pub max_elements: usize,

    /// Random seed for reproducible index construction
    pub seed: Option<u64>,

    /// Distance metric to use
    pub metric: DistanceMetric,

    /// Whether to allow replacing deleted elements
    pub allow_replace_deleted: bool,

    /// Number of threads to use for index operations
    pub num_threads: Option<usize>,

    /// Number of hierarchical layers in the graph (auto-calculated if None)
    pub nb_layers: Option<usize>,

    /// Batch size for parallel insertion operations
    pub parallel_batch_size: Option<usize>,

    /// Enable memory-mapped storage for large datasets (future feature)
    pub enable_mmap: bool,

    /// Memory map sync interval in seconds (if mmap enabled)
    pub mmap_sync_interval: Option<u64>,
}

impl Default for HNSWConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            max_elements: 1_000_000,
            seed: None,
            metric: DistanceMetric::Cosine,
            allow_replace_deleted: true,
            num_threads: None,
            nb_layers: None,
            parallel_batch_size: Some(128),
            enable_mmap: false,
            mmap_sync_interval: Some(60),
        }
    }
}

impl HNSWConfig {
    /// Create a configuration optimized for speed
    #[must_use]
    pub fn fast() -> Self {
        Self {
            m: 12,
            ef_construction: 100,
            ef_search: 50,
            ..Default::default()
        }
    }

    /// Create a configuration optimized for accuracy
    #[must_use]
    pub fn accurate() -> Self {
        Self {
            m: 32,
            ef_construction: 400,
            ef_search: 200,
            ..Default::default()
        }
    }

    /// Create a configuration balanced between speed and accuracy
    #[must_use]
    pub fn balanced() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            ..Default::default()
        }
    }
}

/// Distance metrics supported by HNSW
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Cosine similarity (most common for embeddings)
    Cosine,

    /// Euclidean (L2) distance
    Euclidean,

    /// Inner product (dot product)
    InnerProduct,

    /// Manhattan (L1) distance
    Manhattan,
}

impl Default for DistanceMetric {
    fn default() -> Self {
        Self::Cosine
    }
}

/// Statistics for a specific namespace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamespaceStats {
    /// Namespace identifier
    pub namespace: String,

    /// Number of vectors in this namespace
    pub vector_count: usize,

    /// Total memory used in bytes
    pub memory_bytes: usize,

    /// Average number of connections per node
    pub avg_connections: f32,

    /// Index build time in milliseconds
    pub build_time_ms: Option<u64>,

    /// Last optimization timestamp
    pub last_optimized: Option<SystemTime>,
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

    #[test]
    fn test_hnsw_config_presets() {
        let fast = HNSWConfig::fast();
        assert_eq!(fast.m, 12);
        assert_eq!(fast.ef_construction, 100);

        let accurate = HNSWConfig::accurate();
        assert_eq!(accurate.m, 32);
        assert_eq!(accurate.ef_construction, 400);

        let balanced = HNSWConfig::balanced();
        assert_eq!(balanced.m, 16);
        assert_eq!(balanced.ef_construction, 200);
    }

    #[test]
    fn test_distance_metric_default() {
        assert_eq!(DistanceMetric::default(), DistanceMetric::Cosine);
    }
}
