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
    ///
    /// # Errors
    ///
    /// Returns an error if the message cannot be parsed
    fn parse_message(&self, data: &[u8]) -> Result<HashMap<String, Value>>;

    /// Create a response message
    ///
    /// # Errors
    ///
    /// Returns an error if the response cannot be created
    fn create_response(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;

    /// Create a request message
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be created
    fn create_request(&self, msg_type: &str, content: Value) -> Result<Vec<u8>>;

    /// Sign message components for authentication (e.g., HMAC for Jupyter)
    ///
    /// # Errors
    ///
    /// Returns an error if signing fails
    fn sign_message(
        &self,
        header: &[u8],
        parent_header: &[u8],
        metadata: &[u8],
        content: &[u8],
    ) -> Result<String>;

    /// Set the HMAC key for message authentication
    fn set_hmac_key(&mut self, key: &str);
}
