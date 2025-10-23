//! Memory manager implementations
//!
//! Coordinates episodic, semantic, and procedural memory subsystems.
//! Provides unified API for memory operations and orchestrates consolidation.

use async_trait::async_trait;
use std::sync::Arc;

use crate::consolidation::NoopConsolidationEngine;
use crate::episodic::InMemoryEpisodicMemory;
use crate::error::Result;
use crate::procedural::NoopProceduralMemory;
use crate::semantic::GraphSemanticMemory;
use crate::traits::{EpisodicMemory, MemoryManager, ProceduralMemory, SemanticMemory};
use crate::types::{ConsolidationMode, ConsolidationResult};

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
/// └── Consolidation: NoopConsolidationEngine (stub for Phase 13.3.2)
/// ```
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::prelude::*;
///
/// // Create manager with in-memory backends (testing)
/// let manager = DefaultMemoryManager::new_in_memory().await?;
///
/// // Access subsystems
/// manager.episodic().add(entry).await?;
/// let results = manager.episodic().search("query", 5).await?;
///
/// // Consolidation (Phase 13.3.2)
/// manager.consolidate("session-1", ConsolidationMode::Immediate).await?;
/// ```
pub struct DefaultMemoryManager {
    episodic: Arc<dyn EpisodicMemory>,
    semantic: Arc<dyn SemanticMemory>,
    procedural: Arc<dyn ProceduralMemory>,
    #[allow(dead_code)] // Will be used in Task 13.3.2
    consolidation: Arc<NoopConsolidationEngine>,
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
    /// ```rust,ignore
    /// let episodic = Arc::new(InMemoryEpisodicMemory::new());
    /// let semantic = Arc::new(GraphSemanticMemory::new(graph_backend));
    /// let procedural = Arc::new(NoopProceduralMemory);
    ///
    /// let manager = DefaultMemoryManager::new(episodic, semantic, procedural);
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
            consolidation: Arc::new(NoopConsolidationEngine),
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
    /// ```rust,ignore
    /// let manager = DefaultMemoryManager::new_in_memory().await?;
    /// ```
    pub async fn new_in_memory() -> Result<Self> {
        let episodic: Arc<dyn EpisodicMemory> = Arc::new(InMemoryEpisodicMemory::new());
        let semantic: Arc<dyn SemanticMemory> = Arc::new(GraphSemanticMemory::new_temp().await?);
        let procedural: Arc<dyn ProceduralMemory> = Arc::new(NoopProceduralMemory);

        Ok(Self::new(episodic, semantic, procedural))
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
        // Stub implementation - full consolidation in Task 13.3.2
        // For now, just return empty result
        let _ = (session_id, mode);
        Ok(ConsolidationResult::empty())
    }

    async fn shutdown(&self) -> Result<()> {
        // Graceful shutdown - could flush pending writes, close connections
        // For now, no-op as in-memory backends don't need cleanup
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
