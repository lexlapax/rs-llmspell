-- ABOUTME: Event pattern matching with wildcards and advanced filtering
-- ABOUTME: Demonstrates complex pattern matching including *.error, user.*, hierarchical patterns

print("=== Event Pattern Matching Example ===")
print("Demonstrates: Advanced event pattern matching and filtering")
print()

local subscriptions = {}
local pattern_stats = {
    patterns_tested = 0,
    events_published = 0,
    matches_found = 0,
    pattern_types = {}
}

print("1. Wildcard pattern subscriptions:")

-- Create various wildcard pattern subscriptions
local patterns = {
    -- Suffix wildcards
    user_all = "user.*",          -- All user events
    system_all = "system.*",      -- All system events
    database_all = "database.*",  -- All database events
    
    -- Prefix wildcards  
    all_errors = "*.error",       -- All error events
    all_warnings = "*.warning",   -- All warning events
    all_success = "*.success",    -- All success events
    all_info = "*.info",         -- All info events
    
    -- Multi-level patterns
    api_endpoints = "api.*.*",    -- api.v1.users, api.v2.orders, etc.
    user_actions = "user.action.*", -- user.action.login, user.action.logout, etc.
    system_health = "system.health.*", -- system.health.cpu, system.health.memory, etc.
    
    -- Specific patterns
    auth_events = "auth.*",       -- Authentication events
    payment_events = "payment.*", -- Payment events
    notification_events = "notification.*" -- Notification events
}

print("   📡 Creating pattern subscriptions:")
for pattern_name, pattern in pairs(patterns) do
    subscriptions[pattern_name] = Event.subscribe(pattern)
    pattern_stats.patterns_tested = pattern_stats.patterns_tested + 1
    print(string.format("   • %s: %s", pattern_name, pattern))
end

print(string.format("   ✅ Created %d pattern subscriptions", pattern_stats.patterns_tested))

print()
print("2. Publishing events to test pattern matching:")

-- Test events designed to match different patterns
local test_events = {
    -- User events (should match user.* and specific patterns)
    {name = "user.login", data = {user_id = "u123", method = "password", ip = "192.168.1.1"}},
    {name = "user.logout", data = {user_id = "u123", session_duration = 1800}},
    {name = "user.profile.update", data = {user_id = "u123", field = "email"}},
    {name = "user.action.view", data = {user_id = "u123", page = "/dashboard"}},
    {name = "user.action.click", data = {user_id = "u123", element = "button_submit"}},
    
    -- System events (should match system.* and specific patterns)
    {name = "system.startup", data = {service = "web_server", version = "1.2.3"}},
    {name = "system.shutdown", data = {service = "web_server", reason = "maintenance"}},
    {name = "system.health.cpu", data = {usage = 75.5, cores = 4}},
    {name = "system.health.memory", data = {used = 8192, total = 16384}},
    
    -- Database events
    {name = "database.connection", data = {host = "db.example.com", status = "connected"}},
    {name = "database.query", data = {table = "users", duration_ms = 45}},
    
    -- Error events (should match *.error pattern)
    {name = "user.error", data = {error_code = "AUTH_FAILED", user_id = "u456"}},
    {name = "system.error", data = {error_code = "OUT_OF_MEMORY", severity = "critical"}},
    {name = "database.error", data = {error_code = "CONNECTION_TIMEOUT", query = "SELECT * FROM users"}},
    {name = "api.error", data = {endpoint = "/api/v1/users", status_code = 500}},
    
    -- Warning events (should match *.warning pattern)
    {name = "system.warning", data = {message = "High CPU usage", threshold = 80}},
    {name = "database.warning", data = {message = "Slow query detected", duration_ms = 5000}},
    
    -- Success events (should match *.success pattern)
    {name = "payment.success", data = {transaction_id = "tx_789", amount = 99.99}},
    {name = "user.success", data = {action = "password_reset", user_id = "u789"}},
    
    -- Info events (should match *.info pattern)
    {name = "system.info", data = {message = "Backup completed", size_mb = 1024}},
    {name = "user.info", data = {message = "Profile updated", user_id = "u123"}},
    
    -- API events (should match api.*.* pattern)
    {name = "api.v1.users", data = {method = "GET", count = 10}},
    {name = "api.v1.orders", data = {method = "POST", order_id = "ord_456"}},
    {name = "api.v2.products", data = {method = "PUT", product_id = "prod_789"}},
    
    -- Auth events
    {name = "auth.login", data = {user_id = "u123", method = "2fa"}},
    {name = "auth.logout", data = {user_id = "u123", reason = "timeout"}},
    {name = "auth.token.refresh", data = {user_id = "u123", token_type = "jwt"}},
    
    -- Payment events
    {name = "payment.initiate", data = {amount = 49.99, currency = "USD"}},
    {name = "payment.process", data = {gateway = "stripe", status = "processing"}},
    {name = "payment.complete", data = {transaction_id = "tx_101", status = "completed"}},
    
    -- Notification events
    {name = "notification.email", data = {recipient = "user@example.com", template = "welcome"}},
    {name = "notification.sms", data = {phone = "+1234567890", message = "OTP: 123456"}},
    {name = "notification.push", data = {device_id = "dev_456", title = "New message"}}
}

print("   📤 Publishing test events:")
for i, event in ipairs(test_events) do
    local published = Event.publish(event.name, event.data)
    if published then
        pattern_stats.events_published = pattern_stats.events_published + 1
        print(string.format("   %2d. ✅ %s", i, event.name))
    else
        print(string.format("   %2d. ❌ %s", i, event.name))
    end
end

print(string.format("   📊 Published %d test events", pattern_stats.events_published))

print()
print("3. Receiving and analyzing pattern matches:")

-- Check each pattern subscription for matches
print("   📥 Checking pattern matches:")

local pattern_results = {}

for pattern_name, sub_id in pairs(subscriptions) do
    local matches = {}
    local pattern_string = patterns[pattern_name]
    
    -- Try to receive multiple events (some patterns may match multiple events)
    for attempt = 1, 5 do
        local received = Event.receive(sub_id, 200) -- 200ms timeout per attempt
        if received then
            table.insert(matches, {
                event_type = received.event_type or "unknown",
                data_keys = (function()
                    if received.data then
                        local keys = {}
                        for key, _ in pairs(received.data) do
                            table.insert(keys, key)
                        end
                        return keys
                    end
                    return {}
                end)()
            })
            pattern_stats.matches_found = pattern_stats.matches_found + 1
        else
            break -- No more events for this pattern
        end
    end
    
    pattern_results[pattern_name] = {
        pattern = pattern_string,
        matches = matches,
        match_count = #matches
    }
    
    if #matches > 0 then
        print(string.format("   🎯 %-20s (%s): %d matches", pattern_name, pattern_string, #matches))
        for i, match in ipairs(matches) do
            print(string.format("     %d. %s (keys: %s)", i, match.event_type, 
                  table.concat(match.data_keys, ", ")))
        end
    else
        print(string.format("   ⚪ %-20s (%s): no matches", pattern_name, pattern_string))
    end
end

print()
print("4. Pattern matching analysis:")

-- Analyze pattern effectiveness
print("   📊 Pattern Matching Analysis:")

local effective_patterns = 0
local total_matches = 0

for pattern_name, result in pairs(pattern_results) do
    if result.match_count > 0 then
        effective_patterns = effective_patterns + 1
        total_matches = total_matches + result.match_count
        
        -- Categorize pattern types
        local pattern_type = "unknown"
        if result.pattern:find("^%*%.") then
            pattern_type = "prefix_wildcard"
        elseif result.pattern:find("%*$") then
            pattern_type = "suffix_wildcard"
        elseif result.pattern:find("%*.*%*") then
            pattern_type = "multi_wildcard"
        else
            pattern_type = "specific"
        end
        
        pattern_stats.pattern_types[pattern_type] = (pattern_stats.pattern_types[pattern_type] or 0) + 1
    end
end

print(string.format("   • Effective patterns: %d/%d (%.1f%%)", 
      effective_patterns, pattern_stats.patterns_tested, 
      (effective_patterns / pattern_stats.patterns_tested) * 100))
print("   • Total matches found:", total_matches)
print("   • Average matches per effective pattern:", 
      effective_patterns > 0 and string.format("%.1f", total_matches / effective_patterns) or "0")

print()
print("   🏷️  Pattern Type Distribution:")
for pattern_type, count in pairs(pattern_stats.pattern_types) do
    print(string.format("   • %s: %d patterns", pattern_type:gsub("_", " "), count))
end

print()
print("5. Advanced pattern matching scenarios:")

-- Test edge cases and complex patterns
print("   🔬 Testing edge cases and complex patterns:")

local advanced_patterns = {
    empty_pattern = "",           -- Should match nothing
    exact_match = "user.login",   -- Should match exactly
    deep_nesting = "api.v1.users.profile.avatar", -- Deep hierarchy
    numbers_in_pattern = "metric.cpu.core.1", -- Numbers in event names
    special_chars = "event-with-dashes", -- Special characters
}

print("   📡 Creating advanced pattern subscriptions:")
local advanced_subs = {}
for pattern_name, pattern in pairs(advanced_patterns) do
    local success, sub_id = pcall(function()
        return Event.subscribe(pattern)
    end)
    
    if success then
        advanced_subs[pattern_name] = sub_id
        print(string.format("   ✅ %s: '%s'", pattern_name, pattern))
    else
        print(string.format("   ❌ %s: '%s' (failed)", pattern_name, pattern))
    end
end

-- Publish events for advanced pattern testing
local advanced_events = {
    "user.login",                    -- Should match exact_match
    "api.v1.users.profile.avatar",  -- Should match deep_nesting
    "metric.cpu.core.1",            -- Should match numbers_in_pattern
    "event-with-dashes",            -- Should match special_chars
    "random.event.that.matches.nothing" -- Should match nothing specific
}

print("   📤 Publishing advanced test events:")
for i, event_name in ipairs(advanced_events) do
    local published = Event.publish(event_name, {test_id = i, advanced_test = true})
    if published then
        print(string.format("   %d. ✅ %s", i, event_name))
    end
end

-- Check advanced pattern matches
print("   📥 Checking advanced pattern matches:")
for pattern_name, sub_id in pairs(advanced_subs) do
    local received = Event.receive(sub_id, 300)
    if received then
        print(string.format("   🎯 %s matched: %s", pattern_name, received.event_type or "unknown"))
    else
        print(string.format("   ⚪ %s: no match", pattern_name))
    end
end

print()
print("6. Pattern performance analysis:")

-- Measure pattern matching performance
print("   ⚡ Pattern Performance Analysis:")

local performance_start = os.clock() 

-- Create many subscriptions quickly
local perf_subs = {}
for i = 1, 20 do
    perf_subs[i] = Event.subscribe("perf.test." .. i .. ".*")
end

local subscription_time = (os.clock() - performance_start) * 1000

-- Publish events quickly
local publish_start = os.clock()
for i = 1, 50 do
    Event.publish("perf.test." .. (i % 20 + 1) .. ".event", {seq = i})
end
local publish_time = (os.clock() - publish_start) * 1000

-- Receive events quickly
local receive_start = os.clock()
local received_count = 0
for i = 1, 20 do
    local received = Event.receive(perf_subs[i], 50) -- Short timeout
    if received then
        received_count = received_count + 1
    end
end
local receive_time = (os.clock() - receive_start) * 1000

print(string.format("   • Subscription creation: %.2fms (20 patterns)", subscription_time))
print(string.format("   • Event publishing: %.2fms (50 events)", publish_time))
print(string.format("   • Event receiving: %.2fms (%d events)", receive_time, received_count))

-- Clean up performance test subscriptions
for i = 1, 20 do
    Event.unsubscribe(perf_subs[i])
end

print()
print("7. Pattern best practices:")

print("   💡 Pattern Matching Best Practices:")
print("   • Use specific patterns when possible (user.login vs user.*)")
print("   • Prefix wildcards (*.error) are useful for cross-component error handling")
print("   • Suffix wildcards (user.*) are good for component-specific filtering")
print("   • Avoid overly broad patterns that match too many events")
print("   • Use hierarchical naming for better pattern organization")
print("   • Consider performance impact of complex pattern matching")
print("   • Test pattern effectiveness with representative event data")
print("   • Document pattern conventions in your application")

print()
print("8. Pattern matching summary:")

print("   📈 Session Pattern Summary:")
print("   • Total patterns tested:", pattern_stats.patterns_tested + #advanced_patterns)
print("   • Events published for testing:", pattern_stats.events_published + #advanced_events)
print("   • Total pattern matches found:", pattern_stats.matches_found)
print("   • Pattern categories tested: suffix wildcards, prefix wildcards, multi-level, exact match")
print("   • Edge cases tested: empty patterns, deep nesting, special characters")

print()
print("9. Cleaning up pattern subscriptions:")

-- Clean up all subscriptions
local cleanup_count = 0

for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name)
    end
end

for name, sub_id in pairs(advanced_subs) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        cleanup_count = cleanup_count + 1
        print("   🧹 Unsubscribed from", name, "(advanced)")
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", cleanup_count, "pattern subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event pattern matching example complete!")
print("   Key concepts demonstrated:")
print("   • Wildcard pattern matching with * placeholder")
print("   • Suffix patterns (user.*) for component-specific events")
print("   • Prefix patterns (*.error) for cross-component event types")
print("   • Multi-level patterns (api.*.*) for hierarchical events")
print("   • Exact match patterns for specific event targeting")
print("   • Pattern performance characteristics and optimization")
print("   • Edge case handling and complex pattern scenarios")