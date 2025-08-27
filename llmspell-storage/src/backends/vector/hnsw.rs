//! HNSW vector storage implementation with multi-tenant support

use anyhow::Result;
use async_trait::async_trait;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::vector_storage::{
    DistanceMetric, HNSWConfig, HNSWStorage, NamespaceStats, ScopedStats, StorageStats,
    VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
use llmspell_state_traits::StateScope;

/// HNSW vector storage with multi-tenant namespace support
#[derive(Debug)]
pub struct HNSWVectorStorage {
    /// Map of namespace name to HNSW index
    namespaces: DashMap<String, Arc<RwLock<NamespaceIndex>>>,

    /// Global configuration
    config: HNSWConfig,

    /// Persistence directory
    persistence_dir: Option<PathBuf>,

    /// Metadata storage for each vector
    metadata: DashMap<String, VectorMetadata>,

    /// Dimension of vectors (must be consistent)
    dimensions: usize,
}

// We'll use a simple vector storage for now since hnsw crate has complex const generics
// In production, we would use const generics properly or use hnsw_rs crate instead
type SimpleHnsw = Vec<(Vec<f32>, String, usize)>; // (vector, id, index)

/// Individual namespace index
#[derive(Debug)]
struct NamespaceIndex {
    /// The vectors in this namespace (simplified for now)
    vectors: SimpleHnsw,

    /// Namespace statistics
    stats: NamespaceStats,

    /// Creation time
    #[allow(dead_code)]
    created_at: std::time::SystemTime,
}

/// Vector metadata storage
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VectorMetadata {
    /// Original vector entry data
    entry: VectorEntry,

    /// Namespace this vector belongs to
    namespace: String,

    /// Internal index ID
    index_id: usize,
}

impl HNSWVectorStorage {
    /// Create a new HNSW vector storage
    #[must_use]
    pub fn new(dimensions: usize, config: HNSWConfig) -> Self {
        Self {
            namespaces: DashMap::new(),
            config,
            persistence_dir: None,
            metadata: DashMap::new(),
            dimensions,
        }
    }

    /// Set persistence directory for saving indices to disk
    #[must_use]
    pub fn with_persistence(mut self, dir: PathBuf) -> Self {
        self.persistence_dir = Some(dir);
        self
    }

    /// Get or create a namespace index
    fn get_or_create_namespace(&self, namespace: &str) -> Arc<RwLock<NamespaceIndex>> {
        self.namespaces
            .entry(namespace.to_string())
            .or_insert_with(|| {
                Arc::new(RwLock::new(NamespaceIndex {
                    vectors: Vec::new(),
                    stats: NamespaceStats {
                        namespace: namespace.to_string(),
                        vector_count: 0,
                        memory_bytes: 0,
                        avg_connections: 0.0,
                        build_time_ms: None,
                        last_optimized: None,
                    },
                    created_at: std::time::SystemTime::now(),
                }))
            })
            .clone()
    }

    /// Get namespace name from `StateScope`
    fn scope_to_namespace(scope: &StateScope) -> String {
        match scope {
            StateScope::Global => "__global__".to_string(),
            StateScope::User(id) => format!("user:{id}"),
            StateScope::Session(id) => format!("session:{id}"),
            StateScope::Agent(id) => format!("agent:{id}"),
            StateScope::Tool(id) => format!("tool:{id}"),
            StateScope::Workflow(id) => format!("workflow:{id}"),
            StateScope::Hook(id) => format!("hook:{id}"),
            StateScope::Custom(s) if s.starts_with("tenant:") => s.clone(),
            StateScope::Custom(s) => format!("custom:{s}"),
        }
    }

    /// Calculate cosine distance (converted to similarity)
    fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 1.0; // Maximum distance for zero vectors
        }

        // Convert cosine similarity to distance (0 = identical, 2 = opposite)
        1.0 - (dot / (norm_a.sqrt() * norm_b.sqrt()))
    }

    /// Calculate euclidean distance
    fn euclidean_distance(a: &[f32], b: &[f32]) -> f32 {
        let mut sum = 0.0;
        for i in 0..a.len().min(b.len()) {
            let diff = a[i] - b[i];
            sum += diff * diff;
        }
        sum.sqrt()
    }

    /// Calculate distance based on configured metric
    fn calculate_distance(&self, a: &[f32], b: &[f32]) -> f32 {
        match self.config.metric {
            DistanceMetric::Cosine => Self::cosine_distance(a, b),
            DistanceMetric::Euclidean => Self::euclidean_distance(a, b),
            DistanceMetric::InnerProduct => {
                // Negative dot product (so that higher is better becomes lower distance)
                let mut dot = 0.0;
                for i in 0..a.len().min(b.len()) {
                    dot += a[i] * b[i];
                }
                -dot
            }
            DistanceMetric::Manhattan => {
                let mut sum = 0.0;
                for i in 0..a.len().min(b.len()) {
                    sum += (a[i] - b[i]).abs();
                }
                sum
            }
        }
    }
}

#[async_trait]
impl VectorStorage for HNSWVectorStorage {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>> {
        let mut ids = Vec::with_capacity(vectors.len());

        for entry in vectors {
            // Validate dimensions
            if entry.embedding.len() != self.dimensions {
                anyhow::bail!(
                    "Vector dimension mismatch: expected {}, got {}",
                    self.dimensions,
                    entry.embedding.len()
                );
            }

            let namespace = Self::scope_to_namespace(&entry.scope);
            let namespace_index = self.get_or_create_namespace(&namespace);

            // Generate ID if not provided
            let id = if entry.id.is_empty() {
                Uuid::new_v4().to_string()
            } else {
                entry.id.clone()
            };

            // Store metadata
            let metadata = VectorMetadata {
                entry: entry.clone(),
                namespace: namespace.clone(),
                index_id: 0, // Will be updated after insertion
            };

            // Insert into simplified index
            {
                let mut index = namespace_index.write();
                let index_id = index.vectors.len();

                // Store vector with ID and index
                index
                    .vectors
                    .push((entry.embedding.clone(), id.clone(), index_id));

                // Update metadata with actual index ID
                let mut metadata = metadata;
                metadata.index_id = index_id;
                self.metadata.insert(id.clone(), metadata);

                // Update stats
                index.stats.vector_count += 1;
            }

            ids.push(id);
        }

        debug!("Inserted {} vectors: {:?}", ids.len(), ids);
        Ok(ids)
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>> {
        // Default to global namespace if no scope specified
        let scope = query.scope.as_ref().unwrap_or(&StateScope::Global);
        self.search_scoped(query, scope).await
    }

    #[allow(clippy::significant_drop_tightening)]
    async fn search_scoped(
        &self,
        query: &VectorQuery,
        search_scope: &StateScope,
    ) -> Result<Vec<VectorResult>> {
        // Validate query vector dimensions
        if query.vector.len() != self.dimensions {
            anyhow::bail!(
                "Query vector dimension mismatch: expected {}, got {}",
                self.dimensions,
                query.vector.len()
            );
        }

        let namespace = Self::scope_to_namespace(search_scope);

        // Get the namespace index
        let namespace_index = self
            .namespaces
            .get(&namespace)
            .ok_or_else(|| anyhow::anyhow!("Namespace {} not found", namespace))?;

        let mut results = Vec::new();

        {
            let index = namespace_index.read();

            // Simple brute-force search for now (would use proper HNSW in production)
            let mut distances: Vec<(f32, &str, usize)> = Vec::new();

            for (vec, id, idx) in &index.vectors {
                let distance = self.calculate_distance(&query.vector, vec);
                distances.push((distance, id.as_str(), *idx));
            }

            // Sort by distance and take top k
            distances.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            distances.truncate(query.k);

            // Convert to our result format
            for (distance, id, _idx) in distances {
                // Check threshold if specified
                if let Some(threshold) = query.threshold {
                    let similarity = match self.config.metric {
                        DistanceMetric::Cosine => 1.0 - (distance / 2.0), // Convert back to similarity
                        _ => 1.0 / (1.0 + distance),                      // Generic conversion
                    };

                    if similarity < threshold {
                        continue;
                    }
                }

                // Get metadata for this vector
                if let Some(metadata_entry) = self.metadata.get(id) {
                    let metadata = metadata_entry.value();
                    let similarity_score = match self.config.metric {
                        DistanceMetric::Cosine => 1.0 - (distance / 2.0),
                        _ => 1.0 / (1.0 + distance),
                    };

                    // Apply metadata filters if specified
                    if let Some(filters) = &query.filter {
                        let matches = filters
                            .iter()
                            .all(|(key, value)| metadata.entry.metadata.get(key) == Some(value));

                        if !matches {
                            continue;
                        }
                    }

                    results.push(VectorResult {
                        id: metadata.entry.id.clone(),
                        score: similarity_score,
                        vector: if query.include_metadata {
                            Some(metadata.entry.embedding.clone())
                        } else {
                            None
                        },
                        metadata: if query.include_metadata {
                            Some(metadata.entry.metadata.clone())
                        } else {
                            None
                        },
                        distance,
                    });
                }
            }
        }

        Ok(results)
    }

    async fn update_metadata(
        &self,
        id: &str,
        metadata: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(mut entry) = self.metadata.get_mut(id) {
            entry.entry.metadata = metadata;
            Ok(())
        } else {
            anyhow::bail!("Vector with ID {} not found", id)
        }
    }

    async fn delete(&self, ids: &[String]) -> Result<()> {
        for id in ids {
            if self.metadata.remove(id).is_none() {
                warn!("Vector with ID {} not found during deletion", id);
            }
            // Note: hnsw crate doesn't support deletion directly
            // In production, we'd mark as deleted and handle during compaction
        }
        Ok(())
    }

    async fn delete_scope(&self, deletion_scope: &StateScope) -> Result<usize> {
        let namespace = Self::scope_to_namespace(deletion_scope);

        // Remove namespace index
        if let Some((_, namespace_index)) = self.namespaces.remove(&namespace) {
            let count = namespace_index.read().stats.vector_count;

            // Remove all metadata for this namespace
            self.metadata.retain(|_, v| v.namespace != namespace);

            Ok(count)
        } else {
            Ok(0)
        }
    }

    async fn stats(&self) -> Result<StorageStats> {
        let mut total_vectors = 0;
        let mut namespace_count = 0;

        for namespace in &self.namespaces {
            namespace_count += 1;
            let index = namespace.value().read();
            total_vectors += index.stats.vector_count;
        }

        Ok(StorageStats {
            total_vectors,
            storage_bytes: total_vectors * self.dimensions * 4, // Approximate
            namespace_count,
            dimensions: Some(self.dimensions),
            ..Default::default()
        })
    }

    async fn stats_for_scope(&self, stats_scope: &StateScope) -> Result<ScopedStats> {
        let namespace = Self::scope_to_namespace(stats_scope);

        self.namespaces.get(&namespace).map_or_else(
            || {
                Ok(ScopedStats {
                    scope: stats_scope.clone(),
                    vector_count: 0,
                    storage_bytes: 0,
                    query_count: 0,
                    tokens_processed: 0,
                    estimated_cost: 0.0,
                })
            },
            |namespace_index| {
                let index = namespace_index.read();
                Ok(ScopedStats {
                    scope: stats_scope.clone(),
                    vector_count: index.stats.vector_count,
                    storage_bytes: index.stats.vector_count * self.dimensions * 4,
                    query_count: 0,      // Would track this in production
                    tokens_processed: 0, // Would track this in production
                    estimated_cost: 0.0, // Would calculate based on usage
                })
            },
        )
    }
}

#[async_trait]
impl HNSWStorage for HNSWVectorStorage {
    fn configure_hnsw(&mut self, config: HNSWConfig) {
        self.config = config;
    }

    async fn build_index(&self) -> Result<()> {
        // HNSW builds incrementally, no explicit build needed
        info!("HNSW index builds incrementally during insertion");
        Ok(())
    }

    async fn create_namespace(&self, namespace: &str) -> Result<()> {
        self.get_or_create_namespace(namespace);
        info!("Created namespace: {}", namespace);
        Ok(())
    }

    async fn delete_namespace(&self, namespace: &str) -> Result<()> {
        if self.namespaces.remove(namespace).is_some() {
            // Remove all metadata for this namespace
            self.metadata.retain(|_, v| v.namespace != namespace);
            info!("Deleted namespace: {}", namespace);
            Ok(())
        } else {
            anyhow::bail!("Namespace {} not found", namespace)
        }
    }

    fn hnsw_params(&self) -> &HNSWConfig {
        &self.config
    }

    async fn optimize_index(&self) -> Result<()> {
        // HNSW doesn't have explicit optimization
        // Could implement compaction here
        info!("HNSW index optimization not required");
        Ok(())
    }

    async fn namespace_stats(&self, namespace: &str) -> Result<NamespaceStats> {
        if let Some(namespace_index) = self.namespaces.get(namespace) {
            Ok(namespace_index.read().stats.clone())
        } else {
            anyhow::bail!("Namespace {} not found", namespace)
        }
    }
}

/// Persistence support
impl HNSWVectorStorage {
    /// Save index to disk
    ///
    /// # Errors
    ///
    /// Returns error if persistence fails
    #[allow(clippy::unused_async)]
    pub async fn save(&self, _path: &Path) -> Result<()> {
        // TODO: Implement persistence
        // Would serialize namespaces and metadata to disk
        warn!("Persistence not yet implemented");
        Ok(())
    }

    /// Load index from disk
    ///
    /// # Errors
    ///
    /// Returns error if persistence fails
    #[allow(clippy::unused_async)]
    pub async fn load(_path: &Path, dimensions: usize, config: HNSWConfig) -> Result<Self> {
        // TODO: Implement persistence
        // Would deserialize namespaces and metadata from disk
        warn!("Persistence not yet implemented");
        Ok(Self::new(dimensions, config))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_insert_and_search() {
        let storage = HNSWVectorStorage::new(3, HNSWConfig::default());

        // Insert some vectors
        let vectors = vec![
            VectorEntry::new("vec1".to_string(), vec![1.0, 0.0, 0.0])
                .with_scope(StateScope::Global),
            VectorEntry::new("vec2".to_string(), vec![0.0, 1.0, 0.0])
                .with_scope(StateScope::Global),
            VectorEntry::new("vec3".to_string(), vec![0.0, 0.0, 1.0])
                .with_scope(StateScope::Global),
        ];

        let ids = storage.insert(vectors).await.unwrap();
        assert_eq!(ids.len(), 3);

        // Search for similar vectors
        let query = VectorQuery::new(vec![0.9, 0.1, 0.0], 2);
        let results = storage.search(&query).await.unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "vec1");
    }

    #[tokio::test]
    async fn test_namespace_isolation() {
        let storage = HNSWVectorStorage::new(3, HNSWConfig::default());

        // Insert vectors in different namespaces
        let tenant1_vectors = vec![VectorEntry::new("t1_vec1".to_string(), vec![1.0, 0.0, 0.0])
            .with_scope(StateScope::Custom("tenant:tenant-1".to_string()))];

        let tenant2_vectors = vec![VectorEntry::new("t2_vec1".to_string(), vec![0.0, 1.0, 0.0])
            .with_scope(StateScope::Custom("tenant:tenant-2".to_string()))];

        storage.insert(tenant1_vectors).await.unwrap();
        storage.insert(tenant2_vectors).await.unwrap();

        // Search in tenant1 namespace - should only find tenant1 vectors
        let query = VectorQuery::new(vec![1.0, 0.0, 0.0], 10);
        let results = storage
            .search_scoped(&query, &StateScope::Custom("tenant:tenant-1".to_string()))
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "t1_vec1");

        // Search in tenant2 namespace - should only find tenant2 vectors
        let results = storage
            .search_scoped(&query, &StateScope::Custom("tenant:tenant-2".to_string()))
            .await
            .unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "t2_vec1");
    }

    #[tokio::test]
    async fn test_metadata_filtering() {
        let storage = HNSWVectorStorage::new(3, HNSWConfig::default());

        // Insert vectors with metadata
        let vectors = vec![
            VectorEntry::new("vec1".to_string(), vec![1.0, 0.0, 0.0])
                .with_scope(StateScope::Global)
                .with_metadata(HashMap::from([(
                    "type".to_string(),
                    serde_json::Value::String("document".to_string()),
                )])),
            VectorEntry::new("vec2".to_string(), vec![0.9, 0.1, 0.0])
                .with_scope(StateScope::Global)
                .with_metadata(HashMap::from([(
                    "type".to_string(),
                    serde_json::Value::String("image".to_string()),
                )])),
        ];

        storage.insert(vectors).await.unwrap();

        // Search with metadata filter
        let query = VectorQuery::new(vec![1.0, 0.0, 0.0], 10).with_filter(HashMap::from([(
            "type".to_string(),
            serde_json::Value::String("document".to_string()),
        )]));

        let results = storage.search(&query).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "vec1");
    }
}
