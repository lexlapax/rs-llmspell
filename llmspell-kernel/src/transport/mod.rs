//! Transport layer for multi-protocol messaging
//!
//! This module provides clean transport implementations for various protocols.
//! It handles message transport for:
//! - Jupyter (5 channels: shell, iopub, stdin, control, heartbeat)
//! - LSP (Language Server Protocol) via JSON-RPC
//! - DAP (Debug Adapter Protocol) via JSON-RPC
//! - WebSocket for real-time communication
//!
//! All transports implement the Transport trait and know nothing about
//! the specific protocol semantics - they just move bytes.

#[cfg(feature = "zeromq")]
pub mod zeromq;

pub mod jupyter;

// Re-export commonly used types
#[cfg(feature = "zeromq")]
pub use zeromq::ZmqTransport;

pub use jupyter::JupyterTransport;
