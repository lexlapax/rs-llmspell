//! Lua-specific debug bridge implementation
//!
//! This module provides Lua-specific debugging capabilities including
//! hook integration, state inspection, and breakpoint handling.
//!
//! Migrated from Phase-9 branch `lua_debug_bridge.rs` and related files

use super::execution_bridge::{Breakpoint, StackFrame, Variable};
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, instrument, trace};

/// Lua debug hook types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LuaHookType {
    /// Called when entering a function
    Call,
    /// Called when returning from a function
    Return,
    /// Called for each line executed
    Line,
    /// Called periodically during execution
    Count,
}

/// Lua debug state information
#[derive(Debug, Clone)]
pub struct LuaDebugState {
    /// Current function name
    pub function_name: Option<String>,
    /// Current source file
    pub source: String,
    /// Current line number
    pub line: u32,
    /// Stack depth
    pub stack_depth: usize,
    /// Local variables
    pub locals: Vec<Variable>,
    /// Upvalues
    pub upvalues: Vec<Variable>,
}

/// Lua debug bridge for debugging Lua scripts
pub struct LuaDebugBridge {
    /// Active breakpoints
    breakpoints: Arc<RwLock<HashMap<String, Vec<Breakpoint>>>>,
    /// Debug hook enabled
    hook_enabled: Arc<RwLock<bool>>,
    /// Current debug state
    current_state: Arc<RwLock<Option<LuaDebugState>>>,
    /// Cached stack frames
    cached_frames: Arc<RwLock<Vec<StackFrame>>>,
    /// Session ID
    _session_id: String,
}

impl LuaDebugBridge {
    /// Create a new Lua debug bridge
    pub fn new(session_id: String) -> Self {
        Self {
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            hook_enabled: Arc::new(RwLock::new(false)),
            current_state: Arc::new(RwLock::new(None)),
            cached_frames: Arc::new(RwLock::new(Vec::new())),
            _session_id: session_id,
        }
    }

    /// Enable debug hook
    pub fn enable_hook(&self) {
        *self.hook_enabled.write() = true;
        debug!("Lua debug hook enabled");
    }

    /// Disable debug hook
    pub fn disable_hook(&self) {
        *self.hook_enabled.write() = false;
        debug!("Lua debug hook disabled");
    }

    /// Check if hook is enabled
    pub fn is_hook_enabled(&self) -> bool {
        *self.hook_enabled.read()
    }

    /// Handle debug hook callback
    #[instrument(level = "trace", skip(self))]
    pub fn on_hook(&self, hook_type: LuaHookType, state: &LuaDebugState) -> bool {
        if !self.is_hook_enabled() {
            return false;
        }

        trace!(
            "Lua hook: {:?} at {}:{}",
            hook_type,
            state.source,
            state.line
        );

        // Update current state
        *self.current_state.write() = Some(state.clone());

        // Check for breakpoints on line hooks
        if hook_type == LuaHookType::Line {
            if let Some(bp_list) = self.breakpoints.read().get(&state.source) {
                for bp in bp_list {
                    if bp.line == state.line && bp.should_break() {
                        debug!("Hit breakpoint at {}:{}", state.source, state.line);
                        return true; // Pause execution
                    }
                }
            }
        }

        false
    }

    /// Set a breakpoint
    pub fn set_breakpoint(&self, source: String, line: u32) -> Breakpoint {
        let breakpoint = Breakpoint::new(source.clone(), line);

        self.breakpoints
            .write()
            .entry(source)
            .or_default()
            .push(breakpoint.clone());

        debug!(
            "Lua breakpoint set at {}:{}",
            breakpoint.source, breakpoint.line
        );
        breakpoint
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

    /// Get current stack frames
    pub fn get_stack_frames(&self) -> Vec<StackFrame> {
        self.cached_frames.read().clone()
    }

    /// Update stack frames from Lua state
    pub fn update_stack_frames(&self, frames: Vec<StackFrame>) {
        *self.cached_frames.write() = frames;
    }

    /// Get local variables for current frame
    pub fn get_locals(&self) -> Vec<Variable> {
        self.current_state
            .read()
            .as_ref()
            .map(|state| state.locals.clone())
            .unwrap_or_default()
    }

    /// Get upvalues for current frame
    pub fn get_upvalues(&self) -> Vec<Variable> {
        self.current_state
            .read()
            .as_ref()
            .map(|state| state.upvalues.clone())
            .unwrap_or_default()
    }

    /// Evaluate expression in current context
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails
    pub fn evaluate(&self, expression: &str) -> Result<String> {
        // Simplified evaluation - would integrate with actual Lua state
        trace!("Evaluating expression: {}", expression);
        Ok(format!("<evaluated: {expression}>"))
    }
}

/// Debug hook adapter for integrating with Lua runtime
pub struct DebugHookAdapter {
    /// Debug bridge
    bridge: Arc<LuaDebugBridge>,
    /// Execution manager
    execution_manager: Option<Arc<super::execution_bridge::ExecutionManager>>,
}

impl DebugHookAdapter {
    /// Create a new debug hook adapter
    pub fn new(bridge: Arc<LuaDebugBridge>) -> Self {
        Self {
            bridge,
            execution_manager: None,
        }
    }

    /// Set execution manager
    pub fn set_execution_manager(
        &mut self,
        manager: Arc<super::execution_bridge::ExecutionManager>,
    ) {
        self.execution_manager = Some(manager);
    }

    /// Handle Lua debug hook
    pub fn handle_hook(&self, hook_type: LuaHookType, state: &LuaDebugState) -> bool {
        // Check with execution manager first
        if let Some(ref manager) = self.execution_manager {
            if manager.should_pause(&state.source, state.line) {
                manager.pause();
                return true;
            }
        }

        // Then check with Lua bridge
        self.bridge.on_hook(hook_type, state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_debug_bridge() {
        let bridge = LuaDebugBridge::new("test-session".to_string());

        assert!(!bridge.is_hook_enabled());

        bridge.enable_hook();
        assert!(bridge.is_hook_enabled());

        // Set breakpoint
        let bp = bridge.set_breakpoint("test.lua".to_string(), 10);
        assert!(!bp.id.is_empty());

        // Create debug state
        let state = LuaDebugState {
            function_name: Some("test_func".to_string()),
            source: "test.lua".to_string(),
            line: 10,
            stack_depth: 1,
            locals: vec![],
            upvalues: vec![],
        };

        // Should pause at breakpoint
        assert!(bridge.on_hook(LuaHookType::Line, &state.clone()));

        // Should not pause at different line
        let mut state2 = state;
        state2.line = 11;
        assert!(!bridge.on_hook(LuaHookType::Line, &state2));
    }

    #[test]
    fn test_debug_hook_adapter() {
        let bridge = Arc::new(LuaDebugBridge::new("test-session".to_string()));
        let adapter = DebugHookAdapter::new(bridge.clone());

        bridge.enable_hook();
        bridge.set_breakpoint("test.lua".to_string(), 5);

        let state = LuaDebugState {
            function_name: None,
            source: "test.lua".to_string(),
            line: 5,
            stack_depth: 0,
            locals: vec![],
            upvalues: vec![],
        };

        // Should pause at breakpoint
        assert!(adapter.handle_hook(LuaHookType::Line, &state));
    }
}
