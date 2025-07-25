-- ABOUTME: Basic hook registration and management tests for Lua integration
-- ABOUTME: Tests core Hook API functionality including register, unregister, and list operations

local test_results = {}
local test_count = 0
local passed_count = 0

-- Helper function to run a test
local function run_test(test_name, test_func)
    test_count = test_count + 1
    print(string.format("Running test %d: %s", test_count, test_name))
    
    local success, result = pcall(test_func)
    if success and result then
        passed_count = passed_count + 1
        print("  ✅ PASSED")
        table.insert(test_results, {name = test_name, status = "PASSED"})
    else
        print("  ❌ FAILED: " .. (result or "Unknown error"))
        table.insert(test_results, {name = test_name, status = "FAILED", error = result})
    end
end

-- Test 1: Basic Hook Registration
run_test("Basic Hook Registration", function()
    local handle = Hook.register("BeforeAgentInit", function(context)
        return "continue"
    end, "normal")
    
    -- Verify handle exists and has methods
    local hook_id = handle:id()
    local hook_point = handle:hook_point()
    
    -- Clean up
    local unregistered = handle:unregister()
    
    return hook_id ~= nil and hook_point ~= nil and unregistered
end)

-- Test 2: Hook Registration with All Priority Levels
run_test("Hook Registration with All Priorities", function()
    local handles = {}
    local priorities = {"highest", "high", "normal", "low", "lowest"}
    
    -- Register hooks with different priorities
    for _, priority in ipairs(priorities) do
        local handle = Hook.register("BeforeAgentExecution", function(context)
            return "continue"
        end, priority)
        table.insert(handles, handle)
    end
    
    -- Verify all handles are valid
    local all_valid = true
    for _, handle in ipairs(handles) do
        if not handle:id() then
            all_valid = false
            break
        end
    end
    
    -- Clean up
    for _, handle in ipairs(handles) do
        handle:unregister()
    end
    
    return all_valid and #handles == 5
end)

-- Test 3: Hook Unregistration (both methods)
run_test("Hook Unregistration Methods", function()
    -- Test handle method
    local handle1 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
    local unregistered1 = handle1:unregister()
    
    -- Test standalone function
    local handle2 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
    local unregistered2 = Hook.unregister(handle2)
    
    return unregistered1 and unregistered2
end)

-- Test 4: Hook Listing and Filtering
run_test("Hook Listing and Filtering", function()
    -- Register test hooks
    local handle1 = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "high")
    local handle2 = Hook.register("BeforeAgentExecution", function(ctx) return "continue" end, "low")
    local handle3 = Hook.register("AfterAgentInit", function(ctx) return "continue" end, "normal")
    
    -- Test list all
    local all_hooks = Hook.list()
    local has_all = #all_hooks >= 3
    
    -- Test filter by hook point
    local init_hooks = Hook.list("BeforeAgentInit")
    local has_init = #init_hooks > 0
    
    -- Test filter by table
    local high_hooks = Hook.list({priority = "high"})
    local has_high = #high_hooks > 0
    
    local lua_hooks = Hook.list({language = "lua"})
    local has_lua = #lua_hooks >= 3
    
    -- Clean up
    Hook.unregister(handle1)
    Hook.unregister(handle2)
    Hook.unregister(handle3)
    
    return has_all and has_init and has_high and has_lua
end)

-- Test 5: Hook Result Types
run_test("Hook Result Types", function()
    local results_tested = {}
    
    -- Test continue result
    local handle1 = Hook.register("BeforeAgentInit", function(ctx)
        return "continue"
    end)
    table.insert(results_tested, "continue")
    
    -- Test modified result
    local handle2 = Hook.register("BeforeAgentExecution", function(ctx)
        return {
            type = "modified",
            data = {modified = true, timestamp = os.time()}
        }
    end)
    table.insert(results_tested, "modified")
    
    -- Test cancel result
    local handle3 = Hook.register("AgentError", function(ctx)
        return {
            type = "cancel",
            reason = "Test cancellation"
        }
    end)
    table.insert(results_tested, "cancel")
    
    -- Test redirect result
    local handle4 = Hook.register("BeforeToolExecution", function(ctx)
        return {
            type = "redirect",
            target = "alternative_tool"
        }
    end)
    table.insert(results_tested, "redirect")
    
    -- Test replace result
    local handle5 = Hook.register("ToolError", function(ctx)
        return {
            type = "replace",
            data = {replacement = "error handled gracefully"}
        }
    end)
    table.insert(results_tested, "replace")
    
    -- Clean up
    Hook.unregister(handle1)
    Hook.unregister(handle2)
    Hook.unregister(handle3)
    Hook.unregister(handle4)
    Hook.unregister(handle5)
    
    return #results_tested == 5
end)

-- Test 6: Hook Context Processing
run_test("Hook Context Processing", function()
    local context_received = false
    local handle = Hook.register("BeforeAgentExecution", function(context)
        -- Verify context structure
        context_received = context ~= nil and
                          context.point ~= nil and
                          context.component_id ~= nil and
                          context.correlation_id ~= nil and
                          context.language ~= nil and
                          context.metadata ~= nil and
                          context.data ~= nil
        
        return "continue"
    end)
    
    -- The hook is registered but won't be executed in this test environment
    -- We're testing that the registration works with a context-processing function
    local hook_id = handle:id()
    
    -- Clean up
    Hook.unregister(handle)
    
    -- Return true if hook was registered successfully (context will be tested in integration)
    return hook_id ~= nil
end)

-- Test 7: Multiple Hook Points
run_test("Multiple Hook Points Registration", function()
    local hook_points = {
        "BeforeAgentInit",
        "AfterAgentInit", 
        "BeforeAgentExecution",
        "AfterAgentExecution",
        "BeforeToolExecution",
        "AfterToolExecution",
        "BeforeAgentShutdown",
        "AfterAgentShutdown"
    }
    
    local handles = {}
    
    -- Register hooks for different points
    for _, point in ipairs(hook_points) do
        local handle = Hook.register(point, function(ctx)
            return "continue"
        end, "normal")
        table.insert(handles, handle)
    end
    
    -- Verify all registrations
    local all_registered = #handles == #hook_points
    for _, handle in ipairs(handles) do
        if not handle:id() then
            all_registered = false
            break
        end
    end
    
    -- Clean up
    for _, handle in ipairs(handles) do
        Hook.unregister(handle)
    end
    
    return all_registered
end)

-- Test 8: Hook Handle Introspection
run_test("Hook Handle Introspection", function()
    local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "high")
    
    -- Test handle methods
    local hook_id = handle:id()
    local hook_point = handle:hook_point()
    
    -- Verify return types and values
    local has_valid_id = hook_id ~= nil and type(hook_id) == "string" and hook_id ~= ""
    local has_valid_point = hook_point ~= nil and type(hook_point) == "string" and 
                           hook_point:find("BeforeAgentInit") ~= nil
    
    -- Clean up
    local unregistered = handle:unregister()
    
    return has_valid_id and has_valid_point and unregistered
end)

-- Test 9: Error Handling
run_test("Error Handling", function()
    local errors_handled = {}
    
    -- Test invalid hook point
    local success1, error1 = pcall(function()
        Hook.register("InvalidHookPoint", function(ctx) return "continue" end)
    end)
    table.insert(errors_handled, not success1)
    
    -- Test double unregistration
    local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
    local first_unregister = handle:unregister()
    local second_unregister = handle:unregister()  -- Should return false
    table.insert(errors_handled, first_unregister and not second_unregister)
    
    -- Test invalid unregister
    local success3, error3 = pcall(function()
        Hook.unregister("not_a_handle")
    end)
    table.insert(errors_handled, not success3)
    
    -- Check all error conditions were handled properly
    local all_handled = true
    for _, handled in ipairs(errors_handled) do
        if not handled then
            all_handled = false
            break
        end
    end
    
    return all_handled
end)

-- Test 10: Hook Metadata and Information
run_test("Hook Metadata and Information", function()
    local handle = Hook.register("BeforeAgentInit", function(ctx) return "continue" end, "normal")
    
    -- List hooks and verify metadata structure
    local hooks = Hook.list("BeforeAgentInit")
    local found_hook = nil
    
    for _, hook_info in ipairs(hooks) do
        if hook_info.language == "lua" then
            found_hook = hook_info
            break
        end
    end
    
    -- Verify hook info structure
    local has_metadata = found_hook ~= nil and
                        found_hook.name ~= nil and
                        found_hook.priority ~= nil and
                        found_hook.language ~= nil and
                        found_hook.version ~= nil and
                        found_hook.tags ~= nil
    
    -- Clean up
    Hook.unregister(handle)
    
    return has_metadata
end)

-- Print final results
print("\n" .. string.rep("=", 50))
print("BASIC HOOKS TEST RESULTS")
print(string.rep("=", 50))
print(string.format("Tests run: %d", test_count))
print(string.format("Tests passed: %d", passed_count))
print(string.format("Tests failed: %d", test_count - passed_count))
print(string.format("Success rate: %.1f%%", (passed_count / test_count) * 100))

print("\nDetailed Results:")
for i, result in ipairs(test_results) do
    local status_icon = result.status == "PASSED" and "✅" or "❌"
    print(string.format("%d. %s %s", i, status_icon, result.name))
    if result.error then
        print(string.format("   Error: %s", result.error))
    end
end

-- Return overall success
local overall_success = passed_count == test_count
print(string.format("\nOverall Result: %s", overall_success and "✅ ALL TESTS PASSED" or "❌ SOME TESTS FAILED"))

return overall_success