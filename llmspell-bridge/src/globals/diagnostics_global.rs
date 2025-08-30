//! Diagnostics global object implementation for script engines
//!
//! Provides unified diagnostics infrastructure (logging, profiling, metrics) access across all script languages.

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::diagnostics_bridge::DiagnosticsBridge;
use llmspell_core::Result;
use std::sync::Arc;

/// Diagnostics global object for script engines
pub struct DiagnosticsGlobal {
    bridge: Arc<DiagnosticsBridge>,
}

impl DiagnosticsGlobal {
    /// Create a new Diagnostics global
    #[must_use]
    pub fn new() -> Self {
        Self {
            bridge: Arc::new(DiagnosticsBridge::new()),
        }
    }

    /// Get the diagnostics bridge
    #[must_use]
    pub const fn bridge(&self) -> &Arc<DiagnosticsBridge> {
        &self.bridge
    }
}

impl Default for DiagnosticsGlobal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalObject for DiagnosticsGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Console".to_string(),
            description: "Diagnostics infrastructure for logging, profiling, and troubleshooting"
                .to_string(),
            dependencies: vec![],
            required: false, // Debug is optional
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::diagnostics::inject_diagnostics_global(lua, context, &self.bridge)
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Console global: {e}"),
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
            message: "JavaScript Debug global not yet implemented".to_string(),
            source: None,
        })
    }
}
