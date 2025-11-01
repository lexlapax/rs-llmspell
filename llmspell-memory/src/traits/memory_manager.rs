//! Memory manager trait
//!
//! The `MemoryManager` trait provides unified access to all memory subsystems:
//! - Episodic memory (vector-indexed interactions)
//! - Semantic memory (bi-temporal knowledge graph)
//! - Procedural memory (learned patterns)
//!
//! It also coordinates consolidation (episodic → semantic transformation).

use async_trait::async_trait;
use std::sync::Arc;

use crate::consolidation::ConsolidationEngine;
use crate::error::Result;
use crate::types::{ConsolidationMode, ConsolidationResult};
use crate::EpisodicMemory;

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
///     memory.consolidate("session-1", ConsolidationMode::Immediate, None).await?;
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
    /// * `priority_entries` - Optional list of entry IDs to prioritize (processed first)
    ///
    /// # Returns
    ///
    /// Statistics about the consolidation operation (entries processed,
    /// entities added/updated/deleted, duration).
    ///
    /// # Priority Processing
    ///
    /// If `priority_entries` is provided, those entries are consolidated first
    /// before processing remaining entries chronologically. This enables
    /// consolidation feedback where frequently-retrieved episodic entries
    /// are prioritized for semantic memory promotion.
    async fn consolidate(
        &self,
        session_id: &str,
        mode: ConsolidationMode,
        priority_entries: Option<&[String]>,
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

    /// Get Arc reference to episodic memory (Phase 13.7.2 - daemon integration)
    ///
    /// Returns an Arc-wrapped reference to the episodic memory component,
    /// required by `ConsolidationDaemon` for background processing.
    ///
    /// # Returns
    ///
    /// `None` if episodic memory not available (should not happen in current design).
    fn episodic_arc(&self) -> Option<Arc<dyn EpisodicMemory>> {
        None
    }

    /// Get Arc reference to consolidation engine (Phase 13.7.2 - daemon integration)
    ///
    /// Returns an Arc-wrapped reference to the consolidation engine,
    /// required by `ConsolidationDaemon` for background processing.
    ///
    /// # Returns
    ///
    /// `None` if consolidation engine not available or is no-op.
    fn consolidation_engine_arc(&self) -> Option<Arc<dyn ConsolidationEngine>> {
        None
    }

    /// Shutdown and cleanup resources
    ///
    /// Gracefully shuts down all memory subsystems, flushing any
    /// pending writes and closing connections.
    async fn shutdown(&self) -> Result<()>;
}
