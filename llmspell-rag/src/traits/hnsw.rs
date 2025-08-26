//! HNSW-specific storage trait and configuration

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::storage::VectorStorage;

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
    pub last_optimized: Option<std::time::SystemTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

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
