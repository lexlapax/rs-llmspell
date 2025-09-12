//! Jupyter protocol message types for `LLMSpell` kernel
//!
//! Implements standard Jupyter messaging protocol with extensions for:
//! - DAP (Debug Adapter Protocol) support via debug messages
//! - Daemon management for kernel lifecycle control
//! - `LLMSpell`-specific script execution features

use crate::traits::KernelMessage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use uuid::Uuid;

/// Main Jupyter message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterMessage {
    pub header: MessageHeader,
    pub parent_header: Option<MessageHeader>,
    pub metadata: Value,
    pub content: MessageContent,
}

/// Standard Jupyter message header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    pub msg_id: String,
    pub msg_type: String,
    pub username: String,
    pub session: String,
    pub date: DateTime<Utc>,
    pub version: String,
}

/// All supported message content types  
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    // === KERNEL LIFECYCLE ===
    #[serde(rename = "kernel_info_request")]
    KernelInfoRequest {},

    #[serde(rename = "kernel_info_reply")]
    KernelInfoReply {
        status: String,
        protocol_version: String,
        implementation: String,
        implementation_version: String,
        language_info: LanguageInfo,
        banner: String,
        help_links: Vec<HelpLink>,
        // LLMSpell extension for session metadata
        #[serde(skip_serializing_if = "Option::is_none")]
        llmspell_session_metadata: Option<Value>,
    },

    // === CODE EXECUTION ===
    #[serde(rename = "execute_request")]
    ExecuteRequest {
        code: String,
        silent: bool,
        store_history: Option<bool>,
        user_expressions: Option<HashMap<String, String>>,
        allow_stdin: Option<bool>,
        stop_on_error: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        script_args: Option<Vec<String>>,
    },

    #[serde(rename = "execute_reply")]
    ExecuteReply {
        status: ExecutionStatus,
        execution_count: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        user_expressions: Option<HashMap<String, Value>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        payload: Option<Vec<Value>>,
        // Error fields (only present when status = "error")
        #[serde(skip_serializing_if = "Option::is_none")]
        ename: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        evalue: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        traceback: Option<Vec<String>>,
    },

    #[serde(rename = "execute_input")]
    ExecuteInput { code: String, execution_count: u32 },

    #[serde(rename = "execute_result")]
    ExecuteResult {
        execution_count: u32,
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    },

    // === OUTPUT STREAMS ===
    #[serde(rename = "stream")]
    Stream { name: StreamType, text: String },

    #[serde(rename = "display_data")]
    DisplayData {
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
        transient: Option<Value>,
    },

    #[serde(rename = "error")]
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },

    // === STATUS UPDATES ===
    #[serde(rename = "status")]
    Status { execution_state: ExecutionState },

    // === INPUT REQUESTS ===
    #[serde(rename = "input_request")]
    InputRequest { prompt: String, password: bool },

    #[serde(rename = "input_reply")]
    InputReply { value: String },

    // === CONTROL MESSAGES ===
    #[serde(rename = "shutdown_request")]
    ShutdownRequest { restart: bool },

    #[serde(rename = "shutdown_reply")]
    ShutdownReply { status: String, restart: bool },

    #[serde(rename = "interrupt_request")]
    InterruptRequest {},

    #[serde(rename = "interrupt_reply")]
    InterruptReply { status: String },

    // === DEBUG SUPPORT (DAP integration) ===
    #[serde(rename = "debug_request")]
    DebugRequest {
        command: String,
        arguments: Value,
        seq: u32,
    },

    #[serde(rename = "debug_reply")]
    DebugReply {
        success: bool,
        command: String,
        request_seq: u32,
        seq: u32,
        message: Option<String>,
        body: Option<Value>,
    },

    #[serde(rename = "debug_event")]
    DebugEvent {
        event: String,
        seq: u32,
        body: Option<Value>,
    },

    // === DAEMON MANAGEMENT (Custom extension for LLMSpell) ===
    #[serde(rename = "daemon_request")]
    DaemonRequest {
        command: DaemonCommand,
        kernel_id: Option<String>,
        config: Option<Value>,
    },

    #[serde(rename = "daemon_reply")]
    DaemonReply {
        status: String,
        command: DaemonCommand,
        result: Option<Value>,
        error: Option<String>,
        kernels: Option<Vec<KernelInfo>>,
    },

    // === COMPLETION & INSPECTION ===
    #[serde(rename = "complete_request")]
    CompleteRequest { code: String, cursor_pos: u32 },

    #[serde(rename = "complete_reply")]
    CompleteReply {
        matches: Vec<String>,
        cursor_start: u32,
        cursor_end: u32,
        metadata: HashMap<String, Value>,
        status: String,
    },

    #[serde(rename = "inspect_request")]
    InspectRequest {
        code: String,
        cursor_pos: u32,
        detail_level: u32,
    },

    #[serde(rename = "inspect_reply")]
    InspectReply {
        status: String,
        found: bool,
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    },

    // === COMM CHANNELS FOR CUSTOM COMMUNICATION ===
    #[serde(rename = "comm_open")]
    CommOpen {
        comm_id: String,
        target_name: String,
        data: Value,
        metadata: Option<HashMap<String, Value>>,
    },

    #[serde(rename = "comm_msg")]
    CommMsg {
        comm_id: String,
        data: Value,
        metadata: Option<HashMap<String, Value>>,
    },

    #[serde(rename = "comm_close")]
    CommClose {
        comm_id: String,
        data: Option<Value>,
        metadata: Option<HashMap<String, Value>>,
    },

    #[serde(rename = "comm_info_request")]
    CommInfoRequest { target_name: Option<String> },

    #[serde(rename = "comm_info_reply")]
    CommInfoReply {
        status: String,
        comms: HashMap<String, CommInfo>,
    },
}

/// Information about an open comm channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommInfo {
    pub target_name: String,
}

/// Execution status for `execute_reply` messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionStatus {
    Ok,
    Error,
    Aborted,
}

/// Execution state for status messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionState {
    Busy,
    Idle,
    Starting,
}

impl ExecutionState {
    #[allow(dead_code)]
    fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "busy" => Self::Busy,
            "starting" => Self::Starting,
            _ => Self::Idle,
        }
    }
}

/// Stream types for output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Stdout,
    Stderr,
}

/// Language information for `kernel_info_reply`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    pub name: String,
    pub version: String,
    pub mimetype: String,
    pub file_extension: String,
    pub pygments_lexer: Option<String>,
    pub codemirror_mode: Option<String>,
    pub nbconvert_exporter: Option<String>,
}

/// Help link for `kernel_info_reply`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpLink {
    pub text: String,
    pub url: String,
}

/// Daemon management commands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DaemonCommand {
    ListKernels,
    StartKernel,
    StopKernel,
    RestartKernel,
    KernelStatus,
    DaemonStatus,
    UpdateConfig,
    GetLogs,
}

/// Kernel information for daemon replies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    pub kernel_id: String,
    pub status: String,
    pub engine: String,
    pub connections: u32,
    pub uptime: u64,
    pub last_activity: DateTime<Utc>,
}

// === MESSAGE CONSTRUCTION HELPERS ===

impl JupyterMessage {
    /// Create new message with generated header
    #[must_use]
    pub fn new(msg_type: &str, content: MessageContent) -> Self {
        Self {
            header: MessageHeader::new(msg_type),
            parent_header: None,
            metadata: Value::Object(Map::new()),
            content,
        }
    }

    /// Create reply message with parent header
    #[must_use]
    pub fn reply(parent: &Self, msg_type: &str, content: MessageContent) -> Self {
        Self {
            header: MessageHeader::new(msg_type),
            parent_header: Some(parent.header.clone()),
            metadata: Value::Object(Map::new()),
            content,
        }
    }

    /// Get message type from header
    #[must_use]
    pub fn msg_type(&self) -> &str {
        &self.header.msg_type
    }

    /// Check if this is a request message
    #[must_use]
    pub fn is_request(&self) -> bool {
        self.header.msg_type.ends_with("_request")
    }

    /// Check if this is a reply message  
    #[must_use]
    pub fn is_reply(&self) -> bool {
        self.header.msg_type.ends_with("_reply")
    }
}

impl MessageHeader {
    /// Create new header with generated ID and current timestamp
    #[must_use]
    pub fn new(msg_type: &str) -> Self {
        Self {
            msg_id: Uuid::new_v4().to_string(),
            msg_type: msg_type.to_string(),
            username: "kernel".to_string(),
            session: Uuid::new_v4().to_string(),
            date: Utc::now(),
            version: "5.3".to_string(), // Jupyter protocol version
        }
    }
}

/// Implement `KernelMessage` trait for `JupyterMessage`
impl KernelMessage for JupyterMessage {
    fn msg_type(&self) -> &str {
        &self.header.msg_type
    }

    fn msg_id(&self) -> &str {
        &self.header.msg_id
    }

    fn session_id(&self) -> &str {
        &self.header.session
    }

    fn parent_id(&self) -> Option<&str> {
        self.parent_header.as_ref().map(|h| h.msg_id.as_str())
    }

    fn content(&self) -> Value {
        // Extract the inner content data without the enum wrapper
        match &self.content {
            MessageContent::CommOpen {
                comm_id,
                target_name,
                data,
                metadata,
            } => {
                let mut obj = serde_json::json!({
                    "comm_id": comm_id,
                    "target_name": target_name,
                    "data": data,
                });
                if let Some(meta) = metadata {
                    obj["metadata"] = serde_json::to_value(meta).unwrap_or(Value::Null);
                }
                obj
            }
            MessageContent::CommMsg {
                comm_id,
                data,
                metadata,
            } => {
                let mut obj = serde_json::json!({
                    "comm_id": comm_id,
                    "data": data,
                });
                if let Some(meta) = metadata {
                    obj["metadata"] = serde_json::to_value(meta).unwrap_or(Value::Null);
                }
                obj
            }
            MessageContent::CommClose {
                comm_id,
                data,
                metadata,
            } => {
                let mut obj = serde_json::json!({
                    "comm_id": comm_id,
                });
                if let Some(d) = data {
                    obj["data"] = d.clone();
                }
                if let Some(meta) = metadata {
                    obj["metadata"] = serde_json::to_value(meta).unwrap_or(Value::Null);
                }
                obj
            }
            MessageContent::ExecuteRequest {
                code,
                silent,
                store_history,
                user_expressions,
                allow_stdin,
                stop_on_error,
                script_args,
            } => {
                let mut obj = serde_json::json!({
                    "code": code,
                    "silent": silent,
                });
                if let Some(sh) = store_history {
                    obj["store_history"] = serde_json::json!(sh);
                }
                if let Some(ue) = user_expressions {
                    obj["user_expressions"] = serde_json::to_value(ue).unwrap_or(Value::Null);
                }
                if let Some(ai) = allow_stdin {
                    obj["allow_stdin"] = serde_json::json!(ai);
                }
                if let Some(soe) = stop_on_error {
                    obj["stop_on_error"] = serde_json::json!(soe);
                }
                if let Some(args) = script_args {
                    obj["script_args"] = serde_json::to_value(args).unwrap_or(Value::Null);
                }
                obj
            }
            MessageContent::ShutdownRequest { restart } => {
                serde_json::json!({
                    "restart": restart,
                })
            }
            MessageContent::CommInfoRequest { target_name } => {
                let mut obj = serde_json::json!({});
                if let Some(tn) = target_name {
                    obj["target_name"] = serde_json::json!(tn);
                }
                obj
            }
            // For other message types, serialize the whole enum (fallback)
            _ => serde_json::to_value(&self.content).unwrap_or(Value::Null),
        }
    }

    fn metadata(&self) -> Value {
        self.metadata.clone()
    }

    fn set_parent(&mut self, parent_id: String, parent_type: String) {
        let mut parent_header = MessageHeader::new(&parent_type);
        parent_header.msg_id = parent_id;
        parent_header.session.clone_from(&self.header.session);
        self.parent_header = Some(parent_header);
    }

    fn new(msg_type: String, content: Value) -> Self {
        // Convert Value back to MessageContent
        // For simplicity, we'll use a generic content type
        let content_enum = serde_json::from_value::<MessageContent>(content).map_or(
            MessageContent::Status {
                execution_state: ExecutionState::Idle,
            },
            |parsed| parsed,
        );

        Self {
            header: MessageHeader::new(&msg_type),
            parent_header: None,
            metadata: Value::Object(Map::new()),
            content: content_enum,
        }
    }

    fn header_for_parent(&self) -> Option<Value> {
        // Convert header to JSON for parent tracking
        serde_json::to_value(&self.header).ok()
    }

    fn set_parent_from_json(&mut self, parent_header: Value) {
        // Try to deserialize the JSON into a MessageHeader
        if let Ok(header) = serde_json::from_value::<MessageHeader>(parent_header) {
            self.parent_header = Some(header);
        }
    }
}

impl Default for LanguageInfo {
    fn default() -> Self {
        Self {
            name: "llmspell".to_string(),
            version: "0.8.0".to_string(),
            mimetype: "text/x-lua".to_string(),
            file_extension: ".lua".to_string(),
            pygments_lexer: Some("lua".to_string()),
            codemirror_mode: Some("lua".to_string()),
            nbconvert_exporter: None,
        }
    }
}

// === JUPYTER PROTOCOL IMPLEMENTATION ===
use super::WireProtocol;
use crate::connection::ConnectionInfo;
use crate::traits::{transport::ChannelConfig, Protocol, TransportConfig};
use async_trait::async_trait;

/// Jupyter protocol implementation
pub struct JupyterProtocol {
    wire: WireProtocol,
    connection_info: ConnectionInfo,
}

impl JupyterProtocol {
    /// Create a new Jupyter protocol handler
    #[must_use]
    pub fn new(connection_info: ConnectionInfo) -> Self {
        let wire = WireProtocol::new(connection_info.key.clone());
        Self {
            wire,
            connection_info,
        }
    }
}

/// Output context for Jupyter protocol - buffers output for efficient messaging
#[derive(Debug, Default)]
pub struct JupyterOutputContext {
    /// Buffered stdout output
    pub stdout_buffer: Vec<String>,
    /// Buffered stderr output
    pub stderr_buffer: Vec<String>,
    /// Execution results
    pub results: Vec<serde_json::Value>,
    /// Errors encountered
    pub errors: Vec<(String, String, Vec<String>)>, // (name, message, traceback)
}

#[async_trait]
impl Protocol for JupyterProtocol {
    type Message = JupyterMessage;
    type OutputContext = JupyterOutputContext;

    fn decode(&self, parts: Vec<Vec<u8>>, channel: &str) -> Result<Self::Message, anyhow::Error> {
        self.wire.decode_message(&parts, channel)
    }

    fn encode(&self, msg: &Self::Message, channel: &str) -> Result<Vec<Vec<u8>>, anyhow::Error> {
        self.wire.encode_message(msg, channel)
    }

    fn transport_config(&self) -> TransportConfig {
        let mut channels = HashMap::new();

        // Shell channel (ROUTER for request-reply)
        channels.insert(
            "shell".to_string(),
            ChannelConfig {
                endpoint: self.connection_info.shell_port.to_string(),
                pattern: "router".to_string(),
            },
        );

        // IOPub channel (PUB for broadcasting)
        channels.insert(
            "iopub".to_string(),
            ChannelConfig {
                endpoint: self.connection_info.iopub_port.to_string(),
                pattern: "pub".to_string(),
            },
        );

        // Stdin channel (ROUTER for input requests)
        channels.insert(
            "stdin".to_string(),
            ChannelConfig {
                endpoint: self.connection_info.stdin_port.to_string(),
                pattern: "router".to_string(),
            },
        );

        // Control channel (ROUTER for control messages)
        channels.insert(
            "control".to_string(),
            ChannelConfig {
                endpoint: self.connection_info.control_port.to_string(),
                pattern: "router".to_string(),
            },
        );

        // Heartbeat channel (REP for keep-alive)
        channels.insert(
            "heartbeat".to_string(),
            ChannelConfig {
                endpoint: self.connection_info.hb_port.to_string(),
                pattern: "rep".to_string(),
            },
        );

        TransportConfig {
            transport_type: self.connection_info.transport.clone(),
            base_address: self.connection_info.ip.clone(),
            channels,
        }
    }

    fn name(&self) -> &'static str {
        "jupyter"
    }

    fn version(&self) -> &'static str {
        "5.3"
    }

    fn requires_reply(&self, msg: &Self::Message) -> bool {
        msg.header.msg_type.ends_with("_request")
    }

    fn create_reply(
        &self,
        request: &Self::Message,
        content: Value,
    ) -> Result<Self::Message, anyhow::Error> {
        // Create reply message type
        let reply_type = request.header.msg_type.replace("_request", "_reply");

        tracing::debug!("Creating reply for {}: {:?}", reply_type, content);

        // Create appropriate MessageContent based on reply type
        let content_enum = match reply_type.as_str() {
            "kernel_info_reply" => Self::create_kernel_info_reply_content(&content),
            "execute_reply" => Self::create_execute_reply_content(&content),
            _ => Self::create_default_reply_content(content, &reply_type),
        };

        let mut reply = JupyterMessage::new(&reply_type, content_enum);

        // Set parent header
        reply.parent_header = Some(request.header.clone());

        // Copy identities from request metadata for routing
        if let Some(identities) = request.metadata.get("__identities") {
            reply.metadata["__identities"] = identities.clone();
        }

        Ok(reply)
    }

    fn reply_channel(&self, msg: &Self::Message) -> &'static str {
        // Most replies go back on the same channel as the request
        // Except for IOPub which is for broadcasting only
        match msg.header.msg_type.as_str() {
            t if t.starts_with("stream") => "iopub",
            t if t.starts_with("display") => "iopub",
            t if t.starts_with("status") => "iopub",
            t if t.starts_with("execute_result") => "iopub",
            t if t.starts_with("error") => "iopub",
            _ => {
                // For requests, reply on the same channel
                // This is determined by where the request came from
                if msg.header.msg_type.contains("control") {
                    "control"
                } else if msg.header.msg_type.contains("stdin") {
                    "stdin"
                } else {
                    "shell" // Default to shell channel
                }
            }
        }
    }

    fn create_broadcast(
        &self,
        msg_type: &str,
        content: serde_json::Value,
        parent_msg: Option<&Self::Message>,
        kernel_id: &str,
    ) -> Result<Self::Message, anyhow::Error> {
        // Create header
        let header = MessageHeader {
            msg_id: uuid::Uuid::new_v4().to_string(),
            msg_type: msg_type.to_string(),
            username: "kernel".to_string(),
            session: kernel_id.to_string(),
            date: chrono::Utc::now(),
            version: "5.3".to_string(),
        };

        // Set parent header if provided
        let parent_header = parent_msg.map(|p| p.header.clone());

        // Convert content to MessageContent enum based on msg_type
        let message_content = match msg_type {
            "comm_open" => MessageContent::CommOpen {
                comm_id: content["comm_id"].as_str().unwrap_or("").to_string(),
                target_name: content["target_name"].as_str().unwrap_or("").to_string(),
                data: content
                    .get("data")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                metadata: None,
            },
            "comm_msg" => MessageContent::CommMsg {
                comm_id: content["comm_id"].as_str().unwrap_or("").to_string(),
                data: content
                    .get("data")
                    .cloned()
                    .unwrap_or_else(|| serde_json::json!({})),
                metadata: None,
            },
            "status" => {
                let state = match content["execution_state"].as_str().unwrap_or("idle") {
                    "busy" => ExecutionState::Busy,
                    "starting" => ExecutionState::Starting,
                    _ => ExecutionState::Idle,
                };
                MessageContent::Status {
                    execution_state: state,
                }
            }
            "stream" => MessageContent::Stream {
                name: match content["name"].as_str().unwrap_or("stdout") {
                    "stderr" => StreamType::Stderr,
                    _ => StreamType::Stdout,
                },
                text: content["text"].as_str().unwrap_or("").to_string(),
            },
            "execute_input" => MessageContent::ExecuteInput {
                code: content["code"].as_str().unwrap_or("").to_string(),
                execution_count: content["execution_count"].as_u64().unwrap_or(0) as u32,
            },
            "execute_result" => MessageContent::ExecuteResult {
                execution_count: content["execution_count"].as_u64().unwrap_or(0) as u32,
                data: content["data"]
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                metadata: content["metadata"]
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            },
            "error" => MessageContent::Error {
                ename: content["ename"].as_str().unwrap_or("Error").to_string(),
                evalue: content["evalue"].as_str().unwrap_or("").to_string(),
                traceback: content["traceback"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default(),
            },
            "display_data" => MessageContent::DisplayData {
                data: content["data"]
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                metadata: content["metadata"]
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
                transient: content.get("transient").cloned(),
            },
            _ => {
                // Unknown message type - this should be an error in production
                return Err(anyhow::anyhow!(
                    "Unknown IOPub message type '{}' in create_broadcast",
                    msg_type
                ));
            }
        };

        Ok(JupyterMessage {
            header,
            parent_header,
            metadata: serde_json::json!({}),
            content: message_content,
        })
    }

    fn create_output_context(&self) -> Self::OutputContext {
        JupyterOutputContext::default()
    }

    // === Message Lifecycle Methods ===

    fn create_execution_flow(
        &self,
        request: &Self::Message,
    ) -> crate::traits::ExecutionFlow<Self::Message> {
        use crate::traits::{ExecutionFlow, KernelStatus};

        // Only create execution flow for execute_request messages
        if request.header.msg_type != "execute_request" {
            return ExecutionFlow {
                pre_execution: Vec::new(),
                capture_output: false,
                post_execution: Vec::new(),
            };
        }

        // Extract execution count from request or use 0
        let execution_count = if let MessageContent::ExecuteRequest { .. } = &request.content {
            // TODO: Get actual execution count from kernel state
            1
        } else {
            0
        };

        // Pre-execution messages
        let mut pre_execution = Vec::new();

        // Send status: busy on IOPub
        if let Ok(status_msg) = self.create_status_message(KernelStatus::Busy) {
            if let Some(parent) = Some(request.header.clone()) {
                let mut msg = status_msg;
                msg.parent_header = Some(parent);
                pre_execution.push(("iopub".to_string(), msg));
            }
        }

        // Send execute_input on IOPub
        if let MessageContent::ExecuteRequest { code, .. } = &request.content {
            if let Ok(input_msg) = self.create_execute_input_message(code, execution_count) {
                if let Some(parent) = Some(request.header.clone()) {
                    let mut msg = input_msg;
                    msg.parent_header = Some(parent);
                    pre_execution.push(("iopub".to_string(), msg));
                }
            }
        }

        // Post-execution messages (status: idle will be sent after reply)
        let post_execution = Vec::new();

        ExecutionFlow {
            pre_execution,
            capture_output: true, // Capture output during execution
            post_execution,
        }
    }

    fn create_status_message(
        &self,
        status: crate::traits::KernelStatus,
    ) -> anyhow::Result<Self::Message> {
        use crate::traits::KernelStatus;

        let execution_state = match status {
            KernelStatus::Idle => ExecutionState::Idle,
            KernelStatus::Busy => ExecutionState::Busy,
            KernelStatus::Starting => ExecutionState::Starting,
        };

        let content = MessageContent::Status { execution_state };

        Ok(JupyterMessage {
            header: MessageHeader {
                msg_id: uuid::Uuid::new_v4().to_string(),
                msg_type: "status".to_string(),
                username: "kernel".to_string(),
                session: uuid::Uuid::new_v4().to_string(), // TODO: Use actual session
                date: chrono::Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::json!({}),
            content,
        })
    }

    fn create_execute_input_message(
        &self,
        code: &str,
        count: u32,
    ) -> anyhow::Result<Self::Message> {
        let content = MessageContent::ExecuteInput {
            code: code.to_string(),
            execution_count: count,
        };

        Ok(JupyterMessage {
            header: MessageHeader {
                msg_id: uuid::Uuid::new_v4().to_string(),
                msg_type: "execute_input".to_string(),
                username: "kernel".to_string(),
                session: uuid::Uuid::new_v4().to_string(), // TODO: Use actual session
                date: chrono::Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::json!({}),
            content,
        })
    }

    fn create_stream_message(
        &self,
        stream: crate::traits::StreamData,
    ) -> anyhow::Result<Self::Message> {
        use crate::traits::StreamType as TraitStreamType;

        let name = match stream.stream_type {
            TraitStreamType::Stdout => StreamType::Stdout,
            TraitStreamType::Stderr => StreamType::Stderr,
        };

        let content = MessageContent::Stream {
            name,
            text: stream.text,
        };

        Ok(JupyterMessage {
            header: MessageHeader {
                msg_id: uuid::Uuid::new_v4().to_string(),
                msg_type: "stream".to_string(),
                username: "kernel".to_string(),
                session: uuid::Uuid::new_v4().to_string(), // TODO: Use actual session
                date: chrono::Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::json!({}),
            content,
        })
    }

    fn create_execute_result(
        &self,
        result: crate::traits::ExecutionResult,
    ) -> anyhow::Result<Self::Message> {
        // Create data dict with text/plain representation
        let mut data = HashMap::new();
        if let Some(value) = result.result_value {
            data.insert("text/plain".to_string(), value);
        }

        let content = MessageContent::ExecuteResult {
            execution_count: result.execution_count,
            data,
            metadata: HashMap::new(),
        };

        Ok(JupyterMessage {
            header: MessageHeader {
                msg_id: uuid::Uuid::new_v4().to_string(),
                msg_type: "execute_result".to_string(),
                username: "kernel".to_string(),
                session: uuid::Uuid::new_v4().to_string(), // TODO: Use actual session
                date: chrono::Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::json!({}),
            content,
        })
    }

    fn create_error_message(
        &self,
        error: crate::traits::ExecutionError,
    ) -> anyhow::Result<Self::Message> {
        let content = MessageContent::Error {
            ename: error.name,
            evalue: error.message,
            traceback: error.traceback,
        };

        Ok(JupyterMessage {
            header: MessageHeader {
                msg_id: uuid::Uuid::new_v4().to_string(),
                msg_type: "error".to_string(),
                username: "kernel".to_string(),
                session: uuid::Uuid::new_v4().to_string(), // TODO: Use actual session
                date: chrono::Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::json!({}),
            content,
        })
    }

    fn handle_output(&self, ctx: &mut Self::OutputContext, output: crate::traits::OutputChunk) {
        use crate::traits::OutputChunk;

        match output {
            OutputChunk::Stdout(text) => {
                ctx.stdout_buffer.push(text);
            }
            OutputChunk::Stderr(text) => {
                ctx.stderr_buffer.push(text);
            }
            OutputChunk::Result(value) => {
                ctx.results.push(value);
            }
            OutputChunk::Error(error) => {
                ctx.errors
                    .push((error.name, error.message, error.traceback));
            }
        }
    }

    fn flush_output(&self, ctx: Self::OutputContext) -> Vec<(String, Self::Message)> {
        use crate::traits::{StreamData, StreamType as TraitStreamType};

        let mut messages = Vec::new();

        // Flush stdout buffer
        if !ctx.stdout_buffer.is_empty() {
            let text = ctx.stdout_buffer.join("");
            if !text.is_empty() {
                if let Ok(msg) = self.create_stream_message(StreamData {
                    stream_type: TraitStreamType::Stdout,
                    text,
                }) {
                    messages.push(("iopub".to_string(), msg));
                }
            }
        }

        // Flush stderr buffer
        if !ctx.stderr_buffer.is_empty() {
            let text = ctx.stderr_buffer.join("");
            if !text.is_empty() {
                if let Ok(msg) = self.create_stream_message(StreamData {
                    stream_type: TraitStreamType::Stderr,
                    text,
                }) {
                    messages.push(("iopub".to_string(), msg));
                }
            }
        }

        // Send any results
        for (i, value) in ctx.results.into_iter().enumerate() {
            // Use saturating conversion to avoid overflow on 64-bit systems
            let count = u32::try_from(i).unwrap_or(u32::MAX).saturating_add(1);
            if let Ok(msg) = self.create_execute_result(crate::traits::ExecutionResult {
                output: Vec::new(),
                errors: Vec::new(),
                result_value: Some(value),
                execution_count: count, // TODO: Get actual count
            }) {
                messages.push(("iopub".to_string(), msg));
            }
        }

        // Send any errors
        for (name, message, traceback) in ctx.errors {
            if let Ok(msg) = self.create_error_message(crate::traits::ExecutionError {
                name,
                message,
                traceback,
            }) {
                messages.push(("iopub".to_string(), msg));
            }
        }

        messages
    }
}

impl JupyterProtocol {
    fn create_kernel_info_reply_content(content: &Value) -> MessageContent {
        // Extract llmspell_session_metadata if present
        let session_metadata = content.get("llmspell_session_metadata").cloned();

        MessageContent::KernelInfoReply {
            status: content["status"].as_str().unwrap_or("ok").to_string(),
            protocol_version: content["protocol_version"]
                .as_str()
                .unwrap_or("5.3")
                .to_string(),
            implementation: content["implementation"]
                .as_str()
                .unwrap_or("llmspell")
                .to_string(),
            implementation_version: content["implementation_version"]
                .as_str()
                .unwrap_or("0.8.0")
                .to_string(),
            language_info: serde_json::from_value(content["language_info"].clone()).unwrap_or_else(
                |e| {
                    tracing::warn!(
                        "Failed to parse language_info, using generic fallback: {}",
                        e
                    );
                    LanguageInfo {
                        name: "unknown".to_string(),
                        version: "0.0.0".to_string(),
                        mimetype: "text/plain".to_string(),
                        file_extension: ".txt".to_string(),
                        pygments_lexer: None,
                        codemirror_mode: None,
                        nbconvert_exporter: None,
                    }
                },
            ),
            banner: content["banner"]
                .as_str()
                .unwrap_or("LLMSpell Kernel")
                .to_string(),
            help_links: vec![],
            llmspell_session_metadata: session_metadata,
        }
    }

    fn create_execute_reply_content(content: &Value) -> MessageContent {
        // Parse the raw JSON into ExecuteReply
        let status = content["status"].as_str().unwrap_or("ok");
        let execution_count =
            u32::try_from(content["execution_count"].as_u64().unwrap_or(0)).unwrap_or(0);

        if status == "error" {
            MessageContent::ExecuteReply {
                status: ExecutionStatus::Error,
                execution_count,
                user_expressions: None,
                payload: None,
                ename: content["ename"]
                    .as_str()
                    .map(std::string::ToString::to_string),
                evalue: content["evalue"]
                    .as_str()
                    .map(std::string::ToString::to_string),
                traceback: content["traceback"].as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                        .collect()
                }),
            }
        } else {
            MessageContent::ExecuteReply {
                status: ExecutionStatus::Ok,
                execution_count,
                user_expressions: content
                    .get("user_expressions")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                payload: content
                    .get("payload")
                    .and_then(|v| serde_json::from_value(v.clone()).ok()),
                ename: None,
                evalue: None,
                traceback: None,
            }
        }
    }

    fn create_default_reply_content(content: Value, reply_type: &str) -> MessageContent {
        // Try to parse directly or fall back to Status
        serde_json::from_value::<MessageContent>(content).unwrap_or_else(|e| {
            tracing::warn!(
                "Failed to parse content as MessageContent for {}: {}",
                reply_type,
                e
            );
            MessageContent::Status {
                execution_state: ExecutionState::Idle,
            }
        })
    }
}
