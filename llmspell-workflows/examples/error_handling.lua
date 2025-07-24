-- ABOUTME: Workflow error handling and debugging patterns
-- ABOUTME: Demonstrates retry strategies, error recovery, debugging, and monitoring

-- Error Handling Strategies Example

-- 1. Fail-Fast Strategy
local fail_fast_workflow = Workflow.sequential({
    name = "fail_fast_example",
    description = "Stops immediately on first error",
    
    error_strategy = "fail_fast",
    
    steps = {
        {
            name = "step1_success",
            type = "tool",
            tool = "calculator",
            input = { input = "10 + 20" }
        },
        {
            name = "step2_error",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/nonexistent/file.txt"  -- This will fail
            }
        },
        {
            name = "step3_skipped",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        }
    },
    
    -- Global error handler
    on_error = function(error, step_name)
        print("ERROR in " .. step_name .. ": " .. tostring(error))
        State.set("last_error", {
            step = step_name,
            message = tostring(error),
            timestamp = os.time()
        })
    end
})

print("Testing fail-fast strategy...")
local fail_fast_result = fail_fast_workflow:execute()
print("Result: " .. (fail_fast_result.success and "Success" or "Failed"))
print("Steps completed: " .. fail_fast_result.metadata.steps_executed)

-- 2. Continue Strategy with Error Collection
local continue_workflow = Workflow.sequential({
    name = "continue_on_error",
    description = "Continues execution despite errors",
    
    error_strategy = "continue",
    
    steps = {
        {
            name = "step1_success",
            type = "tool",
            tool = "calculator",
            input = { input = "5 * 5" }
        },
        {
            name = "step2_error",
            type = "custom",
            execute = function()
                error("Simulated error for testing")
            end,
            -- Step-level error handler
            on_error = function(error)
                print("Step 2 error handled: " .. tostring(error))
                -- Return fallback value
                return {
                    success = false,
                    output = "fallback_value",
                    error = tostring(error)
                }
            end
        },
        {
            name = "step3_recovery",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Recovered from error. Previous output: {{prev}}",
                variables = {
                    prev = "{{step:step2_error:output}}"
                }
            }
        },
        {
            name = "step4_success",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        }
    },
    
    -- Collect all errors
    on_complete = function(success)
        local errors = State.get("workflow_errors") or {}
        print("\nWorkflow completed with " .. #errors .. " errors")
        for i, err in ipairs(errors) do
            print("  Error " .. i .. ": " .. err.step .. " - " .. err.message)
        end
    end
})

print("\n\nTesting continue strategy...")
local continue_result = continue_workflow:execute()
print("Result: " .. (continue_result.success and "Success" or "Completed with errors"))

-- 3. Retry Strategy with Exponential Backoff
local retry_workflow = Workflow.sequential({
    name = "retry_with_backoff",
    description = "Retries failed steps with exponential backoff",
    
    error_strategy = {
        type = "retry",
        max_attempts = 3,
        backoff_ms = 1000,
        backoff_multiplier = 2  -- 1s, 2s, 4s
    },
    
    steps = {
        {
            name = "flaky_operation",
            type = "custom",
            execute = function()
                -- Simulate flaky operation that fails 2/3 times
                local attempt = (State.get("retry_attempt") or 0) + 1
                State.set("retry_attempt", attempt)
                
                if attempt < 3 then
                    error("Temporary failure (attempt " .. attempt .. ")")
                end
                
                return {
                    success = true,
                    output = "Success on attempt " .. attempt
                }
            end,
            on_retry = function(attempt, error)
                print("Retrying after error: " .. tostring(error))
                print("Attempt " .. attempt .. " starting...")
            end
        },
        {
            name = "process_result",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:flaky_operation:output}}",
                operation = "uppercase"
            }
        }
    }
})

State.set("retry_attempt", 0)
print("\n\nTesting retry strategy...")
local retry_result = retry_workflow:execute()
print("Result: " .. (retry_result.success and "Success after retries" or "Failed"))

-- 4. Circuit Breaker Pattern
local circuit_breaker = {
    failure_count = 0,
    failure_threshold = 3,
    reset_timeout = 5000,  -- 5 seconds
    state = "closed",  -- closed, open, half-open
    last_failure_time = 0
}

local circuit_breaker_workflow = Workflow.loop({
    name = "circuit_breaker_example",
    description = "Implements circuit breaker pattern for external service calls",
    
    iterator = {
        range = { start = 1, ["end"] = 10, step = 1 }
    },
    
    body = {
        {
            name = "check_circuit",
            type = "custom",
            execute = function()
                local now = os.time() * 1000  -- milliseconds
                
                -- Check if circuit should be reset
                if circuit_breaker.state == "open" and 
                   (now - circuit_breaker.last_failure_time) > circuit_breaker.reset_timeout then
                    circuit_breaker.state = "half-open"
                    print("Circuit breaker: half-open (testing)")
                end
                
                if circuit_breaker.state == "open" then
                    return {
                        success = false,
                        output = "Circuit breaker OPEN - skipping call"
                    }
                end
                
                return { success = true, output = "Circuit breaker: " .. circuit_breaker.state }
            end
        },
        {
            name = "external_call",
            type = "custom",
            execute = function(context)
                if circuit_breaker.state == "open" then
                    return { success = false, output = "Skipped due to open circuit" }
                end
                
                -- Simulate external service call (fails 40% of time)
                local success = math.random() > 0.4
                
                if success then
                    -- Reset on success
                    if circuit_breaker.state == "half-open" then
                        circuit_breaker.state = "closed"
                        circuit_breaker.failure_count = 0
                        print("Circuit breaker: closed (recovered)")
                    end
                    return { success = true, output = "External call succeeded" }
                else
                    -- Track failures
                    circuit_breaker.failure_count = circuit_breaker.failure_count + 1
                    circuit_breaker.last_failure_time = os.time() * 1000
                    
                    if circuit_breaker.failure_count >= circuit_breaker.failure_threshold then
                        circuit_breaker.state = "open"
                        print("Circuit breaker: OPEN (too many failures)")
                    end
                    
                    error("External service unavailable")
                end
            end,
            on_error = function(error)
                return {
                    success = false,
                    output = "Call failed: " .. tostring(error),
                    error = error
                }
            end
        }
    },
    
    error_strategy = "continue"
})

print("\n\nTesting circuit breaker pattern...")
local circuit_result = circuit_breaker_workflow:execute()
print("Completed " .. circuit_result.data.completed_iterations .. " iterations")

-- 5. Debugging Workflow with Detailed Logging
local debug_workflow = Workflow.sequential({
    name = "debug_example",
    description = "Workflow with comprehensive debugging features",
    
    -- Enable debug mode
    debug = true,
    log_level = "debug",
    
    steps = {
        {
            name = "init_debug",
            type = "custom",
            execute = function()
                -- Set up debug context
                State.set("debug_context", {
                    workflow_id = "debug_" .. os.time(),
                    start_time = os.time(),
                    breakpoints = {},
                    watch_variables = {"user_data", "process_status"}
                })
                
                return { success = true, output = "Debug mode initialized" }
            end
        },
        {
            name = "process_with_logging",
            type = "tool",
            tool = "calculator",
            input = { input = "42 * 10" },
            
            -- Pre-execution hook for debugging
            before_execute = function(context)
                print("[DEBUG] Before step: process_with_logging")
                print("[DEBUG] Input: " .. tostring(context.input))
                print("[DEBUG] State keys: " .. table.concat(State.keys(), ", "))
                
                -- Breakpoint simulation
                if State.get("debug_break") then
                    print("[BREAKPOINT] Paused at step: process_with_logging")
                    print("Press Enter to continue...")
                    -- io.read()  -- Uncomment for actual breakpoint
                end
            end,
            
            -- Post-execution hook
            after_execute = function(result)
                print("[DEBUG] After step: process_with_logging")
                print("[DEBUG] Success: " .. tostring(result.success))
                print("[DEBUG] Output: " .. tostring(result.output))
                
                -- Watch variable changes
                local watches = State.get("debug_context").watch_variables
                for _, var in ipairs(watches) do
                    local value = State.get(var)
                    if value then
                        print("[WATCH] " .. var .. " = " .. tostring(value))
                    end
                end
            end
        },
        {
            name = "validate_with_trace",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:process_with_logging:output}}",
                schema = { type = "number", minimum = 0 }
            },
            
            -- Trace execution time
            trace = function(duration_ms)
                print("[TRACE] Step 'validate_with_trace' took " .. duration_ms .. "ms")
                
                -- Performance warning
                if duration_ms > 100 then
                    print("[PERF WARNING] Step took longer than 100ms")
                end
            end
        }
    },
    
    -- Global debugging handlers
    on_step_start = function(step_name)
        print("\n[WORKFLOW] Starting step: " .. step_name)
        State.set("current_step", step_name)
    end,
    
    on_step_complete = function(step_name, result)
        print("[WORKFLOW] Completed step: " .. step_name .. " (success: " .. tostring(result.success) .. ")")
        
        -- Log to debug history
        local history = State.get("debug_history") or {}
        table.insert(history, {
            step = step_name,
            success = result.success,
            timestamp = os.time(),
            output_preview = string.sub(tostring(result.output), 1, 50)
        })
        State.set("debug_history", history)
    end,
    
    on_error = function(error, step_name)
        print("\n[ERROR] Step '" .. step_name .. "' failed:")
        print("[ERROR] Type: " .. type(error))
        print("[ERROR] Message: " .. tostring(error))
        
        -- Stack trace
        if debug and debug.traceback then
            print("[ERROR] Stack trace:")
            print(debug.traceback())
        end
        
        -- Dump state on error
        print("\n[STATE DUMP]")
        for key, value in pairs(State.all()) do
            print("  " .. key .. " = " .. tostring(value))
        end
    end
})

print("\n\nTesting debug workflow...")
local debug_result = debug_workflow:execute()

-- 6. Validation and Safety Checks
local safe_workflow = Workflow.sequential({
    name = "safe_workflow",
    description = "Workflow with input validation and safety checks",
    
    steps = {
        -- Validate inputs before processing
        {
            name = "validate_inputs",
            type = "custom",
            execute = function()
                local required_keys = {"user_id", "operation", "data"}
                local inputs = State.get("workflow_inputs") or {}
                local missing = {}
                
                for _, key in ipairs(required_keys) do
                    if not inputs[key] then
                        table.insert(missing, key)
                    end
                end
                
                if #missing > 0 then
                    error("Missing required inputs: " .. table.concat(missing, ", "))
                end
                
                return { success = true, output = "Inputs validated" }
            end
        },
        
        -- Sanitize user input
        {
            name = "sanitize_data",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = State.get("workflow_inputs").data or "",
                operation = "sanitize",
                options = {
                    remove_html = true,
                    trim_whitespace = true,
                    max_length = 1000
                }
            }
        },
        
        -- Resource limits check
        {
            name = "check_resources",
            type = "custom",
            execute = function()
                local limits = {
                    max_memory_mb = 100,
                    max_cpu_percent = 80,
                    max_execution_time_ms = 5000
                }
                
                -- Check current resource usage
                local current_memory = 50  -- Simulated
                local current_cpu = 30     -- Simulated
                
                if current_memory > limits.max_memory_mb then
                    error("Memory limit exceeded: " .. current_memory .. "MB")
                end
                
                if current_cpu > limits.max_cpu_percent then
                    error("CPU limit exceeded: " .. current_cpu .. "%")
                end
                
                return {
                    success = true,
                    output = "Resources within limits"
                }
            end
        }
    },
    
    -- Timeout for entire workflow
    timeout_ms = 10000,
    
    on_timeout = function()
        print("[TIMEOUT] Workflow exceeded time limit")
        State.set("workflow_status", "timeout")
    end
})

-- Test with sample inputs
State.set("workflow_inputs", {
    user_id = "user123",
    operation = "process",
    data = "Sample data to process"
})

print("\n\nTesting safe workflow with validation...")
local safe_result = safe_workflow:execute()
print("Safe workflow result: " .. (safe_result.success and "Success" or "Failed"))