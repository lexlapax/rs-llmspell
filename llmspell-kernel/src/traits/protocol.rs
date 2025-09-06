//! Protocol trait for message encoding/decoding and lifecycle management
//!
//! This trait abstracts the protocol layer (Jupyter, LSP, DAP, MCP, etc.)
//! It defines both wire format (encoding/decoding) and protocol semantics
//! (message lifecycle, output handling, channel topology)

use super::message::KernelMessage;
use super::transport::TransportConfig;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Kernel execution status for protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KernelStatus {
    Idle,
    Busy,
    Starting,
}

/// Stream data for output messages
#[derive(Debug, Clone)]
pub struct StreamData {
    pub stream_type: StreamType,
    pub text: String,
}

/// Stream type for output routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Stdout,
    Stderr,
}

/// Execution result from script runtime
#[derive(Debug, Clone, Default)]
pub struct ExecutionResult {
    pub output: Vec<String>,
    pub errors: Vec<String>,
    pub result_value: Option<serde_json::Value>,
    pub execution_count: u32,
}

/// Execution error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionError {
    pub name: String,
    pub message: String,
    pub traceback: Vec<String>,
}

/// Output chunk captured during execution
#[derive(Debug, Clone)]
pub enum OutputChunk {
    Stdout(String),
    Stderr(String),
    Result(serde_json::Value),
    Error(ExecutionError),
}

impl OutputChunk {
    /// Check if this chunk represents a newline
    #[must_use]
    pub fn is_newline(&self) -> bool {
        match self {
            Self::Stdout(s) | Self::Stderr(s) => s.ends_with('\n'),
            _ => false,
        }
    }
}

/// Execution flow defines the message sequence for a protocol
#[derive(Debug, Clone)]
pub struct ExecutionFlow<M: KernelMessage> {
    /// Messages to send before execution
    pub pre_execution: Vec<(String, M)>,
    /// Whether to capture output during execution
    pub capture_output: bool,
    /// Messages to send after execution
    pub post_execution: Vec<(String, M)>,
}

/// Channel topology describes protocol's channel requirements
#[derive(Debug, Clone)]
pub struct ChannelTopology {
    /// Required channels and their patterns
    pub channels: HashMap<String, ChannelPattern>,
    /// Primary shell/request channel
    pub shell_channel: String,
    /// Broadcast/event channel (if any)
    pub broadcast_channel: Option<String>,
}

/// Channel communication pattern
#[derive(Debug, Clone)]
pub enum ChannelPattern {
    RequestReply,
    PubSub,
    Push,
    Pull,
}

/// Expected response flow for a message type
#[derive(Debug, Clone)]
pub struct ResponseFlow {
    /// Expected message types in order
    pub expected_messages: Vec<ExpectedMessage>,
    /// Timeout for complete flow (milliseconds)
    pub timeout_ms: u64,
}

/// Expected message in response flow
#[derive(Debug, Clone)]
pub struct ExpectedMessage {
    pub channel: String,
    pub message_type: String,
    pub required: bool,
}

impl ResponseFlow {
    /// Create a response collector for this flow
    #[must_use]
    pub fn create_response_collector(&self) -> ResponseCollector {
        let mut collector = ResponseCollector::default();
        collector.expected.clone_from(&self.expected_messages);
        collector
    }
}

/// Collects responses according to protocol flow
#[derive(Default)]
pub struct ResponseCollector {
    #[allow(dead_code)]
    expected: Vec<ExpectedMessage>,
    received: Vec<(String, serde_json::Value)>, // Store as JSON instead of trait object
    completed: bool,
    /// Shell channel messages for testing
    pub shell_messages: Vec<serde_json::Value>,
    /// `IOPub` channel messages for testing
    pub iopub_messages: Vec<serde_json::Value>,
    /// Whether idle status was received
    pub received_idle: bool,
}

impl ResponseCollector {
    /// Add a received message
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but may in future implementations.
    pub fn add_message(&mut self, channel: &str, msg_content: serde_json::Value) -> Result<()> {
        // Store in channel-specific collections
        match channel {
            "shell" => self.shell_messages.push(msg_content.clone()),
            "iopub" => {
                self.iopub_messages.push(msg_content.clone());
                // Check for idle status
                if let Some(msg_type) = msg_content.get("msg_type").and_then(|t| t.as_str()) {
                    if msg_type == "status" {
                        if let Some(state) = msg_content
                            .get("content")
                            .and_then(|c| c.get("execution_state"))
                            .and_then(|s| s.as_str())
                        {
                            if state == "idle" {
                                self.received_idle = true;
                            }
                        }
                    }
                }
            }
            _ => {}
        }

        self.received.push((channel.to_string(), msg_content));
        // Check if we've received all required messages
        self.check_completion();
        Ok(())
    }

    /// Check if collection is complete
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.completed
    }

    /// Build the final result
    #[must_use]
    pub fn build_result(self) -> ExecutionResult {
        // TODO: Extract result from collected messages
        ExecutionResult::default()
    }

    const fn check_completion(&mut self) {
        // Mark complete after receiving idle status
        if self.received_idle {
            self.completed = true;
        }
    }
}

/// Generic protocol for encoding/decoding messages
#[async_trait]
pub trait Protocol: Send + Sync {
    /// The concrete message type for this protocol
    type Message: KernelMessage;

    /// Output context type for buffering output during execution
    type OutputContext: Send;

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

    // === NEW: Message Lifecycle Methods ===

    /// Create execution flow for a request message
    ///
    /// Defines the complete message sequence for handling a request
    fn create_execution_flow(&self, _request: &Self::Message) -> ExecutionFlow<Self::Message> {
        // Default implementation: no additional messages
        ExecutionFlow {
            pre_execution: Vec::new(),
            capture_output: false,
            post_execution: Vec::new(),
        }
    }

    /// Create a status message
    ///
    /// # Errors
    ///
    /// Returns an error if status messages are not supported by this protocol.
    fn create_status_message(&self, _status: KernelStatus) -> Result<Self::Message> {
        // Default: protocols without status messages can override
        Err(anyhow::anyhow!(
            "Status messages not supported by this protocol"
        ))
    }

    /// Create an execute input message
    ///
    /// # Errors
    ///
    /// Returns an error if execute input messages are not supported by this protocol.
    fn create_execute_input_message(&self, _code: &str, _count: u32) -> Result<Self::Message> {
        // Default: protocols without execute_input can override
        Err(anyhow::anyhow!(
            "Execute input messages not supported by this protocol"
        ))
    }

    /// Create a stream message for stdout/stderr
    ///
    /// # Errors
    ///
    /// Returns an error if stream messages are not supported by this protocol.
    fn create_stream_message(&self, _stream: StreamData) -> Result<Self::Message> {
        // Default: protocols without stream messages can override
        Err(anyhow::anyhow!(
            "Stream messages not supported by this protocol"
        ))
    }

    /// Create an execution result message
    ///
    /// # Errors
    ///
    /// Returns an error if execute result messages are not supported by this protocol.
    fn create_execute_result(&self, _result: ExecutionResult) -> Result<Self::Message> {
        // Default: protocols without execute_result can override
        Err(anyhow::anyhow!(
            "Execute result messages not supported by this protocol"
        ))
    }

    /// Create an error message
    ///
    /// # Errors
    ///
    /// Returns an error if error messages are not supported by this protocol.
    fn create_error_message(&self, _error: ExecutionError) -> Result<Self::Message> {
        // Default: protocols without error messages can override
        Err(anyhow::anyhow!(
            "Error messages not supported by this protocol"
        ))
    }

    // === Output Handling Strategy ===

    /// Create output context for capturing execution output
    fn create_output_context(&self) -> Self::OutputContext;

    /// Handle an output chunk during execution
    fn handle_output(&self, _ctx: &mut Self::OutputContext, _output: OutputChunk) {
        // Default: no-op, protocols can override to buffer/process output
    }

    /// Flush buffered output and create messages
    fn flush_output(&self, _ctx: Self::OutputContext) -> Vec<(String, Self::Message)> {
        // Default: no messages to flush
        Vec::new()
    }

    // === Channel Topology ===

    /// Get the channel topology for this protocol
    fn channel_topology(&self) -> ChannelTopology {
        // Default: simple request-reply on "shell" channel
        let mut channels = HashMap::new();
        channels.insert("shell".to_string(), ChannelPattern::RequestReply);

        ChannelTopology {
            channels,
            shell_channel: "shell".to_string(),
            broadcast_channel: None,
        }
    }

    /// Get expected response flow for a message type
    fn expected_response_flow(&self, msg_type: &str) -> ResponseFlow {
        // Default: expect a single reply on shell channel
        ResponseFlow {
            expected_messages: vec![ExpectedMessage {
                channel: "shell".to_string(),
                message_type: format!("{}_reply", msg_type.trim_end_matches("_request")),
                required: true,
            }],
            timeout_ms: 30000, // 30 second default timeout
        }
    }
}
