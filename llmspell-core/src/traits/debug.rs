//! Multi-language debug architecture traits for Phase 15/18
//!
//! Provides traits and types for implementing language-agnostic debugging support
//! with breakpoints, stepping, variable inspection, and expression evaluation.

use crate::error::LLMSpellError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug as FmtDebug;

/// Debug session information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    /// Session identifier
    pub id: String,
    /// Language being debugged
    pub language: String,
    /// Active breakpoints
    pub breakpoints: Vec<Breakpoint>,
    /// Current execution state
    pub state: DebugState,
    /// Variables in current scope
    pub variables: HashMap<String, serde_json::Value>,
    /// Call stack frames
    pub call_stack: Vec<StackFrame>,
    /// Session start time
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Unique breakpoint ID
    pub id: String,
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Optional condition
    pub condition: Option<String>,
    /// Hit count
    pub hit_count: usize,
    /// Whether breakpoint is enabled
    pub enabled: bool,
    /// Log message to output when hit
    pub log_message: Option<String>,
}

/// Debug execution state
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DebugState {
    /// Not running
    Idle,
    /// Running normally
    Running,
    /// Paused at breakpoint
    Paused,
    /// Stepping through code
    Stepping,
    /// Terminated
    Terminated,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame ID
    pub id: String,
    /// Function name
    pub function_name: String,
    /// Source file
    pub file: String,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: Option<usize>,
    /// Local variables
    pub locals: HashMap<String, serde_json::Value>,
}

/// Variable watch expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    /// Watch ID
    pub id: String,
    /// Expression to evaluate
    pub expression: String,
    /// Current value
    pub value: Option<serde_json::Value>,
    /// Whether watch is enabled
    pub enabled: bool,
}

/// Multi-language debug architecture trait
///
/// This trait defines the interface for implementing debugging support across
/// multiple script languages (Lua, JavaScript, etc.). Implementations should
/// integrate with DAP (Debug Adapter Protocol) for IDE compatibility.
#[async_trait]
pub trait MultiLanguageDebug: Send + Sync + FmtDebug {
    /// Start a debug session
    async fn start_debug_session(
        &self,
        language: &str,
        script_path: &str,
    ) -> Result<DebugSession, LLMSpellError>;

    /// Set a breakpoint
    async fn set_breakpoint(
        &self,
        session_id: &str,
        file: &str,
        line: usize,
        condition: Option<String>,
    ) -> Result<String, LLMSpellError>; // Returns breakpoint ID

    /// Remove a breakpoint
    async fn remove_breakpoint(
        &self,
        session_id: &str,
        breakpoint_id: &str,
    ) -> Result<(), LLMSpellError>;

    /// Continue execution
    async fn continue_execution(&self, session_id: &str) -> Result<DebugState, LLMSpellError>;

    /// Step over (next line in current scope)
    async fn step_over(&self, session_id: &str) -> Result<DebugState, LLMSpellError>;

    /// Step into (enter function calls)
    async fn step_into(&self, session_id: &str) -> Result<DebugState, LLMSpellError>;

    /// Step out (exit current function)
    async fn step_out(&self, session_id: &str) -> Result<DebugState, LLMSpellError>;

    /// Evaluate expression in current context
    async fn evaluate(
        &self,
        session_id: &str,
        expression: &str,
    ) -> Result<serde_json::Value, LLMSpellError>;

    /// Get current variables in scope
    async fn get_variables(
        &self,
        session_id: &str,
        frame_id: Option<&str>,
    ) -> Result<HashMap<String, serde_json::Value>, LLMSpellError>;

    /// Get call stack
    async fn get_call_stack(&self, session_id: &str) -> Result<Vec<StackFrame>, LLMSpellError>;

    /// Add a watch expression
    async fn add_watch(&self, session_id: &str, expression: &str) -> Result<String, LLMSpellError>; // Returns watch ID

    /// Remove a watch expression
    async fn remove_watch(&self, session_id: &str, watch_id: &str) -> Result<(), LLMSpellError>;

    /// Get all watches
    async fn get_watches(&self, session_id: &str) -> Result<Vec<WatchExpression>, LLMSpellError>;

    /// Terminate debug session
    async fn terminate_session(&self, session_id: &str) -> Result<(), LLMSpellError>;
}
