//! Lua-specific implementation of stack navigation
//!
//! This module provides the Lua-specific implementation of stack navigation,
//! including Lua-specific frame formatting and variable extraction.
//!
//! Follows the three-layer bridge architecture:
//! - Bridge Layer: `StackNavigator` trait (script-agnostic)
//! - Shared Layer: `SharedStackNavigator` (common implementation)  
//! - Script Layer: `LuaStackNavigator` (Lua-specific, this file)

use crate::execution_bridge::StackFrame;
use crate::execution_context::SharedExecutionContext;
use crate::stack_navigator::{SharedStackNavigator, StackNavigator};
use mlua::Lua;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::error::Error;

/// Lua-specific stack navigator
///
/// This struct provides Lua-specific functionality for stack navigation,
/// including enhanced frame formatting and Lua-aware variable extraction.
/// All script-agnostic operations are delegated to the shared implementation.
pub struct LuaStackNavigator {
    /// Shared implementation for script-agnostic operations
    shared: SharedStackNavigator,
}

impl LuaStackNavigator {
    /// Create a new Lua stack navigator
    #[must_use]
    pub const fn new() -> Self {
        Self {
            shared: SharedStackNavigator::new(),
        }
    }

    /// Format a stack frame with Lua-specific enhancements
    ///
    /// This method adds Lua-specific details to stack frames,
    /// such as chunk names, tail call information, etc.
    pub fn format_frame_with_lua(&self, frame: &StackFrame, _lua: &Lua) -> String {
        // Start with basic formatting
        let mut result = self.shared.format_frame_common(frame);

        // Add Lua-specific details if available
        // Check if this is a Lua chunk
        if frame.source.to_lowercase().ends_with(".lua") || frame.source.starts_with('[') {
            // Lua chunks often have names like "[string \"...\"]"
            if frame.source.starts_with("[string") {
                result = format!("{result} (Lua chunk)");
            }

            // Check for special Lua function names
            match frame.name.as_str() {
                "(tail call)" => result = format!("{result} [tail call optimization]"),
                "(main chunk)" => result = format!("{result} [main]"),
                "(C)" => result = format!("{result} [C function]"),
                _ => {}
            }
        }

        result
    }

    /// Extract Lua-specific variables from a frame
    ///
    /// This method would ideally extract local variables, upvalues,
    /// and other Lua-specific context. For now, it delegates to the
    /// shared implementation.
    pub fn get_lua_frame_variables(
        &self,
        frame: &StackFrame,
        context: &SharedExecutionContext,
        _lua: &Lua,
    ) -> HashMap<String, JsonValue> {
        // For now, use the common implementation
        // In a full implementation, we would:
        // 1. Extract local variables from the Lua debug info
        // 2. Get upvalues for the current function
        // 3. Filter by scope/frame
        self.shared.get_frame_variables_common(frame, context)
    }

    /// Get reference to the shared navigator
    #[must_use]
    pub const fn shared(&self) -> &SharedStackNavigator {
        &self.shared
    }
}

impl Default for LuaStackNavigator {
    fn default() -> Self {
        Self::new()
    }
}

impl StackNavigator for LuaStackNavigator {
    fn navigate_to_frame(
        &self,
        frame_index: usize,
        stack: &[StackFrame],
    ) -> Result<StackFrame, Box<dyn Error>> {
        // Use shared implementation for navigation
        self.shared.navigate_to_frame_common(frame_index, stack)
    }

    fn format_frame(&self, frame: &StackFrame) -> String {
        // Use Lua-aware formatting
        // In a real implementation, we'd have access to Lua context here
        // For now, use enhanced formatting without Lua context
        let mut result = self.shared.format_frame_common(frame);

        // Add Lua-specific enhancements based on frame data alone
        // Check for special Lua function names regardless of source
        if frame.name == "(main chunk)" {
            result = format!("{result} [main]");
        } else if frame.name == "(tail call)" {
            result = format!("{result} [tail]");
        } else if frame.name == "(C)" {
            result = format!("{result} [C function]");
        }

        result
    }

    fn get_frame_variables(
        &self,
        frame: &StackFrame,
        context: &SharedExecutionContext,
    ) -> HashMap<String, JsonValue> {
        // Use shared implementation
        // In a full implementation, we'd extract Lua-specific variables
        self.shared.get_frame_variables_common(frame, context)
    }

    fn format_stack_trace(&self, stack: &[StackFrame], current_frame: usize) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        for (i, frame) in stack.iter().enumerate() {
            let marker = if i == current_frame { "→" } else { " " };
            let formatted_frame = self.format_frame(frame);
            let _ = writeln!(result, "{marker} [{i}] {formatted_frame}");
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_navigator_creation() {
        let navigator = LuaStackNavigator::new();
        assert_eq!(navigator.shared().get_current_frame(), 0);
    }

    #[test]
    fn test_lua_frame_formatting() {
        let navigator = LuaStackNavigator::new();

        let frame = StackFrame {
            id: "frame1".to_string(),
            name: "(main chunk)".to_string(),
            source: "test.lua".to_string(),
            line: 10,
            column: None,
            locals: Vec::new(),
            is_user_code: true,
        };

        let formatted = navigator.format_frame(&frame);
        assert!(formatted.contains("test.lua"));
        assert!(formatted.contains("[main]"));
    }

    #[test]
    fn test_tail_call_formatting() {
        let navigator = LuaStackNavigator::new();

        let frame = StackFrame {
            id: "frame1".to_string(),
            name: "(tail call)".to_string(),
            source: "test.lua".to_string(),
            line: 20,
            column: Some(5),
            locals: Vec::new(),
            is_user_code: true,
        };

        let formatted = navigator.format_frame(&frame);
        assert!(formatted.contains("[tail]"));
    }

    #[test]
    fn test_lua_stack_trace_formatting() {
        let navigator = LuaStackNavigator::new();

        let stack = vec![
            StackFrame {
                id: "frame1".to_string(),
                name: "(main chunk)".to_string(),
                source: "main.lua".to_string(),
                line: 1,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: "frame2".to_string(),
                name: "helper".to_string(),
                source: "helper.lua".to_string(),
                line: 10,
                column: Some(5),
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: "frame3".to_string(),
                name: "(tail call)".to_string(),
                source: "utils.lua".to_string(),
                line: 20,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
        ];

        let trace = navigator.format_stack_trace(&stack, 1);

        // Check that current frame is marked
        assert!(trace.contains("→ [1]"));

        // Check that Lua-specific formatting is applied
        assert!(trace.contains("[main]"));
        assert!(trace.contains("[tail]"));
    }

    #[test]
    fn test_navigation_with_lua_stack() {
        let navigator = LuaStackNavigator::new();

        let stack = vec![
            StackFrame {
                id: "frame1".to_string(),
                name: "main".to_string(),
                source: "test.lua".to_string(),
                line: 1,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
            StackFrame {
                id: "frame2".to_string(),
                name: "helper".to_string(),
                source: "test.lua".to_string(),
                line: 10,
                column: None,
                locals: Vec::new(),
                is_user_code: true,
            },
        ];

        let result = navigator.navigate_to_frame(1, &stack);
        assert!(result.is_ok());

        let frame = result.unwrap();
        assert_eq!(frame.name, "helper");
        assert_eq!(navigator.shared().get_current_frame(), 1);
    }
}
