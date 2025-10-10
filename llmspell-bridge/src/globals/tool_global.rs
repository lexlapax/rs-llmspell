//! ABOUTME: Tool global object implementation for script engines
//! ABOUTME: Provides Tool discovery and invocation functionality

#[cfg(any(feature = "lua", feature = "javascript"))]
use super::types::GlobalContext;
use super::types::{GlobalMetadata, GlobalObject};
use crate::ComponentRegistry;
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::Result;
use std::sync::Arc;

/// Tool global object for script engines
pub struct ToolGlobal {
    registry: Arc<ComponentRegistry>,
}

impl ToolGlobal {
    /// Create a new Tool global
    #[must_use]
    pub const fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self { registry }
    }

    /// Get the component registry
    #[must_use]
    pub const fn registry(&self) -> &Arc<ComponentRegistry> {
        &self.registry
    }
}

impl GlobalObject for ToolGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Tool".to_string(),
            description: "Tool discovery and invocation".to_string(),
            dependencies: vec![],
            required: true,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::tool::inject_tool_global(lua, context, self.registry.clone()).map_err(
            |e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Tool global: {e}"),
                source: None,
            },
        )
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::tool::inject_tool_global(ctx, context).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Tool global for JavaScript: {e}"),
                source: None,
            }
        })
    }
}
