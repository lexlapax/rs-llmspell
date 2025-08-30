//! Lua-specific Execution debugging global implementation (exposed as Debugger)
//!
//! Provides execution debugging capabilities for Lua scripts including breakpoints,
//! variable inspection, stepping, and execution control. This is distinct from
//! diagnostics (logging/profiling) which is handled by the Console global.

use crate::execution_bridge::{
    Breakpoint, DebugState, ExecutionLocation, ExecutionManager, PauseReason, StackFrame, Variable,
};
use mlua::{DebugEvent, Lua, Result as LuaResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Handle;

/// Lua execution debug hook handler
pub struct LuaExecutionHook {
    /// Reference to the execution manager
    execution_manager: Arc<ExecutionManager>,
    /// Map of source to breakpoints
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    /// Current execution state
    current_line: u32,
    current_source: String,
    /// Tokio runtime handle for async operations
    runtime_handle: Handle,
}

impl LuaExecutionHook {
    /// Create a new Lua execution hook
    #[must_use]
    pub fn new(execution_manager: Arc<ExecutionManager>) -> Self {
        Self {
            execution_manager,
            breakpoints: HashMap::new(),
            current_line: 0,
            current_source: String::new(),
            runtime_handle: Handle::current(),
        }
    }

    /// Update breakpoints from debug manager
    pub fn update_breakpoints(&mut self, breakpoints: Vec<Breakpoint>) {
        self.breakpoints.clear();
        for bp in breakpoints {
            self.breakpoints
                .entry(bp.source.clone())
                .or_default()
                .push(bp);
        }
    }

    /// Check if we should break at current location
    fn should_break(&mut self, source: &str, line: u32) -> bool {
        if let Some(breakpoints) = self.breakpoints.get_mut(source) {
            for bp in breakpoints.iter_mut() {
                if bp.line == line && bp.should_break() {
                    bp.current_hits += 1;
                    return true;
                }
            }
        }
        false
    }

    /// Handle a debug event
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint handling fails
    pub fn handle_event(
        &mut self,
        lua: &Lua,
        ar: &mlua::Debug,
        event: DebugEvent,
    ) -> LuaResult<()> {
        match event {
            DebugEvent::Line => {
                // Get source and line info
                let source = ar
                    .source()
                    .source
                    .as_deref()
                    .unwrap_or("<unknown>")
                    .to_string();
                #[allow(clippy::cast_sign_loss)]
                let line = ar.curr_line() as u32;

                self.current_source.clone_from(&source);
                self.current_line = line;

                // Check for breakpoint
                if self.should_break(&source, line) {
                    // Pause execution
                    self.pause_at_breakpoint(lua, source, line);
                }
            }
            DebugEvent::Call
            | DebugEvent::TailCall
            | DebugEvent::Ret
            | DebugEvent::Count
            | DebugEvent::Unknown(_) => {
                // These events are not used for breakpoint debugging
                // Call/TailCall/Ret: Could be used for stack trace (not implemented)
                // Count: For instruction counting
                // Unknown: Ignored
            }
        }
        Ok(())
    }

    /// Pause execution at a breakpoint
    fn pause_at_breakpoint(&self, lua: &Lua, source: String, line: u32) {
        // Update debug state
        let location = ExecutionLocation {
            source,
            line,
            column: None,
        };

        // Set paused state (blocking call to async)
        self.runtime_handle.block_on(async {
            self.execution_manager
                .set_state(DebugState::Paused {
                    reason: PauseReason::Breakpoint,
                    location: location.clone(),
                })
                .await;
        });

        // Extract stack trace
        let stack_trace = Self::extract_stack_trace(lua);
        self.runtime_handle.block_on(async {
            self.execution_manager.set_stack_trace(stack_trace).await;
        });

        // Extract variables for top frame
        let variables = Self::extract_variables(lua, 0);
        self.runtime_handle.block_on(async {
            self.execution_manager
                .cache_variables("frame_0".to_string(), variables)
                .await;
        });

        // Wait for continue command
        self.wait_for_continue();
    }

    /// Extract stack trace from Lua
    fn extract_stack_trace(lua: &Lua) -> Vec<StackFrame> {
        let mut frames = Vec::new();
        let mut level = 0;

        while let Some(debug) = lua.inspect_stack(level) {
            let frame = StackFrame {
                id: format!("frame_{level}"),
                name: debug
                    .names()
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous>")
                    .to_string(),
                source: debug
                    .source()
                    .source
                    .as_deref()
                    .unwrap_or("<unknown>")
                    .to_string(),
                #[allow(clippy::cast_sign_loss)]
                line: debug.curr_line() as u32,
                column: None,
                locals: Vec::new(), // Will be populated on demand
                is_user_code: !debug
                    .source()
                    .source
                    .as_deref()
                    .unwrap_or("")
                    .starts_with('@'),
            };
            frames.push(frame);
            level += 1;
        }

        frames
    }

    /// Extract local variables from a stack frame
    fn extract_variables(lua: &Lua, frame_level: usize) -> Vec<Variable> {
        let variables = Vec::new();

        // Get debug info for the frame
        if let Some(_debug) = lua.inspect_stack(frame_level) {
            // Note: mlua doesn't provide direct access to locals and upvalues
            // through the Debug interface. This would require using the Lua debug
            // library directly or implementing a custom solution.
            // TODO: Implement proper variable extraction using Lua debug library
        }

        variables
    }

    /// Wait for continue command from debug manager
    fn wait_for_continue(&self) {
        // Simple busy wait - in production, use proper synchronization
        loop {
            let state = self
                .runtime_handle
                .block_on(async { self.execution_manager.get_state().await });

            if state == DebugState::Running {
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
}

/// Install debug hooks in a Lua instance
///
/// # Errors
///
/// Returns an error if debug hook installation fails
pub fn install_debug_hooks(
    lua: &Lua,
    execution_manager: Arc<ExecutionManager>,
) -> LuaResult<Arc<parking_lot::Mutex<LuaExecutionHook>>> {
    let hook = Arc::new(parking_lot::Mutex::new(LuaExecutionHook::new(
        execution_manager,
    )));
    let hook_clone = hook.clone();

    // Set up the debug hook with line and call events
    lua.set_hook(
        mlua::HookTriggers {
            on_calls: true,
            on_returns: true,
            every_line: true,
            ..Default::default()
        },
        move |lua, ar| {
            let mut hook = hook_clone.lock();
            // Determine the event type based on the activation record
            let event = if ar.curr_line() != -1 {
                DebugEvent::Line
            } else if ar.event() == mlua::DebugEvent::Call {
                DebugEvent::Call
            } else if ar.event() == mlua::DebugEvent::TailCall {
                DebugEvent::TailCall
            } else if ar.event() == mlua::DebugEvent::Ret {
                DebugEvent::Ret
            } else {
                return Ok(());
            };
            hook.handle_event(lua, &ar, event)
        },
    );

    Ok(hook)
}

/// Remove debug hooks from a Lua instance
pub fn remove_debug_hooks(lua: &Lua) {
    lua.remove_hook();
}

/// Check if a Lua value can be inspected further
#[must_use]
pub const fn is_inspectable(value: &mlua::Value) -> bool {
    matches!(value, mlua::Value::Table(_) | mlua::Value::UserData(_))
}

/// Inject Debugger global into Lua environment (execution debugging)
///
/// This provides script-accessible debugging capabilities like setting breakpoints,
/// stepping through code, and inspecting variables.
///
/// # Errors
///
/// Returns an error if global injection fails
pub fn inject_execution_global(
    lua: &mlua::Lua,
    _context: &crate::globals::GlobalContext,
    manager: &Arc<ExecutionManager>,
) -> mlua::Result<()> {
    let debugger_table = lua.create_table()?;

    // Debugger.break() - trigger a breakpoint programmatically
    let _manager_clone = manager.clone();
    let break_fn = lua.create_function(move |_, ()| {
        // This would trigger a breakpoint at the current location
        // For now, just return a placeholder
        Ok("Breakpoint functionality not yet implemented")
    })?;
    debugger_table.set("break", break_fn)?;

    // Debugger.step() - step to next line
    let step_fn = lua.create_function(|_, ()| Ok("Step functionality not yet implemented"))?;
    debugger_table.set("step", step_fn)?;

    // Debugger.continue() - continue execution
    let continue_fn =
        lua.create_function(|_, ()| Ok("Continue functionality not yet implemented"))?;
    debugger_table.set("continue", continue_fn)?;

    // Debugger.inspect(value) - inspect a value in detail
    let inspect_fn =
        lua.create_function(|_, value: mlua::Value| Ok(crate::lua::output::format_simple(&value)))?;
    debugger_table.set("inspect", inspect_fn)?;

    // Debugger.getLocals() - get local variables
    let get_locals_fn = lua.create_function(|lua, ()| {
        let table = lua.create_table()?;
        // Would return actual locals when debugging is active
        Ok(table)
    })?;
    debugger_table.set("getLocals", get_locals_fn)?;

    // Set the Debugger global
    lua.globals().set("Debugger", debugger_table)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_inspectable() {
        assert!(!is_inspectable(&mlua::Value::Nil));
        assert!(!is_inspectable(&mlua::Value::Boolean(true)));
        assert!(!is_inspectable(&mlua::Value::Integer(42)));
    }
}
