//! ABOUTME: Provider global object implementation for script engines
//! ABOUTME: Provides access to LLM provider information and capabilities

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::ProviderManager;
use llmspell_core::Result;
use std::sync::Arc;

/// Provider global object for script engines
pub struct ProviderGlobal {
    providers: Arc<ProviderManager>,
}

impl ProviderGlobal {
    /// Create a new Provider global with access to provider manager
    #[must_use]
    pub fn new(providers: Arc<ProviderManager>) -> Self {
        Self { providers }
    }
}

impl GlobalObject for ProviderGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Provider".to_string(),
            description: "LLM provider information and capability detection".to_string(),
            dependencies: vec![],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::provider::inject_provider_global(lua, context, &self.providers)
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Provider global: {e}"),
                source: None,
            })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // TODO: Implement JavaScript provider support in Phase 2
        // For now, return a stub implementation
        crate::javascript::globals::provider::inject_provider_global_stub()
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Provider global (JavaScript): {e}"),
                source: None,
            })
    }

    #[cfg(feature = "python")]
    fn inject_python(&self, _py: &pyo3::Python, _context: &GlobalContext) -> Result<()> {
        // TODO: Implement Python provider support in Phase 3
        Ok(())
    }
}