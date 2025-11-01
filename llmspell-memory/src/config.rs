//! ABOUTME: Memory system configuration with backend selection
//!
//! Provides configuration types for selecting and tuning memory backends,
//! enabling flexible deployment from testing to production.

use llmspell_storage::HNSWConfig;
use std::sync::Arc;

use crate::embeddings::EmbeddingService;

/// Episodic memory backend type
///
/// Determines which storage backend to use for episodic memory operations.
///
/// # Performance Characteristics
///
/// - **`InMemory`**: O(n) search, good for <1K entries, testing only
/// - **HNSW**: O(log n) search, production-ready for 10K+ entries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpisodicBackendType {
    /// Simple `HashMap` (for testing, <1K entries)
    InMemory,

    /// HNSW vector index (for production, 10K+ entries)
    HNSW,
}

impl Default for EpisodicBackendType {
    fn default() -> Self {
        Self::HNSW // HNSW is now the production default!
    }
}

/// Memory system configuration
///
/// Controls backend selection and tuning parameters for the memory subsystem.
///
/// # Example
///
/// ```rust
/// use llmspell_memory::config::{MemoryConfig, EpisodicBackendType};
///
/// // Testing configuration (no embeddings required)
/// let test_config = MemoryConfig::for_testing();
///
/// // Production configuration with custom HNSW tuning
/// # use llmspell_memory::embeddings::EmbeddingService;
/// # use llmspell_core::traits::embedding::EmbeddingProvider;
/// # use std::sync::Arc;
/// # use async_trait::async_trait;
/// # struct MockProvider;
/// # #[async_trait]
/// # impl EmbeddingProvider for MockProvider {
/// #     fn name(&self) -> &str { "mock" }
/// #     async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, llmspell_core::LLMSpellError> { Ok(vec![]) }
/// #     fn embedding_dimensions(&self) -> usize { 384 }
/// #     fn embedding_model(&self) -> Option<&str> { None }
/// # }
/// # let provider: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider);
/// let service = Arc::new(EmbeddingService::new(provider));
/// let prod_config = MemoryConfig::for_production(service);
/// ```
#[derive(Clone)]
pub struct MemoryConfig {
    /// Episodic backend selection
    pub episodic_backend: EpisodicBackendType,

    /// HNSW configuration (used if backend = HNSW)
    pub hnsw_config: HNSWConfig,

    /// Embedding service (required for HNSW)
    pub embedding_service: Option<Arc<EmbeddingService>>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            episodic_backend: EpisodicBackendType::HNSW, // Production default
            hnsw_config: HNSWConfig::default(),
            embedding_service: None,
        }
    }
}

impl MemoryConfig {
    /// Testing configuration (`InMemory`, no embeddings required)
    ///
    /// Suitable for unit tests and development where real embeddings are not needed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_memory::config::MemoryConfig;
    ///
    /// let config = MemoryConfig::for_testing();
    /// assert_eq!(config.episodic_backend, llmspell_memory::config::EpisodicBackendType::InMemory);
    /// ```
    #[must_use]
    pub fn for_testing() -> Self {
        Self {
            episodic_backend: EpisodicBackendType::InMemory,
            hnsw_config: HNSWConfig::default(),
            embedding_service: None,
        }
    }

    /// Production configuration (HNSW, requires embedding service)
    ///
    /// Suitable for production deployments with real embeddings and vector search.
    ///
    /// # Arguments
    ///
    /// * `embedding_service` - Service for generating embeddings
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::embeddings::EmbeddingService;
    /// # use llmspell_core::traits::embedding::EmbeddingProvider;
    /// # use std::sync::Arc;
    /// # use async_trait::async_trait;
    /// # struct MockProvider;
    /// # #[async_trait]
    /// # impl EmbeddingProvider for MockProvider {
    /// #     fn name(&self) -> &str { "mock" }
    /// #     async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, llmspell_core::LLMSpellError> { Ok(vec![]) }
    /// #     fn embedding_dimensions(&self) -> usize { 384 }
    /// #     fn embedding_model(&self) -> Option<&str> { None }
    /// # }
    /// use llmspell_memory::config::MemoryConfig;
    ///
    /// # let provider: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider);
    /// let service = Arc::new(EmbeddingService::new(provider));
    /// let config = MemoryConfig::for_production(service);
    /// ```
    #[must_use]
    pub fn for_production(embedding_service: Arc<EmbeddingService>) -> Self {
        Self {
            episodic_backend: EpisodicBackendType::HNSW,
            hnsw_config: HNSWConfig::default(),
            embedding_service: Some(embedding_service),
        }
    }

    /// Custom HNSW tuning (for Task 13.14.3 parameter optimization)
    ///
    /// Allows fine-tuning HNSW parameters for specific workload requirements.
    ///
    /// # Arguments
    ///
    /// * `config` - HNSW configuration (`m`, `ef_construct`, `ef_search`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_memory::config::MemoryConfig;
    /// use llmspell_storage::HNSWConfig;
    ///
    /// let mut hnsw_config = HNSWConfig::default();
    /// hnsw_config.m = 32;                 // Higher connectivity
    /// hnsw_config.ef_construction = 400;  // Better index quality
    /// hnsw_config.ef_search = 100;        // Higher recall
    /// let config = MemoryConfig::for_testing()
    ///     .with_hnsw_config(hnsw_config);
    /// ```
    #[must_use]
    pub const fn with_hnsw_config(mut self, config: HNSWConfig) -> Self {
        self.hnsw_config = config;
        self
    }

    /// Override backend type (advanced use)
    ///
    /// Allows explicit backend selection, useful for A/B testing or gradual migration.
    ///
    /// # Arguments
    ///
    /// * `backend` - Backend type to use
    #[must_use]
    pub const fn with_backend(mut self, backend: EpisodicBackendType) -> Self {
        self.episodic_backend = backend;
        self
    }
}

impl std::fmt::Debug for MemoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryConfig")
            .field("episodic_backend", &self.episodic_backend)
            .field("hnsw_config", &self.hnsw_config)
            .field(
                "embedding_service",
                &self
                    .embedding_service
                    .as_ref()
                    .map(|s| s.provider_name().to_string()),
            )
            .finish()
    }
}
