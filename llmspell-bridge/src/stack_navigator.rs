//! Script-agnostic stack navigation for debugging
//!
//! This module provides trait definitions for stack navigation that work
//! across any supported script language. Implementations are provided in the
//! respective language modules (lua/, js/, python/, etc.).
//!
//! Follows the three-layer architecture:
//! - Bridge Layer: `StackNavigator` trait (this file, script-agnostic)
//! - Shared Layer: `SharedStackNavigator` (common implementation)
//! - Script Layer: Language-specific implementations in respective modules

use crate::execution_bridge::StackFrame;
use crate::execution_context::SharedExecutionContext;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

/// Script-agnostic trait for stack navigation
///
/// This trait defines the interface for navigating and inspecting
/// call stacks during debugging. Implementations handle language-specific
/// formatting and variable extraction.
pub trait StackNavigator: Send + Sync {
    /// Navigate to a specific frame in the stack
    ///
    /// # Arguments
    /// * `frame_index` - The index of the frame to navigate to
    /// * `stack` - The current stack frames
    ///
    /// # Returns
    /// The selected stack frame or an error if the index is invalid
    ///
    /// # Errors
    /// Returns an error if the frame index is out of bounds
    fn navigate_to_frame(
        &self,
        frame_index: usize,
        stack: &[StackFrame],
    ) -> Result<StackFrame, Box<dyn Error>>;

    /// Format a stack frame for display
    ///
    /// # Arguments
    /// * `frame` - The frame to format
    ///
    /// # Returns
    /// A formatted string representation of the frame
    fn format_frame(&self, frame: &StackFrame) -> String;

    /// Get variables available in a specific frame
    ///
    /// # Arguments
    /// * `frame` - The frame to inspect
    /// * `context` - The shared execution context containing variables
    ///
    /// # Returns
    /// A map of variable names to their values
    fn get_frame_variables(
        &self,
        frame: &StackFrame,
        context: &SharedExecutionContext,
    ) -> HashMap<String, JsonValue>;

    /// Format the entire stack trace
    ///
    /// # Arguments
    /// * `stack` - The stack frames to format
    /// * `current_frame` - The index of the current frame
    ///
    /// # Returns
    /// A formatted string representation of the entire stack
    fn format_stack_trace(&self, stack: &[StackFrame], current_frame: usize) -> String;
}

/// Shared implementation of stack navigation
///
/// This struct provides common stack navigation functionality
/// that can be used by all script-specific implementations.
pub struct SharedStackNavigator {
    /// Current frame index (cached)
    current_frame: std::sync::atomic::AtomicUsize,
}

impl SharedStackNavigator {
    /// Create a new shared stack navigator
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_frame: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Get the current frame index
    #[must_use]
    pub fn get_current_frame(&self) -> usize {
        self.current_frame
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Set the current frame index
    pub fn set_current_frame(&self, index: usize) {
        self.current_frame
            .store(index, std::sync::atomic::Ordering::Relaxed);
    }

    /// Navigate to a frame (common implementation)
    ///
    /// # Errors
    /// Returns an error if the frame index is out of bounds
    pub fn navigate_to_frame_common(
        &self,
        frame_index: usize,
        stack: &[StackFrame],
    ) -> Result<StackFrame, Box<dyn Error>> {
        if frame_index >= stack.len() {
            return Err(format!(
                "Invalid frame index: {} (stack has {} frames)",
                frame_index,
                stack.len()
            )
            .into());
        }

        // Update current frame
        self.set_current_frame(frame_index);

        // Return the selected frame
        Ok(stack[frame_index].clone())
    }

    /// Format a basic stack frame (common implementation)
    pub fn format_frame_common(&self, frame: &StackFrame) -> String {
        frame.column.map_or_else(
            || format!("{}:{} in {}", frame.source, frame.line, frame.name),
            |column| {
                format!(
                    "{}:{}:{} in {}",
                    frame.source, frame.line, column, frame.name
                )
            },
        )
    }

    /// Get frame variables from context (common implementation)
    pub fn get_frame_variables_common(
        &self,
        _frame: &StackFrame,
        context: &SharedExecutionContext,
    ) -> HashMap<String, JsonValue> {
        // For now, return all variables in context
        // In a real implementation, we'd filter by scope
        context.variables.clone()
    }

    /// Format the entire stack trace (common implementation)
    pub fn format_stack_trace_common(&self, stack: &[StackFrame], current_frame: usize) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        for (i, frame) in stack.iter().enumerate() {
            let marker = if i == current_frame { ">" } else { " " };
            let formatted_frame = self.format_frame_common(frame);
            let _ = writeln!(result, "{marker} [{i}] {formatted_frame}");
        }

        result
    }
}

impl Default for SharedStackNavigator {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating script-specific stack navigators
pub trait StackNavigatorFactory {
    /// Create a new stack navigator for the current script engine
    fn create_navigator(&self) -> Arc<dyn StackNavigator>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_navigator_creation() {
        let navigator = SharedStackNavigator::new();
        assert_eq!(navigator.get_current_frame(), 0);
    }

    #[test]
    fn test_current_frame_tracking() {
        let navigator = SharedStackNavigator::new();
        navigator.set_current_frame(5);
        assert_eq!(navigator.get_current_frame(), 5);
    }

    #[test]
    fn test_navigate_to_valid_frame() {
        let navigator = SharedStackNavigator::new();
        let stack = vec![
            StackFrame {
                id: "frame1".to_string(),
                name: "main".to_string(),
                source: "test.lua".to_string(),
                line: 10,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: "frame2".to_string(),
                name: "helper".to_string(),
                source: "test.lua".to_string(),
                line: 20,
                column: Some(5),
                locals: Vec::new(),
                is_user_code: true,
            },
        ];

        let result = navigator.navigate_to_frame_common(1, &stack);
        assert!(result.is_ok());
        let frame = result.unwrap();
        assert_eq!(frame.name, "helper");
        assert_eq!(navigator.get_current_frame(), 1);
    }

    #[test]
    fn test_navigate_to_invalid_frame() {
        let navigator = SharedStackNavigator::new();
        let stack = vec![StackFrame {
            id: "frame1".to_string(),
            name: "main".to_string(),
            source: "test.lua".to_string(),
            line: 10,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        }];

        let result = navigator.navigate_to_frame_common(5, &stack);
        assert!(result.is_err());
    }

    #[test]
    fn test_format_frame() {
        let navigator = SharedStackNavigator::new();
        let frame = StackFrame {
            id: "frame1".to_string(),
            name: "main".to_string(),
            source: "test.lua".to_string(),
            line: 10,
            column: Some(5),
            locals: Vec::new(),
            is_user_code: true,
        };

        let formatted = navigator.format_frame_common(&frame);
        assert_eq!(formatted, "test.lua:10:5 in main");

        let frame_no_column = StackFrame {
            id: "frame2".to_string(),
            name: "helper".to_string(),
            source: "test.lua".to_string(),
            line: 20,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        };

        let formatted = navigator.format_frame_common(&frame_no_column);
        assert_eq!(formatted, "test.lua:20 in helper");
    }

    #[test]
    fn test_format_stack_trace() {
        let navigator = SharedStackNavigator::new();
        let stack = vec![
            StackFrame {
                id: "frame1".to_string(),
                name: "main".to_string(),
                source: "test.lua".to_string(),
                line: 10,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: "frame2".to_string(),
                name: "helper".to_string(),
                source: "test.lua".to_string(),
                line: 20,
                column: Some(5),
                locals: Vec::new(),
                is_user_code: true,
            },
        ];

        let trace = navigator.format_stack_trace_common(&stack, 0);
        assert!(trace.contains("> [0] test.lua:10 in main"));
        assert!(trace.contains("  [1] test.lua:20:5 in helper"));
    }
}
