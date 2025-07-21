-- ABOUTME: Sequential workflow example demonstrating step-by-step execution
-- ABOUTME: Shows how to use Workflow.sequential() for ordered task processing

-- Sequential Workflow Example
-- Demonstrates step-by-step workflow execution with dependencies

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
local basic_result = basic_workflow:execute()

if basic_result.success then
    print("✓ Workflow completed successfully!")
    print("Steps executed: " .. basic_result.data.steps_executed)
    print("Duration: " .. basic_result.duration_ms .. "ms")
else
    print("✗ Workflow failed: " .. (basic_result.error and basic_result.error.message or "Unknown error"))
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

Tools.get("file_operations"):execute({
    operation = "write",
    path = "/tmp/students.csv",
    content = raw_data
})

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
        -- Filter active students
        {
            name = "filter_active",
            type = "custom",
            execute = function(context)
                -- This would normally use the parsed data
                -- For demo, we'll simulate filtering
                return {
                    success = true,
                    output = {
                        filtered_count = 3,
                        active_students = {"Alice", "Bob", "David"}
                    }
                }
            end
        },
        -- Calculate statistics
        {
            name = "calculate_stats",
            type = "tool",
            tool = "calculator",
            input = {
                input = "(95 + 87 + 78) / 3"  -- Average of active students
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
                    active_count = "{{step:filter_active:output.filtered_count}}",
                    average = "{{step:calculate_stats:output}}",
                    students = "{{step:filter_active:output.active_students}}",
                    timestamp = os.date("%Y-%m-%d %H:%M:%S")
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
local pipeline_result = pipeline_workflow:execute()

print("Pipeline completed:")
print("- Success: " .. tostring(pipeline_result.success))
print("- Steps completed: " .. pipeline_result.metadata.steps_executed)
if pipeline_result.data.final_output then
    print("- Report saved to: /tmp/student_analysis_report.txt")
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
        -- Initialize
        {
            name = "init",
            type = "custom",
            execute = function()
                workflow_context.start_time = os.time()
                workflow_context.status = "running"
                
                return {
                    success = true,
                    output = "Initialized " .. workflow_context.project_name .. " v" .. workflow_context.version
                }
            end
        },
        -- Process file 1
        {
            name = "process_file1",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/output1.txt",
                content = "Processing file 1 for " .. workflow_context.project_name
            },
            on_complete = function(result)
                if result.success then
                    workflow_context.processed_files = workflow_context.processed_files + 1
                end
            end
        },
        -- Process file 2
        {
            name = "process_file2",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/output2.txt",
                content = "Processing file 2 - Total processed: {{state:workflow_context.processed_files}}"
            },
            on_complete = function(result)
                if result.success then
                    workflow_context.processed_files = workflow_context.processed_files + 1
                end
            end
        },
        -- Generate summary
        {
            name = "summary",
            type = "custom",
            execute = function()
                local ctx = workflow_context
                ctx.end_time = os.time()
                ctx.duration = ctx.end_time - ctx.start_time
                ctx.status = "completed"
                -- Context updated directly
                
                return {
                    success = true,
                    output = string.format(
                        "Processed %d files in %d seconds",
                        ctx.processed_files,
                        ctx.duration
                    )
                }
            end
        }
    },
    
    -- Global hooks
    on_start = function()
        print("Workflow starting...")
    end,
    
    on_complete = function(success)
        local ctx = workflow_context
        print(string.format(
            "Workflow %s: %s",
            ctx.status,
            success and "SUCCESS" or "FAILED"
        ))
    end
})

print("Executing stateful sequential workflow...")
local stateful_result = stateful_workflow:execute()

-- Display final state
local final_context = workflow_context
print("\nFinal State:")
print("- Project: " .. final_context.project_name)
print("- Files Processed: " .. final_context.processed_files)
print("- Duration: " .. (final_context.duration or 0) .. " seconds")

-- Example 4: Complex Sequential with Error Recovery
print("\n\nExample 4: Complex Sequential with Error Recovery")
print("-" .. string.rep("-", 49))

local recovery_workflow = Workflow.sequential({
    name = "error_recovery_sequential",
    description = "Demonstrates error handling and recovery",
    
    steps = {
        -- Step that might fail
        {
            name = "risky_operation",
            type = "custom",
            execute = function()
                -- Simulate 50% failure rate
                if math.random() > 0.5 then
                    return { success = true, output = "Operation succeeded!" }
                else
                    error("Simulated failure in risky operation")
                end
            end,
            -- Retry configuration
            retry = {
                max_attempts = 3,
                backoff_ms = 1000
            },
            -- Error handler
            on_error = function(err)
                print("Error caught: " .. tostring(err))
                print("Attempting recovery...")
                return { retry = true }
            end
        },
        -- Validation step
        {
            name = "validate",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:risky_operation:output}}",
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
                content = "Checkpoint reached at {{timestamp}}"
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
local recovery_result = recovery_workflow:execute()

print("\nRecovery workflow result:")
print("- Success: " .. tostring(recovery_result.success))
print("- Total steps: " .. (recovery_result.metadata.steps_executed or 0))
if recovery_result.error then
    print("- Error: " .. recovery_result.error.message)
end

-- Example 5: Performance Optimized Sequential
print("\n\nExample 5: Performance Optimized Sequential")
print("-" .. string.rep("-", 43))

-- Measure performance
local start_time = os.clock()

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
    local result = optimized_workflow:execute()
    local iter_time = (os.clock() - iter_start) * 1000
    total_time = total_time + iter_time
    
    if i == 1 then
        print("First execution: " .. result.data.final_output)
    end
end

local avg_time = total_time / iterations
print(string.format("\nPerformance Results:"))
print(string.format("- Average execution time: %.2f ms", avg_time))
print(string.format("- Total time for %d runs: %.2f ms", iterations, total_time))
print(string.format("- Throughput: %.1f executions/second", 1000 / avg_time))

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