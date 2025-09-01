//! Server configuration types
//!
//! This module now only contains configuration types.
//! The actual server implementation has been moved to `UnifiedProtocolEngine`.

use thiserror::Error;

/// Server-side protocol errors
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport error: {0}")]
    Transport(#[from] crate::TransportError),

    #[error("Bind error: {0}")]
    Bind(String),

    #[error("Handler error: {0}")]
    Handler(String),
}

/// Protocol server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// IP address to bind to
    pub ip: String,

    /// Port for shell channel
    pub shell_port: u16,

    /// Port for `IOPub` channel
    pub iopub_port: u16,

    /// Port for stdin channel
    pub stdin_port: u16,

    /// Port for control channel
    pub control_port: u16,

    /// Port for heartbeat channel
    pub heartbeat_port: u16,

    /// Maximum number of concurrent connections
    pub max_connections: usize,
}

impl ServerConfig {
    /// Create default server configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create configuration from base port
    #[must_use]
    pub fn from_base_port(ip: impl Into<String>, base_port: u16) -> Self {
        Self {
            ip: ip.into(),
            shell_port: base_port,
            iopub_port: base_port + 1,
            stdin_port: base_port + 2,
            control_port: base_port + 3,
            heartbeat_port: base_port + 4,
            max_connections: 100,
        }
    }

    /// Get the shell channel address
    #[must_use]
    pub fn shell_addr(&self) -> String {
        format!("{}:{}", self.ip, self.shell_port)
    }

    /// Get the `IOPub` channel address
    #[must_use]
    pub fn iopub_addr(&self) -> String {
        format!("{}:{}", self.ip, self.iopub_port)
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            shell_port: 9555,
            iopub_port: 9556,
            stdin_port: 9557,
            control_port: 9558,
            heartbeat_port: 9559,
            max_connections: 100,
        }
    }
}
