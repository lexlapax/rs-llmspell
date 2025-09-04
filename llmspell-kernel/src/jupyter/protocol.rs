//! Jupyter protocol message types for LLMSpell kernel
//!
//! Implements standard Jupyter messaging protocol with extensions for:
//! - DAP (Debug Adapter Protocol) support via debug messages
//! - Daemon management for kernel lifecycle control
//! - LLMSpell-specific script execution features

use serde::{Deserialize, Serialize};
use serde_json::{Value, Map};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
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
    },

    #[serde(rename = "execute_reply")]
    ExecuteReply {
        status: ExecutionStatus,
        execution_count: u32,
        user_expressions: Option<HashMap<String, Value>>,
        payload: Option<Vec<Value>>,
    },

    #[serde(rename = "execute_input")]
    ExecuteInput {
        code: String,
        execution_count: u32,
    },

    #[serde(rename = "execute_result")]  
    ExecuteResult {
        execution_count: u32,
        data: HashMap<String, Value>,
        metadata: HashMap<String, Value>,
    },

    // === OUTPUT STREAMS ===
    #[serde(rename = "stream")]
    Stream {
        name: StreamType,
        text: String,
    },

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
    Status {
        execution_state: ExecutionState,
    },

    // === INPUT REQUESTS ===
    #[serde(rename = "input_request")]
    InputRequest {
        prompt: String,
        password: bool,
    },

    #[serde(rename = "input_reply")]
    InputReply {
        value: String,
    },

    // === CONTROL MESSAGES ===
    #[serde(rename = "shutdown_request")]
    ShutdownRequest {
        restart: bool,
    },

    #[serde(rename = "shutdown_reply")]
    ShutdownReply {
        status: String,
        restart: bool,
    },

    #[serde(rename = "interrupt_request")]
    InterruptRequest {},

    #[serde(rename = "interrupt_reply")]
    InterruptReply {
        status: String,
    },

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
    CompleteRequest {
        code: String,
        cursor_pos: u32,
    },

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
}

/// Execution status for execute_reply messages
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Stream types for output
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StreamType {
    Stdout,
    Stderr,
}

/// Language information for kernel_info_reply
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

/// Help link for kernel_info_reply
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
    pub fn new(msg_type: &str, content: MessageContent) -> Self {
        Self {
            header: MessageHeader::new(msg_type),
            parent_header: None,
            metadata: Value::Object(Map::new()),
            content,
        }
    }

    /// Create reply message with parent header
    pub fn reply(parent: &JupyterMessage, msg_type: &str, content: MessageContent) -> Self {
        Self {
            header: MessageHeader::new(msg_type),
            parent_header: Some(parent.header.clone()),
            metadata: Value::Object(Map::new()),
            content,
        }
    }

    /// Get message type from header
    pub fn msg_type(&self) -> &str {
        &self.header.msg_type
    }

    /// Check if this is a request message
    pub fn is_request(&self) -> bool {
        self.header.msg_type.ends_with("_request")
    }

    /// Check if this is a reply message  
    pub fn is_reply(&self) -> bool {
        self.header.msg_type.ends_with("_reply")
    }
}

impl MessageHeader {
    /// Create new header with generated ID and current timestamp
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