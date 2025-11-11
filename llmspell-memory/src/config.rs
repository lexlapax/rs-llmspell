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
/// - **HNSW**: O(log n) search, production-ready for 10K+ entries - **DEPRECATED: will be removed in Task 13c.2.8**
/// - **Sqlite**: O(log n) search with SQLite + HNSW, persistent local storage (NEW - replacement for HNSW)
/// - **`PostgreSQL`**: O(log n) search with `pgvector` HNSW, multi-tenant `RLS` support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EpisodicBackendType {
    /// Simple `HashMap` (for testing, <1K entries)
    InMemory,

    /// HNSW vector index (for production, 10K+ entries) - **DEPRECATED: will be removed in Task 13c.2.8**
    HNSW,

    /// SQLite with HNSW (for production, persistent local storage, 10K+ entries)
    #[default]
    Sqlite,

    /// `PostgreSQL` with `pgvector` (for production, multi-tenant, `RLS`-enabled)
    #[cfg(feature = "postgres")]
    PostgreSQL,
}

/// Semantic memory backend type
///
/// Determines which knowledge graph backend to use for semantic memory operations.
///
/// # Backend Characteristics
///
/// - **`SurrealDB`**: Default bi-temporal graph database with embedded storage
/// - **`PostgreSQL`**: Production-ready with `PostgreSQL` bi-temporal graph tables, multi-tenant `RLS`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SemanticBackendType {
    /// `SurrealDB` embedded graph database (default, for development and production)
    #[default]
    SurrealDB,

    /// `PostgreSQL` bi-temporal graph tables (for production, multi-tenant, `RLS`-enabled)
    #[cfg(feature = "postgres")]
    PostgreSQL,
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

    /// Semantic backend selection
    pub semantic_backend: SemanticBackendType,

    /// HNSW configuration (used if backend = HNSW)
    pub hnsw_config: HNSWConfig,

    /// Embedding service (required for `HNSW`, `Sqlite`, and `PostgreSQL`)
    pub embedding_service: Option<Arc<EmbeddingService>>,

    /// SQLite backend for episodic memory (used if `episodic_backend` = `Sqlite`)
    pub sqlite_backend: Option<Arc<llmspell_storage::backends::sqlite::SqliteBackend>>,

    /// `PostgreSQL` backend for episodic memory (used if `episodic_backend` = `PostgreSQL`)
    #[cfg(feature = "postgres")]
    pub postgres_backend: Option<Arc<llmspell_storage::PostgresBackend>>,

    /// `PostgreSQL` backend for semantic memory (used if `semantic_backend` = `PostgreSQL`)
    #[cfg(feature = "postgres")]
    pub semantic_postgres_backend: Option<Arc<llmspell_storage::PostgresBackend>>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            episodic_backend: EpisodicBackendType::Sqlite, // NEW production default
            semantic_backend: SemanticBackendType::SurrealDB,
            hnsw_config: HNSWConfig::default(),
            embedding_service: None,
            sqlite_backend: None,
            #[cfg(feature = "postgres")]
            postgres_backend: None,
            #[cfg(feature = "postgres")]
            semantic_postgres_backend: None,
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
            semantic_backend: SemanticBackendType::SurrealDB,
            hnsw_config: HNSWConfig::default(),
            embedding_service: None,
            sqlite_backend: None,
            #[cfg(feature = "postgres")]
            postgres_backend: None,
            #[cfg(feature = "postgres")]
            semantic_postgres_backend: None,
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
            episodic_backend: EpisodicBackendType::HNSW, // Still using HNSW by default until users migrate
            semantic_backend: SemanticBackendType::SurrealDB,
            hnsw_config: HNSWConfig::default(),
            embedding_service: Some(embedding_service),
            sqlite_backend: None,
            #[cfg(feature = "postgres")]
            postgres_backend: None,
            #[cfg(feature = "postgres")]
            semantic_postgres_backend: None,
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

    /// `PostgreSQL` configuration (production, multi-tenant, `RLS`-enabled)
    ///
    /// Suitable for production deployments with `PostgreSQL` backend, embedding service, and `RLS`.
    ///
    /// # Arguments
    ///
    /// * `postgres_backend` - `PostgreSQL` backend with connection pool
    /// * `embedding_service` - Service for generating embeddings
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use llmspell_memory::config::MemoryConfig;
    /// # use llmspell_memory::embeddings::EmbeddingService;
    /// # use llmspell_storage::{PostgresBackend, PostgresConfig};
    /// # use llmspell_core::traits::embedding::EmbeddingProvider;
    /// # use std::sync::Arc;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// # use async_trait::async_trait;
    /// # struct MockProvider;
    /// # #[async_trait]
    /// # impl EmbeddingProvider for MockProvider {
    /// #     fn name(&self) -> &str { "mock" }
    /// #     async fn embed(&self, _texts: &[String]) -> Result<Vec<Vec<f32>>, llmspell_core::LLMSpellError> { Ok(vec![]) }
    /// #     fn embedding_dimensions(&self) -> usize { 384 }
    /// #     fn embedding_model(&self) -> Option<&str> { None }
    /// # }
    /// let pg_config = PostgresConfig::new("postgresql://localhost/llmspell");
    /// let pg_backend = Arc::new(PostgresBackend::new(pg_config).await?);
    ///
    /// # let provider: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider);
    /// let service = Arc::new(EmbeddingService::new(provider));
    /// let config = MemoryConfig::for_postgresql(pg_backend, service);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "postgres")]
    #[must_use]
    pub fn for_postgresql(
        postgres_backend: Arc<llmspell_storage::PostgresBackend>,
        embedding_service: Arc<EmbeddingService>,
    ) -> Self {
        Self {
            episodic_backend: EpisodicBackendType::PostgreSQL,
            semantic_backend: SemanticBackendType::PostgreSQL,
            hnsw_config: HNSWConfig::default(),
            embedding_service: Some(embedding_service),
            postgres_backend: Some(postgres_backend.clone()),
            semantic_postgres_backend: Some(postgres_backend),
        }
    }

    /// Override semantic backend type
    ///
    /// Allows explicit semantic backend selection for testing or migration scenarios.
    ///
    /// # Arguments
    ///
    /// * `backend` - Semantic backend type to use
    #[must_use]
    pub const fn with_semantic_backend(mut self, backend: SemanticBackendType) -> Self {
        self.semantic_backend = backend;
        self
    }

    /// Configure semantic memory with `PostgreSQL` backend
    ///
    /// Sets semantic backend to `PostgreSQL` and configures the backend instance.
    ///
    /// # Arguments
    ///
    /// * `postgres_backend` - `PostgreSQL` backend instance
    #[cfg(feature = "postgres")]
    #[must_use]
    pub fn with_semantic_postgres(
        mut self,
        postgres_backend: Arc<llmspell_storage::PostgresBackend>,
    ) -> Self {
        self.semantic_backend = SemanticBackendType::PostgreSQL;
        self.semantic_postgres_backend = Some(postgres_backend);
        self
    }
}

impl std::fmt::Debug for MemoryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug_struct = f.debug_struct("MemoryConfig");
        debug_struct
            .field("episodic_backend", &self.episodic_backend)
            .field("semantic_backend", &self.semantic_backend)
            .field("hnsw_config", &self.hnsw_config)
            .field(
                "embedding_service",
                &self
                    .embedding_service
                    .as_ref()
                    .map(|s| s.provider_name().to_string()),
            );

        #[cfg(feature = "postgres")]
        {
            debug_struct.field("postgres_backend", &self.postgres_backend.is_some());
            debug_struct.field(
                "semantic_postgres_backend",
                &self.semantic_postgres_backend.is_some(),
            );
        }

        debug_struct.finish()
    }
}
