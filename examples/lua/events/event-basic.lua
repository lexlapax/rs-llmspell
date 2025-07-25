-- ABOUTME: Basic event publish/subscribe patterns and fundamental event system usage
-- ABOUTME: Demonstrates Event.publish(), Event.subscribe(), Event.receive(), Event.unsubscribe()

print("=== Basic Event Patterns Example ===")
print("Demonstrates: Fundamental event system operations")
print()

local subscriptions = {}
local published_events = 0
local received_events = 0

print("1. Basic event subscription:")

-- Subscribe to different event patterns
subscriptions.user_events = Event.subscribe("user.*")
print("   📡 Subscribed to user.* events")

subscriptions.system_events = Event.subscribe("system.*") 
print("   📡 Subscribed to system.* events")

subscriptions.test_events = Event.subscribe("test.basic.*")
print("   📡 Subscribed to test.basic.* events")

-- List current subscriptions
local all_subscriptions = Event.list_subscriptions()
print("   📋 Total active subscriptions:", #all_subscriptions)

print()
print("2. Basic event publishing:")

-- Publish simple events
print("   📤 Publishing user events...")
local user_login = Event.publish("user.login", {
    user_id = "user123",
    username = "john_doe",
    login_time = os.time(),
    ip_address = "192.168.1.100",
    user_agent = "Mozilla/5.0"
})

if user_login then
    published_events = published_events + 1
    print("   ✅ Published user.login event")
end

local user_action = Event.publish("user.action", {
    user_id = "user123",
    action_type = "view_page",
    page_url = "/dashboard",
    timestamp = os.time()
})

if user_action then
    published_events = published_events + 1
    print("   ✅ Published user.action event")
end

print("   📤 Publishing system events...")
local system_status = Event.publish("system.status", {
    service_name = "web_server",
    status = "healthy",
    cpu_usage = 45.2,
    memory_usage = 67.8,
    uptime = 86400 -- 1 day in seconds
})

if system_status then
    published_events = published_events + 1
    print("   ✅ Published system.status event")
end

local system_alert = Event.publish("system.alert", {
    alert_level = "warning",
    message = "High memory usage detected",
    threshold = 80.0,
    current_value = 82.5,
    timestamp = os.time()
})

if system_alert then
    published_events = published_events + 1
    print("   ✅ Published system.alert event")
end

print("   📤 Publishing test events...")
local basic_test = Event.publish("test.basic.example", {
    test_name = "basic_event_test",
    test_data = "Hello from basic event example",
    test_number = 42,
    test_boolean = true,
    test_timestamp = os.time()
})

if basic_test then
    published_events = published_events + 1
    print("   ✅ Published test.basic.example event")
end

print("   📊 Total events published:", published_events)

print()
print("3. Basic event receiving:")

-- Receive events from subscriptions
print("   📥 Receiving events from subscriptions...")

-- Helper function to display received event
local function display_received_event(event, subscription_name)
    if event then
        received_events = received_events + 1
        print(string.format("   📨 Received event via %s:", subscription_name))
        print("     • Event type:", event.event_type or "unknown")
        print("     • Data keys:", (function()
            if event.data then
                local keys = {}
                for key, _ in pairs(event.data) do
                    table.insert(keys, key)
                end
                return table.concat(keys, ", ")
            end
            return "none"
        end)())
        
        if event.metadata then
            print("     • Source language:", event.metadata.source_language or "unknown")
            print("     • Timestamp:", event.metadata.timestamp or "unknown")
        end
        
        return true
    else
        print(string.format("   ⏰ No events received via %s (timeout)", subscription_name))
        return false
    end
end

-- Try to receive events from each subscription
print("   🔍 Checking user events...")
local user_event = Event.receive(subscriptions.user_events, 1000) -- 1 second timeout
display_received_event(user_event, "user_events")

print("   🔍 Checking system events...")
local system_event = Event.receive(subscriptions.system_events, 1000)
display_received_event(system_event, "system_events")

print("   🔍 Checking test events...")
local test_event = Event.receive(subscriptions.test_events, 1000)
display_received_event(test_event, "test_events")

print("   📊 Total events received:", received_events)

print()
print("4. Event data structure exploration:")

-- Publish an event with rich data structure for exploration
print("   📤 Publishing rich data structure event...")
local rich_event = Event.publish("test.basic.rich_data", {
    -- Simple data types
    string_value = "Hello World",
    number_value = 123.45,
    boolean_value = true,
    
    -- Complex data structures
    array_data = {"item1", "item2", "item3"},
    nested_object = {
        level1 = {
            level2 = {
                deep_value = "nested content",
                deep_number = 999
            },
            level2_array = {1, 2, 3, 4, 5}
        },
        parallel_data = {
            name = "parallel structure",
            active = true
        }
    },
    
    -- Metadata
    event_metadata = {
        created_by = "lua_basic_example",
        version = "1.0",
        schema_version = "2023.1"
    }
})

if rich_event then
    print("   ✅ Published rich data structure event")
    
    -- Try to receive it
    print("   📥 Receiving rich data event...")
    local rich_received = Event.receive(subscriptions.test_events, 1000)
    
    if rich_received and rich_received.data then
        print("   🔍 Rich data event received successfully:")
        print("     • String value:", rich_received.data.string_value)
        print("     • Number value:", rich_received.data.number_value)
        print("     • Boolean value:", rich_received.data.boolean_value)
        print("     • Array length:", rich_received.data.array_data and #rich_received.data.array_data or 0)
        print("     • Has nested object:", rich_received.data.nested_object ~= nil)
        
        if rich_received.data.nested_object and rich_received.data.nested_object.level1 then
            print("     • Deep nested value:", rich_received.data.nested_object.level1.level2 and 
                  rich_received.data.nested_object.level1.level2.deep_value or "not found")
        end
    end
end

print()
print("5. Event pattern matching demonstration:")

-- Create specific pattern subscriptions
print("   📡 Creating pattern-specific subscriptions...")
local pattern_subs = {
    error_events = Event.subscribe("*.error"),
    success_events = Event.subscribe("*.success"), 
    user_login_events = Event.subscribe("user.login"),
    all_test_events = Event.subscribe("test.*")
}

print("   📤 Publishing events to test pattern matching...")

-- Publish events that should match different patterns
local pattern_events = {
    {event = "application.error", data = {error_code = 500, message = "Server error"}},
    {event = "database.error", data = {error_type = "connection_timeout"}},
    {event = "payment.success", data = {amount = 99.99, transaction_id = "tx_123"}},
    {event = "user.login", data = {user = "jane_doe", method = "oauth"}},
    {event = "test.pattern.example", data = {pattern_test = true}}
}

for _, event_info in ipairs(pattern_events) do
    local published = Event.publish(event_info.event, event_info.data)
    if published then
        print("   ✅ Published:", event_info.event)
    end
end

-- Try to receive pattern-matched events
print("   📥 Checking pattern-matched events...")
for pattern_name, sub_id in pairs(pattern_subs) do
    local received = Event.receive(sub_id, 500) -- 500ms timeout
    if received then
        print(string.format("   📨 %s matched: %s", pattern_name, received.event_type or "unknown"))
    else
        print(string.format("   ⏰ %s: no matches", pattern_name))
    end
end

print()
print("6. Event subscription management:")

-- Demonstrate subscription lifecycle
print("   🔄 Subscription lifecycle management:")

-- Show current subscriptions
local current_subs = Event.list_subscriptions()
print("   📋 Subscriptions before cleanup:", #current_subs)

-- Unsubscribe from pattern subscriptions
for pattern_name, sub_id in pairs(pattern_subs) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        print("   🗑️  Unsubscribed from", pattern_name)
    end
end

-- Check subscription count after cleanup
local after_cleanup_subs = Event.list_subscriptions()
print("   📋 Subscriptions after cleanup:", #after_cleanup_subs)

print()
print("7. Event system statistics:")

-- Get and display event system statistics
local stats = Event.get_stats()
if stats then
    print("   📊 Event System Statistics:")
    print("   • Event bus active:", stats.event_bus_stats ~= nil)
    print("   • Bridge active:", stats.bridge_stats ~= nil)
    
    if stats.event_bus_stats then
        print("   • Event bus status: operational")
    end
    
    if stats.bridge_stats then
        print("   • Cross-language bridge status: operational")
    end
else
    print("   ⚠️  Event system statistics not available")
end

print()
print("8. Basic event patterns summary:")

print("   📈 Session Summary:")
print("   • Events published:", published_events)
print("   • Events received:", received_events)
print("   • Subscriptions created:", #subscriptions + #pattern_subs)
print("   • Pattern matching demonstrated: *.error, *.success, user.login, test.*")
print("   • Data types tested: strings, numbers, booleans, arrays, nested objects")

print()
print("9. Event system best practices:")

print("   💡 Best Practices Demonstrated:")
print("   • Use descriptive event names with dot notation (user.login, system.alert)")
print("   • Include relevant metadata in event data")
print("   • Use pattern matching for flexible event filtering")
print("   • Set appropriate timeouts for event receiving")
print("   • Clean up subscriptions when no longer needed")
print("   • Structure event data consistently within your application")
print("   • Monitor event system statistics for health checking")

print()
print("10. Cleaning up basic event subscriptions:")

-- Clean up remaining subscriptions
for name, sub_id in pairs(subscriptions) do
    local unsubscribed = Event.unsubscribe(sub_id)
    if unsubscribed then
        print("   🧹 Unsubscribed from", name)
    end
end

-- Final subscription count
local final_subs = Event.list_subscriptions()
print("   ✅ Final subscription count:", #final_subs)

print()
print("✨ Basic event patterns example complete!")
print("   Key concepts demonstrated:")
print("   • Event publishing with Event.publish()")
print("   • Event subscription with Event.subscribe()")
print("   • Event receiving with Event.receive() and timeouts")
print("   • Event unsubscription with Event.unsubscribe()")
print("   • Pattern matching with wildcards (*, user.*, *.error)")
print("   • Rich data structures in events")
print("   • Subscription lifecycle management")