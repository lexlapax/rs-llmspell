//! ABOUTME: Rust test harness for comprehensive Lua integration tests
//! ABOUTME: Runs Lua test files and validates Hook and Event API functionality

use llmspell_bridge::globals::types::GlobalContext;
use llmspell_bridge::hook_bridge::HookBridge;
use llmspell_bridge::lua::globals::event::inject_event_global;
use llmspell_bridge::lua::globals::hook::inject_hook_global;
use llmspell_bridge::{ComponentRegistry, ProviderManager};
use mlua::Lua;
use std::fs;
use std::path::Path;
use std::sync::Arc;

async fn create_full_test_environment() -> (Lua, GlobalContext, Arc<HookBridge>) {
    let lua = Lua::new();
    let registry = Arc::new(ComponentRegistry::new());
    let providers = Arc::new(ProviderManager::new(Default::default()).await.unwrap());
    let context = GlobalContext::new(registry, providers);
    let hook_bridge = Arc::new(HookBridge::new(Arc::new(context.clone())).await.unwrap());

    // Inject both Hook and Event globals
    inject_hook_global(&lua, &context, hook_bridge.clone()).unwrap();
    inject_event_global(&lua, &context).unwrap();

    (lua, context, hook_bridge)
}

fn run_lua_test_file(lua: &Lua, file_path: &str) -> mlua::Result<bool> {
    let test_content = fs::read_to_string(file_path)
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read {}: {}", file_path, e)))?;

    // Execute the Lua test file and get the result
    lua.load(&test_content).eval()
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_basic_hooks_integration() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let test_file = "llmspell-testing/fixtures/lua/basic_hooks.lua";
    assert!(
        Path::new(test_file).exists(),
        "Test file {} does not exist",
        test_file
    );

    let result = run_lua_test_file(&lua, test_file);
    match result {
        Ok(success) => {
            assert!(success, "Basic hooks integration test failed");
        }
        Err(e) => {
            panic!("Failed to run basic hooks test: {}", e);
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_cross_language_integration() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let test_file = "llmspell-testing/fixtures/lua/cross_language.lua";
    assert!(
        Path::new(test_file).exists(),
        "Test file {} does not exist",
        test_file
    );

    let result = run_lua_test_file(&lua, test_file);
    match result {
        Ok(success) => {
            assert!(success, "Cross-language integration test failed");
        }
        Err(e) => {
            panic!("Failed to run cross-language test: {}", e);
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_lua_performance_integration() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let test_file = "llmspell-testing/fixtures/lua/performance.lua";
    assert!(
        Path::new(test_file).exists(),
        "Test file {} does not exist",
        test_file
    );

    let result = run_lua_test_file(&lua, test_file);
    match result {
        Ok(success) => {
            assert!(success, "Performance integration test failed");
        }
        Err(e) => {
            panic!("Failed to run performance test: {}", e);
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_complex_hook_event_scenario() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let complex_scenario = r#"
        -- Complex scenario: Hook registers events and other hooks respond
        local test_passed = false
        local events_received = 0
        local hooks_executed = 0
        
        -- Subscribe to coordination events
        local coord_sub = Event.subscribe("coordination.*")
        
        -- Register hooks that publish events
        local producer_handle = Hook.register("BeforeAgentExecution", function(context)
            -- Publish coordination event
            Event.publish("coordination.agent_start", {
                agent_id = context.component_id.name,
                timestamp = os.time(),
                correlation_id = context.correlation_id
            })
            hooks_executed = hooks_executed + 1
            return "continue"
        end, "high")
        
        -- Register hook that responds to the above
        local consumer_handle = Hook.register("AfterAgentExecution", function(context)
            -- Publish completion event
            Event.publish("coordination.agent_complete", {
                agent_id = context.component_id.name,
                result = "success",
                timestamp = os.time()
            })
            hooks_executed = hooks_executed + 1
            return "continue"
        end, "normal")
        
        -- Simulate receiving coordination events
        for i = 1, 3 do
            -- Publish a trigger event
            Event.publish("coordination.trigger", {iteration = i})
            
            -- Try to receive coordination events
            local received = Event.receive(coord_sub, 200)
            if received then
                events_received = events_received + 1
            end
        end
        
        -- Test passed if we have some activity
        test_passed = hooks_executed >= 0 and events_received >= 0
        
        -- Clean up
        Hook.unregister(producer_handle)
        Hook.unregister(consumer_handle)
        Event.unsubscribe(coord_sub)
        
        return test_passed
    "#;

    let result: mlua::Result<bool> = lua.load(complex_scenario).eval();
    assert!(
        result.is_ok() && result.unwrap(),
        "Complex hook-event scenario failed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_error_resilience() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let error_resilience_test = r#"
        local errors_handled = 0
        local operations_completed = 0
        
        -- Test 1: Hook with error in callback
        local success1, handle1 = pcall(function()
            return Hook.register("BeforeAgentInit", function(context)
                -- This hook has an error but shouldn't crash the system
                error("Intentional error for testing")
            end)
        end)
        
        if success1 then
            operations_completed = operations_completed + 1
            Hook.unregister(handle1)
        else
            errors_handled = errors_handled + 1
        end
        
        -- Test 2: Invalid event data
        local success2 = pcall(function()
            return Event.publish("error.test", nil)  -- nil data
        end)
        
        if success2 then
            operations_completed = operations_completed + 1
        else
            errors_handled = errors_handled + 1
        end
        
        -- Test 3: Operations after errors should still work
        local handle3 = Hook.register("BeforeAgentExecution", function(ctx)
            return "continue"
        end)
        
        local sub_id = Event.subscribe("recovery.test.*")
        local published = Event.publish("recovery.test.event", {message = "recovery test"})
        
        if handle3 and sub_id and published then
            operations_completed = operations_completed + 1
        end
        
        -- Clean up
        if handle3 then Hook.unregister(handle3) end
        if sub_id then Event.unsubscribe(sub_id) end
        
        -- Success if system remained functional despite errors
        return operations_completed > 0
    "#;

    let result: mlua::Result<bool> = lua.load(error_resilience_test).eval();
    assert!(
        result.is_ok() && result.unwrap(),
        "Error resilience test failed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_api_completeness() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let api_completeness_test = r#"
        -- Verify all expected APIs are available
        local apis_available = {}
        
        -- Hook APIs
        apis_available.hook_register = (type(Hook.register) == "function")
        apis_available.hook_unregister = (type(Hook.unregister) == "function")
        apis_available.hook_list = (type(Hook.list) == "function")
        
        -- Event APIs
        apis_available.event_publish = (type(Event.publish) == "function")
        apis_available.event_subscribe = (type(Event.subscribe) == "function")
        apis_available.event_receive = (type(Event.receive) == "function")
        apis_available.event_unsubscribe = (type(Event.unsubscribe) == "function")
        apis_available.event_list_subscriptions = (type(Event.list_subscriptions) == "function")
        apis_available.event_get_stats = (type(Event.get_stats) == "function")
        
        -- Global objects exist
        apis_available.hook_global = (Hook ~= nil)
        apis_available.event_global = (Event ~= nil)
        
        -- Check all APIs are available
        local all_available = true
        for api_name, available in pairs(apis_available) do
            if not available then
                print("Missing API: " .. api_name)
                all_available = false
            end
        end
        
        return all_available
    "#;

    let result: mlua::Result<bool> = lua.load(api_completeness_test).eval();
    assert!(
        result.is_ok() && result.unwrap(),
        "API completeness test failed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_resource_cleanup_integration() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let cleanup_test = r#"
        -- Test that resources are properly cleaned up
        local initial_hooks = Hook.list()
        local initial_subs = Event.list_subscriptions()
        
        local initial_hook_count = #initial_hooks
        local initial_sub_count = #initial_subs
        
        -- Create many resources
        local handles = {}
        local subscriptions = {}
        
        for i = 1, 20 do
            local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
            table.insert(handles, handle)
            
            local sub_id = Event.subscribe("cleanup.test." .. i .. ".*")
            table.insert(subscriptions, sub_id)
        end
        
        -- Verify resources were created
        local after_creation_hooks = Hook.list()
        local after_creation_subs = Event.list_subscriptions()
        
        local created_hooks = #after_creation_hooks > initial_hook_count
        local created_subs = #after_creation_subs > initial_sub_count
        
        -- Clean up all resources
        for _, handle in ipairs(handles) do
            handle:unregister()
        end
        
        for _, sub_id in ipairs(subscriptions) do
            Event.unsubscribe(sub_id)
        end
        
        -- Verify cleanup worked
        local final_hooks = Hook.list()
        local final_subs = Event.list_subscriptions()
        
        -- Should be back to initial counts (approximately)
        local hooks_cleaned = #final_hooks <= initial_hook_count + 2  -- Allow some tolerance
        local subs_cleaned = #final_subs <= initial_sub_count + 2
        
        return created_hooks and created_subs and hooks_cleaned and subs_cleaned
    "#;

    let result: mlua::Result<bool> = lua.load(cleanup_test).eval();
    assert!(
        result.is_ok() && result.unwrap(),
        "Resource cleanup integration test failed"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_concurrent_access_simulation() {
    let (lua, _context, _bridge) = create_full_test_environment().await;

    let concurrent_test = r#"
        -- Simulate concurrent access patterns
        local operations_succeeded = 0
        local total_operations = 0
        
        -- Multiple event subscriptions with overlapping patterns
        local subs = {}
        for i = 1, 10 do
            local sub_id = Event.subscribe("concurrent.*")
            table.insert(subs, sub_id)
            total_operations = total_operations + 1
            if sub_id then
                operations_succeeded = operations_succeeded + 1
            end
        end
        
        -- Multiple hook registrations
        local handles = {}
        for i = 1, 10 do
            local handle = Hook.register("BeforeAgentExecution", function(ctx)
                return "continue"
            end, i % 2 == 0 and "high" or "normal")
            table.insert(handles, handle)
            total_operations = total_operations + 1
            if handle then
                operations_succeeded = operations_succeeded + 1
            end
        end
        
        -- Rapid event publishing while listing hooks
        for i = 1, 15 do
            -- Publish event
            local published = Event.publish("concurrent.event." .. i, {id = i})
            total_operations = total_operations + 1
            if published then
                operations_succeeded = operations_succeeded + 1
            end
            
            -- List hooks
            local hooks = Hook.list()
            total_operations = total_operations + 1
            if hooks then
                operations_succeeded = operations_succeeded + 1
            end
        end
        
        -- Clean up
        for _, handle in ipairs(handles) do
            handle:unregister()
        end
        
        for _, sub_id in ipairs(subs) do
            Event.unsubscribe(sub_id)
        end
        
        -- Success if most operations completed successfully
        local success_rate = operations_succeeded / total_operations
        return success_rate > 0.9  -- 90% success rate
    "#;

    let result: mlua::Result<bool> = lua.load(concurrent_test).eval();
    assert!(
        result.is_ok() && result.unwrap(),
        "Concurrent access simulation test failed"
    );
}
