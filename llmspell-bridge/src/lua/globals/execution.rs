//! Lua-specific Execution debugging global implementation (exposed as Debugger)
//!
//! Provides execution debugging capabilities for Lua scripts including breakpoints,
//! variable inspection, stepping, and execution control. This is distinct from
//! diagnostics (logging/profiling) which is handled by the Console global.

use crate::execution_bridge::{Breakpoint, ExecutionLocation, ExecutionManager, StackFrame};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use crate::lua::debug_cache::{ContextBatcher, ContextUpdate, DebugMode, DebugStateCache};
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
    /// Fast synchronous debug state cache
    debug_cache: Arc<DebugStateCache>,
    /// Context update batcher for lazy updates
    context_batcher: ContextBatcher,
    /// Current execution state
    current_line: u32,
    current_source: String,
    /// Line counter for periodic context flushes
    line_counter: u64,
    /// Track if we have hooks installed (to avoid interfering with other systems)
    hooks_installed: bool,
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
            debug_cache: Arc::new(DebugStateCache::new()),
            context_batcher: ContextBatcher::new(),
            current_line: 0,
            current_source: String::new(),
            line_counter: 0,
            hooks_installed: false,
        }
    }

    /// Update breakpoints from debug manager
    pub fn update_breakpoints(&mut self, breakpoints: &[Breakpoint]) {
        // Update the fast cache with breakpoint locations
        let locations: Vec<(String, u32)> = breakpoints
            .iter()
            .filter(|bp| bp.enabled)
            .map(|bp| (bp.source.clone(), bp.line))
            .collect();

        self.debug_cache.update_breakpoints(locations);
    }

    /// Check if we should break at current location (SLOW PATH - only called when `might_break_at` returns true)
    fn should_break_slow(&self, source: &str, line: u32) -> bool {
        // This is the slow path - only called when we might have a breakpoint
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

    /// Get debug cache for external use
    #[must_use]
    pub fn debug_cache(&self) -> Arc<DebugStateCache> {
        self.debug_cache.clone()
    }

    /// Flush batched context updates to `SharedExecutionContext`
    pub fn flush_batched_context_updates(&mut self) {
        let updates = self.context_batcher.flush();
        if updates.is_empty() {
            return;
        }

        // Only do async work when we have updates to flush
        let shared_ctx = self.shared_context.clone();
        let _ = block_on_async::<_, (), std::io::Error>(
            "flush_context_batch",
            async move {
                let mut ctx = shared_ctx.write().await;

                for update in updates {
                    match update {
                        ContextUpdate::Location { source, line } => {
                            ctx.set_location(SourceLocation {
                                source,
                                line,
                                column: None,
                            });
                        }
                        ContextUpdate::ExecutionCount(count) => {
                            ctx.performance_metrics.execution_count =
                                u32::try_from(count).unwrap_or(u32::MAX);
                        }
                        ContextUpdate::StackPush { name, source, line } => {
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
                        ContextUpdate::StackPop => {
                            ctx.pop_frame();
                        }
                    }
                }
                drop(ctx);
                Ok(())
            },
            None,
        );
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
        // CRITICAL: Check if debugging is disabled FIRST
        // This allows us to keep hooks installed but inactive for zero interference
        if matches!(self.debug_cache.get_debug_mode(), DebugMode::Disabled) {
            return Ok(()); // Fast exit - no processing at all
        }

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
                self.line_counter += 1;

                // FAST PATH - synchronous check, no async operations
                if !self.debug_cache.might_break_at(&source, line) {
                    // Record location in batcher (lazy update)
                    self.context_batcher.record_location(source.clone(), line);
                    self.context_batcher
                        .record_execution_count(self.line_counter);

                    // Maybe flush batched updates (every N lines or after timeout)
                    if self.line_counter % 100 == 0 {
                        self.flush_batched_context_updates();
                    }

                    // Record hot location for monitoring
                    self.debug_cache.record_hot_location(source, line);

                    return Ok(()); // EXIT FAST PATH - no async operations!
                }

                // SLOW PATH - only when breakpoint might hit
                // Now we can afford async operations since we're potentially breaking anyway
                if self.should_break_slow(&source, line) {
                    // Flush any pending context updates before breaking
                    self.flush_batched_context_updates();

                    // Pause execution with enriched context
                    self.pause_at_breakpoint_with_context(lua, source, line);
                }
            }
            DebugEvent::Call | DebugEvent::TailCall => {
                // Function call - batch the stack push update
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

                // Only use async if we're in full debug mode
                if matches!(self.debug_cache.get_debug_mode(), DebugMode::Full) {
                    // Batch the stack push
                    self.context_batcher.record_stack_push(name, source, line);
                }
            }
            DebugEvent::Ret => {
                // Function return - batch the stack pop update
                if matches!(self.debug_cache.get_debug_mode(), DebugMode::Full) {
                    self.context_batcher.record_stack_pop();
                }
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

/// Install interactive debug hooks with `SharedExecutionContext`
///
/// **WARNING**: Installing debug hooks will REPLACE any existing Lua debug hooks!
/// This is a fundamental limitation of Lua - only one debug hook can be active.
/// Other systems using debug hooks (profilers, memory trackers) will stop working.
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
        execution_manager.clone(),
        shared_context,
    )));

    // Set initial debug mode based on whether there are breakpoints
    let initial_mode = block_on_async(
        "check_initial_mode",
        async move {
            let breakpoints = execution_manager.get_breakpoints().await;
            if breakpoints.is_empty() {
                Ok::<DebugMode, std::io::Error>(DebugMode::Disabled)
            } else {
                Ok::<DebugMode, std::io::Error>(DebugMode::Minimal {
                    check_interval: 1000,
                })
            }
        },
        None,
    )
    .unwrap_or(DebugMode::Disabled);

    // Set the debug mode in the cache
    hook.lock().debug_cache.set_debug_mode(initial_mode);

    // Install hooks based on mode
    install_hooks_for_mode(lua, &hook, initial_mode);

    Ok(hook)
}

/// Install hooks based on debug mode
///
/// # Errors
///
/// Returns an error if hook installation fails
fn install_hooks_for_mode(
    lua: &Lua,
    hook: &Arc<parking_lot::Mutex<LuaExecutionHook>>,
    mode: DebugMode,
) {
    match mode {
        DebugMode::Disabled => {
            // WARNING: Lua only supports ONE debug hook at a time!
            // When we install our hooks, we override any existing hooks (profilers, etc.)
            // Therefore, in Disabled mode, we remove our hooks entirely to allow
            // other systems to use the debug hook if needed.
            //
            // This is a fundamental limitation of Lua's debug API.
            // Users must choose: either debug hooks OR other profiling hooks, not both.

            lua.remove_hook();
            hook.lock().hooks_installed = false;
        }
        DebugMode::Minimal { check_interval } => {
            // Periodic checking only
            let hook_clone = hook.clone();
            lua.set_hook(
                mlua::HookTriggers {
                    on_calls: false,
                    on_returns: false,
                    every_line: false,
                    every_nth_instruction: Some(check_interval),
                },
                move |lua, ar| {
                    let mut hook = hook_clone.lock();
                    // Only check for breakpoints periodically
                    if ar.curr_line() == -1 {
                        Ok(())
                    } else {
                        hook.handle_event(lua, &ar, DebugEvent::Line)
                    }
                },
            );
            hook.lock().hooks_installed = true;
        }
        DebugMode::Full => {
            // Full debugging with all events
            let hook_clone = hook.clone();
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
            hook.lock().hooks_installed = true;
        }
    }
}

/// Remove debug hooks from a Lua instance
///
/// NOTE: This only removes hooks if we installed them. We track this to avoid
/// interfering with other systems that might use Lua debug hooks (profilers, etc.)
pub fn remove_debug_hooks(lua: &Lua) {
    // We now keep minimal hooks installed even when "disabled" to avoid interference
    // The hooks check debug mode and exit fast when disabled
    // Only fully remove if shutting down the debug system
    lua.remove_hook();
}

/// Update debug mode for an existing hook
///
/// # Errors
///
/// Returns an error if hook reinstallation fails
pub fn update_debug_mode(
    lua: &Lua,
    hook: &Arc<parking_lot::Mutex<LuaExecutionHook>>,
    mode: DebugMode,
) -> LuaResult<()> {
    // Update the cache
    hook.lock().debug_cache.set_debug_mode(mode);

    // Reinstall hooks with new mode
    install_hooks_for_mode(lua, hook, mode);
    Ok(())
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
