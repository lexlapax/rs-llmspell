//! Core traits for memory system (placeholder - will be implemented in Task 13.1.2)

// Re-export submodules when they're created
pub use memory_manager::*;
pub use episodic::*;
pub use semantic::*;

pub mod memory_manager;
pub mod episodic;
pub mod semantic;
pub mod procedural;
pub mod consolidation;
