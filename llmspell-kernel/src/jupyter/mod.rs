//! Jupyter protocol implementation
//!
//! This module implements the Jupyter messaging protocol for LLMSpell kernel.
//! Supports standard Jupyter messages plus custom daemon management extensions.

pub mod protocol;
pub mod connection;

pub use protocol::JupyterMessage;
pub use connection::ConnectionInfo;