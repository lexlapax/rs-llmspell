//! ABOUTME: Hook type definitions for workflow lifecycle events
//! ABOUTME: Prepares data structures for Phase 4 hook implementation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Context passed to hook functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookContext {
    /// Workflow ID
    pub workflow_id: String,

    /// Workflow name
    pub workflow_name: String,

    /// Current hook point
    pub hook_point: String,

    /// Step information (if applicable)
    pub step: Option<StepContext>,

    /// Current workflow state
    pub state: HashMap<String, serde_json::Value>,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Step-specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepContext {
    /// Step name
    pub name: String,

    /// Step index (0-based)
    pub index: usize,

    /// Step type (tool, agent, workflow)
    pub step_type: String,

    /// Step input
    pub input: Option<serde_json::Value>,

    /// Step output (for after_step hooks)
    pub output: Option<serde_json::Value>,

    /// Step duration (for after_step hooks)
    pub duration_ms: Option<u64>,
}

/// Result returned by hook functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookResult {
    /// Whether to continue workflow execution
    pub continue_execution: bool,

    /// Optional modifications to workflow state
    pub state_updates: Option<HashMap<String, serde_json::Value>>,

    /// Optional message to log
    pub message: Option<String>,
}

impl Default for HookResult {
    fn default() -> Self {
        Self {
            continue_execution: true,
            state_updates: None,
            message: None,
        }
    }
}

/// Hook execution error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookError {
    /// Error message
    pub message: String,

    /// Hook point where error occurred
    pub hook_point: String,

    /// Whether the error is recoverable
    pub recoverable: bool,
}

impl std::fmt::Display for HookError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Hook error at {}: {}", self.hook_point, self.message)
    }
}

impl std::error::Error for HookError {}
