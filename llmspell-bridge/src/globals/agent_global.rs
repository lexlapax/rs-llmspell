//! ABOUTME: Agent global object implementation for script engines
//! ABOUTME: Provides Agent creation and management functionality

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::agent_bridge::AgentBridge;
use crate::ComponentRegistry;
use crate::ProviderManager;
use llmspell_core::Result;
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
    pub async fn new(
        registry: Arc<ComponentRegistry>,
        providers: Arc<ProviderManager>,
    ) -> Result<Self> {
        // Create a core provider manager for the agent bridge
        let core_providers = providers.create_core_manager_arc().await?;
        let bridge = Arc::new(AgentBridge::new(registry.clone(), core_providers));
        Ok(Self {
            registry,
            providers,
            bridge,
        })
    }

    /// Create with state manager support
    pub async fn with_state_manager(
        registry: Arc<ComponentRegistry>,
        providers: Arc<ProviderManager>,
        state_manager: Arc<llmspell_state_persistence::StateManager>,
    ) -> Result<Self> {
        // Create a core provider manager for the agent bridge
        let core_providers = providers.create_core_manager_arc().await?;
        let mut bridge = AgentBridge::new(registry.clone(), core_providers);
        bridge.set_state_manager(state_manager);
        Ok(Self {
            registry,
            providers,
            bridge: Arc::new(bridge),
        })
    }

    /// Get the agent bridge
    #[must_use]
    pub const fn bridge(&self) -> &Arc<AgentBridge> {
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
                message: format!("Failed to inject Agent global: {e}"),
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
        crate::javascript::globals::agent::inject_agent_global(ctx, context).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to inject Agent global for JavaScript: {}", e),
                source: None,
            }
        })
    }
}
