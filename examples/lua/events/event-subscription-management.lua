-- ABOUTME: Event subscription lifecycle and cleanup management patterns
-- ABOUTME: Demonstrates subscription creation, monitoring, cleanup, and advanced subscription management

print("=== Event Subscription Management Example ===")
print("Demonstrates: Subscription lifecycle, cleanup patterns, monitoring, and advanced management")
print()

local subscriptions = {}
local subscription_stats = {
    created = 0,
    cleaned_up = 0,
    active = 0,
    failed_creations = 0,
    lifecycle_events = {},
    patterns_used = {},
    performance_metrics = {}
}

-- Helper function to track subscription lifecycle
local function track_subscription_event(event_type, subscription_name, details)
    local entry = {
        timestamp = os.time(),
        event_type = event_type,
        subscription_name = subscription_name,
        details = details or {}
    }
    table.insert(subscription_stats.lifecycle_events, entry)
    print(string.format("   📋 [%s] %s: %s", 
          os.date("%H:%M:%S", entry.timestamp), event_type, subscription_name))
end

-- Helper function to create tracked subscription
local function create_tracked_subscription(name, pattern, category)
    local start_time = os.clock()
    
    local success, sub_id = pcall(function()
        return Event.subscribe(pattern)
    end)
    
    local creation_time = (os.clock() - start_time) * 1000
    
    if success and sub_id then
        subscriptions[name] = {
            id = sub_id,
            pattern = pattern,
            category = category or "general",
            created_at = os.time(),
            creation_time_ms = creation_time,
            events_received = 0,
            last_activity = os.time()
        }
        
        subscription_stats.created = subscription_stats.created + 1
        subscription_stats.active = subscription_stats.active + 1
        subscription_stats.patterns_used[pattern] = (subscription_stats.patterns_used[pattern] or 0) + 1
        
        track_subscription_event("CREATED", name, {
            pattern = pattern,
            category = category,
            creation_time_ms = creation_time
        })
        
        return sub_id
    else
        subscription_stats.failed_creations = subscription_stats.failed_creations + 1
        track_subscription_event("FAILED", name, {
            pattern = pattern,
            error = tostring(sub_id or "unknown error")
        })
        return nil
    end
end

print("1. Basic subscription lifecycle management:")

print("   📡 Creating basic subscriptions:")

-- Create subscriptions with different patterns and categories
local basic_subscriptions = {
    {name = "user_events", pattern = "user.*", category = "user_management"},
    {name = "system_events", pattern = "system.*", category = "system_monitoring"},
    {name = "error_events", pattern = "*.error", category = "error_handling"},
    {name = "auth_events", pattern = "auth.*", category = "authentication"},
    {name = "payment_events", pattern = "payment.*", category = "financial"}
}

for i, sub_info in ipairs(basic_subscriptions) do
    local sub_id = create_tracked_subscription(sub_info.name, sub_info.pattern, sub_info.category)
    if sub_id then
        print(string.format("   %d. ✅ %s (%s)", i, sub_info.name, sub_info.pattern))
    else
        print(string.format("   %d. ❌ %s (%s)", i, sub_info.name, sub_info.pattern))
    end
end

print("   📊 Initial subscription count:", subscription_stats.active)

print()
print("2. Advanced subscription patterns:")

print("   🎯 Creating specialized subscriptions:")

-- Create more complex subscription patterns
local advanced_patterns = {
    {name = "high_priority_alerts", pattern = "*.critical", category = "alerts"},
    {name = "workflow_coordination", pattern = "workflow.*", category = "orchestration"},
    {name = "api_monitoring", pattern = "api.v*.response", category = "api_monitoring"},
    {name = "data_pipeline", pattern = "data.pipeline.*", category = "data_processing"},
    {name = "audit_trail", pattern = "audit.*", category = "compliance"},
    {name = "performance_metrics", pattern = "metrics.*", category = "performance"},
    {name = "notification_delivery", pattern = "notification.*.delivered", category = "notifications"}
}

for i, sub_info in ipairs(advanced_patterns) do
    local sub_id = create_tracked_subscription(sub_info.name, sub_info.pattern, sub_info.category)
    if sub_id then
        print(string.format("   %d. ✅ %s (%s)", i, sub_info.name, sub_info.pattern))
    else
        print(string.format("   %d. ❌ %s (%s)", i, sub_info.name, sub_info.pattern))
    end
end

print("   📊 Total subscriptions after advanced patterns:", subscription_stats.active)

print()
print("3. Subscription monitoring and health checks:")

print("   🔍 Performing subscription health checks:")

-- Health check all subscriptions
local healthy_subscriptions = 0
local unhealthy_subscriptions = 0

for name, sub_info in pairs(subscriptions) do
    -- Check if subscription is still valid (basic health check)
    local is_healthy = sub_info.id ~= nil
    
    if is_healthy then
        healthy_subscriptions = healthy_subscriptions + 1
        print(string.format("   ✅ %s: healthy (age: %ds)", 
              name, os.time() - sub_info.created_at))
    else
        unhealthy_subscriptions = unhealthy_subscriptions + 1
        print(string.format("   ❌ %s: unhealthy", name))
    end
end

print(string.format("   📊 Health check results: %d healthy, %d unhealthy", 
      healthy_subscriptions, unhealthy_subscriptions))

print()
print("4. Subscription activity monitoring:")

print("   📈 Generating test events for activity monitoring:")

-- Generate events to test subscription activity
local test_events = {
    {name = "user.login", data = {user_id = "u123", timestamp = os.time()}},
    {name = "system.startup", data = {service = "web_server", version = "1.0"}},
    {name = "auth.failed", data = {user_id = "u456", reason = "invalid_password"}},
    {name = "payment.completed", data = {amount = 99.99, transaction_id = "tx_789"}},
    {name = "database.error", data = {error_code = "CONNECTION_TIMEOUT"}},
    {name = "workflow.started", data = {workflow_id = "wf_001", user_id = "u123"}},
    {name = "api.v1.response", data = {endpoint = "/users", status = 200}},
    {name = "metrics.cpu", data = {usage_percent = 75.5, timestamp = os.time()}},
    {name = "audit.user.login", data = {user_id = "u123", ip = "192.168.1.1"}},
    {name = "notification.email.delivered", data = {recipient = "user@example.com"}}
}

print("   📤 Publishing test events:")
for i, event in ipairs(test_events) do
    local published = Event.publish(event.name, event.data)
    if published then
        print(string.format("   %d. ✅ %s", i, event.name))
    end
end

print()
print("   📥 Monitoring subscription activity:")

-- Monitor activity on all subscriptions
local activity_results = {}

for name, sub_info in pairs(subscriptions) do
    local events_received = 0
    
    -- Try to receive events with short timeout
    for attempt = 1, 3 do
        local received = Event.receive(sub_info.id, 200) -- 200ms timeout
        if received then
            events_received = events_received + 1
            sub_info.events_received = sub_info.events_received + 1
            sub_info.last_activity = os.time()
        else
            break
        end
    end
    
    activity_results[name] = events_received
    
    if events_received > 0 then
        print(string.format("   📨 %s: %d events received (total: %d)", 
              name, events_received, sub_info.events_received))
        track_subscription_event("ACTIVITY", name, {
            events_received = events_received,
            total_events = sub_info.events_received
        })
    else
        print(string.format("   ⏰ %s: no activity", name))
    end
end

print()
print("5. Subscription categorization and filtering:")

-- Categorize subscripions for management
print("   🏷️  Subscription Categorization:")

local categories = {}

for name, sub_info in pairs(subscriptions) do
    local category = sub_info.category
    if not categories[category] then
        categories[category] = {}
    end
    table.insert(categories[category], {
        name = name,
        pattern = sub_info.pattern,
        events_received = sub_info.events_received,
        age_seconds = os.time() - sub_info.created_at
    })
end

for category, subs in pairs(categories) do
    print(string.format("   • %s: %d subscriptions", category, #subs))
    
    -- Show most active subscriptions in each category
    table.sort(subs, function(a, b) return a.events_received > b.events_received end)
    
    for i = 1, math.min(3, #subs) do
        local sub = subs[i]
        print(string.format("     %d. %s (%s) - %d events", 
              i, sub.name, sub.pattern, sub.events_received))
    end
end

print()
print("6. Inactive subscription detection and cleanup:")

print("   🧹 Detecting inactive subscriptions:")

-- Detect subscriptions that haven't received events
local inactive_threshold = 5 -- seconds
local current_time = os.time()
local inactive_subscriptions = {}
local active_subscriptions = {}

for name, sub_info in pairs(subscriptions) do
    local inactive_time = current_time - sub_info.last_activity
    
    if sub_info.events_received == 0 or inactive_time > inactive_threshold then
        table.insert(inactive_subscriptions, {
            name = name,
            inactive_time = inactive_time,
            events_received = sub_info.events_received
        })
    else
        table.insert(active_subscriptions, name)
    end
end

print(string.format("   📊 Found %d inactive and %d active subscriptions", 
      #inactive_subscriptions, #active_subscriptions))

if #inactive_subscriptions > 0 then
    print("   🔍 Inactive subscriptions:")
    for i, sub in ipairs(inactive_subscriptions) do
        print(string.format("     %d. %s (inactive for %ds, %d events)", 
              i, sub.name, sub.inactive_time, sub.events_received))
    end
end

print()
print("7. Selective subscription cleanup:")

print("   🗑️  Performing selective cleanup:")

-- Clean up some inactive subscriptions (keep some for demonstration)
local cleanup_candidates = {}
for i = 1, math.min(3, #inactive_subscriptions) do
    table.insert(cleanup_candidates, inactive_subscriptions[i].name)
end

for _, name in ipairs(cleanup_candidates) do
    local sub_info = subscriptions[name]
    if sub_info then
        local unsubscribed = Event.unsubscribe(sub_info.id)
        if unsubscribed then
            subscription_stats.cleaned_up = subscription_stats.cleaned_up + 1
            subscription_stats.active = subscription_stats.active - 1
            
            track_subscription_event("CLEANED_UP", name, {
                reason = "inactive",
                events_received = sub_info.events_received,
                lifetime_seconds = os.time() - sub_info.created_at
            })
            
            subscriptions[name] = nil
            print("   ✅ Cleaned up inactive subscription:", name)
        else
            print("   ❌ Failed to clean up subscription:", name)
        end
    end
end

local remaining_subs = Event.list_subscriptions()
print("   📊 Subscriptions after cleanup:", #remaining_subs)

print()
print("8. Subscription performance analysis:")

print("   ⚡ Subscription Performance Analysis:")

-- Analyze creation performance
local creation_times = {}
for _, event in ipairs(subscription_stats.lifecycle_events) do
    if event.event_type == "CREATED" and event.details.creation_time_ms then
        table.insert(creation_times, event.details.creation_time_ms)
    end
end

if #creation_times > 0 then
    local total_time = 0
    local min_time = math.huge
    local max_time = 0
    
    for _, time in ipairs(creation_times) do
        total_time = total_time + time
        min_time = math.min(min_time, time)
        max_time = math.max(max_time, time)
    end
    
    local avg_time = total_time / #creation_times
    
    print(string.format("   • Subscription creation times:"))
    print(string.format("     - Average: %.2fms", avg_time))
    print(string.format("     - Min: %.2fms", min_time))
    print(string.format("     - Max: %.2fms", max_time))
end

-- Analyze pattern popularity
print("   📈 Pattern Usage Analysis:")
local pattern_usage = {}
for pattern, count in pairs(subscription_stats.patterns_used) do
    table.insert(pattern_usage, {pattern = pattern, count = count})
end

table.sort(pattern_usage, function(a, b) return a.count > b.count end)

for i = 1, math.min(5, #pattern_usage) do
    local usage = pattern_usage[i]
    print(string.format("   %d. %s: %d subscriptions", i, usage.pattern, usage.count))
end

print()
print("9. Subscription lifecycle timeline:")

print("   📅 Subscription Lifecycle Timeline:")

-- Show recent lifecycle events
local recent_events = {}
for i = math.max(1, #subscription_stats.lifecycle_events - 9), #subscription_stats.lifecycle_events do
    table.insert(recent_events, subscription_stats.lifecycle_events[i])
end

for i, event in ipairs(recent_events) do
    local time_str = os.date("%H:%M:%S", event.timestamp)
    print(string.format("   %d. [%s] %s - %s", 
          i, time_str, event.event_type, event.subscription_name))
end

print()
print("10. Advanced subscription management patterns:")

print("   🎛️  Advanced Management Patterns:")

-- Demonstrate batch operations
print("   📦 Batch subscription management:")

-- Create multiple subscriptions in batch
local batch_patterns = {
    "batch.test.1.*",
    "batch.test.2.*", 
    "batch.test.3.*"
}

local batch_start = os.clock()
local batch_subs = {}

for i, pattern in ipairs(batch_patterns) do
    local sub_id = create_tracked_subscription("batch_" .. i, pattern, "batch_test")
    if sub_id then
        table.insert(batch_subs, {name = "batch_" .. i, id = sub_id})
    end
end

local batch_time = (os.clock() - batch_start) * 1000
print(string.format("   ✅ Created %d subscriptions in %.2fms", #batch_subs, batch_time))

-- Batch cleanup
local cleanup_start = os.clock()
local cleaned_count = 0

for _, sub in ipairs(batch_subs) do
    if subscriptions[sub.name] then
        local unsubscribed = Event.unsubscribe(sub.id)
        if unsubscribed then
            subscriptions[sub.name] = nil
            cleaned_count = cleaned_count + 1
            subscription_stats.cleaned_up = subscription_stats.cleaned_up + 1
            subscription_stats.active = subscription_stats.active - 1
        end
    end
end

local cleanup_time = (os.clock() - cleanup_start) * 1000
print(string.format("   🧹 Cleaned up %d subscriptions in %.2fms", cleaned_count, cleanup_time))

print()
print("11. Subscription management best practices:")

print("   💡 Subscription Management Best Practices:")
print("   • Monitor subscription health and activity regularly")
print("   • Clean up inactive subscriptions to prevent resource leaks")
print("   • Use descriptive names and categorization for management")
print("   • Track subscription lifecycle events for debugging")
print("   • Implement batch operations for efficiency")
print("   • Set appropriate timeouts for event receiving")
print("   • Monitor subscription creation performance")
print("   • Use pattern analysis to optimize subscription strategies")
print("   • Implement graceful cleanup in error scenarios")
print("   • Document subscription patterns and their purposes")

print()
print("12. Final cleanup and statistics:")

print("   📊 Session Statistics:")
print("   • Total subscriptions created:", subscription_stats.created)
print("   • Total subscriptions cleaned up:", subscription_stats.cleaned_up)
print("   • Currently active subscriptions:", subscription_stats.active)
print("   • Failed subscription creations:", subscription_stats.failed_creations)
print("   • Lifecycle events recorded:", #subscription_stats.lifecycle_events)
print("   • Unique patterns used:", (function()
    local count = 0
    for _ in pairs(subscription_stats.patterns_used) do count = count + 1 end
    return count
end)())

print()
print("   🧹 Final cleanup of remaining subscriptions:")

local final_cleanup_count = 0
for name, sub_info in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_info.id)
    if unsubscribed then
        final_cleanup_count = final_cleanup_count + 1
        track_subscription_event("FINAL_CLEANUP", name, {
            events_received = sub_info.events_received,
            lifetime_seconds = os.time() - sub_info.created_at
        })
        print("   🧹 Cleaned up", name)
    end
end

local final_subs = Event.list_subscriptions()
print("   ✅ Cleaned up", final_cleanup_count, "remaining subscriptions")
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Event subscription management example complete!")
print("   Key concepts demonstrated:")
print("   • Subscription lifecycle tracking and management")
print("   • Health monitoring and activity detection")
print("   • Categorization and filtering of subscriptions")
print("   • Inactive subscription detection and cleanup")
print("   • Performance analysis and optimization")
print("   • Batch subscription operations")
print("   • Advanced management patterns and best practices")
print("   • Comprehensive cleanup and resource management")