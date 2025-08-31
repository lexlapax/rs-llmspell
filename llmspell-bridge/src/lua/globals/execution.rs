//! Lua-specific Execution debugging global implementation (exposed as Debugger)
//!
//! Provides execution debugging capabilities for Lua scripts including breakpoints,
//! variable inspection, stepping, and execution control. This is distinct from
//! diagnostics (logging/profiling) which is handled by the Console global.

use crate::condition_evaluator::{ConditionEvaluator, SharedDebugContext};
use crate::debug_state_cache::{DebugMode, DebugStateCache};
use crate::execution_bridge::{
    Breakpoint, DebugState, ExecutionLocation, ExecutionManager, PauseReason, StackFrame,
};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use crate::lua::condition_evaluator_impl::LuaConditionEvaluator;
use crate::lua::debug_state_cache_impl::LuaDebugStateCache;
use crate::lua::output::{capture_stack_trace, StackTraceOptions};
use crate::lua::sync_utils::block_on_async;
use crate::lua::variable_inspector_impl::LuaVariableInspector;
use crate::variable_inspector::VariableInspector;
use crate::variable_inspector::{ContextBatcher, ContextUpdate};
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
    debug_cache: Arc<LuaDebugStateCache>,
    /// Context update batcher for lazy updates
    context_batcher: ContextBatcher,
    /// Variable inspector for slow path variable operations
    variable_inspector: Arc<LuaVariableInspector>,
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
        let debug_cache = Arc::new(LuaDebugStateCache::new());
        let variable_inspector = Arc::new(LuaVariableInspector::new(
            debug_cache.clone(),
            shared_context.clone(),
        ));

        Self {
            execution_manager,
            shared_context,
            debug_cache,
            context_batcher: ContextBatcher::new(),
            variable_inspector,
            current_line: 0,
            current_source: String::new(),
            line_counter: 0,
            hooks_installed: false,
        }
    }

    /// Update breakpoints from debug manager
    pub fn update_breakpoints(&mut self, breakpoints: &[Breakpoint], _lua: &Lua) {
        // Update the fast cache with breakpoint locations
        let locations: Vec<(String, u32)> = breakpoints
            .iter()
            .filter(|bp| bp.enabled)
            .map(|bp| (bp.source.clone(), bp.line))
            .collect();

        self.debug_cache.update_breakpoints(locations.clone());

        // Compile and cache conditions for breakpoints that have them
        for bp in breakpoints
            .iter()
            .filter(|bp| bp.enabled && bp.condition.is_some())
        {
            if let Some(ref condition_expr) = bp.condition {
                let evaluator = LuaConditionEvaluator::new();
                match evaluator.compile_condition(condition_expr) {
                    Ok(compiled) => {
                        self.debug_cache
                            .set_condition(bp.source.clone(), bp.line, compiled);
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to compile condition for breakpoint at {}:{}: {}",
                            bp.source,
                            bp.line,
                            e
                        );
                    }
                }
            }
        }

        // Remove conditions for disabled or removed breakpoints
        let _active_locations: std::collections::HashSet<(String, u32)> =
            locations.into_iter().collect();
        // Note: We'd need to track previous breakpoints to remove stale conditions
        // For now, conditions are cleared when breakpoints are updated
    }

    /// Check if we should break at current location (SLOW PATH - only called when `might_break_at` returns true)
    fn should_break_slow(&self, source: &str, line: u32, lua: &Lua) -> bool {
        // This is the slow path - only called when we might have a breakpoint
        let exec_mgr = self.execution_manager.clone();
        let source_str = source.to_string();
        let line_num = line;

        // First check basic breakpoint conditions (hit count, etc.)
        let breakpoint = block_on_async(
            "get_breakpoint",
            async move {
                Ok::<_, std::io::Error>(exec_mgr.get_breakpoint_at(&source_str, line_num).await)
            },
            None,
        )
        .ok()
        .flatten();

        let Some(mut bp) = breakpoint else {
            return false;
        };

        // Update hit count
        bp.current_hits += 1;

        // Check hit count condition
        if !bp.should_break() {
            // Update the stored breakpoint with new hit count
            let exec_mgr = self.execution_manager.clone();
            block_on_async(
                "update_breakpoint_hits",
                async move {
                    exec_mgr
                        .update_breakpoint_hits(&bp.id, bp.current_hits)
                        .await;
                    Ok::<_, std::io::Error>(())
                },
                None,
            )
            .ok();
            return false;
        }

        // Check if there's a condition to evaluate
        if self.debug_cache.has_condition(source, line) {
            // Evaluate condition in slow path
            let evaluator = LuaConditionEvaluator::new();
            let debug_context = SharedDebugContext::new(self.shared_context.clone());
            let result = bp.condition.as_ref().is_none_or(|condition| {
                evaluator
                    .evaluate_condition_with_lua(condition, None, &debug_context, lua)
                    .unwrap_or(true)
            });

            if result {
                // Update the stored breakpoint
                let exec_mgr = self.execution_manager.clone();
                let bp_clone = bp;
                block_on_async(
                    "update_breakpoint_final",
                    async move {
                        exec_mgr
                            .update_breakpoint_hits(&bp_clone.id, bp_clone.current_hits)
                            .await;
                        Ok::<_, std::io::Error>(())
                    },
                    None,
                )
                .ok();
            }

            result
        } else {
            // No condition, break normally
            let exec_mgr = self.execution_manager.clone();
            let bp_clone = bp;
            block_on_async(
                "update_breakpoint_final",
                async move {
                    exec_mgr
                        .update_breakpoint_hits(&bp_clone.id, bp_clone.current_hits)
                        .await;
                    Ok::<_, std::io::Error>(())
                },
                None,
            )
            .ok();
            true
        }
    }

    /// Get debug cache for external use
    #[must_use]
    pub fn debug_cache(&self) -> Arc<LuaDebugStateCache> {
        self.debug_cache.clone()
    }

    /// Flush batched context updates to `SharedExecutionContext`
    pub fn flush_batched_context_updates(&mut self) {
        let updates = self.context_batcher.flush();
        if updates.is_empty() {
            return;
        }

        // Process variable-related updates through the inspector
        self.variable_inspector
            .process_context_updates(updates.clone());

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
                        ContextUpdate::ReadVariables(_)
                        | ContextUpdate::CacheVariable { .. }
                        | ContextUpdate::WatchVariable(_)
                        | ContextUpdate::UnwatchVariable(_) => {
                            // Variables are handled by the inspector
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

                // FAST PATH - synchronous checks, no async operations
                // Check stepping first (atomic flag), then breakpoints
                if !self.debug_cache.is_stepping()
                    && !self.debug_cache.might_break_at(&source, line)
                {
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

                // SLOW PATH - handle stepping or breakpoints
                // Check if we're stepping first
                if self.debug_cache.is_stepping() {
                    self.handle_step_slow_path(lua, ar, &source, line);
                    return Ok(());
                }

                // Otherwise check for breakpoints
                // Now we can afford async operations since we're potentially breaking anyway
                if self.should_break_slow(&source, line, lua) {
                    // Flush any pending context updates before breaking
                    self.flush_batched_context_updates();

                    // Pause execution with enriched context
                    self.pause_at_breakpoint_with_context(lua, source, line);
                }
            }
            DebugEvent::Call | DebugEvent::TailCall => {
                // Function call - track depth for stepping
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

                // Update depth for step operations
                if self.debug_cache.is_stepping() {
                    let new_depth = self.debug_cache.get_current_depth() + 1;
                    self.debug_cache.set_current_depth(new_depth);
                }

                // Only use async if we're in full debug mode
                if matches!(self.debug_cache.get_debug_mode(), DebugMode::Full) {
                    // Batch the stack push
                    self.context_batcher.record_stack_push(name, source, line);
                }
            }
            DebugEvent::Ret => {
                // Function return - track depth for stepping
                if self.debug_cache.is_stepping() {
                    let new_depth = self.debug_cache.get_current_depth() - 1;
                    self.debug_cache.set_current_depth(new_depth);
                }

                // Batch the stack pop update
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

    /// Handle step execution in slow path
    fn handle_step_slow_path(&mut self, lua: &Lua, _ar: &mlua::Debug, source: &str, line: u32) {
        use crate::debug_state_cache::StepMode;

        let current_depth = self.debug_cache.get_current_depth();
        let step_mode = self.debug_cache.get_step_mode();

        let should_pause = match step_mode {
            StepMode::None => false, // Shouldn't happen
            StepMode::StepIn { .. } => {
                // Step into - pause on every line
                true
            }
            StepMode::StepOver { target_depth } => {
                // Step over - pause when at or below target depth
                current_depth <= target_depth
            }
            StepMode::StepOut { target_depth } => {
                // Step out - pause when we've returned to target depth
                current_depth <= target_depth
            }
        };

        if should_pause {
            // Flush context updates
            self.flush_batched_context_updates();

            // Complete the step
            self.execution_manager.complete_step();

            // Pause with step reason
            self.pause_for_step(lua, source.to_string(), line);
        }
    }

    /// Pause execution for a step operation
    fn pause_for_step(&self, lua: &Lua, source: String, line: u32) {
        // Similar to pause_at_breakpoint_with_context but with Step reason
        let location = ExecutionLocation {
            source: source.clone(),
            line,
            column: None,
        };

        // Capture stack trace
        let stack_options = StackTraceOptions {
            max_depth: 20,
            capture_locals: true,
            capture_upvalues: false,
            include_source: true,
        };
        let stack_trace = capture_stack_trace(lua, &stack_options);
        let stack_frames = stack_trace.frames.clone();

        // Extract variables
        let mut variables = HashMap::new();
        if let Some(first_frame) = stack_trace.frames.first() {
            for local in &first_frame.locals {
                let json_value = Self::lua_value_to_json(&local.value);
                variables.insert(local.name.clone(), json_value);
            }
        }

        // Update context and set paused state
        let shared_ctx = self.shared_context.clone();

        // Evaluate watch expressions when paused (slow path only - Task 9.2.8)
        let evaluator = LuaConditionEvaluator::new();
        let debug_context = SharedDebugContext::new(shared_ctx.clone());
        let _watch_results =
            self.debug_cache
                .evaluate_watches_with_lua(lua, &debug_context, &evaluator);

        let exec_mgr = self.execution_manager.clone();

        block_on_async::<_, (), std::io::Error>(
            "pause_for_step",
            async move {
                let mut ctx = shared_ctx.write().await;
                ctx.stack.clone_from(&stack_frames);
                ctx.variables = variables;
                ctx.set_location(SourceLocation {
                    source: source.clone(),
                    line,
                    column: None,
                });
                drop(ctx);

                // Set paused state with Step reason
                exec_mgr
                    .set_state(DebugState::Paused {
                        reason: PauseReason::Step,
                        location,
                    })
                    .await;

                Ok(())
            },
            None,
        )
        .ok();
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
        // Invalidate condition cache when variables change
        self.debug_cache.invalidate_condition_cache();

        // Evaluate watch expressions when paused (slow path only - Task 9.2.8)
        let evaluator = LuaConditionEvaluator::new();
        let debug_context = SharedDebugContext::new(shared_ctx.clone());
        let _watch_results =
            self.debug_cache
                .evaluate_watches_with_lua(lua, &debug_context, &evaluator);

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
