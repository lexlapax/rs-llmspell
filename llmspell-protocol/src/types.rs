//! LRP/LDP Protocol definitions
//!
//! Defines the `LLMSpell` REPL Protocol (LRP) and `LLMSpell` Debug Protocol (LDP)
//! for communication between kernel and clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
        user_expressions: Option<serde_json::Value>,
        /// Allow stdin during execution
        allow_stdin: bool,
        /// Stop execution on error
        stop_on_error: bool,
    },

    /// Code completion request
    CompleteRequest {
        /// Code context
        code: String,
        /// Cursor position in code
        cursor_pos: usize,
    },

    /// Object inspection request
    InspectRequest {
        /// Code context
        code: String,
        /// Cursor position in code
        cursor_pos: usize,
        /// Level of detail (0 or 1)
        detail_level: u8,
    },

    /// History request
    HistoryRequest {
        /// Output format
        output: bool,
        /// Include raw input
        raw: bool,
        /// History access type
        hist_access_type: String,
        /// Session number
        session: Option<u32>,
        /// Start index
        start: Option<u32>,
        /// Stop index
        stop: Option<u32>,
        /// Number of entries
        n: Option<u32>,
        /// Search pattern
        pattern: Option<String>,
        /// Unique history entries only
        unique: bool,
    },

    /// Kernel info request
    KernelInfoRequest,

    /// Shutdown request
    ShutdownRequest {
        /// Whether to restart after shutdown
        restart: bool,
    },

    /// Interrupt request
    InterruptRequest,

    /// Check if code is complete
    IsCompleteRequest {
        /// Code to check
        code: String,
    },

    /// Comm info request
    CommInfoRequest {
        /// Target comm name (optional)
        target_name: Option<String>,
    },

    /// Connect request to get port information
    ConnectRequest,
}

/// `LLMSpell` REPL Protocol (LRP) response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum LRPResponse {
    /// Execute reply
    ExecuteReply {
        /// Execution status
        status: String,
        /// Execution counter
        execution_count: u32,
        /// User expressions results
        user_expressions: Option<serde_json::Value>,
        /// Payload for special actions
        payload: Option<Vec<serde_json::Value>>,
    },

    /// Completion reply
    CompleteReply {
        /// List of completions
        matches: Vec<String>,
        /// Start position of completion
        cursor_start: usize,
        /// End position of completion
        cursor_end: usize,
        /// Metadata about completions
        metadata: Option<serde_json::Value>,
        /// Completion status
        status: String,
    },

    /// Inspection reply
    InspectReply {
        /// Inspection status
        status: String,
        /// Found object
        found: bool,
        /// Inspection data
        data: Option<serde_json::Value>,
        /// Metadata
        metadata: Option<serde_json::Value>,
    },

    /// History reply
    HistoryReply {
        /// History entries
        history: Vec<HistoryEntry>,
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
        /// Banner text
        banner: String,
        /// Debug support
        debugger: bool,
        /// Available help links
        help_links: Vec<HelpLink>,
    },

    /// Shutdown reply
    ShutdownReply {
        /// Whether kernel will restart
        restart: bool,
    },

    /// Interrupt reply
    InterruptReply,

    /// Is complete reply
    IsCompleteReply {
        /// Completion status: "complete", "incomplete", "invalid", or "unknown"
        status: String,
        /// Indentation string if incomplete
        indent: String,
    },

    /// Comm info reply
    CommInfoReply {
        /// Dictionary of comm info
        comms: serde_json::Value,
    },

    /// Connect reply with port information
    ConnectReply {
        /// Shell channel port
        shell_port: u16,
        /// IOPub channel port
        iopub_port: u16,
        /// Stdin channel port
        stdin_port: u16,
        /// Control channel port
        control_port: u16,
        /// Heartbeat channel port
        hb_port: u16,
    },
}

/// History entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    /// Session number
    pub session: u32,
    /// Line number
    pub line: u32,
    /// Input text
    pub input: String,
    /// Output text (if available)
    pub output: Option<String>,
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
    /// Pygments lexer (for syntax highlighting)
    pub pygments_lexer: Option<String>,
    /// Code mirror mode
    pub codemirror_mode: Option<String>,
    /// Notebook converter version
    pub nbconvert_exporter: Option<String>,
}

/// Help link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpLink {
    /// Display text
    pub text: String,
    /// URL
    pub url: String,
}

/// `LLMSpell` Debug Protocol (LDP) request messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum LDPRequest {
    /// Initialize debugger request
    InitializeRequest {
        /// Client ID
        client_id: String,
        /// Client name
        client_name: Option<String>,
        /// Adapter ID
        adapter_id: Option<String>,
    },

    /// Set breakpoint request
    SetBreakpointRequest {
        /// File path
        file: String,
        /// Line number
        line: u32,
        /// Breakpoint condition
        condition: Option<String>,
        /// Hit count
        hit_count: Option<u32>,
        /// Ignore count
        ignore_count: Option<u32>,
    },

    /// Remove breakpoint request
    RemoveBreakpointRequest {
        /// Breakpoint ID
        breakpoint_id: String,
    },

    /// Step request (step into)
    StepRequest,

    /// Next request (step over)
    NextRequest,

    /// Continue request
    ContinueRequest,

    /// Pause request
    PauseRequest,

    /// Get variables request
    VariablesRequest {
        /// Stack frame ID
        frame_id: Option<String>,
        /// Variable filter
        filter: Option<String>,
        /// Start index
        start: Option<u32>,
        /// Count
        count: Option<u32>,
    },

    /// Get stack trace request
    StackTraceRequest {
        /// Thread ID
        thread_id: Option<String>,
        /// Start frame
        start_frame: Option<u32>,
        /// Levels to retrieve
        levels: Option<u32>,
    },

    /// Evaluate expression request
    EvaluateRequest {
        /// Expression to evaluate
        expression: String,
        /// Stack frame ID for context
        frame_id: Option<String>,
        /// Evaluation context
        context: Option<String>,
    },
}

/// `LLMSpell` Debug Protocol (LDP) response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum LDPResponse {
    /// Initialize debugger response
    InitializeResponse {
        /// Debugger capabilities
        capabilities: serde_json::Value,
    },

    /// Error response
    ErrorResponse {
        /// Error message
        message: String,
    },

    /// Set breakpoint reply
    SetBreakpointReply {
        /// Breakpoint ID
        breakpoint_id: String,
        /// Whether breakpoint was verified
        verified: bool,
        /// Actual line where breakpoint was set
        line: u32,
    },

    /// Remove breakpoint reply
    RemoveBreakpointReply {
        /// Success status
        success: bool,
    },

    /// Step reply
    StepReply {
        /// New location after step
        location: Location,
    },

    /// Next reply
    NextReply {
        /// New location after next
        location: Location,
    },

    /// Continue reply
    ContinueReply,

    /// Pause reply
    PauseReply {
        /// Location where paused
        location: Location,
    },

    /// Variables reply
    VariablesReply {
        /// List of variables
        variables: Vec<Variable>,
    },

    /// Stack trace reply
    StackTraceReply {
        /// Stack frames
        stack_frames: Vec<StackFrame>,
        /// Total frames available
        total_frames: Option<u32>,
    },

    /// Evaluate reply
    EvaluateReply {
        /// Evaluation result
        result: String,
        /// Result type
        type_: Option<String>,
        /// Variable reference for complex types
        variable_reference: Option<u32>,
    },
}

/// Debug location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    /// File path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: Option<u32>,
}

/// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
    /// Variable type
    pub type_: Option<String>,
    /// Reference for complex types
    pub variable_reference: Option<u32>,
    /// Named child variables count
    pub named_variables: Option<u32>,
    /// Indexed child variables count
    pub indexed_variables: Option<u32>,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame ID
    pub id: String,
    /// Frame name
    pub name: String,
    /// Source location
    pub location: Location,
    /// Whether this is from user code
    pub is_user_code: bool,
}
