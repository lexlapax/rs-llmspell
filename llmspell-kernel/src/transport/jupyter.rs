//! Jupyter-specific transport implementation
//!
//! This module provides a Jupyter protocol transport that manages the
//! 5-channel architecture required by Jupyter:
//! - Shell: Execute requests and replies
//! - `IOPub`: Output publishing and status updates
//! - Stdin: Input requests from kernel to frontend
//! - Control: Shutdown, interrupt, and daemon management
//! - Heartbeat: Keepalive mechanism
//!
//! This is a thin wrapper around the generic Transport trait that ensures
//! all 5 channels are properly configured for Jupyter communication.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, instrument, warn};

use crate::traits::{ChannelConfig, Transport, TransportConfig};
use crate::traits::transport::BoundPorts;

/// Jupyter connection file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterConnectionInfo {
    /// Transport type (always "tcp" for standard Jupyter)
    pub transport: String,
    /// IP address to bind/connect to
    pub ip: String,
    /// Shell channel port (REQ/REP for execute requests)
    pub shell_port: u16,
    /// `IOPub` channel port (PUB for output broadcasting)
    pub iopub_port: u16,
    /// Stdin channel port (REQ/REP for input requests)
    pub stdin_port: u16,
    /// Control channel port (REQ/REP for control commands)
    pub control_port: u16,
    /// Heartbeat channel port (REQ/REP for keepalive)
    pub hb_port: u16,
    /// Authentication key for message signing
    pub key: String,
    /// Signature scheme (e.g., "hmac-sha256")
    pub signature_scheme: String,
    /// Kernel ID
    pub kernel_id: String,
}

/// Jupyter-specific transport wrapper
///
/// Ensures proper 5-channel setup and Jupyter protocol compliance
pub struct JupyterTransport {
    /// Underlying transport implementation
    transport: Box<dyn Transport>,
    /// Connection information
    connection_info: Option<JupyterConnectionInfo>,
}

impl JupyterTransport {
    /// Create a new Jupyter transport with the specified implementation
    #[instrument(level = "debug", skip_all)]
    pub fn new(transport: Box<dyn Transport>) -> Self {
        debug!("Creating new Jupyter transport wrapper");
        Self {
            transport,
            connection_info: None,
        }
    }

    /// Create from a Jupyter connection file
    ///
    /// # Errors
    ///
    /// Returns an error if the connection file cannot be read or parsed
    #[instrument(level = "info")]
    pub async fn from_connection_file(path: &Path) -> Result<Self> {
        info!("Loading Jupyter connection from {:?}", path);

        // Read and parse connection file
        let contents = tokio::fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read connection file: {}", path.display()))?;

        let connection_info: JupyterConnectionInfo = serde_json::from_str(&contents)
            .with_context(|| format!("Failed to parse connection file: {}", path.display()))?;

        // Create transport based on type
        let transport = crate::traits::create_transport("zeromq")?;

        let mut jupyter_transport = Self {
            transport,
            connection_info: Some(connection_info.clone()),
        };

        // Build configuration for 5 channels
        let config = Self::build_config(&connection_info);

        // Bind to all channels and get actual ports
        let bound_ports = jupyter_transport.transport.bind(&config).await?;

        // Store bound ports if available
        if let Some(ports) = bound_ports {
            debug!("Jupyter transport bound to ports - shell: {}, iopub: {}, stdin: {}, control: {}, hb: {}",
                  ports.shell, ports.iopub, ports.stdin, ports.control, ports.hb);
        }

        Ok(jupyter_transport)
    }

    /// Build transport configuration from Jupyter connection info
    fn build_config(info: &JupyterConnectionInfo) -> TransportConfig {
        let mut channels = HashMap::new();

        // Shell channel (ROUTER for kernel, DEALER for client)
        channels.insert(
            "shell".to_string(),
            ChannelConfig {
                endpoint: info.shell_port.to_string(),
                pattern: "router".to_string(),
                options: HashMap::new(),
            },
        );

        // IOPub channel (PUB for kernel, SUB for client)
        channels.insert(
            "iopub".to_string(),
            ChannelConfig {
                endpoint: info.iopub_port.to_string(),
                pattern: "pub".to_string(),
                options: HashMap::new(),
            },
        );

        // Stdin channel (ROUTER for kernel, DEALER for client)
        channels.insert(
            "stdin".to_string(),
            ChannelConfig {
                endpoint: info.stdin_port.to_string(),
                pattern: "router".to_string(),
                options: HashMap::new(),
            },
        );

        // Control channel (ROUTER for kernel, DEALER for client)
        channels.insert(
            "control".to_string(),
            ChannelConfig {
                endpoint: info.control_port.to_string(),
                pattern: "router".to_string(),
                options: HashMap::new(),
            },
        );

        // Heartbeat channel (REP for kernel, REQ for client)
        channels.insert(
            "heartbeat".to_string(),
            ChannelConfig {
                endpoint: info.hb_port.to_string(),
                pattern: "rep".to_string(),
                options: HashMap::new(),
            },
        );

        TransportConfig {
            transport_type: info.transport.clone(),
            base_address: info.ip.clone(),
            channels,
            auth_key: Some(info.key.clone()),
        }
    }

    /// Connect as a Jupyter client (for testing or external communication)
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails
    #[instrument(level = "info", skip(self))]
    pub async fn connect_as_client(&mut self, info: &JupyterConnectionInfo) -> Result<()> {
        info!("Connecting as Jupyter client");

        self.connection_info = Some(info.clone());

        // Build client configuration (reversed socket types)
        let mut config = Self::build_config(info);

        // Reverse socket patterns for client mode
        for channel in config.channels.values_mut() {
            channel.pattern = match channel.pattern.as_str() {
                "router" => "dealer".to_string(),
                "pub" => "sub".to_string(),
                "rep" => "req".to_string(),
                pattern => pattern.to_string(),
            };
        }

        // Connect instead of bind
        self.transport.connect(&config).await?;

        Ok(())
    }

    /// Get connection info
    pub fn connection_info(&self) -> Option<&JupyterConnectionInfo> {
        self.connection_info.as_ref()
    }

    /// Check if all 5 Jupyter channels are ready
    pub fn is_ready(&self) -> bool {
        self.transport.has_channel("shell")
            && self.transport.has_channel("iopub")
            && self.transport.has_channel("stdin")
            && self.transport.has_channel("control")
            && self.transport.has_channel("heartbeat")
    }
}

// Delegate Transport trait to underlying implementation
#[async_trait]
impl Transport for JupyterTransport {
    async fn bind(&mut self, config: &TransportConfig) -> Result<Option<BoundPorts>> {
        // Validate that all 5 channels are present
        if !config.channels.contains_key("shell")
            || !config.channels.contains_key("iopub")
            || !config.channels.contains_key("stdin")
            || !config.channels.contains_key("control")
            || !config.channels.contains_key("heartbeat")
        {
            return Err(anyhow::anyhow!(
                "Jupyter transport requires all 5 channels: shell, iopub, stdin, control, heartbeat"
            ));
        }

        self.transport.bind(config).await
    }

    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        self.transport.connect(config).await
    }

    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        self.transport.recv(channel).await
    }

    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
        self.transport.send(channel, parts).await
    }

    async fn heartbeat(&self) -> Result<bool> {
        self.transport.heartbeat().await
    }

    fn has_channel(&self, channel: &str) -> bool {
        self.transport.has_channel(channel)
    }

    fn channels(&self) -> Vec<String> {
        self.transport.channels()
    }

    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Jupyter transport");
        self.transport.shutdown().await
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        // Note: This requires the underlying transport to support cloning
        self.transport.box_clone()
    }
}

/// Create a standard Jupyter kernel transport
///
/// # Errors
///
/// Returns an error if transport creation fails
#[instrument(level = "info")]
pub async fn create_jupyter_kernel_transport(
    connection_file: Option<&Path>,
) -> Result<JupyterTransport> {
    if let Some(path) = connection_file {
        JupyterTransport::from_connection_file(path).await
    } else {
        // Create with default configuration
        let transport = crate::traits::create_transport("zeromq")?;
        Ok(JupyterTransport::new(transport))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[test]
    fn test_connection_info_parsing() {
        let json = r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 60001,
            "iopub_port": 60002,
            "stdin_port": 60003,
            "control_port": 60004,
            "hb_port": 60005,
            "key": "test-key",
            "signature_scheme": "hmac-sha256",
            "kernel_id": "test-kernel"
        }"#;

        let info: JupyterConnectionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.transport, "tcp");
        assert_eq!(info.ip, "127.0.0.1");
        assert_eq!(info.shell_port, 60001);
        assert_eq!(info.key, "test-key");
    }

    #[test]
    fn test_build_config() {
        let info = JupyterConnectionInfo {
            transport: "tcp".to_string(),
            ip: "127.0.0.1".to_string(),
            shell_port: 60001,
            iopub_port: 60002,
            stdin_port: 60003,
            control_port: 60004,
            hb_port: 60005,
            key: "test-key".to_string(),
            signature_scheme: "hmac-sha256".to_string(),
            kernel_id: "test-kernel".to_string(),
        };

        let config = JupyterTransport::build_config(&info);

        assert_eq!(config.transport_type, "tcp");
        assert_eq!(config.base_address, "127.0.0.1");
        assert_eq!(config.channels.len(), 5);
        assert!(config.channels.contains_key("shell"));
        assert!(config.channels.contains_key("heartbeat"));
        assert_eq!(config.channels["shell"].pattern, "router");
        assert_eq!(config.channels["iopub"].pattern, "pub");
    }

    #[tokio::test]
    async fn test_jupyter_transport_creation() {
        // Create a mock transport
        let transport = crate::traits::create_transport("zeromq").unwrap();
        let jupyter = JupyterTransport::new(transport);

        assert!(!jupyter.is_ready());
        assert!(jupyter.connection_info().is_none());
    }
}
