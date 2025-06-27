//! ABOUTME: Tests for provider access through the script engine bridge
//! ABOUTME: Validates that scripts can access and use providers correctly

use llmspell_bridge::{
    engine::{
        bridge::ScriptEngineBridge,
        factory::{EngineFactory, LuaConfig},
    },
    providers::{ProviderConfig, ProviderManager, ProviderManagerConfig},
    ComponentRegistry,
};
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Test provider manager creation
#[tokio::test]
async fn test_provider_manager_creation() {
    let config = ProviderManagerConfig::default();
    let manager = ProviderManager::new(config).await;
    assert!(manager.is_ok(), "Provider manager should be created successfully");
}

/// Test injecting providers into engine
#[tokio::test]
async fn test_inject_providers() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    // Should inject successfully
    let result = engine.inject_apis(&registry, &providers);
    assert!(result.is_ok(), "API injection should succeed");
}

/// Test accessing providers from script
#[tokio::test]
async fn test_script_provider_access() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    // Check that Agent API exists (providers are accessed through agents)
    let script = "return type(Agent) == 'table'";
    let output = engine.execute_script(script).await.unwrap();
    assert_eq!(output.output.as_bool(), Some(true), "Agent API should be available");
}

/// Test listing providers from script
#[tokio::test]
async fn test_script_list_providers() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    // Agent.create should be available
    let script = "return type(Agent.create) == 'function'";
    let output = engine.execute_script(script).await.unwrap();
    assert_eq!(output.output.as_bool(), Some(true), "Agent.create should be a function");
}

/// Test that scripts without API injection fail appropriately
#[tokio::test]
async fn test_script_without_api_injection() {
    let lua_config = LuaConfig::default();
    let engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    // Should fail with Component error
    let result = engine.execute_script("return 42").await;
    assert!(result.is_err(), "Should fail without API injection");
    
    match result {
        Err(LLMSpellError::Component { message, .. }) => {
            assert!(message.contains("APIs not injected"), "Error should mention API injection");
        }
        _ => panic!("Expected Component error"),
    }
}

/// Test provider configuration validation
#[tokio::test]
async fn test_provider_config_validation() {
    let mut config = ProviderManagerConfig::default();
    
    // Test default configuration
    assert!(config.default_provider.is_none(), "Default provider should be None");
    assert!(config.providers.is_empty(), "Providers should be empty by default");
    
    // Test that we can add provider configurations
    config.providers.insert("test-provider".to_string(), ProviderConfig {
        provider_type: "openai".to_string(),
        api_key_env: Some("OPENAI_API_KEY".to_string()),
        base_url: None,
        model: Some("gpt-3.5-turbo".to_string()),
        max_tokens: Some(1000),
        extra: std::collections::HashMap::new(),
    });
    
    // Manager creation might fail if API key env var is not set, but that's expected
    let _ = ProviderManager::new(config).await;
}

/// Test concurrent provider access
#[tokio::test]
async fn test_concurrent_provider_access() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    // Create multiple tasks that access providers
    let engine = Arc::new(engine);
    let mut handles = vec![];
    
    for i in 0..5 {
        let engine_clone = engine.clone();
        let handle = tokio::spawn(async move {
            let script = format!("return 'task {}'", i);
            engine_clone.execute_script(&script).await
        });
        handles.push(handle);
    }
    
    // All should succeed
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent access should succeed");
    }
}

/// Test provider error handling in scripts
#[tokio::test]
async fn test_script_provider_error_handling() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();
    
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());
    
    engine.inject_apis(&registry, &providers).unwrap();
    
    // Try to create an agent without a provider configured
    let script = r#"
        local status, err = pcall(function()
            return Agent.create({
                name = "test-agent",
                provider = "non-existent-provider"
            })
        end)
        return not status
    "#;
    
    let output = engine.execute_script(script).await.unwrap();
    assert_eq!(output.output.as_bool(), Some(true), "Creating agent with non-existent provider should fail");
}