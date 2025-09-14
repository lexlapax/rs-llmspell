//! Execution debugging bridge for script engines
//!
//! Provides execution debugging capabilities for script engines including breakpoints,
//! variable inspection, stepping, and execution control.
//!
//! Migrated from Phase-9 branch (originally 642 lines)

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, instrument, trace};

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

    /// Check if breakpoint should trigger
    #[must_use]
    pub fn should_break(&self) -> bool {
        if !self.enabled {
            return false;
        }
        self.hit_count
            .is_none_or(|hit_count| self.current_hits >= hit_count)
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
}

/// Variable scope for inspection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableScope {
    /// Local variables in current frame
    Local,
    /// Global variables
    Global,
    /// Upvalues/closures
    Upvalue,
    /// Watch expressions
    Watch,
}

/// Step mode for debugging
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepMode {
    /// Step into functions
    StepIn,
    /// Step over functions
    StepOver,
    /// Step out of current function
    StepOut,
    /// Continue execution
    Continue,
    /// Pause execution
    Pause,
}

/// Manages execution state and debugging operations
pub struct ExecutionManager {
    /// Active breakpoints
    breakpoints: Arc<RwLock<HashMap<String, Vec<Breakpoint>>>>,
    /// Current execution state
    paused: Arc<RwLock<bool>>,
    /// Current step mode
    step_mode: Arc<RwLock<Option<StepMode>>>,
    /// Stack frames
    stack_frames: Arc<RwLock<Vec<StackFrame>>>,
    /// Variable references
    _variable_refs: Arc<RwLock<HashMap<String, Vec<Variable>>>>,
    /// Session ID
    _session_id: String,
}

impl ExecutionManager {
    /// Create a new execution manager
    pub fn new(session_id: String) -> Self {
        Self {
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            paused: Arc::new(RwLock::new(false)),
            step_mode: Arc::new(RwLock::new(None)),
            stack_frames: Arc::new(RwLock::new(Vec::new())),
            _variable_refs: Arc::new(RwLock::new(HashMap::new())),
            _session_id: session_id,
        }
    }

    /// Set a breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint already exists
    #[instrument(level = "debug", skip(self))]
    pub fn set_breakpoint(&self, source: String, line: u32) -> Result<Breakpoint> {
        let breakpoint = Breakpoint::new(source.clone(), line);

        let mut breakpoints = self.breakpoints.write();
        breakpoints
            .entry(source)
            .or_default()
            .push(breakpoint.clone());

        debug!(
            "Set breakpoint at {}:{}",
            breakpoint.source, breakpoint.line
        );
        Ok(breakpoint)
    }

    /// Remove a breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint doesn't exist
    pub fn remove_breakpoint(&self, id: &str) -> Result<()> {
        let mut breakpoints = self.breakpoints.write();

        for bp_list in breakpoints.values_mut() {
            bp_list.retain(|bp| bp.id != id);
        }

        Ok(())
    }

    /// Check if execution should pause at current location
    pub fn should_pause(&self, source: &str, line: u32) -> bool {
        // Check if already paused
        if *self.paused.read() {
            return false;
        }

        // Check step mode
        if let Some(StepMode::StepIn | StepMode::StepOver) = *self.step_mode.read() {
            return true;
        }

        // Check breakpoints
        if let Some(bp_list) = self.breakpoints.read().get(source) {
            for bp in bp_list {
                if bp.line == line && bp.should_break() {
                    return true;
                }
            }
        }

        false
    }

    /// Pause execution
    pub fn pause(&self) {
        *self.paused.write() = true;
        trace!("Execution paused");
    }

    /// Resume execution with given step mode
    pub fn resume(&self, mode: StepMode) {
        *self.paused.write() = false;
        *self.step_mode.write() = Some(mode);
        trace!("Execution resumed with mode: {:?}", mode);
    }

    /// Get current stack frames
    pub fn get_stack_frames(&self) -> Vec<StackFrame> {
        self.stack_frames.read().clone()
    }

    /// Update stack frames
    pub fn update_stack_frames(&self, frames: Vec<StackFrame>) {
        *self.stack_frames.write() = frames;
    }

    /// Get variables for a scope
    pub fn get_variables(&self, scope: &VariableScope, frame_id: Option<&str>) -> Vec<Variable> {
        match scope {
            VariableScope::Local => {
                if let Some(frame_id) = frame_id {
                    self.stack_frames
                        .read()
                        .iter()
                        .find(|f| f.id == frame_id)
                        .map(|f| f.locals.clone())
                        .unwrap_or_default()
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(), // Simplified for now
        }
    }

    /// Check if currently paused
    pub fn is_paused(&self) -> bool {
        *self.paused.read()
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
    fn test_execution_manager() {
        let manager = ExecutionManager::new("test-session".to_string());

        // Set breakpoint
        let bp = manager.set_breakpoint("test.lua".to_string(), 10).unwrap();
        assert!(!bp.id.is_empty());

        // Check should pause
        assert!(manager.should_pause("test.lua", 10));
        assert!(!manager.should_pause("test.lua", 11));

        // Remove breakpoint
        manager.remove_breakpoint(&bp.id).unwrap();
        assert!(!manager.should_pause("test.lua", 10));
    }

    #[test]
    fn test_pause_resume() {
        let manager = ExecutionManager::new("test-session".to_string());

        assert!(!manager.is_paused());

        manager.pause();
        assert!(manager.is_paused());

        manager.resume(StepMode::Continue);
        assert!(!manager.is_paused());
    }
}
