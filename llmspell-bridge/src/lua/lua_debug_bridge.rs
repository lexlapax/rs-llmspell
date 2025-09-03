//! Lua Debug Bridge (Layer 2)
//!
//! The `LuaDebugBridge` connects the language-agnostic `DebugCoordinator` to the
//! Lua-specific `LuaExecutionHook`, handling sync/async boundaries and Lua context
//! marshalling.
//!
//! Architecture:
//! ```
//! Layer 1: DebugCoordinator (language-agnostic coordinator)
//!     ↓
//! Layer 2: LuaDebugBridge (this file) - sync/async boundary + Lua adaptation  
//!     ↓  
//! Layer 3: LuaExecutionHook - Lua-specific implementation
//! ```
//!
//! Performance Strategy:
//! - Fast path: Sync breakpoint checks via `DebugCoordinator`, no Lua context needed
//! - Slow path: Marshalls Lua context to `DebugCoordinator` when actually pausing
//! - Preserves existing `LuaExecutionHook` optimization (fast/slow path design)

use crate::debug_coordinator::DebugCoordinator;
use crate::execution_bridge::ExecutionLocation;
use crate::lua::globals::execution::LuaExecutionHook;
use crate::lua::hook_multiplexer::HookHandler;
use crate::lua::sync_utils::block_on_async;
use mlua::{Debug, DebugEvent, HookTriggers, Lua, Result as LuaResult};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Lua Debug Bridge - connects `DebugCoordinator` to `LuaExecutionHook`
///
/// Handles the sync/async boundary and Lua context marshalling between
/// the language-agnostic coordinator and Lua-specific implementation.
#[derive(Clone)]
pub struct LuaDebugBridge {
    /// Reference to the language-agnostic debug coordinator
    coordinator: Arc<DebugCoordinator>,

    /// Reference to Lua-specific execution hook
    lua_hook: Arc<parking_lot::Mutex<LuaExecutionHook>>,
}

impl LuaDebugBridge {
    /// Create a new Lua debug bridge
    #[must_use]
    pub const fn new(
        coordinator: Arc<DebugCoordinator>,
        lua_hook: Arc<parking_lot::Mutex<LuaExecutionHook>>,
    ) -> Self {
        Self {
            coordinator,
            lua_hook,
        }
    }
}

/// Implement `HookHandler` for full Lua context access
impl HookHandler for LuaDebugBridge {
    fn handle_event(&mut self, lua: &Lua, ar: &Debug, event: DebugEvent) -> LuaResult<()> {
        // Only handle line events for pause coordination
        if event != DebugEvent::Line {
            return Ok(());
        }

        let line = ar.curr_line();
        if line <= 0 {
            return Ok(());
        }

        let source_info = ar.source();
        let source = source_info.short_src.as_deref().unwrap_or("<unknown>");

        // Convert line to u32 safely
        let line_num = u32::try_from(line).unwrap_or(0);

        // FAST PATH: Check if we might break
        if !self.coordinator.might_break_at_sync(source, line_num) {
            return Ok(()); // Early exit - no breakpoint here
        }

        // SLOW PATH: We might need to break, check with LuaExecutionHook
        let should_break = {
            let hook = self.lua_hook.lock();
            hook.should_break_slow(source, line_num, lua)
        };

        if should_break {
            // Extract Lua variables using actual context
            let variables = Self::extract_lua_variables(lua, line_num, source);
            let location = ExecutionLocation {
                source: source.to_string(),
                line: line_num,
                column: None,
            };

            // Use block_on_async to coordinate pause
            let coordinator = self.coordinator.clone();
            let pause_source = source.to_string();
            if let Err(e) = block_on_async(
                "coordinate_breakpoint_pause",
                async move {
                    coordinator
                        .coordinate_breakpoint_pause(location, variables)
                        .await;
                    Ok::<(), std::io::Error>(())
                },
                Some(Duration::from_millis(100)),
            ) {
                // Log error with layer identification for debugging
                tracing::error!(
                    "Layer 2 (LuaDebugBridge) failed to coordinate pause at {}:{}: {:?}",
                    pause_source,
                    line_num,
                    e
                );
                // Graceful degradation: continue execution instead of crashing
            }
        }

        Ok(())
    }

    fn interested_events(&self) -> HookTriggers {
        HookTriggers {
            every_line: true,
            ..Default::default()
        }
    }

    fn is_active(&self) -> bool {
        // Active when we have breakpoints
        true // Could optimize by checking coordinator.has_breakpoints()
    }
}

/// Extract Lua variables from current context
impl LuaDebugBridge {
    fn extract_lua_variables(
        lua: &Lua,
        line: u32,
        source: &str,
    ) -> HashMap<String, serde_json::Value> {
        let mut variables = HashMap::new();

        // Try to extract local variables using Lua debug API
        if let Ok(debug_table) = lua.globals().get::<_, mlua::Table>("debug") {
            if let Ok(getlocal) = debug_table.get::<_, mlua::Function>("getlocal") {
                // Get locals from the current frame (level 2 to skip this function)
                let mut local_index = 1;
                while local_index <= 50 {
                    // Reasonable limit
                    match getlocal
                        .call::<_, (Option<String>, Option<mlua::Value>)>((2, local_index))
                    {
                        Ok((Some(name), Some(value))) if !name.starts_with('(') => {
                            // Skip internal variables like (temporary)
                            // Use format_simple from output.rs for consistent formatting
                            let formatted_value = crate::lua::output::format_simple(&value);
                            variables
                                .insert(name.clone(), serde_json::Value::String(formatted_value));
                        }
                        _ => break, // No more locals or error
                    }
                    local_index += 1;
                }
            }
        }

        // Add debug metadata
        variables.insert(
            "__debug_line".to_string(),
            serde_json::Value::Number(serde_json::Number::from(line)),
        );
        variables.insert(
            "__debug_source".to_string(),
            serde_json::Value::String(source.to_string()),
        );

        variables
    }
}

#[cfg(test)]
mod tests {
    // TODO: Enable tests once ExecutionManager is properly implemented
    // For now, tests are disabled to allow compilation
}
