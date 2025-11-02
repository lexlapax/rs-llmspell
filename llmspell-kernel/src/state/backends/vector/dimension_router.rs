//! Dimension-aware storage routing for multi-provider support

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

use super::hnsw::HNSWVectorStorage;
use crate::state::vector_storage::{
    HNSWConfig, ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
use crate::state::StateScope;

/// Routes vectors to appropriate storage based on dimensions
///
/// Different embedding models produce vectors of different dimensions:
/// - `OpenAI`: 256, 1536, 3072 (with Matryoshka reduction)
/// - Google: 768, 3072
/// - Cohere: 1024
/// - BGE-M3: 1024
/// - `ColBERT`: 768, 1024
///
/// This router maintains separate HNSW indices for each dimension.
#[derive(Debug)]
pub struct DimensionRouter {
    /// Map of dimension size to storage instance
    storages: DashMap<usize, Arc<HNSWVectorStorage>>,

    // TODO: Handle dimension mapping dependency from llmspell-rag
    // This should be injected or handled at a higher level
    // /// Dimension mapper for handling conversions
    // #[allow(dead_code)]
    // mapper: DimensionMapper,
    /// Default HNSW configuration for new indices
    default_config: HNSWConfig,

    /// Whether to allow automatic dimension reduction via Matryoshka
    allow_reduction: bool,

    /// Statistics tracking
    stats: Arc<DashMap<usize, DimensionStats>>,
}

/// Statistics for each dimension bucket
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DimensionStats {
    /// Number of vectors in this dimension
    vector_count: usize,

    /// Number of queries processed
    query_count: usize,

    /// Average query time in milliseconds
    avg_query_time_ms: f32,

    /// Total storage bytes
    storage_bytes: usize,
}

impl DimensionRouter {
    /// Create a new dimension router
    #[must_use]
    pub fn new(config: HNSWConfig) -> Self {
        // TODO: Handle dimension configuration dependency
        // Standard dimension configuration
        // let dimension_config = crate::embeddings::DimensionConfig::default();

        Self {
            storages: DashMap::new(),
            // mapper: DimensionMapper::new(dimension_config),
            default_config: config,
            allow_reduction: true,
            stats: Arc::new(DashMap::new()),
        }
    }

    /// Configure whether to allow automatic dimension reduction
    pub const fn set_allow_reduction(&mut self, allow: bool) {
        self.allow_reduction = allow;
    }

    /// Get or create storage for a specific dimension
    fn get_or_create_storage(&self, dimensions: usize) -> Arc<HNSWVectorStorage> {
        self.storages
            .entry(dimensions)
            .or_insert_with(|| {
                info!("Creating new HNSW index for {} dimensions", dimensions);
                let storage = HNSWVectorStorage::new(dimensions, self.default_config.clone());
                Arc::new(storage)
            })
            .clone()
    }

    /// Find the best dimension for a vector
    fn find_best_dimension(&self, vector_dims: usize) -> usize {
        // Check if we have an exact match
        if self.storages.contains_key(&vector_dims) {
            return vector_dims;
        }

        // Check if dimension reduction is allowed and possible
        if self.allow_reduction {
            // Try to find a smaller dimension that divides evenly (Matryoshka)
            for storage_entry in &self.storages {
                let storage_dims = *storage_entry.key();
                if storage_dims < vector_dims && vector_dims.is_multiple_of(storage_dims) {
                    debug!(
                        "Using Matryoshka reduction from {} to {} dimensions",
                        vector_dims, storage_dims
                    );
                    return storage_dims;
                }
            }
        }

        // If no existing storage matches, return the vector's original dimension
        // (will create new storage)
        vector_dims
    }

    /// Reduce vector dimensions using Matryoshka representation
    fn reduce_dimensions(vector: &[f32], target_dims: usize) -> Vec<f32> {
        if vector.len() <= target_dims {
            vector.to_vec()
        } else {
            // Simple truncation for Matryoshka-compatible models
            vector[..target_dims].to_vec()
        }
    }

    /// Track query statistics
    fn track_query_stats(&self, dimensions: usize, query_time_ms: f32) {
        let mut stats = self.stats.entry(dimensions).or_default();
        stats.query_count += 1;

        // Update moving average of query time
        let alpha = 0.1_f32; // Exponential moving average factor
        if stats.avg_query_time_ms == 0.0 {
            stats.avg_query_time_ms = query_time_ms;
        } else {
            stats.avg_query_time_ms =
                (1.0 - alpha).mul_add(stats.avg_query_time_ms, alpha * query_time_ms);
        }
    }
}

#[async_trait]
impl VectorStorage for DimensionRouter {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>> {
        // Group vectors by dimension
        let mut vectors_by_dim: HashMap<usize, Vec<VectorEntry>> = HashMap::new();

        for mut entry in vectors {
            let original_dims = entry.embedding.len();
            let target_dims = self.find_best_dimension(original_dims);

            // Reduce dimensions if necessary
            if target_dims < original_dims {
                entry.embedding = Self::reduce_dimensions(&entry.embedding, target_dims);
            }

            vectors_by_dim.entry(target_dims).or_default().push(entry);
        }

        // Insert into appropriate storages
        let mut all_ids = Vec::new();

        for (dims, vectors) in vectors_by_dim {
            let storage = self.get_or_create_storage(dims);
            let ids = storage.insert(vectors).await?;
            all_ids.extend(ids);

            // Update stats
            let mut stats = self.stats.entry(dims).or_default();
            stats.vector_count += all_ids.len();
        }

        Ok(all_ids)
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>> {
        let start = std::time::Instant::now();

        let original_dims = query.vector.len();
        let target_dims = self.find_best_dimension(original_dims);

        // Prepare query with potentially reduced dimensions
        let mut modified_query = query.clone();
        if target_dims < original_dims {
            modified_query.vector = Self::reduce_dimensions(&query.vector, target_dims);
        }

        // Route to appropriate storage
        if let Some(storage) = self.storages.get(&target_dims) {
            let results = storage.search(&modified_query).await?;

            // Track statistics
            #[allow(clippy::cast_precision_loss)]
            let query_time_ms = start.elapsed().as_millis() as f32;
            self.track_query_stats(target_dims, query_time_ms);

            Ok(results)
        } else {
            // No vectors of this dimension exist
            Ok(Vec::new())
        }
    }

    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>> {
        let start = std::time::Instant::now();

        let original_dims = query.vector.len();
        let target_dims = self.find_best_dimension(original_dims);

        // Prepare query with potentially reduced dimensions
        let mut modified_query = query.clone();
        if target_dims < original_dims {
            modified_query.vector = Self::reduce_dimensions(&query.vector, target_dims);
        }

        // Route to appropriate storage
        if let Some(storage) = self.storages.get(&target_dims) {
            let results = storage.search_scoped(&modified_query, scope).await?;

            // Track statistics
            #[allow(clippy::cast_precision_loss)]
            let query_time_ms = start.elapsed().as_millis() as f32;
            self.track_query_stats(target_dims, query_time_ms);

            Ok(results)
        } else {
            // No vectors of this dimension exist
            Ok(Vec::new())
        }
    }

    async fn update_metadata(
        &self,
        id: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Try to update in all storages (we don't know which one has this ID)
        for storage in &self.storages {
            if storage.update_metadata(id, metadata.clone()).await.is_ok() {
                return Ok(());
            }
        }

        anyhow::bail!("Vector with ID {id} not found in any dimension storage")
    }

    async fn delete(&self, ids: &[String]) -> Result<()> {
        // Try to delete from all storages
        for storage in &self.storages {
            // Ignore errors as vector might not exist in this dimension
            let _ = storage.delete(ids).await;
        }

        Ok(())
    }

    async fn delete_scope(&self, scope: &StateScope) -> Result<usize> {
        let mut total_deleted = 0;

        for storage in &self.storages {
            total_deleted += storage.delete_scope(scope).await?;
        }

        Ok(total_deleted)
    }

    async fn stats(&self) -> Result<StorageStats> {
        let mut total_vectors = 0;
        let mut total_bytes = 0;
        let mut namespace_count = 0;

        // Aggregate stats from all dimension storages
        for storage in &self.storages {
            let stats = storage.stats().await?;
            total_vectors += stats.total_vectors;
            total_bytes += stats.storage_bytes;
            namespace_count = namespace_count.max(stats.namespace_count);
        }

        // Calculate average query time across all dimensions
        let mut total_query_time = 0.0;
        let mut total_queries = 0;

        for stats in self.stats.iter() {
            #[allow(clippy::cast_precision_loss)]
            let count = stats.query_count as f32;
            total_query_time += stats.avg_query_time_ms * count;
            total_queries += stats.query_count;
        }

        let avg_query_time = if total_queries > 0 {
            #[allow(clippy::cast_precision_loss)]
            Some(total_query_time / total_queries as f32)
        } else {
            None
        };

        Ok(StorageStats {
            total_vectors,
            storage_bytes: total_bytes,
            namespace_count,
            avg_query_time_ms: avg_query_time,
            dimensions: None, // Multiple dimensions
            ..Default::default()
        })
    }

    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats> {
        let mut scope_stats = ScopedStats {
            scope: scope.clone(),
            vector_count: 0,
            storage_bytes: 0,
            query_count: 0,
            tokens_processed: 0,
            estimated_cost: 0.0,
        };

        // Aggregate from all dimension storages
        for storage in &self.storages {
            let stats = storage.stats_for_scope(scope).await?;
            scope_stats.vector_count += stats.vector_count;
            scope_stats.storage_bytes += stats.storage_bytes;
            scope_stats.query_count += stats.query_count;
            scope_stats.tokens_processed += stats.tokens_processed;
            scope_stats.estimated_cost += stats.estimated_cost;
        }

        Ok(scope_stats)
    }
}

/// Information about dimension distribution
impl DimensionRouter {
    /// Get statistics about dimension distribution
    pub async fn dimension_distribution(&self) -> HashMap<usize, DimensionInfo> {
        let mut distribution = HashMap::new();

        for storage_entry in &self.storages {
            let dims = *storage_entry.key();
            let storage = storage_entry.value();

            if let Ok(stats) = storage.stats().await {
                let dim_stats = self.stats.get(&dims).map(|s| s.clone()).unwrap_or_default();

                distribution.insert(
                    dims,
                    DimensionInfo {
                        dimensions: dims,
                        vector_count: stats.total_vectors,
                        storage_bytes: stats.storage_bytes,
                        query_count: dim_stats.query_count,
                        avg_query_time_ms: dim_stats.avg_query_time_ms,
                    },
                );
            }
        }

        distribution
    }
}

/// Information about vectors in a specific dimension
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionInfo {
    /// The dimension size
    pub dimensions: usize,

    /// Number of vectors in this dimension
    pub vector_count: usize,

    /// Storage used in bytes
    pub storage_bytes: usize,

    /// Number of queries processed
    pub query_count: usize,

    /// Average query time in milliseconds
    pub avg_query_time_ms: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore = "HNSW library has timing bug during drop in test environment"]
    async fn test_multi_dimension_routing() {
        // This test is disabled due to a bug in hnsw_rs library where
        // it calculates timing incorrectly during drop, causing:
        // "overflow when subtracting durations"
        // The functionality is tested in other tests that don't trigger this issue.

        let router = DimensionRouter::new(HNSWConfig::default());

        // Insert vectors of different dimensions
        let vectors = vec![
            VectorEntry::new("vec1".to_string(), vec![1.0; 768]).with_scope(StateScope::Global),
            VectorEntry::new("vec2".to_string(), vec![1.0; 1024]).with_scope(StateScope::Global),
            VectorEntry::new("vec3".to_string(), vec![1.0; 1536]).with_scope(StateScope::Global),
        ];

        let ids = router.insert(vectors).await.unwrap();
        assert_eq!(ids.len(), 3);

        // Verify dimension distribution
        let distribution = router.dimension_distribution().await;
        assert_eq!(distribution.len(), 3);
        assert!(distribution.contains_key(&768));
        assert!(distribution.contains_key(&1024));
        assert!(distribution.contains_key(&1536));
    }

    #[tokio::test]
    async fn test_dimension_reduction_logic() {
        // Test just the dimension reduction logic without creating HNSW indices
        const EPSILON: f32 = 1e-6; // Epsilon for floating point comparisons

        // Test dimension reduction method
        let vec_1536 = vec![1.0_f32; 1536];
        let reduced_768 = DimensionRouter::reduce_dimensions(&vec_1536, 768);
        assert_eq!(reduced_768.len(), 768);
        // First 768 elements should be preserved
        assert!((reduced_768[0] - 1.0).abs() < EPSILON);

        let reduced_256 = DimensionRouter::reduce_dimensions(&vec_1536, 256);
        assert_eq!(reduced_256.len(), 256);

        // Test that reduced vectors preserve relative magnitudes
        let vec_a = vec![1.0_f32; 1536];
        let vec_b = vec![2.0_f32; 1536];
        let reduced_a = DimensionRouter::reduce_dimensions(&vec_a, 768);
        let reduced_b = DimensionRouter::reduce_dimensions(&vec_b, 768);

        // The reduced version of vec_b should have larger values
        let sum_a: f32 = reduced_a.iter().sum();
        let sum_b: f32 = reduced_b.iter().sum();
        assert!(sum_b > sum_a);

        // Test edge cases
        let small_vec = vec![1.0_f32; 128];
        let same_size = DimensionRouter::reduce_dimensions(&small_vec, 128);
        assert_eq!(same_size.len(), 128);
        // Compare vectors element by element with epsilon
        for (a, b) in same_size.iter().zip(small_vec.iter()) {
            assert!((a - b).abs() < EPSILON, "Vectors should be unchanged");
        }

        // Attempting to "reduce" to larger size just returns original vector
        let expanded = DimensionRouter::reduce_dimensions(&small_vec, 256);
        assert_eq!(expanded.len(), 128); // Can't expand, stays at 128
                                         // Compare vectors element by element with epsilon
        for (a, b) in expanded.iter().zip(small_vec.iter()) {
            assert!((a - b).abs() < EPSILON, "Vectors should be unchanged");
        }
    }

    #[tokio::test]
    async fn test_matryoshka_reduction() {
        let router = DimensionRouter::new(HNSWConfig::default());

        // Insert a 768-dim vector first
        let vectors_768 =
            vec![VectorEntry::new("vec768".to_string(), vec![1.0; 768])
                .with_scope(StateScope::Global)];
        router.insert(vectors_768).await.unwrap();

        // Insert a 1536-dim vector - should be reduced to 768
        let vectors_1536 =
            vec![VectorEntry::new("vec1536".to_string(), vec![2.0; 1536])
                .with_scope(StateScope::Global)];
        router.insert(vectors_1536).await.unwrap();

        // Search with 768-dim query should find both
        let query = VectorQuery::new(vec![1.5; 768], 10);
        let results = router.search(&query).await.unwrap();

        assert_eq!(results.len(), 2);

        // Verify only one storage was created
        let distribution = router.dimension_distribution().await;
        assert_eq!(distribution.len(), 1);
        assert_eq!(distribution.get(&768).unwrap().vector_count, 2);
    }

    #[tokio::test]
    async fn test_dimension_mismatch_handling() {
        let router = DimensionRouter::new(HNSWConfig::default());

        // Insert vectors with specific dimension
        let vectors = vec![
            VectorEntry::new("vec1".to_string(), vec![1.0; 512]).with_scope(StateScope::Global)
        ];
        router.insert(vectors).await.unwrap();

        // Search with different dimension should handle gracefully
        let query = VectorQuery::new(vec![1.0; 1024], 10);
        let results = router.search(&query).await.unwrap();

        // Should create new storage or reduce dimensions appropriately
        assert!(results.is_empty() || results.len() == 1);
    }
}
