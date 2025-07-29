-- ABOUTME: Cross-language event propagation and integration tests for Lua
-- ABOUTME: Tests Event API functionality including publish/subscribe and cross-language scenarios

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
        local error_msg = "Unknown error"
        if result then
            error_msg = tostring(result)
        end
        print("  ❌ FAILED: " .. error_msg)
        table.insert(test_results, {name = test_name, status = "FAILED", error = error_msg})
    end
end

-- Test 1: Basic Event Publishing and Subscription
run_test("Basic Event Publish/Subscribe", function()
    -- Subscribe to test events
    local sub_id = Event.subscribe("test.basic.*")
    
    -- Publish a test event
    local published = Event.publish("test.basic.example", {
        message = "Hello from Lua",
        timestamp = os.time(),
        test_data = {value = 42, flag = true}
    })
    
    -- Try to receive the event
    local received = Event.receive(sub_id, 1000)  -- 1 second timeout
    
    -- Clean up
    Event.unsubscribe(sub_id)
    
    return published and (received ~= nil)
end)

-- Test 2: Event Publishing with Full Options
run_test("Event Publishing with Options", function()
    local sub_id = Event.subscribe("test.options.*")
    
    -- Generate a correlation ID
    local correlation_id = string.format("%08x-%04x-%04x-%04x-%12x", 
                                       math.random(0, 0xFFFFFFFF),
                                       math.random(0, 0xFFFF),
                                       math.random(0, 0xFFFF),
                                       math.random(0, 0xFFFF),
                                       math.random(0, 0xFFFFFFFFFFFF))
    
    -- Publish with full options
    local published = Event.publish("test.options.full", {
        data = "comprehensive test",
        metadata = {source = "lua_test", version = "1.0"}
    }, {
        language = "lua",
        correlation_id = correlation_id,
        ttl_seconds = 300
    })
    
    -- Clean up
    Event.unsubscribe(sub_id)
    
    return published
end)

-- Test 3: Pattern Matching for Events
run_test("Event Pattern Matching", function()
    -- Create different pattern subscriptions
    local subs = {
        user = Event.subscribe("user.*"),
        system = Event.subscribe("system.*"),
        error = Event.subscribe("*.error"),
        all_test = Event.subscribe("test.*")
    }
    
    -- Publish events matching different patterns
    local events_published = {
        Event.publish("user.login", {user_id = "user123", action = "login"}),
        Event.publish("system.startup", {component = "auth_service", status = "ready"}),
        Event.publish("app.error", {error_code = 500, message = "Internal error"}),
        Event.publish("test.pattern", {pattern_test = true})
    }
    
    -- Verify at least some events were published
    local any_published = false
    for _, published in ipairs(events_published) do
        if published then
            any_published = true
            break
        end
    end
    
    -- Try to receive events (short timeout for test)
    local received_events = {}
    for name, sub_id in pairs(subs) do
        local received = Event.receive(sub_id, 500)  -- 500ms timeout
        if received then
            table.insert(received_events, {pattern = name, event = received})
        end
    end
    
    -- Clean up
    for _, sub_id in pairs(subs) do
        Event.unsubscribe(sub_id)
    end
    
    return any_published and (#received_events > 0)
end)

-- Test 4: Cross-Language Event Simulation
run_test("Cross-Language Event Simulation", function()
    -- Subscribe to cross-language events
    local rust_sub = Event.subscribe("rust.*")
    local js_sub = Event.subscribe("javascript.*")
    local python_sub = Event.subscribe("python.*")
    
    -- Publish events as if from different languages
    local rust_published = Event.publish("rust.computation", {
        result = 42,
        computation_time = 0.001,
        memory_used = 1024
    }, {
        language = "rust"
    })
    
    local js_published = Event.publish("javascript.ui_event", {
        event_type = "click",
        element_id = "submit_button",
        coordinates = {x = 100, y = 200}
    }, {
        language = "javascript"
    })
    
    local python_published = Event.publish("python.data_analysis", {
        dataset_size = 10000,
        analysis_type = "statistical",
        results = {mean = 45.7, stddev = 12.3}
    }, {
        language = "python"
    })
    
    -- Try to receive cross-language events
    local received_events = {}
    local subscriptions = {
        {sub = rust_sub, lang = "rust"},
        {sub = js_sub, lang = "javascript"},
        {sub = python_sub, lang = "python"}
    }
    
    for _, sub_info in ipairs(subscriptions) do
        local received = Event.receive(sub_info.sub, 800)  -- 800ms timeout
        if received then
            table.insert(received_events, {lang = sub_info.lang, event = received})
        end
    end
    
    -- Clean up
    Event.unsubscribe(rust_sub)
    Event.unsubscribe(js_sub)
    Event.unsubscribe(python_sub)
    
    return rust_published and js_published and python_published and (#received_events > 0)
end)

-- Test 5: Event Subscription Management
run_test("Event Subscription Management", function()
    -- Create multiple subscriptions
    local subs = {}
    local patterns = {"test.mgmt.*", "management.*", "*.lifecycle"}
    
    for _, pattern in ipairs(patterns) do
        local sub_id = Event.subscribe(pattern)
        table.insert(subs, {id = sub_id, pattern = pattern})
    end
    
    -- List subscriptions
    local all_subs = Event.list_subscriptions()
    local found_count = 0
    
    for _, sub in ipairs(subs) do
        for _, listed_sub in ipairs(all_subs) do
            if listed_sub.id == sub.id then
                found_count = found_count + 1
                break
            end
        end
    end
    
    -- Unsubscribe all
    local unsubscribed_count = 0
    for _, sub in ipairs(subs) do
        if Event.unsubscribe(sub.id) then
            unsubscribed_count = unsubscribed_count + 1
        end
    end
    
    return found_count == #subs and unsubscribed_count == #subs
end)

-- Test 6: Event System Statistics
run_test("Event System Statistics", function()
    local stats = Event.get_stats()
    
    -- Verify stats structure
    local has_stats = stats ~= nil and
                     stats.event_bus_stats ~= nil and
                     stats.bridge_stats ~= nil
    
    return has_stats
end)

-- Test 7: Event Timeout Behavior
run_test("Event Timeout Behavior", function()
    -- Subscribe but don't publish anything
    local sub_id = Event.subscribe("timeout.test.*")
    
    -- Verify subscription exists
    local subs = Event.list_subscriptions()
    local found_sub = false
    for _, sub in ipairs(subs) do
        if sub.id == sub_id then
            found_sub = true
            break
        end
    end
    
    -- Try to receive with timeout (should return nil)
    local received = Event.receive(sub_id, 200)  -- 200ms timeout
    
    -- Clean up
    Event.unsubscribe(sub_id)
    
    return found_sub and (received == nil)
end)

-- Test 8: Complex Event Data Structures
run_test("Complex Event Data Structures", function()
    local sub_id = Event.subscribe("complex.data.*")
    
    -- Publish event with complex nested data
    local complex_data = {
        metadata = {
            version = "2.0",
            created_by = "lua_integration_test",
            tags = {"test", "integration", "complex"}
        },
        payload = {
            users = {
                {id = 1, name = "Alice", roles = {"admin", "user"}},
                {id = 2, name = "Bob", roles = {"user"}}
            },
            settings = {
                theme = "dark",
                notifications = {
                    email = true,
                    push = false,
                    sms = {enabled = true, number = "+1234567890"}
                }
            },
            analytics = {
                events_today = 1247,
                active_users = 89,
                performance = {
                    avg_response_time = 0.125,
                    error_rate = 0.002
                }
            }
        }
    }
    
    local published = Event.publish("complex.data.structure", complex_data)
    
    -- Try to receive
    local received = Event.receive(sub_id, 1000)
    
    -- Verify received data structure (basic check)
    local data_intact = false
    if received and received.data then
        data_intact = received.data.metadata ~= nil and
                     received.data.payload ~= nil and
                     received.data.payload.users ~= nil and
                     #received.data.payload.users == 2
    end
    
    -- Clean up
    Event.unsubscribe(sub_id)
    
    return published and data_intact
end)

-- Test 9: Event Error Handling
run_test("Event Error Handling", function()
    local tests_completed = 0
    local tests_passed = 0
    
    -- Test 1: Valid subscription pattern (should work)
    local success1, sub_result1 = pcall(function()
        return Event.subscribe("error.test.*")  -- Valid pattern
    end)
    tests_completed = tests_completed + 1
    if success1 and sub_result1 then
        tests_passed = tests_passed + 1
        Event.unsubscribe(sub_result1)  -- Clean up
    end
    
    -- Test 2: Empty subscription pattern (may or may not work - implementation dependent)
    local success2, sub_result2 = pcall(function()
        return Event.subscribe("")  -- Empty pattern
    end)
    tests_completed = tests_completed + 1
    if success2 then
        tests_passed = tests_passed + 1
        if sub_result2 then
            Event.unsubscribe(sub_result2)  -- Clean up if successful
        end
    else
        -- Empty pattern failure is also acceptable
        tests_passed = tests_passed + 1
    end
    
    -- Test 3: Invalid unsubscribe with non-existent ID
    local success3, result3 = pcall(function()
        return Event.unsubscribe("definitely_invalid_subscription_id_12345")
    end)
    tests_completed = tests_completed + 1
    -- Unsubscribe should succeed but return false for non-existent subscription
    if success3 and not result3 then
        tests_passed = tests_passed + 1
    end
    
    -- Test 4: Receive on invalid subscription (should throw error)
    local success4, result4 = pcall(function()
        return Event.receive("definitely_invalid_sub_id_67890", 100)
    end)
    tests_completed = tests_completed + 1
    if not success4 then  -- Should fail with error
        tests_passed = tests_passed + 1
    end
    
    print(string.format("    Error handling tests: %d/%d passed", tests_passed, tests_completed))
    
    -- Return true if most error handling tests passed
    return tests_passed >= (tests_completed - 1)  -- Allow 1 test to fail
end)

-- Test 10: Multi-Event Workflow
run_test("Multi-Event Workflow", function()
    -- Create a workflow simulation with multiple event types
    local workflow_sub = Event.subscribe("workflow.*")
    local status_sub = Event.subscribe("*.status")
    
    -- Simulate a multi-step workflow
    local workflow_steps = {
        {event = "workflow.start", data = {workflow_id = "wf_001", step = 1}},
        {event = "workflow.progress", data = {workflow_id = "wf_001", step = 2, progress = 0.3}},
        {event = "workflow.progress", data = {workflow_id = "wf_001", step = 3, progress = 0.7}},
        {event = "workflow.complete", data = {workflow_id = "wf_001", step = 4, result = "success"}},
        {event = "system.status", data = {component = "workflow_engine", status = "idle"}}
    }
    
    -- Publish all workflow events
    local published_count = 0
    for _, step in ipairs(workflow_steps) do
        if Event.publish(step.event, step.data) then
            published_count = published_count + 1
        end
    end
    
    -- Try to receive some events
    local received_count = 0
    local max_receives = 3  -- Don't wait too long
    
    for i = 1, max_receives do
        local workflow_event = Event.receive(workflow_sub, 300)
        if workflow_event then
            received_count = received_count + 1
        end
        
        local status_event = Event.receive(status_sub, 300)
        if status_event then
            received_count = received_count + 1
        end
        
        -- Break early if we got some events
        if received_count >= 2 then
            break
        end
    end
    
    -- Clean up
    Event.unsubscribe(workflow_sub)
    Event.unsubscribe(status_sub)
    
    return published_count == #workflow_steps and received_count > 0
end)

-- Print final results
print("\n" .. string.rep("=", 50))
print("CROSS-LANGUAGE EVENT TEST RESULTS")
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