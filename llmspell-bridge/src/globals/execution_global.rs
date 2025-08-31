//! Execution debugging global object implementation for script engines
//!
//! Provides unified execution debugging infrastructure (breakpoints, stepping, inspection)
//! access across all script languages. This is distinct from diagnostics (logging/profiling).

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::debug_state_cache::DebugStateCache;
use crate::execution_bridge::ExecutionManager;
use llmspell_core::Result;
use std::sync::Arc;

/// Execution debugging global object for script engines
pub struct ExecutionGlobal {
    manager: Arc<ExecutionManager>,
}

impl ExecutionGlobal {
    /// Create a new Execution global with the specified debug cache
    #[must_use]
    pub fn new(debug_cache: Arc<dyn DebugStateCache>) -> Self {
        Self {
            manager: Arc::new(ExecutionManager::new(debug_cache)),
        }
    }

    /// Get the execution manager
    #[must_use]
    pub const fn manager(&self) -> &Arc<ExecutionManager> {
        &self.manager
    }
}

impl GlobalObject for ExecutionGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Debugger".to_string(),
            description:
                "Execution debugging infrastructure for breakpoints, stepping, and inspection"
                    .to_string(),
            dependencies: vec![],
            required: false, // Debugging is optional
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::execution::inject_execution_global(lua, context, &self.manager)
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Debugger global: {e}"),
                source: None,
            })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // JavaScript implementation will be added in Phase 5
        Err(llmspell_core::LLMSpellError::Component {
            message: "JavaScript Debugger global not yet implemented".to_string(),
            source: None,
        })
    }
}
