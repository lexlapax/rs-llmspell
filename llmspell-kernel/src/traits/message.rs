//! `KernelMessage` trait for generic message handling
//!
//! This trait abstracts protocol-specific messages to allow
//! the kernel to process messages without knowing their format

use serde_json::Value;
use std::fmt::Debug;

/// Generic kernel message interface
pub trait KernelMessage: Send + Sync + Debug + Clone {
    /// Get message type identifier (e.g., "`execute_request`", "`kernel_info_request`")
    fn msg_type(&self) -> &str;

    /// Get unique message ID
    fn msg_id(&self) -> &str;

    /// Get session ID
    fn session_id(&self) -> &str;

    /// Get parent message ID if this is a reply
    fn parent_id(&self) -> Option<&str>;

    /// Convert message content to JSON for processing
    fn content(&self) -> Value;

    /// Get message metadata
    fn metadata(&self) -> Value;

    /// Set parent information for creating replies
    fn set_parent(&mut self, parent_id: String, parent_type: String);

    /// Create a new message with given type and content
    fn new(msg_type: String, content: Value) -> Self;

    /// Check if this is a request message (vs reply/event)
    fn is_request(&self) -> bool {
        self.msg_type().ends_with("_request")
    }

    /// Get message header as JSON for parent tracking (for `IOPub` messages)
    /// Returns None if not applicable for this protocol
    fn header_for_parent(&self) -> Option<Value> {
        None
    }

    /// Check if this is a reply message
    fn is_reply(&self) -> bool {
        self.msg_type().ends_with("_reply")
    }

    /// Check if this is an event/status message
    fn is_event(&self) -> bool {
        !self.is_request() && !self.is_reply()
    }

    /// Set parent header from JSON value
    /// Default implementation does nothing - protocols can override
    fn set_parent_from_json(&mut self, _parent_header: Value) {
        // Default: no-op
    }
}
