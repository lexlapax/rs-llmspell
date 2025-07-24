-- ABOUTME: Hook data modification patterns showing all result types
-- ABOUTME: Demonstrates continue, modified, cancel, redirect, replace, retry, skipped hook result types

print("=== Hook Data Modification Patterns Example ===")
print("Demonstrates: All hook result types and data flow control")
print()

local handles = {}
local modification_log = {}

-- Helper function to log modifications
local function log_modification(result_type, hook_point, details)
    local entry = {
        timestamp = os.time(),
        result_type = result_type,
        hook_point = hook_point,
        details = details or {}
    }
    table.insert(modification_log, entry)
    print(string.format("   📋 [%s] %s → %s", 
          os.date("%H:%M:%S", entry.timestamp), hook_point, result_type))
end

print("1. Continue result - Standard flow control:")

-- Continue: Allow execution to proceed normally
handles.continue = Hook.register("BeforeAgentExecution", function(context)
    log_modification("CONTINUE", "BeforeAgentExecution", {
        agent = context.component_id.name
    })
    
    print("   ➡️  Continuing normal execution flow")
    print("   📝 Agent context preserved unchanged")
    
    -- This is the default behavior - just let execution continue
    return "continue"
end, "normal")
print("   ✅ Registered continue result hook")

print()
print("2. Modified result - Data transformation:")

-- Modified: Transform data while continuing execution
handles.modified = Hook.register("BeforeAgentExecution", function(context)
    log_modification("MODIFIED", "BeforeAgentExecution", {
        original_data = context.data,
        modification_type = "data_enhancement"
    })
    
    print("   🔄 Modifying execution data")
    
    -- Extract and enhance the original data
    local original_prompt = context.data and context.data.prompt or "default prompt"
    local enhanced_data = {
        -- Keep original data
        original_prompt = original_prompt,
        
        -- Add enhancements
        enhanced_prompt = "Enhanced: " .. original_prompt,
        enhancement_timestamp = os.time(),
        enhancement_source = "lua_hook",
        
        -- Add context information
        execution_context = {
            agent_name = context.component_id.name,
            correlation_id = context.correlation_id,
            language = context.language
        },
        
        -- Add processing metadata
        processing_metadata = {
            enhanced = true,
            enhancement_level = "standard",
            processing_time = os.clock()
        }
    }
    
    print("   📝 Original prompt:", original_prompt)
    print("   ✨ Enhanced prompt:", enhanced_data.enhanced_prompt)
    
    return {
        type = "modified",
        data = enhanced_data
    }
end, "high")
print("   ✅ Registered modified result hook")

print()
print("3. Cancel result - Execution prevention:")

-- Cancel: Stop execution with a reason
handles.cancel = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Example: Cancel dangerous operations
    local dangerous_patterns = {"delete", "remove", "destroy", "dangerous"}
    local is_dangerous = false
    local danger_reason = ""
    
    for _, pattern in ipairs(dangerous_patterns) do
        if tool_name:lower():find(pattern) then
            is_dangerous = true
            danger_reason = "Tool contains dangerous pattern: " .. pattern
            break
        end
    end
    
    if is_dangerous then
        log_modification("CANCEL", "BeforeToolExecution", {
            tool = tool_name,
            reason = danger_reason
        })
        
        print("   🚫 CANCELLING execution of:", tool_name)
        print("   ⚠️  Reason:", danger_reason)
        
        return {
            type = "cancel",
            reason = danger_reason
        }
    end
    
    -- Otherwise continue normally
    return "continue"
end, "highest")
print("   ✅ Registered cancel result hook")

print()
print("4. Redirect result - Execution redirection:")

-- Redirect: Change execution target
handles.redirect = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Example: Redirect deprecated tools to new versions
    local redirections = {
        ["old_filesystem_tool"] = "filesystem-read",
        ["deprecated_web_tool"] = "web-fetch",
        ["legacy_json_tool"] = "json-parse"
    }
    
    local redirect_target = redirections[tool_name]
    if redirect_target then
        log_modification("REDIRECT", "BeforeToolExecution", {
            original_tool = tool_name,
            redirect_target = redirect_target
        })
        
        print("   🔄 REDIRECTING tool execution")
        print("   📍 From:", tool_name)
        print("   🎯 To:", redirect_target)
        
        return {
            type = "redirect",
            target = redirect_target
        }
    end
    
    return "continue"
end, "high")
print("   ✅ Registered redirect result hook")

print()
print("5. Replace result - Complete substitution:")

-- Replace: Completely replace execution result
handles.replace = Hook.register("AgentError", function(context)
    local error_message = context.data and context.data.error_message or "Unknown error"
    
    log_modification("REPLACE", "AgentError", {
        original_error = error_message
    })
    
    print("   🔄 REPLACING error with graceful fallback")
    print("   ❌ Original error:", error_message)
    
    -- Create a replacement result that provides a graceful fallback
    local replacement_data = {
        success = true,
        result_type = "fallback_response",
        fallback_message = "I encountered an issue, but I've handled it gracefully.",
        
        error_context = {
            original_error = error_message,
            handled_by = "lua_error_hook",
            timestamp = os.time(),
            recovery_strategy = "graceful_fallback"
        },
        
        suggested_actions = {
            "The request has been processed with a fallback method",
            "You may want to try rephrasing your request",
            "Contact support if this issue persists"
        }
    }
    
    print("   ✨ Replacement data created with graceful fallback")
    
    return {
        type = "replace",
        data = replacement_data
    }
end, "highest")
print("   ✅ Registered replace result hook")

print()
print("6. Retry result - Execution retry logic:")

-- Retry: Request retry with specific parameters
handles.retry = Hook.register("BeforeAgentExecution", function(context)
    -- Simulate a condition that requires retry (e.g., rate limiting)
    local should_retry = (math.random() < 0.3) -- 30% chance for demo
    
    if should_retry then
        log_modification("RETRY", "BeforeAgentExecution", {
            retry_reason = "simulated_rate_limit",
            delay_ms = 1000,
            max_attempts = 3
        })
        
        print("   🔄 REQUESTING retry due to rate limiting")
        print("   ⏱️  Retry delay: 1000ms")
        print("   🔢 Max attempts: 3")
        
        return {
            type = "retry",
            delay_ms = 1000,    -- Wait 1 second before retry
            max_attempts = 3    -- Maximum 3 retry attempts
        }
    end
    
    return "continue"
end, "normal")
print("   ✅ Registered retry result hook")

print()
print("7. Skipped result - Execution skipping:")

-- Skipped: Skip execution with reason
handles.skipped = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    
    -- Example: Skip tools during maintenance periods
    local maintenance_tools = {"maintenance_tool", "system_update", "backup_tool"}
    local is_maintenance = false
    local skip_reason = ""
    
    for _, pattern in ipairs(maintenance_tools) do
        if tool_name:find(pattern) then
            is_maintenance = true
            skip_reason = "Tool skipped during maintenance period: " .. pattern
            break
        end
    end
    
    if is_maintenance then
        log_modification("SKIPPED", "BeforeToolExecution", {
            tool = tool_name,
            reason = skip_reason
        })
        
        print("   ⏭️  SKIPPING tool execution:", tool_name)
        print("   📝 Reason:", skip_reason)
        
        return {
            type = "skipped",
            reason = skip_reason
        }
    end
    
    return "continue"
end, "normal")
print("   ✅ Registered skipped result hook")

print()
print("8. Result type demonstration summary:")

print("   📊 Hook result types registered:")
local result_types = {
    "continue", "modified", "cancel", "redirect", 
    "replace", "retry", "skipped"
}

for i, result_type in ipairs(result_types) do
    print(string.format("   %d. %s - %s", i, result_type:upper(), 
          result_type == "continue" and "Allow normal execution flow" or
          result_type == "modified" and "Transform data and continue" or
          result_type == "cancel" and "Stop execution with reason" or
          result_type == "redirect" and "Change execution target" or
          result_type == "replace" and "Substitute complete result" or
          result_type == "retry" and "Request retry with delay" or
          result_type == "skipped" and "Skip execution with reason" or
          "Unknown"))
end

print()
print("9. Data flow patterns:")

print("   🔄 Data Flow Patterns Demonstrated:")
print("   • Input Enhancement: Original data + additional context")
print("   • Security Filtering: Block dangerous operations")
print "   • Legacy Support: Redirect old tools to new versions")
print("   • Error Recovery: Replace errors with graceful fallbacks")
print("   • Rate Limiting: Retry with exponential backoff")
print("   • Maintenance Mode: Skip tools during maintenance")
print("   • Context Preservation: Maintain correlation IDs and metadata")

print()
print("10. Modification log analysis:")

print("   📋 Modification Log Summary:")
print("   • Total modifications logged:", #modification_log)

-- Group by result type
local result_counts = {}
for _, entry in ipairs(modification_log) do
    result_counts[entry.result_type] = (result_counts[entry.result_type] or 0) + 1
end

for result_type, count in pairs(result_counts) do
    print(string.format("   • %s: %d occurrences", result_type, count))
end

if #modification_log > 0 then
    print("   📅 Recent modifications:")
    for i = math.max(1, #modification_log - 3), #modification_log do
        local entry = modification_log[i]
        print(string.format("     %d. %s at %s", 
              i, entry.result_type, entry.hook_point))
    end
end

print()
print("11. Cleaning up modification hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "result hook")
end

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Hook data modification patterns example complete!")
print("   Key concepts demonstrated:")
print("   • All 7 hook result types with practical examples")
print("   • Data transformation and enhancement patterns")
print("   • Security and validation through hook results")
print("   • Error recovery and graceful fallback mechanisms")
print("   • Execution flow control (cancel, redirect, skip)")
print("   • Retry logic with configurable delays and limits")
print("   • Context preservation across modifications")