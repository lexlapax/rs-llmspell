//! ABOUTME: Context global object providing context assembly for scripts
//! ABOUTME: Integrates with context retrieval via `ContextBridge` for language-specific bindings

use crate::context_bridge::ContextBridge;
#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::globals::types::GlobalContext;
use crate::globals::types::{GlobalMetadata, GlobalObject};
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Context global object providing context assembly for scripts
///
/// This wraps `ContextBridge` and provides language-specific bindings,
/// converting between async Rust operations and synchronous script calls.
pub struct ContextGlobal {
    /// Context bridge for core operations
    pub context_bridge: Arc<ContextBridge>,
}

impl ContextGlobal {
    /// Create a new Context global
    #[must_use]
    pub const fn new(context_bridge: Arc<ContextBridge>) -> Self {
        Self { context_bridge }
    }
}

impl GlobalObject for ContextGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Context".to_string(),
            version: "1.0.0".to_string(),
            description: "Context assembly and retrieval with BM25 ranking".to_string(),
            dependencies: vec!["Memory".to_string()], // Requires Memory for retrieval
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        crate::lua::globals::context::inject_context_global(lua, context, &self.context_bridge)
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to inject Context global: {e}"),
                source: None,
            })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO: Implement JavaScript bindings for Context global
        Ok(())
    }
}
