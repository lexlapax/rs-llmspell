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
use tracing::field::Empty;
use tracing::{debug, info, instrument, trace, warn, Span};
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
    /// Current correlation ID for message tracking
    correlation_id: Arc<RwLock<Option<Uuid>>>,
    /// Track which client sent which message (`msg_id` -> `client_id`)
    message_origins: Arc<RwLock<HashMap<String, String>>>,
}

impl MessageRouter {
    /// Create a new message router
    #[instrument(level = "debug", fields(router_id = Empty))]
    pub fn new(max_history: usize) -> Self {
        let router_id = Uuid::new_v4();
        Span::current().record("router_id", router_id.to_string());

        info!("Creating MessageRouter with max_history={}", max_history);

        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::with_capacity(max_history))),
            max_history,
            tracing: None,
            correlation_id: Arc::new(RwLock::new(None)),
            message_origins: Arc::new(RwLock::new(HashMap::new())),
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

            // Clean up message origins for this client
            let mut origins = self.message_origins.write();
            origins.retain(|_, v| v != client_id);

            if let Some(ref tracing) = self.tracing {
                tracing.trace_session_operation("client_unregister", client_id);
            }
        }

        removed
    }

    /// Track that a message originated from a specific client
    #[instrument(level = "trace", skip(self))]
    pub fn track_message_origin(&self, msg_id: &str, client_id: &str) {
        self.message_origins
            .write()
            .insert(msg_id.to_string(), client_id.to_string());
        trace!("Tracked message {} from client {}", msg_id, client_id);
    }

    /// Mark a client as inactive
    #[instrument(level = "debug", skip(self))]
    pub fn deactivate_client(&self, client_id: &str) {
        if let Some(client) = self.clients.write().get_mut(client_id) {
            client.active = false;
            info!("Deactivated client {}", client_id);
        } else {
            warn!("Attempted to deactivate non-existent client {}", client_id);
        }
    }

    /// Route a message to destinations
    ///
    /// # Errors
    ///
    /// Returns an error if message routing fails
    #[instrument(level = "trace", skip(self, message), fields(
        correlation_id = Empty,
        destination = ?destination,
        message_type = Empty
    ))]
    pub async fn route_message(
        &self,
        message: IOPubMessage,
        destination: MessageDestination,
    ) -> Result<()> {
        // Generate or use existing correlation ID
        let correlation_id = self.get_or_create_correlation_id();
        Span::current().record("correlation_id", correlation_id.to_string());

        trace!("Routing message with correlation_id={}", correlation_id);

        // Store in history
        self.store_message(&message);

        // Route based on destination
        match destination {
            MessageDestination::Broadcast => {
                debug!("Broadcasting message to all clients");
                self.broadcast_message(message).await?;
            }
            MessageDestination::Client(ref client_id) => {
                debug!("Sending message to client {}", client_id);
                self.send_to_client(client_id, message).await?;
            }
            MessageDestination::Requester => {
                debug!("Sending message to original requester");
                self.send_to_requester(message).await?;
            }
        }

        Ok(())
    }

    /// Get or create a correlation ID for message tracking
    fn get_or_create_correlation_id(&self) -> Uuid {
        let mut correlation_id = self.correlation_id.write();
        if let Some(id) = *correlation_id {
            id
        } else {
            let id = Uuid::new_v4();
            *correlation_id = Some(id);
            id
        }
    }

    /// Set a new correlation ID for a new message flow
    #[instrument(level = "debug", skip(self))]
    pub fn set_correlation_id(&self, id: Option<Uuid>) {
        *self.correlation_id.write() = id;
        if let Some(id) = id {
            debug!("Set correlation_id={}", id);
        } else {
            debug!("Cleared correlation_id");
        }
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
                return Err(anyhow::anyhow!("Client {client_id} is inactive"));
            }

            if client.iopub_sender.send(message).await.is_err() {
                warn!("Failed to send to client {}, marking inactive", client_id);
                self.deactivate_client(client_id);
                return Err(anyhow::anyhow!("Failed to send to client {client_id}"));
            }

            debug!("Sent message to client {}", client_id);
        } else {
            return Err(anyhow::anyhow!("Client {client_id} not found"));
        }

        Ok(())
    }

    /// Send message to original requester using parent header
    async fn send_to_requester(&self, message: IOPubMessage) -> Result<()> {
        if let Some(ref parent_header) = message.parent_header {
            // First, try to find the specific client that sent this message
            let client_id = self
                .message_origins
                .read()
                .get(&parent_header.msg_id)
                .cloned();

            if let Some(client_id) = client_id {
                // Send to the specific client that originated the request
                debug!("Routing message to originating client {}", client_id);
                return self.send_to_client(&client_id, message).await;
            }

            // Fallback: Find clients with matching session
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

    #[tokio::test]
    async fn test_message_origin_tracking() {
        let router = MessageRouter::new(100);

        let (id1, mut rx1) = router.create_client_channel("session1".to_string());
        let (_id2, mut rx2) = router.create_client_channel("session2".to_string());

        // Track that a message originated from client 1
        let msg_id = "test-msg-123";
        router.track_message_origin(msg_id, &id1);

        // Create a response message with parent header pointing to that message
        let mut parent_header = MessageHeader::new("execute_request", "session1");
        parent_header.msg_id = msg_id.to_string();

        let response = IOPubMessage {
            parent_header: Some(parent_header),
            header: MessageHeader::new("execute_result", "session1"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        // Route to requester - should go only to client 1
        router
            .route_message(response, MessageDestination::Requester)
            .await
            .unwrap();

        // Only client 1 should receive the message
        let msg1 = rx1.recv().await.unwrap();
        assert_eq!(msg1.header.msg_type, "execute_result");

        // Client 2 should not receive
        assert!(rx2.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_concurrent_client_requests() {
        let router = MessageRouter::new(100);

        let (id1, mut rx1) = router.create_client_channel("session1".to_string());
        let (id2, mut rx2) = router.create_client_channel("session2".to_string());

        // Track two different requests from different clients
        router.track_message_origin("msg-from-client1", &id1);
        router.track_message_origin("msg-from-client2", &id2);

        // Create responses for both requests
        let mut parent1 = MessageHeader::new("execute_request", "session1");
        parent1.msg_id = "msg-from-client1".to_string();

        let mut parent2 = MessageHeader::new("execute_request", "session2");
        parent2.msg_id = "msg-from-client2".to_string();

        let response1 = IOPubMessage {
            parent_header: Some(parent1),
            header: MessageHeader::new("execute_result", "session1"),
            metadata: HashMap::new(),
            content: {
                let mut content = HashMap::new();
                content.insert("client".to_string(), serde_json::Value::Number(1.into()));
                content
            },
        };

        let response2 = IOPubMessage {
            parent_header: Some(parent2),
            header: MessageHeader::new("execute_result", "session2"),
            metadata: HashMap::new(),
            content: {
                let mut content = HashMap::new();
                content.insert("client".to_string(), serde_json::Value::Number(2.into()));
                content
            },
        };

        // Route both responses
        router
            .route_message(response1, MessageDestination::Requester)
            .await
            .unwrap();
        router
            .route_message(response2, MessageDestination::Requester)
            .await
            .unwrap();

        // Each client should receive only their response
        let msg1 = rx1.recv().await.unwrap();
        assert_eq!(msg1.content.get("client").unwrap().as_i64().unwrap(), 1);

        let msg2 = rx2.recv().await.unwrap();
        assert_eq!(msg2.content.get("client").unwrap().as_i64().unwrap(), 2);

        // No cross-talk
        assert!(rx1.try_recv().is_err());
        assert!(rx2.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_client_cleanup_on_unregister() {
        let router = MessageRouter::new(100);

        let (id1, _rx1) = router.create_client_channel("session1".to_string());

        // Track some message origins
        router.track_message_origin("msg1", &id1);
        router.track_message_origin("msg2", &id1);

        // Verify client is registered
        assert_eq!(router.active_client_count(), 1);

        // Unregister client
        assert!(router.unregister_client(&id1));

        // Client should be gone
        assert_eq!(router.active_client_count(), 0);

        // Message origins should be cleaned up (indirectly verify through behavior)
        // After cleanup, routing to requester should not crash or route to non-existent client
        let mut parent = MessageHeader::new("execute_request", "session1");
        parent.msg_id = "msg1".to_string();

        let message = IOPubMessage {
            parent_header: Some(parent),
            header: MessageHeader::new("execute_result", "session1"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        // Should not error even though client is gone
        router
            .route_message(message, MessageDestination::Requester)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_client_deactivation() {
        let router = MessageRouter::new(100);

        let (id1, mut rx1) = router.create_client_channel("session1".to_string());

        assert_eq!(router.active_client_count(), 1);

        // Deactivate client
        router.deactivate_client(&id1);

        // Client should be marked inactive but still registered
        assert_eq!(router.active_client_count(), 0);
        assert!(router.get_client_ids().contains(&id1));

        // Broadcast should skip inactive client
        let message = IOPubMessage {
            parent_header: None,
            header: MessageHeader::new("status", "session1"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        router
            .route_message(message.clone(), MessageDestination::Broadcast)
            .await
            .unwrap();

        // Inactive client should not receive
        assert!(rx1.try_recv().is_err());

        // Direct send to inactive client should fail
        let result = router
            .route_message(message, MessageDestination::Client(id1))
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_correlation_id_tracking() {
        let router = MessageRouter::new(100);

        // Initially no correlation ID
        assert!(router.correlation_id.read().is_none());

        // Create correlation ID on first message
        let id1 = router.get_or_create_correlation_id();
        assert!(router.correlation_id.read().is_some());

        // Same ID returned on subsequent calls
        let id2 = router.get_or_create_correlation_id();
        assert_eq!(id1, id2);

        // Can set new correlation ID
        let new_id = Uuid::new_v4();
        router.set_correlation_id(Some(new_id));
        let id3 = router.get_or_create_correlation_id();
        assert_eq!(id3, new_id);

        // Can clear correlation ID
        router.set_correlation_id(None);
        assert!(router.correlation_id.read().is_none());
    }

    #[tokio::test]
    async fn test_message_history_trimming() {
        let max_history = 5;
        let router = MessageRouter::new(max_history);

        // Send more messages than max_history
        for i in 0..10 {
            let mut content = HashMap::new();
            content.insert("index".to_string(), serde_json::Value::Number(i.into()));

            let message = IOPubMessage {
                parent_header: None,
                header: MessageHeader::new("test", "session"),
                metadata: HashMap::new(),
                content,
            };

            router
                .route_message(message, MessageDestination::Broadcast)
                .await
                .unwrap();
        }

        // Create a new client and replay history
        let (id, mut rx) = router.create_client_channel("session".to_string());
        router.replay_history(&id, max_history).await.unwrap();

        // Should only receive the last max_history messages (5-9)
        for i in 5..10 {
            let msg = rx.recv().await.unwrap();
            let index = msg.content.get("index").unwrap().as_i64().unwrap();
            assert_eq!(index, i);
        }

        // No more messages
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_session_fallback_routing() {
        let router = MessageRouter::new(100);

        // Create two clients with same session
        let (_id1, mut rx1) = router.create_client_channel("shared-session".to_string());
        let (_id2, mut rx2) = router.create_client_channel("shared-session".to_string());

        // Don't track message origin, rely on session fallback
        let mut parent = MessageHeader::new("execute_request", "shared-session");
        parent.msg_id = "unknown-msg".to_string();

        let message = IOPubMessage {
            parent_header: Some(parent),
            header: MessageHeader::new("execute_result", "shared-session"),
            metadata: HashMap::new(),
            content: HashMap::new(),
        };

        // Route to requester should fall back to session matching
        router
            .route_message(message, MessageDestination::Requester)
            .await
            .unwrap();

        // Both clients with same session should receive
        let msg1 = rx1.recv().await.unwrap();
        let msg2 = rx2.recv().await.unwrap();

        assert_eq!(msg1.header.msg_type, "execute_result");
        assert_eq!(msg2.header.msg_type, "execute_result");
    }

    #[tokio::test]
    async fn test_clear_history() {
        let router = MessageRouter::new(100);

        // Add some messages to history
        for i in 0..5 {
            let mut content = HashMap::new();
            content.insert("index".to_string(), serde_json::Value::Number(i.into()));

            let message = IOPubMessage {
                parent_header: None,
                header: MessageHeader::new("test", "session"),
                metadata: HashMap::new(),
                content,
            };

            router
                .route_message(message, MessageDestination::Broadcast)
                .await
                .unwrap();
        }

        // Clear history
        router.clear_history();

        // Create a new client and replay history
        let (id, mut rx) = router.create_client_channel("session".to_string());
        router.replay_history(&id, 10).await.unwrap();

        // Should receive no messages
        assert!(rx.try_recv().is_err());
    }
}
