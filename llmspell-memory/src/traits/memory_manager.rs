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
/// ```rust,ignore
/// use llmspell_memory::prelude::*;
///
/// let memory = DefaultMemoryManager::new_in_memory().await?;
/// memory.episodic().add(entry).await?;
/// memory.consolidate("session-1", ConsolidationMode::Immediate).await?;
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

    /// Shutdown and cleanup resources
    ///
    /// Gracefully shuts down all memory subsystems, flushing any
    /// pending writes and closing connections.
    async fn shutdown(&self) -> Result<()>;
}
