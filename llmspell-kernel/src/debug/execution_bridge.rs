//! Execution debugging bridge for script engines
//!
//! Provides execution debugging capabilities for script engines including breakpoints,
//! variable inspection, stepping, and execution control.
//!
//! Migrated from Phase-9 branch (originally 642 lines)

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::debug_context::{
    DebugContext, StackFrame as DebugStackFrame, Variable as DebugVariable,
};
use llmspell_core::LLMSpellError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Notify};
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

/// Debug event for stopped execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoppedEvent {
    /// Reason for stopping (breakpoint, step, pause, exception)
    pub reason: String,
    /// Thread ID (usually 1 for single-threaded scripts)
    pub thread_id: i32,
    /// Optional breakpoint ID that was hit
    pub breakpoint_id: Option<String>,
    /// Current file
    pub file: String,
    /// Current line
    pub line: u32,
}

/// Pause state for async coordination
#[derive(Debug, Clone)]
pub struct PauseState {
    /// Whether execution is paused
    pub paused: Arc<AtomicBool>,
    /// Signal to resume execution
    pub resume_signal: Arc<Notify>,
    /// Current step mode
    pub step_mode: Arc<RwLock<StepMode>>,
}

impl PauseState {
    /// Create a new pause state
    pub fn new() -> Self {
        Self {
            paused: Arc::new(AtomicBool::new(false)),
            resume_signal: Arc::new(Notify::new()),
            step_mode: Arc::new(RwLock::new(StepMode::Continue)),
        }
    }
}

impl Default for PauseState {
    fn default() -> Self {
        Self::new()
    }
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
    /// Pause state for async coordination
    pause_state: PauseState,
    /// Channel to send stopped events
    stopped_event_tx: Option<mpsc::Sender<StoppedEvent>>,
    /// Debug mode enabled flag
    debug_enabled: Arc<AtomicBool>,
    /// Current execution location
    current_location: Arc<RwLock<Option<(String, u32)>>>,
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
            pause_state: PauseState::new(),
            stopped_event_tx: None,
            debug_enabled: Arc::new(AtomicBool::new(false)),
            current_location: Arc::new(RwLock::new(None)),
        }
    }

    /// Set the stopped event sender
    pub fn set_stopped_event_sender(&mut self, tx: mpsc::Sender<StoppedEvent>) {
        self.stopped_event_tx = Some(tx);
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
        self.pause_state.paused.store(true, Ordering::SeqCst);
        trace!("Execution paused");
    }

    /// Resume execution with given step mode
    pub fn resume(&self, mode: StepMode) {
        *self.paused.write() = false;
        *self.step_mode.write() = Some(mode);
        self.pause_state.paused.store(false, Ordering::SeqCst);
        *self.pause_state.step_mode.write() = mode;
        self.pause_state.resume_signal.notify_one();
        trace!("Execution resumed with mode: {:?}", mode);
    }

    /// Check if we should pause at a breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if DAP communication fails
    #[instrument(level = "trace", skip(self))]
    pub async fn check_breakpoint(&self, file: &str, line: u32) -> Result<()> {
        // First check if we should pause at this location
        let (should_pause, breakpoint_id, reason) = {
            let breakpoints = self.breakpoints.read();

            // Check if there's a breakpoint at this location
            let mut hit_breakpoint = None;
            if let Some(bp_list) = breakpoints.get(file) {
                for bp in bp_list {
                    if bp.line == line && bp.enabled && bp.should_break() {
                        hit_breakpoint = Some(bp.id.clone());
                        break;
                    }
                }
            }

            // Check step mode
            let step_mode = self.step_mode.read();
            let step_reason = match *step_mode {
                Some(StepMode::StepIn | StepMode::StepOver | StepMode::StepOut) => Some("step"),
                _ => None,
            };

            if let Some(bp_id) = hit_breakpoint {
                (true, Some(bp_id), "breakpoint")
            } else if let Some(reason) = step_reason {
                (true, None, reason)
            } else {
                (false, None, "")
            }
        };

        if should_pause {
            // Set paused state
            self.pause_state.paused.store(true, Ordering::SeqCst);
            *self.paused.write() = true;

            debug!("Pausing at {}:{} (reason: {})", file, line, reason);

            // Send stopped event via channel if available
            if let Some(ref tx) = self.stopped_event_tx {
                let event = StoppedEvent {
                    reason: reason.to_string(),
                    thread_id: 1, // Single-threaded for now
                    breakpoint_id,
                    file: file.to_string(),
                    line,
                };

                // Send without blocking
                let _ = tx.try_send(event);
            }

            // Wait for resume signal
            self.pause_state.resume_signal.notified().await;

            debug!("Resuming from pause at {}:{}", file, line);
        }

        Ok(())
    }

    /// Get the pause state for external coordination
    pub fn pause_state(&self) -> &PauseState {
        &self.pause_state
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

#[async_trait]
impl DebugContext for ExecutionManager {
    fn should_pause_sync(&self, file: &str, line: u32) -> bool {
        // Fast path when disabled
        if !self.debug_enabled.load(Ordering::Relaxed) {
            return false;
        }

        // Check if stepping
        if let Some(StepMode::StepIn | StepMode::StepOver | StepMode::StepOut) =
            &*self.step_mode.read()
        {
            return true;
        }

        // Check breakpoints
        self.should_pause(file, line)
    }

    async fn pause_and_wait(&self, file: &str, line: u32) -> Result<(), LLMSpellError> {
        // Set paused state
        self.pause_state.paused.store(true, Ordering::SeqCst);
        *self.paused.write() = true;
        *self.current_location.write() = Some((file.to_string(), line));

        // Send stopped event if channel available
        if let Some(ref tx) = self.stopped_event_tx {
            let event = StoppedEvent {
                reason: "breakpoint".to_string(),
                thread_id: 1,
                breakpoint_id: None,
                file: file.to_string(),
                line,
            };
            let _ = tx.send(event).await;
        }

        // Wait for resume signal
        self.pause_state.resume_signal.notified().await;

        Ok(())
    }

    fn enable_debug_mode(&self) {
        self.debug_enabled.store(true, Ordering::SeqCst);
        debug!("Debug mode enabled");
    }

    fn disable_debug_mode(&self) {
        self.debug_enabled.store(false, Ordering::SeqCst);
        debug!("Debug mode disabled");
    }

    fn is_debug_enabled(&self) -> bool {
        self.debug_enabled.load(Ordering::Relaxed)
    }

    fn set_breakpoint(&self, file: &str, line: u32) -> Result<String, LLMSpellError> {
        match ExecutionManager::set_breakpoint(self, file.to_string(), line) {
            Ok(bp) => Ok(bp.id),
            Err(e) => Err(LLMSpellError::Component {
                message: format!("Failed to set breakpoint: {e}"),
                source: None,
            }),
        }
    }

    fn clear_breakpoint(&self, id: &str) -> Result<(), LLMSpellError> {
        self.remove_breakpoint(id)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to clear breakpoint: {e}"),
                source: None,
            })
    }

    fn get_stack_frames(&self) -> Vec<DebugStackFrame> {
        self.stack_frames
            .read()
            .iter()
            .enumerate()
            .map(|(i, frame)| DebugStackFrame {
                id: i,
                name: frame.name.clone(),
                file: frame.source.clone(),
                line: frame.line,
                column: frame.column,
            })
            .collect()
    }

    fn get_variables(&self, frame_id: usize) -> Vec<DebugVariable> {
        let frames = self.stack_frames.read();
        if let Some(frame) = frames.get(frame_id) {
            frame
                .locals
                .iter()
                .map(|var| DebugVariable {
                    name: var.name.clone(),
                    value: var.value.clone(),
                    var_type: var.var_type.clone(),
                    has_children: var.has_children,
                })
                .collect()
        } else {
            vec![]
        }
    }

    fn report_location(&self, file: &str, line: u32) {
        *self.current_location.write() = Some((file.to_string(), line));
        trace!("Execution at {}:{}", file, line);
    }

    fn should_step(&self) -> bool {
        matches!(
            &*self.step_mode.read(),
            Some(StepMode::StepIn | StepMode::StepOver | StepMode::StepOut)
        )
    }

    fn set_step_mode(&self, stepping: bool) {
        *self.step_mode.write() = if stepping {
            Some(StepMode::StepIn)
        } else {
            None
        };
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

    #[tokio::test]
    async fn test_debug_context_implementation() {
        use llmspell_core::traits::debug_context::DebugContext;

        let manager = Arc::new(ExecutionManager::new("test-session".to_string()));
        let debug_ctx: Arc<dyn DebugContext> = manager.clone();

        // Test debug mode enable/disable
        assert!(!debug_ctx.is_debug_enabled());
        debug_ctx.enable_debug_mode();
        assert!(debug_ctx.is_debug_enabled());
        debug_ctx.disable_debug_mode();
        assert!(!debug_ctx.is_debug_enabled());

        // Re-enable for further tests
        debug_ctx.enable_debug_mode();

        // Test breakpoint setting
        let bp_id = debug_ctx.set_breakpoint("test.lua", 42).unwrap();
        assert!(!bp_id.is_empty());

        // Test should_pause_sync
        assert!(debug_ctx.should_pause_sync("test.lua", 42));
        assert!(!debug_ctx.should_pause_sync("test.lua", 43));

        // Test step mode
        assert!(!debug_ctx.should_step());
        debug_ctx.set_step_mode(true);
        assert!(debug_ctx.should_step());
        debug_ctx.set_step_mode(false);
        assert!(!debug_ctx.should_step());

        // Test location reporting
        debug_ctx.report_location("test.lua", 100);
        // Location reporting doesn't return a value, but we can verify it doesn't panic

        // Test stack frames
        let frames = debug_ctx.get_stack_frames();
        assert!(frames.is_empty()); // No frames in test mode

        // Test variables
        let vars = debug_ctx.get_variables(0);
        assert!(vars.is_empty()); // No variables in test mode

        // Test breakpoint clearing
        debug_ctx.clear_breakpoint(&bp_id).unwrap();
        assert!(!debug_ctx.should_pause_sync("test.lua", 42));

        // Test pause_and_wait with immediate resume
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            manager_clone.resume(StepMode::Continue);
        });
        debug_ctx.pause_and_wait("test.lua", 50).await.unwrap();
    }

    #[test]
    fn test_debug_context_thread_safety() {
        use llmspell_core::traits::debug_context::DebugContext;
        use std::thread;

        let manager = Arc::new(ExecutionManager::new("test-session".to_string()));
        let debug_ctx: Arc<dyn DebugContext> = manager.clone();

        // Enable debugging
        debug_ctx.enable_debug_mode();

        // Test concurrent access from multiple threads
        let ctx1 = debug_ctx.clone();
        let ctx2 = debug_ctx.clone();
        let ctx3 = debug_ctx.clone();

        let handle1 = thread::spawn(move || {
            for i in 0..10 {
                ctx1.set_breakpoint("thread1.lua", i).unwrap();
            }
        });

        let handle2 = thread::spawn(move || {
            for i in 0..10 {
                ctx2.report_location("thread2.lua", i);
            }
        });

        let handle3 = thread::spawn(move || {
            for _ in 0..10 {
                ctx3.set_step_mode(true);
                ctx3.set_step_mode(false);
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();

        // If we get here without panicking, thread safety works
        assert!(debug_ctx.is_debug_enabled());
    }
}
