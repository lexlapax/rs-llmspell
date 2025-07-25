-- ABOUTME: Hook priority levels demonstration with execution ordering
-- ABOUTME: Shows all 5 priority levels (highest, high, normal, low, lowest) and their execution order

print("=== Hook Priority Levels Example ===")
print("Demonstrates: Priority execution order (highest → lowest)")
print()

-- Register hooks with different priorities
print("1. Registering hooks with different priorities:")

local handles = {}

-- Highest priority hook
handles.highest = Hook.register("BeforeAgentExecution", function(context)
    print("   🔥 HIGHEST priority hook executing first")
    return "continue"
end, "highest")
print("   ✅ Registered HIGHEST priority hook")

-- High priority hook
handles.high = Hook.register("BeforeAgentExecution", function(context)
    print("   ⚡ HIGH priority hook executing second")
    return "continue"
end, "high")
print("   ✅ Registered HIGH priority hook")

-- Normal priority hook (default)
handles.normal = Hook.register("BeforeAgentExecution", function(context)
    print("   ➡️  NORMAL priority hook executing third")
    return "continue"
end, "normal")
print("   ✅ Registered NORMAL priority hook")

-- Low priority hook
handles.low = Hook.register("BeforeAgentExecution", function(context)
    print("   ⬇️  LOW priority hook executing fourth")
    return "continue"
end, "low")
print("   ✅ Registered LOW priority hook")

-- Lowest priority hook
handles.lowest = Hook.register("BeforeAgentExecution", function(context)
    print("   🐌 LOWEST priority hook executing last")
    return "continue"
end, "lowest")
print("   ✅ Registered LOWEST priority hook")

print()

-- List hooks to show priority ordering
print("2. Hook listing shows priority ordering:")
local agent_hooks = Hook.list("BeforeAgentExecution")
print("   📋 BeforeAgentExecution hooks (should be ordered by priority):")
for i, hook in ipairs(agent_hooks) do
    print(string.format("   %d. Priority: %s, Language: %s", 
          i, hook.priority, hook.language))
end
print()

-- Demonstrate priority filtering
print("3. Filtering hooks by priority:")

-- Filter high priority hooks
local high_priority_hooks = Hook.list({priority = "high"})
print("   🔍 HIGH priority hooks:", #high_priority_hooks)
for i, hook in ipairs(high_priority_hooks) do
    print(string.format("   • %s (priority: %s)", hook.name, hook.priority))
end

-- Filter normal priority hooks
local normal_priority_hooks = Hook.list({priority = "normal"})
print("   🔍 NORMAL priority hooks:", #normal_priority_hooks)
for i, hook in ipairs(normal_priority_hooks) do
    print(string.format("   • %s (priority: %s)", hook.name, hook.priority))
end

-- Filter lowest priority hooks
local lowest_priority_hooks = Hook.list({priority = "lowest"})
print("   🔍 LOWEST priority hooks:", #lowest_priority_hooks)
for i, hook in ipairs(lowest_priority_hooks) do
    print(string.format("   • %s (priority: %s)", hook.name, hook.priority))
end

print()

-- Demonstrate priority use cases
print("4. Priority use cases:")
print("   🔥 HIGHEST: Critical system hooks, security validation")
print("   ⚡ HIGH: Important preprocessing, authentication")
print("   ➡️  NORMAL: Standard business logic, default processing")
print("   ⬇️  LOW: Logging, metrics collection, cleanup")
print("   🐌 LOWEST: Debug information, optional processing")
print()

-- Advanced: Multiple hooks on same point with different priorities
print("5. Registering multiple hooks on different points:")

-- Add some hooks to different points to show mixed priority handling
local tool_handle = Hook.register("BeforeToolExecution", function(context)
    print("   🛠️  HIGH priority tool hook")
    return "continue"
end, "high")

local error_handle = Hook.register("AgentError", function(context)
    print("   ❌ HIGHEST priority error handler")
    return "continue"
end, "highest")

print("   ✅ Added hooks to BeforeToolExecution and AgentError")
print()

-- Show all hooks with their priorities
print("6. Complete hook inventory by priority:")
local all_hooks = Hook.list()

-- Group hooks by priority
local priority_groups = {
    ["Priority(2147483647)"] = "LOWEST",
    ["Priority(100)"] = "LOW", 
    ["Priority(0)"] = "NORMAL",
    ["Priority(-100)"] = "HIGH",
    ["Priority(-2147483648)"] = "HIGHEST"
}

for i, hook in ipairs(all_hooks) do
    local priority_label = priority_groups[hook.priority] or hook.priority
    print(string.format("   %d. %s - %s (%s)", 
          i, priority_label, hook.name, hook.language))
end
print()

-- Cleanup
print("7. Cleaning up all hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "priority hook")
end

Hook.unregister(tool_handle)
Hook.unregister(error_handle)
print("   🧹 Unregistered tool and error hooks")

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Priority levels example complete!")
print("   Key concepts demonstrated:")
print("   • All five priority levels: highest, high, normal, low, lowest")
print("   • Priority-based execution ordering")
print("   • Priority filtering with Hook.list({priority = 'level'})")
print("   • Real-world priority use cases")
print("   • Mixed priority hooks across different hook points")