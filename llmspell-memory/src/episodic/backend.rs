//! ABOUTME: Episodic memory backend abstraction with enum dispatch
//!
//! Provides unified interface over multiple episodic storage backends,
//! allowing runtime selection between `InMemory` (testing), `Sqlite`, and `PostgreSQL` (production).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::sync::Arc;
use tracing::info;

use crate::config::{EpisodicBackendType, MemoryConfig};
#[cfg(feature = "postgres")]
use crate::episodic::PostgreSQLEpisodicMemory;
use crate::episodic::{InMemoryEpisodicMemory, SqliteEpisodicMemory};
use crate::error::{MemoryError, Result};
use crate::traits::EpisodicMemory;
use crate::types::EpisodicEntry;

/// Episodic memory backend (enum dispatch pattern)
///
/// Abstracts over different episodic storage implementations, allowing
/// runtime selection based on configuration. All methods dispatch to the
/// underlying backend implementation.
///
/// # Example
///
/// ```rust
/// use llmspell_memory::episodic::EpisodicBackend;
/// use llmspell_memory::config::MemoryConfig;
/// use llmspell_memory::traits::EpisodicMemory;
///
/// # async fn example() -> llmspell_memory::Result<()> {
/// // Create from configuration
/// let config = MemoryConfig::for_testing();
/// let backend = EpisodicBackend::from_config(&config)?;
///
/// // Use like any EpisodicMemory
/// # use llmspell_memory::types::EpisodicEntry;
/// let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Hello".into());
/// backend.add(entry).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub enum EpisodicBackend {
    /// In-memory `HashMap` backend (testing, <1K entries)
    InMemory(Arc<InMemoryEpisodicMemory>),

    /// `SQLite` with vectorlite HNSW backend (production, persistent local storage, 10K+ entries)
    Sqlite(Arc<SqliteEpisodicMemory>),

    /// `PostgreSQL` with `pgvector` backend (production, multi-tenant, `RLS`-enabled)
    #[cfg(feature = "postgres")]
    PostgreSQL(Arc<PostgreSQLEpisodicMemory>),
}

impl EpisodicBackend {
    /// Create backend from configuration
    ///
    /// Factory method that constructs the appropriate backend based on
    /// the configuration's `episodic_backend` field.
    ///
    /// # Arguments
    ///
    /// * `config` - Memory configuration specifying backend type and parameters
    ///
    /// # Returns
    ///
    /// The configured episodic backend, ready for use
    ///
    /// # Errors
    ///
    /// Returns error if backend initialization fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use llmspell_memory::episodic::EpisodicBackend;
    /// use llmspell_memory::config::MemoryConfig;
    ///
    /// # fn example() -> llmspell_memory::Result<()> {
    /// // Testing backend (no embeddings required)
    /// let test_config = MemoryConfig::for_testing();
    /// let test_backend = EpisodicBackend::from_config(&test_config)?;
    ///
    /// // Production backend (requires embedding service)
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
    /// let prod_backend = EpisodicBackend::from_config(&prod_config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_config(config: &MemoryConfig) -> Result<Self> {
        match config.episodic_backend {
            EpisodicBackendType::InMemory => Ok(Self::create_inmemory_backend(config)),
            EpisodicBackendType::Sqlite => Self::create_sqlite_backend(config),
            #[cfg(feature = "postgres")]
            EpisodicBackendType::PostgreSQL => Self::create_postgresql_backend(config),
        }
    }

    /// Create `InMemory` backend from configuration
    fn create_inmemory_backend(config: &MemoryConfig) -> Self {
        info!("Creating InMemory episodic backend (testing mode)");

        // If embedding service provided, use it with InMemory
        let backend = config.embedding_service.as_ref().map_or_else(
            || {
                info!("InMemory backend using test embeddings (cosine similarity only)");
                InMemoryEpisodicMemory::new()
            },
            |service| {
                info!(
                    "InMemory backend using embedding service: {}",
                    service.provider_name()
                );
                InMemoryEpisodicMemory::new_with_embeddings(Arc::clone(service))
            },
        );

        Self::InMemory(Arc::new(backend))
    }

    /// Create `SQLite` backend from configuration
    fn create_sqlite_backend(config: &MemoryConfig) -> Result<Self> {
        info!("Creating SQLite episodic backend (production mode, persistent local storage)");

        let service = config.embedding_service.as_ref().ok_or_else(|| {
            MemoryError::InvalidInput(
                "SQLite backend requires embedding service (use MemoryConfig::for_production)"
                    .to_string(),
            )
        })?;

        let sqlite_backend = config.sqlite_backend.as_ref().ok_or_else(|| {
            MemoryError::InvalidInput(
                "SQLite backend requires sqlite_backend (provide SqliteBackend instance)"
                    .to_string(),
            )
        })?;

        info!(
            "SQLite backend using embedding service: {}, dimensions: {}",
            service.provider_name(),
            service.dimensions()
        );

        // Create SqliteEpisodicMemory with async initialization
        // Note: This is a blocking call in from_config, but SqliteEpisodicMemory::new is async
        // We use block_on here since from_config is synchronous
        let sqlite_memory = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                SqliteEpisodicMemory::new(Arc::clone(sqlite_backend), Arc::clone(service)).await
            })
        })?;

        Ok(Self::Sqlite(Arc::new(sqlite_memory)))
    }

    /// Create `PostgreSQL` backend from configuration
    #[cfg(feature = "postgres")]
    fn create_postgresql_backend(config: &MemoryConfig) -> Result<Self> {
        info!("Creating PostgreSQL episodic backend (production mode, RLS-enabled)");

        let service = config.embedding_service.as_ref().ok_or_else(|| {
            MemoryError::InvalidInput(
                "PostgreSQL backend requires embedding service (use MemoryConfig::for_postgresql)"
                    .to_string(),
            )
        })?;

        let backend = config.postgres_backend.as_ref().ok_or_else(|| {
            MemoryError::InvalidInput(
                "PostgreSQL backend requires postgres_backend (use MemoryConfig::for_postgresql)"
                    .to_string(),
            )
        })?;

        info!(
            "PostgreSQL backend using embedding service: {}, dimensions: {}",
            service.provider_name(),
            service.dimensions()
        );

        let pg_memory = PostgreSQLEpisodicMemory::new(Arc::clone(backend), Arc::clone(service))?;

        Ok(Self::PostgreSQL(Arc::new(pg_memory)))
    }

    /// Get backend type name for logging/debugging
    #[must_use]
    pub const fn backend_name(&self) -> &'static str {
        match self {
            Self::InMemory(_) => "InMemory",
            Self::Sqlite(_) => "Sqlite",
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(_) => "PostgreSQL",
        }
    }
}

#[async_trait]
impl EpisodicMemory for EpisodicBackend {
    async fn add(&self, entry: EpisodicEntry) -> Result<String> {
        match self {
            Self::InMemory(backend) => backend.add(entry).await,
            Self::Sqlite(backend) => backend.add(entry).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.add(entry).await,
        }
    }

    async fn get(&self, id: &str) -> Result<EpisodicEntry> {
        match self {
            Self::InMemory(backend) => backend.get(id).await,
            Self::Sqlite(backend) => backend.get(id).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.get(id).await,
        }
    }

    async fn search(&self, query: &str, top_k: usize) -> Result<Vec<EpisodicEntry>> {
        match self {
            Self::InMemory(backend) => backend.search(query, top_k).await,
            Self::Sqlite(backend) => backend.search(query, top_k).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.search(query, top_k).await,
        }
    }

    async fn list_unprocessed(&self, session_id: &str) -> Result<Vec<EpisodicEntry>> {
        match self {
            Self::InMemory(backend) => backend.list_unprocessed(session_id).await,
            Self::Sqlite(backend) => backend.list_unprocessed(session_id).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.list_unprocessed(session_id).await,
        }
    }

    async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>> {
        match self {
            Self::InMemory(backend) => backend.get_session(session_id).await,
            Self::Sqlite(backend) => backend.get_session(session_id).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.get_session(session_id).await,
        }
    }

    async fn mark_processed(&self, entry_ids: &[String]) -> Result<()> {
        match self {
            Self::InMemory(backend) => backend.mark_processed(entry_ids).await,
            Self::Sqlite(backend) => backend.mark_processed(entry_ids).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.mark_processed(entry_ids).await,
        }
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize> {
        match self {
            Self::InMemory(backend) => backend.delete_before(timestamp).await,
            Self::Sqlite(backend) => backend.delete_before(timestamp).await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.delete_before(timestamp).await,
        }
    }

    async fn list_sessions_with_unprocessed(&self) -> Result<Vec<String>> {
        match self {
            Self::InMemory(backend) => backend.list_sessions_with_unprocessed().await,
            Self::Sqlite(backend) => backend.list_sessions_with_unprocessed().await,
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(backend) => backend.list_sessions_with_unprocessed().await,
        }
    }
}

impl std::fmt::Debug for EpisodicBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InMemory(_) => f.debug_tuple("InMemory").finish(),
            Self::Sqlite(_) => f.debug_tuple("Sqlite").finish(),
            #[cfg(feature = "postgres")]
            Self::PostgreSQL(_) => f.debug_tuple("PostgreSQL").finish(),
        }
    }
}
