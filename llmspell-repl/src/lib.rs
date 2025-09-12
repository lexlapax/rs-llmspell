//! `LLMSpell` REPL - Core REPL business logic
//!
//! This crate provides the core REPL functionality including:
//! - Command parsing and handling
//! - Kernel communication
//! - Session state management
//! - Debug command interfaces
//!
//! The CLI layer provides only terminal I/O - all business logic lives here.

pub mod client; // Legacy client connection handling (to be removed)
pub mod session; // Core REPL session management

#[cfg(test)]
mod tests;

// Re-export main types
pub use session::{KernelConnection, ReplConfig, ReplResponse, ReplSession, WorkloadType};

// Re-export types from llmspell-kernel for convenience
pub use llmspell_kernel::{ConnectionInfo, LLMSpellKernel};
