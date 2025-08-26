//! Test provider integration with script runtime

use llmspell_bridge::ScriptRuntime;
use llmspell_config::{LLMSpellConfig, ProviderConfig, ProviderManagerConfig};
use std::collections::HashMap;
#[tokio::test]
async fn test_lua_agent_creation_with_mock_provider() {
    // Create runtime config with a mock provider
    let mut provider_config = HashMap::new();
    provider_config.insert(
        "test".to_string(),
        ProviderConfig {
            name: "test".to_string(),
            provider_type: "mock".to_string(),
            enabled: true,
            base_url: None,
            api_key_env: None,
            api_key: None,
            default_model: Some("test-model".to_string()),
            max_tokens: None,
            timeout_seconds: None,
            max_retries: None,
            rate_limit: None,
            retry: None,
            options: HashMap::new(),
        },
    );

    let runtime_config = LLMSpellConfig {
        default_engine: "lua".to_string(),
        providers: ProviderManagerConfig {
            default_provider: Some("test".to_string()),
            providers: provider_config,
        },
        ..Default::default()
    };

    // Create runtime with Lua engine
    let runtime = ScriptRuntime::new_with_lua(runtime_config).await;

    // For now, we expect this to fail since we don't have a mock provider implementation
    assert!(runtime.is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_script_provider_access() {
    let runtime_config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(runtime_config).await.unwrap();

    // Test script that tries to create an agent
    let script = r#"
        -- Try to create an agent (will fail without provider config)
        local success, result = pcall(function()
            return Agent.create({
                system_prompt = "You are a helpful assistant",
                temperature = 0.7
            })
        end)
        
        -- We expect this to fail for now
        return not success
    "#;

    let result = runtime.execute_script(script).await;
    assert!(result.is_ok());

    // The script should return true (meaning Agent.create failed as expected)
    if let Ok(output) = result {
        assert_eq!(output.output, serde_json::Value::Bool(true));
    }
}
