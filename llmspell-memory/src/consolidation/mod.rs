//! Memory consolidation logic (episodic → semantic)
//!
//! Converts episodic entries to semantic knowledge through entity extraction
//! and relationship mapping.
//!
//! # Consolidation Strategies
//!
//! - **Manual**: Explicit trigger for testing/development
//! - **Immediate**: Consolidate on every episodic add (Phase 13.5)
//! - **Background**: Daemon-based consolidation (Phase 13.6)
//! - **LLM-Driven**: Advanced extraction with ADD/UPDATE/DELETE/NOOP decisions (Phase 13.5)
//!
//! # Example
//!
//! ```rust,ignore
//! use llmspell_memory::consolidation::ManualConsolidationEngine;
//!
//! let engine = ManualConsolidationEngine::new(extractor, graph);
//! let result = engine.consolidate(&["session-123"], &mut entries).await?;
//! println!("Processed {} entries, added {} entities",
//!          result.entries_processed, result.entities_added);
//! ```

use async_trait::async_trait;

use crate::error::Result;
use crate::types::{ConsolidationResult, EpisodicEntry};

pub mod context_assembly;
pub mod daemon;
pub mod llm_engine;
pub mod manual;
pub mod noop;
pub mod prompt_schema;
pub mod prompts;
pub mod validator;

pub use context_assembly::ContextAssembler;
pub use daemon::{ConsolidationDaemon, DaemonConfig, DaemonMetrics};
pub use llm_engine::{LLMConsolidationConfig, LLMConsolidationEngine};
pub use manual::ManualConsolidationEngine;
pub use noop::NoopConsolidationEngine;
pub use prompt_schema::{
    ConsolidationResponse, DecisionPayload, EntityPayload, OutputFormat, RelationshipPayload,
};
pub use prompts::{
    ConsolidationPromptBuilder, ConsolidationPromptConfig, PromptVersion, TokenBudget,
    parse_llm_response,
};
pub use validator::DecisionValidator;

/// Trait for consolidation engines that convert episodic entries to semantic knowledge
///
/// Implementations extract entities and relationships from episodic content
/// and update the knowledge graph accordingly.
#[async_trait]
pub trait ConsolidationEngine: Send + Sync {
    /// Consolidate episodic entries into semantic memory
    ///
    /// # Arguments
    ///
    /// * `session_ids` - Filter to only consolidate entries from these sessions (empty = all)
    /// * `entries` - Mutable slice of entries to consolidate (will be marked as processed)
    ///
    /// # Returns
    ///
    /// Consolidation metrics (entries processed, entities added/updated/deleted)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let result = engine.consolidate(&["session-123"], &mut entries).await?;
    /// assert!(result.entries_processed > 0);
    /// ```
    async fn consolidate(
        &self,
        session_ids: &[&str],
        entries: &mut [EpisodicEntry],
    ) -> Result<ConsolidationResult>;

    /// Check if engine is ready for consolidation
    ///
    /// Returns false if dependencies (e.g., LLM service) are unavailable.
    fn is_ready(&self) -> bool {
        true
    }
}
