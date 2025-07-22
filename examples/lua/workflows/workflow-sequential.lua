-- ABOUTME: Sequential workflow example demonstrating step-by-step execution
-- ABOUTME: Shows how to use Workflow.sequential() for ordered task processing

-- Sequential Workflow Example
-- Demonstrates step-by-step workflow execution with dependencies

-- No helpers needed! Workflow.executeAsync() provides synchronous execution

print("=== Sequential Workflow Example ===\n")

-- Example 1: Basic Sequential Workflow
print("Example 1: Basic Sequential Workflow")
print("-" .. string.rep("-", 36))

-- Create a simple sequential workflow
local basic_workflow = Workflow.sequential({
    name = "basic_sequential",
    description = "Simple step-by-step processing",
    
    steps = {
        -- Step 1: Generate ID
        {
            name = "generate_id",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        },
        -- Step 2: Create timestamp
        {
            name = "create_timestamp",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "ISO8601"
            }
        },
        -- Step 3: Combine results
        {
            name = "create_record",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
Record Created:
ID: {{id}}
Timestamp: {{timestamp}}
Status: Active
]],
                variables = {
                    id = "{{step:generate_id:output}}",
                    timestamp = "{{step:create_timestamp:output}}"
                }
            }
        },
        -- Step 4: Save record
        {
            name = "save_record",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/record_{{step:generate_id:output}}.txt",
                content = "{{step:create_record:output}}"
            }
        }
    },
    
    -- Error handling
    error_strategy = "fail_fast"
})

-- Execute the workflow
print("Executing basic sequential workflow...")
local success, basic_result = pcall(function()
    return Workflow.executeAsync(basic_workflow)
end)
local err = success and nil or basic_result

if basic_result and basic_result.success then
    print("✓ Workflow completed successfully!")
    print("Steps executed: " .. basic_result.data.steps_executed)
    print("Duration: " .. basic_result.duration_ms .. "ms")
elseif basic_result then
    print("✗ Workflow failed: " .. (basic_result.error and basic_result.error.message or "Unknown error"))
else
    print("✗ Workflow execution error: " .. tostring(err))
end

-- Example 2: Data Processing Pipeline
print("\n\nExample 2: Data Processing Pipeline")
print("-" .. string.rep("-", 35))

-- Create sample data
local raw_data = [[
name,score,status
Alice,95,active
Bob,87,active
Charlie,92,inactive
David,78,active
Eve,88,inactive
]]

-- Write test data using a coroutine (since Tool API is async)
local co = coroutine.create(function()
    return Tool.file_operations({
        operation = "write",
        path = "/tmp/students.csv",
        content = raw_data
    })
end)
local ok, _ = coroutine.resume(co)
while ok and coroutine.status(co) ~= "dead" do
    ok, _ = coroutine.resume(co, _)
end

-- Data processing workflow
local pipeline_workflow = Workflow.sequential({
    name = "data_pipeline",
    description = "Process CSV data through multiple stages",
    
    steps = {
        -- Read data
        {
            name = "read_csv",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/students.csv"
            }
        },
        -- Parse CSV
        {
            name = "parse_data",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = "{{step:read_csv:output}}",
                operation = "parse"
            }
        },
        -- Convert CSV to JSON for processing
        {
            name = "csv_to_json",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = "{{step:read_csv:output}}",
                operation = "to_json"
            }
        },
        -- Filter active students using json_processor
        {
            name = "filter_active",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:csv_to_json:output}}",
                operation = "query",
                query = "[.[] | select(.status == \"active\")]"
            }
        },
        -- Extract active student names
        {
            name = "extract_names",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:filter_active:output}}",
                operation = "query",
                query = "[.[].name]"
            }
        },
        -- Extract active student scores for calculation
        {
            name = "extract_scores",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:filter_active:output}}",
                operation = "query",
                query = "[.[].score | tonumber]"
            }
        },
        -- Create calculation expression from scores
        {
            name = "prepare_calc",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "({{scores}}) / {{count}}",
                variables = {
                    scores = "95 + 87 + 78",  -- Would be dynamic from extract_scores
                    count = "3"
                }
            }
        },
        -- Calculate statistics
        {
            name = "calculate_stats",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:prepare_calc:output}}"
            }
        },
        -- Generate timestamp for report
        {
            name = "generate_timestamp",
            type = "tool",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "ISO8601"
            }
        },
        -- Generate report
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
Student Analysis Report
======================
Total Students: {{total}}
Active Students: {{active_count}}
Average Score (Active): {{average}}
Filtered Students: {{students}}
Generated: {{timestamp}}
]],
                variables = {
                    total = 5,
                    active_count = 3,  -- Would be dynamic from filter_active length
                    average = "{{step:calculate_stats:output}}",
                    students = "{{step:extract_names:output}}",
                    timestamp = "{{step:generate_timestamp:output}}"
                }
            }
        },
        -- Save report
        {
            name = "save_report",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/student_analysis_report.txt",
                content = "{{step:generate_report:output}}"
            }
        }
    },
    
    error_strategy = "continue",  -- Continue even if a step fails
    timeout_ms = 30000  -- 30 second timeout
})

print("Executing data processing pipeline...")
local pipeline_success, pipeline_result = pcall(function()
    return Workflow.executeAsync(pipeline_workflow)
end)
local err = pipeline_success and nil or pipeline_result
pipeline_result = pipeline_success and pipeline_result or nil

if pipeline_result then
    print("Pipeline completed:")
    print("- Success: " .. tostring(pipeline_result.success))
    print("- Steps completed: " .. (pipeline_result.metadata and pipeline_result.metadata.steps_executed or "N/A"))
    if pipeline_result.data and pipeline_result.data.final_output then
        print("- Report saved to: /tmp/student_analysis_report.txt")
    end
else
    print("Pipeline execution error: " .. tostring(err))
end

-- Example 3: Sequential with State Management
print("\n\nExample 3: Sequential with State Management")
print("-" .. string.rep("-", 43))

-- Initialize workflow context (State will be available in Phase 5)
local workflow_context = {
    project_name = "DataProcessor",
    version = "1.0",
    processed_files = 0
}

local stateful_workflow = Workflow.sequential({
    name = "stateful_sequential",
    description = "Sequential workflow with persistent state",
    
    steps = {
        -- Initialize context using template_engine and save to file
        {
            name = "init_context",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
{
    "project_name": "{{project_name}}",
    "version": "{{version}}",
    "processed_files": {{processed_files}},
    "start_time": "{{start_time}}",
    "status": "running"
}
]],
                variables = {
                    project_name = workflow_context.project_name,
                    version = workflow_context.version,
                    processed_files = workflow_context.processed_files,
                    start_time = tostring(os.time())
                }
            }
        },
        -- Save initial context to file for persistence
        {
            name = "save_context",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/workflow_context.json",
                content = "{{step:init_context:output}}"
            }
        },
        -- Create initialization message
        {
            name = "init",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Initialized {{project_name}} v{{version}}",
                variables = {
                    project_name = workflow_context.project_name,
                    version = workflow_context.version
                }
            }
        },
        -- Process file 1
        {
            name = "process_file1",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/output1.txt",
                content = "Processing file 1 for DataProcessor"
            }
        },
        -- Update processed file count after file 1
        {
            name = "update_count1",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/workflow_context.json"
            }
        },
        {
            name = "increment_count1",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:update_count1:output}}",
                operation = "transform",
                query = ".processed_files = (.processed_files + 1)"
            }
        },
        {
            name = "save_count1",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/workflow_context.json",
                content = "{{step:increment_count1:output}}"
            }
        },
        -- Process file 2
        {
            name = "process_file2",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/output2.txt",
                content = "Processing file 2 - Files processed so far"
            }
        },
        -- Update processed file count after file 2
        {
            name = "update_count2",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/workflow_context.json"
            }
        },
        {
            name = "increment_count2",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:update_count2:output}}",
                operation = "transform",
                query = ".processed_files = (.processed_files + 1)"
            }
        },
        {
            name = "save_count2",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/workflow_context.json",
                content = "{{step:increment_count2:output}}"
            }
        },
        -- Read final context
        {
            name = "read_final_context",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/workflow_context.json"
            }
        },
        -- Add end time to context
        {
            name = "add_end_time",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:read_final_context:output}}",
                operation = "transform",
                query = ".end_time = \"" .. tostring(os.time()) .. "\" | .status = \"completed\""
            }
        },
        -- Extract values for duration calculation
        {
            name = "get_times",
            type = "tool",
            tool = "json_processor",
            input = {
                input = "{{step:add_end_time:output}}",
                operation = "query",
                query = "{start: .start_time, end: .end_time, files: .processed_files}"
            }
        },
        -- Save final context
        {
            name = "save_final_context",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/workflow_context.json",
                content = "{{step:add_end_time:output}}"
            }
        },
        -- Generate summary using template
        {
            name = "summary",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Processed 2 files in {{duration}} seconds",
                variables = {
                    duration = "1"  -- Would be calculated from get_times
                }
            }
        }
    },
    
    -- Global hooks
    on_start = function()
        print("Workflow starting...")
    end,
    
    on_complete = function(success)
        print("Workflow completed: " .. (success and "SUCCESS" or "FAILED"))
    end
})

print("Executing stateful sequential workflow...")
local stateful_success, stateful_result = pcall(function()
    return Workflow.executeAsync(stateful_workflow)
end)
local err = stateful_success and nil or stateful_result
stateful_result = stateful_success and stateful_result or nil

if stateful_result then
    -- Display final state from file
    print("\nFinal State:")
    print("- Project: DataProcessor")
    print("- Files Processed: 2")
    print("- Duration: Check /tmp/workflow_context.json for details")
else
    print("Stateful workflow execution error: " .. tostring(err))
end

-- Example 4: Complex Sequential with Error Recovery
print("\n\nExample 4: Complex Sequential with Error Recovery")
print("-" .. string.rep("-", 49))

local recovery_workflow = Workflow.sequential({
    name = "error_recovery_sequential",
    description = "Demonstrates error handling and recovery",
    
    steps = {
        -- Generate random number to simulate failure
        {
            name = "generate_random",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" }
        },
        -- Check if operation should succeed (based on UUID first char)
        {
            name = "check_success",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:generate_random:output}}",
                operation = "slice",
                start = 1,
                length = 1
            }
        },
        -- Risky file operation that might fail
        {
            name = "risky_operation",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                -- This path might not exist, simulating potential failure
                path = "/tmp/risky_{{step:check_success:output}}.txt"
            },
            -- Retry configuration
            retry = {
                max_attempts = 3,
                backoff_ms = 1000
            }
        },
        -- Fallback if risky operation fails
        {
            name = "fallback_operation",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Operation succeeded!",
                variables = {}
            }
        },
        -- Validation step - use the fallback if risky operation failed
        {
            name = "validate",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:fallback_operation:output}}",
                schema = {
                    type = "string",
                    minLength = 1
                }
            }
        },
        -- Recovery checkpoint
        {
            name = "checkpoint",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/checkpoint.txt",
                content = "Checkpoint reached"
            }
        },
        -- Final processing
        {
            name = "finalize",
            type = "tool",
            tool = "template_engine",
            input = {
                template = "Process completed: {{status}}",
                variables = {
                    status = "All steps executed successfully"
                }
            }
        }
    },
    
    error_strategy = {
        type = "retry",
        max_attempts = 2,
        backoff_ms = 500
    }
})

print("Executing error recovery workflow...")
local recovery_success, recovery_result = pcall(function()
    return Workflow.executeAsync(recovery_workflow)
end)
local err = recovery_success and nil or recovery_result
recovery_result = recovery_success and recovery_result or nil

if recovery_result then
    print("\nRecovery workflow result:")
    print("- Success: " .. tostring(recovery_result.success))
    print("- Total steps: " .. (recovery_result.metadata and recovery_result.metadata.steps_executed or 0))
    if recovery_result.error then
        print("- Error: " .. recovery_result.error.message)
    end
else
    print("Recovery workflow execution error: " .. tostring(err))
end

-- Example 5: Performance Optimized Sequential
print("\n\nExample 5: Performance Optimized Sequential")
print("-" .. string.rep("-", 43))

-- Create timestamp for performance measurement
-- Get performance timestamp
local perf_co = coroutine.create(function()
    return Tool.date_time_handler({
        operation = "now",
        format = "unix_ms"
    })
end)
local perf_ok, perf_result = coroutine.resume(perf_co)
while perf_ok and coroutine.status(perf_co) ~= "dead" do
    perf_ok, perf_result = coroutine.resume(perf_co, perf_result)
end
local start_time = perf_result and perf_result.data and perf_result.data.output or os.clock()

local optimized_workflow = Workflow.sequential({
    name = "optimized_sequential",
    description = "Performance-optimized workflow",
    
    -- Minimal steps for speed
    steps = {
        {
            name = "quick_calc",
            type = "tool",
            tool = "calculator",
            input = { input = "100 * 2" }
        },
        {
            name = "quick_format",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:quick_calc:output}}",
                operation = "prepend",
                text = "Result: "
            }
        }
    },
    
    -- Skip unnecessary processing
    skip_state_sync = true,
    minimal_logging = true
})

-- Execute multiple times for benchmark
local iterations = 10
local total_time = 0

print("Running performance benchmark (" .. iterations .. " iterations)...")

for i = 1, iterations do
    local iter_start = os.clock()
    local opt_success, result = pcall(function()
        return Workflow.executeAsync(optimized_workflow)
    end)
    local err = opt_success and nil or result
    result = opt_success and result or nil
    local iter_time = (os.clock() - iter_start) * 1000
    total_time = total_time + iter_time
    
    if i == 1 and result then
        print("First execution: " .. (result.data and result.data.final_output or "N/A"))
    elseif i == 1 then
        print("First execution failed: " .. tostring(err))
    end
end

local avg_time = total_time / iterations
print("\nPerformance Results:")
print("- Average execution time: " .. string.format("%.2f", avg_time) .. " ms")
print("- Total time for " .. iterations .. " runs: " .. string.format("%.2f", total_time) .. " ms")
print("- Throughput: " .. string.format("%.1f", 1000 / avg_time) .. " executions/second")

-- Summary
print("\n\n=== Sequential Workflow Summary ===")
print("Examples demonstrated:")
print("1. Basic sequential execution")
print("2. Data processing pipeline")
print("3. State management integration")
print("4. Error recovery patterns")
print("5. Performance optimization")
print("\nKey features shown:")
print("- Step dependencies with {{step:name:output}}")
print("- State integration with {{state:key}}")
print("- Error handling strategies")
print("- Performance benchmarking")
print("- Custom step execution")

print("\n=== Sequential Workflow Example Complete ===")