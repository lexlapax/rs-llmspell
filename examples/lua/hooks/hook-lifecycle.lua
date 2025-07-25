-- ABOUTME: Agent lifecycle hooks demonstration covering initialization to shutdown
-- ABOUTME: Shows all agent lifecycle phases with BeforeAgentInit, AfterAgentInit, BeforeAgentExecution, AfterAgentExecution, BeforeAgentShutdown, AfterAgentShutdown

print("=== Agent Lifecycle Hooks Example ===")
print("Demonstrates: Complete agent lifecycle hook coverage")
print()

local handles = {}
local execution_log = {}

-- Helper function to log execution
local function log_execution(phase, context)
    local entry = {
        timestamp = os.time(),
        phase = phase,
        agent_name = context and context.component_id.name or "unknown",
        correlation_id = context and context.correlation_id or "none"
    }
    table.insert(execution_log, entry)
    print(string.format("   📋 [%s] %s - Agent: %s", 
          os.date("%H:%M:%S", entry.timestamp), phase, entry.agent_name))
end

print("1. Registering agent lifecycle hooks:")

-- Before Agent Init - Setup and preparation
handles.before_init = Hook.register("BeforeAgentInit", function(context)
    log_execution("BEFORE_INIT", context)
    print("   🚀 Preparing agent initialization...")
    
    -- Could perform setup tasks like:
    -- - Validate configuration
    -- - Initialize resources
    -- - Set up monitoring
    
    return "continue"
end, "high")
print("   ✅ Registered BeforeAgentInit hook")

-- After Agent Init - Post-initialization tasks
handles.after_init = Hook.register("AfterAgentInit", function(context)
    log_execution("AFTER_INIT", context)
    print("   ✨ Agent initialization completed!")
    
    -- Could perform post-init tasks like:
    -- - Register with service discovery
    -- - Start health checks
    -- - Load additional resources
    
    return "continue"
end, "normal")
print("   ✅ Registered AfterAgentInit hook")

-- Before Agent Execution - Pre-execution preparation
handles.before_execution = Hook.register("BeforeAgentExecution", function(context)
    log_execution("BEFORE_EXECUTION", context)
    print("   ⚡ Preparing for agent execution...")
    
    -- Could perform pre-execution tasks like:
    -- - Validate input parameters
    -- - Check rate limits
    -- - Initialize execution context
    -- - Log request details
    
    return "continue"
end, "normal")
print("   ✅ Registered BeforeAgentExecution hook")

-- After Agent Execution - Post-execution cleanup
handles.after_execution = Hook.register("AfterAgentExecution", function(context)
    log_execution("AFTER_EXECUTION", context)
    print("   🎯 Agent execution completed!")
    
    -- Could perform post-execution tasks like:
    -- - Log execution results
    -- - Update metrics
    -- - Clean up temporary resources
    -- - Send notifications
    
    return "continue"
end, "normal")
print("   ✅ Registered AfterAgentExecution hook")

-- Before Agent Shutdown - Pre-shutdown preparation
handles.before_shutdown = Hook.register("BeforeAgentShutdown", function(context)
    log_execution("BEFORE_SHUTDOWN", context)
    print("   ⚠️  Preparing for agent shutdown...")
    
    -- Could perform pre-shutdown tasks like:
    -- - Save state to persistent storage
    -- - Cancel ongoing operations
    -- - Notify dependent services
    -- - Create shutdown checkpoints
    
    return "continue"
end, "high")
print("   ✅ Registered BeforeAgentShutdown hook")

-- After Agent Shutdown - Post-shutdown cleanup
handles.after_shutdown = Hook.register("AfterAgentShutdown", function(context)
    log_execution("AFTER_SHUTDOWN", context)
    print("   💤 Agent shutdown completed!")
    
    -- Could perform post-shutdown tasks like:
    -- - Release system resources
    -- - Update service registry
    -- - Send shutdown notifications
    -- - Archive logs and metrics
    
    return "continue"
end, "low")
print("   ✅ Registered AfterAgentShutdown hook")

print()

-- Show the complete lifecycle hook chain
print("2. Complete agent lifecycle hook chain:")
local agent_hooks = Hook.list()
local lifecycle_hooks = {}

-- Filter for agent lifecycle hooks
for _, hook in ipairs(agent_hooks) do
    if hook.name:find("Agent") then
        table.insert(lifecycle_hooks, hook)
    end
end

print("   🔗 Agent lifecycle hooks registered:", #lifecycle_hooks)
for i, hook in ipairs(lifecycle_hooks) do
    print(string.format("   %d. %s (%s priority)", i, hook.name, hook.priority))
end
print()

-- Demonstrate lifecycle with error handling
print("3. Adding error handling to lifecycle:")

handles.agent_error = Hook.register("AgentError", function(context)
    log_execution("ERROR", context)
    print("   ❌ Agent error occurred!")
    
    -- Error recovery logic
    local error_message = context.data and context.data.error_message or "Unknown error"
    print("   🔧 Error details:", error_message)
    
    -- Could perform error handling like:
    -- - Log detailed error information
    -- - Attempt automatic recovery
    -- - Send error notifications
    -- - Update error metrics
    
    return {
        type = "modified",
        data = {
            error_handled = true,
            recovery_attempted = true,
            original_error = error_message,
            handled_at = os.time()
        }
    }
end, "highest")
print("   ✅ Registered AgentError hook with recovery logic")
print()

-- Show execution log so far
print("4. Execution log from lifecycle hooks:")
if #execution_log > 0 then
    print("   📊 Hook execution timeline:")
    for i, entry in ipairs(execution_log) do
        print(string.format("   %d. %s: %s (%s)", 
              i, entry.phase, entry.agent_name, entry.correlation_id))
    end
else
    print("   ℹ️  No lifecycle events triggered yet (hooks are waiting for actual agent operations)")
end
print()

-- Demonstrate lifecycle monitoring
print("5. Lifecycle monitoring and metrics:")

local lifecycle_stats = {
    hooks_registered = 0,
    phases_covered = {},
    error_handlers = 0
}

-- Count lifecycle hooks
for name, handle in pairs(handles) do
    lifecycle_stats.hooks_registered = lifecycle_stats.hooks_registered + 1
    
    local hook_point = handle:hook_point()
    if hook_point:find("Agent") then
        local phase = hook_point:gsub("Agent", ""):lower()
        lifecycle_stats.phases_covered[phase] = true
    end
    
    if hook_point:find("Error") then
        lifecycle_stats.error_handlers = lifecycle_stats.error_handlers + 1
    end
end

print("   📈 Lifecycle monitoring stats:")
print("   • Total hooks registered:", lifecycle_stats.hooks_registered)
print("   • Lifecycle phases covered:", table.concat(
    (function()
        local phases = {}
        for phase, _ in pairs(lifecycle_stats.phases_covered) do
            table.insert(phases, phase)
        end
        return phases
    end)(), ", "))
print("   • Error handlers:", lifecycle_stats.error_handlers)
print()

-- Best practices for lifecycle hooks
print("6. Lifecycle hook best practices:")
print("   💡 Best practices demonstrated:")
print("   • Use HIGH priority for critical init/shutdown operations")
print("   • Use NORMAL priority for standard processing")
print("   • Use LOW priority for cleanup and logging")
print("   • Always include error handling hooks")
print("   • Log execution for debugging and monitoring")
print("   • Keep hook logic lightweight and fast")
print("   • Return appropriate result types for flow control")
print()

-- Cleanup
print("7. Cleaning up lifecycle hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "hook")
end

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Agent lifecycle hooks example complete!")
print("   Key concepts demonstrated:")
print("   • Complete agent lifecycle coverage (6 phases)")
print("   • Priority-based execution ordering for lifecycle events")
print("   • Error handling integration with lifecycle hooks")
print("   • Execution logging and monitoring patterns")
print("   • Best practices for lifecycle hook implementation")
print("   • Resource management across agent lifecycle")