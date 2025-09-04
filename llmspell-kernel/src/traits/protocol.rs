//! Protocol trait for message encoding/decoding
//!
//! This trait abstracts the protocol layer (Jupyter, LSP, DAP, MCP, etc.)
//! It knows about transport configuration but not transport implementation

use super::message::KernelMessage;
use super::transport::TransportConfig;
use anyhow::Result;
use async_trait::async_trait;

/// Generic protocol for encoding/decoding messages
#[async_trait]
pub trait Protocol: Send + Sync {
    /// The concrete message type for this protocol
    type Message: KernelMessage;

    /// Decode multipart message from transport into protocol message
    ///
    /// # Errors
    ///
    /// Returns an error if message decoding fails.
    fn decode(&self, parts: Vec<Vec<u8>>, channel: &str) -> Result<Self::Message>;

    /// Encode protocol message into multipart format for transport
    ///
    /// # Errors
    ///
    /// Returns an error if message encoding fails.
    fn encode(&self, msg: &Self::Message, channel: &str) -> Result<Vec<Vec<u8>>>;

    /// Get transport configuration required by this protocol
    fn transport_config(&self) -> TransportConfig;

    /// Protocol name for identification
    fn name(&self) -> &str;

    /// Protocol version
    fn version(&self) -> &str;

    /// Check if a message requires a reply
    fn requires_reply(&self, msg: &Self::Message) -> bool;

    /// Create a reply message for a given request
    ///
    /// # Errors
    ///
    /// Returns an error if reply creation fails.
    fn create_reply(
        &self,
        request: &Self::Message,
        content: serde_json::Value,
    ) -> Result<Self::Message>;

    /// Get the channel a message should be sent on
    fn reply_channel(&self, msg: &Self::Message) -> &str;

    /// Create a broadcast/event message (e.g., for `IOPub` channel)
    ///
    /// # Errors
    ///
    /// Returns an error if message creation fails.
    fn create_broadcast(
        &self,
        msg_type: &str,
        content: serde_json::Value,
        parent_msg: Option<&Self::Message>,
        kernel_id: &str,
    ) -> Result<Self::Message>;
}
