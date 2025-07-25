-- ABOUTME: Tool execution hooks with validation, security, and monitoring
-- ABOUTME: Demonstrates BeforeToolDiscovery, AfterToolDiscovery, BeforeToolExecution, AfterToolExecution, ToolValidation, ToolError hooks

print("=== Tool Integration Hooks Example ===")
print("Demonstrates: Complete tool lifecycle with security and validation")
print()

local handles = {}
local tool_metrics = {
    discoveries = 0,
    executions = 0,
    validations = 0,
    errors = 0,
    blocked_tools = {},
    execution_times = {}
}

-- Security configuration
local security_config = {
    allowed_tools = {
        "filesystem-read", "filesystem-write", "web-fetch", 
        "json-parse", "string-process", "math-calculate"
    },
    blocked_tools = {
        "system-execute", "dangerous-tool"
    },
    require_validation = {
        "filesystem-write", "web-fetch"
    }
}

print("1. Registering tool discovery hooks:")

-- Before Tool Discovery - Control what tools are available
handles.before_discovery = Hook.register("BeforeToolDiscovery", function(context)
    print("   🔍 Starting tool discovery...")
    tool_metrics.discoveries = tool_metrics.discoveries + 1
    
    -- Could perform discovery preparation like:
    -- - Check tool permissions
    -- - Load tool configurations
    -- - Initialize tool registry
    -- - Set discovery filters
    
    print("   📋 Discovery #" .. tool_metrics.discoveries .. " initiated")
    return "continue"
end, "normal")
print("   ✅ Registered BeforeToolDiscovery hook")

-- After Tool Discovery - Post-process discovered tools
handles.after_discovery = Hook.register("AfterToolDiscovery", function(context)
    print("   ✨ Tool discovery completed!")
    
    -- Could perform post-discovery tasks like:
    -- - Filter tools by security policy
    -- - Cache tool metadata
    -- - Initialize tool connections
    -- - Log available tools
    
    local tool_count = context.data and context.data.discovered_count or "unknown"
    print("   📊 Tools discovered:", tool_count)
    
    return "continue"
end, "normal")
print("   ✅ Registered AfterToolDiscovery hook")

print()
print("2. Registering tool execution hooks with security:")

-- Before Tool Execution - Security validation and preprocessing
handles.before_execution = Hook.register("BeforeToolExecution", function(context)
    local tool_name = context.component_id.name
    print("   🛠️  About to execute tool:", tool_name)
    
    tool_metrics.executions = tool_metrics.executions + 1
    
    -- Security check: Block dangerous tools
    for _, blocked_tool in ipairs(security_config.blocked_tools) do
        if tool_name:find(blocked_tool) then
            print("   🚫 BLOCKED: Tool '" .. tool_name .. "' is not allowed")
            table.insert(tool_metrics.blocked_tools, {
                tool = tool_name,
                timestamp = os.time(),
                reason = "Security policy violation"
            })
            
            return {
                type = "cancel",
                reason = "Tool blocked by security policy: " .. blocked_tool
            }
        end
    end
    
    -- Validation check: Some tools need extra validation
    for _, validation_tool in ipairs(security_config.require_validation) do
        if tool_name:find(validation_tool) then
            print("   ⚠️  Tool requires validation:", tool_name)
            
            -- Could perform validation like:
            -- - Check file permissions for filesystem tools
            -- - Validate URLs for web tools
            -- - Check data formats for processing tools
            
            local validation_passed = true -- Simulate validation
            if not validation_passed then
                return {
                    type = "cancel",
                    reason = "Tool validation failed"
                }
            end
            print("   ✅ Validation passed for:", tool_name)
        end
    end
    
    -- Log execution attempt
    print("   ▶️  Executing tool:", tool_name)
    tool_metrics.execution_times[tool_name] = os.clock()
    
    return "continue"
end, "high")
print("   ✅ Registered BeforeToolExecution hook with security checks")

-- After Tool Execution - Post-processing and monitoring
handles.after_execution = Hook.register("AfterToolExecution", function(context)
    local tool_name = context.component_id.name
    print("   ✅ Tool execution completed:", tool_name)
    
    -- Calculate execution time
    local start_time = tool_metrics.execution_times[tool_name]
    if start_time then
        local execution_time = (os.clock() - start_time) * 1000 -- Convert to milliseconds
        print(string.format("   ⏱️  Execution time: %.2fms", execution_time))
        tool_metrics.execution_times[tool_name] = nil -- Clean up
    end
    
    -- Could perform post-execution tasks like:
    -- - Log execution results
    -- - Update tool performance metrics
    -- - Cache tool outputs
    -- - Send completion notifications
    
    return "continue"
end, "normal")
print("   ✅ Registered AfterToolExecution hook with timing")

print()
print("3. Registering tool validation and error hooks:")

-- Tool Validation - Validate tool inputs and configuration
handles.validation = Hook.register("ToolValidation", function(context)
    local tool_name = context.component_id.name
    print("   🔍 Validating tool:", tool_name)
    
    tool_metrics.validations = tool_metrics.validations + 1
    
    -- Perform input validation based on tool type
    local validation_rules = {
        ["filesystem"] = function(data)
            -- Validate file paths
            if data.path and data.path:find("%.%.") then
                return false, "Path traversal detected"
            end
            return true, "File path valid"
        end,
        ["web"] = function(data)
            -- Validate URLs
            if data.url and not data.url:match("^https?://") then
                return false, "Invalid URL protocol"
            end
            return true, "URL valid"
        end,
        ["string"] = function(data)
            -- Validate string inputs
            if data.input and #data.input > 10000 then
                return false, "Input string too long"
            end
            return true, "String input valid"
        end
    }
    
    -- Find applicable validation rule
    local validation_result = true
    local validation_message = "No specific validation required"
    
    for tool_type, validator in pairs(validation_rules) do
        if tool_name:find(tool_type) then
            validation_result, validation_message = validator(context.data or {})
            break
        end
    end
    
    print("   📋 Validation result:", validation_message)
    
    if not validation_result then
        return {
            type = "cancel",
            reason = "Tool validation failed: " .. validation_message
        }
    end
    
    return "continue"
end, "high")
print("   ✅ Registered ToolValidation hook with input validation")

-- Tool Error - Handle tool execution errors
handles.error = Hook.register("ToolError", function(context)
    local tool_name = context.component_id.name
    local error_message = context.data and context.data.error_message or "Unknown error"
    
    print("   ❌ Tool error in:", tool_name)
    print("   🔧 Error:", error_message)
    
    tool_metrics.errors = tool_metrics.errors + 1
    
    -- Error recovery strategies based on error type
    local recovery_strategies = {
        ["timeout"] = "retry with increased timeout",
        ["network"] = "retry with exponential backoff",
        ["permission"] = "escalate to administrator",
        ["validation"] = "sanitize input and retry",
        ["resource"] = "wait and retry later"
    }
    
    -- Determine recovery strategy
    local recovery_strategy = "log error and continue"
    for error_type, strategy in pairs(recovery_strategies) do
        if error_message:lower():find(error_type) then
            recovery_strategy = strategy
            break
        end
    end
    
    print("   🔧 Recovery strategy:", recovery_strategy)
    
    -- Implement graceful fallback
    return {
        type = "replace",
        data = {
            error_handled = true,
            original_error = error_message,
            recovery_strategy = recovery_strategy,
            fallback_result = "Tool error handled gracefully - " .. tool_name,
            timestamp = os.time()
        }
    }
end, "highest")
print("   ✅ Registered ToolError hook with recovery strategies")

print()
print("4. Tool security and monitoring dashboard:")

-- Display current tool metrics
print("   📊 Tool Integration Metrics:")
print("   • Tool discoveries:", tool_metrics.discoveries)
print("   • Tool executions:", tool_metrics.executions)
print("   • Validations performed:", tool_metrics.validations)
print("   • Errors handled:", tool_metrics.errors)
print("   • Tools currently blocked:", #tool_metrics.blocked_tools)

if #tool_metrics.blocked_tools > 0 then
    print("   🚫 Blocked tools:")
    for i, blocked in ipairs(tool_metrics.blocked_tools) do
        print(string.format("     %d. %s - %s", 
              i, blocked.tool, blocked.reason))
    end
end

print()
print("5. Security configuration:")
print("   🔒 Security Policy:")
print("   • Allowed tools:", table.concat(security_config.allowed_tools, ", "))
print("   • Blocked tools:", table.concat(security_config.blocked_tools, ", "))
print("   • Validation required:", table.concat(security_config.require_validation, ", "))

print()
print("6. Tool hook chain summary:")
local tool_hooks = Hook.list()
local tool_hook_count = 0

for _, hook in ipairs(tool_hooks) do
    if hook.name:find("Tool") then
        tool_hook_count = tool_hook_count + 1
        print(string.format("   🔗 %s (%s priority)", hook.name, hook.priority))
    end
end

print("   📋 Total tool hooks active:", tool_hook_count)

print()
print("7. Cleaning up tool hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "hook")
end

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Tool integration hooks example complete!")
print("   Key concepts demonstrated:")
print("   • Complete tool lifecycle coverage (6 hook points)")
print("   • Security validation and tool blocking")
print("   • Input validation with type-specific rules")
print("   • Error handling with recovery strategies")
print("   • Performance monitoring and metrics collection")
print("   • Configurable security policies for tool access")
print("   • Graceful error recovery and fallback mechanisms")