//! `ZeroMQ` transport implementation
//!
//! This module provides a clean transport layer for messaging using `ZeroMQ`.
//! It knows NOTHING about Jupyter or any other protocol specifics.
//! It only handles raw multipart message transport over `ZeroMQ` sockets.
//!
//! ## Thread Safety
//!
//! `ZeroMQ` sockets are not thread-safe, so we wrap them in Arc<Mutex<>>.
//! All socket operations use the global IO runtime to ensure consistent
//! runtime context across the system.

use anyhow::{Context, Result};
use async_trait::async_trait;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, instrument, trace, warn};
use zmq::{Context as ZmqContext, Socket, SocketType};

use crate::runtime::io_runtime::create_io_bound_resource;
use crate::traits::transport::BoundPorts;
use crate::traits::{ChannelConfig, Transport, TransportConfig};

/// Thread-safe wrapper for `ZeroMQ` Socket
///
/// `ZeroMQ` sockets are actually thread-safe when used correctly,
/// but the Rust bindings don't mark them as Send/Sync.
/// We wrap them to provide the necessary safety guarantees.
struct SafeSocket {
    socket: Socket,
}

// SAFETY: ZeroMQ sockets are thread-safe when protected by a mutex
unsafe impl Send for SafeSocket {}
unsafe impl Sync for SafeSocket {}

/// `ZeroMQ` transport for multipart messaging
///
/// Provides high-performance message passing over various patterns
/// (REQ/REP, PUB/SUB, ROUTER/DEALER, etc.)
#[derive(Clone)]
pub struct ZmqTransport {
    /// `ZeroMQ` context (thread-safe)
    context: Arc<ZmqContext>,
    /// Channel name to socket mapping
    sockets: Arc<Mutex<HashMap<String, SafeSocket>>>,
    /// Special handling for heartbeat socket
    heartbeat_socket: Arc<Mutex<Option<SafeSocket>>>,
    /// Transport configuration
    config: Arc<Mutex<Option<TransportConfig>>>,
}

impl ZmqTransport {
    /// Create a new `ZeroMQ` transport
    ///
    /// This creates the transport within the global IO runtime context
    /// to ensure proper runtime consistency.
    ///
    /// # Errors
    ///
    /// Returns an error if the `ZeroMQ` context cannot be created
    #[instrument(level = "debug")]
    pub fn new() -> Result<Self> {
        debug!("Creating new ZeroMQ transport");

        // Create context within the global runtime
        let context = create_io_bound_resource(ZmqContext::new);

        Ok(Self {
            context: Arc::new(context),
            sockets: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_socket: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        })
    }

    /// Convert pattern string to `ZeroMQ` socket type
    fn pattern_to_socket_type(pattern: &str) -> Result<SocketType> {
        match pattern.to_lowercase().as_str() {
            "router" => Ok(SocketType::ROUTER),
            "dealer" => Ok(SocketType::DEALER),
            "pub" => Ok(SocketType::PUB),
            "sub" => Ok(SocketType::SUB),
            "rep" => Ok(SocketType::REP),
            "req" => Ok(SocketType::REQ),
            "push" => Ok(SocketType::PUSH),
            "pull" => Ok(SocketType::PULL),
            "pair" => Ok(SocketType::PAIR),
            _ => Err(anyhow::anyhow!("Unknown socket pattern: {}", pattern)),
        }
    }

    /// Create and configure a socket
    #[instrument(level = "debug", skip(self))]
    fn create_socket(
        &self,
        channel_name: &str,
        channel_config: &ChannelConfig,
    ) -> Result<SafeSocket> {
        let socket_type = Self::pattern_to_socket_type(&channel_config.pattern)?;

        // Create socket within IO runtime context
        let socket = self.context.socket(socket_type).with_context(|| {
            format!(
                "Failed to create {} socket for {}",
                channel_config.pattern, channel_name
            )
        })?;

        // Configure socket options
        Self::configure_socket(&socket, channel_config)?;

        Ok(SafeSocket { socket })
    }

    /// Configure socket with appropriate options
    fn configure_socket(socket: &Socket, channel_config: &ChannelConfig) -> Result<()> {
        // Set receive timeout for non-blocking operation
        socket
            .set_rcvtimeo(100)
            .context("Failed to set receive timeout")?;

        // Set send timeout
        socket
            .set_sndtimeo(1000)
            .context("Failed to set send timeout")?;

        // Set linger period (wait for pending messages on close)
        socket
            .set_linger(1000)
            .context("Failed to set linger period")?;

        // Apply custom options from configuration
        for (key, value) in &channel_config.options {
            match key.as_str() {
                "sndhwm" => {
                    let hwm: i32 = value.parse().context("Invalid sndhwm value")?;
                    socket.set_sndhwm(hwm).context("Failed to set sndhwm")?;
                }
                "rcvhwm" => {
                    let hwm: i32 = value.parse().context("Invalid rcvhwm value")?;
                    socket.set_rcvhwm(hwm).context("Failed to set rcvhwm")?;
                }
                "identity" => {
                    socket
                        .set_identity(value.as_bytes())
                        .context("Failed to set identity")?;
                }
                _ => {
                    warn!("Unknown socket option: {}", key);
                }
            }
        }

        Ok(())
    }

    /// Build address string from configuration
    fn build_address(config: &TransportConfig, endpoint: &str) -> String {
        match config.transport_type.as_str() {
            "tcp" => format!("tcp://{}:{}", config.base_address, endpoint),
            "ipc" => format!("ipc://{}{}", config.base_address, endpoint),
            "inproc" => format!("inproc://{}{}", config.base_address, endpoint),
            _ => format!(
                "{}://{}:{}",
                config.transport_type, config.base_address, endpoint
            ),
        }
    }

    /// Extract port number from a `ZeroMQ` endpoint string
    ///
    /// Examples:
    /// - <tcp://127.0.0.1:5555> -> 5555
    /// - "tcp://*:5556" -> 5556
    fn extract_port_from_endpoint(endpoint: &str) -> Result<u16> {
        // For TCP endpoints, extract the port after the last colon
        if let Some(pos) = endpoint.rfind(':') {
            let port_str = &endpoint[pos + 1..];
            port_str
                .parse::<u16>()
                .with_context(|| format!("Failed to parse port from endpoint: {endpoint}"))
        } else {
            // For non-TCP endpoints (IPC, inproc), return 0
            Ok(0)
        }
    }
}

#[async_trait]
impl Transport for ZmqTransport {
    #[instrument(level = "info", skip(self, config))]
    async fn bind(&mut self, config: &TransportConfig) -> Result<Option<BoundPorts>> {
        info!(
            "Binding ZeroMQ transport to {} channels",
            config.channels.len()
        );

        // Store configuration
        *self.config.lock() = Some(config.clone());

        // Track actual bound ports for Jupyter channels
        let mut bound_ports = BoundPorts::default();

        // Create and bind sockets for each channel
        for (channel_name, channel_config) in &config.channels {
            let socket = self.create_socket(channel_name, channel_config)?;
            let addr = Self::build_address(config, &channel_config.endpoint);

            socket
                .socket
                .bind(&addr)
                .with_context(|| format!("Failed to bind {channel_name} to {addr}"))?;

            // Get actual bound endpoint (important when port 0 is used)
            let actual_endpoint = socket
                .socket
                .get_last_endpoint()
                .context("Failed to get last endpoint")?;

            // Convert Result<String, Vec<u8>> to String
            let Ok(actual_endpoint) = actual_endpoint else {
                warn!("Could not get endpoint for channel {}", channel_name);
                continue;
            };

            // Extract port from actual endpoint
            let actual_port = Self::extract_port_from_endpoint(&actual_endpoint)?;

            info!(
                "Bound {} channel to {} (actual port: {})",
                channel_name, addr, actual_port
            );

            // Store actual ports for Jupyter channels
            match channel_name.as_str() {
                "shell" => bound_ports.shell = actual_port,
                "iopub" => bound_ports.iopub = actual_port,
                "stdin" => bound_ports.stdin = actual_port,
                "control" => bound_ports.control = actual_port,
                "heartbeat" | "hb" => bound_ports.hb = actual_port,
                _ => {}
            }

            // Special handling for heartbeat channel
            if channel_name == "heartbeat" {
                *self.heartbeat_socket.lock() = Some(socket);
            } else {
                self.sockets.lock().insert(channel_name.clone(), socket);
            }
        }

        // Return bound ports if we have Jupyter channels
        if bound_ports.shell > 0 || bound_ports.control > 0 {
            Ok(Some(bound_ports))
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "info", skip(self, config))]
    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        info!(
            "Connecting ZeroMQ transport to {} channels",
            config.channels.len()
        );

        // Store configuration
        *self.config.lock() = Some(config.clone());

        // Create and connect sockets for each channel
        for (channel_name, channel_config) in &config.channels {
            let socket = self.create_socket(channel_name, channel_config)?;
            let addr = Self::build_address(config, &channel_config.endpoint);

            // Connect instead of bind
            socket
                .socket
                .connect(&addr)
                .with_context(|| format!("Failed to connect {channel_name} to {addr}"))?;

            // For SUB sockets, subscribe to all messages by default
            if channel_config.pattern.to_lowercase() == "sub" {
                socket
                    .socket
                    .set_subscribe(b"")
                    .context("Failed to subscribe to all messages")?;
            }

            info!("Connected {} channel to {}", channel_name, addr);

            // Store the socket
            if channel_name == "heartbeat" {
                *self.heartbeat_socket.lock() = Some(socket);
            } else {
                self.sockets.lock().insert(channel_name.clone(), socket);
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        // Special handling for heartbeat
        if channel == "heartbeat" {
            return self.recv_heartbeat();
        }

        let sockets = self.sockets.lock();
        let socket = sockets
            .get(channel)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel))?;

        // Try to receive with non-blocking flag
        let result = socket.socket.recv_multipart(zmq::DONTWAIT);

        match result {
            Ok(parts) => {
                trace!("Received {} parts on {} channel", parts.len(), channel);
                Ok(Some(parts))
            }
            Err(zmq::Error::EAGAIN) => {
                // No message available (non-blocking)
                Ok(None)
            }
            Err(e) => Err(anyhow::Error::from(e))
                .context(format!("Failed to receive on {channel} channel")),
        }
    }

    #[instrument(level = "trace", skip(self, parts))]
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
        trace!("Sending {} parts on {} channel", parts.len(), channel);

        // Special handling for heartbeat
        if channel == "heartbeat" {
            return self.send_heartbeat(&parts);
        }

        let sockets = self.sockets.lock();
        let socket = sockets
            .get(channel)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel))?;

        socket
            .socket
            .send_multipart(parts, 0)
            .with_context(|| format!("Failed to send on {channel} channel"))?;

        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    async fn heartbeat(&self) -> Result<bool> {
        let heartbeat = self.heartbeat_socket.lock();
        if let Some(socket) = heartbeat.as_ref() {
            match socket.socket.recv_bytes(zmq::DONTWAIT) {
                Ok(data) => {
                    // Echo back immediately
                    socket
                        .socket
                        .send(&data, 0)
                        .context("Failed to send heartbeat response")?;
                    trace!("Heartbeat echoed");
                    Ok(true)
                }
                Err(zmq::Error::EAGAIN) => {
                    // No heartbeat received
                    Ok(false)
                }
                Err(e) => Err(anyhow::Error::from(e)).context("Heartbeat receive error"),
            }
        } else {
            Ok(false)
        }
    }

    fn has_channel(&self, channel: &str) -> bool {
        if channel == "heartbeat" {
            self.heartbeat_socket.lock().is_some()
        } else {
            self.sockets.lock().contains_key(channel)
        }
    }

    fn channels(&self) -> Vec<String> {
        let mut channels: Vec<String> = self.sockets.lock().keys().cloned().collect();
        if self.heartbeat_socket.lock().is_some() {
            channels.push("heartbeat".to_string());
        }
        channels.sort();
        channels
    }

    #[instrument(level = "info", skip(self))]
    async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down ZeroMQ transport");

        // Clear all sockets
        self.sockets.lock().clear();
        *self.heartbeat_socket.lock() = None;

        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }
}

impl ZmqTransport {
    /// Receive from heartbeat channel
    fn recv_heartbeat(&self) -> Result<Option<Vec<Vec<u8>>>> {
        let heartbeat = self.heartbeat_socket.lock();
        if let Some(socket) = heartbeat.as_ref() {
            match socket.socket.recv_bytes(zmq::DONTWAIT) {
                Ok(data) => Ok(Some(vec![data])),
                Err(zmq::Error::EAGAIN) => Ok(None),
                Err(e) => Err(anyhow::Error::from(e)).context("Heartbeat receive error"),
            }
        } else {
            Ok(None)
        }
    }

    /// Send to heartbeat channel
    fn send_heartbeat(&self, parts: &[Vec<u8>]) -> Result<()> {
        let heartbeat = self.heartbeat_socket.lock();
        if let Some(socket) = heartbeat.as_ref() {
            if !parts.is_empty() {
                socket
                    .socket
                    .send(&parts[0], 0)
                    .context("Failed to send heartbeat")?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("Heartbeat channel not initialized"))
        }
    }
}

impl Drop for ZmqTransport {
    fn drop(&mut self) {
        // ZeroMQ sockets are automatically closed when dropped
        debug!("ZeroMQ transport dropped, sockets closed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_pattern_to_socket_type() {
        assert!(matches!(
            ZmqTransport::pattern_to_socket_type("router"),
            Ok(SocketType::ROUTER)
        ));
        assert!(matches!(
            ZmqTransport::pattern_to_socket_type("REQ"),
            Ok(SocketType::REQ)
        ));
        assert!(matches!(
            ZmqTransport::pattern_to_socket_type("pub"),
            Ok(SocketType::PUB)
        ));
        assert!(ZmqTransport::pattern_to_socket_type("invalid").is_err());
    }

    #[test]
    fn test_build_address() {
        let config = TransportConfig {
            transport_type: "tcp".to_string(),
            base_address: "127.0.0.1".to_string(),
            channels: HashMap::new(),
            auth_key: None,
        };

        assert_eq!(
            ZmqTransport::build_address(&config, "5555"),
            "tcp://127.0.0.1:5555"
        );

        let config = TransportConfig {
            transport_type: "ipc".to_string(),
            base_address: "/tmp/kernel".to_string(),
            channels: HashMap::new(),
            auth_key: None,
        };

        assert_eq!(
            ZmqTransport::build_address(&config, "-shell"),
            "ipc:///tmp/kernel-shell"
        );
    }

    #[tokio::test]
    async fn test_zmq_transport_creation() {
        let transport = ZmqTransport::new();
        assert!(transport.is_ok());

        let transport = transport.unwrap();
        assert_eq!(transport.channels().len(), 0);
        assert!(!transport.has_channel("shell"));
    }
}
