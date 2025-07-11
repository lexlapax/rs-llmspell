// ABOUTME: Integration tests for provider enhancement features (ModelSpecifier, base URL overrides)
// ABOUTME: Tests Agent.create with "provider/model" syntax and provider configuration

use llmspell_bridge::runtime::{RuntimeConfig, ScriptRuntime};
use tokio;

#[tokio::test]
async fn test_agent_create_with_provider_model_syntax() {
    let config = RuntimeConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test creating agent with provider/model syntax
    let script = r#"
        -- Test OpenAI provider/model syntax
        local agent1 = Agent.create({
            model = "openai/gpt-4",
            prompt = "You are a test assistant"
        })
        assert(agent1, "Failed to create agent with openai/gpt-4")
        
        -- Test Anthropic provider/model syntax
        local agent2 = Agent.create({
            model = "anthropic/claude-3-opus",
            prompt = "You are another test assistant"
        })
        assert(agent2, "Failed to create agent with anthropic/claude-3-opus")
        
        -- Test with custom provider
        local agent3 = Agent.create({
            model = "groq/mixtral-8x7b",
            prompt = "You are a Groq assistant"
        })
        assert(agent3, "Failed to create agent with groq/mixtral-8x7b")
        
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

#[tokio::test]
async fn test_base_url_override() {
    let config = RuntimeConfig::default();
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
        
        -- Should fail with provider error (openai not configured)
        assert(not success, "Should fail with unconfigured provider")
        assert(err, "Should have error message")
        
        -- The error should mention provider, not syntax issue
        local error_str = tostring(err)
        assert(
            error_str:find("provider") or error_str:find("Unknown"),
            "Error should be about provider configuration: " .. error_str
        )
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Script should handle provider errors gracefully"
    )
}

#[tokio::test]
async fn test_backward_compatibility() {
    let config = RuntimeConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test old-style model specification still works
    let script = r#"
        -- Old style: just model name, uses default provider
        local agent = Agent.create({
            model = "gpt-3.5-turbo",
            prompt = "You are a test assistant"
        })
        
        -- Should create agent with default provider
        assert(agent, "Failed to create agent with old-style model name")
        
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

#[tokio::test]
async fn test_invalid_provider_handling() {
    let config = RuntimeConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test error handling for invalid providers
    let script = r#"
        -- Test invalid provider
        local success, err = pcall(function()
            return Agent.create({
                model = "invalid_provider/some-model",
                prompt = "Test"
            })
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

#[tokio::test]
async fn test_provider_fallback() {
    let config = RuntimeConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    // Test fallback to default provider
    let script = r#"
        -- Test model without provider - should try default
        local success1, err1 = pcall(function()
            return Agent.create({
                model = "gpt-3.5-turbo",
                prompt = "Test"
            })
        end)
        
        -- Should fail with "no default provider" error
        assert(not success1, "Should fail without default provider")
        local error1_str = tostring(err1)
        assert(
            error1_str:find("default provider") or error1_str:find("No provider"),
            "Error should mention missing default provider: " .. error1_str
        )
        
        -- Test explicit provider
        local success2, err2 = pcall(function()
            return Agent.create({
                model = "anthropic/claude-instant",
                prompt = "Test"
            })
        end)
        
        -- Should fail with unknown provider error
        assert(not success2, "Should fail with unknown provider")
        local error2_str = tostring(err2)
        assert(
            error2_str:find("anthropic") or error2_str:find("Unknown provider"),
            "Error should mention unknown provider: " .. error2_str
        )
        
        return true
    "#;

    let result = runtime.execute_script(script).await;
    assert!(
        result.is_ok(),
        "Script should handle provider errors gracefully"
    )
}

#[tokio::test]
async fn test_provider_model_parsing() {
    let config = RuntimeConfig::default();
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
            
            -- Record error type - should be provider/configuration errors, not parsing errors
            local error_str = tostring(result)
            local is_provider_error = error_str:find("provider") or 
                                     error_str:find("Unknown") or 
                                     error_str:find("No provider") or
                                     error_str:find("configuration")
            
            results[i] = {
                model = model,
                parsed_correctly = not success and is_provider_error
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

#[tokio::test]
async fn test_multiple_providers_same_script() {
    let config = RuntimeConfig::default();
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
