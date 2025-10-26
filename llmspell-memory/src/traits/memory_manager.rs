//! Memory manager trait
//!
//! The `MemoryManager` trait provides unified access to all memory subsystems:
//! - Episodic memory (vector-indexed interactions)
//! - Semantic memory (bi-temporal knowledge graph)
//! - Procedural memory (learned patterns)
//!
//! It also coordinates consolidation (episodic → semantic transformation).

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{ConsolidationMode, ConsolidationResult};

/// Memory manager coordinates all memory subsystems
///
/// # Example
///
/// ```rust,no_run
/// use llmspell_memory::prelude::*;
/// use llmspell_memory::DefaultMemoryManager;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let memory = DefaultMemoryManager::new_in_memory().await?;
///     let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Hello".into());
///     memory.episodic().add(entry).await?;
///     memory.consolidate("session-1", ConsolidationMode::Immediate).await?;
///     Ok(())
/// }
/// ```
#[async_trait]
pub trait MemoryManager: Send + Sync {
    /// Access episodic memory subsystem
    ///
    /// Returns a reference to the episodic memory implementation,
    /// which stores vector-indexed interaction history.
    fn episodic(&self) -> &dyn super::EpisodicMemory;

    /// Access semantic memory subsystem
    ///
    /// Returns a reference to the semantic memory implementation,
    /// which stores the bi-temporal knowledge graph.
    fn semantic(&self) -> &dyn super::SemanticMemory;

    /// Access procedural memory subsystem (placeholder for Phase 13.3)
    ///
    /// Returns a reference to the procedural memory implementation,
    /// which stores learned patterns and skills.
    fn procedural(&self) -> &dyn super::ProceduralMemory;

    /// Consolidate episodic memories into semantic knowledge
    ///
    /// Processes unprocessed episodic entries for the given session,
    /// extracting entities and relationships to update the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `session_id` - Session to consolidate (empty string = all sessions)
    /// * `mode` - Consolidation mode (Immediate, Background, or Manual)
    ///
    /// # Returns
    ///
    /// Statistics about the consolidation operation (entries processed,
    /// entities added/updated/deleted, duration).
    async fn consolidate(
        &self,
        session_id: &str,
        mode: ConsolidationMode,
    ) -> Result<ConsolidationResult>;

    // ========== Phase 13.7.1: Kernel Integration Helpers ==========

    /// Check if episodic memory is present
    ///
    /// Returns `true` if episodic memory subsystem is available.
    /// Always returns `true` in current design (episodic memory always present).
    fn has_episodic(&self) -> bool {
        true
    }

    /// Check if semantic memory is present
    ///
    /// Returns `true` if semantic memory subsystem is available.
    /// Always returns `true` in current design (semantic memory always present).
    fn has_semantic(&self) -> bool {
        true
    }

    /// Check if real consolidation is enabled (not using no-op engine)
    ///
    /// Returns `true` if a real consolidation engine (manual or LLM-driven) is configured,
    /// `false` if using `NoopConsolidationEngine`.
    ///
    /// Used by kernel integration to determine if consolidation daemon should be started.
    fn has_consolidation(&self) -> bool {
        false // Default: no consolidation (override in concrete implementations)
    }

    /// Shutdown and cleanup resources
    ///
    /// Gracefully shuts down all memory subsystems, flushing any
    /// pending writes and closing connections.
    async fn shutdown(&self) -> Result<()>;
}
