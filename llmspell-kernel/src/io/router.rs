//! Message Router for Multi-Client Support
//!
//! This module provides message routing capabilities to support multiple
//! Jupyter clients connected to the same kernel, ensuring all clients
//! receive I/O outputs and status updates.

use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc::{self, Sender};
use tracing::{debug, instrument, warn};
use uuid::Uuid;

use super::manager::IOPubMessage;
use crate::runtime::tracing::TracingInstrumentation;

/// Message destination for routing
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MessageDestination {
    /// All connected clients
    Broadcast,
    /// Specific client by ID
    Client(String),
    /// Original requester (using parent header)
    Requester,
}

/// Client connection information
#[derive(Debug, Clone)]
pub struct ClientConnection {
    /// Client ID
    pub id: String,
    /// Client session ID
    pub session_id: String,
    /// `IOPub` channel sender for this client
    pub iopub_sender: Sender<IOPubMessage>,
    /// Active status
    pub active: bool,
}

/// Message router for multi-client support
pub struct MessageRouter {
    /// Connected clients
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,
    /// Message history for replay
    message_history: Arc<RwLock<Vec<IOPubMessage>>>,
    /// Maximum history size
    max_history: usize,
    /// Tracing instrumentation
    tracing: Option<TracingInstrumentation>,
}

impl MessageRouter {
    /// Create a new message router
    #[instrument(level = "debug")]
    pub fn new(max_history: usize) -> Self {
        debug!("Creating MessageRouter with max_history={}", max_history);

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history,
            tracing: None,
        }
    }

    /// Set tracing instrumentation
    pub fn set_tracing(&mut self, tracing: TracingInstrumentation) {
        self.tracing = Some(tracing);
    }

    /// Register a new client
    #[instrument(level = "info", skip(self, sender))]
    pub fn register_client(&self, session_id: String, sender: Sender<IOPubMessage>) -> String {
        let client_id = Uuid::new_v4().to_string();

        let connection = ClientConnection {
            id: client_id.clone(),
            session_id,
            iopub_sender: sender,
            active: true,
        };

        self.clients.write().insert(client_id.clone(), connection);

        debug!("Registered client {}", client_id);

        if let Some(ref tracing) = self.tracing {
            tracing.trace_session_operation("client_register", &client_id);
        }

        client_id
    }

    /// Unregister a client
    #[instrument(level = "info", skip(self))]
    pub fn unregister_client(&self, client_id: &str) -> bool {
        let removed = self.clients.write().remove(client_id).is_some();

        if removed {
            debug!("Unregistered client {}", client_id);

            if let Some(ref tracing) = self.tracing {
                tracing.trace_session_operation("client_unregister", client_id);
            }
        }

        removed
    }

    /// Mark a client as inactive
    pub fn deactivate_client(&self, client_id: &str) {
        if let Some(client) = self.clients.write().get_mut(client_id) {
            client.active = false;
            debug!("Deactivated client {}", client_id);
        }
    }

    /// Route a message to destinations
    ///
    /// # Errors
    ///
    /// Returns an error if message routing fails
    #[instrument(level = "trace", skip(self, message))]
    pub async fn route_message(
        &self,
        message: IOPubMessage,
        destination: MessageDestination,
    ) -> Result<()> {
        // Store in history
        self.store_message(&message);

        // Route based on destination
        match destination {
            MessageDestination::Broadcast => {
                self.broadcast_message(message).await?;
            }
            MessageDestination::Client(client_id) => {
                self.send_to_client(&client_id, message).await?;
            }
            MessageDestination::Requester => {
                self.send_to_requester(message).await?;
            }
        }

        Ok(())
    }

    /// Broadcast message to all active clients
    async fn broadcast_message(&self, message: IOPubMessage) -> Result<()> {
        let active_clients: Vec<_> = {
            let clients = self.clients.read();
            clients.values().filter(|c| c.active).cloned().collect()
        };

        let mut sent = 0;
        let mut failed = 0;

        for client in active_clients {
            if client.iopub_sender.send(message.clone()).await.is_err() {
                warn!("Failed to send to client {}, marking inactive", client.id);
                self.deactivate_client(&client.id);
                failed += 1;
            } else {
                sent += 1;
            }
        }

        debug!("Broadcast message to {} clients ({} failed)", sent, failed);

        if let Some(ref tracing) = self.tracing {
            tracing.trace_transport_operation("jupyter", "iopub", &format!("broadcast_{sent}"));
        }

        Ok(())
    }

    /// Send message to specific client
    async fn send_to_client(&self, client_id: &str, message: IOPubMessage) -> Result<()> {
        let client = {
            let clients = self.clients.read();
            clients.get(client_id).cloned()
        };

        if let Some(client) = client {
            if !client.active {
                return Err(anyhow::anyhow!("Client {} is inactive", client_id));
            }

            if client.iopub_sender.send(message).await.is_err() {
                warn!("Failed to send to client {}, marking inactive", client_id);
                self.deactivate_client(client_id);
                return Err(anyhow::anyhow!("Failed to send to client {}", client_id));
            }

            debug!("Sent message to client {}", client_id);
        } else {
            return Err(anyhow::anyhow!("Client {} not found", client_id));
        }

        Ok(())
    }

    /// Send message to original requester using parent header
    async fn send_to_requester(&self, message: IOPubMessage) -> Result<()> {
        if let Some(ref parent_header) = message.parent_header {
            // Find client with matching session
            let matching_clients: Vec<_> = {
                let clients = self.clients.read();
                clients
                    .values()
                    .filter(|c| c.active && c.session_id == parent_header.session)
                    .cloned()
                    .collect()
            };

            if matching_clients.is_empty() {
                debug!(
                    "No active clients found for session {}",
                    parent_header.session
                );
                return Ok(());
            }

            let num_clients = matching_clients.len();
            for client in matching_clients {
                if client.iopub_sender.send(message.clone()).await.is_err() {
                    warn!("Failed to send to requester client {}", client.id);
                }
            }

            debug!("Sent message to {} requesters", num_clients);
        } else {
            debug!("No parent header, cannot route to requester");
        }

        Ok(())
    }

    /// Store message in history
    fn store_message(&self, message: &IOPubMessage) {
        let mut history = self.message_history.write();

        history.push(message.clone());

        // Trim history if needed
        if history.len() > self.max_history {
            let to_remove = history.len() - self.max_history;
            history.drain(0..to_remove);
            debug!("Trimmed {} messages from history", to_remove);
        }
    }

    /// Replay message history to a client
    ///
    /// # Errors
    ///
    /// Returns an error if replay fails
    #[instrument(level = "debug", skip(self))]
    pub async fn replay_history(&self, client_id: &str, count: usize) -> Result<()> {
        let messages_to_send: Vec<_> = {
            let history = self.message_history.read();
            history.iter().rev().take(count).rev().cloned().collect()
        };

        debug!(
            "Replaying {} messages to client {}",
            messages_to_send.len(),
            client_id
        );

        for message in messages_to_send {
            self.send_to_client(client_id, message).await?;
        }

        Ok(())
    }

    /// Get number of active clients
    pub fn active_client_count(&self) -> usize {
        self.clients.read().values().filter(|c| c.active).count()
    }

    /// Get all client IDs
    pub fn get_client_ids(&self) -> Vec<String> {
        self.clients.read().keys().cloned().collect()
    }

    /// Clear message history
    pub fn clear_history(&self) {
        self.message_history.write().clear();
        debug!("Cleared message history");
    }

    /// Create a channel for a new client
    pub fn create_client_channel(
        &self,
        session_id: String,
    ) -> (String, mpsc::Receiver<IOPubMessage>) {
        let (tx, rx) = mpsc::channel(100);
        let client_id = self.register_client(session_id, tx);
        (client_id, rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MessageHeader;

    #[tokio::test]
    async fn test_router_creation() {
        let router = MessageRouter::new(100);
        assert_eq!(router.active_client_count(), 0);
    }

    #[tokio::test]
    async fn test_client_registration() {
        let router = MessageRouter::new(100);

        let (tx, _rx) = mpsc::channel(10);
        let client_id = router.register_client("session1".to_string(), tx);

        assert!(!client_id.is_empty());
        assert_eq!(router.active_client_count(), 1);

        // Unregister
        assert!(router.unregister_client(&client_id));
        assert_eq!(router.active_client_count(), 0);
    }

    #[tokio::test]
    async fn test_broadcast_routing() {
        let router = MessageRouter::new(100);

        // Create multiple clients
        let (_id1, mut rx1) = router.create_client_channel("session1".to_string());
        let (_id2, mut rx2) = router.create_client_channel("session2".to_string());

        // Create test message
        let message = IOPubMessage {
            parent_header: None,
            header: MessageHeader::new("test", "test-session"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        // Broadcast
        router
            .route_message(message.clone(), MessageDestination::Broadcast)
            .await
            .unwrap();

        // Both clients should receive
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        assert_eq!(msg1.header.msg_type, "test");
        assert_eq!(msg2.header.msg_type, "test");
    }

    #[tokio::test]
    async fn test_targeted_routing() {
        let router = MessageRouter::new(100);

        let (id1, mut rx1) = router.create_client_channel("session1".to_string());
        let (_id2, mut rx2) = router.create_client_channel("session2".to_string());

        let message = IOPubMessage {
            parent_header: None,
            header: MessageHeader::new("test", "test-session"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        // Send to specific client
        router
            .route_message(message, MessageDestination::Client(id1))
            .await
            .unwrap();

        // Only first client should receive
        let msg1 = rx1.recv().await.unwrap();
        assert_eq!(msg1.header.msg_type, "test");

        // Second client should not receive
        assert!(rx2.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_history_replay() {
        let router = MessageRouter::new(100);

        // Send some messages
        for i in 0..5 {
            let mut content = HashMap::new();
            content.insert("index".to_string(), serde_json::Value::Number(i.into()));

            let message = IOPubMessage {
                parent_header: None,
                header: MessageHeader::new("test", "test-session"),
                metadata: HashMap::new(),
                content,
            };

            router
                .route_message(message, MessageDestination::Broadcast)
                .await
                .unwrap();
        }

        // Register new client
        let (id, mut rx) = router.create_client_channel("session1".to_string());

        // Replay history
        router.replay_history(&id, 3).await.unwrap();

        // Should receive 3 messages
        for i in 2..5 {
            let msg = rx.recv().await.unwrap();
            let index = msg.content.get("index").unwrap().as_i64().unwrap();
            assert_eq!(index, i);
        }
    }
}
