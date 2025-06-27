//! ABOUTME: Integration tests for LuaEngine implementation
//! ABOUTME: Validates basic script execution and API injection

#[cfg(feature = "lua")]
mod tests {
    use llmspell_bridge::{
        engine::factory::{LuaConfig, EngineFactory},
        registry::ComponentRegistry,
        providers::{ProviderManager, ProviderManagerConfig},
    };
    use std::sync::Arc;

    #[tokio::test]
    async fn test_lua_engine_creation() {
        let config = LuaConfig::default();
        let engine = EngineFactory::create_lua_engine(&config);
        assert!(engine.is_ok(), "Failed to create Lua engine");
        
        let engine = engine.unwrap();
        assert_eq!(engine.get_engine_name(), "lua");
        assert!(engine.supports_streaming());
        assert!(engine.supports_multimodal());
    }

    #[tokio::test]
    async fn test_lua_simple_execution() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        let result = engine.inject_apis(&registry, &providers);
        assert!(result.is_ok(), "Failed to inject APIs");
        
        // Execute simple script
        let script = "return 42";
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                assert_eq!(result.output.as_i64(), Some(42));
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_api_injection() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test that Agent global exists
        let script = "return Agent ~= nil";
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                assert_eq!(result.output.as_bool(), Some(true), "Agent global not found");
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_agent_create_placeholder() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test that Agent.create exists but returns error (placeholder)
        let script = r#"
            local ok, err = pcall(function()
                return Agent.create({system_prompt = "test"})
            end)
            return {ok = ok, error = tostring(err)}
        "#;
        
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("ok").and_then(|v| v.as_bool()), Some(false));
                let error = obj.get("error").and_then(|v| v.as_str()).unwrap_or("");
                assert!(error.contains("not yet implemented"), "Expected placeholder error");
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }
}