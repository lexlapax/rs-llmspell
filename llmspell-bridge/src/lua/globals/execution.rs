//! Lua-specific Execution debugging global implementation (exposed as Debugger)
//!
//! Provides execution debugging capabilities for Lua scripts including breakpoints,
//! variable inspection, stepping, and execution control. This is distinct from
//! diagnostics (logging/profiling) which is handled by the Console global.

use crate::execution_bridge::{Breakpoint, ExecutionLocation, ExecutionManager, StackFrame};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use crate::lua::output::{capture_stack_trace, StackTraceOptions};
use crate::lua::sync_utils::block_on_async;
use mlua::{DebugEvent, Lua, Result as LuaResult};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Lua execution debug hook handler
pub struct LuaExecutionHook {
    /// Reference to the execution manager
    execution_manager: Arc<ExecutionManager>,
    /// Shared execution context for enriched debugging
    shared_context: Arc<RwLock<SharedExecutionContext>>,
    /// Map of source to breakpoints (cached for performance)
    breakpoints: HashMap<String, Vec<Breakpoint>>,
    /// Current execution state
    current_line: u32,
    current_source: String,
}

impl LuaExecutionHook {
    /// Create a new Lua execution hook
    #[must_use]
    pub fn new(
        execution_manager: Arc<ExecutionManager>,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
    ) -> Self {
        Self {
            execution_manager,
            shared_context,
            breakpoints: HashMap::new(),
            current_line: 0,
            current_source: String::new(),
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

    /// Check if we should break at current location (now uses `ExecutionManager`)
    fn should_break(&self, source: &str, line: u32) -> bool {
        // Use ExecutionManager's should_break_at for consistency
        let exec_mgr = self.execution_manager.clone();
        let source = source.to_string();
        block_on_async(
            "check_breakpoint",
            async move {
                if exec_mgr.should_break_at(&source, line).await {
                    Ok(())
                } else {
                    Err(std::io::Error::other("not breaking"))
                }
            },
            None,
        )
        .is_ok()
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

                // Update shared context location
                let shared_ctx = self.shared_context.clone();
                let source_clone = source.clone();
                let _ = block_on_async::<_, (), std::io::Error>(
                    "update_location",
                    async move {
                        {
                            let mut ctx = shared_ctx.write().await;
                            ctx.set_location(SourceLocation {
                                source: source_clone,
                                line,
                                column: None,
                            });
                            ctx.performance_metrics.execution_count += 1;
                        }
                        Ok(())
                    },
                    None,
                );

                // Check for breakpoint
                if self.should_break(&source, line) {
                    // Pause execution with enriched context
                    self.pause_at_breakpoint_with_context(lua, source, line);
                }
            }
            DebugEvent::Call | DebugEvent::TailCall => {
                // Function call - push to stack in shared context
                let source = ar
                    .source()
                    .source
                    .as_deref()
                    .unwrap_or("<unknown>")
                    .to_string();
                let name = ar
                    .names()
                    .name
                    .as_deref()
                    .unwrap_or("<anonymous>")
                    .to_string();
                #[allow(clippy::cast_sign_loss)]
                let line = ar.curr_line() as u32;

                let shared_ctx = self.shared_context.clone();
                let _ = block_on_async::<_, (), std::io::Error>(
                    "push_frame",
                    async move {
                        {
                            let mut ctx = shared_ctx.write().await;
                            let frame_id = format!("frame_{}", ctx.stack.len());
                            ctx.push_frame(StackFrame {
                                id: frame_id,
                                name,
                                source,
                                line,
                                column: None,
                                locals: Vec::new(),
                                is_user_code: true,
                            });
                        }
                        Ok(())
                    },
                    None,
                );
            }
            DebugEvent::Ret => {
                // Function return - pop from stack in shared context
                let shared_ctx = self.shared_context.clone();
                let _ = block_on_async::<_, (), std::io::Error>(
                    "pop_frame",
                    async move {
                        {
                            let mut ctx = shared_ctx.write().await;
                            ctx.pop_frame();
                        }
                        Ok(())
                    },
                    None,
                );
            }
            DebugEvent::Count | DebugEvent::Unknown(_) => {
                // Count: For instruction counting (not used)
                // Unknown: Ignored
            }
        }
        Ok(())
    }

    /// Pause execution at a breakpoint with enriched context
    fn pause_at_breakpoint_with_context(&self, lua: &Lua, source: String, line: u32) {
        // Create execution location
        let location = ExecutionLocation {
            source: source.clone(),
            line,
            column: None,
        };

        // Capture comprehensive stack trace using output.rs
        let stack_options = StackTraceOptions {
            max_depth: 20,
            capture_locals: true,
            capture_upvalues: false,
            include_source: true,
        };
        let stack_trace = capture_stack_trace(lua, &stack_options);

        // Use stack frames directly from capture_stack_trace
        let stack_frames = stack_trace.frames.clone();

        // Extract local variables for context
        let mut variables = HashMap::new();
        if let Some(first_frame) = stack_trace.frames.first() {
            for local in &first_frame.locals {
                // Convert Lua value string representation to JSON for SharedExecutionContext
                let json_value = Self::lua_value_to_json(&local.value);
                variables.insert(local.name.clone(), json_value);
            }
        }

        // Update shared context and suspend (but don't block waiting for resume)
        let shared_ctx = self.shared_context.clone();
        let exec_mgr = self.execution_manager.clone();
        let _ = block_on_async::<_, (), std::io::Error>(
            "suspend_for_debugging",
            async move {
                let mut ctx = shared_ctx.write().await;
                ctx.stack.clone_from(&stack_frames);
                ctx.variables = variables;
                ctx.set_location(SourceLocation {
                    source,
                    line,
                    column: None,
                });

                let context_clone = ctx.clone();
                drop(ctx); // Release the lock before suspension

                // Suspend with enriched context (sets paused state but doesn't block)
                exec_mgr
                    .suspend_for_debugging(location, context_clone)
                    .await;

                // NOTE: We don't wait for resume here as that would block the Lua execution.
                // The debugger client should handle resuming when ready.
                Ok(())
            },
            None,
        );
    }

    /// Convert Lua value representation to JSON
    fn lua_value_to_json(value_str: &str) -> serde_json::Value {
        // Try to parse as various JSON types
        if value_str == "nil" {
            serde_json::Value::Null
        } else if let Ok(b) = value_str.parse::<bool>() {
            serde_json::Value::Bool(b)
        } else if let Ok(n) = value_str.parse::<f64>() {
            serde_json::Value::Number(
                serde_json::Number::from_f64(n).unwrap_or_else(|| serde_json::Number::from(0)),
            )
        } else {
            // Default to string
            serde_json::Value::String(value_str.to_string())
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
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
    install_interactive_debug_hooks(lua, execution_manager, shared_context)
}

/// Install interactive debug hooks with `SharedExecutionContext`
///
/// # Errors
///
/// Returns an error if debug hook installation fails
pub fn install_interactive_debug_hooks(
    lua: &Lua,
    execution_manager: Arc<ExecutionManager>,
    shared_context: Arc<RwLock<SharedExecutionContext>>,
) -> LuaResult<Arc<parking_lot::Mutex<LuaExecutionHook>>> {
    let hook = Arc::new(parking_lot::Mutex::new(LuaExecutionHook::new(
        execution_manager,
        shared_context,
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
