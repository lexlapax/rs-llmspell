//! Jupyter protocol implementation
//!
//! This module implements the Jupyter messaging protocol for `LLMSpell` kernel.
//! Supports standard Jupyter messages plus custom daemon management extensions.

pub mod connection;
pub mod protocol;
pub mod wire;

pub use connection::ConnectionInfo;
pub use protocol::{JupyterMessage, JupyterProtocol};
pub use wire::WireProtocol;
