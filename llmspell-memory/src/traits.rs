//! Core traits for memory system
//!
//! This module defines the trait hierarchy for the adaptive memory system:
//!
//! - **`MemoryManager`**: Unified interface to all memory subsystems
//! - **`EpisodicMemory`**: Vector-indexed interaction history
//! - **`SemanticMemory`**: Bi-temporal knowledge graph
//! - **`ProceduralMemory`**: Learned patterns (Phase 13.3 placeholder)
//!
//! Additionally, it exports supporting types:
//! - **Entity** & **Relationship**: Knowledge graph types
//! - **`ConsolidationDecision`**: LLM-driven knowledge extraction

// Re-export trait types
pub use consolidation::*;
pub use episodic::*;
pub use memory_manager::*;
pub use procedural::*;
pub use semantic::*;

pub mod consolidation;
pub mod episodic;
pub mod memory_manager;
pub mod procedural;
pub mod semantic;
