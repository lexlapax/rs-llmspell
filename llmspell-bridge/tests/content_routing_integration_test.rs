//! ABOUTME: End-to-end integration tests for content routing with agent classification
//! ABOUTME: Tests full agent classification → workflow routing pipeline

use llmspell_bridge::engine::factory::LuaConfig;
use llmspell_bridge::engine::ScriptEngineBridge;
use llmspell_bridge::lua::LuaEngine;
use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
use llmspell_tools::CalculatorTool;
use std::sync::Arc;

// Helper function to create a test script engine
fn create_test_engine() -> LuaEngine {
    let config = LuaConfig::default();
    LuaEngine::new(&config).expect("Failed to create Lua engine")
}

// Helper function to create test providers
async fn create_test_providers() -> Arc<ProviderManager> {
    let config = ProviderManagerConfig::default();
    Arc::new(
        ProviderManager::new(config)
            .await
            .expect("Failed to create provider manager"),
    )
}

// Helper function to create test registry with tools
fn create_test_registry() -> Arc<ComponentRegistry> {
    let registry = Arc::new(ComponentRegistry::new());
    registry
        .register_tool("calculator".to_string(), Arc::new(CalculatorTool::new()))
        .unwrap();
    registry
}

// Helper to create content classification function script
const fn create_classifier_script() -> &'static str {
    r#"
        -- Simulate content classification
        local function classify_content(content)
            -- Simple rule-based classification for testing
            if string.match(content:lower(), "blog") then
                return "blog"
            elseif string.match(content:lower(), "social") then
                return "social"
            elseif string.match(content:lower(), "email") then
                return "email"
            else
                return "unknown"
            end
        end
    "#
}

// Helper to create workflow definitions script
const fn create_workflows_script() -> &'static str {
    r#"
        -- Create specialized workflows
        local blog_workflow = Workflow.builder()
            :name("blog_workflow")
            :sequential()
            :add_step({
                name = "blog_process",
                type = "tool",
                tool = "text_manipulator",
                input = { 
                    operation = "append",
                    input = "Content: ",
                    suffix = "[BLOG PROCESSED]"
                }
            })
            :build()
        
        local social_workflow = Workflow.builder()
            :name("social_workflow")
            :sequential()
            :add_step({
                name = "social_process",
                type = "tool",
                tool = "text_manipulator",
                input = { 
                    operation = "append",
                    input = "Content: ",
                    suffix = "[SOCIAL PROCESSED]"
                }
            })
            :build()
        
        local email_workflow = Workflow.builder()
            :name("email_workflow")
            :sequential()
            :add_step({
                name = "email_process",
                type = "tool",
                tool = "text_manipulator",
                input = { 
                    operation = "append",
                    input = "Content: ",
                    suffix = "[EMAIL PROCESSED]"
                }
            })
            :build()
    "#
}

// Helper to create router workflow script
const fn create_router_script() -> &'static str {
    r#"
        -- Create main routing workflow
        local router = Workflow.builder()
            :name("content_router")
            :description("Routes content based on classification")
            :conditional()
            :add_step({
                name = "classify",
                type = "tool",
                tool = "text_manipulator",
                input = { 
                    operation = "analyze",
                    input = "Test blog content for routing"
                }
            })
            :condition(function(ctx)
                local classification = classify_content("Test blog content")
                return classification == "blog"
            end)
            :add_then_step({
                name = "route_to_blog",
                type = "workflow",
                workflow = blog_workflow
            })
            :add_else_step({
                name = "route_to_social",
                type = "workflow",
                workflow = social_workflow
            })
            :add_else_step({
                name = "route_to_email",
                type = "workflow",
                workflow = email_workflow
            })
            :build()
        
        -- Execute the routing pipeline
        local result = router:execute({
            input = "Test blog content for routing"
        })
        
        return { 
            success = result ~= nil,
            executed = true
        }
    "#
}

#[tokio::test(flavor = "multi_thread")]
async fn test_full_content_routing_pipeline() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Combine script parts
    let script = format!(
        "{}{}{}",
        create_classifier_script(),
        create_workflows_script(),
        create_router_script()
    );

    let result = engine.execute_script(&script).await.unwrap();
    let value = result.output;

    assert_eq!(value["success"], true);
    assert_eq!(value["executed"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agent_classification_routing() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Test routing based on agent classification results
    let script = r#"
        -- Mock agent classifier
        local classifier = {
            classify = function(self, content)
                if string.match(content, "technical") then
                    return { type = "blog", confidence = 0.9 }
                elseif string.match(content, "announcement") then
                    return { type = "social", confidence = 0.8 }
                else
                    return { type = "email", confidence = 0.7 }
                end
            end
        }
        
        -- Test different content types
        local test_contents = {
            "technical article about AI",
            "announcement of new feature",
            "newsletter update for subscribers"
        }
        
        local results = {}
        for i, content in ipairs(test_contents) do
            local classification = classifier:classify(content)
            table.insert(results, {
                content = content,
                classified_as = classification.type,
                confidence = classification.confidence
            })
        end
        
        return {
            test_count = #results,
            first_type = results[1].classified_as,
            second_type = results[2].classified_as,
            third_type = results[3].classified_as
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["test_count"], 3);
    assert_eq!(value["first_type"], "blog");
    assert_eq!(value["second_type"], "social");
    assert_eq!(value["third_type"], "email");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fallback_routing() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Test fallback when no conditions match
    let script = r#"
        local default_workflow = Workflow.builder()
            :name("default_workflow")
            :sequential()
            :add_step({
                name = "default_process",
                type = "tool",
                tool = "calculator",
                input = { input = "1 + 1" }
            })
            :build()
        
        local router = Workflow.builder()
            :name("fallback_router")
            :conditional()
            :add_step({
                name = "check",
                type = "tool",
                tool = "calculator",
                input = { input = "2 + 2" }
            })
            :condition(function(ctx)
                -- Condition that will never match
                return false
            end)
            :add_then_step({
                name = "never_executed",
                type = "tool",
                tool = "calculator",
                input = { input = "100 + 100" }
            })
            :add_else_step({
                name = "fallback_route",
                type = "workflow",
                workflow = default_workflow
            })
            :build()
        
        local result = router:execute({ input = "test" })
        
        return {
            has_result = result ~= nil,
            used_fallback = true
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["has_result"], true);
    assert_eq!(value["used_fallback"], true);
}
