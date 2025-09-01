//! Server-side protocol handler
//!
//! Accepts connections and routes messages to handlers

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::message::{MessageHandler, ProtocolMessage};
use crate::transport::tcp::TcpTransport;
use crate::transport::{Transport, TransportError};

/// Server-side protocol errors
#[derive(Error, Debug)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Bind error: {0}")]
    Bind(String),

    #[error("Handler error: {0}")]
    Handler(String),
}

/// Connected client information
#[derive(Debug, Clone)]
pub struct ConnectedClient {
    pub id: String,
    pub addr: SocketAddr,
    pub channels: Vec<String>,
}

/// Protocol server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// IP address to bind to
    pub ip: String,

    /// Port for shell channel
    pub shell_port: u16,

    /// Port for IOPub channel
    pub iopub_port: u16,

    /// Port for stdin channel
    pub stdin_port: u16,

    /// Port for control channel
    pub control_port: u16,

    /// Port for heartbeat channel
    pub heartbeat_port: u16,

    /// Maximum number of clients
    pub max_clients: usize,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            ip: "127.0.0.1".to_string(),
            shell_port: 5555,
            iopub_port: 5556,
            stdin_port: 5557,
            control_port: 5558,
            heartbeat_port: 5559,
            max_clients: 10,
        }
    }
}

/// Protocol server that accepts connections and routes messages
pub struct ProtocolServer {
    /// Server configuration
    config: ServerConfig,

    /// Message handler
    handler: Arc<dyn MessageHandler>,

    /// Connected clients
    clients: Arc<RwLock<HashMap<String, ConnectedClient>>>,

    /// IOPub broadcast channel
    iopub_tx: broadcast::Sender<ProtocolMessage>,

    /// Shutdown signal
    shutdown_tx: Option<broadcast::Sender<()>>,
}

impl ProtocolServer {
    /// Create a new protocol server
    pub fn new(config: ServerConfig, handler: Arc<dyn MessageHandler>) -> Self {
        let (iopub_tx, _) = broadcast::channel(1024);
        let (shutdown_tx, _) = broadcast::channel(1);

        Self {
            config,
            handler,
            clients: Arc::new(RwLock::new(HashMap::new())),
            iopub_tx,
            shutdown_tx: Some(shutdown_tx),
        }
    }

    /// Start the server and accept connections
    pub async fn start(&mut self) -> Result<(), ServerError> {
        info!(
            "Starting protocol server on {}:{}",
            self.config.ip, self.config.shell_port
        );

        // Bind to shell channel (main request/response channel)
        let shell_addr = format!("{}:{}", self.config.ip, self.config.shell_port);
        let shell_listener = TcpListener::bind(&shell_addr).await?;

        // Bind to IOPub channel (broadcast channel)
        let iopub_addr = format!("{}:{}", self.config.ip, self.config.iopub_port);
        let iopub_listener = TcpListener::bind(&iopub_addr).await?;

        info!(
            "Protocol server listening on shell: {}, iopub: {}",
            shell_addr, iopub_addr
        );

        // Accept loop
        let mut shutdown_rx = self.shutdown_tx.as_ref().unwrap().subscribe();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Protocol server shutting down");
                    break;
                }

                // Accept shell connections
                result = shell_listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New shell connection from {}", addr);
                            self.handle_shell_connection(stream, addr).await;
                        }
                        Err(e) => {
                            error!("Error accepting shell connection: {}", e);
                        }
                    }
                }

                // Accept IOPub connections
                result = iopub_listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            info!("New IOPub connection from {}", addr);
                            self.handle_iopub_connection(stream, addr).await;
                        }
                        Err(e) => {
                            error!("Error accepting IOPub connection: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle a shell channel connection
    async fn handle_shell_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let client_id = uuid::Uuid::new_v4().to_string();
        let transport = Box::new(TcpTransport::new(stream));

        // Add client
        {
            let mut clients = self.clients.write().await;
            clients.insert(
                client_id.clone(),
                ConnectedClient {
                    id: client_id.clone(),
                    addr,
                    channels: vec!["shell".to_string()],
                },
            );
        }

        // Spawn handler task
        let handler = self.handler.clone();
        let clients = self.clients.clone();
        let iopub_tx = self.iopub_tx.clone();

        tokio::spawn(async move {
            if let Err(e) =
                Self::handle_client(client_id.clone(), transport, handler, iopub_tx).await
            {
                error!("Error handling client {}: {}", client_id, e);
            }

            // Remove client on disconnect
            let mut clients = clients.write().await;
            clients.remove(&client_id);
            info!("Client {} disconnected", client_id);
        });
    }

    /// Handle an IOPub channel connection
    async fn handle_iopub_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let mut transport = TcpTransport::new(stream);
        let mut iopub_rx = self.iopub_tx.subscribe();

        // Forward IOPub messages to client
        tokio::spawn(async move {
            while let Ok(msg) = iopub_rx.recv().await {
                if let Err(e) = transport.send(msg).await {
                    warn!("Error sending IOPub message to {}: {}", addr, e);
                    break;
                }
            }
        });
    }

    /// Handle messages from a connected client
    async fn handle_client(
        client_id: String,
        mut transport: Box<dyn Transport>,
        handler: Arc<dyn MessageHandler>,
        iopub_tx: broadcast::Sender<ProtocolMessage>,
    ) -> Result<(), ServerError> {
        loop {
            // Receive request
            let request = match transport.recv().await {
                Ok(msg) => msg,
                Err(TransportError::ConnectionClosed) => {
                    debug!("Client {} connection closed", client_id);
                    break;
                }
                Err(e) => {
                    error!("Error receiving from client {}: {}", client_id, e);
                    break;
                }
            };

            debug!("Received request from {}: {:?}", client_id, request.msg_id);

            // Handle request
            if let Some(response) = handler.handle(request.clone()).await {
                // Send response
                if let Err(e) = transport.send(response.clone()).await {
                    error!("Error sending response to {}: {}", client_id, e);
                    break;
                }

                // Broadcast on IOPub if needed
                if response.channel == "iopub" {
                    let _ = iopub_tx.send(response);
                }
            }
        }

        Ok(())
    }

    /// Shutdown the server
    pub async fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}
