-- ABOUTME: Basic sequential workflow example demonstrating tool integration
-- ABOUTME: Shows step-by-step tool execution with data passing between steps

-- Sequential ETL Pipeline Example
-- This demonstrates a basic Extract-Transform-Load pipeline using tools

-- Create sequential workflow for data processing
local workflow = Workflow.sequential({
    name = "etl_pipeline",
    description = "Extract, Transform, and Load data pipeline",
    
    -- Define steps
    steps = {
        -- Step 1: Extract - Read CSV data
        {
            name = "extract_csv",
            type = "tool",
            tool = "csv_analyzer",
            input = {
                input = [[
name,age,city
Alice,30,New York
Bob,25,San Francisco
Charlie,35,Chicago
]],
                operation = "analyze"
            }
        },
        
        -- Step 2: Transform - Process the data
        {
            name = "transform_data",
            type = "tool", 
            tool = "json_processor",
            input = {
                -- Reference output from previous step
                input = "{{step:extract_csv:output}}",
                operation = "parse"
            }
        },
        
        -- Step 3: Validate - Check data integrity
        {
            name = "validate_data",
            type = "tool",
            tool = "data_validation",
            input = {
                input = "{{step:transform_data:output}}",
                schema = {
                    type = "object",
                    properties = {
                        rows = { type = "number" },
                        columns = { type = "array" }
                    }
                }
            }
        },
        
        -- Step 4: Load - Write to file
        {
            name = "save_results",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/etl_results.json",
                content = "{{step:transform_data:output}}"
            }
        }
    },
    
    -- Error handling configuration
    error_strategy = "fail_fast",  -- Stop on first error
    timeout_ms = 30000  -- 30 second timeout
})

-- Execute the workflow
print("Starting ETL pipeline...")
local result = workflow:execute()

-- Check results
if result.success then
    print("✓ ETL pipeline completed successfully!")
    print("Steps executed: " .. result.data.steps_executed)
    print("Duration: " .. result.duration_ms .. "ms")
else
    print("✗ ETL pipeline failed: " .. (result.error and result.error.message or "Unknown error"))
end

-- Sequential Calculation Pipeline
-- Demonstrates mathematical operations in sequence

local calc_workflow = Workflow.sequential({
    name = "calculation_pipeline",
    description = "Multi-step calculation workflow",
    
    steps = {
        {
            name = "initial_calc",
            type = "tool",
            tool = "calculator",
            input = { input = "100 * 2.5" }
        },
        {
            name = "add_tax",
            type = "tool",
            tool = "calculator",
            input = { 
                -- Use previous result in calculation
                input = "{{step:initial_calc:output}} * 1.08"
            }
        },
        {
            name = "apply_discount",
            type = "tool",
            tool = "calculator",
            input = {
                input = "{{step:add_tax:output}} * 0.9"
            }
        },
        {
            name = "format_result",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "Final price: ${{step:apply_discount:output}}",
                operation = "format"
            }
        }
    }
})

print("\nStarting calculation pipeline...")
local calc_result = calc_workflow:execute()

if calc_result.success then
    print("✓ Calculation completed!")
    print("Final output: " .. calc_result.data.final_output)
end

-- File Processing Pipeline
-- Demonstrates file operations in sequence

local file_workflow = Workflow.sequential({
    name = "file_processor",
    description = "Process and archive files",
    
    steps = {
        -- Create a test file
        {
            name = "create_file",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/test_data.txt",
                content = "Sample data for processing\nLine 2\nLine 3"
            }
        },
        
        -- Read and process
        {
            name = "read_file", 
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/test_data.txt"
            }
        },
        
        -- Convert to uppercase
        {
            name = "process_content",
            type = "tool",
            tool = "text_manipulator",
            input = {
                input = "{{step:read_file:output}}",
                operation = "uppercase"
            }
        },
        
        -- Save processed file
        {
            name = "save_processed",
            type = "tool",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/processed_data.txt",
                content = "{{step:process_content:output}}"
            }
        },
        
        -- Create archive
        {
            name = "archive_files",
            type = "tool",
            tool = "archive_handler",
            input = {
                operation = "create",
                format = "zip",
                source_path = "/tmp",
                target_path = "/tmp/processed_archive.zip",
                include_patterns = {"*data.txt"}
            }
        }
    },
    
    error_strategy = "continue"  -- Continue even if a step fails
})

print("\nStarting file processing pipeline...")
local file_result = file_workflow:execute()

print("File processing completed:")
print("- Success: " .. tostring(file_result.success))
print("- Steps completed: " .. file_result.metadata.steps_executed)
print("- Steps succeeded: " .. file_result.metadata.steps_succeeded)