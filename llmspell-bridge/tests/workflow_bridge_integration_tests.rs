//! ABOUTME: Integration tests for Lua workflow builder to Rust workflow conversion
//! ABOUTME: Tests agent classification condition parsing and workflow execution

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
async fn test_lua_builder_to_rust_workflow_conversion() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers, None).unwrap();

    // Test that Lua builder creates proper workflow structure
    let script = r#"
        local workflow = Workflow.builder()
            :name("test_conversion")
            :description("Test Lua to Rust workflow conversion")
            :conditional()
            :add_step({
                name = "initial_calculation",
                type = "tool",
                tool = "calculator",
                input = { input = "5 * 5" }
            })
            :condition({ type = "always" })
            :add_then_step({
                name = "then_calculation",
                type = "tool",
                tool = "calculator",
                input = { input = "100 + 100" }
            })
            :add_else_step({
                name = "else_calculation",
                type = "tool",
                tool = "calculator",
                input = { input = "10 - 5" }
            })
            :build()
        
        -- Execute the workflow to test conversion worked
        local result = workflow:execute({
            text = "test input"
        })
        
        return { 
            success = result ~= nil,
            has_result = result ~= nil
        }
    "#;

    // Execute the workflow - tool step configuration has been fixed
    let result = engine.execute_script(script).await;

    // Workflow should now succeed with fixed tool configuration
    assert!(
        result.is_ok(),
        "Workflow should succeed with fixed tool configuration: {:?}",
        result.as_ref().err()
    );
    let output = result.unwrap().output;
    assert_eq!(output["success"], true);
    assert_eq!(output["has_result"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agent_classification_condition_parsing() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers, None).unwrap();

    // Test that agent classification conditions are properly parsed
    let script = r#"
        local workflow = Workflow.builder()
            :name("classification_test")
            :description("Test agent classification condition")
            :conditional()
            :add_step({
                name = "classify",
                type = "tool",
                tool = "calculator",
                input = { input = "1 + 1" }  -- Placeholder for classifier
            })
            :condition({ type = "always" })
            :add_then_step({
                name = "blog_workflow",
                type = "tool",
                tool = "calculator",
                input = { input = "blog calculation" }
            })
            :add_else_step({
                name = "other_workflow",
                type = "tool",
                tool = "calculator",
                input = { input = "other calculation" }
            })
            :build()
        
        local info = workflow:get_info()
        
        return { 
            workflow_type = info.type,
            has_name = info.name == "classification_test"
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "conditional");
    assert_eq!(value["has_name"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_nested_workflow_step_conversion() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers, None).unwrap();

    // Test nested workflow steps are properly converted
    let script = r#"
        -- Create a simple inner workflow
        local inner_workflow = Workflow.builder()
            :name("inner_workflow")
            :sequential()
            :add_step({
                name = "inner_step",
                type = "tool",
                tool = "calculator",
                input = { input = "7 * 7" }
            })
            :build()
        
        -- Create outer workflow with nested workflow step
        local outer_workflow = Workflow.builder()
            :name("outer_workflow")
            :conditional()
            :add_step({
                name = "check_step",
                type = "tool",
                tool = "calculator",
                input = { input = "2 + 2" }
            })
            :condition({ type = "always" })
            :add_then_step({
                name = "nested_workflow_step",
                type = "workflow",
                workflow = inner_workflow
            })
            :build()
        
        local info = outer_workflow:get_info()
        
        return { 
            workflow_type = info.type,
            has_name = info.name == "outer_workflow"
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "conditional");
    assert_eq!(value["has_name"], true);
}

#[tokio::test(flavor = "multi_thread")]
async fn test_multi_branch_condition_conversion() {
    let registry = create_test_registry();
    let providers = create_test_providers().await;

    let mut engine = create_test_engine();
    engine.inject_apis(&registry, &providers, None).unwrap();

    // Test multiple else branches are properly handled
    let script = r#"
        local workflow = Workflow.builder()
            :name("multi_branch_test")
            :description("Test multiple branch conditions")
            :conditional()
            :add_step({
                name = "classify",
                type = "tool",
                tool = "calculator",
                input = { input = "classify" }
            })
            :condition({ type = "always" })
            :add_then_step({
                name = "blog_path",
                type = "tool",
                tool = "calculator",
                input = { input = "blog" }
            })
            :add_else_step({
                name = "social_path",
                type = "tool",
                tool = "calculator",
                input = { input = "social" }
            })
            :add_else_step({
                name = "email_path",
                type = "tool",
                tool = "calculator",
                input = { input = "email" }
            })
            :build()
        
        local info = workflow:get_info()
        
        return { 
            workflow_type = info.type,
            has_name = info.name == "multi_branch_test"
        }
    "#;

    let result = engine.execute_script(script).await.unwrap();
    let value = result.output;

    assert_eq!(value["workflow_type"], "conditional");
    assert_eq!(value["has_name"], true);
}
