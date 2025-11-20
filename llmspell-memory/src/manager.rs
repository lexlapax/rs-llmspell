//! Memory manager implementations
//!
//! Coordinates episodic, semantic, and procedural memory subsystems.
//! Provides unified API for memory operations and orchestrates consolidation.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::consolidation::{ConsolidationEngine, NoopConsolidationEngine};
use crate::error::Result;
use crate::procedural::InMemoryPatternTracker;
use crate::semantic::GraphSemanticMemory;
use crate::traits::{EpisodicMemory, MemoryManager, ProceduralMemory, SemanticMemory};
use crate::types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};

#[cfg(test)]
use crate::episodic::InMemoryEpisodicMemory;

/// Default memory manager implementation
///
/// Coordinates episodic, semantic, and procedural memory with hot-swappable backends.
///
/// # Architecture
///
/// ```text
/// DefaultMemoryManager
/// ├── Episodic: InMemoryEpisodicMemory (or SqliteEpisodicMemory with vectorlite-rs)
/// ├── Semantic: GraphSemanticMemory (wraps SqliteGraphStorage or PostgresGraphStorage)
/// ├── Procedural: NoopProceduralMemory (placeholder)
/// └── Consolidation: ConsolidationEngine (NoopConsolidationEngine by default)
/// ```
///
/// # Example
///
/// ```rust,no_run
/// use llmspell_memory::prelude::*;
/// use llmspell_memory::DefaultMemoryManager;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Create manager with in-memory backends (testing)
///     let manager = DefaultMemoryManager::new_in_memory().await?;
///
///     // Access subsystems
///     let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Hello".into());
///     manager.episodic().add(entry).await?;
///     let results = manager.episodic().search("query", 5).await?;
///
///     // Consolidation (Phase 13.3.2)
///     manager.consolidate("session-1", ConsolidationMode::Immediate, None).await?;
///     Ok(())
/// }
/// ```
pub struct DefaultMemoryManager {
    episodic: Arc<dyn EpisodicMemory>,
    semantic: Arc<dyn SemanticMemory>,
    procedural: Arc<dyn ProceduralMemory>,
    consolidation: Arc<dyn ConsolidationEngine>,
}

impl DefaultMemoryManager {
    /// Create new memory manager with custom backends
    ///
    /// Allows full control over storage backends for each memory type.
    ///
    /// # Arguments
    ///
    /// * `episodic` - Episodic memory implementation
    /// * `semantic` - Semantic memory implementation
    /// * `procedural` - Procedural memory implementation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use llmspell_memory::{DefaultMemoryManager, InMemoryEpisodicMemory};
    /// use llmspell_memory::semantic::GraphSemanticMemory;
    /// use llmspell_memory::procedural::NoopProceduralMemory;
    /// use llmspell_storage::backends::sqlite::SqliteBackend;
    ///
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     let episodic = Arc::new(InMemoryEpisodicMemory::new());
    ///     let sqlite_backend = Arc::new(SqliteBackend::new(llmspell_storage::backends::sqlite::SqliteConfig::in_memory()).await.map_err(|e| llmspell_memory::MemoryError::Storage(e.to_string()))?);
    ///     let semantic = Arc::new(GraphSemanticMemory::new_with_sqlite(sqlite_backend));
    ///     let procedural = Arc::new(NoopProceduralMemory);
    ///
    ///     let manager = DefaultMemoryManager::new(episodic, semantic, procedural);
    ///     Ok(())
    /// }
    /// ```
    pub fn new(
        episodic: Arc<dyn EpisodicMemory>,
        semantic: Arc<dyn SemanticMemory>,
        procedural: Arc<dyn ProceduralMemory>,
    ) -> Self {
        Self {
            episodic,
            semantic,
            procedural,
            consolidation: Arc::new(NoopConsolidationEngine::new()),
        }
    }

    /// Create new memory manager with custom consolidation engine
    ///
    /// Allows overriding the default no-op consolidation with a real implementation.
    ///
    /// # Arguments
    ///
    /// * `episodic` - Episodic memory implementation
    /// * `semantic` - Semantic memory implementation
    /// * `procedural` - Procedural memory implementation
    /// * `consolidation` - Consolidation engine implementation
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::sync::Arc;
    /// use llmspell_memory::{DefaultMemoryManager, InMemoryEpisodicMemory};
    /// use llmspell_memory::semantic::GraphSemanticMemory;
    /// use llmspell_memory::procedural::NoopProceduralMemory;
    /// use llmspell_memory::consolidation::ManualConsolidationEngine;
    /// use llmspell_graph::extraction::RegexExtractor;
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteGraphStorage};
    ///
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     let episodic = Arc::new(InMemoryEpisodicMemory::new());
    ///     let sqlite_backend = Arc::new(SqliteBackend::new(llmspell_storage::backends::sqlite::SqliteConfig::in_memory()).await.map_err(|e| llmspell_memory::MemoryError::Storage(e.to_string()))?);
    ///     let semantic = Arc::new(GraphSemanticMemory::new_with_sqlite(Arc::clone(&sqlite_backend)));
    ///     let procedural = Arc::new(NoopProceduralMemory);
    ///
    ///     let extractor = Arc::new(RegexExtractor::new());
    ///     let graph = Arc::new(SqliteGraphStorage::new(sqlite_backend));
    ///     let engine = Arc::new(ManualConsolidationEngine::new(extractor, graph));
    ///     let manager = DefaultMemoryManager::with_consolidation(
    ///         episodic, semantic, procedural, engine
    ///     );
    ///     Ok(())
    /// }
    /// ```
    pub fn with_consolidation(
        episodic: Arc<dyn EpisodicMemory>,
        semantic: Arc<dyn SemanticMemory>,
        procedural: Arc<dyn ProceduralMemory>,
        consolidation: Arc<dyn ConsolidationEngine>,
    ) -> Self {
        Self {
            episodic,
            semantic,
            procedural,
            consolidation,
        }
    }

    /// Create memory manager with configuration (NEW: preferred method)
    ///
    /// Constructs memory manager using `MemoryConfig` to select episodic backend
    /// (`InMemory` or `HNSW`) and configure parameters. This is the recommended way
    /// to create a memory manager for both testing and production.
    ///
    /// # Arguments
    ///
    /// * `config` - Memory configuration specifying backend and parameters
    ///
    /// # Returns
    ///
    /// Configured memory manager with selected backends
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Backend initialization fails
    /// - HNSW selected but no embedding service provided
    /// - Semantic memory initialization fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use llmspell_memory::{DefaultMemoryManager, MemoryConfig};
    ///
    /// # async fn example() -> llmspell_memory::Result<()> {
    /// // Testing configuration (InMemory backend, no embeddings)
    /// let test_config = MemoryConfig::for_testing();
    /// let test_manager = DefaultMemoryManager::with_config(&test_config)?;
    ///
    /// // Production configuration (HNSW backend with embeddings)
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
    /// let prod_manager = DefaultMemoryManager::with_config(&prod_config)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_config(config: &crate::config::MemoryConfig) -> Result<Self> {
        use crate::episodic::EpisodicBackend;

        info!(
            "Initializing DefaultMemoryManager with config: episodic={:?}, semantic={:?}",
            config.episodic_backend, config.semantic_backend
        );

        // Create episodic backend from configuration
        let episodic = EpisodicBackend::from_config(config)?;
        info!("Episodic backend created: {}", episodic.backend_name());

        // Create other subsystems
        let semantic = Self::create_semantic_memory(config)?;
        let procedural = Self::create_procedural_memory();

        info!("DefaultMemoryManager initialized successfully with config");
        Ok(Self::new(Arc::new(episodic), semantic, procedural))
    }

    /// Create memory manager with in-memory backends (for testing/development)
    ///
    /// Uses `InMemory` episodic backend (simple `HashMap` with O(n) search).
    /// For production use with HNSW and real embeddings, use `with_config()` or `new_in_memory_with_embeddings()`.
    ///
    /// All memory subsystems use in-memory storage:
    /// - Episodic: `InMemory` backend (test embeddings, cosine similarity)
    /// - Semantic: `SQLite` in-memory backend (`:memory:`)
    /// - Procedural: `InMemoryPatternTracker`
    ///
    /// # Errors
    ///
    /// Returns error if backends fail to initialize
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use llmspell_memory::DefaultMemoryManager;
    ///
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     let manager = DefaultMemoryManager::new_in_memory().await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_in_memory() -> Result<Self> {
        info!("Initializing DefaultMemoryManager with InMemory backends (testing mode)");

        // Create in-memory backends directly
        let episodic = crate::episodic::InMemoryEpisodicMemory::new();

        // Create in-memory SQLite backend for semantic memory
        let sqlite_backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .map_err(|e| crate::error::MemoryError::Storage(e.to_string()))?,
        );

        // Run migrations for semantic backend
        sqlite_backend
            .run_migrations()
            .await
            .map_err(|e| crate::error::MemoryError::Storage(e.to_string()))?;

        let semantic = Arc::new(crate::semantic::GraphSemanticMemory::new_with_sqlite(
            Arc::clone(&sqlite_backend),
        ));

        let procedural = Arc::new(crate::procedural::InMemoryPatternTracker::new());

        Ok(Self::new(Arc::new(episodic), semantic, procedural))
    }

    /// Create memory manager with in-memory backends and real embeddings (production)
    ///
    /// Uses `SQLite` episodic backend (O(log n) vector search via vectorlite-rs) with real embeddings.
    /// This is the recommended production configuration.
    ///
    /// All memory subsystems use in-memory storage:
    /// - Episodic: `SQLite` vector index (real embeddings via `EmbeddingService`)
    /// - Semantic: Requires explicit `SQLite` backend configuration
    /// - Procedural: No-op placeholder
    ///
    /// # Arguments
    ///
    /// * `embedding_service` - Embedding service for generating vectors
    ///
    /// # Errors
    ///
    /// Returns error if `SQLite` backend is not configured
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use llmspell_memory::{DefaultMemoryManager, embeddings::EmbeddingService};
    /// use llmspell_core::traits::embedding::EmbeddingProvider;
    /// use async_trait::async_trait;
    /// use std::sync::Arc;
    ///
    /// # struct MyProvider;
    /// # #[async_trait]
    /// # impl EmbeddingProvider for MyProvider {
    /// #     fn name(&self) -> &str { "test" }
    /// #     async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, llmspell_core::LLMSpellError> {
    /// #         Ok(vec![])
    /// #     }
    /// #     fn embedding_dimensions(&self) -> usize { 384 }
    /// #     fn embedding_model(&self) -> Option<&str> { None }
    /// # }
    /// #
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     // Create embedding provider (e.g., OpenAI, Ollama, etc.)
    ///     let provider: Arc<dyn EmbeddingProvider> = Arc::new(MyProvider);
    ///     let service = Arc::new(EmbeddingService::new(provider));
    ///
    ///     // Create manager with HNSW backend and real embeddings
    ///     let manager = DefaultMemoryManager::new_in_memory_with_embeddings(service).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new_in_memory_with_embeddings(
        embedding_service: Arc<crate::embeddings::EmbeddingService>,
    ) -> Result<Self> {
        info!(
            "Initializing DefaultMemoryManager with InMemory backend and embedding service: {}",
            embedding_service.provider_name()
        );

        // Create in-memory SQLite backend for semantic memory
        let sqlite_backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .map_err(|e| crate::error::MemoryError::Storage(e.to_string()))?,
        );

        // Use InMemory backend for episodic, SQLite for semantic (both in-memory)
        let config = crate::config::MemoryConfig::default()
            .with_backend(crate::config::EpisodicBackendType::InMemory)
            .with_embedding_service(embedding_service)
            .with_semantic_sqlite(Arc::clone(&sqlite_backend));
        Self::with_config(&config)
    }

    /// Helper: Create semantic memory from configuration
    fn create_semantic_memory(
        config: &crate::config::MemoryConfig,
    ) -> Result<Arc<dyn SemanticMemory>> {
        use crate::config::SemanticBackendType;

        debug!(
            "Creating GraphSemanticMemory with backend: {:?}",
            config.semantic_backend
        );

        let semantic = match config.semantic_backend {
            SemanticBackendType::Sqlite => {
                debug!("Initializing SQLite semantic memory");
                let sqlite_backend = config.semantic_sqlite_backend.as_ref().ok_or_else(|| {
                    error!("SQLite semantic backend requested but not configured");
                    crate::error::MemoryError::InvalidInput(
                        "SQLite semantic backend not configured".to_string(),
                    )
                })?;
                GraphSemanticMemory::new_with_sqlite(Arc::clone(sqlite_backend))
            }
            #[cfg(feature = "postgres")]
            SemanticBackendType::PostgreSQL => {
                debug!("Initializing PostgreSQL semantic memory");
                let postgres_backend =
                    config.semantic_postgres_backend.as_ref().ok_or_else(|| {
                        error!("PostgreSQL semantic backend requested but not configured");
                        crate::error::MemoryError::InvalidInput(
                            "PostgreSQL semantic backend not configured".to_string(),
                        )
                    })?;
                GraphSemanticMemory::new_with_postgres(Arc::clone(postgres_backend))
            }
        };

        Ok(Arc::new(semantic))
    }

    /// Helper: Create no-op procedural memory
    fn create_procedural_memory() -> Arc<dyn ProceduralMemory> {
        debug!("Creating InMemoryPatternTracker for procedural memory");
        Arc::new(InMemoryPatternTracker::new())
    }

    // ========== Phase 13.6.4: Kernel Integration API Helpers ==========

    /// Check if real consolidation is enabled (not using no-op engine)
    ///
    /// Returns `true` if a real consolidation engine (manual or LLM-driven) is configured,
    /// `false` if using `NoopConsolidationEngine`.
    ///
    /// Used by kernel integration to determine if consolidation daemon should be started.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::DefaultMemoryManager;
    /// # async fn example() {
    /// let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    /// if manager.has_consolidation() {
    ///     println!("Real consolidation engine enabled");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn has_consolidation(&self) -> bool {
        !self.consolidation.is_noop()
    }

    /// Check if episodic memory is present
    ///
    /// Always returns `true` in current design (episodic memory always present).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::DefaultMemoryManager;
    /// # async fn example() {
    /// let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    /// assert!(manager.has_episodic());
    /// # }
    /// ```
    #[must_use]
    pub const fn has_episodic(&self) -> bool {
        true
    }

    /// Check if semantic memory is present
    ///
    /// Always returns `true` in current design (semantic memory always present).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::DefaultMemoryManager;
    /// # async fn example() {
    /// let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    /// assert!(manager.has_semantic());
    /// # }
    /// ```
    #[must_use]
    pub const fn has_semantic(&self) -> bool {
        true
    }

    /// Get consolidation engine as Arc for daemon construction
    ///
    /// Returns `None` if using `NoopConsolidationEngine` (no real consolidation).
    /// Returns `Some(Arc<dyn ConsolidationEngine>)` if real engine configured.
    ///
    /// Used by kernel integration to construct `ConsolidationDaemon`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::DefaultMemoryManager;
    /// # async fn example() {
    /// let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    /// if let Some(engine) = manager.consolidation_engine_arc() {
    ///     println!("Consolidation engine available for daemon");
    /// }
    /// # }
    /// ```
    #[must_use]
    pub fn consolidation_engine_arc(&self) -> Option<Arc<dyn ConsolidationEngine>> {
        if self.has_consolidation() {
            Some(self.consolidation.clone())
        } else {
            None
        }
    }

    /// Get episodic memory as Arc for daemon construction
    ///
    /// Returns owned `Arc` to episodic memory implementation.
    /// Used by kernel integration to construct `ConsolidationDaemon`.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use llmspell_memory::DefaultMemoryManager;
    /// # async fn example() {
    /// let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
    /// let episodic = manager.episodic_arc();
    /// println!("Episodic memory Arc obtained");
    /// # }
    /// ```
    #[must_use]
    pub fn episodic_arc(&self) -> Arc<dyn EpisodicMemory> {
        self.episodic.clone()
    }

    /// Reorder entries to prioritize specified entry IDs
    ///
    /// Partitions entries into priority (matching IDs) and rest,
    /// returning priority entries first to enable consolidation feedback.
    ///
    /// # Arguments
    ///
    /// * `entries` - All unprocessed entries
    /// * `priority_ids` - Entry IDs to prioritize
    ///
    /// # Returns
    ///
    /// Reordered vector with priority entries first, then remaining entries
    fn reorder_by_priority(
        entries: Vec<EpisodicEntry>,
        priority_ids: &[String],
    ) -> Vec<EpisodicEntry> {
        use std::collections::HashSet;

        let priority_set: HashSet<&str> = priority_ids.iter().map(String::as_str).collect();

        // Partition: priority entries first, rest after
        let mut priority = Vec::new();
        let mut rest = Vec::new();

        for entry in entries {
            if priority_set.contains(entry.id.as_str()) {
                priority.push(entry);
            } else {
                rest.push(entry);
            }
        }

        debug!(
            "Reordered {} total entries: {} priority, {} rest",
            priority.len() + rest.len(),
            priority.len(),
            rest.len()
        );

        // Combine: priority first, then rest
        priority.extend(rest);
        priority
    }
}

#[async_trait]
impl MemoryManager for DefaultMemoryManager {
    fn episodic(&self) -> &dyn EpisodicMemory {
        self.episodic.as_ref()
    }

    fn semantic(&self) -> &dyn SemanticMemory {
        self.semantic.as_ref()
    }

    fn procedural(&self) -> &dyn ProceduralMemory {
        self.procedural.as_ref()
    }

    async fn consolidate(
        &self,
        session_id: &str,
        mode: ConsolidationMode,
        priority_entries: Option<&[String]>,
    ) -> Result<ConsolidationResult> {
        info!(
            "Triggering consolidation: session_id={}, mode={:?}, priority_count={}",
            session_id,
            mode,
            priority_entries.map_or(0, <[std::string::String]>::len)
        );

        // Get all entries for the session
        let entries = self.episodic.get_session(session_id).await.map_err(|e| {
            error!(
                "Failed to retrieve session entries for {}: {}",
                session_id, e
            );
            e
        })?;

        debug!(
            "Retrieved {} total entries for session {}",
            entries.len(),
            session_id
        );

        // Filter to only unprocessed entries
        let mut unprocessed: Vec<EpisodicEntry> =
            entries.into_iter().filter(|e| !e.processed).collect();

        if unprocessed.is_empty() {
            info!(
                "No unprocessed entries for session {}, skipping consolidation",
                session_id
            );
            return Ok(ConsolidationResult::empty());
        }

        // Reorder entries if priority list provided
        if let Some(priority_ids) = priority_entries {
            if !priority_ids.is_empty() {
                unprocessed = Self::reorder_by_priority(unprocessed, priority_ids);
                info!(
                    "Prioritized {} entries for consolidation based on retrieval frequency",
                    priority_ids.len()
                );
            }
        }

        debug!(
            "Found {} unprocessed entries to consolidate",
            unprocessed.len()
        );

        // Run consolidation based on mode
        let result = match mode {
            ConsolidationMode::Manual | ConsolidationMode::Immediate => {
                debug!("Running consolidation in {:?} mode", mode);
                self.consolidation
                    .consolidate(&[session_id], &mut unprocessed)
                    .await?
            }
            ConsolidationMode::Background => {
                // Background mode not yet implemented
                // For now, treat as manual trigger
                debug!("Background mode not yet implemented, treating as manual");
                self.consolidation
                    .consolidate(&[session_id], &mut unprocessed)
                    .await?
            }
        };

        debug!(
            "Consolidation complete: entities_added={}, entries_processed={}",
            result.entities_added, result.entries_processed
        );

        // Mark processed entries in episodic storage
        let processed_ids: Vec<String> = unprocessed
            .iter()
            .filter(|e| e.processed)
            .map(|e| e.id.clone())
            .collect();

        if !processed_ids.is_empty() {
            debug!("Marking {} entries as processed", processed_ids.len());
            self.episodic
                .mark_processed(&processed_ids)
                .await
                .map_err(|e| {
                    error!("Failed to mark entries as processed: {}", e);
                    e
                })?;
        }

        info!(
            "Consolidation succeeded: {} entities added, {} entries processed",
            result.entities_added, result.entries_processed
        );

        Ok(result)
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down DefaultMemoryManager");
        // Graceful shutdown - could flush pending writes, close connections
        // For now, no-op as in-memory backends don't need cleanup
        debug!("Shutdown complete (no cleanup needed for in-memory backends)");
        Ok(())
    }

    // ========== Phase 13.7.2: Daemon Integration Trait Methods ==========

    fn has_consolidation(&self) -> bool {
        !self.consolidation.is_noop()
    }

    fn episodic_arc(&self) -> Option<Arc<dyn EpisodicMemory>> {
        Some(self.episodic.clone())
    }

    fn consolidation_engine_arc(&self) -> Option<Arc<dyn ConsolidationEngine>> {
        if self.has_consolidation() {
            Some(self.consolidation.clone())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::procedural::NoopProceduralMemory;
    use crate::types::EpisodicEntry;
    use serde_json::json;

    #[tokio::test]
    async fn test_create_in_memory_manager() {
        let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

        // Verify all subsystems are accessible
        let _ = manager.episodic();
        let _ = manager.semantic();
        let _ = manager.procedural();
    }

    #[tokio::test]
    async fn test_episodic_memory_integration() {
        let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

        let entry = EpisodicEntry::new("test-session".into(), "user".into(), "Hello world".into());

        manager.episodic().add(entry).await.unwrap();

        let results = manager.episodic().search("Hello", 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello world");
    }

    #[tokio::test]
    async fn test_semantic_memory_integration() {
        let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

        let entity = llmspell_graph::Entity::new(
            "Rust".into(),
            "programming_language".into(),
            json!({"paradigm": "systems"}),
        );
        let id = entity.id.clone();

        manager.semantic().upsert_entity(entity).await.unwrap();

        let retrieved = manager.semantic().get_entity(&id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Rust");
    }

    #[tokio::test]
    async fn test_consolidation_stub() {
        let manager = DefaultMemoryManager::new_in_memory().await.unwrap();

        let result = manager
            .consolidate("test-session", ConsolidationMode::Immediate, None)
            .await
            .unwrap();

        // Stub returns empty result
        assert_eq!(result.entries_processed, 0);
        assert_eq!(result.entities_added, 0);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let manager = DefaultMemoryManager::new_in_memory().await.unwrap();
        manager.shutdown().await.unwrap();
    }

    #[tokio::test]
    async fn test_create_in_memory_manager_with_embeddings() {
        use crate::embeddings::EmbeddingService;
        use async_trait::async_trait;
        use llmspell_core::error::LLMSpellError;
        use llmspell_core::traits::embedding::EmbeddingProvider;

        // Mock embedding provider for testing
        struct TestEmbeddingProvider;

        #[async_trait]
        impl EmbeddingProvider for TestEmbeddingProvider {
            fn name(&self) -> &'static str {
                "test-provider"
            }

            async fn embed(
                &self,
                texts: &[String],
            ) -> std::result::Result<Vec<Vec<f32>>, LLMSpellError> {
                Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
            }

            fn embedding_dimensions(&self) -> usize {
                3
            }

            fn supports_dimension_reduction(&self) -> bool {
                false
            }

            fn set_embedding_dimensions(
                &mut self,
                _dims: usize,
            ) -> std::result::Result<(), LLMSpellError> {
                Err(LLMSpellError::Provider {
                    message: "Dimension configuration not supported".to_string(),
                    provider: Some(self.name().to_string()),
                    source: None,
                })
            }

            fn embedding_model(&self) -> Option<&str> {
                Some("test-model")
            }

            fn embedding_cost_per_token(&self) -> Option<f64> {
                None
            }
        }

        // Create embedding service with mock provider
        let provider = Arc::new(TestEmbeddingProvider);
        let service = Arc::new(EmbeddingService::new(provider));

        // Create manager with embeddings
        let manager = DefaultMemoryManager::new_in_memory_with_embeddings(service)
            .await
            .unwrap();

        // Verify all subsystems are accessible
        let _ = manager.episodic();
        let _ = manager.semantic();
        let _ = manager.procedural();

        // Test episodic memory with embedding service
        let entry = EpisodicEntry::new("test-session".into(), "user".into(), "Hello".into());
        manager.episodic().add(entry).await.unwrap();

        let results = manager.episodic().search("Hello", 5).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Hello");
    }

    // ========== Phase 13.6.4: API Helper Tests ==========

    #[tokio::test]
    async fn test_has_consolidation_with_noop() {
        let mgr = DefaultMemoryManager::new_in_memory().await.unwrap();
        assert!(!mgr.has_consolidation()); // Uses noop by default
        assert!(mgr.has_episodic());
        assert!(mgr.has_semantic());
        assert!(mgr.consolidation_engine_arc().is_none()); // Should return None for noop
    }

    #[tokio::test]
    async fn test_has_consolidation_with_real_engine() {
        use crate::consolidation::ManualConsolidationEngine;
        use llmspell_graph::extraction::RegexExtractor;
        use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteGraphStorage};

        let episodic = Arc::new(InMemoryEpisodicMemory::new());
        let sqlite_backend = Arc::new(
            SqliteBackend::new(llmspell_storage::backends::sqlite::SqliteConfig::in_memory())
                .await
                .unwrap(),
        );
        let semantic = Arc::new(GraphSemanticMemory::new_with_sqlite(Arc::clone(
            &sqlite_backend,
        )));
        let procedural = Arc::new(NoopProceduralMemory);

        let extractor = Arc::new(RegexExtractor::new());
        let graph = Arc::new(SqliteGraphStorage::new(sqlite_backend));
        let engine = Arc::new(ManualConsolidationEngine::new(extractor, graph));

        let mgr = DefaultMemoryManager::with_consolidation(episodic, semantic, procedural, engine);

        assert!(mgr.has_consolidation()); // Real engine should return true
        assert!(mgr.consolidation_engine_arc().is_some()); // Should return Some for real engine
    }

    #[tokio::test]
    async fn test_episodic_arc_returns_same_instance() {
        let mgr = DefaultMemoryManager::new_in_memory().await.unwrap();
        let arc1 = mgr.episodic_arc();
        let arc2 = mgr.episodic_arc();
        assert!(Arc::ptr_eq(&arc1, &arc2)); // Should be same Arc instance
    }
}
