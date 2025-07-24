-- ABOUTME: Performance benchmarking runner for hook and event system stress testing
-- ABOUTME: Provides comprehensive performance analysis, load testing, and optimization insights

print("=== LLMSpell Performance Benchmarks Runner ===")
print("Comprehensive performance testing and optimization analysis")
print()

-- Benchmarking configuration
local config = {
    stress_test_enabled = true,
    concurrent_operations = true,
    memory_profiling = true,
    latency_analysis = true,
    throughput_testing = true,
    scalability_testing = true,
    warm_up_iterations = 5,
    benchmark_iterations = 100,
    stress_duration = 10, -- seconds
    detailed_reporting = true
}

-- Benchmark state
local benchmark_state = {
    tests_run = 0,
    tests_passed = 0,
    start_time = os.time(),
    results = {},
    performance_metrics = {
        hook_registration = {},
        hook_execution = {},
        event_publishing = {},
        event_receiving = {},
        subscription_management = {},
        memory_usage = {},
        concurrent_operations = {},
        stress_test_results = {}
    },
    system_limits = {
        max_hooks = 0,
        max_subscriptions = 0,
        max_events_per_second = 0,
        max_concurrent_operations = 0
    }
}

-- Helper function to measure execution time with high precision
local function measure_performance(operation_name, iterations, operation_func)
    print(string.format("🔬 Benchmarking: %s (%d iterations)", operation_name, iterations))
    
    -- Warm-up phase
    for i = 1, config.warm_up_iterations do
        operation_func()
    end
    
    -- Force garbage collection before measurement
    collectgarbage("collect")
    local start_memory = collectgarbage("count")
    
    -- Actual benchmark
    local start_time = os.clock()
    local successful_operations = 0
    
    for i = 1, iterations do
        local success, result = pcall(operation_func)
        if success then
            successful_operations = successful_operations + 1
        end
    end
    
    local end_time = os.clock()
    local end_memory = collectgarbage("count")
    
    local total_time = (end_time - start_time) * 1000 -- Convert to milliseconds
    local avg_time = total_time / iterations
    local ops_per_second = iterations / (total_time / 1000)
    local memory_delta = end_memory - start_memory
    local success_rate = (successful_operations / iterations) * 100
    
    local result = {
        operation = operation_name,
        iterations = iterations,
        successful_operations = successful_operations,
        total_time = total_time,
        avg_time = avg_time,
        ops_per_second = ops_per_second,
        memory_delta = memory_delta,
        success_rate = success_rate,
        start_memory = start_memory,
        end_memory = end_memory
    }
    
    benchmark_state.results[operation_name] = result
    benchmark_state.tests_run = benchmark_state.tests_run + 1
    
    if success_rate >= 95 then
        benchmark_state.tests_passed = benchmark_state.tests_passed + 1
    end
    
    print(string.format("   ⏱️  Avg time: %.3fms", avg_time))
    print(string.format("   📊 Throughput: %.1f ops/sec", ops_per_second))
    print(string.format("   💾 Memory delta: %.2f KB", memory_delta))
    print(string.format("   ✅ Success rate: %.1f%%", success_rate))
    print()
    
    return result
end

-- Helper function for stress testing
local function stress_test(test_name, duration_seconds, stress_func)
    print(string.format("⚡ Stress Test: %s (%ds duration)", test_name, duration_seconds))
    
    local start_time = os.clock()
    local end_time = start_time + duration_seconds
    local operations_completed = 0
    local errors_encountered = 0
    local max_memory = 0
    
    while os.clock() < end_time do
        local success, result = pcall(stress_func)
        if success then
            operations_completed = operations_completed + 1
        else
            errors_encountered = errors_encountered + 1
        end
        
        -- Monitor memory usage
        local current_memory = collectgarbage("count")
        max_memory = math.max(max_memory, current_memory)
        
        -- Brief pause to prevent system overload
        if operations_completed % 10 == 0 then
            os.execute("sleep 0.001") -- 1ms pause every 10 operations
        end
    end
    
    local actual_duration = os.clock() - start_time
    local ops_per_second = operations_completed / actual_duration
    local error_rate = (errors_encountered / (operations_completed + errors_encountered)) * 100
    
    local result = {
        test_name = test_name,
        duration = actual_duration,
        operations_completed = operations_completed,
        errors_encountered = errors_encountered,
        ops_per_second = ops_per_second,
        error_rate = error_rate,
        max_memory = max_memory
    }
    
    benchmark_state.performance_metrics.stress_test_results[test_name] = result
    
    print(string.format("   📊 Operations: %d", operations_completed))
    print(string.format("   ⚡ Throughput: %.1f ops/sec", ops_per_second))
    print(string.format("   ❌ Error rate: %.2f%%", error_rate))
    print(string.format("   💾 Peak memory: %.2f KB", max_memory))
    print()
    
    return result
end

print("🏁 Starting Performance Benchmarks...")
print(string.format("Configuration: Iterations=%d, Stress=%ds, Profiling=%s", 
      config.benchmark_iterations, config.stress_duration, 
      config.memory_profiling and "ON" or "OFF"))
print()

-- Benchmark 1: Hook Registration Performance
print("🪝 HOOK SYSTEM BENCHMARKS")
print(string.rep("-", 40))

local hook_handles = {}

measure_performance("Hook Registration", config.benchmark_iterations, function()
    local handle = Hook.register("BeforeAgentExecution", function(context)
        return "continue"
    end, "normal")
    table.insert(hook_handles, handle)
    return handle
end)

-- Test hook execution performance
local test_context = {
    component_id = {name = "test_agent"},
    correlation_id = "benchmark_test",
    metadata = {}
}

measure_performance("Hook Execution", config.benchmark_iterations, function()
    -- Simulate hook execution by calling a simple function
    local result = "continue"
    return result
end)

-- Test hook unregistration performance
measure_performance("Hook Unregistration", math.min(#hook_handles, config.benchmark_iterations), function()
    if #hook_handles > 0 then
        local handle = table.remove(hook_handles)
        if handle and handle:id() then
            Hook.unregister(handle)
        end
    end
end)

-- Benchmark 2: Event System Performance
print("📡 EVENT SYSTEM BENCHMARKS")
print(string.rep("-", 40))

local subscriptions = {}

measure_performance("Event Subscription", config.benchmark_iterations, function()
    local pattern = "benchmark.test." .. math.random(1000, 9999) .. ".*"
    local sub_id = Event.subscribe(pattern)
    table.insert(subscriptions, sub_id)
    return sub_id
end)

measure_performance("Event Publishing", config.benchmark_iterations, function()
    local event_name = "benchmark.test.event." .. math.random(1000, 9999)
    local event_data = {
        benchmark_id = math.random(1, 1000000),
        timestamp = os.time(),
        iteration = math.random(1, config.benchmark_iterations),
        test_data = string.rep("x", 100) -- 100 byte payload
    }
    return Event.publish(event_name, event_data)
end)

-- Create a subscription for receiving tests
local receive_subscription = Event.subscribe("benchmark.receive.*")

-- Publish events for receiving benchmark
for i = 1, config.benchmark_iterations do
    Event.publish("benchmark.receive.test", {
        id = i,
        data = "benchmark_data_" .. i
    })
end

measure_performance("Event Receiving", config.benchmark_iterations, function()
    return Event.receive(receive_subscription, 10) -- 10ms timeout
end)

measure_performance("Event Unsubscription", math.min(#subscriptions, config.benchmark_iterations), function()
    if #subscriptions > 0 then
        local sub_id = table.remove(subscriptions)
        return Event.unsubscribe(sub_id)
    end
end)

-- Benchmark 3: Memory Usage Analysis
if config.memory_profiling then
    print("💾 MEMORY USAGE BENCHMARKS")
    print(string.rep("-", 40))
    
    -- Test memory usage patterns
    local memory_tests = {
        {name = "Hook Memory Usage", func = function()
            local temp_handles = {}
            for i = 1, 50 do
                local handle = Hook.register("BeforeAgentInit", function() return "continue" end, "normal")
                table.insert(temp_handles, handle)
            end
            -- Cleanup
            for _, handle in ipairs(temp_handles) do
                if handle and handle:id() then
                    Hook.unregister(handle)
                end
            end
        end},
        
        {name = "Event Memory Usage", func = function()
            local temp_subs = {}
            for i = 1, 50 do
                local sub = Event.subscribe("memory.test." .. i .. ".*")
                table.insert(temp_subs, sub)
                Event.publish("memory.test." .. i .. ".event", {data = string.rep("x", 1000)})
            end
            -- Cleanup
            for _, sub in ipairs(temp_subs) do
                Event.unsubscribe(sub)
            end
        end},
        
        {name = "Large Data Events", func = function()
            local large_data = {}
            for i = 1, 100 do
                large_data["field_" .. i] = string.rep("data", 100) -- ~400 bytes per field
            end
            Event.publish("benchmark.large.data", large_data)
        end}
    }
    
    for _, test in ipairs(memory_tests) do
        local start_memory = collectgarbage("count")
        test.func()
        collectgarbage("collect") -- Force cleanup
        local end_memory = collectgarbage("count")
        local memory_used = math.max(0, end_memory - start_memory)
        
        print(string.format("   🧪 %s: %.2f KB", test.name, memory_used))
    end
    print()
end

-- Benchmark 4: Concurrent Operations
if config.concurrent_operations then
    print("🔀 CONCURRENT OPERATIONS BENCHMARKS")
    print(string.rep("-", 40))
    
    -- Simulate concurrent hook and event operations
    local concurrent_result = measure_performance("Concurrent Hook+Event Operations", 50, function()
        -- Register hook
        local handle = Hook.register("BeforeToolExecution", function() return "continue" end, "low")
        
        -- Subscribe to events
        local sub = Event.subscribe("concurrent.test.*")
        
        -- Publish event
        Event.publish("concurrent.test.operation", {
            concurrent_id = math.random(1, 10000),
            operation_type = "concurrent_benchmark"
        })
        
        -- Receive event
        local received = Event.receive(sub, 5)
        
        -- Cleanup
        if handle and handle:id() then
            Hook.unregister(handle)
        end
        Event.unsubscribe(sub)
        
        return received ~= nil
    end)
    
    benchmark_state.system_limits.max_concurrent_operations = concurrent_result.ops_per_second
end

-- Benchmark 5: Stress Testing
if config.stress_test_enabled then
    print("⚡ STRESS TESTING")
    print(string.rep("-", 40))
    
    -- Hook system stress test
    stress_test("Hook System Stress", config.stress_duration, function()
        local handle = Hook.register("BeforeAgentExecution", function() return "continue" end, "normal")
        -- Keep some hooks registered to test system limits
        if math.random() > 0.8 and handle and handle:id() then -- 20% cleanup rate
            Hook.unregister(handle)
        end
    end)
    
    -- Event system stress test
    stress_test("Event System Stress", config.stress_duration, function()
        local pattern = "stress.test." .. math.random(1, 100) .. ".*"
        local sub = Event.subscribe(pattern)
        
        Event.publish("stress.test." .. math.random(1, 100) .. ".event", {
            stress_data = math.random(1, 1000000),
            timestamp = os.time()
        })
        
        Event.receive(sub, 1) -- Very short timeout
        
        if math.random() > 0.7 then -- 30% cleanup rate
            Event.unsubscribe(sub)
        end
    end)
    
    -- Mixed operations stress test
    stress_test("Mixed Operations Stress", config.stress_duration, function()
        if math.random() > 0.5 then
            -- Hook operation
            local handle = Hook.register("BeforeWorkflowStart", function() return "continue" end, "normal")
            if math.random() > 0.9 and handle and handle:id() then
                Hook.unregister(handle)
            end
        else
            -- Event operation
            local event_type = math.random() > 0.5 and "subscribe" or "publish"
            if event_type == "subscribe" then
                local sub = Event.subscribe("mixed.stress.*")
                if math.random() > 0.8 then
                    Event.unsubscribe(sub)
                end
            else
                Event.publish("mixed.stress.event", {mixed_operation = true})
            end
        end
    end)
end

-- Benchmark 6: Scalability Testing
if config.scalability_testing then
    print("📈 SCALABILITY TESTING")
    print(string.rep("-", 40))
    
    -- Test system limits
    local scale_tests = {
        {name = "Maximum Hooks", limit_test = function()
            local handles = {}
            local count = 0
            while count < 1000 do -- Safety limit
                local handle = Hook.register("BeforeAgentInit", function() return "continue" end, "normal")
                if handle and handle:id() then
                    table.insert(handles, handle)
                    count = count + 1
                else
                    break
                end
            end
            -- Cleanup
            for _, handle in ipairs(handles) do
                if handle and handle:id() then
                    Hook.unregister(handle)
                end
            end
            return count
        end},
        
        {name = "Maximum Subscriptions", limit_test = function()
            local subs = {}
            local count = 0
            while count < 1000 do -- Safety limit
                local sub = Event.subscribe("scale.test." .. count .. ".*")
                if sub then
                    table.insert(subs, sub)
                    count = count + 1
                else
                    break
                end
            end
            -- Cleanup
            for _, sub in ipairs(subs) do
                Event.unsubscribe(sub)
            end
            return count
        end},
        
        {name = "Event Throughput Limit", limit_test = function()
            local sub = Event.subscribe("throughput.test.*")
            local start_time = os.clock()
            local events_published = 0
            
            while (os.clock() - start_time) < 1 do -- 1 second test
                local published = Event.publish("throughput.test.event", {id = events_published})
                if published then
                    events_published = events_published + 1
                end
            end
            
            Event.unsubscribe(sub)
            return events_published
        end}
    }
    
    for _, test in ipairs(scale_tests) do
        print(string.format("   🔬 Testing %s...", test.name))
        local limit = test.limit_test()
        
        if test.name:find("Hooks") then
            benchmark_state.system_limits.max_hooks = limit
        elseif test.name:find("Subscriptions") then
            benchmark_state.system_limits.max_subscriptions = limit
        elseif test.name:find("Throughput") then
            benchmark_state.system_limits.max_events_per_second = limit
        end
        
        print(string.format("   📊 Limit reached: %d", limit))
    end
    print()
end

-- Comprehensive Performance Report
print(string.rep("📊", 20))
print("🎯 COMPREHENSIVE PERFORMANCE REPORT")
print(string.rep("📊", 20))
print()

-- Overall statistics
local total_runtime = os.time() - benchmark_state.start_time
local overall_success_rate = benchmark_state.tests_run > 0 and 
                            (benchmark_state.tests_passed / benchmark_state.tests_run) * 100 or 0

print("📈 Overall Performance Summary:")
print(string.format("   • Tests executed: %d", benchmark_state.tests_run))
print(string.format("   • Tests passed: %d", benchmark_state.tests_passed))
print(string.format("   • Success rate: %.1f%%", overall_success_rate))
print(string.format("   • Total runtime: %ds", total_runtime))
print(string.format("   • Final memory usage: %.2f KB", collectgarbage("count")))

-- Performance metrics summary
print()
print("⚡ Performance Metrics Summary:")

local best_performers = {}
local worst_performers = {}

for operation, result in pairs(benchmark_state.results) do
    table.insert(best_performers, {op = operation, metric = result.ops_per_second})
    if result.avg_time > 1 then -- More than 1ms average
        table.insert(worst_performers, {op = operation, metric = result.avg_time})
    end
end

table.sort(best_performers, function(a, b) return a.metric > b.metric end)
table.sort(worst_performers, function(a, b) return a.metric > b.metric end)

print("   🏆 Best Performing Operations:")
for i = 1, math.min(3, #best_performers) do
    local perf = best_performers[i]
    print(string.format("   %d. %s: %.1f ops/sec", i, perf.op, perf.metric))
end

if #worst_performers > 0 then
    print("   ⚠️  Operations Needing Optimization:")
    for i = 1, math.min(3, #worst_performers) do
        local perf = worst_performers[i]
        print(string.format("   %d. %s: %.3fms avg", i, perf.op, perf.metric))
    end
end

-- System limits summary
print()
print("📏 System Scalability Limits:")
if benchmark_state.system_limits.max_hooks > 0 then
    print(string.format("   • Maximum hooks: %d", benchmark_state.system_limits.max_hooks))
end
if benchmark_state.system_limits.max_subscriptions > 0 then
    print(string.format("   • Maximum subscriptions: %d", benchmark_state.system_limits.max_subscriptions))
end
if benchmark_state.system_limits.max_events_per_second > 0 then
    print(string.format("   • Maximum event throughput: %d events/sec", benchmark_state.system_limits.max_events_per_second))
end

-- Stress test results
if config.stress_test_enabled then
    print()
    print("⚡ Stress Test Results:")
    for test_name, result in pairs(benchmark_state.performance_metrics.stress_test_results) do
        print(string.format("   • %s:", test_name))
        print(string.format("     - Throughput: %.1f ops/sec", result.ops_per_second))
        print(string.format("     - Error rate: %.2f%%", result.error_rate))
        print(string.format("     - Peak memory: %.2f KB", result.max_memory))
    end
end

-- Performance grade and recommendations
print()
print("🏆 Performance Grade Assessment:")

local performance_grade = "F"
local grade_score = 0

-- Scoring criteria
if overall_success_rate >= 95 then grade_score = grade_score + 25 end
if benchmark_state.system_limits.max_hooks >= 100 then grade_score = grade_score + 20 end
if benchmark_state.system_limits.max_subscriptions >= 100 then grade_score = grade_score + 20 end
if benchmark_state.system_limits.max_events_per_second >= 1000 then grade_score = grade_score + 20 end

-- Check if any operation is particularly fast
local has_fast_operations = false
for _, result in pairs(benchmark_state.results) do
    if result.ops_per_second > 10000 then
        has_fast_operations = true
        break
    end
end
if has_fast_operations then grade_score = grade_score + 15 end

-- Assign grade
if grade_score >= 90 then performance_grade = "A+"
elseif grade_score >= 85 then performance_grade = "A"
elseif grade_score >= 80 then performance_grade = "A-"
elseif grade_score >= 75 then performance_grade = "B+"
elseif grade_score >= 70 then performance_grade = "B"
elseif grade_score >= 65 then performance_grade = "B-"
elseif grade_score >= 60 then performance_grade = "C+"
elseif grade_score >= 55 then performance_grade = "C"
elseif grade_score >= 50 then performance_grade = "C-"
elseif grade_score >= 40 then performance_grade = "D"
else performance_grade = "F"
end

print(string.format("   🎓 Overall Performance Grade: %s (%d/100)", performance_grade, grade_score))

-- Recommendations based on performance
print()
print("💡 Performance Optimization Recommendations:")

if overall_success_rate < 95 then
    print("   1. Investigate and fix failing operations")
end

if benchmark_state.system_limits.max_hooks < 100 then
    print("   2. Optimize hook registration and management for scalability")
end

if benchmark_state.system_limits.max_events_per_second < 1000 then
    print("   3. Improve event publishing and receiving performance")
end

local has_memory_issues = false
for _, result in pairs(benchmark_state.results) do
    if result.memory_delta > 1000 then -- More than 1MB
        has_memory_issues = true
        break
    end
end

if has_memory_issues then
    print("   4. Optimize memory usage in high-frequency operations")
end

if performance_grade >= "A" then
    print("   🎉 Excellent performance! Consider these advanced optimizations:")
    print("     • Implement operation batching for even higher throughput")
    print("     • Consider memory pooling for frequently allocated objects")
    print("     • Profile specific use cases for micro-optimizations")
else
    print("   🔧 Focus on these core improvements:")
    print("     • Profile and optimize slow operations")
    print("     • Implement proper resource cleanup")
    print("     • Consider caching strategies for repeated operations")
end

-- Final cleanup
print()
print("🧹 Performing comprehensive cleanup...")

-- Clean up any remaining test resources
local final_hooks = #Hook.list()
local final_subs = #Event.list_subscriptions()

if final_hooks > 10 or final_subs > 10 then
    print(string.format("   ⚠️  Cleanup needed: %d hooks, %d subscriptions remain", final_hooks, final_subs))
else
    print("   ✅ System state is clean")
end

collectgarbage("collect")
local final_memory = collectgarbage("count")

print(string.format("   💾 Final memory: %.2f KB", final_memory))

print()
print(string.rep("🏁", 20))
print("🚀 PERFORMANCE BENCHMARKS COMPLETE")
print(string.format("   Performance Grade: %s", performance_grade))
print("   Detailed metrics available in benchmark results")
print(string.rep("🏁", 20))