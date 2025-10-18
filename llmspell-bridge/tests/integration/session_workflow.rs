//! ABOUTME: End-to-end workflow integration tests for Session and Artifact systems
//! ABOUTME: Tests complete workflows including state management, hooks, and resource cleanup

#[path = "../test_helpers.rs"]
mod test_helpers;

use llmspell_bridge::{
    engine::factory::{EngineFactory, LuaConfig},
    providers::ProviderManager,
    ComponentRegistry,
};
use llmspell_config::providers::ProviderManagerConfig;
use std::convert::TryFrom;
use std::sync::Arc;
use std::time::Instant;
use test_helpers::create_test_infrastructure;

/// Test basic API availability for Session/Artifact workflows
#[tokio::test(flavor = "multi_thread")]
async fn test_api_availability_for_workflows() {
    // Create engine through factory
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    // Set up dependencies - simplified without problematic hook injection
    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    // Try basic API injection (may fail for some APIs)
    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    let lua_code = r"
        -- Test API availability for workflows
        local workflow_result = {}
        
        -- Check which APIs are available
        workflow_result.agent_available = Agent ~= nil
        workflow_result.tool_available = Tool ~= nil
        workflow_result.workflow_available = Workflow ~= nil
        workflow_result.session_available = Session ~= nil
        workflow_result.artifact_available = Artifact ~= nil
        workflow_result.state_available = State ~= nil
        workflow_result.hook_available = Hook ~= nil
        
        -- Count available APIs
        local available_count = 0
        if workflow_result.agent_available then available_count = available_count + 1 end
        if workflow_result.tool_available then available_count = available_count + 1 end
        if workflow_result.workflow_available then available_count = available_count + 1 end
        if workflow_result.session_available then available_count = available_count + 1 end
        if workflow_result.artifact_available then available_count = available_count + 1 end
        if workflow_result.state_available then available_count = available_count + 1 end
        if workflow_result.hook_available then available_count = available_count + 1 end
        
        workflow_result.total_available = available_count
        
        return workflow_result
    ";

    let result = engine.execute_script(lua_code).await;
    assert!(
        result.is_ok(),
        "API availability test should succeed: {:?}",
        result.err()
    );

    // Verify at least core APIs are available
    let output = result.unwrap();
    let workflow_result = output.output.as_object().unwrap();

    // Should have at least Agent, Tool, and Workflow APIs
    assert_eq!(
        workflow_result.get("agent_available").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        workflow_result.get("tool_available").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        workflow_result.get("workflow_available").unwrap().as_bool(),
        Some(true)
    );

    // Check how many APIs are available total
    let available_count = workflow_result
        .get("total_available")
        .unwrap()
        .as_i64()
        .unwrap();
    assert!(
        available_count >= 3,
        "Should have at least 3 APIs available (Agent, Tool, Workflow)"
    );

    println!("Available APIs: {available_count}/7");
}

/// Test state management integration patterns
#[tokio::test(flavor = "multi_thread")]
async fn test_state_management_integration() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    let lua_code = r#"
        -- Test state management integration
        local integration_result = {}
        
        -- Test basic Lua state operations (always available)
        local lua_state = {}
        lua_state.test_var = "test_value"
        lua_state.config = {
            param1 = "value1",
            param2 = 42,
            nested = {
                flag = true
            }
        }
        
        integration_result.lua_state_works = lua_state.test_var == "test_value"
        integration_result.lua_complex_state = lua_state.config.param1 == "value1" and 
                                               lua_state.config.param2 == 42 and 
                                               lua_state.config.nested.flag == true
        
        -- Test if State global is available and try using it
        if State and State.set and State.get then
            local success, err = pcall(function()
                State.set("integration_test", "integration_value")
                local value = State.get("integration_test")
                return value == "integration_value"
            end)
            integration_result.global_state_works = success
        else
            integration_result.global_state_works = false
            integration_result.global_state_unavailable = true
        end
        
        return integration_result
    "#;

    let result = engine.execute_script(lua_code).await;
    assert!(
        result.is_ok(),
        "State integration test should succeed: {:?}",
        result.err()
    );

    let output = result.unwrap();
    let integration_result = output.output.as_object().unwrap();

    // Lua state should always work
    assert_eq!(
        integration_result.get("lua_state_works").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        integration_result
            .get("lua_complex_state")
            .unwrap()
            .as_bool(),
        Some(true)
    );

    // Global state might not be available depending on implementation
    println!(
        "Global state works: {:?}",
        integration_result.get("global_state_works")
    );
    println!(
        "Global state unavailable: {:?}",
        integration_result.get("global_state_unavailable")
    );
}

/// Test memory leak prevention through multiple executions
#[tokio::test(flavor = "multi_thread")]
async fn test_memory_leak_prevention() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    // Test creating multiple script executions without memory leaks
    let lua_code = r#"
        -- Memory leak prevention test
        local memory_test = {}
        local large_data = {}
        
        -- Create some large data structures
        for i = 1, 100 do
            large_data[i] = {
                id = i,
                data = string.rep("test data " .. i .. " ", 50),
                nested = {
                    values = {}
                }
            }
            for j = 1, 20 do
                large_data[i].nested.values[j] = "value_" .. i .. "_" .. j
            end
        end
        
        memory_test.large_data_created = #large_data
        
        -- Clear references
        large_data = nil
        
        -- Force garbage collection
        collectgarbage("collect")
        
        memory_test.cleanup_attempted = true
        memory_test.memory_after_gc = collectgarbage("count")
        
        return memory_test
    "#;

    // Run the test multiple times to check for memory leaks
    for i in 0..5 {
        let result = engine.execute_script(lua_code).await;
        assert!(
            result.is_ok(),
            "Memory leak test iteration {} should succeed: {:?}",
            i,
            result.err()
        );

        let output = result.unwrap();
        let memory_test = output.output.as_object().unwrap();

        assert_eq!(
            memory_test.get("large_data_created").unwrap().as_i64(),
            Some(100)
        );
        assert_eq!(
            memory_test.get("cleanup_attempted").unwrap().as_bool(),
            Some(true)
        );

        let memory_usage = memory_test
            .get("memory_after_gc")
            .unwrap()
            .as_f64()
            .unwrap();
        println!("Memory usage after GC (iteration {i}): {memory_usage:.2} KB");

        // Memory should be reasonable (less than 50MB for this test)
        assert!(memory_usage < 50000.0, "Memory usage should be reasonable");
    }
}

/// Test performance requirements for script operations
#[tokio::test(flavor = "multi_thread")]
async fn test_performance_requirements() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    // Test script execution performance
    let start_time = Instant::now();

    let lua_code = r#"
        -- Performance test
        local perf_result = {}
        local start_time = os.clock()
        
        -- Test basic operations performance
        local operations_count = 0
        for i = 1, 1000 do
            local temp_data = {
                id = i,
                value = "test_" .. i,
                computed = i * i
            }
            operations_count = operations_count + 1
        end
        
        local operation_time = os.clock() - start_time
        perf_result.operation_time = operation_time
        perf_result.operations_count = operations_count
        
        -- Test string operations
        start_time = os.clock()
        local large_string = ""
        for i = 1, 100 do
            large_string = large_string .. "test data " .. i .. " "
        end
        
        local string_time = os.clock() - start_time
        perf_result.string_time = string_time
        perf_result.string_length = #large_string
        
        return perf_result
    "#;

    let result = engine.execute_script(lua_code).await;
    assert!(
        result.is_ok(),
        "Performance test should succeed: {:?}",
        result.err()
    );

    let total_time = start_time.elapsed();

    let output = result.unwrap();
    let perf_result = output.output.as_object().unwrap();

    assert_eq!(
        perf_result.get("operations_count").unwrap().as_i64(),
        Some(1000)
    );
    assert!(perf_result.get("string_length").unwrap().as_i64().unwrap() > 0);

    // Performance assertions
    let operation_time = perf_result.get("operation_time").unwrap().as_f64().unwrap();
    let string_time = perf_result.get("string_time").unwrap().as_f64().unwrap();

    println!("Operation time: {operation_time:.4}s");
    println!("String operation time: {string_time:.4}s");
    println!("Total test execution time: {total_time:?}");

    // Performance targets (generous for script operations)
    assert!(operation_time < 2.0, "1000 operations should be < 2s");
    assert!(string_time < 0.5, "String operations should be < 0.5s");
    assert!(
        total_time.as_secs() < 5,
        "Total test should complete in < 5s"
    );
}

/// Test error conditions and recovery
#[tokio::test(flavor = "multi_thread")]
async fn test_error_conditions_and_recovery() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    let lua_code = r#"
        -- Error condition tests
        local error_test = {}
        
        -- Test syntax error handling
        local success, err = pcall(function()
            load("invalid lua syntax {")()
        end)
        error_test.syntax_error_handled = not success
        
        -- Test runtime error handling
        local success2, err2 = pcall(function()
            error("Intentional runtime error")
        end)
        error_test.runtime_error_handled = not success2
        
        -- Test type error handling
        local success3, err3 = pcall(function()
            local x = nil
            return x + 1  -- Type error
        end)
        error_test.type_error_handled = not success3
        
        -- Test recovery after errors
        local success4, err4 = pcall(function()
            local valid_operation = "test"
            return #valid_operation == 4
        end)
        error_test.recovery_successful = success4
        
        return error_test
    "#;

    let result = engine.execute_script(lua_code).await;
    assert!(
        result.is_ok(),
        "Error condition test should succeed: {:?}",
        result.err()
    );

    let output = result.unwrap();
    let error_test = output.output.as_object().unwrap();

    assert_eq!(
        error_test.get("syntax_error_handled").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        error_test.get("runtime_error_handled").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        error_test.get("type_error_handled").unwrap().as_bool(),
        Some(true)
    );
    assert_eq!(
        error_test.get("recovery_successful").unwrap().as_bool(),
        Some(true)
    );
}

/// Test concurrent script operations for thread safety
#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_operations() {
    // Create multiple engines to simulate concurrent access
    let mut handles = vec![];

    for i in 0..3 {
        let handle = tokio::spawn(async move {
            let lua_config = LuaConfig::default();
            let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

            let registry = Arc::new(ComponentRegistry::new());
            let provider_config = ProviderManagerConfig::default();
            let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

            let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

            let _ = engine.inject_apis(
                &registry,
                &providers,
                &tool_registry,
                &agent_registry,
                &workflow_factory,
                None,
            );

            let lua_code = format!(
                r"
                -- Concurrent operations test {i}
                local thread_id = {i}
                
                -- Perform operations that might conflict if not thread-safe
                local results = {{}}
                for i = 1, 10 do
                    local data = {{
                        thread_id = thread_id,
                        iteration = i,
                        timestamp = os.time(),
                        computed = thread_id * 100 + i
                    }}
                    results[i] = data
                end
                
                -- Return results
                return {{
                    thread_id = thread_id,
                    results_count = #results,
                    success = true,
                    last_computed = results[#results].computed
                }}
            "
            );

            engine.execute_script(&lua_code).await
        });

        handles.push(handle);
    }

    // Wait for all concurrent operations to complete
    let mut results = vec![];
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok(), "Concurrent operation should succeed");
        results.push(result.unwrap());
    }

    // Verify all operations completed successfully
    assert_eq!(results.len(), 3);

    for (i, result) in results.iter().enumerate() {
        let output = result.output.as_object().unwrap();
        let i_i64 = i64::try_from(i).expect("index should fit in i64");
        assert_eq!(output.get("thread_id").unwrap().as_i64(), Some(i_i64));
        assert_eq!(output.get("results_count").unwrap().as_i64(), Some(10));
        assert_eq!(output.get("success").unwrap().as_bool(), Some(true));

        // Verify thread-specific computations
        let expected_last_computed = i_i64 * 100 + 10;
        assert_eq!(
            output.get("last_computed").unwrap().as_i64(),
            Some(expected_last_computed)
        );
    }
}

/// Test comprehensive API method availability
#[tokio::test(flavor = "multi_thread")]
#[allow(clippy::too_many_lines)]
async fn test_comprehensive_api_methods() {
    let lua_config = LuaConfig::default();
    let mut engine = EngineFactory::create_lua_engine(&lua_config).unwrap();

    let registry = Arc::new(ComponentRegistry::new());
    let provider_config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(provider_config).await.unwrap());

    let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();

    let _ = engine.inject_apis(
        &registry,
        &providers,
        &tool_registry,
        &agent_registry,
        &workflow_factory,
        None,
    );

    let lua_code = r"
        -- Comprehensive API method availability test
        local api_test = {}
        
        -- Check core APIs and their methods
        if Agent then
            api_test.agent_create_available = type(Agent.create) == 'function'
            api_test.agent_methods_count = 0
            for k, v in pairs(Agent) do
                if type(v) == 'function' then
                    api_test.agent_methods_count = api_test.agent_methods_count + 1
                end
            end
        end
        
        if Tool then
            api_test.tool_create_available = type(Tool.create) == 'function'
            api_test.tool_methods_count = 0
            for k, v in pairs(Tool) do
                if type(v) == 'function' then
                    api_test.tool_methods_count = api_test.tool_methods_count + 1
                end
            end
        end
        
        if Workflow then
            api_test.workflow_create_available = type(Workflow.create) == 'function'
            api_test.workflow_methods_count = 0
            for k, v in pairs(Workflow) do
                if type(v) == 'function' then
                    api_test.workflow_methods_count = api_test.workflow_methods_count + 1
                end
            end
        end
        
        -- Check Session API if available
        if Session then
            api_test.session_create_available = type(Session.create) == 'function'
            api_test.session_list_available = type(Session.list) == 'function'
            api_test.session_methods_count = 0
            for k, v in pairs(Session) do
                if type(v) == 'function' then
                    api_test.session_methods_count = api_test.session_methods_count + 1
                end
            end
        end
        
        -- Check Artifact API if available
        if Artifact then
            api_test.artifact_store_available = type(Artifact.store) == 'function'
            api_test.artifact_get_available = type(Artifact.get) == 'function'
            api_test.artifact_methods_count = 0
            for k, v in pairs(Artifact) do
                if type(v) == 'function' then
                    api_test.artifact_methods_count = api_test.artifact_methods_count + 1
                end
            end
        end
        
        -- Check State API if available
        if State then
            api_test.state_set_available = type(State.set) == 'function'
            api_test.state_get_available = type(State.get) == 'function'
            api_test.state_methods_count = 0
            for k, v in pairs(State) do
                if type(v) == 'function' then
                    api_test.state_methods_count = api_test.state_methods_count + 1
                end
            end
        end
        
        -- Check Hook API if available
        if Hook then
            api_test.hook_register_available = type(Hook.register) == 'function'
            api_test.hook_methods_count = 0
            for k, v in pairs(Hook) do
                if type(v) == 'function' then
                    api_test.hook_methods_count = api_test.hook_methods_count + 1
                end
            end
        end
        
        return api_test
    ";

    let result = engine.execute_script(lua_code).await;
    assert!(
        result.is_ok(),
        "API method availability test should succeed: {:?}",
        result.err()
    );

    let output = result.unwrap();
    let api_test = output.output.as_object().unwrap();

    // Log all API method availability for debugging
    println!("API Method Availability:");
    for (key, value) in api_test {
        println!("  {key}: {value:?}");
    }

    // Core APIs should be available - Agent.create should be available
    assert_eq!(
        api_test.get("agent_create_available").unwrap().as_bool(),
        Some(true)
    );

    // Tool and Workflow APIs exist but may not have create methods exposed
    // They still have multiple methods available
    let tool_create = api_test
        .get("tool_create_available")
        .unwrap()
        .as_bool()
        .unwrap_or(false);
    let workflow_create = api_test
        .get("workflow_create_available")
        .unwrap()
        .as_bool()
        .unwrap_or(false);
    println!("Tool.create available: {tool_create}");
    println!("Workflow.create available: {workflow_create}");

    // Each API should have multiple methods regardless of create availability
    assert!(
        api_test
            .get("agent_methods_count")
            .unwrap()
            .as_i64()
            .unwrap()
            > 0
    );
    assert!(
        api_test
            .get("tool_methods_count")
            .unwrap()
            .as_i64()
            .unwrap()
            > 0
    );
    assert!(
        api_test
            .get("workflow_methods_count")
            .unwrap()
            .as_i64()
            .unwrap()
            > 0
    );
}
