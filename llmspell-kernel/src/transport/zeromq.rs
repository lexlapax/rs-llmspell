//! `ZeroMQ` transport implementation
//!
//! This module provides a clean transport layer for messaging.
//! It knows NOTHING about Jupyter or any other protocol specifics.
//! It only handles raw multipart message transport over `ZeroMQ` sockets.

use anyhow::{Context, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use zmq::{Context as ZmqContext, Socket, SocketType};

use crate::traits::{Transport, TransportConfig};

/// `ZeroMQ` transport for multipart messaging
///
/// Note: `ZeroMQ` sockets are not thread-safe, so we wrap them in Arc<Mutex<>>
pub struct ZmqTransport {
    context: Arc<Mutex<ZmqContext>>,
    sockets: Arc<Mutex<HashMap<String, Socket>>>,
    heartbeat_socket: Arc<Mutex<Option<Socket>>>,
}

impl ZmqTransport {
    /// Create a new `ZeroMQ` transport
    ///
    /// # Errors
    ///
    /// Returns an error if the `ZeroMQ` context cannot be created.
    pub fn new() -> Result<Self> {
        let context = ZmqContext::new();
        Ok(Self {
            context: Arc::new(Mutex::new(context)),
            sockets: Arc::new(Mutex::new(HashMap::new())),
            heartbeat_socket: Arc::new(Mutex::new(None)),
        })
    }

    /// Convert pattern string to `ZeroMQ` socket type
    fn pattern_to_socket_type(pattern: &str) -> Result<SocketType> {
        match pattern.to_lowercase().as_str() {
            "router" => Ok(SocketType::ROUTER),
            "pub" => Ok(SocketType::PUB),
            "rep" => Ok(SocketType::REP),
            "req" => Ok(SocketType::REQ),
            "dealer" => Ok(SocketType::DEALER),
            "sub" => Ok(SocketType::SUB),
            "push" => Ok(SocketType::PUSH),
            "pull" => Ok(SocketType::PULL),
            _ => Err(anyhow::anyhow!("Unknown socket pattern: {}", pattern)),
        }
    }
}

#[async_trait]
impl Transport for ZmqTransport {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()> {
        // Create and bind sockets for each channel
        for (channel_name, channel_config) in &config.channels {
            let socket_type = Self::pattern_to_socket_type(&channel_config.pattern)?;
            let socket = {
                let context = self.context.lock().unwrap();
                context.socket(socket_type).with_context(|| {
                    format!(
                        "Failed to create {} socket for {}",
                        channel_config.pattern, channel_name
                    )
                })?
            };

            // Build the address
            let addr = if config.transport_type == "tcp" {
                format!(
                    "{}://{}:{}",
                    config.transport_type, config.base_address, channel_config.endpoint
                )
            } else {
                format!(
                    "{}://{}{}",
                    config.transport_type, config.base_address, channel_config.endpoint
                )
            };

            socket
                .bind(&addr)
                .with_context(|| format!("Failed to bind {channel_name} to {addr}"))?;

            // Set socket options for non-blocking operation
            socket
                .set_rcvtimeo(100)
                .context("Failed to set receive timeout")?;

            // Special handling for heartbeat channel
            if channel_name == "heartbeat" {
                *self.heartbeat_socket.lock().unwrap() = Some(socket);
            } else {
                self.sockets
                    .lock()
                    .unwrap()
                    .insert(channel_name.clone(), socket);
            }

            tracing::info!("Bound {} channel to {}", channel_name, addr);
        }

        Ok(())
    }

    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        // Create and connect sockets for each channel (client mode)
        for (channel_name, channel_config) in &config.channels {
            let socket_type = Self::pattern_to_socket_type(&channel_config.pattern)?;
            let socket = {
                let context = self.context.lock().unwrap();
                context.socket(socket_type).with_context(|| {
                    format!(
                        "Failed to create {} socket for {}",
                        channel_config.pattern, channel_name
                    )
                })?
            };

            // Build the address
            let addr = if config.transport_type == "tcp" {
                format!(
                    "{}://{}:{}",
                    config.transport_type, config.base_address, channel_config.endpoint
                )
            } else {
                format!(
                    "{}://{}{}",
                    config.transport_type, config.base_address, channel_config.endpoint
                )
            };

            // Connect instead of bind
            socket
                .connect(&addr)
                .with_context(|| format!("Failed to connect {channel_name} to {addr}"))?;

            // Set socket options for non-blocking operation
            socket
                .set_rcvtimeo(100)
                .context("Failed to set receive timeout")?;

            // For SUB sockets, subscribe to all messages
            if channel_config.pattern == "sub" {
                socket
                    .set_subscribe(b"")
                    .context("Failed to subscribe to all messages")?;
            }

            // Store the socket
            self.sockets
                .lock()
                .unwrap()
                .insert(channel_name.clone(), socket);

            tracing::info!("Connected {} channel to {}", channel_name, addr);
        }

        Ok(())
    }

    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        let result = self
            .sockets
            .lock()
            .unwrap()
            .get(channel)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel))?
            .recv_multipart(zmq::DONTWAIT);

        match result {
            Ok(parts) => {
                tracing::debug!("Received {} parts on {} channel", parts.len(), channel);
                Ok(Some(parts))
            }
            Err(zmq::Error::EAGAIN) => Ok(None),
            Err(e) => Err(e).context(format!("Failed to receive on {channel} channel")),
        }
    }

    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
        tracing::debug!("Sending {} parts on {} channel", parts.len(), channel);

        self.sockets
            .lock()
            .unwrap()
            .get(channel)
            .ok_or_else(|| anyhow::anyhow!("Channel {} not found", channel))?
            .send_multipart(parts, 0)
            .with_context(|| format!("Failed to send on {channel} channel"))?;

        Ok(())
    }

    async fn heartbeat(&self) -> Result<bool> {
        let heartbeat = self.heartbeat_socket.lock().unwrap();
        if let Some(socket) = heartbeat.as_ref() {
            match socket.recv_bytes(zmq::DONTWAIT) {
                Ok(data) => {
                    // Echo back immediately
                    socket
                        .send(&data, 0)
                        .context("Failed to send heartbeat response")?;
                    tracing::trace!("Heartbeat echoed");
                    Ok(true)
                }
                Err(zmq::Error::EAGAIN) => Ok(false),
                Err(e) => Err(e).context("Heartbeat receive error"),
            }
        } else {
            Ok(false)
        }
    }

    fn has_channel(&self, channel: &str) -> bool {
        self.sockets.lock().unwrap().contains_key(channel)
            || (channel == "heartbeat" && self.heartbeat_socket.lock().unwrap().is_some())
    }

    fn channels(&self) -> Vec<String> {
        let mut channels: Vec<String> = self.sockets.lock().unwrap().keys().cloned().collect();
        if self.heartbeat_socket.lock().unwrap().is_some() {
            channels.push("heartbeat".to_string());
        }
        channels
    }
}

impl Drop for ZmqTransport {
    fn drop(&mut self) {
        // ZeroMQ sockets are automatically closed when dropped
        tracing::debug!("ZeroMQ transport dropped, sockets closed");
    }
}
