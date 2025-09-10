//! ABOUTME: RAG (Retrieval-Augmented Generation) configuration
//! ABOUTME: Handles vector storage, embedding, and chunking configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Comprehensive RAG configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct RAGConfig {
    /// Enable RAG functionality
    #[serde(default)] // false by default
    pub enabled: bool,
    /// Vector storage configuration
    pub vector_storage: VectorStorageConfig,
    /// Embedding provider configuration
    pub embedding: EmbeddingConfig,
    /// Document chunking configuration
    pub chunking: ChunkingConfig,
    /// Multi-tenant support
    #[serde(default)] // false by default
    pub multi_tenant: bool,
    /// Cache configuration
    pub cache: RAGCacheConfig,
    /// Named RAG profiles for simplified command-line usage
    #[serde(default)]
    pub profiles: std::collections::HashMap<String, RAGProfile>,
}

/// RAG profile for simplified command-line usage
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RAGProfile {
    /// Enable RAG functionality for this profile
    pub enabled: bool,
    /// Vector storage backend (optional override)
    pub backend: Option<VectorBackend>,
    /// Vector dimensions (optional override)
    pub dimensions: Option<usize>,
    /// Custom configuration file (optional override)
    pub config_file: Option<PathBuf>,
    /// Description of this profile
    pub description: Option<String>,
}

impl Default for RAGProfile {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: None,
            dimensions: None,
            config_file: None,
            description: None,
        }
    }
}

impl RAGProfile {
    /// Apply this profile's overrides to a base RAG config
    pub fn apply_to_config(&self, config: &mut RAGConfig) {
        config.enabled = self.enabled;

        if let Some(backend) = &self.backend {
            config.vector_storage.backend = backend.clone();
        }

        if let Some(dimensions) = self.dimensions {
            config.vector_storage.dimensions = dimensions;
        }
    }
}

/// Vector storage backend configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct VectorStorageConfig {
    /// Vector dimensions (384, 768, 1536, etc.)
    pub dimensions: usize,
    /// Storage backend type
    pub backend: VectorBackend,
    /// Persistence directory for storage
    pub persistence_path: Option<PathBuf>,
    /// HNSW-specific configuration
    pub hnsw: HNSWConfig,
    /// Memory limits
    pub max_memory_mb: Option<usize>,
}

impl Default for VectorStorageConfig {
    fn default() -> Self {
        Self {
            dimensions: 384, // OpenAI text-embedding-3-small default
            backend: VectorBackend::HNSW,
            persistence_path: None,
            hnsw: HNSWConfig::default(),
            max_memory_mb: Some(500), // 500MB limit
        }
    }
}

/// Vector storage backend types
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum VectorBackend {
    /// HNSW (Hierarchical Navigable Small World) index
    HNSW,
}

/// HNSW index configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct HNSWConfig {
    /// Number of bi-directional links for each node (typical: 16)
    pub m: usize,
    /// Size of dynamic candidate list (typical: 200)
    pub ef_construction: usize,
    /// Size of search candidate list (typical: 50-100)
    pub ef_search: usize,
    /// Maximum number of elements in index
    pub max_elements: usize,
    /// Random seed for reproducibility
    pub seed: Option<u64>,
    /// Distance metric for similarity
    pub metric: DistanceMetric,
    /// Allow replacing deleted elements
    pub allow_replace_deleted: bool,
    /// Number of threads for parallel operations
    pub num_threads: Option<usize>,
    /// Number of hierarchical layers in the graph (auto-calculated if None)
    /// Formula: min(16, max(1, ln(max_elements)))
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
            max_elements: 1_000_000, // 1M vectors
            seed: None,
            metric: DistanceMetric::Cosine,
            allow_replace_deleted: true,
            num_threads: None,              // Use system default
            nb_layers: None,                // Auto-calculate based on max_elements
            parallel_batch_size: Some(128), // Default batch size for parallel ops
            enable_mmap: false,             // Disabled by default
            mmap_sync_interval: Some(60),   // Sync every minute if enabled
        }
    }
}

impl HNSWConfig {
    /// Validates the configuration parameters
    pub fn validate(&self) -> Result<(), String> {
        // M should be between 2 and 100
        if self.m < 2 || self.m > 100 {
            return Err(format!("m must be between 2 and 100, got {}", self.m));
        }

        // ef_construction should be at least m
        if self.ef_construction < self.m {
            return Err(format!(
                "ef_construction ({}) should be at least m ({})",
                self.ef_construction, self.m
            ));
        }

        // ef_search should be at least k (but we don't know k here, so check minimum)
        if self.ef_search < 1 {
            return Err("ef_search must be at least 1".to_string());
        }

        // max_elements should be reasonable
        if self.max_elements == 0 {
            return Err("max_elements must be greater than 0".to_string());
        }

        // nb_layers validation (if specified)
        if let Some(layers) = self.nb_layers {
            if layers == 0 || layers > 64 {
                return Err(format!(
                    "nb_layers must be between 1 and 64, got {}",
                    layers
                ));
            }
        }

        // parallel_batch_size validation
        if let Some(batch_size) = self.parallel_batch_size {
            if batch_size == 0 {
                return Err("parallel_batch_size must be greater than 0".to_string());
            }
        }

        Ok(())
    }

    /// Configuration for small datasets (<10K vectors)
    pub fn small_dataset() -> Self {
        Self {
            m: 12,
            ef_construction: 100,
            ef_search: 50,
            max_elements: 10_000,
            parallel_batch_size: Some(32),
            ..Default::default()
        }
    }

    /// Configuration for medium datasets (10K-100K vectors)
    pub fn medium_dataset() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 100,
            max_elements: 100_000,
            parallel_batch_size: Some(64),
            ..Default::default()
        }
    }

    /// Configuration for large datasets (100K-1M vectors)
    pub fn large_dataset() -> Self {
        Self {
            m: 32,
            ef_construction: 400,
            ef_search: 200,
            max_elements: 1_000_000,
            parallel_batch_size: Some(128),
            num_threads: Some(4),
            ..Default::default()
        }
    }

    /// Configuration optimized for speed over accuracy
    pub fn speed_optimized() -> Self {
        Self {
            m: 8,
            ef_construction: 50,
            ef_search: 25,
            parallel_batch_size: Some(256),
            ..Default::default()
        }
    }

    /// Configuration optimized for accuracy over speed
    pub fn accuracy_optimized() -> Self {
        Self {
            m: 48,
            ef_construction: 500,
            ef_search: 300,
            parallel_batch_size: Some(32),
            ..Default::default()
        }
    }

    /// Configuration for real-time applications (low latency)
    pub fn real_time() -> Self {
        Self {
            m: 12,
            ef_construction: 100,
            ef_search: 40,
            parallel_batch_size: Some(64),
            num_threads: Some(2),
            ..Default::default()
        }
    }
}

/// Distance metrics for vector similarity
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum DistanceMetric {
    /// Cosine similarity (most common for embeddings)
    Cosine,
    /// Euclidean distance
    Euclidean,
    /// Inner product (dot product)
    InnerProduct,
}

/// Embedding provider configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct EmbeddingConfig {
    /// Default provider to use (openai, local, etc.)
    pub default_provider: String,
    /// Enable embedding caching
    pub cache_enabled: bool,
    /// Cache size in number of embeddings
    pub cache_size: usize,
    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,
    /// Batch size for embedding generation
    pub batch_size: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for failed requests
    pub max_retries: u32,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            default_provider: "openai".to_string(),
            cache_enabled: true,
            cache_size: 10000,       // 10k embeddings
            cache_ttl_seconds: 3600, // 1 hour
            batch_size: 32,
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

/// Document chunking configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ChunkingConfig {
    /// Default chunking strategy
    pub strategy: ChunkingStrategy,
    /// Default chunk size in tokens
    pub chunk_size: usize,
    /// Overlap between chunks in tokens
    pub overlap: usize,
    /// Maximum chunk size (hard limit)
    pub max_chunk_size: usize,
    /// Minimum chunk size (for quality)
    pub min_chunk_size: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            strategy: ChunkingStrategy::SlidingWindow,
            chunk_size: 512, // 512 tokens
            overlap: 64,     // 64 token overlap
            max_chunk_size: 2048,
            min_chunk_size: 100,
        }
    }
}

/// Document chunking strategies
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkingStrategy {
    /// Simple sliding window approach
    SlidingWindow,
    /// Semantic chunking (future implementation)
    Semantic,
    /// Sentence-based chunking
    Sentence,
}

/// RAG caching configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct RAGCacheConfig {
    /// Enable search result caching
    pub search_cache_enabled: bool,
    /// Search cache size (number of queries)
    pub search_cache_size: usize,
    /// Search cache TTL in seconds
    pub search_cache_ttl_seconds: u64,
    /// Enable document cache
    pub document_cache_enabled: bool,
    /// Document cache size in MB
    pub document_cache_size_mb: usize,
}

impl Default for RAGCacheConfig {
    fn default() -> Self {
        Self {
            search_cache_enabled: true,
            search_cache_size: 1000,       // 1k search results
            search_cache_ttl_seconds: 300, // 5 minutes
            document_cache_enabled: true,
            document_cache_size_mb: 100, // 100MB
        }
    }
}

/// Builder for RAGConfig
#[derive(Debug, Clone)]
pub struct RAGConfigBuilder {
    config: RAGConfig,
}

impl RAGConfigBuilder {
    /// Create a new RAG config builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: RAGConfig::default(),
        }
    }

    /// Enable or disable RAG functionality
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set vector dimensions
    #[must_use]
    pub const fn dimensions(mut self, dimensions: usize) -> Self {
        self.config.vector_storage.dimensions = dimensions;
        self
    }

    /// Set vector storage backend
    #[must_use]
    pub fn backend(mut self, backend: VectorBackend) -> Self {
        self.config.vector_storage.backend = backend;
        self
    }

    /// Set persistence path
    #[must_use]
    pub fn persistence_path(mut self, path: Option<PathBuf>) -> Self {
        self.config.vector_storage.persistence_path = path;
        self
    }

    /// Set default embedding provider
    #[must_use]
    pub fn embedding_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.embedding.default_provider = provider.into();
        self
    }

    /// Enable multi-tenant support
    #[must_use]
    pub const fn multi_tenant(mut self, enabled: bool) -> Self {
        self.config.multi_tenant = enabled;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> RAGConfig {
        self.config
    }
}

impl Default for RAGConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RAGConfig {
    /// Create a new builder for RAGConfig
    #[must_use]
    pub fn builder() -> RAGConfigBuilder {
        RAGConfigBuilder::new()
    }

    /// Check if RAG is properly configured
    #[must_use]
    pub const fn is_configured(&self) -> bool {
        self.enabled && self.vector_storage.dimensions > 0
    }

    /// Get effective cache size in bytes
    #[must_use]
    pub const fn effective_cache_size_bytes(&self) -> usize {
        // Rough estimate: embedding cache + document cache + search cache
        let embedding_cache_bytes = self.embedding.cache_size * self.vector_storage.dimensions * 4; // 4 bytes per f32
        let document_cache_bytes = self.cache.document_cache_size_mb * 1_024 * 1_024;
        let search_cache_bytes = self.cache.search_cache_size * 1_024; // 1KB per search result estimate

        embedding_cache_bytes + document_cache_bytes + search_cache_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_config_default() {
        let config = RAGConfig::default();
        assert!(!config.enabled); // Disabled by default
        assert_eq!(config.vector_storage.dimensions, 384);
        assert_eq!(config.embedding.default_provider, "openai");
        assert!(!config.multi_tenant);
    }

    #[test]
    fn test_rag_config_builder() {
        let config = RAGConfig::builder()
            .enabled(true)
            .dimensions(768)
            .embedding_provider("local")
            .multi_tenant(true)
            .build();

        assert!(config.enabled);
        assert_eq!(config.vector_storage.dimensions, 768);
        assert_eq!(config.embedding.default_provider, "local");
        assert!(config.multi_tenant);
    }

    #[test]
    fn test_hnsw_config_defaults() {
        let config = HNSWConfig::default();
        assert_eq!(config.m, 16);
        assert_eq!(config.ef_construction, 200);
        assert_eq!(config.ef_search, 50);
        assert_eq!(config.max_elements, 1_000_000);
        assert!(matches!(config.metric, DistanceMetric::Cosine));
    }

    #[test]
    fn test_is_configured() {
        let mut config = RAGConfig::default();
        assert!(!config.is_configured()); // Disabled by default

        config.enabled = true;
        assert!(config.is_configured()); // Enabled with valid dimensions

        config.vector_storage.dimensions = 0;
        assert!(!config.is_configured()); // Invalid dimensions
    }

    #[test]
    fn test_chunking_config() {
        let config = ChunkingConfig::default();
        assert_eq!(config.chunk_size, 512);
        assert_eq!(config.overlap, 64);
        assert!(matches!(config.strategy, ChunkingStrategy::SlidingWindow));
    }

    #[test]
    fn test_serialization() {
        let config = RAGConfig::builder().enabled(true).dimensions(1536).build();

        // Test JSON serialization
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RAGConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(
            config.vector_storage.dimensions,
            deserialized.vector_storage.dimensions
        );

        // Test TOML serialization
        let toml = toml::to_string(&config).unwrap();
        let deserialized_toml: RAGConfig = toml::from_str(&toml).unwrap();

        assert_eq!(config.enabled, deserialized_toml.enabled);
        assert_eq!(
            config.vector_storage.dimensions,
            deserialized_toml.vector_storage.dimensions
        );
    }

    #[test]
    fn test_effective_cache_size() {
        let config = RAGConfig::builder().dimensions(384).build();

        let cache_size = config.effective_cache_size_bytes();
        assert!(cache_size > 0);

        // Should include embedding cache, document cache, and search cache
        let expected_embedding_cache = 10000 * 384 * 4; // 10k embeddings * 384 dims * 4 bytes
        let expected_document_cache = 100 * 1024 * 1024; // 100MB
        let expected_search_cache = 1000 * 1024; // 1k searches * 1KB each

        assert_eq!(
            cache_size,
            expected_embedding_cache + expected_document_cache + expected_search_cache
        );
    }
}
