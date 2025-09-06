//! Trait-based architecture for protocol and transport abstraction
//!
//! This module defines the core traits that enable clean separation between:
//! - Transport layer (`ZeroMQ`, TCP, IPC, etc.)
//! - Protocol layer (Jupyter, LSP, DAP, MCP, etc.)
//! - Kernel orchestration layer
//!
//! The dependency flow is: Kernel → Protocol → Transport
//! Transport knows nothing about protocols, protocols know nothing about kernel.

pub mod message;
pub mod null;
pub mod output;
pub mod protocol;
pub mod transport;

pub use message::KernelMessage;
pub use null::{NullMessage, NullProtocol, NullTransport};
pub use output::OutputCapture;
pub use protocol::{
    ChannelPattern, ChannelTopology, ExecutionError, ExecutionFlow, ExecutionResult,
    ExpectedMessage, KernelStatus, OutputChunk, Protocol, ResponseCollector, ResponseFlow,
    StreamData, StreamType,
};
pub use transport::{Transport, TransportConfig};
