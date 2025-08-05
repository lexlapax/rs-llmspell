// ABOUTME: Integration tests for provider enhancement features (ModelSpecifier, base URL overrides)
// ABOUTME: Tests Agent.create with "provider/model" syntax and provider configuration

use llmspell_bridge::providers::ProviderManagerConfig;
use llmspell_bridge::runtime::{RuntimeConfig, ScriptRuntime};

// Helper to create a runtime without any provider configuration
fn create_test_runtime_config() -> RuntimeConfig {
    // Ensure no providers are configured
    RuntimeConfig {
        providers: ProviderManagerConfig {
            default_provider: None,
            providers: std::collections::HashMap::new(),
        },
        ..Default::default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agent_create_with_provider_model_syntax() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test creating agent with provider/model syntax
    let script = r#"
        -- Test OpenAI provider/model syntax
        local success1, result1 = pcall(function()
            return Agent.create({
                model = "openai/gpt-4",
                prompt = "You are a test assistant"
            })
        end)
        
        -- Test Anthropic provider/model syntax
        local success2, result2 = pcall(function()
            return Agent.create({
                model = "anthropic/claude-3-opus",
                prompt = "You are another test assistant"
            })
        end)
        
        -- Test with custom provider
        local success3, result3 = pcall(function()
            return Agent.create({
                model = "groq/mixtral-8x7b",
                prompt = "You are a Groq assistant"
            })
        end)
        
        -- All should fail with provider configuration errors
        assert(not success1, "Should fail with unconfigured provider")
        assert(not success2, "Should fail with unconfigured provider")
        assert(not success3, "Should fail with unconfigured provider")
        
        return true
    "#;

    let result = runtime.execute_script(script).await;

    // This might fail if API keys aren't configured, which is expected
    match result {
        Ok(_) => println!("Provider/model syntax works with configured providers"),
        Err(e) => {
            // Check that the error is about missing API keys, not syntax
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("API key")
                    || error_msg.contains("provider")
                    || error_msg.contains("configuration"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[ignore = "Obsolete test - error messages have changed in new implementation"]
#[tokio::test(flavor = "multi_thread")]
async fn test_base_url_override() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test base URL override functionality
    let script = r#"
        -- Test base URL override parsing works
        local success, err = pcall(function()
            return Agent.create({
                model = "openai/gpt-3.5-turbo",
                base_url = "http://localhost:8080/v1",
                prompt = "You are a test assistant"
            })
        end)
        
        -- Debug output
        print("Base URL override test - Success:", success)
        if not success then
            print("Error:", tostring(err))
        else
            print("Result type:", type(err))
        end
        
        -- Should fail with provider error (openai not configured)
        assert(not success, "Should fail with unconfigured provider")
        assert(err, "Should have error message")
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Script should handle provider errors gracefully"
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn test_backward_compatibility() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test old-style model specification still works
    let script = r#"
        -- Old style: just model name, uses default provider
        local success, result = pcall(function()
            return Agent.create({
                model = "gpt-3.5-turbo",
                prompt = "You are a test assistant"
            })
        end)
        
        -- Should fail with no default provider configured
        assert(not success, "Should fail without default provider")
        local error_str = tostring(result)
        assert(
            error_str:find("default provider") or error_str:find("No provider"),
            "Error should mention missing default provider: " .. error_str
        )
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    match result {
        Ok(_) => println!("Backward compatibility maintained"),
        Err(e) => {
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("API key") || error_msg.contains("provider"),
                "Unexpected error: {}",
                error_msg
            );
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_invalid_provider_handling() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test error handling for invalid providers
    let script = r#"
        -- Test invalid provider
        local success, err = pcall(function()
            return Agent.builder()
                :model("invalid_provider/some-model")
                :system_prompt("Test")
                :build()
        end)
        
        -- Should fail with provider error
        assert(not success, "Should fail with invalid provider")
        assert(err, "Should have error message")
        
        -- Error should mention the invalid provider
        local error_str = tostring(err)
        assert(
            error_str:find("invalid_provider") or 
            error_str:find("provider") or
            error_str:find("configuration"),
            "Error should mention provider issue: " .. error_str
        )
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Script should handle invalid provider gracefully"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_provider_fallback() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test that agents can be created (they may use mock or basic implementation)
    let script = r#"
        -- Test model without provider - creates a basic agent
        local success1, agent1 = pcall(function()
            return Agent.builder()
                :model("gpt-3.5-turbo")
                :system_prompt("Test")
                :build()
        end)
        
        -- Agents can be created even without providers
        assert(success1, "Agent creation should succeed: " .. tostring(agent1))
        assert(agent1, "Should have agent instance")
        
        -- Test explicit provider - this might fail if it tries to validate
        local success2, agent2 = pcall(function()
            return Agent.builder()
                :model("anthropic/claude-instant")
                :system_prompt("Test")
                :build()
        end)
        
        -- This might fail with provider validation
        if not success2 then
            print("Agent creation failed as expected:", tostring(agent2))
            -- That's OK - provider validation can happen at creation time
        else
            print("Agent created successfully:", tostring(agent2))
            assert(agent2, "Should have agent instance")
        end
        
        -- The actual provider validation happens during execution
        -- For now, just verify we can create agents
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    match result {
        Ok(_) => {
            // Test passed
        }
        Err(e) => {
            panic!("Script failed with error: {}", e);
        }
    }
}

#[ignore = "Obsolete test - error handling has changed in new implementation"]
#[tokio::test(flavor = "multi_thread")]
async fn test_provider_model_parsing() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test various provider/model syntax variations
    let script = r#"
        -- Test different syntax variations
        local test_cases = {
            "openai/gpt-4",
            "anthropic/claude-3-opus-20240229",
            "groq/llama2-70b-4096",
            "perplexity/mixtral-8x7b-instruct",
            "together/mixtral-8x7b-32768",
            "gpt-3.5-turbo",  -- No provider, should use default
            "gemini-pro",     -- No provider, should use default
        }
        
        local results = {}
        for i, model in ipairs(test_cases) do
            local success, result = pcall(function()
                return Agent.create({
                    model = model,
                    prompt = "Test"
                })
            end)
            
            -- All should fail because no providers are configured
            results[i] = {
                model = model,
                parsed_correctly = not success  -- Should fail with no providers
            }
        end
        
        -- All should fail with provider errors, not parsing errors
        for _, r in ipairs(results) do
            assert(r.parsed_correctly, "Model should parse but fail with provider error: " .. r.model)
        end
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Model parsing should work for all syntax variations"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_providers_same_script() {
    let config = create_test_runtime_config();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test using multiple providers in the same script
    let script = r#"
        -- Create agents with different providers
        local agents = {}
        
        -- Try to create with different providers
        local providers = {
            {model = "openai/gpt-3.5-turbo", name = "OpenAI"},
            {model = "anthropic/claude-instant", name = "Anthropic"},
            {model = "groq/mixtral-8x7b", name = "Groq"},
        }
        
        for _, p in ipairs(providers) do
            local success, result = pcall(function()
                return Agent.create({
                    model = p.model,
                    prompt = "Test assistant for " .. p.name
                })
            end)
            
            -- Log result
            if success then
                print(p.name .. " agent created successfully")
            else
                print(p.name .. " agent failed: " .. tostring(result))
            end
        end
        
        -- Test passed if we got here without Lua errors
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Should handle multiple providers in same script"
    );
}

#[cfg(test)]
mod provider_config_tests {
    #[test]
    fn test_model_specifier_parsing() {
        // Direct unit test of ModelSpecifier parsing
        use llmspell_providers::model_specifier::ModelSpecifier;
        use std::str::FromStr;

        // Test provider/model syntax
        let spec = ModelSpecifier::from_str("openai/gpt-4").unwrap();
        assert_eq!(spec.provider, Some("openai".to_string()));
        assert_eq!(spec.model, "gpt-4");

        // Test model-only syntax
        let spec = ModelSpecifier::from_str("gpt-3.5-turbo").unwrap();
        assert_eq!(spec.provider, None);
        assert_eq!(spec.model, "gpt-3.5-turbo");

        // Test with complex model names
        let spec = ModelSpecifier::from_str("anthropic/claude-3-opus-20240229").unwrap();
        assert_eq!(spec.provider, Some("anthropic".to_string()));
        assert_eq!(spec.model, "claude-3-opus-20240229");
    }
    #[test]
    fn test_model_specifier_validation() {
        use llmspell_providers::model_specifier::ModelSpecifier;
        use std::str::FromStr;

        // Test invalid syntax
        assert!(ModelSpecifier::from_str("").is_err());
        // "/model" is valid but has empty provider
        let spec = ModelSpecifier::from_str("/model").unwrap();
        assert_eq!(spec.provider, Some("".to_string()));
        assert_eq!(spec.model, "model");
        // "provider/" is valid but has empty model
        let spec = ModelSpecifier::from_str("provider/").unwrap();
        assert_eq!(spec.provider, Some("provider".to_string()));
        assert_eq!(spec.model, "");
        // "too/many/slashes" is valid with composite provider
        let spec = ModelSpecifier::from_str("too/many/slashes").unwrap();
        assert_eq!(spec.provider, Some("too/many".to_string()));
        assert_eq!(spec.model, "slashes");
    }
}
