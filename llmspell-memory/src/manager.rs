//! Memory manager implementations
//!
//! Coordinates episodic, semantic, and procedural memory subsystems.
//! Provides unified API for memory operations and orchestrates consolidation.

use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, error, info};

use crate::consolidation::{ConsolidationEngine, NoopConsolidationEngine};
use crate::episodic::InMemoryEpisodicMemory;
use crate::error::Result;
use crate::procedural::InMemoryPatternTracker;
use crate::semantic::GraphSemanticMemory;
use crate::traits::{EpisodicMemory, MemoryManager, ProceduralMemory, SemanticMemory};
use crate::types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};

/// Default memory manager implementation
///
/// Coordinates episodic, semantic, and procedural memory with hot-swappable backends.
///
/// # Architecture
///
/// ```text
/// DefaultMemoryManager
/// ├── Episodic: InMemoryEpisodicMemory (HNSW vector search)
/// ├── Semantic: GraphSemanticMemory (wraps SurrealDB)
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
///     manager.consolidate("session-1", ConsolidationMode::Immediate).await?;
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
    ///
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     let episodic = Arc::new(InMemoryEpisodicMemory::new());
    ///     let semantic = Arc::new(GraphSemanticMemory::new_temp().await?);
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
    /// use llmspell_graph::storage::surrealdb::SurrealDBBackend;
    /// use tempfile::TempDir;
    ///
    /// #[tokio::main]
    /// async fn main() -> llmspell_memory::Result<()> {
    ///     let temp = TempDir::new().unwrap();
    ///     let episodic = Arc::new(InMemoryEpisodicMemory::new());
    ///     let semantic = Arc::new(GraphSemanticMemory::new_temp().await?);
    ///     let procedural = Arc::new(NoopProceduralMemory);
    ///
    ///     let extractor = Arc::new(RegexExtractor::new());
    ///     let graph = Arc::new(SurrealDBBackend::new(temp.path().to_path_buf()).await.unwrap());
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

    /// Create memory manager with in-memory backends (for testing/development)
    ///
    /// All memory subsystems use in-memory storage:
    /// - Episodic: HNSW vector index
    /// - Semantic: Temporary `SurrealDB` instance
    /// - Procedural: No-op placeholder
    ///
    /// # Errors
    ///
    /// Returns error if temporary `SurrealDB` initialization fails
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
        info!("Initializing DefaultMemoryManager with in-memory backends");

        let episodic = Self::create_episodic_memory();
        let semantic = Self::create_semantic_memory().await?;
        let procedural = Self::create_procedural_memory();

        info!("DefaultMemoryManager initialized successfully");
        Ok(Self::new(episodic, semantic, procedural))
    }

    /// Helper: Create in-memory episodic memory
    fn create_episodic_memory() -> Arc<dyn EpisodicMemory> {
        debug!("Creating InMemoryEpisodicMemory");
        Arc::new(InMemoryEpisodicMemory::new())
    }

    /// Helper: Create temporary semantic memory with `SurrealDB`
    async fn create_semantic_memory() -> Result<Arc<dyn SemanticMemory>> {
        debug!("Creating temporary GraphSemanticMemory (SurrealDB)");
        let semantic = GraphSemanticMemory::new_temp().await.map_err(|e| {
            error!("Failed to initialize semantic memory: {}", e);
            e
        })?;
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
    ) -> Result<ConsolidationResult> {
        info!(
            "Triggering consolidation: session_id={}, mode={:?}",
            session_id, mode
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

        let entity = llmspell_graph::types::Entity::new(
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
            .consolidate("test-session", ConsolidationMode::Immediate)
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
        use llmspell_graph::storage::surrealdb::SurrealDBBackend;
        use tempfile::TempDir;

        let temp = TempDir::new().unwrap();
        let episodic = Arc::new(InMemoryEpisodicMemory::new());
        let semantic = Arc::new(GraphSemanticMemory::new_temp().await.unwrap());
        let procedural = Arc::new(NoopProceduralMemory);

        let extractor = Arc::new(RegexExtractor::new());
        let graph = Arc::new(
            SurrealDBBackend::new(temp.path().to_path_buf())
                .await
                .unwrap(),
        );
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
