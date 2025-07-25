-- ABOUTME: High-throughput event scenarios and performance optimization
-- ABOUTME: Demonstrates event system performance, throughput testing, bottleneck analysis, and optimization

print("=== Event Performance Example ===")
print("Demonstrates: High-throughput scenarios, performance testing, optimization, and bottleneck analysis")
print()

local subscriptions = {}
local performance_stats = {
    events_published = 0,
    events_received = 0,
    publish_times = {},
    receive_times = {},
    subscription_times = {},
    throughput_tests = {},
    memory_usage = {}
}

-- Helper function to measure execution time
local function measure_time(func, description)
    local start_time = os.clock()
    local result = func()
    local end_time = os.clock()
    local duration = (end_time - start_time) * 1000 -- Convert to milliseconds
    
    if description then
        print(string.format("   ⏱️  %s: %.2fms", description, duration))
    end
    
    return result, duration
end

-- Helper function to estimate memory usage (simplified)
local function estimate_memory_usage()
    -- This is a simplified estimation - in real scenarios you'd use more sophisticated methods
    local count = 0
    for _ in pairs(_G) do count = count + 1 end
    return count * 100 -- Rough estimate in bytes
end

print("1. Baseline performance measurement:")

print("   📏 Measuring baseline performance:")

-- Measure subscription creation performance
local baseline_subscription, sub_time = measure_time(function()
    return Event.subscribe("performance.baseline.*")
end, "Baseline subscription creation")

subscriptions.baseline = baseline_subscription
table.insert(performance_stats.subscription_times, sub_time)

-- Measure single event publish performance
local single_publish_time = measure_time(function()
    return Event.publish("performance.baseline.single", {
        test_id = "baseline_001",
        timestamp = os.time(),
        data = "baseline test data"
    })
end, "Single event publish")

table.insert(performance_stats.publish_times, single_publish_time)
performance_stats.events_published = performance_stats.events_published + 1

-- Measure single event receive performance
local single_receive_time = measure_time(function()
    return Event.receive(baseline_subscription, 1000)
end, "Single event receive")

table.insert(performance_stats.receive_times, single_receive_time)
if single_receive_time then
    performance_stats.events_received = performance_stats.events_received + 1
end

local baseline_memory = estimate_memory_usage()
table.insert(performance_stats.memory_usage, {operation = "baseline", memory = baseline_memory})

print()
print("2. Burst publishing performance:")

print("   🚀 Burst publishing test:")

-- Test burst publishing (many events quickly)
local burst_sizes = {10, 50, 100, 250, 500}

for _, burst_size in ipairs(burst_sizes) do
    local burst_data = {}
    for i = 1, burst_size do
        table.insert(burst_data, {
            event_id = "burst_" .. i,
            sequence = i,
            payload = string.rep("x", 100), -- 100 byte payload
            timestamp = os.time()
        })
    end
    
    local burst_start = os.clock()
    local successful_publishes = 0
    
    for i, data in ipairs(burst_data) do
        local published = Event.publish("performance.burst.test", data)
        if published then
            successful_publishes = successful_publishes + 1
        end
    end
    
    local burst_time = (os.clock() - burst_start) * 1000
    local throughput = successful_publishes / (burst_time / 1000) -- events per second
    
    performance_stats.events_published = performance_stats.events_published + successful_publishes
    
    table.insert(performance_stats.throughput_tests, {
        test_type = "burst_publish",
        event_count = burst_size,
        successful = successful_publishes,
        duration_ms = burst_time,
        throughput_eps = throughput
    })
    
    print(string.format("   • %d events: %.2fms (%.0f events/sec, %d successful)", 
          burst_size, burst_time, throughput, successful_publishes))
end

print()
print("3. Concurrent subscription performance:")

print("   📡 Concurrent subscription test:")

-- Create multiple subscriptions rapidly
local concurrent_patterns = {}
for i = 1, 20 do
    table.insert(concurrent_patterns, "performance.concurrent." .. i .. ".*")
end

local concurrent_start = os.clock()
local concurrent_subs = {}

for i, pattern in ipairs(concurrent_patterns) do
    local sub_id = Event.subscribe(pattern)
    if sub_id then
        concurrent_subs["concurrent_" .. i] = sub_id
        subscriptions["concurrent_" .. i] = sub_id
    end
end

local concurrent_time = (os.clock() - concurrent_start) * 1000
local sub_throughput = #concurrent_patterns / (concurrent_time / 1000)

print(string.format("   ✅ Created %d subscriptions in %.2fms (%.0f subs/sec)", 
      #concurrent_patterns, concurrent_time, sub_throughput))

table.insert(performance_stats.throughput_tests, {
    test_type = "concurrent_subscribe",
    event_count = #concurrent_patterns,
    successful = #concurrent_patterns,
    duration_ms = concurrent_time,
    throughput_eps = sub_throughput
})

print()
print("4. High-frequency event streaming:")

print("   📊 High-frequency streaming test:")

-- Create a dedicated subscription for streaming
local streaming_sub = Event.subscribe("performance.streaming.*")
subscriptions.streaming = streaming_sub

-- Generate high-frequency events
local streaming_duration = 2 -- seconds
local target_frequency = 100 -- events per second
local total_streaming_events = streaming_duration * target_frequency

print(string.format("   🎯 Target: %d events over %d seconds (%d events/sec)", 
      total_streaming_events, streaming_duration, target_frequency))

local streaming_start = os.clock()
local streaming_published = 0

-- Simulate real-time data streaming
for i = 1, total_streaming_events do
    local event_data = {
        stream_id = "stream_001",
        sequence = i,
        timestamp = os.time(),
        sensor_data = {
            temperature = 20 + math.random() * 10,
            humidity = 40 + math.random() * 20,
            pressure = 1000 + math.random() * 50,
            wind_speed = math.random() * 25
        },
        quality_score = math.random()
    }
    
    local published = Event.publish("performance.streaming.sensor", event_data)
    if published then
        streaming_published = streaming_published + 1
    end
    
    -- Small delay to simulate real-time streaming
    if i % 10 == 0 then
        os.execute("sleep 0.01") -- 10ms delay every 10 events
    end
end

local streaming_time = (os.clock() - streaming_start) * 1000
local actual_frequency = streaming_published / (streaming_time / 1000)

performance_stats.events_published = performance_stats.events_published + streaming_published

table.insert(performance_stats.throughput_tests, {
    test_type = "streaming",
    event_count = total_streaming_events,
    successful = streaming_published,
    duration_ms = streaming_time,
    throughput_eps = actual_frequency
})

print(string.format("   ✅ Published %d events in %.2fms (%.0f events/sec)", 
      streaming_published, streaming_time, actual_frequency))

print()
print("5. Bulk event receiving performance:")

print("   📥 Bulk receiving test:")

-- Test receiving performance with the streaming events
local receive_start = os.clock()
local received_events = 0
local receive_attempts = 0

-- Receive events in batches
while received_events < streaming_published and receive_attempts < streaming_published do
    local received = Event.receive(streaming_sub, 50) -- 50ms timeout
    receive_attempts = receive_attempts + 1
    
    if received then
        received_events = received_events + 1
        performance_stats.events_received = performance_stats.events_received + 1
    else
        break -- No more events available
    end
    
    -- Prevent infinite loop
    if receive_attempts > streaming_published * 2 then
        break
    end
end

local receive_time = (os.clock() - receive_start) * 1000
local receive_throughput = received_events / (receive_time / 1000)

table.insert(performance_stats.throughput_tests, {
    test_type = "bulk_receive",
    event_count = received_events,
    successful = received_events,
    duration_ms = receive_time,
    throughput_eps = receive_throughput
})

print(string.format("   ✅ Received %d events in %.2fms (%.0f events/sec, %d attempts)", 
      received_events, receive_time, receive_throughput, receive_attempts))

print()
print("6. Memory usage analysis:")

print("   💾 Memory usage analysis:")

-- Measure memory usage at different points
local memory_points = {
    {label = "After concurrent subscriptions", memory = estimate_memory_usage()},
    {label = "After streaming events", memory = estimate_memory_usage()},
    {label = "After bulk receiving", memory = estimate_memory_usage()}
}

for _, point in ipairs(memory_points) do
    table.insert(performance_stats.memory_usage, {
        operation = point.label,
        memory = point.memory
    })
    print(string.format("   • %s: ~%d bytes", point.label, point.memory))
end

-- Calculate memory efficiency
local memory_per_event = memory_points[#memory_points].memory / performance_stats.events_published
print(string.format("   📊 Estimated memory per event: ~%.2f bytes", memory_per_event))

print()
print("7. Latency analysis:")

print("   ⏱️  Event latency analysis:")

-- Test end-to-end latency
local latency_sub = Event.subscribe("performance.latency.*")
subscriptions.latency = latency_sub

local latency_tests = {}
for i = 1, 10 do
    local publish_start = os.clock()
    
    local latency_data = {
        test_id = "latency_" .. i,
        publish_timestamp = publish_start,
        payload = string.rep("latency_test_", 10) -- ~120 bytes
    }
    
    local published = Event.publish("performance.latency.test", latency_data)
    local publish_end = os.clock()
    
    if published then
        local receive_start = os.clock()
        local received = Event.receive(latency_sub, 1000) -- 1 second timeout
        local receive_end = os.clock()
        
        if received then
            local publish_latency = (publish_end - publish_start) * 1000
            local receive_latency = (receive_end - receive_start) * 1000
            local total_latency = (receive_end - publish_start) * 1000
            
            table.insert(latency_tests, {
                test_id = i,
                publish_latency = publish_latency,
                receive_latency = receive_latency,
                total_latency = total_latency
            })
            
            print(string.format("   %d. Total: %.2fms (Publish: %.2fms, Receive: %.2fms)", 
                  i, total_latency, publish_latency, receive_latency))
        else
            print(string.format("   %d. ❌ Event not received (timeout)", i))
        end
    end
end

-- Calculate latency statistics
if #latency_tests > 0 then
    local total_latency_sum = 0
    local min_latency = math.huge
    local max_latency = 0
    
    for _, test in ipairs(latency_tests) do
        total_latency_sum = total_latency_sum + test.total_latency
        min_latency = math.min(min_latency, test.total_latency)
        max_latency = math.max(max_latency, test.total_latency)
    end
    
    local avg_latency = total_latency_sum / #latency_tests
    
    print(string.format("   📊 Latency Summary: Avg: %.2fms, Min: %.2fms, Max: %.2fms", 
          avg_latency, min_latency, max_latency))
end

print()
print("8. Pattern matching performance:")

print("   🎯 Pattern matching performance test:")

-- Create subscriptions with different pattern complexities
local pattern_tests = {
    {name = "exact_match", pattern = "performance.pattern.exact"},
    {name = "simple_wildcard", pattern = "performance.pattern.*"},
    {name = "prefix_wildcard", pattern = "*.pattern.prefix"},
    {name = "multi_level", pattern = "performance.*.pattern.*"},
    {name = "complex_pattern", pattern = "performance.pattern.*.complex.*"}
}

local pattern_subs = {}
for _, test in ipairs(pattern_tests) do
    local sub_time = measure_time(function()
        return Event.subscribe(test.pattern)
    end, "Subscribe to " .. test.name)
    
    pattern_subs[test.name] = Event.subscribe(test.pattern)
    subscriptions["pattern_" .. test.name] = pattern_subs[test.name]
end

-- Test events for pattern matching
local pattern_events = {
    "performance.pattern.exact",
    "performance.pattern.simple",
    "data.pattern.prefix",
    "performance.complex.pattern.test",
    "performance.pattern.multi.complex.nested"
}

print("   📤 Publishing pattern test events:")
for i, event_name in ipairs(pattern_events) do
    local pub_time = measure_time(function()
        return Event.publish(event_name, {test_id = i, pattern_test = true})
    end)
    
    print(string.format("   %d. %s (%.2fms)", i, event_name, pub_time))
end

-- Check pattern matching results
print("   📥 Pattern matching results:")
for _, test in ipairs(pattern_tests) do
    local matches = 0
    for attempt = 1, 3 do
        local received = Event.receive(pattern_subs[test.name], 100)
        if received then
            matches = matches + 1
        else
            break
        end
    end
    print(string.format("   • %s (%s): %d matches", 
          test.name, test.pattern, matches))
end

print()
print("9. Performance optimization analysis:")

print("   🔧 Performance optimization analysis:")

-- Analyze throughput test results
print("   📈 Throughput Analysis:")
for _, test in ipairs(performance_stats.throughput_tests) do
    local efficiency = (test.successful / test.event_count) * 100
    print(string.format("   • %s: %.0f events/sec (%.1f%% efficiency)", 
          test.test_type, test.throughput_eps, efficiency))
end

-- Identify bottlenecks
local bottlenecks = {}

-- Check if receiving is slower than publishing
local avg_publish_time = 0
local avg_receive_time = 0

if #performance_stats.publish_times > 0 then
    for _, time in ipairs(performance_stats.publish_times) do
        avg_publish_time = avg_publish_time + time
    end
    avg_publish_time = avg_publish_time / #performance_stats.publish_times
end

if #performance_stats.receive_times > 0 then
    for _, time in ipairs(performance_stats.receive_times) do
        avg_receive_time = avg_receive_time + time
    end
    avg_receive_time = avg_receive_time / #performance_stats.receive_times
end

print("   🔍 Bottleneck Analysis:")
if avg_receive_time > avg_publish_time * 2 then
    table.insert(bottlenecks, "Event receiving is significantly slower than publishing")
end

if performance_stats.events_received < performance_stats.events_published * 0.8 then
    table.insert(bottlenecks, "Event loss detected - only " .. 
                 math.floor((performance_stats.events_received / performance_stats.events_published) * 100) .. 
                 "% of published events were received")
end

if #bottlenecks > 0 then
    for i, bottleneck in ipairs(bottlenecks) do
        print(string.format("   ⚠️  %d. %s", i, bottleneck))
    end
else
    print("   ✅ No significant bottlenecks detected")
end

print()
print("10. Performance recommendations:")

print("   💡 Performance Optimization Recommendations:")

-- Generate recommendations based on test results
local recommendations = {}

local highest_throughput_test = nil
local highest_throughput = 0
for _, test in ipairs(performance_stats.throughput_tests) do
    if test.throughput_eps > highest_throughput then
        highest_throughput = test.throughput_eps
        highest_throughput_test = test
    end
end

if highest_throughput_test then
    table.insert(recommendations, 
                 string.format("Best throughput achieved with %s pattern (%.0f events/sec)", 
                               highest_throughput_test.test_type, highest_throughput))
end

if avg_receive_time > 50 then
    table.insert(recommendations, "Consider reducing receive timeout for better throughput")
end

if performance_stats.events_published > 1000 then
    table.insert(recommendations, "For high-volume scenarios, consider batch processing")
end

table.insert(recommendations, "Monitor memory usage in production environments")
table.insert(recommendations, "Use pattern matching efficiently to avoid performance overhead")
table.insert(recommendations, "Implement subscription cleanup to prevent resource leaks")

for i, recommendation in ipairs(recommendations) do
    print(string.format("   %d. %s", i, recommendation))
end

print()
print("11. Performance summary:")

print("   📊 Performance Test Summary:")
print("   • Total events published:", performance_stats.events_published)
print("   • Total events received:", performance_stats.events_received)
print(string.format("   • Event delivery rate: %.1f%%", 
      (performance_stats.events_received / performance_stats.events_published) * 100))
print(string.format("   • Average publish time: %.2fms", avg_publish_time))
print(string.format("   • Average receive time: %.2fms", avg_receive_time))
print("   • Throughput tests completed:", #performance_stats.throughput_tests)
print("   • Memory usage samples:", #performance_stats.memory_usage)

-- Best and worst performance
local best_throughput = 0
local worst_throughput = math.huge
for _, test in ipairs(performance_stats.throughput_tests) do
    best_throughput = math.max(best_throughput, test.throughput_eps)
    worst_throughput = math.min(worst_throughput, test.throughput_eps)
end

print(string.format("   • Peak throughput: %.0f events/sec", best_throughput))
print(string.format("   • Lowest throughput: %.0f events/sec", worst_throughput))

print()
print("12. Cleaning up performance test subscriptions:")

local cleanup_count = 0
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "performance test subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event performance example complete!")
print("   Key concepts demonstrated:")
print("   • Baseline performance measurement and benchmarking")
print("   • Burst publishing and high-throughput scenarios")
print("   • Concurrent subscription creation and management")
print("   • High-frequency event streaming simulation")
print("   • Bulk event receiving and processing")
print("   • Memory usage analysis and optimization")
print("   • End-to-end latency measurement and analysis")
print("   • Pattern matching performance comparison")
print("   • Bottleneck identification and optimization recommendations")
print("   • Comprehensive performance testing methodology")