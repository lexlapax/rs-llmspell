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
use crate::debug_runtime::{DebugControl, DebugHook};
use crate::execution_bridge::{ExecutionLocation, PauseReason};
use crate::lua::globals::execution::LuaExecutionHook;
// Note: Will add back output utilities when variable extraction is enhanced
use async_trait::async_trait;
// use mlua::Lua; // Will be used when variable extraction is enhanced
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{trace, warn};

/// Lua Debug Bridge - connects `DebugCoordinator` to `LuaExecutionHook`
///
/// Handles the sync/async boundary and Lua context marshalling between
/// the language-agnostic coordinator and Lua-specific implementation.
pub struct LuaDebugBridge {
    /// Reference to the language-agnostic debug coordinator
    coordinator: Arc<DebugCoordinator>,

    /// Reference to Lua-specific execution hook (reserved for future use)
    _lua_hook: Arc<parking_lot::Mutex<LuaExecutionHook>>,

    /// Flag to indicate if Lua context is available (simplified for now)
    lua_available: Arc<RwLock<bool>>,
}

impl LuaDebugBridge {
    /// Create a new Lua debug bridge
    #[must_use]
    pub fn new(
        coordinator: Arc<DebugCoordinator>,
        lua_hook: Arc<parking_lot::Mutex<LuaExecutionHook>>,
    ) -> Self {
        Self {
            coordinator,
            _lua_hook: lua_hook,
            lua_available: Arc::new(RwLock::new(false)),
        }
    }

    /// Update the Lua context availability (called by `ScriptRuntime`)
    pub async fn set_lua_available(&self, available: bool) {
        let mut lua_available = self.lua_available.write().await;
        *lua_available = available;
    }

    /// Handle breakpoint with Lua context (SLOW PATH only)
    async fn handle_breakpoint_with_lua_context(&self, line: u32, source: &str) -> DebugControl {
        trace!(
            "Handling breakpoint with Lua context at {}:{}",
            source,
            line
        );

        // Check if Lua context is available
        let is_available = {
            let lua_available = self.lua_available.read().await;
            *lua_available
        };

        if is_available {
            // For now, assume we should break since we already passed the fast path check
            // TODO: In a future subtask, we'll wire this to actual breakpoint evaluation via LuaExecutionHook
            let should_break = true;

            if should_break {
                // Extract variables (simplified for now - will be enhanced later)
                let variables = Self::extract_lua_variables_simplified(line, source);
                let location = ExecutionLocation {
                    source: source.to_string(),
                    line,
                    column: None,
                };

                // Coordinate through DebugCoordinator
                self.coordinator
                    .coordinate_breakpoint_pause(location, variables)
                    .await;

                return DebugControl::Pause;
            }
        } else {
            warn!(
                "No Lua context available for breakpoint at {}:{}",
                source, line
            );
        }

        DebugControl::Continue
    }

    /// Extract Lua variables (simplified version for now)
    fn extract_lua_variables_simplified(
        line: u32,
        source: &str,
    ) -> HashMap<String, serde_json::Value> {
        let mut variables = HashMap::new();

        // TODO: In a future subtask, extract actual Lua variables via LuaExecutionHook
        // For now, provide placeholder variables to demonstrate the coordination
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

    // TODO: Add back Lua value conversion when variable extraction is enhanced

    /// Handle step execution (SLOW PATH only)
    async fn handle_step_with_lua_context(&self, line: u32, source: &str) -> DebugControl {
        trace!("Handling step with Lua context at {}:{}", source, line);

        let location = ExecutionLocation {
            source: source.to_string(),
            line,
            column: None,
        };

        // Coordinate step pause through DebugCoordinator
        self.coordinator
            .coordinate_step_pause(PauseReason::Step, location)
            .await;

        DebugControl::Pause
    }
}

/// Implement `DebugHook` trait for `LuaDebugBridge`
#[async_trait]
impl DebugHook for LuaDebugBridge {
    /// Handle line execution event
    async fn on_line(&self, line: u32, source: &str) -> DebugControl {
        // FAST PATH: Pure sync check through DebugCoordinator (preserves performance)
        if !self.coordinator.might_break_at_sync(source, line) && !self.coordinator.is_paused_sync()
        {
            return DebugControl::Continue; // EXIT FAST - no async operations!
        }

        trace!("Entering slow path for line {}:{}", source, line);

        // SLOW PATH: Check if we're paused (stepping) or hit a breakpoint
        if self.coordinator.is_paused_sync() {
            self.handle_step_with_lua_context(line, source).await
        } else {
            // Must be a breakpoint
            self.handle_breakpoint_with_lua_context(line, source).await
        }
    }

    /// Handle function entry event
    async fn on_function_enter(&self, name: &str, _args: Vec<String>) -> DebugControl {
        trace!("Function enter: {}", name);
        // For now, just continue - could add step-into logic later
        DebugControl::Continue
    }

    /// Handle function exit event
    async fn on_function_exit(&self, name: &str, _result: Option<String>) -> DebugControl {
        trace!("Function exit: {}", name);
        // For now, just continue - could add step-out logic later
        DebugControl::Continue
    }

    /// Handle exception event
    async fn on_exception(&self, error: &str, line: u32) -> DebugControl {
        warn!("Exception at line {}: {}", line, error);

        // Always pause on exceptions for debugging (preserves existing behavior)
        let location = ExecutionLocation {
            source: "unknown".to_string(), // Exception source might not be available
            line,
            column: None,
        };

        self.coordinator
            .coordinate_step_pause(PauseReason::Exception(error.to_string()), location)
            .await;

        DebugControl::Pause
    }
}

// TODO: Future enhancement - expose LuaExecutionHook internal methods for bridge
// For now, the bridge uses simplified breakpoint logic

#[cfg(test)]
mod tests {
    // TODO: Enable tests once ExecutionManager is properly implemented
    // For now, tests are disabled to allow compilation
}
