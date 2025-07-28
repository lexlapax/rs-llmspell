--! ABOUTME: Comprehensive examples of using the Lua Hook API
--! ABOUTME: Demonstrates registration, unregistration, filtering, and cross-language events

-- Example 1: Basic Hook Registration
print("=== Example 1: Basic Hook Registration ===")

-- Register a simple hook with normal priority
local handle1 = Hook.register("BeforeAgentInit", function(context)
    print("Agent initializing:", context.component_id.name)
    return "continue"
end, "normal")

print("Registered hook with ID:", handle1:id())

-- Example 2: Hook with High Priority and Data Modification
print("\n=== Example 2: High Priority Hook with Data Modification ===")

local handle2 = Hook.register("BeforeAgentExecution", function(context)
    print("Intercepting agent execution for:", context.component_id.name)
    
    -- Modify the execution data
    local modified_data = {
        original_prompt = context.data.prompt or "default",
        enhanced_prompt = "Enhanced: " .. (context.data.prompt or "default"),
        timestamp = os.time(),
        source = "lua_hook"
    }
    
    return {
        type = "modified",
        data = modified_data
    }
end, "high")

-- Example 3: Error Handling Hook
print("\n=== Example 3: Error Handling Hook ===")

local handle3 = Hook.register("AgentError", function(context)
    print("Agent error occurred:", context.data.error_message or "unknown error")
    
    -- Log the error with additional context
    print("Error details:", context.data)
    
    -- Try to provide a fallback response
    return {
        type = "replace",
        data = {
            response = "An error occurred, but I'm handling it gracefully.",
            error_handled = true,
            original_error = context.data.error_message
        }
    }
end, "highest")

-- Example 4: Conditional Hook with Cancellation
print("\n=== Example 4: Conditional Hook with Cancellation ===")

local handle4 = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Cancel execution for specific tools
    if tool_name == "dangerous_tool" then
        return {
            type = "cancel",
            reason = "Tool execution cancelled for safety reasons"
        }
    end
    
    -- Skip processing for certain tools
    if tool_name == "skip_me" then
        return {
            type = "skipped",
            reason = "Tool processing skipped by configuration"
        }
    end
    
    -- Redirect to alternative tool
    if tool_name == "redirect_me" then
        return {
            type = "redirect",
            target = "alternative_tool"
        }
    end
    
    return "continue"
end)

-- Example 5: Hook Listing and Filtering
print("\n=== Example 5: Hook Listing and Filtering ===")

-- List all hooks
print("All registered hooks:")
local all_hooks = Hook.list()
for i, hook in ipairs(all_hooks) do
    print(string.format("  %d. %s (%s, %s)", i, hook.name, hook.language, hook.priority))
end

-- List hooks for specific hook point
print("\nHooks for BeforeAgentInit:")
local agent_init_hooks = Hook.list("BeforeAgentInit")
for i, hook in ipairs(agent_init_hooks) do
    print(string.format("  %d. %s", i, hook.name))
end

-- List hooks with table filter
print("\nHigh priority Lua hooks:")
local high_lua_hooks = Hook.list({
    language = "lua",
    priority = "high"
})
for i, hook in ipairs(high_lua_hooks) do
    print(string.format("  %d. %s (%s)", i, hook.name, hook.description or "no description"))
end

-- Example 6: Event Publishing and Subscription
print("\n=== Example 6: Event Publishing and Subscription ===")

-- Subscribe to agent events
local sub_id = Event.subscribe("agent.*")
print("Subscribed to agent events with ID:", sub_id)

-- Publish a custom event
local published = Event.publish("agent.custom_event", {
    message = "This is a custom event from Lua",
    timestamp = os.time(),
    data = {
        user_id = "user123",
        action = "custom_action"
    }
}, {
    language = "lua",
    ttl_seconds = 300  -- 5 minutes
})

if published then
    print("Event published successfully")
end

-- Try to receive the event (with timeout)
print("Waiting for events...")
local received = Event.receive(sub_id, 2000)  -- 2 second timeout

if received then
    print("Received event:")
    print("  Type:", received.event_type)
    print("  Data:", received.data.message)
    print("  Source Language:", received.metadata.source_language)
else
    print("No events received (timeout)")
end

-- Example 7: Cross-Language Event Communication
print("\n=== Example 7: Cross-Language Event Communication ===")

-- Subscribe to events from all languages
local cross_lang_sub = Event.subscribe("cross_lang.*")

-- Publish events that other languages can receive
Event.publish("cross_lang.lua_to_rust", {
    message = "Hello from Lua!",
    sender = "lua_script",
    recipient = "rust_component"
})

Event.publish("cross_lang.lua_to_js", {
    message = "Lua says hello to JavaScript",
    data_type = "greeting",
    payload = {hello = "world", from = "lua"}
})

-- Example 8: Event Statistics and Monitoring
print("\n=== Example 8: Event Statistics and Monitoring ===")

local stats = Event.get_stats()
if stats then
    print("Event system statistics:")
    print("  Event bus stats:", stats.event_bus_stats ~= nil)
    print("  Bridge stats:", stats.bridge_stats ~= nil)
end

-- List all subscriptions
local subscriptions = Event.list_subscriptions()
print(string.format("Active subscriptions: %d", #subscriptions))
for i, sub in ipairs(subscriptions) do
    print(string.format("  %d. %s -> %s (%s)", i, sub.id, sub.pattern, sub.language))
end

-- Example 9: Hook Unregistration
print("\n=== Example 9: Hook Unregistration ===")

-- Unregister using handle method
local unregistered1 = handle1:unregister()
print("Handle1 unregistered (method):", unregistered1)

-- Unregister using standalone function
local unregistered2 = Hook.unregister(handle2)
print("Handle2 unregistered (function):", unregistered2)

-- Verify hooks are removed
print("Remaining hooks after unregistration:")
local remaining_hooks = Hook.list()
for i, hook in ipairs(remaining_hooks) do
    print(string.format("  %d. %s", i, hook.name))
end

-- Example 10: Retry Logic Hook
print("\n=== Example 10: Retry Logic Hook ===")

local retry_count = 0
local handle_retry = Hook.register("BeforeAgentExecution", function(context)
    retry_count = retry_count + 1
    print("Attempt #" .. retry_count .. " for agent:", context.component_id.name)
    
    -- Simulate failure on first two attempts
    if retry_count <= 2 then
        return {
            type = "retry",
            delay_ms = 1000,  -- 1 second delay
            max_attempts = 3
        }
    end
    
    print("Success on attempt #" .. retry_count)
    return "continue"
end, "low")

-- Cleanup
print("\n=== Cleanup ===")

-- Unregister remaining hooks
Hook.unregister(handle3)
Hook.unregister(handle4) 
Hook.unregister(handle_retry)

-- Unsubscribe from events
Event.unsubscribe(sub_id)
Event.unsubscribe(cross_lang_sub)

print("All hooks unregistered and events unsubscribed")
print("Example complete!")