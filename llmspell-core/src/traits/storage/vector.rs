//! Vector storage traits
//!
//! Defines core abstractions for vector storage operations including:
//! - `VectorStorage`: Core trait for vector embedding storage with multi-tenant support
//! - `HNSWStorage`: Extension trait for HNSW-specific operations
//!
//! These traits enable pluggable vector storage backends (in-memory, SQLite with vectorlite,
//! PostgreSQL with pgvector/pgvectorscale) while keeping domain logic backend-agnostic.
//!
//! Migrated from llmspell-storage/src/vector_storage.rs as part of Phase 13c.3.

use crate::state::StateScope;
use crate::types::storage::vector::{
    HNSWConfig, NamespaceStats, ScopedStats, StorageStats, VectorEntry,
    VectorQuery, VectorResult,
};
use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

/// Core vector storage trait with multi-tenant support
///
/// Provides a unified interface for storing and searching vector embeddings across
/// different backend implementations. Supports multi-tenancy via scopes, bi-temporal
/// queries, and comprehensive metadata filtering.
///
/// # Performance Characteristics
///
/// | Backend | Insert | Search | Storage |
/// |---------|--------|--------|---------|
/// | Memory | <1ms | <2ms | RAM-only |
/// | SQLite+vectorlite | ~5ms | ~10ms | Persistent |
/// | PostgreSQL+pgvector | ~15ms | ~20ms | Distributed |
///
/// # Multi-Tenancy
///
/// All operations support scope-based tenant isolation through `StateScope`.
/// Use `search_scoped()` and `delete_scope()` for strict tenant boundaries.
///
/// # Examples
///
/// ```no_run
/// # use llmspell_core::traits::storage::VectorStorage;
/// # use llmspell_core::types::storage::vector::{VectorEntry, VectorQuery};
/// # use llmspell_core::state::StateScope;
/// # async fn example(storage: impl VectorStorage) -> anyhow::Result<()> {
/// // Insert vectors
/// let entry = VectorEntry::new("doc1".to_string(), vec![0.1, 0.2, 0.3])
///     .with_scope(StateScope::User("alice".to_string()));
/// storage.insert(vec![entry]).await?;
///
/// // Search with scope isolation
/// let query = VectorQuery::new(vec![0.15, 0.25, 0.35], 10);
/// let results = storage.search_scoped(&query, &StateScope::User("alice".to_string())).await?;
/// # Ok(())
/// # }
/// ```
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
    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>;

    /// Save vectors to persistent storage (if supported)
    ///
    /// Default implementation for storages that don't support persistence.
    async fn save(&self) -> Result<()> {
        Ok(())
    }

    /// Load vectors from persistent storage (if supported)
    ///
    /// Default implementation for storages that don't support persistence.
    async fn load(&self) -> Result<()> {
        Ok(())
    }
}

/// HNSW-specific storage trait with namespace support
///
/// Extends `VectorStorage` with operations specific to HNSW (Hierarchical Navigable Small World)
/// index management. Provides fine-grained control over index building, optimization,
/// and namespace management for multi-tenant deployments.
///
/// # HNSW Algorithm
///
/// HNSW builds a multi-layered proximity graph that enables fast approximate nearest neighbor
/// search with high recall. Key parameters:
/// - `m`: Connections per node (16-64 typical) - higher = better recall, more memory
/// - `ef_construction`: Build-time candidate list (200 typical) - higher = better quality, slower
/// - `ef_search`: Search-time candidate list (50-200) - higher = better recall, slower
///
/// # Examples
///
/// ```no_run
/// # use llmspell_core::traits::storage::HNSWStorage;
/// # use llmspell_core::types::storage::vector::HNSWConfig;
/// # async fn example(storage: &mut impl HNSWStorage) -> anyhow::Result<()> {
/// // Configure for high accuracy
/// storage.configure_hnsw(HNSWConfig::accurate());
///
/// // Build index
/// storage.build_index().await?;
///
/// // Create tenant-specific namespace
/// storage.create_namespace("tenant:acme").await?;
///
/// // Optimize for search performance
/// storage.optimize_index().await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait HNSWStorage: VectorStorage {
    /// Configure HNSW parameters
    ///
    /// Updates the HNSW algorithm configuration. Must be called before `build_index()`
    /// to take effect. Changes after index building may require rebuilding.
    fn configure_hnsw(&mut self, config: HNSWConfig);

    /// Build or rebuild the HNSW index
    ///
    /// Constructs the proximity graph from all stored vectors. This operation
    /// can be time-consuming for large datasets.
    ///
    /// # Performance
    ///
    /// - 1M vectors @ 768 dims: ~5-10 minutes (with default config)
    /// - Memory usage: ~2-4x vector storage size during build
    async fn build_index(&self) -> Result<()>;

    /// Create tenant-specific namespace/index
    ///
    /// Enables logical separation of vectors by namespace while sharing
    /// the same storage backend. Useful for multi-tenant deployments.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Unique identifier for the namespace
    async fn create_namespace(&self, namespace: &str) -> Result<()>;

    /// Delete a namespace and all its vectors
    ///
    /// Permanently removes a namespace and all associated vectors.
    /// Cannot be undone.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to delete
    async fn delete_namespace(&self, namespace: &str) -> Result<()>;

    /// Get current HNSW parameters
    ///
    /// Returns the active HNSW configuration being used for index operations.
    fn hnsw_params(&self) -> &HNSWConfig;

    /// Optimize index for better performance
    ///
    /// Performs maintenance operations to improve search performance:
    /// - Prunes deleted nodes
    /// - Rebalances layers
    /// - Compacts memory
    async fn optimize_index(&self) -> Result<()>;

    /// Get namespace statistics
    ///
    /// Returns detailed metrics for a specific namespace including
    /// vector counts, memory usage, and index health.
    ///
    /// # Arguments
    ///
    /// * `namespace` - Namespace to get statistics for
    async fn namespace_stats(&self, namespace: &str) -> Result<NamespaceStats>;

    /// Save index to persistent storage
    ///
    /// Persists the HNSW index structure to disk for faster startup.
    /// Required for persistent backends.
    async fn save(&self) -> Result<()>;
}
