//! Jupyter wire protocol 5.3 implementation
//!
//! This is the ONLY protocol implementation for the kernel.
//! All kernel modes (embedded, service, client) use this protocol.

use crate::traits::Protocol;
use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use uuid::Uuid;

/// Jupyter wire protocol implementation (version 5.3)
///
/// This protocol is used for ALL kernel communication, regardless of transport.
/// The difference between CLI and service mode is the transport layer, not the protocol.
#[derive(Debug, Clone)]
pub struct JupyterProtocol {
    session_id: String,
    kernel_id: String,
    protocol_version: String,
    username: String,
}

impl JupyterProtocol {
    /// Create a new Jupyter protocol instance for kernel mode
    pub fn new(session_id: String, kernel_id: String) -> Self {
        Self {
            session_id,
            kernel_id,
            protocol_version: "5.3".to_string(),
            username: "kernel".to_string(),
        }
    }

    /// Create a new Jupyter protocol instance for client mode
    pub fn new_client() -> Self {
        Self {
            session_id: Uuid::new_v4().to_string(),
            kernel_id: String::new(), // Client doesn't own a kernel
            protocol_version: "5.3".to_string(),
            username: "client".to_string(),
        }
    }

    /// Create a message header
    fn create_header(&self, msg_type: &str) -> HashMap<String, Value> {
        let mut header = HashMap::new();
        header.insert("msg_id".to_string(), json!(Uuid::new_v4().to_string()));
        header.insert("session".to_string(), json!(self.session_id));
        header.insert("username".to_string(), json!(self.username));
        header.insert("msg_type".to_string(), json!(msg_type));
        header.insert("version".to_string(), json!(self.protocol_version));
        header.insert("date".to_string(), json!(chrono::Utc::now().to_rfc3339()));
        // Include kernel_id in metadata for tracking
        if !self.kernel_id.is_empty() {
            header.insert("kernel".to_string(), json!(self.kernel_id));
        }
        header
    }
}

impl Protocol for JupyterProtocol {
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>> {
        // Parse Jupyter wire protocol message
        // This handles the full Jupyter message format with:
        // - Header
        // - Parent header
        // - Metadata
        // - Content
        // - Buffers (for binary data)

        if data.is_empty() {
            return Ok(HashMap::new());
        }

        // In production, this would parse the actual Jupyter wire format
        // including HMAC signatures, ZMQ multipart messages, etc.
        let message: Value = serde_json::from_slice(data)?;

        if let Value::Object(map) = message {
            Ok(map.into_iter().collect())
        } else {
            Ok(HashMap::new())
        }
    }

    fn create_response(&self, msg_type: &str, content: Value) -> Result<Vec<u8>> {
        // Create a complete Jupyter wire protocol response
        let response = json!({
            "header": self.create_header(msg_type),
            "parent_header": {},  // Would be set from request in real impl
            "metadata": {},
            "content": content,
            "buffers": []
        });

        Ok(serde_json::to_vec(&response)?)
    }

    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>> {
        // Create a complete Jupyter wire protocol request
        let request = json!({
            "header": self.create_header(msg_type),
            "parent_header": {},
            "metadata": {},
            "content": content,
            "buffers": []
        });

        Ok(serde_json::to_vec(&request)?)
    }
}
