//! ABOUTME: Agent global object implementation for script engines
//! ABOUTME: Provides Agent creation and management functionality

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::agent_bridge::AgentBridge;
use crate::ComponentRegistry;
use llmspell_core::Result;
use llmspell_providers::ProviderManager;
use std::sync::Arc;

/// Agent global object for script engines
pub struct AgentGlobal {
    #[allow(dead_code)]
    registry: Arc<ComponentRegistry>,
    #[allow(dead_code)]
    providers: Arc<ProviderManager>,
    bridge: Arc<AgentBridge>,
}

impl AgentGlobal {
    /// Create a new Agent global
    pub fn new(registry: Arc<ComponentRegistry>, providers: Arc<ProviderManager>) -> Self {
        let bridge = Arc::new(AgentBridge::new(registry.clone()));
        Self {
            registry,
            providers,
            bridge,
        }
    }

    /// Get the agent bridge
    pub fn bridge(&self) -> &Arc<AgentBridge> {
        &self.bridge
    }
}

impl GlobalObject for AgentGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Agent".to_string(),
            description: "Agent creation and management".to_string(),
            dependencies: vec![],
            required: true,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::agent::inject_agent_global(lua, context, self.bridge.clone()).map_err(
            |e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Agent global: {}", e),
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
        // TODO: Implement JavaScript injection
        Ok(())
    }
}
