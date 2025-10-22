//! Prelude for common imports
//!
//! Usage:
//! ```rust
//! use llmspell_memory::prelude::*;
//! ```

pub use crate::error::{MemoryError, Result};
pub use crate::traits::{EpisodicMemory, MemoryManager, SemanticMemory};
pub use crate::types::{ConsolidationMode, ConsolidationResult, EpisodicEntry};
