//! ABOUTME: Tests for Lua workflow API integration
//! ABOUTME: Verifies Lua scripts can create and execute workflows

use llmspell_bridge::engine::factory::LuaConfig;
use llmspell_bridge::engine::ScriptEngineBridge;
use llmspell_bridge::lua::LuaEngine;
use llmspell_bridge::{ComponentRegistry, ProviderManager};
use llmspell_config::providers::ProviderManagerConfig;
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

// Helper function to create test registry with calculator tool
fn create_test_registry() -> Arc<ComponentRegistry> {
    let registry = Arc::new(ComponentRegistry::new());
    registry
        .register_tool("calculator".to_string(), Arc::new(CalculatorTool::new()))
        .unwrap();
    registry
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_sequential_creation() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    let script = r#"
        local workflow = Workflow.sequential({
            name = "test_sequential",
            steps = {
                { name = "step1", type = "tool", tool = "calculator", input = { input = "5 + 3" } }
            }
        })
        
        local info = workflow:get_info()
        
        return { 
            workflow_type = info.type,
            has_name = info.name == "test_sequential"
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "sequential");
    assert_eq!(value["has_name"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_parallel() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    let script = r#"
        local workflow = Workflow.parallel({
            name = "parallel_calc",
            branches = {
                { name = "calc1", steps = {{ name = "add", type = "tool", tool = "calculator", input = { input = "1 + 2" } }} },
                { name = "calc2", steps = {{ name = "multiply", type = "tool", tool = "calculator", input = { input = "3 * 4" } }} }
            },
            max_concurrency = 2
        })
        
        local info = workflow:get_info()
        return { workflow_type = info.type }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "parallel");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_conditional() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Use builder API pattern as per fixed implementation
    let script = r#"
        local workflow = Workflow.builder()
            :name("conditional_test")
            :description("Test conditional workflow with builder API")
            :conditional()
            :add_step({
                name = "initial_step",
                type = "tool",
                tool = "calculator",
                input = { input = "2 + 2" }
            })
            :condition(function(ctx)
                -- Simple condition: always return true for then branch
                return true
            end)
            :add_then_step({
                name = "then_step",
                type = "tool",
                tool = "calculator",
                input = { input = "10 + 10" }
            })
            :add_else_step({
                name = "else_step",
                type = "tool",
                tool = "calculator",
                input = { input = "5 - 3" }
            })
            :build()
        
        local info = workflow:get_info()
        return { 
            workflow_type = info.type,
            has_name = info.name == "conditional_test"
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "conditional");
    assert_eq!(value["has_name"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_loop() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    let script = r#"
        local workflow = Workflow.loop({
            name = "loop_test",
            iterator = {
                range = {
                    start = 1,
                    ["end"] = 5,
                    step = 1
                }
            },
            body = {
                { name = "add_iteration", type = "tool", tool = "calculator", input = { input = "{{loop:current_value}} + 1" } }
            }
        })
        
        local info = workflow:get_info()
        return { workflow_type = info.type }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "loop");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_list() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    let script = r#"
        -- Create a workflow first
        local workflow = Workflow.sequential({
            name = "list_test",
            steps = {
                { name = "step1", type = "tool", tool = "calculator", input = { input = "1 + 1" } }
            }
        })
        
        -- List workflows
        local workflows = Workflow.list()
        
        return {
            count = #workflows,
            has_workflows = #workflows > 0
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["has_workflows"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_discover_types() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    let script = r#"
        local types = Workflow.types()
        
        local result = {
            count = #types,
            has_sequential = false,
            has_parallel = false,
            has_conditional = false,
            has_loop = false
        }
        
        -- Check for specific types
        for _, type_name in ipairs(types) do
            if type_name == "sequential" then
                result.has_sequential = true
            elseif type_name == "parallel" then
                result.has_parallel = true
            elseif type_name == "conditional" then
                result.has_conditional = true
            elseif type_name == "loop" then
                result.has_loop = true
            end
        end
        
        return result
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["count"], 4);
    assert_eq!(value["has_sequential"], true);
    assert_eq!(value["has_parallel"], true);
    assert_eq!(value["has_conditional"], true);
    assert_eq!(value["has_loop"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_workflow_error_handling() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers).unwrap();

    // Test with invalid workflow configuration
    let script = r"
        local success, err = pcall(function()
            local workflow = Workflow.sequential({
                -- Missing required 'name' field
                steps = {}
            })
        end)
        
        return {
            success = success,
            has_error = err ~= nil
        }
    ";

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["success"], false);
    assert_eq!(value["has_error"], true);
}
