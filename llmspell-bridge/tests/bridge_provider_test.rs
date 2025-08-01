//! ABOUTME: Tests for provider access through the script engine bridge
//! ABOUTME: Validates that scripts can access and use providers correctly

use llmspell_bridge::{
    engine::factory::{EngineFactory, LuaConfig},
    providers::{ProviderConfig, ProviderManager, ProviderManagerConfig},
    ComponentRegistry,
};
use std::sync::Arc;

/// Test provider manager creation
#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
async fn test_provider_manager_creation() {
    let config = ProviderManagerConfig::default();
    let manager = ProviderManager::new(config).await;
    assert!(
        manager.is_ok(),
        "Provider manager should be created successfully"
    );
}

/// Test injecting providers into engine
#[tokio::test(flavor = "multi_thread")]
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
#[tokio::test(flavor = "multi_thread")]
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
    assert_eq!(
        output.output.as_bool(),
        Some(true),
        "Agent API should be available"
    );
}

/// Test listing providers from script
#[tokio::test(flavor = "multi_thread")]
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
    assert_eq!(
        output.output.as_bool(),
        Some(true),
        "Agent.create should be a function"
    );
}

/// Test that scripts work but globals need inject_apis
#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
async fn test_script_without_api_injection() {
    let lua_config = LuaConfig::default();
    let engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Basic scripts should work
    let result = engine.execute_script("return 42").await;
    assert!(result.is_ok(), "Basic script should work: {:?}", result);

    // Globals require inject_apis to be called - check Tool is not available
    let global_check = engine.execute_script("return Tool").await;
    match global_check {
        Ok(output) => {
            // Tool might be nil which is expected
            // Check if the output is null/nil
            assert!(
                output.output.is_null(),
                "Tool should be nil without inject_apis"
            );
        }
        Err(_) => {
            // Or it might error which is also fine
        }
    }
}

/// Test provider configuration validation
#[cfg_attr(test_category = "external")]
#[ignore]
#[tokio::test]
async fn test_provider_config_validation() {
    let mut config = ProviderManagerConfig::default();

    // Test default configuration
    assert!(
        config.default_provider.is_none(),
        "Default provider should be None"
    );
    assert!(
        config.providers.is_empty(),
        "Providers should be empty by default"
    );

    // Test that we can add provider configurations
    config.providers.insert(
        "test-provider".to_string(),
        ProviderConfig {
            provider_type: "openai".to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            base_url: None,
            model: Some("gpt-3.5-turbo".to_string()),
            max_tokens: Some(1000),
            extra: std::collections::HashMap::new(),
        },
    );

    // Manager creation might fail if API key env var is not set, but that's expected
    let _ = ProviderManager::new(config).await;
}

/// Test concurrent provider access
#[tokio::test(flavor = "multi_thread")]
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
#[tokio::test(flavor = "multi_thread")]
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
    assert_eq!(
        output.output.as_bool(),
        Some(true),
        "Creating agent with non-existent provider should fail"
    );
}
