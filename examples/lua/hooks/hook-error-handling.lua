-- ABOUTME: Comprehensive error handling with hooks and graceful fallback mechanisms
-- ABOUTME: Demonstrates AgentError, ToolError, WorkflowError hooks with recovery strategies

print("=== Hook Error Handling Example ===")
print("Demonstrates: Comprehensive error handling with recovery strategies")
print()

local handles = {}
local error_statistics = {
    agent_errors = 0,
    tool_errors = 0,
    workflow_errors = 0,
    recovered_errors = 0,
    error_types = {},
    recovery_strategies = {}
}

-- Helper function to classify error types
local function classify_error(error_message)
    local classifications = {
        ["timeout"] = "Network/Timeout Error",
        ["permission"] = "Permission/Security Error", 
        ["validation"] = "Input Validation Error",
        ["resource"] = "Resource Exhaustion Error",
        ["network"] = "Network Connectivity Error",
        ["api"] = "API/Service Error",
        ["memory"] = "Memory/Performance Error",
        ["config"] = "Configuration Error"
    }
    
    local error_lower = error_message:lower()
    for pattern, classification in pairs(classifications) do
        if error_lower:find(pattern) then
            return classification
        end
    end
    
    return "Unknown Error Type"
end

-- Helper function to log error handling
local function log_error_handling(error_type, component, strategy, success)
    error_statistics.error_types[error_type] = (error_statistics.error_types[error_type] or 0) + 1
    error_statistics.recovery_strategies[strategy] = (error_statistics.recovery_strategies[strategy] or 0) + 1
    
    if success then
        error_statistics.recovered_errors = error_statistics.recovered_errors + 1
    end
    
    print(string.format("   📊 Error handled: %s via %s (%s)", 
          error_type, strategy, success and "SUCCESS" or "FAILED"))
end

print("1. Agent error handling with recovery strategies:")

-- Agent Error Handler - Comprehensive agent error recovery
handles.agent_error = Hook.register("AgentError", function(context)
    local agent_name = context.component_id.name
    local error_message = context.data and context.data.error_message or "Unknown agent error"
    local error_type = classify_error(error_message)
    
    error_statistics.agent_errors = error_statistics.agent_errors + 1
    
    print("   ❌ Agent Error Detected:")
    print("   🤖 Agent:", agent_name)
    print("   💥 Error:", error_message)
    print("   🏷️  Type:", error_type)
    
    -- Recovery strategies based on error type
    local recovery_strategies = {
        ["Network/Timeout Error"] = function()
            return {
                type = "retry",
                delay_ms = 2000,
                max_attempts = 3,
                recovery_message = "Retrying due to network timeout",
                fallback_data = {
                    response = "Request timed out, retrying with increased timeout",
                    retry_strategy = "exponential_backoff"
                }
            }
        end,
        
        ["Permission/Security Error"] = function()
            return {
                type = "replace",
                data = {
                    error_handled = true,
                    response = "Access denied. Please check your permissions and try again.",
                    suggested_actions = {
                        "Verify your authentication credentials",
                        "Contact administrator for access rights",
                        "Try with reduced scope request"
                    },
                    security_context = {
                        blocked_operation = error_message,
                        timestamp = os.time()
                    }
                }
            }
        end,
        
        ["Input Validation Error"] = function()
            return {
                type = "modified",
                data = {
                    error_handled = true,
                    validation_failed = true,
                    response = "Input validation failed. Please provide valid input.",
                    validation_hints = {
                        "Check input format and requirements",
                        "Ensure all required fields are provided",
                        "Verify data types match expectations"
                    },
                    original_error = error_message
                }
            }
        end,
        
        ["Resource Exhaustion Error"] = function()
            return {
                type = "retry",
                delay_ms = 5000,
                max_attempts = 2,
                recovery_message = "Waiting for resources to become available",
                fallback_data = {
                    response = "System resources temporarily unavailable. Retrying...",
                    resource_info = {
                        error_type = "resource_exhaustion",
                        retry_delay = "5 seconds"
                    }
                }
            }
        end
    }
    
    -- Apply recovery strategy
    local recovery_fn = recovery_strategies[error_type]
    local recovery_result
    local strategy_name = "graceful_fallback"
    
    if recovery_fn then
        recovery_result = recovery_fn()
        strategy_name = recovery_result.type .. "_recovery"
        print("   🔧 Applying recovery strategy:", strategy_name)
        log_error_handling(error_type, "agent", strategy_name, true)
        return recovery_result
    else
        -- Default fallback for unknown error types
        strategy_name = "default_fallback"
        print("   🔧 Applying default fallback strategy")
        log_error_handling(error_type, "agent", strategy_name, true)
        
        return {
            type = "replace",
            data = {
                error_handled = true,
                response = "An unexpected error occurred, but it has been handled gracefully.",
                error_context = {
                    original_error = error_message,
                    error_type = error_type,
                    agent = agent_name,
                    timestamp = os.time(),
                    correlation_id = context.correlation_id
                },
                support_info = {
                    message = "If this error persists, please contact support",
                    error_id = context.correlation_id
                }
            }
        }
    end
end, "highest")
print("   ✅ Registered comprehensive AgentError handler")

print()
print("2. Tool error handling with tool-specific recovery:")

-- Tool Error Handler - Tool-specific error recovery
handles.tool_error = Hook.register("ToolError", function(context)
    local tool_name = context.component_id.name
    local error_message = context.data and context.data.error_message or "Unknown tool error"
    local error_type = classify_error(error_message)
    
    error_statistics.tool_errors = error_statistics.tool_errors + 1
    
    print("   ❌ Tool Error Detected:")
    print("   🛠️  Tool:", tool_name)
    print("   💥 Error:", error_message)
    print("   🏷️  Type:", error_type)
    
    -- Tool-specific recovery strategies
    local tool_recovery_strategies = {
        ["filesystem"] = function(error_msg)
            if error_msg:find("permission") then
                return {
                    type = "replace",
                    data = {
                        error = "File access denied",
                        alternative = "Try accessing files in user directory",
                        suggested_path = "~/documents/"
                    }
                }
            elseif error_msg:find("not found") then
                return {
                    type = "replace",
                    data = {
                        error = "File not found",
                        suggestion = "Please verify the file path and try again"
                    }
                }
            end
        end,
        
        ["web"] = function(error_msg)
            if error_msg:find("timeout") then
                return {
                    type = "retry",
                    delay_ms = 3000,
                    max_attempts = 2
                }
            elseif error_msg:find("404") then
                return {
                    type = "replace",
                    data = {
                        error = "Web resource not found",
                        suggestion = "Please check the URL and try again"
                    }
                }
            end
        end,
        
        ["json"] = function(error_msg)
            if error_msg:find("parse") then
                return {
                    type = "modified",
                    data = {
                        error = "JSON parsing failed",
                        cleaned_data = "{}",
                        suggestion = "Returning empty JSON object as fallback"
                    }
                }
            end
        end
    }
    
    -- Find applicable recovery strategy
    local recovery_result = nil
    local strategy_name = "default_tool_fallback"
    
    for tool_type, recovery_fn in pairs(tool_recovery_strategies) do
        if tool_name:lower():find(tool_type) then
            recovery_result = recovery_fn(error_message)
            if recovery_result then
                strategy_name = tool_type .. "_specific_recovery"
                break
            end
        end
    end
    
    if recovery_result then
        print("   🔧 Applying tool-specific recovery:", strategy_name)
        log_error_handling(error_type, "tool", strategy_name, true)
        return recovery_result
    else
        -- Default tool error fallback
        print("   🔧 Applying default tool fallback")
        log_error_handling(error_type, "tool", strategy_name, true)
        
        return {
            type = "replace",
            data = {
                tool_error_handled = true,
                tool_name = tool_name,
                error_message = error_message,
                fallback_result = "Tool operation failed but was handled gracefully",
                alternatives = {
                    "Try using a different tool for this operation",
                    "Check tool configuration and retry",
                    "Contact support if this tool is critical"
                },
                timestamp = os.time()
            }
        }
    end
end, "highest")
print("   ✅ Registered tool-specific ToolError handler")

print()
print("3. Workflow error handling with checkpoint recovery:")

-- Workflow Error Handler - Workflow-specific error recovery
handles.workflow_error = Hook.register("WorkflowError", function(context)
    local workflow_name = context.component_id.name
    local error_message = context.data and context.data.error_message or "Unknown workflow error"
    local error_stage = context.data and context.data.error_stage or "unknown_stage"
    local error_type = classify_error(error_message)
    
    error_statistics.workflow_errors = error_statistics.workflow_errors + 1
    
    print("   ❌ Workflow Error Detected:")
    print("   🔄 Workflow:", workflow_name)
    print("   📍 Stage:", error_stage)
    print("   💥 Error:", error_message)
    print("   🏷️  Type:", error_type)
    
    -- Workflow recovery strategies based on error severity
    local workflow_recovery_strategies = {
        ["stage_failure"] = function()
            return {
                type = "modified",
                data = {
                    recovery_action = "skip_failed_stage",
                    skipped_stage = error_stage,
                    continue_workflow = true,
                    stage_error = error_message,
                    recovery_note = "Stage failed but workflow continues"
                }
            }
        end,
        
        ["critical_failure"] = function()
            return {
                type = "cancel",
                reason = "Critical workflow failure - cannot continue safely: " .. error_message
            }
        end,
        
        ["recoverable_failure"] = function()
            return {
                type = "modified",
                data = {
                    recovery_action = "rollback_to_checkpoint",
                    failed_stage = error_stage,
                    rollback_point = "last_checkpoint",
                    retry_possible = true,
                    error_details = {
                        message = error_message,
                        timestamp = os.time(),
                        recovery_strategy = "checkpoint_rollback"
                    }
                }
            }
        end
    }
    
    -- Determine recovery strategy based on error characteristics
    local strategy_name = "recoverable_failure"
    if error_message:find("critical") or error_message:find("fatal") then
        strategy_name = "critical_failure"
    elseif error_message:find("validation") or error_message:find("timeout") then
        strategy_name = "stage_failure"
    end
    
    local recovery_fn = workflow_recovery_strategies[strategy_name]
    if recovery_fn then
        print("   🔧 Applying workflow recovery:", strategy_name)
        log_error_handling(error_type, "workflow", strategy_name, true)
        return recovery_fn()
    else
        -- Default workflow error handling
        print("   🔧 Applying default workflow error handling")
        log_error_handling(error_type, "workflow", "default_workflow_fallback", true)
        
        return {
            type = "replace",
            data = {
                workflow_error_handled = true,
                workflow_name = workflow_name,
                failed_stage = error_stage,
                error_summary = error_message,
                recovery_options = {
                    "Review workflow configuration",
                    "Check stage dependencies",
                    "Monitor resource availability",
                    "Consider manual intervention"
                },
                workflow_state = "error_handled",
                timestamp = os.time()
            }
        }
    end
end, "highest")
print("   ✅ Registered workflow-specific WorkflowError handler")

print()
print("4. Error handling dashboard:")

print("   📊 Error Handling Statistics:")
print("   • Agent errors handled:", error_statistics.agent_errors)
print("   • Tool errors handled:", error_statistics.tool_errors)
print("   • Workflow errors handled:", error_statistics.workflow_errors)
print("   • Total errors recovered:", error_statistics.recovered_errors)

print()
print("   🏷️  Error Types Encountered:")
for error_type, count in pairs(error_statistics.error_types) do
    print(string.format("   • %s: %d occurrences", error_type, count))
end

print()
print("   🔧 Recovery Strategies Used:")
for strategy, count in pairs(error_statistics.recovery_strategies) do
    print(string.format("   • %s: %d times", strategy, count))
end

print()
print("5. Error handling best practices demonstrated:")

print("   💡 Best Practices Implemented:")
print("   • Error Classification: Categorize errors by type and severity")
print("   • Context Preservation: Maintain correlation IDs and error context")
print("   • Graceful Degradation: Provide meaningful fallbacks")
print("   • Recovery Strategies: Multiple recovery approaches per error type")
print("   • User-Friendly Messages: Clear, actionable error messages")
print("   • Retry Logic: Intelligent retry with backoff strategies")
print("   • Resource Management: Handle resource exhaustion gracefully")
print("   • Security Handling: Safe error messages without information disclosure")

print()
print("6. Error monitoring and alerting:")

print("   🔔 Error Monitoring Features:")
print("   • Real-time error statistics collection")
print("   • Error type classification and trending")
print("   • Recovery success rate tracking")
print("   • Context preservation for debugging")
print("   • Correlation ID tracking across components")

local total_errors = error_statistics.agent_errors + error_statistics.tool_errors + error_statistics.workflow_errors
if total_errors > 0 then
    local recovery_rate = (error_statistics.recovered_errors / total_errors) * 100
    print(string.format("   📈 Recovery Success Rate: %.1f%%", recovery_rate))
end

print()
print("7. Cleaning up error handling hooks:")
for name, handle in pairs(handles) do
    Hook.unregister(handle)
    print("   🧹 Unregistered", name, "error handler")
end

local final_count = #Hook.list()
print("   ✅ Final hook count:", final_count)

print()
print("✨ Hook error handling example complete!")
print("   Key concepts demonstrated:")
print("   • Comprehensive error classification and handling")
print("   • Component-specific recovery strategies (agent, tool, workflow)")
print("   • Graceful fallback mechanisms with user-friendly messages")
print("   • Retry logic with configurable delays and limits")
print("   • Error context preservation and correlation tracking")
print("   • Recovery success monitoring and statistics")
print("   • Security-conscious error handling practices")