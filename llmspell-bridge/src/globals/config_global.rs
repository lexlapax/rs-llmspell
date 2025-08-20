//! ABOUTME: Config global object implementation for script engines
//! ABOUTME: Provides configuration access and modification capabilities

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use crate::config_bridge::{ConfigBridge, ConfigPermissions};
use llmspell_config::LLMSpellConfig;
use llmspell_core::Result;
use std::sync::Arc;

/// Config bridge global object for script engines
pub struct ConfigBridgeGlobal {
    bridge: Arc<ConfigBridge>,
}

impl ConfigBridgeGlobal {
    /// Create a new Config global with specified permissions
    #[must_use]
    pub fn new(config: LLMSpellConfig, permissions: ConfigPermissions) -> Self {
        Self {
            bridge: Arc::new(ConfigBridge::new(config, permissions)),
        }
    }

    /// Create a Config global for a specific script
    #[must_use]
    pub fn for_script(
        config: LLMSpellConfig,
        script_id: String,
        permissions: ConfigPermissions,
    ) -> Self {
        Self {
            bridge: Arc::new(ConfigBridge::for_script(config, script_id, permissions)),
        }
    }

    /// Get the config bridge
    #[must_use]
    pub const fn bridge(&self) -> &Arc<ConfigBridge> {
        &self.bridge
    }
}

impl GlobalObject for ConfigBridgeGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Config".to_string(),
            description: "Configuration access and modification".to_string(),
            dependencies: vec![],
            required: false, // Config is optional
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, _context: &GlobalContext) -> Result<()> {
        crate::lua::globals::config::inject_config_global(lua, &self.bridge).map_err(|e| {
            llmspell_core::LLMSpellError::Configuration {
                message: format!("Failed to inject Config global: {e}"),
                source: None,
            }
        })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        crate::javascript::globals::config::inject_config_global(ctx, self.bridge.clone()).map_err(
            |e| llmspell_core::LLMSpellError::Configuration {
                message: format!("Failed to inject Config global for JavaScript: {e}"),
                source: None,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_global_creation() {
        let config = LLMSpellConfig::default();
        let permissions = ConfigPermissions::read_only();
        let global = ConfigBridgeGlobal::new(config, permissions);

        let metadata = global.metadata();
        assert_eq!(metadata.name, "Config");
        assert!(!metadata.required);
    }

    #[test]
    fn test_config_global_for_script() {
        let config = LLMSpellConfig::default();
        let permissions = ConfigPermissions::standard();
        let global = ConfigBridgeGlobal::for_script(config, "test-script".to_string(), permissions);

        assert_eq!(global.bridge().permissions().read, true);
        assert_eq!(global.bridge().permissions().modify_providers, true);
        assert_eq!(global.bridge().permissions().modify_security, false);
    }
}
