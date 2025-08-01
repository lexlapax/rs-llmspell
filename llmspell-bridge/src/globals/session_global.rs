//! ABOUTME: Session global object providing session management for scripts
//! ABOUTME: Integrates with SessionManager via SessionBridge for language-specific bindings

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::session_bridge::SessionBridge;
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Session global object providing session management for scripts
///
/// This wraps SessionBridge and provides language-specific bindings,
/// converting between async Rust operations and synchronous script calls.
pub struct SessionGlobal {
    /// Session bridge for core operations
    pub session_bridge: Arc<SessionBridge>,
}

impl SessionGlobal {
    /// Create a new Session global
    pub fn new(session_bridge: Arc<SessionBridge>) -> Self {
        Self { session_bridge }
    }
}

impl GlobalObject for SessionGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Session".to_string(),
            version: "1.0.0".to_string(),
            description: "Session management system with persistence and replay".to_string(),
            dependencies: vec!["State".to_string()], // Sessions use state persistence
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        crate::lua::globals::session::inject_session_global(
            lua,
            context,
            self.session_bridge.clone(),
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to inject Session global: {}", e),
            source: None,
        })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO: Implement JavaScript bindings for Session global
        Ok(())
    }
}
