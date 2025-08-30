//! Execution debugging bridge for script engines
//!
//! Provides execution debugging capabilities for script engines including breakpoints,
//! variable inspection, stepping, and execution control. This is distinct from
//! diagnostics (logging/profiling) which is handled by `diagnostics_bridge`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Unique breakpoint ID
    pub id: String,
    /// File or script identifier
    pub source: String,
    /// Line number (1-based)
    pub line: u32,
    /// Optional condition expression
    pub condition: Option<String>,
    /// Hit count before breaking
    pub hit_count: Option<u32>,
    /// Current hit count
    pub current_hits: u32,
    /// Whether breakpoint is enabled
    pub enabled: bool,
}

impl Breakpoint {
    /// Create a new breakpoint
    #[must_use]
    pub fn new(source: String, line: u32) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source,
            line,
            condition: None,
            hit_count: None,
            current_hits: 0,
            enabled: true,
        }
    }

    /// Add a condition to the breakpoint (for interactive debugging)
    #[must_use]
    pub fn with_condition(mut self, condition: String) -> Self {
        self.condition = Some(condition);
        self
    }

    /// Add a hit count to the breakpoint (for interactive debugging)
    #[must_use]
    pub const fn with_hit_count(mut self, count: u32) -> Self {
        self.hit_count = Some(count);
        self
    }

    /// Check if breakpoint should trigger
    #[must_use]
    pub fn should_break(&self) -> bool {
        if !self.enabled {
            return false;
        }

        #[allow(clippy::unnecessary_map_or)]
        self.hit_count
            .map_or(true, |hit_count| self.current_hits >= hit_count)
    }
}

/// Variable information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Variable value (as string representation)
    pub value: String,
    /// Variable type
    pub var_type: String,
    /// Whether this variable has children (for complex types)
    pub has_children: bool,
    /// Reference ID for fetching children
    pub reference: Option<String>,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame ID
    pub id: String,
    /// Function/scope name
    pub name: String,
    /// Source file or script
    pub source: String,
    /// Current line number
    pub line: u32,
    /// Current column (optional)
    pub column: Option<u32>,
    /// Local variables in this frame
    pub locals: Vec<Variable>,
    /// Whether this is user code (vs library code)
    pub is_user_code: bool,
}

/// Debug execution state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DebugState {
    /// Script is running normally
    Running,
    /// Script is paused at a breakpoint
    Paused {
        /// Reason for pause
        reason: PauseReason,
        /// Current location
        location: ExecutionLocation,
    },
    /// Script has terminated
    Terminated,
}

/// Reason for pausing execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PauseReason {
    /// Hit a breakpoint
    Breakpoint,
    /// Step operation completed
    Step,
    /// Explicit pause request
    Pause,
    /// Exception occurred
    Exception(String),
    /// Entry point of script
    Entry,
}

/// Execution location information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionLocation {
    /// Source file or script
    pub source: String,
    /// Line number
    pub line: u32,
    /// Column number (optional)
    pub column: Option<u32>,
}

/// Debug control commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugCommand {
    /// Continue execution
    Continue,
    /// Step into next statement
    StepInto,
    /// Step over next statement
    StepOver,
    /// Step out of current function
    StepOut,
    /// Pause execution
    Pause,
    /// Terminate execution
    Terminate,
}

/// Debugger interface for script engines
#[async_trait::async_trait]
pub trait ScriptDebugger: Send + Sync {
    /// Set breakpoints
    async fn set_breakpoints(&self, breakpoints: Vec<Breakpoint>) -> Vec<Breakpoint>;

    /// Remove a breakpoint
    async fn remove_breakpoint(&self, breakpoint_id: &str) -> bool;

    /// Get current debug state
    async fn get_state(&self) -> DebugState;

    /// Get stack trace
    async fn get_stack_trace(&self) -> Vec<StackFrame>;

    /// Get variables in a scope
    async fn get_variables(&self, frame_id: Option<&str>) -> Vec<Variable>;

    /// Evaluate an expression in current context
    async fn evaluate(&self, expression: &str, frame_id: Option<&str>) -> Variable;

    /// Send a debug command
    async fn send_command(&self, command: DebugCommand);

    /// Check if debugging is active
    async fn is_active(&self) -> bool;
}

/// Execution manager that coordinates debugging across engines
pub struct ExecutionManager {
    /// Active breakpoints
    breakpoints: Arc<RwLock<HashMap<String, Breakpoint>>>,
    /// Current debug state
    state: Arc<RwLock<DebugState>>,
    /// Stack frames
    stack_frames: Arc<RwLock<Vec<StackFrame>>>,
    /// Variables cache
    variables: Arc<RwLock<HashMap<String, Vec<Variable>>>>,
}

impl ExecutionManager {
    /// Create a new debug manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(DebugState::Terminated)),
            stack_frames: Arc::new(RwLock::new(Vec::new())),
            variables: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add a breakpoint
    pub async fn add_breakpoint(&self, breakpoint: Breakpoint) -> String {
        let id = breakpoint.id.clone();
        self.breakpoints
            .write()
            .await
            .insert(id.clone(), breakpoint);
        id
    }

    /// Remove a breakpoint
    pub async fn remove_breakpoint(&self, id: &str) -> bool {
        self.breakpoints.write().await.remove(id).is_some()
    }

    /// Get all breakpoints
    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        self.breakpoints.read().await.values().cloned().collect()
    }

    /// Update debug state
    pub async fn set_state(&self, state: DebugState) {
        *self.state.write().await = state;
    }

    /// Get current debug state
    pub async fn get_state(&self) -> DebugState {
        self.state.read().await.clone()
    }

    /// Update stack trace
    pub async fn set_stack_trace(&self, frames: Vec<StackFrame>) {
        *self.stack_frames.write().await = frames;
    }

    /// Get stack trace
    pub async fn get_stack_trace(&self) -> Vec<StackFrame> {
        self.stack_frames.read().await.clone()
    }

    /// Cache variables for a frame
    pub async fn cache_variables(&self, frame_id: String, variables: Vec<Variable>) {
        self.variables.write().await.insert(frame_id, variables);
    }

    /// Get cached variables
    pub async fn get_cached_variables(&self, frame_id: &str) -> Option<Vec<Variable>> {
        self.variables.read().await.get(frame_id).cloned()
    }

    /// Clear all debug data
    pub async fn clear(&self) {
        self.breakpoints.write().await.clear();
        *self.state.write().await = DebugState::Terminated;
        self.stack_frames.write().await.clear();
        self.variables.write().await.clear();
    }

    /// Send a debug command (interactive debugging support)
    pub async fn send_command(&self, command: DebugCommand) {
        // Update state based on command
        match command {
            DebugCommand::Continue => {
                self.set_state(DebugState::Running).await;
            }
            DebugCommand::Pause => {
                self.set_state(DebugState::Paused {
                    reason: PauseReason::Pause,
                    location: ExecutionLocation {
                        source: "unknown".to_string(),
                        line: 0,
                        column: None,
                    },
                })
                .await;
            }
            DebugCommand::Terminate => {
                self.set_state(DebugState::Terminated).await;
            }
            DebugCommand::StepInto | DebugCommand::StepOver | DebugCommand::StepOut => {
                self.set_state(DebugState::Paused {
                    reason: PauseReason::Step,
                    location: ExecutionLocation {
                        source: "unknown".to_string(),
                        line: 0,
                        column: None,
                    },
                })
                .await;
            }
        }
    }

    /// Get variables (interactive debugging support)
    pub async fn get_variables(&self, frame_id: Option<&str>) -> Vec<Variable> {
        if let Some(frame_id) = frame_id {
            self.get_cached_variables(frame_id)
                .await
                .unwrap_or_default()
        } else {
            // Return variables from the top frame if available
            self.variables
                .read()
                .await
                .values()
                .next()
                .cloned()
                .unwrap_or_default()
        }
    }

    /// Evaluate expression (interactive debugging support)
    #[must_use]
    pub fn evaluate(&self, expression: &str, _frame_id: Option<&str>) -> Variable {
        // Basic implementation - in real scenario this would evaluate in Lua context
        Variable {
            name: expression.to_string(),
            value: "<evaluation not implemented>".to_string(),
            var_type: "unknown".to_string(),
            has_children: false,
            reference: None,
        }
    }

    /// Check if debugging is active
    pub async fn is_active(&self) -> bool {
        !matches!(self.get_state().await, DebugState::Terminated)
    }
}

impl Default for ExecutionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_creation() {
        let bp = Breakpoint::new("test.lua".to_string(), 10);
        assert_eq!(bp.source, "test.lua");
        assert_eq!(bp.line, 10);
        assert!(bp.enabled);
        assert!(bp.should_break());
    }

    #[test]
    fn test_breakpoint_hit_count() {
        let mut bp = Breakpoint::new("test.lua".to_string(), 10);
        bp.hit_count = Some(3);
        bp.current_hits = 2;
        assert!(!bp.should_break());

        bp.current_hits = 3;
        assert!(bp.should_break());
    }

    #[tokio::test]
    async fn test_execution_manager() {
        let manager = ExecutionManager::new();

        // Add breakpoint
        let bp = Breakpoint::new("test.lua".to_string(), 10);
        let id = manager.add_breakpoint(bp).await;

        // Check it was added
        let breakpoints = manager.get_breakpoints().await;
        assert_eq!(breakpoints.len(), 1);

        // Remove breakpoint
        assert!(manager.remove_breakpoint(&id).await);
        let breakpoints = manager.get_breakpoints().await;
        assert_eq!(breakpoints.len(), 0);
    }
}
