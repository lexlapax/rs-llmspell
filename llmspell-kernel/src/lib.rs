//! llmspell-kernel: Jupyter-compatible execution kernel for LLMSpell
//!
//! This crate provides the core execution engine that:
//! - Implements Jupyter Messaging Protocol (Task 9.8.5)
//! - Manages ScriptRuntime instances from llmspell-bridge
//! - Handles debug/DAP integration
//! - Supports multiple client connections via ZeroMQ
//!
//! ## Architecture
//!
//! This is a clean-start crate created in Task 9.8.3 to avoid Phase 9.5's
//! multi-protocol abstractions (UnifiedProtocolEngine, adapters, sidecars)
//! that are incompatible with Jupyter's single-protocol model.

// Core modules moved from llmspell-repl
pub mod kernel;
pub mod connection;
pub mod discovery;
pub mod security;
pub mod client;
pub mod protocol;

// These modules will be populated in Task 9.8.5
// pub mod jupyter;
// pub mod transport;
// pub mod execution;
// pub mod debug;

// Re-export key types
pub use kernel::{LLMSpellKernel, KernelConfig, KernelState};
pub use connection::ConnectionInfo;
pub use discovery::KernelDiscovery;

// Re-export key types that will be added later
// pub use jupyter::ConnectionInfo;
// pub use transport::ZmqTransport;
