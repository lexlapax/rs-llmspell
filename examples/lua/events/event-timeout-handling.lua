-- ABOUTME: Event timeouts and error handling patterns with graceful degradation
-- ABOUTME: Demonstrates timeout scenarios, error recovery, retry mechanisms, and robust event handling

print("=== Event Timeout Handling Example ===")
print("Demonstrates: Timeout patterns, error recovery, retry mechanisms, and robust event handling")
print()

local subscriptions = {}
local timeout_stats = {
    timeout_tests = 0,
    successful_receives = 0,
    timeout_occurrences = 0,
    error_recoveries = 0,
    retry_attempts = 0,
    scenarios_tested = {}
}

-- Helper function to track timeout scenarios
local function track_timeout_scenario(scenario_name, result, duration, details)
    timeout_stats.timeout_tests = timeout_stats.timeout_tests + 1
    timeout_stats.scenarios_tested[scenario_name] = {
        result = result,
        duration = duration,
        details = details or {},
        timestamp = os.time()
    }
    
    if result == "success" then
        timeout_stats.successful_receives = timeout_stats.successful_receives + 1
    elseif result == "timeout" then
        timeout_stats.timeout_occurrences = timeout_stats.timeout_occurrences + 1
    end
    
    print(string.format("   📋 [%s] Scenario: %s (%.2fms)", 
          result == "success" and "✅" or "⏰", scenario_name, duration))
end

-- Helper function to measure timeout scenarios
local function test_timeout_scenario(scenario_name, subscription, timeout_ms, expected_result, description)
    print(string.format("   🔍 Testing: %s", description))
    
    local start_time = os.clock()
    local received = Event.receive(subscription, timeout_ms)
    local end_time = os.clock()
    
    local duration = (end_time - start_time) * 1000
    local result = received and "success" or "timeout"
    
    track_timeout_scenario(scenario_name, result, duration, {
        expected = expected_result,
        timeout_ms = timeout_ms,
        description = description
    })
    
    if received then
        print(string.format("   ✅ Event received: %s", received.event_type or "unknown"))
        return received
    else
        print(string.format("   ⏰ Timeout after %.2fms (expected %dms)", duration, timeout_ms))
        return nil
    end
end

print("1. Basic timeout scenarios:")

print("   📡 Setting up timeout test subscriptions:")

-- Create subscriptions for different timeout scenarios
local timeout_patterns = {
    immediate = "timeout.immediate.*",
    short = "timeout.short.*",
    medium = "timeout.medium.*",
    long = "timeout.long.*",
    variable = "timeout.variable.*"
}

for pattern_name, pattern in pairs(timeout_patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print("   ✅ Timeout test subscriptions created")

print()
print("2. Immediate availability scenarios:")

print("   ⚡ Testing immediate event availability:")

-- Publish events that should be immediately available
local immediate_events = {
    {name = "timeout.immediate.available", data = {test_id = "immediate_001", ready = true}},
    {name = "timeout.short.ready", data = {test_id = "short_001", status = "ready"}},
    {name = "timeout.medium.prepared", data = {test_id = "medium_001", prepared = true}}
}

for i, event in ipairs(immediate_events) do
    Event.publish(event.name, event.data)
    print(string.format("   %d. ✅ Published: %s", i, event.name))
end

-- Test immediate receives (events should be available)
test_timeout_scenario("immediate_available", subscriptions.immediate, 100, "success",
                     "Receive immediately available event (100ms timeout)")

test_timeout_scenario("short_available", subscriptions.short, 500, "success",
                     "Receive short-timeout event (500ms timeout)")

test_timeout_scenario("medium_available", subscriptions.medium, 1000, "success",
                     "Receive medium-timeout event (1000ms timeout)")

print()
print("3. Guaranteed timeout scenarios:")

print("   ⏰ Testing guaranteed timeout scenarios:")

-- Test scenarios where no events are available (guaranteed timeouts)
test_timeout_scenario("no_events_short", subscriptions.long, 200, "timeout",
                     "No events available with short timeout (200ms)")

test_timeout_scenario("no_events_medium", subscriptions.variable, 500, "timeout",
                     "No events available with medium timeout (500ms)")

test_timeout_scenario("no_events_long", subscriptions.long, 1000, "timeout",
                     "No events available with long timeout (1000ms)")

print()
print("4. Delayed event scenarios:")

print("   ⌛ Testing delayed event availability:")

-- Create a function to publish events after delay (simulated)
local function simulate_delayed_publish(event_name, data, delay_description)
    print(string.format("   📤 Publishing delayed event: %s (%s)", event_name, delay_description))
    Event.publish(event_name, data)
end

-- Test scenarios with different timeout vs. availability timing
print("   🔄 Scenario: Event published immediately, timeout sufficient")
simulate_delayed_publish("timeout.variable.delayed_short", 
                        {test_id = "delayed_001", delay = "minimal"}, "minimal delay")
test_timeout_scenario("delayed_short_sufficient", subscriptions.variable, 1000, "success",
                     "Sufficient timeout for delayed event (1000ms)")

print("   🔄 Scenario: Testing timeout boundary conditions")
-- Test very short timeouts
test_timeout_scenario("very_short_timeout", subscriptions.immediate, 1, "timeout",
                     "Extremely short timeout (1ms)")

test_timeout_scenario("zero_timeout", subscriptions.immediate, 0, "timeout",
                     "Zero timeout (0ms)")

print()
print("5. Retry mechanism with timeout handling:")

print("   🔄 Implementing retry mechanisms:")

-- Retry function with exponential backoff
local function retry_with_timeout(subscription, max_attempts, initial_timeout, backoff_factor, description)
    print(string.format("   🎯 Retry scenario: %s", description))
    
    local attempts = 0
    local current_timeout = initial_timeout
    
    while attempts < max_attempts do
        attempts = attempts + 1
        timeout_stats.retry_attempts = timeout_stats.retry_attempts + 1
        
        print(string.format("   • Attempt %d/%d (timeout: %dms)", attempts, max_attempts, current_timeout))
        
        local start_time = os.clock()
        local received = Event.receive(subscription, current_timeout)
        local duration = (os.clock() - start_time) * 1000
        
        if received then
            print(string.format("   ✅ Success on attempt %d (%.2fms)", attempts, duration))
            timeout_stats.successful_receives = timeout_stats.successful_receives + 1
            timeout_stats.error_recoveries = timeout_stats.error_recoveries + 1
            return received, attempts
        else
            print(string.format("   ⏰ Timeout on attempt %d (%.2fms)", attempts, duration))
            timeout_stats.timeout_occurrences = timeout_stats.timeout_occurrences + 1
            
            -- Increase timeout for next attempt (exponential backoff)
            current_timeout = math.floor(current_timeout * backoff_factor)
            
            if attempts < max_attempts then
                print(string.format("   ⏳ Next timeout will be %dms", current_timeout))
            end
        end
    end
    
    print(string.format("   ❌ All %d attempts failed", max_attempts))
    return nil, attempts
end

-- Test retry scenarios
-- First, publish some events for successful retry
Event.publish("timeout.variable.retry_test", {test_id = "retry_001", available_after_retry = true})

-- Test successful retry (event is available)
retry_with_timeout(subscriptions.variable, 3, 100, 2.0, 
                  "Retry with available event (exponential backoff)")

-- Test failed retry (no events available)
retry_with_timeout(subscriptions.long, 3, 50, 1.5, 
                  "Retry with no available events (linear backoff)")

print()
print("6. Graceful degradation patterns:")

print("   🛡️  Implementing graceful degradation:")

-- Function to handle timeouts gracefully with fallback strategies
local function graceful_timeout_handler(subscription, timeout_ms, fallback_strategies)
    print("   🎯 Graceful timeout handling with fallback strategies")
    
    local start_time = os.clock()
    local received = Event.receive(subscription, timeout_ms)
    local duration = (os.clock() - start_time) * 1000
    
    if received then
        print(string.format("   ✅ Primary strategy succeeded (%.2fms)", duration))
        return received, "primary"
    else
        print(string.format("   ⏰ Primary strategy timed out (%.2fms)", duration))
        
        -- Try fallback strategies
        for i, strategy in ipairs(fallback_strategies) do
            print(string.format("   🔄 Trying fallback strategy %d: %s", i, strategy.description))
            
            local fallback_start = os.clock()
            local fallback_result = strategy.action()
            local fallback_duration = (os.clock() - fallback_start) * 1000
            
            if fallback_result then
                print(string.format("   ✅ Fallback strategy %d succeeded (%.2fms)", i, fallback_duration))
                timeout_stats.error_recoveries = timeout_stats.error_recoveries + 1
                return fallback_result, "fallback_" .. i
            else
                print(string.format("   ❌ Fallback strategy %d failed (%.2fms)", i, fallback_duration))
            end
        end
        
        print("   ❌ All strategies exhausted, graceful failure")
        return nil, "failed"
    end
end

-- Define fallback strategies
local fallback_strategies = {
    {
        description = "Try alternative subscription with shorter timeout",
        action = function()
            return Event.receive(subscriptions.short, 200) -- Alternative subscription
        end
    },
    {
        description = "Use cached/default data",
        action = function()
            -- Simulate returning cached data
            return {
                event_type = "timeout.fallback.cached",
                data = {
                    source = "cache",
                    cached_at = os.time() - 300,
                    reliability = "fallback"
                }
            }
        end
    },
    {
        description = "Generate synthetic event",
        action = function()
            -- Simulate generating synthetic data
            return {
                event_type = "timeout.fallback.synthetic",
                data = {
                    source = "synthetic",
                    generated_at = os.time(),
                    reliability = "synthetic"
                }
            }
        end
    }
}

-- Test graceful degradation (no events available, should use fallback)
graceful_timeout_handler(subscriptions.long, 300, fallback_strategies)

-- Publish an event and test graceful degradation (should use primary)
Event.publish("timeout.short.graceful", {test_id = "graceful_001", primary = true})
graceful_timeout_handler(subscriptions.short, 500, fallback_strategies)

print()
print("7. Timeout error recovery patterns:")

print("   🚑 Testing error recovery patterns:")

-- Circuit breaker pattern for timeout handling
local circuit_breaker = {
    failure_count = 0,
    failure_threshold = 3,
    recovery_timeout = 2000, -- 2 seconds
    last_failure_time = 0,
    state = "closed" -- closed, open, half_open
}

local function circuit_breaker_receive(subscription, timeout_ms, description)
    print(string.format("   🔌 Circuit breaker receive: %s (state: %s)", description, circuit_breaker.state))
    
    local current_time = os.time() * 1000 -- Convert to milliseconds
    
    -- Check circuit breaker state
    if circuit_breaker.state == "open" then
        if current_time - circuit_breaker.last_failure_time > circuit_breaker.recovery_timeout then
            circuit_breaker.state = "half_open"
            print("   🔄 Circuit breaker transitioning to half-open")
        else
            print("   🚫 Circuit breaker is open, request blocked")
            return nil, "circuit_open"
        end
    end
    
    -- Attempt to receive event
    local start_time = os.clock()
    local received = Event.receive(subscription, timeout_ms)
    local duration = (os.clock() - start_time) * 1000
    
    if received then
        print(string.format("   ✅ Circuit breaker receive succeeded (%.2fms)", duration))
        
        -- Reset circuit breaker on success
        if circuit_breaker.state == "half_open" then
            circuit_breaker.state = "closed"
            circuit_breaker.failure_count = 0
            print("   🔄 Circuit breaker reset to closed")
        end
        
        return received, "success"
    else
        print(string.format("   ⏰ Circuit breaker receive timed out (%.2fms)", duration))
        
        -- Handle failure
        circuit_breaker.failure_count = circuit_breaker.failure_count + 1
        circuit_breaker.last_failure_time = current_time
        
        if circuit_breaker.failure_count >= circuit_breaker.failure_threshold then
            circuit_breaker.state = "open"
            print(string.format("   ⚠️  Circuit breaker opened after %d failures", circuit_breaker.failure_count))
        end
        
        return nil, "timeout"
    end
end

-- Test circuit breaker pattern
circuit_breaker_receive(subscriptions.long, 200, "Test 1 - should timeout")
circuit_breaker_receive(subscriptions.long, 200, "Test 2 - should timeout")  
circuit_breaker_receive(subscriptions.long, 200, "Test 3 - should timeout and open circuit")
circuit_breaker_receive(subscriptions.long, 200, "Test 4 - should be blocked by circuit")

print()
print("8. Timeout monitoring and alerting:")

print("   📊 Timeout monitoring system:")

-- Timeout monitoring system
local timeout_monitor = {
    timeout_counts = {},
    threshold_alerts = {},
    monitoring_window = 60 -- seconds
}

local function monitor_timeout_event(subscription_name, timeout_occurred, timeout_duration)
    local current_time = os.time()
    
    if not timeout_monitor.timeout_counts[subscription_name] then
        timeout_monitor.timeout_counts[subscription_name] = {
            total_timeouts = 0,
            recent_timeouts = {},
            last_alert = 0
        }
    end
    
    local sub_monitor = timeout_monitor.timeout_counts[subscription_name]
    
    if timeout_occurred then
        sub_monitor.total_timeouts = sub_monitor.total_timeouts + 1
        table.insert(sub_monitor.recent_timeouts, {
            timestamp = current_time,
            duration = timeout_duration
        })
        
        -- Clean old timeout records (outside monitoring window)
        local cutoff_time = current_time - timeout_monitor.monitoring_window
        local cleaned_timeouts = {}
        for _, timeout_record in ipairs(sub_monitor.recent_timeouts) do
            if timeout_record.timestamp > cutoff_time then
                table.insert(cleaned_timeouts, timeout_record)
            end
        end
        sub_monitor.recent_timeouts = cleaned_timeouts
        
        -- Check for alert conditions
        local recent_count = #sub_monitor.recent_timeouts
        if recent_count >= 3 and (current_time - sub_monitor.last_alert) > 30 then
            print(string.format("   🚨 ALERT: %s has %d timeouts in the last %ds", 
                  subscription_name, recent_count, timeout_monitor.monitoring_window))
            sub_monitor.last_alert = current_time
        end
    end
    
    print(string.format("   📈 Monitor: %s - Total: %d, Recent: %d", 
          subscription_name, sub_monitor.total_timeouts, #sub_monitor.recent_timeouts))
end

-- Test monitoring system
print("   🔍 Testing timeout monitoring:")
monitor_timeout_event("test_subscription", true, 1000)
monitor_timeout_event("test_subscription", true, 1200)
monitor_timeout_event("test_subscription", true, 800)
monitor_timeout_event("test_subscription", false, 0)

print()
print("9. Advanced timeout patterns:")

print("   🎯 Advanced timeout handling patterns:")

-- Adaptive timeout pattern
local adaptive_timeout = {
    base_timeout = 500,
    adjustment_factor = 0.1,
    min_timeout = 100,
    max_timeout = 5000,
    recent_durations = {}
}

local function adaptive_timeout_receive(subscription, description)
    print(string.format("   🎯 Adaptive timeout: %s", description))
    
    -- Calculate adaptive timeout based on recent performance
    local current_timeout = adaptive_timeout.base_timeout
    
    if #adaptive_timeout.recent_durations > 0 then
        local sum = 0
        for _, duration in ipairs(adaptive_timeout.recent_durations) do
            sum = sum + duration
        end
        local avg_duration = sum / #adaptive_timeout.recent_durations
        
        -- Adjust timeout based on average duration
        current_timeout = math.floor(avg_duration * 1.5) -- 50% buffer
        current_timeout = math.max(adaptive_timeout.min_timeout,
                                  math.min(adaptive_timeout.max_timeout, current_timeout))
    end
    
    print(string.format("   ⏱️  Using adaptive timeout: %dms", current_timeout))
    
    local start_time = os.clock()
    local received = Event.receive(subscription, current_timeout)
    local duration = (os.clock() - start_time) * 1000
    
    -- Record duration for future adaptive calculations
    table.insert(adaptive_timeout.recent_durations, duration)
    if #adaptive_timeout.recent_durations > 10 then
        table.remove(adaptive_timeout.recent_durations, 1) -- Keep only recent 10
    end
    
    if received then
        print(string.format("   ✅ Adaptive receive succeeded (%.2fms)", duration))
    else
        print(string.format("   ⏰ Adaptive receive timed out (%.2fms)", duration))
    end
    
    return received
end

-- Test adaptive timeout
adaptive_timeout_receive(subscriptions.medium, "First adaptive attempt")
adaptive_timeout_receive(subscriptions.medium, "Second adaptive attempt")
adaptive_timeout_receive(subscriptions.medium, "Third adaptive attempt")

print()
print("10. Timeout handling best practices:")

print("   💡 Timeout Handling Best Practices:")
print("   • Set appropriate timeouts based on expected response times")
print("   • Implement retry mechanisms with exponential backoff")
print("   • Use circuit breaker patterns to prevent cascade failures")
print("   • Provide graceful degradation with fallback strategies")
print("   • Monitor timeout patterns for system health insights")
print("   • Use adaptive timeouts based on historical performance")
print("   • Implement proper error recovery and cleanup")
print("   • Log timeout events for debugging and analysis")
print("   • Consider user experience when designing timeout strategies")
print("   • Test timeout scenarios thoroughly in development")

print()
print("11. Timeout statistics summary:")

print("   📊 Timeout Test Statistics:")
print("   • Total timeout tests:", timeout_stats.timeout_tests)
print("   • Successful receives:", timeout_stats.successful_receives)
print("   • Timeout occurrences:", timeout_stats.timeout_occurrences)
print("   • Error recoveries:", timeout_stats.error_recoveries)
print("   • Retry attempts:", timeout_stats.retry_attempts)

-- Calculate success rate
local success_rate = timeout_stats.timeout_tests > 0 and 
                    (timeout_stats.successful_receives / timeout_stats.timeout_tests) * 100 or 0

print(string.format("   • Overall success rate: %.1f%%", success_rate))

-- Recovery rate
local recovery_rate = timeout_stats.timeout_occurrences > 0 and
                     (timeout_stats.error_recoveries / timeout_stats.timeout_occurrences) * 100 or 0

print(string.format("   • Error recovery rate: %.1f%%", recovery_rate))

print()
print("   🎯 Scenario Results:")
for scenario_name, result in pairs(timeout_stats.scenarios_tested) do
    local status_icon = result.result == "success" and "✅" or "⏰"
    print(string.format("   %s %s: %s (%.2fms)", status_icon, scenario_name, result.result, result.duration))
end

print()
print("12. Cleaning up timeout test subscriptions:")

local cleanup_count = 0
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "timeout test subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event timeout handling example complete!")
print("   Key concepts demonstrated:")
print("   • Basic timeout scenarios with various durations")
print("   • Immediate vs. delayed event availability patterns")
print("   • Retry mechanisms with exponential backoff strategies")
print("   • Graceful degradation with multiple fallback options")
print("   • Circuit breaker pattern for fault tolerance")
print("   • Timeout monitoring and alerting systems")
print("   • Adaptive timeout calculation based on performance")
print("   • Comprehensive error recovery and cleanup patterns")
print("   • Best practices for robust timeout handling")
print("   • Statistical analysis of timeout behavior and recovery")