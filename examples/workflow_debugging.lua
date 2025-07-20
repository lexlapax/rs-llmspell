-- ABOUTME: Workflow debugging and error handling example
-- ABOUTME: Demonstrates validation, debugging, metrics, and error handling features

print("=== Workflow Debugging and Error Handling ===\n")

-- Enable debug mode globally
Workflow.enableDebug(true)
print("Debug mode enabled: " .. tostring(Workflow.isDebugEnabled()))

-- Set a default error handler for all workflows
Workflow.setDefaultErrorHandler(function(error, context)
    print("\n!!! DEFAULT ERROR HANDLER !!!")
    print("Error: " .. tostring(error))
    print("Workflow: " .. (context.workflow_id or "unknown"))
    print("Phase: " .. (context.phase or "unknown"))
    print("Step: " .. (context.step or "unknown"))
    print("!!! END ERROR HANDLER !!!\n")
end)

-- Create a workflow with potential issues for debugging
local debug_workflow = Workflow.sequential({
    name = "debug_example_workflow",
    description = "Workflow designed to demonstrate debugging features",
    error_strategy = "continue", -- Continue on errors to see multiple issues
    timeout_ms = 5000,
    steps = {
        {
            name = "validate_input",
            type = "tool",
            tool = "input_validator",
            input = { 
                schema = "user_schema",
                strict = true
            }
        },
        {
            name = "risky_operation",
            type = "tool",
            tool = "network_caller",
            input = {
                url = "https://unreliable-api.example.com/data",
                timeout = 1000,
                retry_count = 2
            }
        },
        {
            name = "process_data",
            type = "tool",
            tool = "data_processor",
            input = {
                algorithm = "complex_transform",
                validate_output = true
            }
        }
    }
})

-- Validate the workflow configuration
print("\n=== Workflow Validation ===")
local validation = debug_workflow:validate()
print("Workflow valid: " .. tostring(validation.valid))
if validation.errors and #validation.errors > 0 then
    print("Errors:")
    for i, err in ipairs(validation.errors) do
        print("  " .. i .. ". " .. err)
    end
end
if validation.warnings and #validation.warnings > 0 then
    print("Warnings:")
    for i, warn in ipairs(validation.warnings) do
        print("  " .. i .. ". " .. warn)
    end
end

-- Get debug information
print("\n=== Debug Information ===")
local debug_info = debug_workflow:debug()
print("Workflow ID: " .. debug_info.workflow_id)
print("Name: " .. debug_info.name)
print("Type: " .. debug_info.type)
print("Runtime info:")
print("  Created at: " .. (debug_info.runtime.created_at or "N/A"))

-- Set up comprehensive error handling
debug_workflow:onError(function(error)
    print("\n=== Workflow Error Handler ===")
    print("Error occurred: " .. tostring(error))
    
    -- Store error information for analysis
    debug_workflow:setState("last_error", tostring(error))
    debug_workflow:setState("error_count", 
        (debug_workflow:getState("error_count") or 0) + 1)
    
    -- Emit error event for monitoring
    debug_workflow:emit("workflow_error", {
        error = tostring(error),
        timestamp = os.time(),
        severity = "high"
    })
    
    -- Get current metrics when error occurs
    local metrics = debug_workflow:getMetrics()
    print("Failed executions so far: " .. metrics.execution.failed_executions)
end)

-- Create a conditional workflow with complex error scenarios
local error_handling_workflow = Workflow.conditional({
    name = "error_scenarios",
    description = "Demonstrates various error handling patterns",
    error_strategy = "retry",
    branches = {
        {
            name = "success_path",
            condition = {
                type = "shared_data_equals",
                key = "force_success",
                expected = true
            },
            steps = {{
                name = "successful_operation",
                type = "tool",
                tool = "simple_calculator",
                input = { operation = "add", a = 1, b = 2 }
            }}
        },
        {
            name = "failure_path",
            condition = {
                type = "shared_data_equals",
                key = "force_failure",
                expected = true
            },
            steps = {{
                name = "failing_operation",
                type = "tool",
                tool = "error_simulator",
                input = { 
                    error_type = "timeout",
                    message = "Simulated timeout error"
                }
            }}
        }
    },
    default_branch = {
        name = "validation_path",
        steps = {{
            name = "validate_conditions",
            type = "tool",
            tool = "condition_validator",
            input = { strict_mode = true }
        }}
    }
})

-- Monitor workflow execution with detailed logging
error_handling_workflow:onBeforeExecute(function(context)
    if Workflow.isDebugEnabled() then
        print("\n[DEBUG] Starting workflow: " .. context.workflow_id)
        print("[DEBUG] Initial state: " .. tostring(context.state))
    end
end)

error_handling_workflow:onAfterExecute(function(context)
    if Workflow.isDebugEnabled() then
        print("\n[DEBUG] Completed workflow: " .. context.workflow_id)
        print("[DEBUG] Final state: " .. tostring(context.state))
        print("[DEBUG] Result: " .. tostring(context.result))
    end
end)

-- Create a loop workflow with break condition debugging
local debug_loop = Workflow.loop({
    name = "debug_loop",
    description = "Loop with extensive debugging",
    iterator = "range",
    start = 1,
    ["end"] = 10,
    step = 1,
    body = {
        {
            name = "process_item",
            type = "tool",
            tool = "item_processor",
            input = { log_level = "debug" }
        }
    },
    break_conditions = {
        {
            type = "shared_data_equals",
            key = "stop_processing",
            expected = true
        }
    },
    error_strategy = "continue"
})

-- Add metrics tracking
debug_loop:onBeforeExecute(function(context)
    debug_loop:setState("iteration_count", 0)
    debug_loop:setState("start_time", os.time())
end)

-- Track each iteration
debug_loop:onAfterExecute(function(context)
    local iterations = debug_loop:getState("iteration_count") or 0
    local duration = os.time() - (debug_loop:getState("start_time") or os.time())
    
    print(string.format("\n=== Loop Metrics ==="))
    print(string.format("Total iterations: %d", iterations))
    print(string.format("Duration: %d seconds", duration))
    print(string.format("Avg time per iteration: %.2f seconds", duration / math.max(iterations, 1)))
end)

-- Get workflow metrics
print("\n=== Workflow Metrics ===")
local metrics = debug_workflow:getMetrics()
print("Total executions: " .. metrics.execution.total_executions)
print("Successful: " .. metrics.execution.successful_executions)
print("Failed: " .. metrics.execution.failed_executions)
print("Average duration: " .. metrics.execution.average_duration_ms .. "ms")

-- Create a parallel workflow with error aggregation
local parallel_debug = Workflow.parallel({
    name = "parallel_error_test",
    description = "Test parallel execution with mixed success/failure",
    max_concurrency = 3,
    error_strategy = "continue", -- Continue even if some branches fail
    branches = {
        {
            name = "branch_success",
            steps = {{
                name = "success_step",
                type = "tool",
                tool = "timer",
                input = { seconds = 1 }
            }}
        },
        {
            name = "branch_timeout",
            steps = {{
                name = "timeout_step",
                type = "tool",
                tool = "error_simulator",
                input = { error_type = "timeout", delay = 5000 }
            }}
        },
        {
            name = "branch_error",
            steps = {{
                name = "error_step",
                type = "tool",
                tool = "error_simulator",
                input = { error_type = "runtime", message = "Intentional error" }
            }}
        }
    }
})

-- Aggregate errors from parallel execution
parallel_debug:onAfterExecute(function(context)
    print("\n=== Parallel Execution Summary ===")
    local errors = parallel_debug:getState("branch_errors") or {}
    local successes = parallel_debug:getState("branch_successes") or {}
    
    print("Successful branches: " .. #successes)
    for _, branch in ipairs(successes) do
        print("  ✓ " .. branch)
    end
    
    print("Failed branches: " .. #errors)
    for _, error in ipairs(errors) do
        print("  ✗ " .. error.branch .. ": " .. error.message)
    end
end)

-- Demonstrate workflow introspection
print("\n=== Workflow Introspection ===")
local all_workflows = Workflow.list()
for _, wf in ipairs(all_workflows) do
    local workflow = Workflow.get(wf.id)
    if workflow then
        local info = workflow:getInfo()
        local validation = workflow:validate()
        print(string.format("\nWorkflow: %s (%s)", info.name, info.type))
        print("  Valid: " .. tostring(validation.valid))
        
        -- Get and display debug info
        local debug = workflow:debug()
        print("  Debug ID: " .. debug.workflow_id)
        
        -- Check for any stored errors
        local error_count = workflow:getState("error_count") or 0
        if error_count > 0 then
            print("  Errors encountered: " .. error_count)
            local last_error = workflow:getState("last_error")
            if last_error then
                print("  Last error: " .. tostring(last_error))
            end
        end
    end
end

-- Cleanup
print("\n=== Disabling Debug Mode ===")
Workflow.enableDebug(false)
print("Debug mode enabled: " .. tostring(Workflow.isDebugEnabled()))

print("\n=== Debugging Example Complete ===")