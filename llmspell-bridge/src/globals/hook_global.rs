//! ABOUTME: Hook global object providing cross-language hook system
//! ABOUTME: Full implementation integrating with llmspell-hooks infrastructure

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::hook_bridge::HookBridge;
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Hook global object providing cross-language hook system
pub struct HookGlobal {
    hook_bridge: Arc<HookBridge>,
}

impl HookGlobal {
    /// Create a new Hook global with the bridge
    pub fn new(hook_bridge: Arc<HookBridge>) -> Self {
        Self { hook_bridge }
    }
}

impl GlobalObject for HookGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Hook".to_string(),
            version: "1.0.0".to_string(),
            description: "Cross-language hook system for lifecycle events and customization"
                .to_string(),
            dependencies: vec![],
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        // Store the hook bridge in the global context
        context.set_bridge("hook_bridge", self.hook_bridge.clone());

        // Inject the Lua-specific hook global
        crate::lua::globals::hook::inject_hook_global(lua, context, self.hook_bridge.clone())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // Store the hook bridge in the global context
        context.set_bridge("hook_bridge", self.hook_bridge.clone());

        // Inject the JavaScript-specific hook global
        crate::javascript::globals::hook::inject_hook_global(ctx, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ComponentRegistry, ProviderManager};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_hook_global_metadata() {
        let context = Arc::new(GlobalContext::new(
            Arc::new(ComponentRegistry::new()),
            Arc::new(ProviderManager::new(Default::default()).await.unwrap()),
        ));
        let hook_bridge = Arc::new(HookBridge::new(context).await.unwrap());
        let global = HookGlobal::new(hook_bridge);
        let metadata = global.metadata();
        assert_eq!(metadata.name, "Hook");
        assert_eq!(metadata.version, "1.0.0");
    }
}
