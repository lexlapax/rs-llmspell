//! Protocol Engine - Unified protocol handling abstraction
//!
//! Provides a unified engine for handling multiple protocols through adapters,
//! enabling protocol bridging and intelligent message routing.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

use crate::protocol::message::ProtocolMessage;
use crate::transport::{Transport, TransportError};

/// Protocol engine errors
#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Transport error: {0}")]
    Transport(#[from] TransportError),

    #[error("Adapter error: {0}")]
    Adapter(String),

    #[error("Routing error: {0}")]
    Routing(String),

    #[error("Channel not found: {0}")]
    ChannelNotFound(String),

    #[error("Protocol not supported: {0:?}")]
    ProtocolNotSupported(ProtocolType),

    #[error("Message conversion error: {0}")]
    Conversion(String),
}

/// Supported protocol types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolType {
    /// `LLMSpell` REPL Protocol
    LRP,
    /// `LLMSpell` Debug Protocol
    LDP,
    /// Model Context Protocol (future)
    MCP,
    /// Language Server Protocol (future)
    LSP,
    /// Debug Adapter Protocol (future)
    DAP,
    /// Agent-to-Agent protocol (future)
    A2A,
}

/// Channel types for message routing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelType {
    /// Shell channel for request/reply
    Shell,
    /// `IOPub` channel for broadcast messages
    IOPub,
    /// Stdin channel for input requests
    Stdin,
    /// Control channel for control messages
    Control,
    /// Heartbeat channel for liveness checks
    Heartbeat,
}

/// Protocol capabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Can handle request/reply patterns
    RequestReply,
    /// Can handle publish/subscribe patterns
    PubSub,
    /// Can handle streaming data
    Streaming,
    /// Can handle binary data
    Binary,
    /// Can handle control messages
    Control,
    /// Can handle heartbeat/keepalive
    Heartbeat,
}

/// Universal message format for cross-protocol compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalMessage {
    /// Unique message identifier
    pub id: String,
    /// Source protocol type
    pub protocol: ProtocolType,
    /// Target channel
    pub channel: ChannelType,
    /// Message content
    pub content: MessageContent,
    /// Protocol-specific metadata
    pub metadata: HashMap<String, Value>,
}

/// Message content variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum MessageContent {
    /// Request message
    Request { method: String, params: Value },
    /// Response message
    Response {
        result: Option<Value>,
        error: Option<Value>,
    },
    /// Notification message
    Notification { event: String, data: Value },
    /// Raw data message
    Raw { data: Vec<u8> },
}

/// Protocol adapter trait for pluggable protocols
#[async_trait]
pub trait ProtocolAdapter: Send + Sync {
    /// Get the protocol type this adapter handles
    fn protocol_type(&self) -> ProtocolType;

    /// Convert inbound raw message to universal format
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the message cannot be parsed or converted
    fn adapt_inbound(&self, raw: &[u8]) -> Result<UniversalMessage, EngineError>;

    /// Convert universal message to outbound raw format
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the message cannot be serialized
    fn adapt_outbound(&self, msg: &UniversalMessage) -> Result<Vec<u8>, EngineError>;

    /// Get capabilities supported by this protocol
    fn capabilities(&self) -> HashSet<Capability>;
}

/// Message routing strategy
#[derive(Debug, Clone)]
pub enum RoutingStrategy {
    /// Direct routing to specific handler
    Direct,
    /// Broadcast to all handlers
    Broadcast,
    /// Round-robin among handlers
    RoundRobin,
    /// Load-balanced routing
    LoadBalanced,
}

/// Message router for intelligent routing
pub struct MessageRouter {
    /// Routing table
    routes: Arc<RwLock<RouteTable>>,
    /// Channel-specific routing strategies
    strategies: HashMap<ChannelType, RoutingStrategy>,
}

/// Routing table for message dispatch
pub struct RouteTable {
    /// Protocol to handler mappings
    handlers: HashMap<(ProtocolType, ChannelType), Vec<String>>,
    /// Active handler registry
    active_handlers: HashSet<String>,
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageRouter {
    /// Create a new message router
    #[must_use]
    pub fn new() -> Self {
        let mut strategies = HashMap::new();
        strategies.insert(ChannelType::Shell, RoutingStrategy::Direct);
        strategies.insert(ChannelType::IOPub, RoutingStrategy::Broadcast);
        strategies.insert(ChannelType::Stdin, RoutingStrategy::Direct);
        strategies.insert(ChannelType::Control, RoutingStrategy::Direct);
        strategies.insert(ChannelType::Heartbeat, RoutingStrategy::Direct);

        Self {
            routes: Arc::new(RwLock::new(RouteTable {
                handlers: HashMap::new(),
                active_handlers: HashSet::new(),
            })),
            strategies,
        }
    }

    /// Route a message to appropriate handlers
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Routing` if no handlers are registered for the protocol/channel
    pub async fn route(&self, msg: &UniversalMessage) -> Result<Vec<String>, EngineError> {
        let key = (msg.protocol, msg.channel);

        let routes = self.routes.read().await;
        let handlers = routes
            .handlers
            .get(&key)
            .ok_or_else(|| EngineError::Routing(format!("No handlers for {key:?}")))?
            .clone();
        drop(routes);

        let strategy = self
            .strategies
            .get(&msg.channel)
            .unwrap_or(&RoutingStrategy::Direct);

        match strategy {
            RoutingStrategy::Broadcast => Ok(handlers),
            RoutingStrategy::Direct => Ok(vec![handlers[0].clone()]),
            RoutingStrategy::RoundRobin => {
                // TODO: Implement round-robin selection
                Ok(vec![handlers[0].clone()])
            }
            RoutingStrategy::LoadBalanced => {
                // TODO: Implement load-balanced selection
                Ok(vec![handlers[0].clone()])
            }
        }
    }

    /// Register a handler for a protocol/channel combination
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    pub async fn register_handler(
        &self,
        protocol: ProtocolType,
        channel: ChannelType,
        handler_id: String,
    ) -> Result<(), EngineError> {
        let mut routes = self.routes.write().await;
        let key = (protocol, channel);

        routes
            .handlers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(handler_id.clone());

        routes.active_handlers.insert(handler_id);
        drop(routes);

        Ok(())
    }
}

/// Channel view for lightweight channel access
pub struct ChannelView<'a> {
    engine: &'a dyn ProtocolEngine,
    channel: ChannelType,
}

impl<'a> ChannelView<'a> {
    /// Create a new channel view
    pub fn new(engine: &'a dyn ProtocolEngine, channel: ChannelType) -> Self {
        Self { engine, channel }
    }

    /// Send a message on this channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send(&self, msg: UniversalMessage) -> Result<(), EngineError> {
        self.engine.send(self.channel, msg).await
    }

    /// Receive a message from this channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive
    pub async fn recv(&self) -> Result<UniversalMessage, EngineError> {
        self.engine.recv(self.channel).await
    }
}

/// Protocol engine trait for unified protocol handling
#[async_trait]
pub trait ProtocolEngine: Send + Sync {
    /// Register a protocol adapter
    async fn register_adapter(
        &mut self,
        protocol: ProtocolType,
        adapter: Box<dyn ProtocolAdapter>,
    ) -> Result<(), EngineError>;

    /// Send a message on a specific channel
    async fn send(&self, channel: ChannelType, msg: UniversalMessage) -> Result<(), EngineError>;

    /// Receive a message from a specific channel
    async fn recv(&self, channel: ChannelType) -> Result<UniversalMessage, EngineError>;

    /// Get a channel view for lightweight access
    fn channel_view(&self, channel: ChannelType) -> ChannelView<'_>;
}

/// Handler registry for protocol handlers
pub struct HandlerRegistry {
    handlers: HashMap<String, Box<dyn MessageHandler>>,
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandlerRegistry {
    /// Create a new handler registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a message handler
    pub fn register(&mut self, id: String, handler: Box<dyn MessageHandler>) {
        self.handlers.insert(id, handler);
    }

    /// Get a handler by ID
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&dyn MessageHandler> {
        self.handlers.get(id).map(std::convert::AsRef::as_ref)
    }
}

/// Message handler trait (imported from `protocol::message`)
pub use crate::protocol::message::MessageHandler;

/// Unified protocol engine implementation
pub struct UnifiedProtocolEngine {
    /// Transport layer
    transport: Arc<RwLock<Box<dyn Transport>>>,
    /// Protocol adapters
    adapters: HashMap<ProtocolType, Box<dyn ProtocolAdapter>>,
    /// Message router
    #[allow(dead_code)] // Will be used in future routing implementation
    router: Arc<MessageRouter>,
    /// Handler registry
    handlers: Arc<RwLock<HandlerRegistry>>,
}

impl UnifiedProtocolEngine {
    /// Create a new unified protocol engine
    #[must_use]
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport: Arc::new(RwLock::new(transport)),
            adapters: HashMap::new(),
            router: Arc::new(MessageRouter::new()),
            handlers: Arc::new(RwLock::new(HandlerRegistry::new())),
        }
    }

    /// Register a message handler
    pub async fn register_handler(&self, id: String, handler: Box<dyn MessageHandler>) {
        let mut handlers = self.handlers.write().await;
        handlers.register(id, handler);
    }
}

#[async_trait]
impl ProtocolEngine for UnifiedProtocolEngine {
    async fn register_adapter(
        &mut self,
        protocol: ProtocolType,
        adapter: Box<dyn ProtocolAdapter>,
    ) -> Result<(), EngineError> {
        self.adapters.insert(protocol, adapter);
        Ok(())
    }

    async fn send(&self, channel: ChannelType, msg: UniversalMessage) -> Result<(), EngineError> {
        // Get the appropriate adapter
        let adapter = self
            .adapters
            .get(&msg.protocol)
            .ok_or(EngineError::ProtocolNotSupported(msg.protocol))?;

        // Convert to raw format
        let _raw = adapter.adapt_outbound(&msg)?;

        // Create a ProtocolMessage for transport
        let protocol_msg = ProtocolMessage {
            msg_id: msg.id.clone(),
            msg_type: crate::protocol::message::MessageType::Request,
            channel: channel.to_string(),
            content: serde_json::to_value(&msg.content)
                .map_err(|e| EngineError::Conversion(e.to_string()))?,
        };

        // Send via transport
        let mut transport = self.transport.write().await;
        transport.send(protocol_msg).await?;
        drop(transport);

        Ok(())
    }

    async fn recv(&self, channel: ChannelType) -> Result<UniversalMessage, EngineError> {
        // Receive from transport
        let mut transport = self.transport.write().await;
        let protocol_msg = transport.recv().await?;
        drop(transport);

        // For now, assume LRP protocol (will be determined by message inspection later)
        let protocol = ProtocolType::LRP;

        // Convert to universal message
        let msg = UniversalMessage {
            id: protocol_msg.msg_id,
            protocol,
            channel,
            content: MessageContent::Request {
                method: "unknown".to_string(),
                params: protocol_msg.content,
            },
            metadata: HashMap::new(),
        };

        Ok(msg)
    }

    fn channel_view(&self, channel: ChannelType) -> ChannelView<'_> {
        ChannelView::new(self, channel)
    }
}

impl std::fmt::Display for ChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Shell => "shell",
            Self::IOPub => "iopub",
            Self::Stdin => "stdin",
            Self::Control => "control",
            Self::Heartbeat => "heartbeat",
        };
        write!(f, "{s}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_type_display() {
        assert_eq!(ChannelType::Shell.to_string(), "shell");
        assert_eq!(ChannelType::IOPub.to_string(), "iopub");
    }

    #[tokio::test]
    async fn test_message_router() {
        let router = MessageRouter::new();

        // Register a handler
        router
            .register_handler(
                ProtocolType::LRP,
                ChannelType::Shell,
                "test_handler".to_string(),
            )
            .await
            .unwrap();

        // Create a test message
        let msg = UniversalMessage {
            id: "test_msg".to_string(),
            protocol: ProtocolType::LRP,
            channel: ChannelType::Shell,
            content: MessageContent::Request {
                method: "test".to_string(),
                params: Value::Null,
            },
            metadata: HashMap::new(),
        };

        // Route the message
        let handlers = router.route(&msg).await.unwrap();
        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0], "test_handler");
    }
}
