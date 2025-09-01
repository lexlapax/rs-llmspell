//! Client-side protocol handler
//!
//! Manages request/response correlation and connection state

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, Mutex, RwLock};
use tracing::{debug, error, warn};

use crate::protocol::message::{MessageType, ProtocolMessage};
use crate::protocol::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};
use crate::transport::{Transport, TransportError};

/// Client-side protocol errors
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Request timeout")]
    Timeout,

    #[error("Invalid response")]
    InvalidResponse,

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Request cancelled")]
    RequestCancelled,
}

/// Protocol client for sending requests and receiving responses
pub struct ProtocolClient {
    /// Transport layer
    transport: Arc<Mutex<Box<dyn Transport>>>,

    /// Pending requests awaiting responses
    pending: Arc<RwLock<HashMap<String, oneshot::Sender<ProtocolMessage>>>>,

    /// Message ID counter
    next_msg_id: AtomicU64,

    /// Receiver task handle
    receiver_handle: Option<tokio::task::JoinHandle<()>>,

    /// Shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl ProtocolClient {
    /// Create a new protocol client with the given transport
    #[must_use]
    pub fn new(transport: Box<dyn Transport>) -> Self {
        let transport = Arc::new(Mutex::new(transport));
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // Spawn receiver task
        let transport_clone = transport.clone();
        let pending_clone = pending.clone();
        let receiver_handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        debug!("Protocol client receiver shutting down");
                        break;
                    }
                    result = Self::receive_message(transport_clone.clone(), pending_clone.clone()) => {
                        if let Err(e) = result {
                            error!("Error receiving message: {}", e);
                            break;
                        }
                    }
                }
            }
        });

        Self {
            transport,
            pending,
            next_msg_id: AtomicU64::new(1),
            receiver_handle: Some(receiver_handle),
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Connect to a server at the given address
    ///
    /// # Errors
    ///
    /// Returns `ClientError::Transport` if connection fails
    pub async fn connect(addr: &str) -> Result<Self, ClientError> {
        use crate::transport::tcp::TcpTransport;

        let transport = TcpTransport::connect(addr).await?;
        Ok(Self::new(Box::new(transport)))
    }

    /// Send an LRP request and wait for response
    ///
    /// # Errors
    ///
    /// Returns `ClientError::InvalidResponse` if response type doesn't match
    /// Returns `ClientError::Timeout` if request times out
    /// Returns `ClientError::Transport` if sending fails
    pub async fn send_lrp_request(&self, request: LRPRequest) -> Result<LRPResponse, ClientError> {
        let msg_id = self.next_msg_id.fetch_add(1, Ordering::SeqCst).to_string();
        let msg = ProtocolMessage::request(&msg_id, request);

        let response = self.send_and_wait(msg).await?;

        response
            .as_lrp_response()
            .ok_or(ClientError::InvalidResponse)
    }

    /// Send an LDP request and wait for response
    ///
    /// # Errors
    ///
    /// Returns `ClientError::InvalidResponse` if response type doesn't match
    /// Returns `ClientError::Timeout` if request times out
    /// Returns `ClientError::Transport` if sending fails
    pub async fn send_ldp_request(&self, request: LDPRequest) -> Result<LDPResponse, ClientError> {
        let msg_id = self.next_msg_id.fetch_add(1, Ordering::SeqCst).to_string();
        let msg = ProtocolMessage::request(&msg_id, request);

        let response = self.send_and_wait(msg).await?;

        response
            .as_ldp_response()
            .ok_or(ClientError::InvalidResponse)
    }

    /// Send a message and wait for response
    async fn send_and_wait(&self, msg: ProtocolMessage) -> Result<ProtocolMessage, ClientError> {
        let msg_id = msg.msg_id.clone();

        // Create response channel
        let (tx, rx) = oneshot::channel();

        // Register pending request
        {
            let mut pending = self.pending.write().await;
            pending.insert(msg_id.clone(), tx);
        }

        // Send request
        self.transport.lock().await.send(msg).await?;

        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(_)) => {
                // Channel closed without response
                self.pending.write().await.remove(&msg_id);
                Err(ClientError::RequestCancelled)
            }
            Err(_) => {
                // Timeout
                self.pending.write().await.remove(&msg_id);
                Err(ClientError::Timeout)
            }
        }
    }

    /// Receive messages from transport and route to pending requests
    async fn receive_message(
        transport: Arc<Mutex<Box<dyn Transport>>>,
        pending: Arc<RwLock<HashMap<String, oneshot::Sender<ProtocolMessage>>>>,
    ) -> Result<(), ClientError> {
        let msg = {
            let mut transport = transport.lock().await;
            transport.recv().await?
        };

        // Route response to waiting request
        if msg.msg_type == MessageType::Response || msg.msg_type == MessageType::Error {
            let mut pending = pending.write().await;
            if let Some(tx) = pending.remove(&msg.msg_id) {
                let _ = tx.send(msg);
            } else {
                warn!("Received response for unknown request: {}", msg.msg_id);
            }
        } else if msg.msg_type == MessageType::Notification {
            // Handle notifications (e.g., IOPub messages)
            debug!(
                "Received notification on channel {}: {:?}",
                msg.channel, msg.content
            );
        }

        Ok(())
    }

    /// Shutdown the client
    pub async fn shutdown(mut self) {
        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Wait for receiver to finish
        if let Some(handle) = self.receiver_handle.take() {
            let _ = handle.await;
        }

        // Close transport
        let _ = self.transport.lock().await.close().await;
    }
}

impl Drop for ProtocolClient {
    fn drop(&mut self) {
        // Attempt to send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.try_send(());
        }
    }
}
