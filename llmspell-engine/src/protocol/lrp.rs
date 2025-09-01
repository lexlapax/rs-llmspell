//! `LLMSpell` REPL Protocol (LRP) definitions
//!
//! Defines the LRP protocol for REPL communication between kernel and clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Common message header for all protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Unique message identifier
    pub msg_id: String,
    /// Session identifier
    pub session: String,
    /// Username of the sender
    pub username: String,
    /// Message creation timestamp
    pub date: DateTime<Utc>,
    /// Message type
    pub msg_type: String,
    /// Protocol version
    pub version: String,
}

impl Default for MessageHeader {
    fn default() -> Self {
        Self {
            msg_id: Uuid::new_v4().to_string(),
            session: String::new(),
            username: String::from("anonymous"),
            date: Utc::now(),
            msg_type: String::new(),
            version: String::from("1.0"),
        }
    }
}

/// `LLMSpell` REPL Protocol (LRP) request messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum LRPRequest {
    /// Execute code request
    ExecuteRequest {
        /// Code to execute
        code: String,
        /// Whether to suppress output
        silent: bool,
        /// Whether to store in history
        store_history: bool,
        /// User expressions to evaluate
        user_expressions: Option<Value>,
        /// Whether to allow stdin
        allow_stdin: bool,
        /// Whether to stop on error
        stop_on_error: bool,
    },
    /// Inspect object request
    InspectRequest {
        /// Code to inspect
        code: String,
        /// Cursor position in code
        cursor_pos: usize,
        /// Detail level (0 or 1)
        detail_level: u8,
    },
    /// Complete code request
    CompleteRequest {
        /// Code to complete
        code: String,
        /// Cursor position in code
        cursor_pos: usize,
    },
    /// History request
    HistoryRequest {
        /// Whether to get output
        output: bool,
        /// Whether to get raw input
        raw: bool,
        /// History access type
        hist_access_type: String,
        /// Session number
        session: Option<i32>,
        /// Start index
        start: Option<i32>,
        /// Stop index
        stop: Option<i32>,
        /// Number of entries
        n: Option<i32>,
        /// Pattern to search
        pattern: Option<String>,
        /// Whether to get unique entries
        unique: bool,
    },
    /// Is complete request
    IsCompleteRequest {
        /// Code to check
        code: String,
    },
    /// Connect request
    ConnectRequest,
    /// Comm info request
    CommInfoRequest {
        /// Target name
        target_name: Option<String>,
    },
    /// Kernel info request
    KernelInfoRequest,
    /// Shutdown request
    ShutdownRequest {
        /// Whether to restart
        restart: bool,
    },
    /// Interrupt request
    InterruptRequest,
}

/// `LLMSpell` REPL Protocol (LRP) response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum LRPResponse {
    /// Execute reply
    ExecuteReply {
        /// Execution status
        status: String,
        /// Execution count
        execution_count: u32,
        /// User expressions results
        user_expressions: Option<Value>,
        /// Payload
        payload: Option<Vec<Value>>,
    },
    /// Inspect reply
    InspectReply {
        /// Status
        status: String,
        /// Whether object was found
        found: bool,
        /// Object data
        data: Option<Value>,
        /// Metadata
        metadata: Option<Value>,
    },
    /// Complete reply
    CompleteReply {
        /// Completion matches
        matches: Vec<String>,
        /// Cursor start position
        cursor_start: usize,
        /// Cursor end position
        cursor_end: usize,
        /// Metadata
        metadata: Option<Value>,
        /// Status
        status: String,
    },
    /// History reply
    HistoryReply {
        /// History entries
        history: Vec<HistoryEntry>,
    },
    /// Is complete reply
    IsCompleteReply {
        /// Status (complete, incomplete, invalid, unknown)
        status: String,
        /// Indentation if incomplete
        indent: String,
    },
    /// Connect reply
    ConnectReply {
        /// Shell port
        shell_port: u16,
        /// `IOPub` port
        iopub_port: u16,
        /// Stdin port
        stdin_port: u16,
        /// Control port
        control_port: u16,
        /// Heartbeat port
        hb_port: u16,
    },
    /// Comm info reply
    CommInfoReply {
        /// Available comms
        comms: Value,
    },
    /// Kernel info reply
    KernelInfoReply {
        /// Protocol version
        protocol_version: String,
        /// Implementation name
        implementation: String,
        /// Implementation version
        implementation_version: String,
        /// Language info
        language_info: LanguageInfo,
        /// Banner
        banner: String,
        /// Whether debugger is available
        debugger: bool,
        /// Help links
        help_links: Vec<HelpLink>,
    },
    /// Shutdown reply
    ShutdownReply {
        /// Whether to restart
        restart: bool,
    },
    /// Interrupt reply
    InterruptReply,
}

/// Language information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Language name
    pub name: String,
    /// Language version
    pub version: String,
    /// MIME type
    pub mimetype: String,
    /// File extension
    pub file_extension: String,
    /// Pygments lexer name
    pub pygments_lexer: Option<String>,
    /// `CodeMirror` mode
    pub codemirror_mode: Option<String>,
    /// nbconvert exporter
    pub nbconvert_exporter: Option<String>,
}

/// Help link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpLink {
    /// Link text
    pub text: String,
    /// Link URL
    pub url: String,
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Session number
    pub session: i32,
    /// Line number
    pub line: i32,
    /// Input
    pub input: String,
    /// Output
    pub output: Option<String>,
}
