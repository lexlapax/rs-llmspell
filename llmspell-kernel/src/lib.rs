//! llmspell-kernel: Jupyter-compatible execution kernel for `LLMSpell`
//!
//! This crate provides the core execution engine that:
//! - Implements Jupyter Messaging Protocol (Task 9.8.5)
//! - Manages `ScriptRuntime` instances from llmspell-bridge
//! - Handles debug/DAP integration
//! - Supports multiple client connections via `ZeroMQ`
//!
//! ## Architecture
//!
//! This is a clean-start crate created in Task 9.8.3 to avoid Phase 9.5's
//! multi-protocol abstractions (`UnifiedProtocolEngine`, adapters, sidecars)
//! that are incompatible with Jupyter's single-protocol model.
//!
//! ## Trait-based Architecture (Task 9.8.5)
//!
//! The kernel uses a trait-based architecture for clean separation:
//! - Transport trait: Abstract message transport (`ZeroMQ`, TCP, IPC)
//! - Protocol trait: Abstract protocol handling (Jupyter, LSP, DAP)
//! - `KernelMessage` trait: Abstract message representation
//!
//! Dependency flow: Kernel → Protocol → Transport

// Trait-based architecture
pub mod traits;

// Core modules
pub mod client;
pub mod comm_handler;
pub mod connection;
pub mod discovery;
pub mod kernel;
pub mod protocol;
pub mod security;
pub mod session_persistence;

// Jupyter protocol implementation (Task 9.8.5)
pub mod jupyter;
pub mod transport;
// pub mod execution;  // Will be implemented later
// pub mod debug;      // Will be implemented later

// Re-export key types
pub use connection::ConnectionInfo;
pub use discovery::KernelDiscovery;
pub use kernel::{GenericKernel, KernelState};

// Re-export trait-based architecture
pub use traits::{KernelMessage, Protocol, Transport, TransportConfig};
pub use traits::{NullMessage, NullProtocol, NullTransport};

// Re-export implementations
pub use jupyter::JupyterProtocol;
pub use transport::ZmqTransport;

// Type alias for common Jupyter kernel configuration
pub type JupyterKernel = GenericKernel<ZmqTransport, JupyterProtocol>;

// Backwards compatibility alias (to be removed)
pub type LLMSpellKernel = JupyterKernel;
