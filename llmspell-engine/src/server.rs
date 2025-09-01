//! Server-side protocol handler
//!
//! Accepts connections and routes messages to handlers

use async_trait::async_trait;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use thiserror::Error;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tracing::{debug, error, info, warn};

use crate::engine::{
    ChannelType, ChannelView, EngineError, ProtocolAdapter, ProtocolEngine, ProtocolType,
    UniversalMessage,
};
use crate::protocol::message::{MessageHandler, ProtocolMessage};
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

    /// Port for `IOPub` channel
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
            shell_port: 9555,
            iopub_port: 9556,
            stdin_port: 9557,
            control_port: 9558,
            heartbeat_port: 9559,
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

    /// `IOPub` broadcast channel
    iopub_tx: broadcast::Sender<ProtocolMessage>,

    /// Shutdown signal
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Channel message queues for `ProtocolEngine` implementation
    channel_senders: Arc<RwLock<HashMap<ChannelType, mpsc::Sender<UniversalMessage>>>>,
    channel_receivers: Arc<RwLock<HashMap<ChannelType, mpsc::Receiver<UniversalMessage>>>>,

    /// Protocol adapters
    adapters: Arc<RwLock<HashMap<ProtocolType, Box<dyn ProtocolAdapter>>>>,
}

impl ProtocolServer {
    /// Create a new protocol server
    pub fn new(config: ServerConfig, handler: Arc<dyn MessageHandler>) -> Self {
        let (iopub_tx, _) = broadcast::channel(1024);
        let (shutdown_tx, _) = broadcast::channel(1);

        // Create channel queues for each channel type
        let mut channel_senders = HashMap::new();
        let mut channel_receivers = HashMap::new();

        for channel_type in &[
            ChannelType::Shell,
            ChannelType::IOPub,
            ChannelType::Stdin,
            ChannelType::Control,
            ChannelType::Heartbeat,
        ] {
            let (tx, rx) = mpsc::channel(100);
            channel_senders.insert(*channel_type, tx);
            channel_receivers.insert(*channel_type, rx);
        }

        Self {
            config,
            handler,
            clients: Arc::new(RwLock::new(HashMap::new())),
            iopub_tx,
            shutdown_tx: Some(shutdown_tx),
            channel_senders: Arc::new(RwLock::new(channel_senders)),
            channel_receivers: Arc::new(RwLock::new(channel_receivers)),
            adapters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start the server and accept connections
    ///
    /// # Errors
    ///
    /// Returns `ServerError::Io` if binding to ports fails
    ///
    /// # Panics
    ///
    /// Panics if shutdown channel is not initialized
    pub async fn start(&mut self) -> Result<(), ServerError> {
        info!(
            "Starting protocol server on {}:{}",
            self.config.ip, self.config.shell_port
        );

        // Set up listeners
        let (shell_listener, iopub_listener) = self.bind_listeners().await?;

        // Run the accept loop
        self.run_accept_loop(shell_listener, iopub_listener).await
    }

    /// Bind TCP listeners for shell and `IOPub` channels
    async fn bind_listeners(&self) -> Result<(TcpListener, TcpListener), ServerError> {
        let shell_addr = format!("{}:{}", self.config.ip, self.config.shell_port);
        let shell_listener = TcpListener::bind(&shell_addr).await?;

        let iopub_addr = format!("{}:{}", self.config.ip, self.config.iopub_port);
        let iopub_listener = TcpListener::bind(&iopub_addr).await?;

        info!(
            "Protocol server listening on shell: {}, iopub: {}",
            shell_addr, iopub_addr
        );

        Ok((shell_listener, iopub_listener))
    }

    /// Run the main accept loop for incoming connections
    async fn run_accept_loop(
        &self,
        shell_listener: TcpListener,
        iopub_listener: TcpListener,
    ) -> Result<(), ServerError> {
        let mut shutdown_rx = self.shutdown_tx.as_ref().unwrap().subscribe();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    info!("Protocol server shutting down");
                    break;
                }
                result = shell_listener.accept() => {
                    self.handle_accept_result(result, true).await;
                }
                result = iopub_listener.accept() => {
                    self.handle_accept_result(result, false).await;
                }
            }
        }

        Ok(())
    }

    /// Handle the result of accepting a connection
    async fn handle_accept_result(
        &self,
        result: std::io::Result<(TcpStream, SocketAddr)>,
        is_shell: bool,
    ) {
        match result {
            Ok((stream, addr)) => self.handle_new_connection(stream, addr, is_shell).await,
            Err(e) => Self::log_accept_error(&e, is_shell),
        }
    }

    /// Handle a new connection
    async fn handle_new_connection(&self, stream: TcpStream, addr: SocketAddr, is_shell: bool) {
        if is_shell {
            info!("New shell connection from {}", addr);
            self.handle_shell_connection(stream, addr).await;
        } else {
            info!("New IOPub connection from {}", addr);
            self.handle_iopub_connection(stream, addr);
        }
    }

    /// Log an error accepting a connection
    fn log_accept_error(error: &std::io::Error, is_shell: bool) {
        let channel = if is_shell { "shell" } else { "IOPub" };
        error!("Error accepting {} connection: {}", channel, error);
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
            clients.write().await.remove(&client_id);
            info!("Client {} disconnected", client_id);
        });
    }

    /// Handle an `IOPub` channel connection
    fn handle_iopub_connection(&self, stream: TcpStream, addr: SocketAddr) {
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
            let request = match Self::receive_message(&mut transport, &client_id).await {
                Ok(Some(msg)) => msg,
                Ok(None) => break, // Connection closed or error
                Err(e) => return Err(e),
            };

            debug!("Received request from {}: {:?}", client_id, request.msg_id);

            // Handle request and send response
            if let Some(response) = handler.handle(request.clone()).await {
                if !Self::send_response(&mut transport, &client_id, response.clone(), &iopub_tx)
                    .await
                {
                    break; // Error sending response
                }
            }
        }

        Ok(())
    }

    /// Receive a message from the transport
    async fn receive_message(
        transport: &mut Box<dyn Transport>,
        client_id: &str,
    ) -> Result<Option<ProtocolMessage>, ServerError> {
        match transport.recv().await {
            Ok(msg) => Ok(Some(msg)),
            Err(TransportError::ConnectionClosed) => {
                debug!("Client {} connection closed", client_id);
                Ok(None)
            }
            Err(e) => {
                error!("Error receiving from client {}: {}", client_id, e);
                Ok(None)
            }
        }
    }

    /// Send a response and broadcast to `IOPub` if needed
    async fn send_response(
        transport: &mut Box<dyn Transport>,
        client_id: &str,
        response: ProtocolMessage,
        iopub_tx: &broadcast::Sender<ProtocolMessage>,
    ) -> bool {
        // Send response
        if let Err(e) = transport.send(response.clone()).await {
            error!("Error sending response to {}: {}", client_id, e);
            return false;
        }

        // Broadcast on IOPub if needed
        if response.channel == "iopub" {
            let _ = iopub_tx.send(response);
        }

        true
    }

    /// Shutdown the server
    pub fn shutdown(mut self) {
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

/// Implement `ProtocolEngine` trait for `ProtocolServer`
/// This allows using `ChannelSet` with the existing server infrastructure
#[async_trait]
impl ProtocolEngine for ProtocolServer {
    async fn register_adapter(
        &mut self,
        protocol: ProtocolType,
        adapter: Box<dyn ProtocolAdapter>,
    ) -> Result<(), EngineError> {
        self.adapters.write().await.insert(protocol, adapter);
        Ok(())
    }

    async fn send(&self, channel: ChannelType, msg: UniversalMessage) -> Result<(), EngineError> {
        // Special handling for IOPub channel - broadcast
        if channel == ChannelType::IOPub {
            // Convert UniversalMessage to ProtocolMessage for IOPub
            let protocol_msg = ProtocolMessage {
                msg_id: msg.id.clone(),
                msg_type: crate::protocol::message::MessageType::Request,
                channel: format!("{channel:?}").to_lowercase(),
                content: serde_json::to_value(&msg.content)
                    .map_err(|e| EngineError::Conversion(e.to_string()))?,
            };

            // Broadcast to IOPub subscribers (ignore if no subscribers)
            let _ = self.iopub_tx.send(protocol_msg);
        }

        // Queue the message for all channels (including IOPub for local delivery)
        let senders = self.channel_senders.read().await;
        if let Some(sender) = senders.get(&channel) {
            sender
                .send(msg)
                .await
                .map_err(|_| EngineError::ChannelNotFound(format!("Channel {channel:?} closed")))?;
        } else {
            return Err(EngineError::ChannelNotFound(format!(
                "Channel {channel:?} not found"
            )));
        }
        drop(senders);

        Ok(())
    }

    async fn recv(&self, channel: ChannelType) -> Result<UniversalMessage, EngineError> {
        // Receive from the channel queue
        let mut receivers = self.channel_receivers.write().await;
        if let Some(receiver) = receivers.get_mut(&channel) {
            receiver
                .recv()
                .await
                .ok_or_else(|| EngineError::ChannelNotFound(format!("Channel {channel:?} closed")))
        } else {
            Err(EngineError::ChannelNotFound(format!(
                "Channel {channel:?} not found"
            )))
        }
    }

    fn channel_view(&self, channel: ChannelType) -> ChannelView<'_> {
        ChannelView::new(self, channel)
    }
}
