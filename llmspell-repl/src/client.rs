//! Client connection handling
//!
//! Manages individual client connections to the kernel.

use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

/// Represents a connected client to the kernel
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    /// Unique client identifier
    pub client_id: String,
    /// Client session ID
    pub session_id: String,
    /// Username associated with the client
    pub username: String,
    /// Client connection timestamp
    pub connected_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: Arc<RwLock<DateTime<Utc>>>,
    /// Client capabilities
    pub capabilities: ClientCapabilities,
    /// Client state
    pub state: Arc<RwLock<ClientState>>,
}

/// Client capabilities
#[derive(Debug, Clone, Default)]
pub struct ClientCapabilities {
    /// Whether client supports debugging
    pub debug_support: bool,
    /// Whether client supports media messages
    pub media_support: bool,
    /// Whether client supports streaming
    pub streaming_support: bool,
    /// Whether client supports hot reload
    pub hot_reload_support: bool,
    /// Protocol version
    pub protocol_version: String,
}

/// Client state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientState {
    /// Client is connected and idle
    Connected,
    /// Client is executing code
    Executing,
    /// Client is in debug mode
    Debugging,
    /// Client is disconnecting
    Disconnecting,
}

impl ConnectedClient {
    /// Create a new connected client
    pub fn new(username: String) -> Self {
        let now = Utc::now();
        Self {
            client_id: Uuid::new_v4().to_string(),
            session_id: Uuid::new_v4().to_string(),
            username,
            connected_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            capabilities: ClientCapabilities::default(),
            state: Arc::new(RwLock::new(ClientState::Connected)),
        }
    }
    
    /// Create a new connected client with specific ID
    pub fn with_id(client_id: String, username: String) -> Self {
        let now = Utc::now();
        Self {
            client_id,
            session_id: Uuid::new_v4().to_string(),
            username,
            connected_at: now,
            last_activity: Arc::new(RwLock::new(now)),
            capabilities: ClientCapabilities::default(),
            state: Arc::new(RwLock::new(ClientState::Connected)),
        }
    }
    
    /// Update last activity timestamp
    pub async fn update_activity(&self) {
        let mut last = self.last_activity.write().await;
        *last = Utc::now();
    }
    
    /// Get idle duration
    pub async fn idle_duration(&self) -> chrono::Duration {
        let last = self.last_activity.read().await;
        Utc::now() - *last
    }
    
    /// Set client state
    pub async fn set_state(&self, state: ClientState) {
        let mut current = self.state.write().await;
        *current = state;
        self.update_activity().await;
    }
    
    /// Get client state
    pub async fn get_state(&self) -> ClientState {
        self.state.read().await.clone()
    }
    
    /// Check if client is active (not idle for more than timeout)
    pub async fn is_active(&self, timeout_secs: i64) -> bool {
        self.idle_duration().await.num_seconds() < timeout_secs
    }
}

/// Client manager for tracking all connected clients
pub struct ClientManager {
    /// Map of client ID to client
    clients: Arc<RwLock<std::collections::HashMap<String, ConnectedClient>>>,
    /// Maximum number of clients
    max_clients: usize,
}

impl ClientManager {
    /// Create a new client manager
    pub fn new(max_clients: usize) -> Self {
        Self {
            clients: Arc::new(RwLock::new(std::collections::HashMap::new())),
            max_clients,
        }
    }
    
    /// Add a new client
    pub async fn add_client(&self, client: ConnectedClient) -> Result<()> {
        let mut clients = self.clients.write().await;
        
        if clients.len() >= self.max_clients {
            anyhow::bail!("Maximum number of clients ({}) reached", self.max_clients);
        }
        
        let client_id = client.client_id.clone();
        clients.insert(client_id.clone(), client);
        
        tracing::info!("Client {} added", client_id);
        Ok(())
    }
    
    /// Remove a client
    pub async fn remove_client(&self, client_id: &str) -> Option<ConnectedClient> {
        let mut clients = self.clients.write().await;
        let client = clients.remove(client_id);
        
        if client.is_some() {
            tracing::info!("Client {} removed", client_id);
        }
        
        client
    }
    
    /// Get a client by ID
    pub async fn get_client(&self, client_id: &str) -> Option<ConnectedClient> {
        let clients = self.clients.read().await;
        clients.get(client_id).cloned()
    }
    
    /// Get all clients
    pub async fn get_all_clients(&self) -> Vec<ConnectedClient> {
        let clients = self.clients.read().await;
        clients.values().cloned().collect()
    }
    
    /// Get active clients
    pub async fn get_active_clients(&self, timeout_secs: i64) -> Vec<ConnectedClient> {
        let clients = self.clients.read().await;
        let mut active = Vec::new();
        
        for client in clients.values() {
            if client.is_active(timeout_secs).await {
                active.push(client.clone());
            }
        }
        
        active
    }
    
    /// Clean up inactive clients
    pub async fn cleanup_inactive(&self, timeout_secs: i64) -> Vec<String> {
        let clients = self.clients.read().await;
        let mut to_remove = Vec::new();
        
        for (id, client) in clients.iter() {
            if !client.is_active(timeout_secs).await {
                to_remove.push(id.clone());
            }
        }
        
        drop(clients);
        
        for id in &to_remove {
            self.remove_client(id).await;
        }
        
        to_remove
    }
}