-- ABOUTME: Hook listing and filtering capabilities with advanced query patterns
-- ABOUTME: Demonstrates Hook.list() with various filtering options and hook management

print("=== Hook Filtering and Listing Example ===")
print("Demonstrates: Advanced hook querying, filtering, and management")
print()

local handles = {}
local hook_registry = {
    total_created = 0,
    by_priority = {},
    by_hook_point = {},
    by_language = {},
    registration_history = {}
}

-- Helper function to register hook with tracking
local function register_tracked_hook(hook_point, callback, priority, metadata)
    local handle = Hook.register(hook_point, callback, priority)
    
    -- Track registration
    hook_registry.total_created = hook_registry.total_created + 1
    hook_registry.by_priority[priority or "normal"] = (hook_registry.by_priority[priority or "normal"] or 0) + 1
    hook_registry.by_hook_point[hook_point] = (hook_registry.by_hook_point[hook_point] or 0) + 1
    hook_registry.by_language["lua"] = (hook_registry.by_language["lua"] or 0) + 1
    
    table.insert(hook_registry.registration_history, {
        timestamp = os.time(),
        hook_point = hook_point,
        priority = priority or "normal",
        metadata = metadata or {}
    })
    
    return handle
end

print("1. Creating diverse hooks for filtering demonstration:")

-- Create hooks with different priorities
handles.auth_highest = register_tracked_hook("BeforeAgentInit", function(ctx)
    print("   🔐 Authentication check (HIGHEST priority)")
    return "continue"
end, "highest", {category = "security", subsystem = "auth"})

handles.auth_high = register_tracked_hook("BeforeAgentInit", function(ctx)
    print("   🛡️  Security validation (HIGH priority)")
    return "continue"
end, "high", {category = "security", subsystem = "validation"})

handles.logging_normal = register_tracked_hook("BeforeAgentInit", function(ctx)
    print("   📝 Standard logging (NORMAL priority)")
    return "continue" 
end, "normal", {category = "monitoring", subsystem = "logging"})

handles.metrics_low = register_tracked_hook("BeforeAgentInit", function(ctx)
    print("   📊 Metrics collection (LOW priority)")
    return "continue"
end, "low", {category = "monitoring", subsystem = "metrics"})

handles.debug_lowest = register_tracked_hook("BeforeAgentInit", function(ctx)
    print("   🐛 Debug information (LOWEST priority)")
    return "continue"
end, "lowest", {category = "development", subsystem = "debug"})

print("   ✅ Created 5 BeforeAgentInit hooks with different priorities")

-- Create hooks for different hook points
handles.agent_exec = register_tracked_hook("BeforeAgentExecution", function(ctx)
    print("   ⚡ Agent execution hook")
    return "continue"
end, "normal", {category = "execution", subsystem = "agent"})

handles.tool_exec = register_tracked_hook("BeforeToolExecution", function(ctx)
    print("   🛠️  Tool execution hook")
    return "continue"
end, "high", {category = "execution", subsystem = "tool"})

handles.workflow_start = register_tracked_hook("BeforeWorkflowStart", function(ctx)
    print("   🔄 Workflow start hook")
    return "continue"
end, "normal", {category = "workflow", subsystem = "orchestration"})

handles.agent_error = register_tracked_hook("AgentError", function(ctx)
    print("   ❌ Agent error handler")
    return "continue"
end, "highest", {category = "error", subsystem = "recovery"})

handles.tool_error = register_tracked_hook("ToolError", function(ctx)
    print("   🔧 Tool error handler")
    return "continue"
end, "high", {category = "error", subsystem = "recovery"})

print("   ✅ Created 5 hooks across different hook points")

print()
print("2. Basic hook listing:")

-- List all hooks
local all_hooks = Hook.list()
print("   📋 Total hooks registered:", #all_hooks)
print("   🔍 All hooks:")
for i, hook in ipairs(all_hooks) do
    print(string.format("     %d. %s (%s priority, %s language)", 
          i, hook.name, hook.priority, hook.language))
end

print()
print("3. Filtering hooks by hook point:")

-- Filter by specific hook points
local hook_points = {"BeforeAgentInit", "BeforeAgentExecution", "BeforeToolExecution", "AgentError", "ToolError"}

for _, hook_point in ipairs(hook_points) do
    local filtered_hooks = Hook.list(hook_point)
    print(string.format("   🎯 %s hooks: %d", hook_point, #filtered_hooks))
    
    for i, hook in ipairs(filtered_hooks) do
        print(string.format("     %d. Priority: %s, Name: %s", 
              i, hook.priority, hook.name))
    end
end

print()
print("4. Filtering hooks by priority levels:")

-- Filter by each priority level
local priorities = {"highest", "high", "normal", "low", "lowest"}

for _, priority in ipairs(priorities) do
    local priority_hooks = Hook.list({priority = priority})
    print(string.format("   ⚡ %s priority hooks: %d", priority:upper(), #priority_hooks))
    
    for i, hook in ipairs(priority_hooks) do
        print(string.format("     %d. %s (%s)", i, hook.name, hook.language))
    end
end

print()
print("5. Advanced filtering with multiple criteria:")

-- Complex filtering examples
print("   🔍 Advanced Filtering Examples:")

-- Filter Lua hooks with high priority
local lua_high_hooks = Hook.list({language = "lua", priority = "high"})
print(string.format("   • Lua + HIGH priority: %d hooks", #lua_high_hooks))
for i, hook in ipairs(lua_high_hooks) do
    print(string.format("     %d. %s", i, hook.name))
end

-- Filter hooks by multiple criteria (simulate with manual filtering)
print("   • Security-related hooks (highest/high priority):")
local security_hooks = {}
for _, hook in ipairs(all_hooks) do
    if hook.priority == "Priority(-2147483648)" or hook.priority == "Priority(-100)" then
        table.insert(security_hooks, hook)
    end
end
print(string.format("     Found %d security-priority hooks", #security_hooks))

-- Filter error handling hooks
print("   • Error handling hooks:")
local error_hooks = {}
for _, hook in ipairs(all_hooks) do
    if hook.name:find("Error") then
        table.insert(error_hooks, hook)
    end
end
print(string.format("     Found %d error handling hooks", #error_hooks))

print()
print("6. Hook statistics and analysis:")

print("   📊 Hook Registry Statistics:")
print("   • Total hooks created:", hook_registry.total_created)
print("   • Currently active:", #all_hooks)

print()
print("   📈 Distribution by Priority:")
for priority, count in pairs(hook_registry.by_priority) do
    print(string.format("   • %s: %d hooks", priority:upper(), count))
end

print()
print("   📍 Distribution by Hook Point:")
for hook_point, count in pairs(hook_registry.by_hook_point) do
    print(string.format("   • %s: %d hooks", hook_point, count))
end

print()
print("   🌍 Distribution by Language:")
for language, count in pairs(hook_registry.by_language) do
    print(string.format("   • %s: %d hooks", language:upper(), count))
end

print()
print("7. Hook metadata analysis:")

-- Analyze hook patterns
print("   🔍 Hook Pattern Analysis:")

-- Priority distribution analysis
local priority_distribution = {}
for _, hook in ipairs(all_hooks) do
    local priority_key = hook.priority
    priority_distribution[priority_key] = (priority_distribution[priority_key] or 0) + 1
end

print("   📊 Active Priority Distribution:")
for priority, count in pairs(priority_distribution) do
    local percentage = (count / #all_hooks) * 100
    print(string.format("   • %s: %d hooks (%.1f%%)", priority, count, percentage))
end

-- Hook point coverage analysis
local unique_hook_points = {}
for _, hook in ipairs(all_hooks) do
    if hook.name:find("Before") or hook.name:find("After") or hook.name:find("Error") then
        local base_name = hook.name:gsub("Priority%(%-?%d+%)", ""):gsub("%s+", "")
        unique_hook_points[base_name] = true
    end
end

local hook_point_count = 0
for _ in pairs(unique_hook_points) do
    hook_point_count = hook_point_count + 1
end

print("   📍 Hook Point Coverage:", hook_point_count, "unique hook points")

print()
print("8. Dynamic hook management:")

print("   🔄 Dynamic Hook Management:")

-- Show registration timeline
print("   📅 Registration Timeline (last 5 registrations):")
local history_start = math.max(1, #hook_registry.registration_history - 4)
for i = history_start, #hook_registry.registration_history do
    local entry = hook_registry.registration_history[i]
    print(string.format("   %d. %s - %s (%s priority) at %s", 
          i, entry.hook_point, entry.metadata.category or "unknown", 
          entry.priority, os.date("%H:%M:%S", entry.timestamp)))
end

-- Demonstrate conditional hook management
print("   🎛️  Conditional Hook Management:")
print("   • High-priority hooks for security:", #lua_high_hooks)
print("   • Error recovery hooks:", #error_hooks)
print("   • Monitoring hooks: 2 (logging + metrics)")

-- Hook health check
local healthy_hooks = 0
for _, handle in pairs(handles) do
    if handle and handle:id() then
        healthy_hooks = healthy_hooks + 1
    end
end

print("   ✅ Hook Health Check:", healthy_hooks, "healthy hooks out of", hook_registry.total_created)

print()
print("9. Search and query patterns:")

print("   🔎 Search and Query Patterns:")

-- Search by name patterns
local search_patterns = {
    ["Agent"] = "Agent-related hooks",
    ["Tool"] = "Tool-related hooks", 
    ["Error"] = "Error handling hooks",
    ["Before"] = "Pre-execution hooks",
    ["After"] = "Post-execution hooks"
}

for pattern, description in pairs(search_patterns) do
    local matching_hooks = {}
    for _, hook in ipairs(all_hooks) do
        if hook.name:find(pattern) then
            table.insert(matching_hooks, hook)
        end
    end
    print(string.format("   • %s: %d hooks found", description, #matching_hooks))
end

-- Performance analysis
print("   ⚡ Query Performance:")
local query_start = os.clock()
for i = 1, 10 do
    Hook.list() -- Test list all performance
end
local query_time = (os.clock() - query_start) * 1000

print(string.format("   • Hook.list() average time: %.2fms (10 queries)", query_time / 10))

print()
print("10. Best practices for hook management:")

print("   💡 Hook Management Best Practices:")
print("   • Use descriptive names for hook identification")
print("   • Organize hooks by priority based on criticality")
print("   • Group related hooks using consistent naming patterns")
print("   • Monitor hook registration and cleanup")
print("   • Use filtering to manage large numbers of hooks")
print("   • Implement hook health checks for reliability")
print("   • Track hook performance impact")
print("   • Document hook purposes and dependencies")

print()
print("11. Cleaning up all demonstration hooks:")

local cleanup_count = 0
for name, handle in pairs(handles) do
    if handle and handle:id() then
        Hook.unregister(handle)
        cleanup_count = cleanup_count + 1
        print("   🧹 Unregistered", name, "hook")
    end
end

local final_count = #Hook.list()
print("   ✅ Cleaned up", cleanup_count, "hooks")
print("   ✅ Final hook count:", final_count)

print()
print("✨ Hook filtering and listing example complete!")
print("   Key concepts demonstrated:")
print("   • Comprehensive hook listing with Hook.list()")
print("   • Filtering by hook point, priority, and language")
print("   • Advanced multi-criteria filtering techniques")
print("   • Hook statistics and distribution analysis")
print("   • Dynamic hook management and health monitoring")
print("   • Search patterns and query optimization")
print("   • Hook registry tracking and lifecycle management")
print("   • Best practices for large-scale hook management")