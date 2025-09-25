//! Debug context abstraction for script execution
//!
//! Provides a trait that allows script executors to interact with debug infrastructure
//! without direct dependencies on kernel types. This maintains clean architecture boundaries
//! and enables debugging support to be optionally added to any script execution.

use crate::error::LLMSpellError;
use async_trait::async_trait;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Stack frame information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    /// Frame identifier
    pub id: usize,
    /// Function or scope name
    pub name: String,
    /// Source file
    pub file: String,
    /// Line number
    pub line: u32,
    /// Column number (optional)
    pub column: Option<u32>,
}

/// Variable information for debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    /// Variable name
    pub name: String,
    /// Variable value as string
    pub value: String,
    /// Variable type
    pub var_type: String,
    /// Whether this variable has children
    pub has_children: bool,
}

/// Debug context trait for script execution
///
/// This trait provides an abstraction layer between script executors
/// and debug infrastructure. It allows script engines to check for
/// breakpoints and pause execution without knowing about the underlying
/// debug implementation.
#[async_trait]
pub trait DebugContext: Send + Sync {
    /// Check if execution should pause at given location (sync for hooks)
    ///
    /// This method must be very fast as it's called for every line of code.
    /// When debug is disabled, it should return false immediately.
    fn should_pause_sync(&self, file: &str, line: u32) -> bool;

    /// Async pause and wait for resume
    ///
    /// Called when execution should pause. This method blocks until
    /// the debugger signals to continue execution.
    async fn pause_and_wait(&self, file: &str, line: u32) -> Result<(), LLMSpellError>;

    /// Enable debug mode
    fn enable_debug_mode(&self);

    /// Disable debug mode
    fn disable_debug_mode(&self);

    /// Check if debug mode is enabled
    fn is_debug_enabled(&self) -> bool;

    /// Set a breakpoint
    ///
    /// Returns the breakpoint ID
    fn set_breakpoint(&self, file: &str, line: u32) -> Result<String, LLMSpellError>;

    /// Clear a breakpoint
    fn clear_breakpoint(&self, id: &str) -> Result<(), LLMSpellError>;

    /// Get current stack frames (when paused)
    fn get_stack_frames(&self) -> Vec<StackFrame>;

    /// Get variables in scope (when paused)
    fn get_variables(&self, frame_id: usize) -> Vec<Variable>;

    /// Report current execution location (for stepping)
    ///
    /// This allows the debug context to track execution flow
    /// and implement step over/into/out functionality.
    fn report_location(&self, file: &str, line: u32);

    /// Check if should step (for step debugging)
    fn should_step(&self) -> bool;

    /// Set step mode
    fn set_step_mode(&self, stepping: bool);
}

/// Mock implementation for testing
pub struct MockDebugContext {
    enabled: AtomicBool,
    stepping: AtomicBool,
    breakpoints: Arc<RwLock<HashMap<(String, u32), String>>>,
    paused: AtomicBool,
}

impl MockDebugContext {
    /// Create a new mock debug context
    pub fn new() -> Self {
        Self {
            enabled: AtomicBool::new(false),
            stepping: AtomicBool::new(false),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            paused: AtomicBool::new(false),
        }
    }
}

impl Default for MockDebugContext {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DebugContext for MockDebugContext {
    fn should_pause_sync(&self, file: &str, line: u32) -> bool {
        // Fast path when disabled
        if !self.enabled.load(Ordering::Relaxed) {
            return false;
        }

        // Check if stepping
        if self.stepping.load(Ordering::Relaxed) {
            return true;
        }

        // Check breakpoints
        self.breakpoints
            .read()
            .contains_key(&(file.to_string(), line))
    }

    async fn pause_and_wait(&self, _file: &str, _line: u32) -> Result<(), LLMSpellError> {
        self.paused.store(true, Ordering::SeqCst);

        // In a real implementation, this would wait for a signal to continue
        // For the mock, we just simulate a brief pause
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        self.paused.store(false, Ordering::SeqCst);
        Ok(())
    }

    fn enable_debug_mode(&self) {
        self.enabled.store(true, Ordering::SeqCst);
    }

    fn disable_debug_mode(&self) {
        self.enabled.store(false, Ordering::SeqCst);
    }

    fn is_debug_enabled(&self) -> bool {
        self.enabled.load(Ordering::Relaxed)
    }

    fn set_breakpoint(&self, file: &str, line: u32) -> Result<String, LLMSpellError> {
        let id = format!("bp-{}-{}", file, line);
        self.breakpoints
            .write()
            .insert((file.to_string(), line), id.clone());
        Ok(id)
    }

    fn clear_breakpoint(&self, id: &str) -> Result<(), LLMSpellError> {
        let mut breakpoints = self.breakpoints.write();
        breakpoints.retain(|_, v| v != id);
        Ok(())
    }

    fn get_stack_frames(&self) -> Vec<StackFrame> {
        // Mock implementation returns a sample stack
        vec![StackFrame {
            id: 0,
            name: "main".to_string(),
            file: "test.lua".to_string(),
            line: 10,
            column: None,
        }]
    }

    fn get_variables(&self, _frame_id: usize) -> Vec<Variable> {
        // Mock implementation returns sample variables
        vec![Variable {
            name: "test_var".to_string(),
            value: "42".to_string(),
            var_type: "number".to_string(),
            has_children: false,
        }]
    }

    fn report_location(&self, _file: &str, _line: u32) {
        // Mock implementation does nothing
    }

    fn should_step(&self) -> bool {
        self.stepping.load(Ordering::Relaxed)
    }

    fn set_step_mode(&self, stepping: bool) {
        self.stepping.store(stepping, Ordering::SeqCst);
    }
}

/// No-op implementation for when debugging is not needed
pub struct NoOpDebugContext;

#[async_trait]
impl DebugContext for NoOpDebugContext {
    #[inline(always)]
    fn should_pause_sync(&self, _file: &str, _line: u32) -> bool {
        false
    }

    async fn pause_and_wait(&self, _file: &str, _line: u32) -> Result<(), LLMSpellError> {
        Ok(())
    }

    fn enable_debug_mode(&self) {}
    fn disable_debug_mode(&self) {}

    #[inline(always)]
    fn is_debug_enabled(&self) -> bool {
        false
    }

    fn set_breakpoint(&self, _file: &str, _line: u32) -> Result<String, LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Debugging not enabled".to_string(),
            source: None,
        })
    }

    fn clear_breakpoint(&self, _id: &str) -> Result<(), LLMSpellError> {
        Err(LLMSpellError::Component {
            message: "Debugging not enabled".to_string(),
            source: None,
        })
    }

    fn get_stack_frames(&self) -> Vec<StackFrame> {
        vec![]
    }

    fn get_variables(&self, _frame_id: usize) -> Vec<Variable> {
        vec![]
    }

    fn report_location(&self, _file: &str, _line: u32) {}

    #[inline(always)]
    fn should_step(&self) -> bool {
        false
    }

    fn set_step_mode(&self, _stepping: bool) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_debug_context() {
        let ctx = MockDebugContext::new();

        // Initially disabled
        assert!(!ctx.is_debug_enabled());
        assert!(!ctx.should_pause_sync("test.lua", 10));

        // Enable debug mode
        ctx.enable_debug_mode();
        assert!(ctx.is_debug_enabled());

        // Set breakpoint
        let bp_id = ctx.set_breakpoint("test.lua", 10).unwrap();
        assert!(ctx.should_pause_sync("test.lua", 10));
        assert!(!ctx.should_pause_sync("test.lua", 11));

        // Clear breakpoint
        ctx.clear_breakpoint(&bp_id).unwrap();
        assert!(!ctx.should_pause_sync("test.lua", 10));

        // Test stepping
        ctx.set_step_mode(true);
        assert!(ctx.should_step());
        assert!(ctx.should_pause_sync("any.lua", 99));

        // Disable debug mode
        ctx.disable_debug_mode();
        assert!(!ctx.should_pause_sync("test.lua", 10));
    }

    #[test]
    fn test_noop_debug_context() {
        let ctx = NoOpDebugContext;

        // Always disabled
        assert!(!ctx.is_debug_enabled());
        assert!(!ctx.should_pause_sync("test.lua", 10));
        assert!(!ctx.should_step());

        // Operations return errors or empty
        assert!(ctx.set_breakpoint("test.lua", 10).is_err());
        assert!(ctx.clear_breakpoint("bp-1").is_err());
        assert!(ctx.get_stack_frames().is_empty());
        assert!(ctx.get_variables(0).is_empty());
    }

    #[test]
    fn test_trait_object_safety() {
        // Verify we can create trait objects
        let _mock: Box<dyn DebugContext> = Box::new(MockDebugContext::new());
        let _noop: Box<dyn DebugContext> = Box::new(NoOpDebugContext);

        // Verify Arc works
        let _arc: Arc<dyn DebugContext> = Arc::new(MockDebugContext::new());
    }
}
