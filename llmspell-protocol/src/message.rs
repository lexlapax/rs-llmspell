//! Protocol message types and utilities
//!
//! Defines the message format for LRP/LDP protocol communication

use crate::types::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Type of protocol message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Error,
}

/// Protocol message wrapper
///
/// Wraps LRP/LDP messages with metadata for routing and correlation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    /// Unique message identifier
    pub msg_id: String,

    /// Type of message
    pub msg_type: MessageType,

    /// Channel this message belongs to (shell, iopub, stdin, control, heartbeat)
    pub channel: String,

    /// Actual message content (LRP or LDP request/response)
    pub content: serde_json::Value,
}

impl ProtocolMessage {
    /// Create a new request message
    pub fn request(msg_id: impl Into<String>, content: impl Serialize) -> Self {
        Self {
            msg_id: msg_id.into(),
            msg_type: MessageType::Request,
            channel: "shell".to_string(),
            content: serde_json::to_value(content).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Create a new response message
    pub fn response(msg_id: impl Into<String>, content: impl Serialize) -> Self {
        Self {
            msg_id: msg_id.into(),
            msg_type: MessageType::Response,
            channel: "shell".to_string(),
            content: serde_json::to_value(content).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Create a notification message (no response expected)
    pub fn notification(channel: impl Into<String>, content: impl Serialize) -> Self {
        Self {
            msg_id: uuid::Uuid::new_v4().to_string(),
            msg_type: MessageType::Notification,
            channel: channel.into(),
            content: serde_json::to_value(content).unwrap_or(serde_json::Value::Null),
        }
    }

    /// Create an error message
    pub fn error(msg_id: impl Into<String>, error: impl fmt::Display) -> Self {
        Self {
            msg_id: msg_id.into(),
            msg_type: MessageType::Error,
            channel: "shell".to_string(),
            content: serde_json::json!({
                "error": error.to_string()
            }),
        }
    }

    /// Try to extract an LRP request from the message
    pub fn as_lrp_request(&self) -> Option<LRPRequest> {
        if self.msg_type == MessageType::Request {
            serde_json::from_value(self.content.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract an LRP response from the message
    pub fn as_lrp_response(&self) -> Option<LRPResponse> {
        if self.msg_type == MessageType::Response {
            serde_json::from_value(self.content.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract an LDP request from the message
    pub fn as_ldp_request(&self) -> Option<LDPRequest> {
        if self.msg_type == MessageType::Request {
            serde_json::from_value(self.content.clone()).ok()
        } else {
            None
        }
    }

    /// Try to extract an LDP response from the message
    pub fn as_ldp_response(&self) -> Option<LDPResponse> {
        if self.msg_type == MessageType::Response {
            serde_json::from_value(self.content.clone()).ok()
        } else {
            None
        }
    }
}

/// Message handler trait for server-side processing
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Handle an incoming message and optionally return a response
    async fn handle(&self, msg: ProtocolMessage) -> Option<ProtocolMessage>;
}
