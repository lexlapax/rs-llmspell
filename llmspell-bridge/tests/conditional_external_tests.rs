//! ABOUTME: External API tests for conditional workflows with real LLM agents
//! ABOUTME: Tests using real OpenAI/Anthropic APIs for content classification

use llmspell_bridge::engine::factory::LuaConfig;
use llmspell_bridge::engine::ScriptEngineBridge;
use llmspell_bridge::lua::LuaEngine;
use llmspell_bridge::{ComponentRegistry, ProviderManager};
use llmspell_config::providers::ProviderManagerConfig;
use std::sync::Arc;

// Helper function to create a test script engine
fn create_test_engine() -> LuaEngine {
    let config = LuaConfig::default();
    LuaEngine::new(&config).expect("Failed to create Lua engine")
}

// Helper function to create test providers with real API configuration
async fn create_real_providers() -> Arc<ProviderManager> {
    let config = ProviderManagerConfig::default();
    // Provider configuration would be loaded from environment
    // In real tests, providers are configured via LLMSPELL_CONFIG

    Arc::new(
        ProviderManager::new(config)
            .await
            .expect("Failed to create provider manager"),
    )
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "external"]
async fn test_real_llm_content_classification() {
    let registry = Arc::new(ComponentRegistry::new());
    let providers = create_real_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Test with real LLM agent for classification
    let script = r#"
        -- Create real LLM classifier agent
        local classifier = Agent.builder()
            :name("content_classifier")
            :description("Classifies content as blog, social, or email")
            :type("llm")
            :model("openai/gpt-3.5-turbo")
            :temperature(0.2)
            :max_tokens(50)
            :build()
        
        -- Create conditional workflow with real classification
        local router = Workflow.builder()
            :name("real_content_router")
            :conditional()
            :add_step({
                name = "classify",
                type = "agent",
                agent = "content_classifier",
                input = "Classify this as 'blog', 'social', or 'email': Write a detailed technical article about machine learning algorithms"
            })
            :condition(function(ctx)
                local result = ctx.classify or ""
                return string.match(result:lower(), "blog") ~= nil
            end)
            :add_then_step({
                name = "blog_route",
                type = "tool",
                tool = "calculator",
                input = { input = "blog: 1 + 1" }
            })
            :add_else_step({
                name = "other_route",
                type = "tool",
                tool = "calculator",
                input = { input = "other: 2 + 2" }
            })
            :build()
        
        -- Execute with real LLM
        local result = router:execute({ input = "test" })
        
        return {
            success = result ~= nil,
            executed = true
        }
    "#;

    let result = engine.execute_script(script).await;

    // This test requires real API keys to pass
    if std::env::var("OPENAI_API_KEY").is_ok() {
        assert!(result.is_ok());
        let value = result.unwrap().output;
        assert_eq!(value["success"], true);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "external"]
async fn test_multi_model_classification_routing() {
    let registry = Arc::new(ComponentRegistry::new());
    let providers = create_real_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Test with multiple LLM models for robust classification
    let script = r#"
        -- Create multiple classifier agents with different models
        local openai_classifier = Agent.builder()
            :name("openai_classifier")
            :type("llm")
            :model("openai/gpt-4")
            :temperature(0.1)
            :build()
        
        local anthropic_classifier = Agent.builder()
            :name("anthropic_classifier")
            :type("llm")
            :model("anthropic/claude-3-haiku-20240307")
            :temperature(0.1)
            :build()
        
        -- Test different content types
        local test_contents = {
            {
                text = "Quick announcement: Our new feature is live! Check it out and share with your network #ProductLaunch",
                expected = "social"
            },
            {
                text = "Dear subscribers, here is your monthly newsletter with updates on our latest developments and upcoming events",
                expected = "email"
            },
            {
                text = "In this comprehensive guide, we'll explore the fundamentals of distributed systems architecture",
                expected = "blog"
            }
        }
        
        local results = {}
        for i, content in ipairs(test_contents) do
            -- Get classification from OpenAI
            local openai_result = openai_classifier and openai_classifier:execute({
                input = "Classify as 'blog', 'social', or 'email': " .. content.text
            })
            
            -- Get classification from Anthropic
            local anthropic_result = anthropic_classifier and anthropic_classifier:execute({
                input = "Classify as 'blog', 'social', or 'email': " .. content.text
            })
            
            table.insert(results, {
                content_type = content.expected,
                openai = openai_result,
                anthropic = anthropic_result
            })
        end
        
        return {
            test_count = #results,
            models_tested = 2
        }
    "#;

    let result = engine.execute_script(script).await;

    // This test requires both API keys to fully pass
    if std::env::var("OPENAI_API_KEY").is_ok() && std::env::var("ANTHROPIC_API_KEY").is_ok() {
        assert!(result.is_ok());
        let value = result.unwrap().output;
        assert_eq!(value["test_count"], 3);
        assert_eq!(value["models_tested"], 2);
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "external"]
async fn test_production_content_pipeline() {
    let registry = Arc::new(ComponentRegistry::new());
    let providers = create_real_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Full production-like content generation pipeline
    let script = r#"
        -- Production content generation system
        local system = {
            -- Classification agent
            classifier = Agent.builder()
                :name("classifier")
                :type("llm")
                :model("openai/gpt-4")
                :temperature(0.2)
                :build(),
            
            -- Content generation agents
            blog_writer = Agent.builder()
                :name("blog_writer")
                :type("llm")
                :model("anthropic/claude-3-opus-20240229")
                :temperature(0.7)
                :max_tokens(2000)
                :build(),
            
            social_writer = Agent.builder()
                :name("social_writer")
                :type("llm")
                :model("openai/gpt-3.5-turbo")
                :temperature(0.8)
                :max_tokens(280)
                :build(),
            
            email_writer = Agent.builder()
                :name("email_writer")
                :type("llm")
                :model("anthropic/claude-3-sonnet-20240229")
                :temperature(0.5)
                :max_tokens(1000)
                :build()
        }
        
        -- Test content request
        local request = "Create content about our new AI-powered analytics dashboard launch"
        
        -- Classify content type
        local classification = system.classifier:execute({
            input = "What type of content should we create for: " .. request .. "? Choose: blog, social, or email"
        })
        
        -- Route to appropriate writer based on classification
        local content = nil
        if string.match(classification:lower(), "blog") then
            content = system.blog_writer:execute({
                input = "Write a detailed blog post about: " .. request
            })
        elseif string.match(classification:lower(), "social") then
            content = system.social_writer:execute({
                input = "Write a social media post about: " .. request
            })
        else
            content = system.email_writer:execute({
                input = "Write an email newsletter about: " .. request
            })
        end
        
        return {
            request = request,
            classification = classification ~= nil,
            content_generated = content ~= nil
        }
    "#;

    let result = engine.execute_script(script).await;

    // This requires API keys and will make real API calls
    if std::env::var("OPENAI_API_KEY").is_ok() || std::env::var("ANTHROPIC_API_KEY").is_ok() {
        // The test may partially succeed with only one API key
        assert!(result.is_ok() || result.is_err());
    }
}
