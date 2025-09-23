//! In-process transport for embedded kernel mode
//!
//! This transport uses tokio channels for communication between
//! kernel and CLI in the same process, avoiding network overhead.

use anyhow::Result;
use async_trait::async_trait;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, instrument, trace};

use crate::traits::transport::BoundPorts;
use crate::traits::{Transport, TransportConfig};

/// Channel pair for bidirectional communication
struct ChannelPair {
    sender: mpsc::UnboundedSender<Vec<Vec<u8>>>,
    receiver: Arc<RwLock<mpsc::UnboundedReceiver<Vec<Vec<u8>>>>>,
}

/// In-process transport using tokio channels
///
/// This transport is used for embedded mode where the kernel
/// runs in the same process as the CLI. It provides zero-copy
/// message passing with minimal overhead.
#[derive(Clone)]
pub struct InProcessTransport {
    channels: Arc<RwLock<HashMap<String, Arc<ChannelPair>>>>,
    /// Reverse channels for bidirectional communication
    reverse_channels: Arc<RwLock<HashMap<String, Arc<ChannelPair>>>>,
}

impl InProcessTransport {
    /// Create a new in-process transport
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            reverse_channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a connected pair of in-process transports
    ///
    /// Returns (`kernel_transport`, `client_transport`) that are connected
    pub fn create_pair() -> (Self, Self) {
        let transport1 = Self::new();

        // Share the channel maps between the two transports
        // but in reverse order for bidirectional communication
        let transport2 = Self {
            channels: transport1.reverse_channels.clone(),
            reverse_channels: transport1.channels.clone(),
        };

        (transport1, transport2)
    }

    /// Setup a channel with the given name
    fn setup_channel(&self, name: &str) {
        let (tx, rx) = mpsc::unbounded_channel();
        let channel = Arc::new(ChannelPair {
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        });

        self.channels
            .write()
            .insert(name.to_string(), channel.clone());

        // Also setup reverse channel for bidirectional communication
        let (rev_tx, rev_rx) = mpsc::unbounded_channel();
        let reverse_channel = Arc::new(ChannelPair {
            sender: rev_tx,
            receiver: Arc::new(RwLock::new(rev_rx)),
        });

        self.reverse_channels
            .write()
            .insert(name.to_string(), reverse_channel);
    }
}

impl Default for InProcessTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for InProcessTransport {
    #[instrument(level = "debug", skip_all)]
    async fn bind(&mut self, config: &TransportConfig) -> Result<Option<BoundPorts>> {
        debug!(
            "Binding in-process transport to {} channels",
            config.channels.len()
        );

        // Setup channels for all configured endpoints
        for name in config.channels.keys() {
            self.setup_channel(name);
            trace!("Setup in-process channel: {}", name);
        }

        Ok(None) // In-process transport doesn't use real ports
    }

    #[instrument(level = "debug", skip_all)]
    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        debug!(
            "Connecting in-process transport to {} channels",
            config.channels.len()
        );

        // For in-process, connect is the same as bind
        // The channels are already setup and shared
        for name in config.channels.keys() {
            if !self.has_channel(name) {
                self.setup_channel(name);
            }
            trace!("Connected to in-process channel: {}", name);
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        // Receive from the reverse channel (what was sent to us)
        let channels = self.reverse_channels.read();

        if let Some(channel_pair) = channels.get(channel) {
            let mut receiver = channel_pair.receiver.write();
            match receiver.try_recv() {
                Ok(message) => {
                    trace!(
                        "Received message on channel {}: {} parts",
                        channel,
                        message.len()
                    );
                    Ok(Some(message))
                }
                Err(mpsc::error::TryRecvError::Empty) => Ok(None),
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    Err(anyhow::anyhow!("Channel {} disconnected", channel))
                }
            }
        } else {
            Ok(None)
        }
    }

    #[instrument(level = "trace", skip_all)]
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
        let channels = self.channels.read();

        if let Some(channel_pair) = channels.get(channel) {
            trace!(
                "Sending message on channel {}: {} parts",
                channel,
                parts.len()
            );
            channel_pair
                .sender
                .send(parts)
                .map_err(|_| anyhow::anyhow!("Failed to send on channel {}", channel))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Channel {} not found", channel))
        }
    }

    async fn heartbeat(&self) -> Result<bool> {
        // In-process transport doesn't need heartbeat
        // Always return true to indicate connection is alive
        Ok(true)
    }

    fn has_channel(&self, channel: &str) -> bool {
        self.channels.read().contains_key(channel)
    }

    fn channels(&self) -> Vec<String> {
        self.channels.read().keys().cloned().collect()
    }

    async fn shutdown(&mut self) -> Result<()> {
        debug!("Shutting down in-process transport");
        // Channels will be dropped automatically
        Ok(())
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_inprocess_transport_pair() {
        let (mut kernel_transport, mut client_transport) = InProcessTransport::create_pair();

        // Setup channels on both sides
        let mut config = TransportConfig {
            transport_type: "inprocess".to_string(),
            base_address: String::new(),
            channels: HashMap::new(),
            auth_key: None,
        };

        config.channels.insert(
            "shell".to_string(),
            crate::traits::ChannelConfig {
                endpoint: String::new(),
                pattern: String::new(),
                options: HashMap::new(),
            },
        );

        let _bound_ports = kernel_transport.bind(&config).await.unwrap();
        client_transport.connect(&config).await.unwrap();

        // Send from client to kernel
        let message = vec![b"test".to_vec(), b"message".to_vec()];
        client_transport
            .send("shell", message.clone())
            .await
            .unwrap();

        // Receive on kernel side
        let received = kernel_transport.recv("shell").await.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap(), message);

        // Send from kernel to client
        let reply = vec![b"reply".to_vec()];
        kernel_transport.send("shell", reply.clone()).await.unwrap();

        // Receive on client side
        let received = client_transport.recv("shell").await.unwrap();
        assert!(received.is_some());
        assert_eq!(received.unwrap(), reply);
    }

    #[tokio::test]
    async fn test_multiple_channels() {
        let transport = InProcessTransport::new();

        transport.setup_channel("shell");
        transport.setup_channel("iopub");
        transport.setup_channel("control");

        assert!(transport.has_channel("shell"));
        assert!(transport.has_channel("iopub"));
        assert!(transport.has_channel("control"));
        assert!(!transport.has_channel("unknown"));

        let channels = transport.channels();
        assert_eq!(channels.len(), 3);
        assert!(channels.contains(&"shell".to_string()));
    }
}
