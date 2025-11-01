//! Prelude for common imports
//!
//! Usage:
//! ```rust
//! use llmspell_memory::prelude::*;
//! ```

pub use crate::episodic::InMemoryEpisodicMemory;
pub use crate::error::{MemoryError, Result};
pub use crate::traits::{
    ConsolidationDecision, Entity, EpisodicMemory, MemoryManager, ProceduralMemory, Relationship,
    SemanticMemory,
};
pub use crate::types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};
