//! Transport trait for generic message passing
//!
//! This trait abstracts the transport layer (`ZeroMQ`, TCP, IPC, etc.)
//! and knows NOTHING about specific protocols (Jupyter, LSP, DAP, etc.)

use anyhow::Result;
use async_trait::async_trait;
use std::collections::HashMap;

/// Generic transport configuration
#[derive(Debug, Clone)]
pub struct TransportConfig {
    /// Transport type (e.g., "tcp", "ipc", "inproc")
    pub transport_type: String,

    /// Base address (e.g., "127.0.0.1" for TCP, "/tmp/kernel" for IPC)
    pub base_address: String,

    /// Channel to port/endpoint mapping
    pub channels: HashMap<String, ChannelConfig>,
}

/// Configuration for a single channel
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    /// Port number for TCP, or endpoint suffix for IPC
    pub endpoint: String,

    /// Socket pattern (e.g., "router", "pub", "rep")
    pub pattern: String,
}

/// Generic transport for sending/receiving multipart messages
///
/// Transport layer knows NOTHING about protocols - it just moves bytes
#[async_trait]
pub trait Transport: Send + Sync {
    /// Bind to specified addresses from configuration
    async fn bind(&mut self, config: &TransportConfig) -> Result<()>;

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
}
