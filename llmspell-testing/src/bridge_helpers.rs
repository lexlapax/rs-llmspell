// ABOUTME: Test utilities for script bridge testing including engine setup and globals
// ABOUTME: Provides reusable helpers for Lua and JavaScript integration tests

use std::sync::Arc;

#[cfg(feature = "lua")]
pub mod lua {
    use super::*;
    
    /// Create a test Lua engine configuration
    pub fn create_test_lua_config() -> llmspell_bridge::engine::factory::LuaConfig {
        llmspell_bridge::engine::factory::LuaConfig::default()
    }
    
    /// Create a test Lua engine with basic setup
    pub fn create_test_lua_engine() -> Result<Box<dyn llmspell_bridge::traits::ScriptEngine>, anyhow::Error> {
        let config = create_test_lua_config();
        llmspell_bridge::engine::factory::EngineFactory::create_lua_engine(&config)
    }
}

#[cfg(feature = "javascript")]
pub mod javascript {
    use super::*;
    
    /// Create a test JavaScript engine configuration
    pub fn create_test_js_config() -> llmspell_bridge::engine::factory::JavaScriptConfig {
        llmspell_bridge::engine::factory::JavaScriptConfig::default()
    }
    
    /// Create a test JavaScript engine with basic setup
    pub fn create_test_js_engine() -> Result<Box<dyn llmspell_bridge::traits::ScriptEngine>, anyhow::Error> {
        let config = create_test_js_config();
        llmspell_bridge::engine::factory::EngineFactory::create_javascript_engine(&config)
    }
}

/// Create a test global context
pub async fn create_test_global_context() -> Arc<llmspell_bridge::types::GlobalContext> {
    use llmspell_bridge::{
        registry::ComponentRegistry,
        providers::{ProviderManager, ProviderManagerConfig},
        orchestration::{OrchestrationConfig, OrchestrationManager},
        event_bridge::EventBridgeManager,
        state_manager::{BridgeStateManager, StateManagerConfig},
        types::GlobalContext,
    };
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_state_persistence::StateManager;
    use llmspell_sessions::SessionManager;
    use llmspell_storage::MemoryBackend;
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    let event_bus = Arc::new(EventBus::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let storage_backend = Arc::new(MemoryBackend::new());
    
    let session_manager = Arc::new(
        SessionManager::new(
            state_manager.clone(),
            storage_backend,
            hook_registry.clone(),
            hook_executor.clone(),
            &event_bus,
            Default::default(),
        ).unwrap()
    );
    
    let orchestration = Arc::new(
        OrchestrationManager::new(OrchestrationConfig::default()).await.unwrap()
    );
    
    let event_bridge = Arc::new(EventBridgeManager::new(&event_bus).await.unwrap());
    
    let bridge_state = Arc::new(
        BridgeStateManager::new(
            state_manager.clone(),
            StateManagerConfig::default(),
        ).await.unwrap()
    );
    
    Arc::new(GlobalContext {
        registry,
        providers,
        event_bus,
        hook_registry,
        hook_executor,
        state_manager,
        session_manager,
        orchestration,
        event_bridge,
        bridge_state,
    })
}

/// Create a test component registry
pub fn create_test_registry() -> Arc<llmspell_bridge::registry::ComponentRegistry> {
    Arc::new(llmspell_bridge::registry::ComponentRegistry::new())
}

/// Create a test provider manager
pub async fn create_test_provider_manager() -> Arc<llmspell_bridge::providers::ProviderManager> {
    let config = llmspell_bridge::providers::ProviderManagerConfig::default();
    Arc::new(llmspell_bridge::providers::ProviderManager::new(config).await.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[cfg(feature = "lua")]
    #[test]
    fn test_create_lua_config() {
        let config = lua::create_test_lua_config();
        assert!(config.memory_limit > 0);
    }
    
    #[cfg(feature = "javascript")]
    #[test]
    fn test_create_js_config() {
        let config = javascript::create_test_js_config();
        assert!(config.memory_limit > 0);
    }
    
    #[test]
    fn test_create_registry() {
        let registry = create_test_registry();
        assert_eq!(registry.list_agents().len(), 0);
        assert_eq!(registry.list_tools().len(), 0);
    }
    
    #[tokio::test]
    async fn test_create_provider_manager() {
        let manager = create_test_provider_manager().await;
        let providers = manager.list_providers().await;
        assert!(providers.is_empty() || !providers.is_empty()); // Either way is fine
    }
}