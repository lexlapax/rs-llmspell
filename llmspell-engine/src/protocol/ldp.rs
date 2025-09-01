//! `LLMSpell` Debug Protocol (LDP) definitions
//!
//! Defines the LDP protocol for debugging communication between kernel and clients.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `LLMSpell` Debug Protocol (LDP) request messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum LDPRequest {
    /// Initialize debug session
    InitializeRequest {
        /// Client ID
        client_id: String,
        /// Client name
        client_name: String,
        /// Adapter ID
        adapter_id: String,
        /// Locale
        locale: Option<String>,
        /// Lines start at 1
        lines_start_at_1: bool,
        /// Columns start at 1
        columns_start_at_1: bool,
        /// Path format
        path_format: Option<String>,
        /// Supports variable type
        supports_variable_type: bool,
        /// Supports variable paging
        supports_variable_paging: bool,
        /// Supports run in terminal request
        supports_run_in_terminal_request: bool,
        /// Supports memory references
        supports_memory_references: bool,
        /// Supports progress reporting
        supports_progress_reporting: bool,
        /// Supports invalidated event
        supports_invalidated_event: bool,
    },
    /// Set breakpoints
    SetBreakpointsRequest {
        /// Source file
        source: Source,
        /// Breakpoint lines
        lines: Vec<u32>,
        /// Breakpoints
        breakpoints: Option<Vec<SourceBreakpoint>>,
        /// Source modified
        source_modified: Option<bool>,
    },
    /// Set function breakpoints
    SetFunctionBreakpointsRequest {
        /// Function breakpoints
        breakpoints: Vec<FunctionBreakpoint>,
    },
    /// Set exception breakpoints
    SetExceptionBreakpointsRequest {
        /// Exception filters
        filters: Vec<String>,
        /// Filter options
        filter_options: Option<Vec<ExceptionFilterOptions>>,
        /// Exception options
        exception_options: Option<Vec<ExceptionOptions>>,
    },
    /// Configuration done
    ConfigurationDoneRequest,
    /// Launch
    LaunchRequest {
        /// Launch arguments
        args: Value,
    },
    /// Attach
    AttachRequest {
        /// Attach arguments
        args: Value,
    },
    /// Restart
    RestartRequest {
        /// Restart arguments
        arguments: Option<Value>,
    },
    /// Disconnect
    DisconnectRequest {
        /// Whether to restart
        restart: Option<bool>,
        /// Whether to terminate debuggee
        terminate_debuggee: Option<bool>,
        /// Whether to suspend debuggee
        suspend_debuggee: Option<bool>,
    },
    /// Terminate
    TerminateRequest {
        /// Whether to restart
        restart: Option<bool>,
    },
    /// Continue
    ContinueRequest {
        /// Thread ID
        thread_id: i32,
        /// Whether to continue all threads
        all_threads: Option<bool>,
    },
    /// Next
    NextRequest {
        /// Thread ID
        thread_id: i32,
        /// Granularity
        granularity: Option<SteppingGranularity>,
    },
    /// Step in
    StepInRequest {
        /// Thread ID
        thread_id: i32,
        /// Target ID
        target_id: Option<i32>,
        /// Granularity
        granularity: Option<SteppingGranularity>,
    },
    /// Step out
    StepOutRequest {
        /// Thread ID
        thread_id: i32,
        /// Granularity
        granularity: Option<SteppingGranularity>,
    },
    /// Step back
    StepBackRequest {
        /// Thread ID
        thread_id: i32,
        /// Granularity
        granularity: Option<SteppingGranularity>,
    },
    /// Reverse continue
    ReverseContinueRequest {
        /// Thread ID
        thread_id: i32,
        /// Whether to continue all threads
        all_threads: Option<bool>,
    },
    /// Pause
    PauseRequest {
        /// Thread ID
        thread_id: i32,
    },
    /// Stack trace
    StackTraceRequest {
        /// Thread ID
        thread_id: i32,
        /// Start frame
        start_frame: Option<i32>,
        /// Levels
        levels: Option<i32>,
        /// Format
        format: Option<StackFrameFormat>,
    },
    /// Scopes
    ScopesRequest {
        /// Frame ID
        frame_id: i32,
    },
    /// Variables
    VariablesRequest {
        /// Variable reference
        variables_reference: i32,
        /// Filter
        filter: Option<String>,
        /// Start
        start: Option<i32>,
        /// Count
        count: Option<i32>,
        /// Format
        format: Option<ValueFormat>,
    },
    /// Set variable
    SetVariableRequest {
        /// Variable reference
        variables_reference: i32,
        /// Name
        name: String,
        /// Value
        value: String,
        /// Format
        format: Option<ValueFormat>,
    },
    /// Evaluate
    EvaluateRequest {
        /// Expression
        expression: String,
        /// Frame ID
        frame_id: Option<i32>,
        /// Context
        context: Option<String>,
        /// Format
        format: Option<ValueFormat>,
    },
}

/// `LLMSpell` Debug Protocol (LDP) response messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command")]
pub enum LDPResponse {
    /// Initialize response
    InitializeResponse {
        /// Capabilities
        capabilities: Value,
    },
    /// Set breakpoints response
    SetBreakpointsResponse {
        /// Breakpoints
        breakpoints: Vec<Breakpoint>,
    },
    /// Set function breakpoints response
    SetFunctionBreakpointsResponse {
        /// Breakpoints
        breakpoints: Vec<Breakpoint>,
    },
    /// Set exception breakpoints response
    SetExceptionBreakpointsResponse {
        /// Breakpoints
        breakpoints: Option<Vec<Breakpoint>>,
    },
    /// Configuration done response
    ConfigurationDoneResponse,
    /// Launch response
    LaunchResponse,
    /// Attach response
    AttachResponse,
    /// Restart response
    RestartResponse,
    /// Disconnect response
    DisconnectResponse,
    /// Terminate response
    TerminateResponse,
    /// Continue response
    ContinueResponse {
        /// Whether all threads continued
        all_threads_continued: Option<bool>,
    },
    /// Next response
    NextResponse,
    /// Step in response
    StepInResponse,
    /// Step out response
    StepOutResponse,
    /// Step back response
    StepBackResponse,
    /// Reverse continue response
    ReverseContinueResponse,
    /// Pause response
    PauseResponse,
    /// Stack trace response
    StackTraceResponse {
        /// Stack frames
        stack_frames: Vec<StackFrame>,
        /// Total frames
        total_frames: Option<i32>,
    },
    /// Scopes response
    ScopesResponse {
        /// Scopes
        scopes: Vec<Scope>,
    },
    /// Variables response
    VariablesResponse {
        /// Variables
        variables: Vec<Variable>,
    },
    /// Set variable response
    SetVariableResponse {
        /// Value
        value: String,
        /// Type
        r#type: Option<String>,
        /// Variable reference
        variables_reference: Option<i32>,
        /// Named variables
        named_variables: Option<i32>,
        /// Indexed variables
        indexed_variables: Option<i32>,
    },
    /// Evaluate response
    EvaluateResponse {
        /// Result
        result: String,
        /// Type
        r#type: Option<String>,
        /// Presentation hint
        presentation_hint: Option<VariablePresentationHint>,
        /// Variable reference
        variables_reference: i32,
        /// Named variables
        named_variables: Option<i32>,
        /// Indexed variables
        indexed_variables: Option<i32>,
        /// Memory reference
        memory_reference: Option<String>,
    },
}

// Supporting types for LDP

/// Source file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    /// Name
    pub name: Option<String>,
    /// Path
    pub path: Option<String>,
    /// Source reference
    pub source_reference: Option<i32>,
    /// Presentation hint
    pub presentation_hint: Option<String>,
    /// Origin
    pub origin: Option<String>,
    /// Sources
    pub sources: Option<Vec<Source>>,
    /// Adapter data
    pub adapter_data: Option<Value>,
    /// Checksums
    pub checksums: Option<Vec<Checksum>>,
}

/// Source breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceBreakpoint {
    /// Line
    pub line: u32,
    /// Column
    pub column: Option<u32>,
    /// Condition
    pub condition: Option<String>,
    /// Hit condition
    pub hit_condition: Option<String>,
    /// Log message
    pub log_message: Option<String>,
}

/// Function breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBreakpoint {
    /// Name
    pub name: String,
    /// Condition
    pub condition: Option<String>,
    /// Hit condition
    pub hit_condition: Option<String>,
}

/// Exception filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionFilterOptions {
    /// Filter ID
    pub filter_id: String,
    /// Condition
    pub condition: Option<String>,
}

/// Exception options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionOptions {
    /// Path
    pub path: Option<Vec<ExceptionPathSegment>>,
    /// Break mode
    pub break_mode: String,
}

/// Exception path segment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionPathSegment {
    /// Negate
    pub negate: Option<bool>,
    /// Names
    pub names: Vec<String>,
}

/// Stepping granularity
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SteppingGranularity {
    Statement,
    Line,
    Instruction,
}

/// Stack frame format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrameFormat {
    /// Parameters
    pub parameters: Option<bool>,
    /// Parameter types
    pub parameter_types: Option<bool>,
    /// Parameter names
    pub parameter_names: Option<bool>,
    /// Parameter values
    pub parameter_values: Option<bool>,
    /// Line
    pub line: Option<bool>,
    /// Module
    pub module: Option<bool>,
    /// Include all
    pub include_all: Option<bool>,
}

/// Value format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueFormat {
    /// Hex
    pub hex: Option<bool>,
}

/// Breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// ID
    pub id: Option<i32>,
    /// Verified
    pub verified: bool,
    /// Message
    pub message: Option<String>,
    /// Source
    pub source: Option<Source>,
    /// Line
    pub line: Option<u32>,
    /// Column
    pub column: Option<u32>,
    /// End line
    pub end_line: Option<u32>,
    /// End column
    pub end_column: Option<u32>,
    /// Instruction reference
    pub instruction_reference: Option<String>,
    /// Offset
    pub offset: Option<i32>,
}

/// Stack frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// ID
    pub id: i32,
    /// Name
    pub name: String,
    /// Source
    pub source: Option<Source>,
    /// Line
    pub line: u32,
    /// Column
    pub column: u32,
    /// End line
    pub end_line: Option<u32>,
    /// End column
    pub end_column: Option<u32>,
    /// Can restart
    pub can_restart: Option<bool>,
    /// Instruction pointer reference
    pub instruction_pointer_reference: Option<String>,
    /// Module ID
    pub module_id: Option<Value>,
    /// Presentation hint
    pub presentation_hint: Option<String>,
}

/// Scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Name
    pub name: String,
    /// Presentation hint
    pub presentation_hint: Option<String>,
    /// Variables reference
    pub variables_reference: i32,
    /// Named variables
    pub named_variables: Option<i32>,
    /// Indexed variables
    pub indexed_variables: Option<i32>,
    /// Expensive
    pub expensive: bool,
    /// Source
    pub source: Option<Source>,
    /// Line
    pub line: Option<u32>,
    /// Column
    pub column: Option<u32>,
    /// End line
    pub end_line: Option<u32>,
    /// End column
    pub end_column: Option<u32>,
}

/// Variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Name
    pub name: String,
    /// Value
    pub value: String,
    /// Type
    pub r#type: Option<String>,
    /// Presentation hint
    pub presentation_hint: Option<VariablePresentationHint>,
    /// Evaluate name
    pub evaluate_name: Option<String>,
    /// Variables reference
    pub variables_reference: i32,
    /// Named variables
    pub named_variables: Option<i32>,
    /// Indexed variables
    pub indexed_variables: Option<i32>,
    /// Memory reference
    pub memory_reference: Option<String>,
}

/// Variable presentation hint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariablePresentationHint {
    /// Kind
    pub kind: Option<String>,
    /// Attributes
    pub attributes: Option<Vec<String>>,
    /// Visibility
    pub visibility: Option<String>,
}

/// Checksum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checksum {
    /// Algorithm
    pub algorithm: String,
    /// Checksum
    pub checksum: String,
}
