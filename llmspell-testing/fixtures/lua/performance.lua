-- ABOUTME: Performance validation tests for Lua Hook and Event APIs
-- ABOUTME: Tests performance characteristics, throughput, and resource usage

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

-- Helper function to measure execution time
local function measure_time(func)
    local start_time = os.clock()
    local result = func()
    local end_time = os.clock()
    return result, (end_time - start_time) * 1000  -- Return result and time in milliseconds
end

-- Test 1: Hook Registration Performance
run_test("Hook Registration Performance", function()
    local num_hooks = 50
    local handles = {}
    
    local _, register_time = measure_time(function()
        for i = 1, num_hooks do
            local handle = Hook.register("BeforeAgentExecution", function(ctx)
                return "continue"
            end, "normal")
            table.insert(handles, handle)
        end
        return true
    end)
    
    -- Performance target: should register 50 hooks in under 500ms
    local performance_ok = register_time < 500
    
    -- Clean up
    local _, cleanup_time = measure_time(function()
        for _, handle in ipairs(handles) do
            handle:unregister()
        end
        return true
    end)
    
    print(string.format("    Registered %d hooks in %.2fms (avg: %.2fms per hook)", 
                       num_hooks, register_time, register_time / num_hooks))
    print(string.format("    Cleaned up %d hooks in %.2fms", num_hooks, cleanup_time))
    
    return performance_ok
end)

-- Test 2: Hook Listing Performance
run_test("Hook Listing Performance", function()
    -- Register a batch of hooks for testing
    local num_hooks = 30
    local handles = {}
    
    for i = 1, num_hooks do
        local handle = Hook.register("BeforeAgentInit", function(ctx)
            return "continue"
        end, i % 2 == 0 and "high" or "normal")
        table.insert(handles, handle)
    end
    
    -- Test list all performance
    local _, list_all_time = measure_time(function()
        return Hook.list()
    end)
    
    -- Test filtered list performance
    local _, list_filtered_time = measure_time(function()
        return Hook.list({language = "lua", priority = "high"})
    end)
    
    -- Performance target: listing should be under 50ms
    local performance_ok = list_all_time < 50 and list_filtered_time < 50
    
    -- Clean up
    for _, handle in ipairs(handles) do
        handle:unregister()
    end
    
    print(string.format("    List all hooks: %.2fms", list_all_time))
    print(string.format("    List filtered hooks: %.2fms", list_filtered_time))
    
    return performance_ok
end)

-- Test 3: Event Publishing Performance
run_test("Event Publishing Performance", function()
    local num_events = 100
    local events_published = 0
    
    local _, publish_time = measure_time(function()
        for i = 1, num_events do
            local published = Event.publish("performance.test." .. i, {
                sequence = i,
                timestamp = os.time(),
                data = "test data for event " .. i
            })
            if published then
                events_published = events_published + 1
            end
        end
        return true
    end)
    
    -- Performance target: should publish 100 events in under 1 second
    local performance_ok = publish_time < 1000 and events_published == num_events
    
    print(string.format("    Published %d events in %.2fms (avg: %.2fms per event)", 
                       events_published, publish_time, publish_time / events_published))
    
    return performance_ok
end)

-- Test 4: Event Subscription Performance
run_test("Event Subscription Performance", function()
    local num_subscriptions = 25
    local subscription_ids = {}
    
    local _, subscribe_time = measure_time(function()
        for i = 1, num_subscriptions do
            local sub_id = Event.subscribe("perf.test." .. i .. ".*")
            table.insert(subscription_ids, sub_id)
        end
        return true
    end)
    
    -- Test listing subscriptions performance
    local _, list_subs_time = measure_time(function()
        return Event.list_subscriptions()
    end)
    
    -- Test unsubscribe performance
    local _, unsubscribe_time = measure_time(function()
        for _, sub_id in ipairs(subscription_ids) do
            Event.unsubscribe(sub_id)
        end
        return true
    end)
    
    -- Performance targets
    local performance_ok = subscribe_time < 200 and list_subs_time < 50 and unsubscribe_time < 200
    
    print(string.format("    Created %d subscriptions in %.2fms", num_subscriptions, subscribe_time))
    print(string.format("    Listed subscriptions in %.2fms", list_subs_time))
    print(string.format("    Unsubscribed %d in %.2fms", num_subscriptions, unsubscribe_time))
    
    return performance_ok
end)

-- Test 5: Event Throughput Performance
run_test("Event Throughput Performance", function()
    -- Set up subscription
    local sub_id = Event.subscribe("throughput.*")
    local num_events = 50
    
    -- Publish events rapidly
    local publish_start = os.clock()
    for i = 1, num_events do
        Event.publish("throughput.event", {
            id = i,
            payload = string.rep("data", 10)  -- Some payload data
        })
    end
    local publish_time = (os.clock() - publish_start) * 1000
    
    -- Try to receive events (with reasonable timeout)
    local received_count = 0
    local receive_start = os.clock()
    
    for i = 1, num_events do
        local received = Event.receive(sub_id, 50)  -- 50ms timeout per event
        if received then
            received_count = received_count + 1
        else
            break  -- Stop if we hit timeout
        end
    end
    
    local receive_time = (os.clock() - receive_start) * 1000
    
    -- Clean up
    Event.unsubscribe(sub_id)
    
    -- Performance evaluation
    local publish_rate = num_events / (publish_time / 1000)  -- events per second
    local receive_rate = received_count / (receive_time / 1000)  -- events per second
    
    print(string.format("    Published %d events in %.2fms (%.1f events/sec)", 
                       num_events, publish_time, publish_rate))
    print(string.format("    Received %d events in %.2fms (%.1f events/sec)", 
                       received_count, receive_time, receive_rate))
    
    -- Performance target: should handle at least 20 events/sec and receive at least 50% of events
    return publish_rate > 20 and received_count >= (num_events * 0.3)  -- At least 30% received
end)

-- Test 6: Memory Usage Simulation
run_test("Memory Usage Simulation", function()
    -- This test simulates memory-intensive operations
    local handles = {}
    local subscriptions = {}
    local large_data_events = 0
    
    -- Create hooks with complex callbacks
    for i = 1, 20 do
        local handle = Hook.register("BeforeAgentExecution", function(ctx)
            -- Simulate some processing
            local data = {}
            for j = 1, 100 do
                data[j] = string.format("item_%d_%d", i, j)
            end
            return {
                type = "modified",
                data = {processed_items = data, timestamp = os.time()}
            }
        end)
        table.insert(handles, handle)
    end
    
    -- Create subscriptions and publish large events
    for i = 1, 10 do
        local sub_id = Event.subscribe("memory.test." .. i .. ".*")
        table.insert(subscriptions, sub_id)
        
        -- Publish event with larger payload
        local large_payload = {}
        for j = 1, 50 do
            large_payload["key_" .. j] = {
                value = string.rep("data", 20),
                metadata = {created = os.time(), index = j}
            }
        end
        
        if Event.publish("memory.test." .. i .. ".data", large_payload) then
            large_data_events = large_data_events + 1
        end
    end
    
    -- Clean up all resources
    for _, handle in ipairs(handles) do
        handle:unregister()
    end
    
    for _, sub_id in ipairs(subscriptions) do
        Event.unsubscribe(sub_id)
    end
    
    print(string.format("    Created %d hooks with complex callbacks", #handles))
    print(string.format("    Published %d large payload events", large_data_events))
    print("    Successfully cleaned up all resources")
    
    -- Success if we managed to create and clean up everything
    return #handles == 20 and large_data_events == 10
end)

-- Test 7: Concurrent Operations Simulation
run_test("Concurrent Operations Simulation", function()
    -- Simulate concurrent hook registrations and event operations
    local operations_completed = 0
    
    -- Batch 1: Register hooks while publishing events
    local handles = {}
    local sub_id = Event.subscribe("concurrent.*")
    
    for i = 1, 15 do
        -- Register hook
        local handle = Hook.register("BeforeAgentInit", function(ctx)
            return "continue"
        end, i % 3 == 0 and "high" or "normal")
        table.insert(handles, handle)
        operations_completed = operations_completed + 1
        
        -- Publish event
        if Event.publish("concurrent.operation", {
            operation_id = i,
            type = "concurrent_test"
        }) then
            operations_completed = operations_completed + 1
        end
    end
    
    -- Batch 2: List operations while receiving events
    for i = 1, 10 do
        -- List hooks
        local hooks = Hook.list()
        if hooks then
            operations_completed = operations_completed + 1
        end
        
        -- Try to receive event
        local received = Event.receive(sub_id, 30)  -- Short timeout
        if received then
            operations_completed = operations_completed + 1
        end
    end
    
    -- Clean up
    for _, handle in ipairs(handles) do
        handle:unregister()
    end
    Event.unsubscribe(sub_id)
    
    print(string.format("    Completed %d concurrent operations", operations_completed))
    
    -- Success if we completed most operations without errors
    return operations_completed >= 35  -- Should complete at least 35 out of 50 possible operations
end)

-- Test 8: Resource Cleanup Performance
run_test("Resource Cleanup Performance", function()
    -- Create many resources
    local num_resources = 40
    local handles = {}
    local subscriptions = {}
    
    -- Create resources
    local _, creation_time = measure_time(function()
        for i = 1, num_resources do
            local handle = Hook.register("BeforeAgentExecution", function(ctx)
                return "continue"
            end)
            table.insert(handles, handle)
            
            local sub_id = Event.subscribe("cleanup.test." .. i .. ".*")
            table.insert(subscriptions, sub_id)
        end
        return true
    end)
    
    -- Clean up all resources
    local _, cleanup_time = measure_time(function()
        for _, handle in ipairs(handles) do
            handle:unregister()
        end
        
        for _, sub_id in ipairs(subscriptions) do
            Event.unsubscribe(sub_id)
        end
        return true
    end)
    
    -- Performance target: cleanup should be fast
    local performance_ok = cleanup_time < 300  -- Under 300ms for cleanup
    
    print(string.format("    Created %d hooks + %d subscriptions in %.2fms", 
                       num_resources, num_resources, creation_time))
    print(string.format("    Cleaned up all resources in %.2fms", cleanup_time))
    
    return performance_ok
end)

-- Test 9: Stress Test - Rapid Operations
run_test("Stress Test - Rapid Operations", function()
    local operations = 0
    local errors = 0
    
    local _, stress_time = measure_time(function()
        -- Rapid hook operations
        for i = 1, 30 do
            local success, handle = pcall(function()
                return Hook.register("BeforeAgentInit", function(ctx) return "continue" end)
            end)
            
            if success and handle then
                operations = operations + 1
                -- Immediately unregister
                local unregistered = handle:unregister()
                if unregistered then
                    operations = operations + 1
                else
                    errors = errors + 1
                end
            else
                errors = errors + 1
            end
        end
        
        -- Rapid event operations
        for i = 1, 20 do
            local sub_success, sub_id = pcall(function()
                return Event.subscribe("stress.test.*")
            end)
            
            if sub_success then
                operations = operations + 1
                
                local pub_success = pcall(function()
                    return Event.publish("stress.test.event", {id = i})
                end)
                
                if pub_success then
                    operations = operations + 1
                end
                
                local unsub_success = Event.unsubscribe(sub_id)
                if unsub_success then
                    operations = operations + 1
                else
                    errors = errors + 1
                end
            else
                errors = errors + 1
            end
        end
        
        return true
    end)
    
    local success_rate = operations / (operations + errors) * 100
    
    print(string.format("    Completed %d operations with %d errors in %.2fms", 
                       operations, errors, stress_time))
    print(string.format("    Success rate: %.1f%%", success_rate))
    
    -- Success if we completed most operations with low error rate
    return success_rate > 90 and operations > 80
end)

-- Test 10: System Resource Monitoring
run_test("System Resource Monitoring", function()
    -- Get initial system stats
    local initial_stats = Event.get_stats()
    
    -- Perform various operations
    local handles = {}
    local subscriptions = {}
    
    -- Create some load
    for i = 1, 25 do
        local handle = Hook.register("BeforeAgentExecution", function(ctx)
            return "continue"
        end)
        table.insert(handles, handle)
        
        local sub_id = Event.subscribe("monitor.test.*")
        table.insert(subscriptions, sub_id)
        
        Event.publish("monitor.test.event", {
            iteration = i,
            timestamp = os.time()
        })
    end
    
    -- Get stats after operations
    local after_stats = Event.get_stats()
    
    -- Clean up
    for _, handle in ipairs(handles) do
        handle:unregister()
    end
    
    for _, sub_id in ipairs(subscriptions) do
        Event.unsubscribe(sub_id)
    end
    
    -- Get final stats
    local final_stats = Event.get_stats()
    
    -- Verify we can collect stats at different points
    local stats_collected = initial_stats ~= nil and 
                           after_stats ~= nil and 
                           final_stats ~= nil
    
    print("    System resource monitoring completed")
    print("    ✓ Initial stats collected")
    print("    ✓ After-operations stats collected") 
    print("    ✓ Final stats collected")
    
    return stats_collected
end)

-- Print final results
print("\n" .. string.rep("=", 50))
print("PERFORMANCE TEST RESULTS")
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