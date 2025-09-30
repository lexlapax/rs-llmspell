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
pub struct ChannelPair {
    pub sender: mpsc::UnboundedSender<Vec<Vec<u8>>>,
    pub receiver: Arc<RwLock<mpsc::UnboundedReceiver<Vec<Vec<u8>>>>>,
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
    /// Get access to channels map for debugging
    pub fn get_channels_map(&self) -> &Arc<RwLock<HashMap<String, Arc<ChannelPair>>>> {
        &self.channels
    }

    /// Get access to reverse channels map for debugging
    pub fn get_reverse_channels_map(&self) -> &Arc<RwLock<HashMap<String, Arc<ChannelPair>>>> {
        &self.reverse_channels
    }

    /// Create a new in-process transport
    pub fn new() -> Self {
        let transport = Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            reverse_channels: Arc::new(RwLock::new(HashMap::new())),
        };
        println!("DEBUG: InProcessTransport::new() creating instance at {:p} with channels: {:p}, reverse: {:p}",
                 &transport as *const _, Arc::as_ptr(&transport.channels), Arc::as_ptr(&transport.reverse_channels));
        trace!("Created new InProcessTransport with channels: {:p}, reverse: {:p}",
               Arc::as_ptr(&transport.channels), Arc::as_ptr(&transport.reverse_channels));
        transport
    }

    /// Create a connected pair of in-process transports
    ///
    /// Returns (`kernel_transport`, `client_transport`) that are connected
    pub fn create_pair() -> (Self, Self) {
        // Create shared channel maps that both transports will use
        let channels_map1 = Arc::new(RwLock::new(HashMap::new()));
        let reverse_channels_map1 = Arc::new(RwLock::new(HashMap::new()));
        let channels_map2 = Arc::new(RwLock::new(HashMap::new()));
        let reverse_channels_map2 = Arc::new(RwLock::new(HashMap::new()));

        // Transport1 uses channels_map1 for sending, reverse_channels_map1 for receiving
        let transport1 = Self {
            channels: channels_map1.clone(),
            reverse_channels: reverse_channels_map1.clone(),
        };

        // Transport2 uses channels_map2 for sending, reverse_channels_map2 for receiving
        let transport2 = Self {
            channels: channels_map2.clone(),
            reverse_channels: reverse_channels_map2.clone(),
        };

        trace!("Created transport pair with separate channel maps for proper bidirectional setup");

        (transport1, transport2)
    }

    /// Setup a channel with the given name
    /// For paired transports, this needs special handling
    fn setup_channel(&self, name: &str) {
        println!("DEBUG: setup_channel() called for '{}'", name);
        // Check if channel already exists
        if self.channels.read().contains_key(name) {
            trace!("Channel {} already exists, skipping setup", name);
            println!("DEBUG: Channel '{}' already exists, skipping", name);
            return;
        }

        let (tx, rx) = mpsc::unbounded_channel();
        let channel = Arc::new(ChannelPair {
            sender: tx,
            receiver: Arc::new(RwLock::new(rx)),
        });

        trace!("Setting up channel '{}' with sender: {:p}", name, &channel.sender as *const _);
        self.channels
            .write()
            .insert(name.to_string(), channel.clone());

        // Also setup reverse channel for bidirectional communication
        let (rev_tx, rev_rx) = mpsc::unbounded_channel();
        let reverse_channel = Arc::new(ChannelPair {
            sender: rev_tx,
            receiver: Arc::new(RwLock::new(rev_rx)),
        });

        trace!("Setting up reverse channel '{}' with sender: {:p}", name, &reverse_channel.sender as *const _);
        self.reverse_channels
            .write()
            .insert(name.to_string(), reverse_channel);
    }

    /// Setup paired channels between two transports for bidirectional communication
    pub fn setup_paired_channel(transport1: &mut Self, transport2: &mut Self, name: &str) {
        // Create two channel pairs for bidirectional communication
        // Pair 1: transport1 sends -> transport2 receives
        let (tx1, rx1) = mpsc::unbounded_channel();
        let pair1 = Arc::new(ChannelPair {
            sender: tx1,
            receiver: Arc::new(RwLock::new(rx1)),
        });

        // Pair 2: transport2 sends -> transport1 receives
        let (tx2, rx2) = mpsc::unbounded_channel();
        let pair2 = Arc::new(ChannelPair {
            sender: tx2,
            receiver: Arc::new(RwLock::new(rx2)),
        });

        // Transport1 (kernel): sends via pair1, receives via pair2
        transport1.channels.write().insert(name.to_string(), pair1.clone());
        transport1.reverse_channels.write().insert(name.to_string(), pair2.clone());

        // Transport2 (client): sends via pair2, receives via pair1
        transport2.channels.write().insert(name.to_string(), pair2.clone());
        transport2.reverse_channels.write().insert(name.to_string(), pair1.clone());

        // Verify the setup
        println!("DEBUG VERIFY [{}]: T1.channels has sender {:p}, T1.reverse has receiver {:p}",
                 name,
                 &transport1.channels.read().get(name).unwrap().sender as *const _,
                 Arc::as_ptr(&transport1.reverse_channels.read().get(name).unwrap().receiver));
        println!("DEBUG VERIFY [{}]: T2.channels has sender {:p}, T2.reverse has receiver {:p}",
                 name,
                 &transport2.channels.read().get(name).unwrap().sender as *const _,
                 Arc::as_ptr(&transport2.reverse_channels.read().get(name).unwrap().receiver));

        println!("DEBUG: Setup paired channel '{}':", name);
        println!("  [{}] T1 sends via sender {:p}, T2 receives via receiver {:p}",
               name, &pair1.sender as *const _, Arc::as_ptr(&pair1.receiver));
        println!("  [{}] T2 sends via sender {:p}, T1 receives via receiver {:p}",
               name, &pair2.sender as *const _, Arc::as_ptr(&pair2.receiver));

        trace!("Setup paired channel '{}' between transports", name);
        trace!("  T1 sends via {:p}, receives via Arc<RwLock> at {:p}",
               &pair1.sender as *const _, Arc::as_ptr(&pair2.receiver));
        trace!("  T2 sends via {:p}, receives via Arc<RwLock> at {:p}",
               &pair2.sender as *const _, Arc::as_ptr(&pair1.receiver));
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
        println!("DEBUG: bind() called on InProcessTransport");
        debug!(
            "Binding in-process transport to {} channels",
            config.channels.len()
        );

        // Setup channels for all configured endpoints
        // Don't overwrite channels that already exist from create_pair()
        for name in config.channels.keys() {
            if !self.has_channel(name) {
                self.setup_channel(name);
                trace!("Setup new in-process channel: {}", name);
            } else {
                trace!("Using existing paired channel: {}", name);
            }
        }

        Ok(None) // In-process transport doesn't use real ports
    }

    #[instrument(level = "debug", skip_all)]
    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        println!("DEBUG: connect() called on InProcessTransport");
        debug!(
            "Connecting in-process transport to {} channels",
            config.channels.len()
        );

        // For in-process paired transports, channels are already shared
        // Only setup missing channels for non-paired transports
        for name in config.channels.keys() {
            if !self.has_channel(name) {
                self.setup_channel(name);
                trace!("Setup new in-process channel: {}", name);
            } else {
                trace!("Using existing paired channel: {}", name);
            }
        }

        Ok(())
    }

    #[instrument(level = "trace", skip_all)]
    async fn recv(&self, channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        // Add println! to ensure we see this
        println!("DEBUG: InProcessTransport::recv() called for channel '{}'", channel);
        println!("DEBUG: recv() on transport at {:p}, reverse_channels Arc: {:p}",
                 self as *const _, Arc::as_ptr(&self.reverse_channels));
        // Receive from the reverse channel (what was sent to us)
        let channels = self.reverse_channels.read();

        // Only log once to avoid spam
        static mut LOGGED: bool = false;
        unsafe {
            if !LOGGED {
                println!("DEBUG: Available reverse_channels in recv(): {:?}",
                         channels.keys().cloned().collect::<Vec<_>>());
                LOGGED = true;
            }
        }

        if let Some(channel_pair) = channels.get(channel) {
            let receiver_ptr = Arc::as_ptr(&channel_pair.receiver);
            trace!("Receiving from channel {} via receiver: {:p}", channel, receiver_ptr);

            // Debug: show the receiver we're using
            println!("DEBUG: recv() on channel '{}' using receiver at {:p}", channel, receiver_ptr);

            let mut receiver = channel_pair.receiver.write();

            // Check if there's a message available
            println!("DEBUG: Checking for message on channel '{}' from receiver", channel);

            match receiver.try_recv() {
                Ok(message) => {
                    debug!(
                        "InProcessTransport::recv() received message on channel {}: {} parts",
                        channel,
                        message.len()
                    );
                    trace!(
                        "Received message on channel {}: {} parts",
                        channel,
                        message.len()
                    );
                    Ok(Some(message))
                }
                Err(mpsc::error::TryRecvError::Empty) => {
                    // Only log at trace level to avoid spam
                    trace!("No message available on channel {}", channel);
                    Ok(None)
                }
                Err(mpsc::error::TryRecvError::Disconnected) => {
                    debug!("Channel {} disconnected!", channel);
                    Err(anyhow::anyhow!("Channel {} disconnected", channel))
                }
            }
        } else {
            debug!("Channel {} not found in reverse_channels! Available: {:?}",
                   channel, channels.keys().cloned().collect::<Vec<_>>());
            Ok(None)
        }
    }

    #[instrument(level = "trace", skip_all)]
    async fn send(&self, channel: &str, parts: Vec<Vec<u8>>) -> Result<()> {
        // Add println! to ensure we see this
        println!("DEBUG: InProcessTransport::send() called on transport at {:p} for channel '{}' with {} parts",
                 self as *const _, channel, parts.len());
        println!("DEBUG: send() channels Arc: {:p}, reverse_channels Arc: {:p}",
                 Arc::as_ptr(&self.channels), Arc::as_ptr(&self.reverse_channels));
        debug!("InProcessTransport::send() called for channel '{}' with {} parts", channel, parts.len());
        trace!("send() called on transport with channels: {:p}, reverse: {:p}",
               Arc::as_ptr(&self.channels), Arc::as_ptr(&self.reverse_channels));
        let channels = self.channels.read();
        println!("DEBUG: Available channels in send(): {:?}", channels.keys().cloned().collect::<Vec<_>>());

        if let Some(channel_pair) = channels.get(channel) {
            trace!(
                "Sending message on channel {}: {} parts",
                channel,
                parts.len()
            );
            let sender_ptr = &channel_pair.sender as *const _;
            debug!("Found channel '{}', sender ptr: {:p}", channel, sender_ptr);
            trace!("Sending to channel {} via sender: {:p}", channel, sender_ptr);

            // Log first part for debugging
            if let Some(first_part) = parts.first() {
                debug!("First part size: {} bytes", first_part.len());
            }

            // Check if sender has capacity
            println!("DEBUG: send() on channel '{}' using sender at {:p}", channel, sender_ptr);
            println!("DEBUG: Sending to channel '{}', sender is_closed: {}", channel, channel_pair.sender.is_closed());

            let send_result = channel_pair
                .sender
                .send(parts.clone());

            match send_result {
                Ok(()) => {
                    println!("DEBUG: Message successfully sent to channel '{}'", channel);
                    debug!("Successfully queued message on channel '{}'", channel);
                }
                Err(e) => {
                    println!("DEBUG: Failed to send to channel '{}': {:?}", channel, e);
                    return Err(anyhow::anyhow!("Failed to send on channel {}: {:?}", channel, e));
                }
            }
            trace!("Successfully sent message on channel {}", channel);
            Ok(())
        } else {
            debug!("Channel '{}' not found in transport! Available channels: {:?}",
                   channel, channels.keys().cloned().collect::<Vec<_>>());
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
        println!("DEBUG: box_clone() called on InProcessTransport!");
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
