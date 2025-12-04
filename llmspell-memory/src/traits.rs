//! Core traits for memory system
//!
//! This module defines the trait hierarchy for the adaptive memory system:
//!
//! - **`MemoryManager`**: Unified interface to all memory subsystems
//! - **`EpisodicMemory`**: Vector-indexed interaction history
//! - **`SemanticMemory`**: Bi-temporal knowledge graph
//! - **`ProceduralMemory`**: Learned patterns (re-exported from llmspell-core)
//!
//! Additionally, it exports supporting types:
//! - **Entity** & **Relationship**: Knowledge graph types
//! - **`ConsolidationDecision`**: LLM-driven knowledge extraction
//! - **Pattern**: Procedural memory pattern (re-exported from llmspell-core)

// Re-export trait types
pub use consolidation::*;
pub use episodic::*;
pub use memory_manager::*;
pub use semantic::*;

// Re-export ProceduralMemory and Pattern from llmspell-core
pub use llmspell_core::traits::storage::ProceduralMemory;
pub use llmspell_core::types::storage::Pattern;

pub mod consolidation;
pub mod episodic;
pub mod memory_manager;
pub mod semantic;
