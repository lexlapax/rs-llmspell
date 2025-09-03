//! Debug Hook Adapter (Layer 2: Shared/Adaptation)
//!
//! The `LuaDebugHookAdapter` bridges the gap between Layer 1 (language-agnostic
//! `DebugHook` trait) and Layer 3 (Lua-specific `HookHandler` implementations).
//!
//! Architecture:
//! ```
//! Layer 1: `DebugHook` trait (language-agnostic)
//!     ↓
//! Layer 2: `LuaDebugHookAdapter` (this file) - bridges traits
//!     ↓
//! Layer 3: `HookHandler` implementations (`LuaExecutionHook`, `LuaDebugBridge`)
//! ```
//!
//! This adapter:
//! - Implements `DebugHook` for engine integration
//! - Contains `HookMultiplexer` to manage Lua-specific handlers
//! - Provides `install_on_lua()` to connect to actual Lua runtime

use crate::debug_coordinator::DebugCoordinator;
use crate::debug_runtime::{DebugControl, DebugHook};
use crate::execution_bridge::ExecutionManager;
use crate::execution_context::SharedExecutionContext;
use crate::lua::globals::execution::LuaExecutionHook;
use crate::lua::hook_multiplexer::{HookMultiplexer, HookPriority};
use crate::lua::lua_debug_bridge::LuaDebugBridge;
use async_trait::async_trait;
use mlua::{Lua, Result as LuaResult};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Layer 2 Adapter that bridges Layer 1 (`DebugHook`) to Layer 3 (`HookHandler`)
#[derive(Clone)]
pub struct LuaDebugHookAdapter {
    /// Manages Layer 3 Lua-specific handlers
    multiplexer: Arc<HookMultiplexer>,
    /// Layer 3 component: Lua-specific breakpoint logic
    _lua_execution_hook: Arc<parking_lot::Mutex<LuaExecutionHook>>,
    /// Layer 3 component: Lua-specific debug coordination
    _lua_debug_bridge: Arc<parking_lot::Mutex<LuaDebugBridge>>,
}

impl LuaDebugHookAdapter {
    /// Create a new adapter that bridges Layer 1 to Layer 3
    ///
    /// # Panics
    ///
    /// Panics if the `LuaDebugBridge` handler cannot be registered with the multiplexer.
    /// This should not happen in normal operation.
    pub fn new(
        execution_manager: Arc<ExecutionManager>,
        coordinator: Arc<DebugCoordinator>,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
    ) -> Self {
        let multiplexer = Arc::new(HookMultiplexer::new());

        // Create Layer 3 component: LuaExecutionHook (has Lua-specific breakpoint logic)
        let lua_execution_hook = Arc::new(parking_lot::Mutex::new(LuaExecutionHook::new(
            execution_manager,
            shared_context,
        )));

        // Create Layer 3 component: LuaDebugBridge (Lua-specific coordination)
        // Note: LuaDebugBridge implements HookHandler and wraps LuaExecutionHook
        let lua_debug_bridge = Arc::new(parking_lot::Mutex::new(LuaDebugBridge::new(
            coordinator,
            lua_execution_hook.clone(),
        )));

        // Register Layer 3 handler with multiplexer
        // Only LuaDebugBridge implements HookHandler
        // We need to clone the inner value to register it
        {
            let bridge_clone = lua_debug_bridge.lock().clone();
            multiplexer
                .register_handler(
                    "debug_bridge".to_string(),
                    HookPriority::DEBUGGER, // Use the correct constant
                    Box::new(bridge_clone),
                )
                .expect("Failed to register LuaDebugBridge");
        }

        Self {
            multiplexer,
            _lua_execution_hook: lua_execution_hook,
            _lua_debug_bridge: lua_debug_bridge,
        }
    }

    /// Install the multiplexer on a Lua instance to connect Layer 2 to Layer 3
    ///
    /// This bridges the gap between the engine's `DebugHook` and Lua's `HookHandler` system
    ///
    /// # Errors
    ///
    /// Returns an error if the hook installation fails in the Lua runtime.
    pub fn install_on_lua(&self, lua: &Lua) -> LuaResult<()> {
        // HookMultiplexer will set up Lua hooks that call our registered handlers
        self.multiplexer.install(lua)
    }
}

// Layer 2 implements Layer 1 trait (`DebugHook`) for engine integration
#[async_trait]
impl DebugHook for LuaDebugHookAdapter {
    async fn on_line(&self, _line: u32, _source: &str) -> DebugControl {
        // Layer 2 doesn't directly handle events - the actual handling
        // happens through HookMultiplexer when install_on_lua() is called
        // The multiplexer sets up Lua hooks that get the full Lua context
        DebugControl::Continue
    }

    async fn on_function_enter(&self, _name: &str, _args: Vec<String>) -> DebugControl {
        // Actual handling happens through HookMultiplexer
        DebugControl::Continue
    }

    async fn on_function_exit(&self, _name: &str, _result: Option<String>) -> DebugControl {
        // Actual handling happens through HookMultiplexer
        DebugControl::Continue
    }

    async fn on_exception(&self, _error: &str, _line: u32) -> DebugControl {
        // Actual handling happens through HookMultiplexer
        DebugControl::Continue
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
