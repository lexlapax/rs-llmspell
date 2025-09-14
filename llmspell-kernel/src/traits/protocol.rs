//! Protocol trait for message handling
//!
//! This module defines the Protocol trait that abstracts message parsing
//! and creation across different protocols (Jupyter, LSP, DAP, WebSocket).

use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;

/// Protocol trait for handling different messaging protocols
pub trait Protocol: Send + Sync {
    /// Parse an incoming message
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>>;

    /// Create a response message
    fn create_response(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;

    /// Create a request message
    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;
}
