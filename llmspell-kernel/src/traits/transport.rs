//! Transport trait for generic message passing
//!
//! This trait abstracts the transport layer (`ZeroMQ`, TCP, IPC, WebSocket, etc.)
//! and knows NOTHING about specific protocols (Jupyter, LSP, DAP, etc.).
//! It provides a clean interface for multipart message passing over various channels.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Generic transport configuration
///
/// This configuration is protocol-agnostic and can be used for any
/// transport type (TCP, IPC, WebSocket, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    /// Transport type (e.g., "tcp", "ipc", "inproc", "websocket")
    pub transport_type: String,

    /// Base address (e.g., "127.0.0.1" for TCP, "/tmp/kernel" for IPC)
    pub base_address: String,

    /// Channel to port/endpoint mapping
    pub channels: HashMap<String, ChannelConfig>,

    /// Optional authentication key for secure connections
    pub auth_key: Option<String>,
}

/// Configuration for a single channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Port number for TCP, or endpoint suffix for IPC
    pub endpoint: String,

    /// Socket pattern (e.g., "router", "pub", "rep", "req", "dealer")
    pub pattern: String,

    /// Optional channel-specific settings
    pub options: HashMap<String, String>,
}

/// Generic transport for sending/receiving multipart messages
///
/// Transport layer knows NOTHING about protocols - it just moves bytes.
/// This allows us to support multiple protocols (Jupyter, LSP, DAP, WebSocket)
/// with the same transport infrastructure.
#[async_trait]
pub trait Transport: Send + Sync {
    /// Bind to specified addresses from configuration (server mode)
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;

    /// Connect to specified addresses from configuration (client mode)
    async fn connect(&mut self, config: &TransportConfig) -> Result<()>;

    /// Receive multipart message from a channel
    /// Returns None if no message available (non-blocking)
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>>;

    /// Send multipart message to a channel
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()>;

    /// Handle heartbeat if the transport requires it
    /// Returns true if heartbeat was handled, false if no heartbeat received
    async fn heartbeat(&self) -> Result<bool>;

    /// Check if a channel exists and is ready
    fn has_channel(&self, channel: &str) -> bool;

    /// Get list of available channels
    fn channels(&self) -> Vec<String>;

    /// Shutdown the transport gracefully
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Clone the transport for multi-threaded usage
    /// Some transports may need special handling for thread safety
    fn box_clone(&self) -> Box<dyn Transport>;
}

/// Boxed transport for dynamic dispatch
pub type BoxedTransport = Box<dyn Transport>;

/// Create a transport based on the configuration type
///
/// # Errors
///
/// Returns an error if the transport type is unknown or not compiled in
pub fn create_transport(transport_type: &str) -> Result<BoxedTransport> {
    match transport_type {
        "zeromq" | "zmq" => {
            #[cfg(feature = "zeromq")]
            {
                use crate::transport::zeromq::ZmqTransport;
                Ok(Box::new(ZmqTransport::new()?))
            }
            #[cfg(not(feature = "zeromq"))]
            {
                Err(anyhow::anyhow!("ZeroMQ support not compiled in"))
            }
        }
        "websocket" | "ws" => Err(anyhow::anyhow!("WebSocket support not yet implemented")),
        _ => Err(anyhow::anyhow!(
            "Unknown transport type: {}",
            transport_type
        )),
    }
}
