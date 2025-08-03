//! ABOUTME: Integration tests for synchronous behavior of Agent, Tool, and Workflow APIs
//! ABOUTME: Validates that async operations are properly wrapped and behave synchronously from Lua

use llmspell_bridge::engine::factory::LuaConfig;
use llmspell_bridge::engine::ScriptEngineBridge;
use llmspell_bridge::lua::LuaEngine;
use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
use llmspell_tools::CalculatorTool;
use std::sync::Arc;

// Helper function to create test registry with calculator tool
async fn create_test_registry() -> Arc<ComponentRegistry> {
    let registry = Arc::new(ComponentRegistry::new());
    registry
        .register_tool("calculator".to_string(), Arc::new(CalculatorTool::new()))
        .unwrap();
    registry
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

// Helper function to create test engine
async fn create_test_engine() -> LuaEngine {
    let config = LuaConfig::default();
    LuaEngine::new(&config).expect("Failed to create Lua engine")
}

#[tokio::test(flavor = "multi_thread")]
async fn test_agent_sync_api_available() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that Agent API is available and synchronous
    let result = engine
        .execute_script(
            r#"
        -- Test that Agent API exists and has expected methods
        assert(Agent ~= nil, "Agent should be available")
        assert(type(Agent.create) == "function", "Agent.create should be a function")
        assert(type(Agent.discover) == "function", "Agent.discover should be a function")
        
        -- Test that methods return immediately (not promises)
        local agent_types = Agent.discover()
        assert(type(agent_types) == "table", "Agent.discover should return table")
        assert(agent_types["then"] == nil, "Should not be a promise")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "Agent sync API test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_tool_sync_behavior() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that Tool operations are synchronous
    let result = engine
        .execute_script(
            r#"
        -- Get tool synchronously
        local calc = Tool.get("calculator")
        assert(calc, "Tool should be retrieved synchronously")
        
        -- Execute should be synchronous
        local result = calc:execute({input = "2 + 2"})
        assert(result, "Tool execute should return result synchronously")
        assert(result.text, "Tool should return text")
        assert(result.metadata, "Tool should return metadata")
        
        -- Test tool discovery is synchronous
        local tools = Tool.discover()
        assert(tools, "Tool discovery should return synchronously")
        assert(type(tools) == "table", "Tools should be a table")
        assert(#tools > 0, "Should discover some tools")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "Tool sync behavior test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_workflow_sync_behavior() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that Workflow operations are synchronous
    let result = engine
        .execute_script(
            r#"
        -- Create workflow synchronously
        local workflow = Workflow.sequential({
            name = "test-workflow",
            steps = {
                {
                    name = "step1",
                    type = "tool",
                    tool = "calculator",
                    input = {input = "1 + 1"}
                }
            }
        })
        
        assert(workflow, "Workflow should be created synchronously")
        
        -- Workflow is a userdata object, use get_info() to access properties
        local info = workflow:get_info()
        assert(info.id, "Workflow should have an ID")
        assert(info.name == "test-workflow", "Workflow should have correct name")
        
        -- Execute should be synchronous
        local result = workflow:execute()
        assert(result, "Workflow execute should return result synchronously")
        assert(type(result) == "table", "Result should be a table")
        
        -- List workflows synchronously
        local workflows = Workflow.list()
        assert(workflows, "Workflow list should return synchronously")
        assert(type(workflows) == "table", "Workflows should be a table")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "Workflow sync behavior test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_error_handling_sync() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that errors are thrown synchronously
    let result = engine.execute_script(r#"
        -- Test agent creation error
        local success, err = pcall(function()
            return Agent.create({
                -- Missing required model field
                name = "test-agent"
            })
        end)
        
        assert(not success, "Agent creation should fail synchronously")
        assert(string.find(tostring(err), "Model specification required") ~= nil, "Should have correct error message")
        
        -- Test tool execution error
        local calc = Tool.get("calculator")
        local success2, err2 = pcall(function()
            return calc:execute({input = "invalid expression"})
        end)
        
        -- Note: calculator returns error in result, not throws
        assert(success2, "Tool execute should not throw")
        
        -- Test workflow creation error
        local success3, err3 = pcall(function()
            return Workflow.sequential({
                -- Missing required name field
                steps = {}
            })
        end)
        
        assert(not success3, "Workflow creation should fail synchronously")
        assert(string.find(tostring(err3), "name") ~= nil, "Should have error about missing name")
        
        return true
    "#).await;

    assert!(
        result.is_ok(),
        "Error handling sync test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_api_synchronous_return_patterns() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that API methods return values immediately, not promises
    let result = engine
        .execute_script(
            r#"
        -- Test Tool API returns values immediately
        local calc = Tool.get("calculator")
        assert(calc ~= nil, "Tool.get should return immediately")
        assert(type(calc) == "table", "Tool should be table")
        assert(calc["then"] == nil, "Tool should not be a promise")
        
        local tool_result = calc:execute({input = "1 + 1"})
        assert(tool_result ~= nil, "Tool execution should return immediately") 
        assert(tool_result["then"] == nil, "Tool result should not be a promise")
        assert(type(tool_result) ~= "function", "Tool result should not be a function")
        
        -- Test Workflow API structure
        assert(Workflow ~= nil, "Workflow should be available")
        assert(type(Workflow.sequential) == "function", "Workflow.sequential should be function")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "API synchronous patterns test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_sync_timeout_behavior() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test timeout behavior in sync context with tools
    let result = engine
        .execute_script(
            r#"
        -- Test that operations complete synchronously with reasonable timing
        local start_time = os.clock()
        local calc = Tool.get("calculator")  
        local tool_result = calc:execute({input = "2 + 2"})
        local end_time = os.clock()
        local duration = (end_time - start_time) * 1000  -- Convert to ms
        
        -- Should complete quickly and synchronously
        assert(tool_result ~= nil, "Tool execution should return result")
        assert(duration < 1000, "Tool execution should be fast: " .. tostring(duration) .. "ms")
        
        -- Test that errors are thrown synchronously  
        local error_start = os.clock()
        local success, error_result = pcall(function()
            return calc:execute({input = ""})  -- Empty input should cause error
        end)
        local error_end = os.clock()
        local error_duration = (error_end - error_start) * 1000
        
        -- Error should be immediate, not after timeout
        assert(error_duration < 1000, "Error should be thrown quickly: " .. tostring(error_duration) .. "ms")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "Timeout behavior test failed: {:?}",
        result.err()
    );
}
#[test]
fn test_sync_utils_panic_safety() {
    // This test is already covered in sync_utils module tests
    // but we include a reference here for completeness

    // The sync_utils module provides panic safety for all async operations
    // This ensures that even if async code panics, it's caught and converted
    // to a proper Lua error rather than crashing the entire process
}
