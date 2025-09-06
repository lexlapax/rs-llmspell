//! Null implementations of traits for testing
//!
//! These implementations allow testing of individual components
//! without requiring full protocol/transport implementations

use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use super::{KernelMessage, Protocol, Transport, TransportConfig};

/// Null transport for testing - does nothing but succeeds
pub struct NullTransport {
    channels: Vec<String>,
}

impl NullTransport {
    #[must_use]
    pub const fn new() -> Self {
        Self { channels: vec![] }
    }
}

impl Default for NullTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for NullTransport {
    async fn bind(&mut self, config: &TransportConfig) -> Result<()> {
        self.channels = config.channels.keys().cloned().collect();
        Ok(())
    }

    async fn connect(&mut self, config: &TransportConfig) -> Result<()> {
        self.channels = config.channels.keys().cloned().collect();
        Ok(())
    }

    async fn recv(&self, _channel: &str) -> Result<Option<Vec<Vec<u8>>>> {
        Ok(None)
    }

    async fn send(&self, _channel: &str, _parts: Vec<Vec<u8>>) -> Result<()> {
        Ok(())
    }

    async fn heartbeat(&self) -> Result<bool> {
        Ok(false)
    }

    fn has_channel(&self, channel: &str) -> bool {
        self.channels.contains(&channel.to_string())
    }

    fn channels(&self) -> Vec<String> {
        self.channels.clone()
    }
}

/// Null protocol for testing
pub struct NullProtocol;

impl NullProtocol {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NullProtocol {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Protocol for NullProtocol {
    type Message = NullMessage;

    fn decode(&self, _parts: Vec<Vec<u8>>, _channel: &str) -> Result<Self::Message> {
        Ok(NullMessage::new("null".to_string(), Value::Null))
    }

    fn encode(&self, _msg: &Self::Message, _channel: &str) -> Result<Vec<Vec<u8>>> {
        Ok(vec![])
    }

    fn transport_config(&self) -> TransportConfig {
        TransportConfig {
            transport_type: "null".to_string(),
            base_address: "null".to_string(),
            channels: HashMap::new(),
        }
    }

    fn name(&self) -> &'static str {
        "null"
    }

    fn version(&self) -> &'static str {
        "0.0.0"
    }

    fn requires_reply(&self, _msg: &Self::Message) -> bool {
        false
    }

    fn create_reply(&self, _request: &Self::Message, content: Value) -> Result<Self::Message> {
        Ok(NullMessage::new("null_reply".to_string(), content))
    }

    fn reply_channel(&self, _msg: &Self::Message) -> &'static str {
        "null"
    }

    fn create_broadcast(
        &self,
        msg_type: &str,
        content: serde_json::Value,
        _parent_msg: Option<&Self::Message>,
        _kernel_id: &str,
    ) -> Result<Self::Message> {
        Ok(NullMessage::new(msg_type.to_string(), content))
    }
}

/// Null message for testing
#[derive(Debug, Clone)]
pub struct NullMessage {
    msg_type: String,
    msg_id: String,
    session_id: String,
    parent_id: Option<String>,
    content: Value,
    metadata: Value,
}

impl NullMessage {
    #[must_use]
    pub fn new(msg_type: String, content: Value) -> Self {
        Self {
            msg_type,
            msg_id: uuid::Uuid::new_v4().to_string(),
            session_id: "null-session".to_string(),
            parent_id: None,
            content,
            metadata: Value::Object(serde_json::Map::new()),
        }
    }
}

impl KernelMessage for NullMessage {
    fn msg_type(&self) -> &str {
        &self.msg_type
    }

    fn msg_id(&self) -> &str {
        &self.msg_id
    }

    fn session_id(&self) -> &str {
        &self.session_id
    }

    fn parent_id(&self) -> Option<&str> {
        self.parent_id.as_deref()
    }

    fn content(&self) -> Value {
        self.content.clone()
    }

    fn metadata(&self) -> Value {
        self.metadata.clone()
    }

    fn set_parent(&mut self, parent_id: String, _parent_type: String) {
        self.parent_id = Some(parent_id);
    }

    fn new(msg_type: String, content: Value) -> Self {
        Self::new(msg_type, content)
    }
}
