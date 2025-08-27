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
    /// Mock storage for testing
    Mock,
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
            num_threads: None, // Use system default
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
