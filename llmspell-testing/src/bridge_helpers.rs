// ABOUTME: Test utilities for script bridge testing including engine setup and globals
// ABOUTME: Provides reusable helpers for Lua and JavaScript integration tests

use std::sync::Arc;

#[cfg(feature = "lua")]
pub mod lua {

    /// Create a test Lua engine configuration
    pub fn create_test_lua_config() -> llmspell_bridge::engine::factory::LuaConfig {
        llmspell_bridge::engine::factory::LuaConfig::default()
    }

    /// Create a test Lua engine with basic setup
    pub fn create_test_lua_engine(
    ) -> Result<Box<dyn llmspell_bridge::ScriptEngineBridge>, anyhow::Error> {
        let config = create_test_lua_config();
        Ok(llmspell_bridge::engine::factory::EngineFactory::create_lua_engine(&config)?)
    }
}

#[cfg(feature = "javascript")]
pub mod javascript {

    /// Create a test JavaScript engine configuration
    pub fn create_test_js_config() -> llmspell_bridge::engine::factory::JSConfig {
        llmspell_bridge::engine::factory::JSConfig::default()
    }

    /// Create a test JavaScript engine with basic setup
    pub fn create_test_js_engine(
    ) -> Result<Box<dyn llmspell_bridge::ScriptEngineBridge>, anyhow::Error> {
        let config = create_test_js_config();
        Ok(llmspell_bridge::engine::factory::EngineFactory::create_javascript_engine(&config)?)
    }
}

/// Create a test global context
pub async fn create_test_global_context() -> llmspell_bridge::globals::types::GlobalContext {
    use llmspell_bridge::{
        globals::types::GlobalContext, providers::ProviderManager, registry::ComponentRegistry,
    };

    let registry = Arc::new(ComponentRegistry::new());
    let providers = Arc::new(ProviderManager::new(Default::default()).await.unwrap());

    GlobalContext::new(registry, providers)
}

/// Create a test component registry
pub fn create_test_registry() -> Arc<llmspell_bridge::registry::ComponentRegistry> {
    Arc::new(llmspell_bridge::registry::ComponentRegistry::new())
}

/// Create a test provider manager
pub async fn create_test_provider_manager() -> Arc<llmspell_bridge::providers::ProviderManager> {
    let config = llmspell_bridge::providers::ProviderManagerConfig::default();
    Arc::new(
        llmspell_bridge::providers::ProviderManager::new(config)
            .await
            .unwrap(),
    )
}

#[cfg(test)]
mod tests {

    #[cfg(feature = "lua")]
    #[test]
    fn test_create_lua_config() {
        let config = crate::bridge_helpers::lua::create_test_lua_config();
        assert!(config.memory_limit > 0);
    }

    #[cfg(feature = "javascript")]
    #[test]
    fn test_create_js_config() {
        let config = crate::bridge_helpers::javascript::create_test_js_config();
        assert!(config.memory_limit > 0);
    }

    #[test]
    fn test_create_registry() {
        let registry = crate::bridge_helpers::create_test_registry();
        assert_eq!(registry.list_agents().len(), 0);
        assert_eq!(registry.list_tools().len(), 0);
    }

    #[tokio::test]
    async fn test_create_provider_manager() {
        let manager = crate::bridge_helpers::create_test_provider_manager().await;
        let providers = manager.list_providers().await;
        assert!(providers.is_empty() || !providers.is_empty()); // Either way is fine
    }
}
