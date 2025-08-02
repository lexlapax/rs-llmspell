//! ABOUTME: Comprehensive tests for enhanced Lua Hook API functionality
//! ABOUTME: Tests Hook.register with priorities, Hook.unregister standalone, enhanced Hook.list filtering

use llmspell_bridge::globals::types::GlobalContext;
use llmspell_bridge::hook_bridge::HookBridge;
use llmspell_bridge::lua::globals::hook::inject_hook_global;
use llmspell_bridge::{ComponentRegistry, ProviderManager};
use mlua::Lua;
use std::sync::Arc;

async fn create_test_environment() -> (Lua, GlobalContext, Arc<HookBridge>) {
    let lua = Lua::new();
    let registry = Arc::new(ComponentRegistry::new());
    let providers = Arc::new(ProviderManager::new(Default::default()).await.unwrap());
    let context = GlobalContext::new(registry, providers);
    let hook_bridge = Arc::new(HookBridge::new(Arc::new(context.clone())).await.unwrap());

    inject_hook_global(&lua, &context, hook_bridge.clone()).unwrap();

    (lua, context, hook_bridge)
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_register_with_priorities() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Test registering hooks with different priorities
        local handle1 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "highest")
        local handle2 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "high") 
        local handle3 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "normal")
        local handle4 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "low")
        local handle5 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "lowest")
        
        -- Verify all handles are valid
        local all_valid = handle1:id() ~= nil and
                         handle2:id() ~= nil and
                         handle3:id() ~= nil and
                         handle4:id() ~= nil and
                         handle5:id() ~= nil
        
        -- Clean up
        handle1:unregister()
        handle2:unregister()
        handle3:unregister()
        handle4:unregister()
        handle5:unregister()
        
        return all_valid
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should register hooks with all priority levels"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_unregister_standalone() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Register a hook
        local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "normal")
        
        -- Verify it's registered
        local hooks_before = Hook.list("BeforeAgentInit")
        local count_before = #hooks_before
        
        -- Unregister using standalone function
        local unregistered = Hook.unregister(handle)
        
        -- Verify it's unregistered
        local hooks_after = Hook.list("BeforeAgentInit")
        local count_after = #hooks_after
        
        return unregistered and (count_after < count_before)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should unregister hook using standalone function"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_list_enhanced_filtering() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<(bool, String)> = lua
        .load(
            r#"
        -- Register hooks with different properties
        local handle1 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "high")
        local handle2 = Hook.register("BeforeAgentExecution", function(ctx) return "continue" end, "low")
        local handle3 = Hook.register("AfterAgentInit", function(ctx) return "continue" end, "normal")
        
        local debug_info = {}
        
        -- Test string filter (hook point)
        local init_hooks = Hook.list("BeforeAgentInit")
        local has_init_hook = #init_hooks > 0
        table.insert(debug_info, "init_hooks: " .. #init_hooks)
        
        -- Test table filter by language
        local lua_hooks = Hook.list({language = "lua"})
        local has_lua_hooks = #lua_hooks >= 3  -- Should have at least our 3 hooks
        table.insert(debug_info, "lua_hooks: " .. #lua_hooks)
        
        -- Test table filter by priority
        local high_hooks = Hook.list({priority = "high"})
        local has_high_hooks = #high_hooks > 0
        table.insert(debug_info, "high_hooks: " .. #high_hooks)
        
        -- Test combined filter
        local combined = Hook.list({
            hook_point = "BeforeAgentInit",
            language = "lua",
            priority = "high"
        })
        local has_combined = #combined > 0
        table.insert(debug_info, "combined: " .. #combined)
        
        -- Clean up
        Hook.unregister(handle1)
        Hook.unregister(handle2)
        Hook.unregister(handle3)
        
        local success = has_init_hook and has_lua_hooks and has_high_hooks and has_combined
        local debug_str = table.concat(debug_info, ", ")
        
        return success, debug_str
    "#,
        )
        .eval();

    match result {
        Ok((success, debug_info)) => {
            if !success {
                println!("Debug info: {debug_info}");
            }
            assert!(
                success,
                "Should filter hooks using enhanced filtering options. Debug: {debug_info}"
            );
        }
        Err(e) => panic!("Test failed with error: {e}"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_list_all_hooks() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Register several hooks
        local handle1 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
        local handle2 = Hook.register("BeforeAgentExecution", function(ctx) return "continue" end)
        local handle3 = Hook.register("AfterAgentInit", function(ctx) return "continue" end)
        
        -- List all hooks (no filter)
        local all_hooks = Hook.list()
        local has_hooks = #all_hooks >= 3
        
        -- Verify hook structure
        local first_hook = all_hooks[1]
        local has_required_fields = first_hook.name ~= nil and
                                   first_hook.priority ~= nil and
                                   first_hook.language ~= nil and
                                   first_hook.version ~= nil and
                                   first_hook.tags ~= nil
        
        -- Clean up
        Hook.unregister(handle1)
        Hook.unregister(handle2)
        Hook.unregister(handle3)
        
        return has_hooks and has_required_fields
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should list all hooks with proper structure"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_context_data_modification() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<String> = lua
        .load(
            r#"
        -- Register a hook that modifies data
        local handle = Hook.register("BeforeAgentExecution", function(context)
            -- Return modified data
            return {
                type = "modified",
                data = {
                    original_prompt = context.data.prompt or "none",
                    enhanced_prompt = "Enhanced: " .. (context.data.prompt or "default"),
                    modification_source = "lua_hook"
                }
            }
        end, "high")
        
        -- The hook is registered successfully
        local hook_id = handle:id()
        
        -- Clean up
        Hook.unregister(handle)
        
        return hook_id
    "#,
        )
        .eval();

    assert!(
        result.is_ok(),
        "Should register hook with data modification capability"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_result_types() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Test different hook result types
        local results = {}
        
        -- Continue result
        local handle1 = Hook.register("BeforeAgentInit", function(ctx)
            return "continue"
        end)
        table.insert(results, handle1:id() ~= nil)
        
        -- Cancel result
        local handle2 = Hook.register("BeforeAgentExecution", function(ctx)
            return {
                type = "cancel",
                reason = "Cancelled for testing"
            }
        end)
        table.insert(results, handle2:id() ~= nil)
        
        -- Modified result
        local handle3 = Hook.register("AfterAgentInit", function(ctx)
            return {
                type = "modified",
                data = {test = "modified data"}
            }
        end)
        table.insert(results, handle3:id() ~= nil)
        
        -- Redirect result
        local handle4 = Hook.register("BeforeToolExecution", function(ctx)
            return {
                type = "redirect",
                target = "alternative_tool"
            }
        end)
        table.insert(results, handle4:id() ~= nil)
        
        -- Replace result
        local handle5 = Hook.register("ToolError", function(ctx)
            return {
                type = "replace",
                data = {replacement = "error handled"}
            }
        end)
        table.insert(results, handle5:id() ~= nil)
        
        -- Retry result
        local handle6 = Hook.register("AgentError", function(ctx)
            return {
                type = "retry",
                delay_ms = 1000,
                max_attempts = 3
            }
        end)
        table.insert(results, handle6:id() ~= nil)
        
        -- Clean up
        Hook.unregister(handle1)
        Hook.unregister(handle2)
        Hook.unregister(handle3)
        Hook.unregister(handle4)
        Hook.unregister(handle5)
        Hook.unregister(handle6)
        
        -- Check all registrations succeeded
        local all_successful = true
        for _, success in ipairs(results) do
            all_successful = all_successful and success
        end
        
        return all_successful
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should support all hook result types"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_invalid_hook_operations() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Test invalid hook point
        local success1, error1 = pcall(function()
            Hook.register("InvalidHookPoint", function(ctx) return "continue" end)
        end)
        
        -- Test double unregistration
        local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
        local first_unregister = handle:unregister()
        local second_unregister = handle:unregister()  -- Should return false
        
        -- Test unregistering invalid handle (create a dummy userdata)
        local success3, error3 = pcall(function()
            Hook.unregister("not_a_handle")
        end)
        
        return (not success1) and first_unregister and (not second_unregister) and (not success3)
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should handle invalid operations gracefully"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_hook_handle_methods() {
    let (lua, _context, _bridge) = create_test_environment().await;

    let result: mlua::Result<bool> = lua
        .load(
            r#"
        -- Register a hook
        local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "high")
        
        -- Test handle methods
        local hook_id = handle:id()
        local hook_point = handle:hook_point()
        
        -- Verify values
        local has_id = hook_id ~= nil and type(hook_id) == "string"
        local has_hook_point = hook_point ~= nil and type(hook_point) == "string"
        local correct_hook_point = hook_point:find("BeforeAgentInit") ~= nil
        
        -- Clean up
        local unregistered = handle:unregister()
        
        return has_id and has_hook_point and correct_hook_point and unregistered
    "#,
        )
        .eval();

    assert!(
        result.is_ok() && result.unwrap(),
        "Should provide handle methods for introspection"
    );
}
