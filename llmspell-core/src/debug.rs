//! Debug protocol types and traits for protocol-first debugging
//!
//! This module provides the core abstraction for debug capabilities that can be
//! implemented by different debug infrastructure components. The protocol-agnostic
//! design allows seamless transition between local and remote debugging.

use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Core trait for debug capabilities
///
/// This trait provides a protocol-agnostic interface for debug operations.
/// Implementations can wrap existing debug infrastructure (like ExecutionManager)
/// or provide native protocol handling (like future kernel implementations).
#[async_trait]
pub trait DebugCapability: Send + Sync {
    /// Process a debug request and return a response
    async fn process_debug_request(&self, request: DebugRequest) -> Result<DebugResponse>;

    /// Get the list of capabilities this implementation provides
    fn capabilities(&self) -> Vec<String>;

    /// Get the name of this capability provider
    fn name(&self) -> &str;
}

/// Protocol-agnostic debug request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugRequest {
    /// Create a new debug session
    CreateSession { script: String, args: Vec<String> },

    /// Set breakpoints in a source file
    SetBreakpoints {
        source: String,
        breakpoints: Vec<(u32, Option<String>)>, // (line, condition)
    },

    /// Remove breakpoints
    RemoveBreakpoints { ids: Vec<String> },

    /// Step execution (in, over, out)
    Step { step_type: StepType },

    /// Continue execution
    Continue,

    /// Pause execution
    Pause,

    /// Inspect variables
    InspectVariables {
        names: Vec<String>,
        frame_id: Option<usize>,
    },

    /// Navigate to stack frame
    NavigateStack { frame_index: usize },

    /// Get current stack trace
    GetStackTrace,

    /// Evaluate expression
    EvaluateExpression {
        expression: String,
        frame_id: Option<usize>,
    },

    /// Get or set debug state
    GetDebugState,

    /// Terminate debug session
    Terminate { session_id: Option<String> },
}

impl DebugRequest {
    /// Get the capability name that should handle this request
    pub fn capability_name(&self) -> String {
        match self {
            DebugRequest::CreateSession { .. }
            | DebugRequest::SetBreakpoints { .. }
            | DebugRequest::RemoveBreakpoints { .. }
            | DebugRequest::Step { .. }
            | DebugRequest::Continue
            | DebugRequest::Pause
            | DebugRequest::GetDebugState
            | DebugRequest::Terminate { .. } => "execution_manager".to_string(),

            DebugRequest::InspectVariables { .. } | DebugRequest::EvaluateExpression { .. } => {
                "variable_inspector".to_string()
            }

            DebugRequest::NavigateStack { .. } | DebugRequest::GetStackTrace => {
                "stack_navigator".to_string()
            }
        }
    }
}

/// Protocol-agnostic debug response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugResponse {
    /// Session created successfully
    SessionCreated {
        session_id: String,
        capabilities: Vec<String>,
    },

    /// Breakpoints set successfully
    BreakpointsSet { breakpoints: Vec<BreakpointInfo> },

    /// Breakpoints removed
    BreakpointsRemoved { count: usize },

    /// Execution state changed
    ExecutionState(DebugState),

    /// Variables retrieved
    Variables(HashMap<String, serde_json::Value>),

    /// Stack frame information
    StackFrame(StackFrameInfo),

    /// Stack trace retrieved
    StackTrace(Vec<StackFrameInfo>),

    /// Expression evaluation result
    EvaluationResult {
        value: serde_json::Value,
        type_name: Option<String>,
    },

    /// Debug state information
    DebugStateInfo(DebugState),

    /// Session terminated
    SessionTerminated,

    /// Error response
    Error {
        message: String,
        details: Option<String>,
    },
}

/// Step type for step debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepType {
    /// Step into function calls
    StepIn,
    /// Step over function calls
    StepOver,
    /// Step out of current function
    StepOut,
}

/// Debug execution state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugState {
    /// Not debugging
    Idle,
    /// Script is running
    Running,
    /// Paused at breakpoint or step
    Paused {
        reason: PauseReason,
        location: Option<LocationInfo>,
    },
    /// Script terminated
    Terminated { exit_code: Option<i32> },
}

/// Reason for pausing execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PauseReason {
    /// Hit a breakpoint
    Breakpoint { id: String },
    /// Step completed
    Step,
    /// Explicit pause request
    PauseRequest,
    /// Exception occurred
    Exception { message: String },
    /// Entry point
    Entry,
}

/// Location information for debugging
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocationInfo {
    /// Source file or script
    pub source: String,
    /// Line number (1-based)
    pub line: u32,
    /// Column number (1-based, optional)
    pub column: Option<u32>,
    /// Function name (optional)
    pub function: Option<String>,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    /// Unique breakpoint ID
    pub id: String,
    /// Source file
    pub source: String,
    /// Line number
    pub line: u32,
    /// Condition (optional)
    pub condition: Option<String>,
    /// Whether breakpoint is verified
    pub verified: bool,
    /// Hit count
    pub hit_count: u32,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrameInfo {
    /// Frame index (0 = top)
    pub index: usize,
    /// Function name
    pub name: String,
    /// Source location
    pub location: LocationInfo,
    /// Local variables (names only, use InspectVariables for values)
    pub locals: Vec<String>,
    /// Whether this is user code
    pub is_user_code: bool,
}

/// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Variable value (as JSON)
    pub value: serde_json::Value,
    /// Type name (optional)
    pub type_name: Option<String>,
    /// Whether the variable is expandable (has children)
    pub expandable: bool,
    /// Variable reference for lazy expansion
    pub reference: Option<usize>,
}
