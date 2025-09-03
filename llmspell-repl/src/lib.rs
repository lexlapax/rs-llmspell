//! LLMSpell REPL - Client interface for kernel connections
//!
//! This crate provides client-side REPL functionality that connects to 
//! the LLMSpell kernel (now in llmspell-kernel crate).
//!
//! The kernel implementation has been moved to llmspell-kernel for better
//! architectural separation: kernel=execution, repl=client interface.

pub mod client; // Client connection handling

// Re-export types from llmspell-kernel for convenience  
pub use llmspell_kernel::{ConnectionInfo, LLMSpellKernel, KernelConfig};
