//! Debug global object implementation for script engines
//!
//! Provides unified debug infrastructure access across all script languages.

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::debug_bridge::DebugBridge;
use llmspell_core::Result;
use std::sync::Arc;

/// Debug global object for script engines
pub struct DebugGlobal {
    bridge: Arc<DebugBridge>,
}

impl DebugGlobal {
    /// Create a new Debug global
    #[must_use]
    pub fn new() -> Self {
        Self {
            bridge: Arc::new(DebugBridge::new()),
        }
    }

    /// Get the debug bridge
    #[must_use]
    pub fn bridge(&self) -> &Arc<DebugBridge> {
        &self.bridge
    }
}

impl Default for DebugGlobal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalObject for DebugGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Debug".to_string(),
            description: "Debug infrastructure for logging, profiling, and troubleshooting"
                .to_string(),
            dependencies: vec![],
            required: false, // Debug is optional
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::debug::inject_debug_global(lua, context, self.bridge.clone()).map_err(
            |e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Debug global: {e}"),
                source: None,
            },
        )
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // JavaScript implementation will be added in Phase 5
        Err(llmspell_core::LLMSpellError::Component {
            message: "JavaScript Debug global not yet implemented".to_string(),
            source: None,
        })
    }
}
