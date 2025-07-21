-- ABOUTME: Sequential workflow with state management integration
-- ABOUTME: Demonstrates using State global for data persistence across steps

-- Sequential workflow with state accumulation
-- Shows how to use State.get/set for workflow data management

-- Initialize state
State.set("workflow_data", {
    processed_items = 0,
    total_size = 0,
    errors = {}
})

-- Create a sequential workflow that processes multiple files
local state_workflow = Workflow.sequential({
    name = "stateful_file_processor",
    description = "Process files with state tracking",
    
    steps = {
        -- Step 1: Initialize processing
        {
            name = "init_processing",
            type = "tool",
            tool = "uuid_generator",
            input = { version = "v4" },
            -- Post-step state update
            on_complete = function(result)
                State.set("session_id", result.output)
                State.set("start_time", os.time())
                print("Session initialized: " .. result.output)
            end
        },
        
        -- Step 2: List files to process
        {
            name = "list_files",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "list",
                path = "/tmp"
            },
            on_complete = function(result)
                local files = result.output.files or {}
                State.set("files_to_process", files)
                State.set("total_files", #files)
                print("Found " .. #files .. " files to process")
            end
        },
        
        -- Step 3: Process each file (simplified - normally would use loop)
        {
            name = "process_files",
            type = "custom",
            execute = function()
                local files = State.get("files_to_process") or {}
                local processed = 0
                local total_size = 0
                
                -- Process first 3 files as example
                for i = 1, math.min(3, #files) do
                    local file = files[i]
                    -- Simulate processing
                    processed = processed + 1
                    total_size = total_size + (file.size or 0)
                end
                
                -- Update state
                local data = State.get("workflow_data")
                data.processed_items = processed
                data.total_size = total_size
                State.set("workflow_data", data)
                
                return {
                    success = true,
                    output = string.format("Processed %d files, total size: %d bytes", processed, total_size)
                }
            end
        },
        
        -- Step 4: Generate report
        {
            name = "generate_report",
            type = "tool",
            tool = "template_engine",
            input = {
                template = [[
Processing Report
================
Session ID: {{session_id}}
Start Time: {{start_time}}
Total Files: {{total_files}}
Processed: {{processed_items}}
Total Size: {{total_size}} bytes
Status: {{status}}
]],
                -- Use state values in template
                variables = {
                    session_id = State.get("session_id"),
                    start_time = os.date("%Y-%m-%d %H:%M:%S", State.get("start_time")),
                    total_files = State.get("total_files") or 0,
                    processed_items = State.get("workflow_data").processed_items,
                    total_size = State.get("workflow_data").total_size,
                    status = "Completed"
                }
            }
        },
        
        -- Step 5: Save report
        {
            name = "save_report",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/processing_report_{{step:init_processing:output}}.txt",
                content = "{{step:generate_report:output}}"
            },
            on_complete = function(result)
                -- Final state update
                State.set("report_path", result.output.path or "/tmp/report.txt")
                State.set("completion_time", os.time())
            end
        }
    },
    
    error_strategy = "continue",
    
    -- Global workflow hooks for state management
    on_start = function()
        print("Workflow started - initializing state")
        State.set("workflow_status", "running")
    end,
    
    on_complete = function(success)
        State.set("workflow_status", success and "completed" or "failed")
        local duration = (State.get("completion_time") or os.time()) - (State.get("start_time") or os.time())
        State.set("duration_seconds", duration)
        
        print("\nWorkflow completed:")
        print("- Status: " .. State.get("workflow_status"))
        print("- Duration: " .. duration .. " seconds")
        print("- Report saved to: " .. (State.get("report_path") or "N/A"))
    end
})

-- Execute workflow
print("Starting stateful file processing workflow...")
local result = state_workflow:execute()

-- Display final state
print("\nFinal State:")
print("- Session ID: " .. (State.get("session_id") or "N/A"))
print("- Processed Items: " .. (State.get("workflow_data").processed_items or 0))
print("- Total Size: " .. (State.get("workflow_data").total_size or 0) .. " bytes")

-- Advanced State Pattern: Accumulating results across steps
local accumulator_workflow = Workflow.sequential({
    name = "data_accumulator",
    description = "Accumulate and transform data using state",
    
    steps = {
        -- Initialize accumulator
        {
            name = "init_accumulator",
            type = "custom",
            execute = function()
                State.set("accumulator", {
                    values = {},
                    sum = 0,
                    count = 0,
                    operations = {}
                })
                return { success = true, output = "Accumulator initialized" }
            end
        },
        
        -- Add calculations
        {
            name = "calc_1",
            type = "tool",
            tool = "calculator",
            input = { input = "10 * 5" },
            on_complete = function(result)
                local acc = State.get("accumulator")
                table.insert(acc.values, tonumber(result.output))
                acc.sum = acc.sum + tonumber(result.output)
                acc.count = acc.count + 1
                table.insert(acc.operations, "10 * 5 = " .. result.output)
                State.set("accumulator", acc)
            end
        },
        
        {
            name = "calc_2", 
            type = "tool",
            tool = "calculator",
            input = { input = "100 / 4" },
            on_complete = function(result)
                local acc = State.get("accumulator")
                table.insert(acc.values, tonumber(result.output))
                acc.sum = acc.sum + tonumber(result.output)
                acc.count = acc.count + 1
                table.insert(acc.operations, "100 / 4 = " .. result.output)
                State.set("accumulator", acc)
            end
        },
        
        {
            name = "calc_3",
            type = "tool",
            tool = "calculator",
            input = { input = "15 + 35" },
            on_complete = function(result)
                local acc = State.get("accumulator")
                table.insert(acc.values, tonumber(result.output))
                acc.sum = acc.sum + tonumber(result.output)
                acc.count = acc.count + 1
                table.insert(acc.operations, "15 + 35 = " .. result.output)
                State.set("accumulator", acc)
            end
        },
        
        -- Calculate average
        {
            name = "calculate_average",
            type = "custom",
            execute = function()
                local acc = State.get("accumulator")
                local average = acc.count > 0 and (acc.sum / acc.count) or 0
                State.set("average_result", average)
                
                return {
                    success = true,
                    output = string.format(
                        "Operations: %s\nValues: %s\nSum: %g\nAverage: %g",
                        table.concat(acc.operations, ", "),
                        table.concat(acc.values, ", "),
                        acc.sum,
                        average
                    )
                }
            end
        }
    }
})

print("\n\nStarting accumulator workflow...")
local acc_result = accumulator_workflow:execute()

if acc_result.success then
    print("✓ Accumulator workflow completed!")
    print(acc_result.data.final_output)
    print("Average calculated: " .. State.get("average_result"))
end