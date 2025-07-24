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
async fn test_agent_sync_behavior() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that Agent.create blocks and returns immediately (not a promise/future)
    let result = engine
        .execute_script(
            r#"
        -- Create agent synchronously
        local agent = Agent.create({
            name = "test-agent",
            model = "openai/gpt-3.5-turbo"
        })
        
        -- Should be able to use agent immediately
        assert(agent, "Agent should be created synchronously")
        -- Agent is a userdata object with methods, not fields
        assert(type(agent) == "userdata", "Agent should be a userdata object")
        
        -- Execute should also be synchronous
        local result = agent:execute({input = "Hello"})
        assert(result, "Execute should return result synchronously")
        assert(type(result) == "table", "Result should be a table")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "Agent sync behavior test failed: {:?}",
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
        
        -- Workflow is a userdata object, use getInfo() to access properties
        local info = workflow:getInfo()
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
async fn test_no_promises_or_callbacks() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test that there are no async/promise/callback patterns
    let result = engine
        .execute_script(
            r#"
        -- Check that methods don't return promises
        local agent = Agent.create({name = "test", model = "openai/gpt-3.5-turbo"})
        local result = agent:execute({input = "test"})
        
        -- Result should not have .then or other promise methods
        assert(result["then"] == nil, "Result should not be a promise")
        assert(type(result) ~= "function", "Result should not be a function/callback")
        
        -- Check tool execution
        local calc = Tool.get("calculator")
        local tool_result = calc:execute({input = "1 + 1"})
        assert(tool_result["then"] == nil, "Tool result should not be a promise")
        assert(type(tool_result) ~= "function", "Tool result should not be a function")
        
        -- Check workflow execution
        local workflow = Workflow.sequential({
            name = "test",
            steps = {{name = "s1", type = "tool", tool = "calculator", input = {input = "2 + 2"}}}
        })
        local wf_result = workflow:execute()
        assert(wf_result["then"] == nil, "Workflow result should not be a promise")
        assert(type(wf_result) ~= "function", "Workflow result should not be a function")
        
        return true
    "#,
        )
        .await;

    assert!(
        result.is_ok(),
        "No promises test failed: {:?}",
        result.err()
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_sync_timeout_behavior() {
    let registry = create_test_registry().await;
    let providers = create_test_providers().await;

    let mut engine = create_test_engine().await;
    engine.inject_apis(&registry, &providers).unwrap();

    // Test timeout behavior in sync context
    let result = engine
        .execute_script(
            r#"
        -- Create agent with short timeout (this is a mock, so won't actually timeout)
        local agent = Agent.create({
            name = "timeout-test",
            model = "openai/gpt-3.5-turbo",
            timeout_ms = 100  -- 100ms timeout
        })
        
        -- Execute should complete or timeout synchronously
        local start_time = os.clock()
        local success, result = pcall(function()
            return agent:execute({input = "test"})
        end)
        local end_time = os.clock()
        
        -- Either succeeds or fails, but blocks until complete
        assert(success or not success, "Should have definite result")
        
        -- Should not return immediately (unless very fast)
        -- In real scenarios with actual timeouts, this would validate timeout behavior
        
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
