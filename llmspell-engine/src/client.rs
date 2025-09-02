//! Client-side protocol handler
//!
//! Manages request/response correlation and connection state

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, oneshot, RwLock};
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
    transport: Arc<RwLock<Box<dyn Transport>>>,

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
        let transport = Arc::new(RwLock::new(transport));
        let pending = Arc::new(RwLock::new(HashMap::new()));
        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);

        // Spawn receiver task
        let transport_clone = transport.clone();
        let pending_clone = pending.clone();
        let receiver_handle = tokio::spawn(async move {
            debug!("Protocol client receiver task starting");
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
                        debug!("Received and processed a message successfully");
                    }
                }
            }
            debug!("Protocol client receiver task exiting");
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
        debug!("Sending LRP request with msg_id={}: {:?}", msg_id, request);
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

    /// Register a pending request for response tracking
    async fn register_pending(&self, msg_id: &str) -> oneshot::Receiver<ProtocolMessage> {
        let (tx, rx) = oneshot::channel();
        self.pending.write().await.insert(msg_id.to_string(), tx);
        debug!("Registered pending request for msg_id={}", msg_id);
        rx
    }

    /// Send a message through the transport
    async fn send_message(&self, msg: ProtocolMessage) -> Result<(), ClientError> {
        let msg_id = msg.msg_id.clone();
        debug!("Sending message via transport: msg_id={}", msg_id);

        self.transport.read().await.send(msg).await?;
        debug!("Message sent successfully: msg_id={}", msg_id);
        Ok(())
    }

    /// Handle successful response
    fn handle_response_success(msg_id: &str, response: &ProtocolMessage) -> ProtocolMessage {
        debug!(
            "Received response for msg_id={}: msg_type={:?}",
            msg_id, response.msg_type
        );
        response.clone()
    }

    /// Handle response channel error
    async fn handle_response_error(&self, msg_id: &str, is_timeout: bool) -> ClientError {
        if is_timeout {
            debug!("Timeout waiting for response to msg_id={}", msg_id);
            self.cleanup_pending(msg_id).await;
            ClientError::Timeout
        } else {
            debug!("Channel closed for msg_id={}", msg_id);
            self.cleanup_pending(msg_id).await;
            ClientError::RequestCancelled
        }
    }

    /// Wait for a response with timeout
    async fn wait_for_response(
        &self,
        msg_id: String,
        rx: oneshot::Receiver<ProtocolMessage>,
    ) -> Result<ProtocolMessage, ClientError> {
        debug!("Waiting for response to msg_id={}", msg_id);

        let timeout_duration = std::time::Duration::from_secs(30);
        let result = tokio::time::timeout(timeout_duration, rx).await;

        match result {
            Ok(Ok(response)) => Ok(Self::handle_response_success(&msg_id, &response)),
            Ok(Err(_)) => Err(self.handle_response_error(&msg_id, false).await),
            Err(_) => Err(self.handle_response_error(&msg_id, true).await),
        }
    }

    /// Remove a pending request from the tracking map
    async fn cleanup_pending(&self, msg_id: &str) {
        self.pending.write().await.remove(msg_id);
    }

    /// Send a message and wait for response
    async fn send_and_wait(&self, msg: ProtocolMessage) -> Result<ProtocolMessage, ClientError> {
        let msg_id = msg.msg_id.clone();
        debug!(
            "send_and_wait: msg_id={}, msg_type={:?}",
            msg_id, msg.msg_type
        );

        // Register pending request
        let rx = self.register_pending(&msg_id).await;

        // Send request
        self.send_message(msg).await?;

        // Wait for response
        self.wait_for_response(msg_id, rx).await
    }

    /// Receive a single message from the transport
    async fn recv_from_transport(
        transport: Arc<RwLock<Box<dyn Transport>>>,
    ) -> Result<ProtocolMessage, ClientError> {
        debug!("Attempting to receive message from transport");

        let result = {
            let transport_guard = transport.read().await;
            debug!("Acquired transport read lock, calling recv");
            transport_guard.recv().await
        };

        debug!(
            "Transport recv completed: {:?}",
            result.as_ref().map(|m| &m.msg_id)
        );
        result.map_err(ClientError::from)
    }

    /// Send response to waiting request
    fn send_to_pending(tx: oneshot::Sender<ProtocolMessage>, msg: ProtocolMessage, msg_id: &str) {
        debug!(
            "Found pending request for msg_id={}, sending response",
            msg_id
        );
        let _ = tx.send(msg);
        debug!("Response sent to pending request: msg_id={}", msg_id);
    }

    /// Route a response message to its waiting request
    async fn route_response(
        msg: ProtocolMessage,
        pending: Arc<RwLock<HashMap<String, oneshot::Sender<ProtocolMessage>>>>,
    ) {
        debug!("Routing response to pending request: msg_id={}", msg.msg_id);
        let msg_id = msg.msg_id.clone();

        let tx_opt = pending.write().await.remove(&msg_id);

        if let Some(tx) = tx_opt {
            Self::send_to_pending(tx, msg, &msg_id);
        } else {
            warn!("Received response for unknown request: {}", msg_id);
        }
    }

    /// Handle a notification message
    fn handle_notification(msg: &ProtocolMessage) {
        debug!(
            "Received notification on channel {}: {:?}",
            msg.channel, msg.content
        );
    }

    /// Receive messages from transport and route to pending requests
    async fn receive_message(
        transport: Arc<RwLock<Box<dyn Transport>>>,
        pending: Arc<RwLock<HashMap<String, oneshot::Sender<ProtocolMessage>>>>,
    ) -> Result<(), ClientError> {
        let msg = Self::recv_from_transport(transport).await?;
        debug!(
            "Received message: msg_id={}, msg_type={:?}",
            msg.msg_id, msg.msg_type
        );

        Self::process_received_message(msg, pending).await;
        Ok(())
    }

    /// Process a received message based on its type
    async fn process_received_message(
        msg: ProtocolMessage,
        pending: Arc<RwLock<HashMap<String, oneshot::Sender<ProtocolMessage>>>>,
    ) {
        match msg.msg_type {
            MessageType::Response | MessageType::Error => {
                Self::route_response(msg, pending).await;
            }
            MessageType::Notification => {
                Self::handle_notification(&msg);
            }
            MessageType::Request => {
                debug!(
                    "Received unexpected request message type: {:?}",
                    msg.msg_type
                );
            }
        }
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
        let _ = self.transport.write().await.close().await;
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
