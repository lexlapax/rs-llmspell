-- ABOUTME: Basic sequential workflow example using only tool steps
-- ABOUTME: Demonstrates simple step-by-step execution without custom functions

-- Load workflow helpers for async execution
local helpers = dofile("examples/lua/workflows/workflow-helpers.lua")
-- Load tool helpers for async tool invocation
local tool_helpers = dofile("examples/lua/tools/tool-helpers.lua")

print("=== Basic Sequential Workflow Example ===\n")

-- Example 1: Simple Tool Chain
print("Example 1: Simple Tool Chain")
print("-" .. string.rep("-", 28))

local simple_workflow = Workflow.sequential({
    name = "simple_tool_chain",
    description = "Execute tools in sequence",
    
    steps = {
        -- Step 1: Generate a UUID
        {
            name = "generate_id",
            tool = "uuid_generator",
            input = { version = "v4" }
        },
        -- Step 2: Get current timestamp
        {
            name = "get_timestamp",
            tool = "date_time_handler",
            input = {
                operation = "now",
                format = "ISO8601"
            }
        },
        -- Step 3: Create a formatted message
        {
            name = "format_message",
            tool = "template_engine",
            input = {
                template = "Task {{id}} started at {{timestamp}}",
                variables = {
                    id = "{{step:generate_id:output}}",
                    timestamp = "{{step:get_timestamp:output}}"
                }
            }
        }
    }
})

print("Executing simple tool chain...")
local result, err = helpers.executeWorkflow(simple_workflow)

if result and result.success then
    print("✓ Workflow completed!")
    print("Final output: " .. (result.data and result.data.final_output or "N/A"))
else
    print("✗ Workflow failed: " .. tostring(err or (result and result.error)))
end

-- Example 2: Data Processing Pipeline
print("\n\nExample 2: Data Processing Pipeline")
print("-" .. string.rep("-", 35))

-- Create some sample data
local sample_data = {
    items = {
        { name = "Apple", price = 1.50, quantity = 10 },
        { name = "Banana", price = 0.75, quantity = 20 },
        { name = "Orange", price = 2.00, quantity = 15 }
    }
}

-- Save data to file for processing
local json_result = tool_helpers.invokeTool("json_processor", {
    operation = "stringify",
    input = sample_data,
    pretty = true
})

if json_result and json_result.output then
    tool_helpers.invokeTool("file_operations", {
        operation = "write",
        path = "/tmp/inventory.json",
        content = json_result.output
    })
end

local data_pipeline = Workflow.sequential({
    name = "data_processing_pipeline",
    description = "Process inventory data through multiple steps",
    
    steps = {
        -- Read the data file
        {
            name = "read_data",
            tool = "file_operations",
            input = {
                operation = "read",
                path = "/tmp/inventory.json"
            }
        },
        -- Parse and query the JSON data
        {
            name = "calculate_total_value",
            tool = "json_processor",
            input = {
                operation = "query",
                input = "{{step:read_data:output}}",
                query = '.items | map(.price * .quantity) | add'
            }
        },
        -- Format the result
        {
            name = "format_report",
            tool = "template_engine",
            input = {
                template = "Inventory Report\n================\nTotal Value: ${{value}}",
                variables = {
                    value = "{{step:calculate_total_value:output}}"
                }
            }
        },
        -- Save the report
        {
            name = "save_report",
            tool = "file_operations",
            input = {
                operation = "write",
                path = "/tmp/inventory_report.txt",
                content = "{{step:format_report:output}}"
            }
        }
    }
})

print("Executing data processing pipeline...")
local pipeline_result, err = helpers.executeWorkflow(data_pipeline)

if pipeline_result and pipeline_result.success then
    print("✓ Pipeline completed!")
    print("Report saved to: /tmp/inventory_report.txt")
else
    print("✗ Pipeline failed: " .. tostring(err or (pipeline_result and pipeline_result.error)))
end

-- Example 3: Mathematical Calculations
print("\n\nExample 3: Mathematical Calculations")
print("-" .. string.rep("-", 35))

local math_workflow = Workflow.sequential({
    name = "math_calculations",
    description = "Perform sequential mathematical operations",
    
    steps = {
        -- Initial calculation
        {
            name = "calc1",
            tool = "calculator",
            input = { input = "100 * 2 + 50" }
        },
        -- Use previous result
        {
            name = "calc2",
            tool = "calculator",
            input = { input = "{{step:calc1:output}} / 5" }
        },
        -- Final calculation
        {
            name = "calc3",
            tool = "calculator",
            input = { input = "sqrt({{step:calc2:output}})" }
        },
        -- Format result
        {
            name = "format_result",
            tool = "template_engine",
            input = {
                template = "Calculation Result: {{result}} (from initial value 100)",
                variables = {
                    result = "{{step:calc3:output}}"
                }
            }
        }
    }
})

print("Executing mathematical calculations...")
local math_result, err = helpers.executeWorkflow(math_workflow)

if math_result and math_result.success then
    print("✓ Calculations completed!")
    print(math_result.data and math_result.data.final_output or "No output")
else
    print("✗ Calculations failed: " .. tostring(err or (math_result and math_result.error)))
end

-- Example 4: Text Processing
print("\n\nExample 4: Text Processing")
print("-" .. string.rep("-", 26))

local text_workflow = Workflow.sequential({
    name = "text_processing",
    description = "Process text through multiple transformations",
    
    steps = {
        -- Start with some text
        {
            name = "create_text",
            tool = "template_engine",
            input = {
                template = "Hello World! This is a TEST message.",
                variables = {}
            }
        },
        -- Convert to lowercase
        {
            name = "lowercase",
            tool = "text_manipulator",
            input = {
                operation = "lowercase",
                input = "{{step:create_text:output}}"
            }
        },
        -- Replace text
        {
            name = "replace",
            tool = "text_manipulator",
            input = {
                operation = "replace",
                input = "{{step:lowercase:output}}",
                pattern = "test",
                replacement = "demo"
            }
        },
        -- Trim and normalize
        {
            name = "normalize",
            tool = "text_manipulator",
            input = {
                operation = "normalize_whitespace",
                input = "{{step:replace:output}}"
            }
        }
    }
})

print("Executing text processing...")
local text_result, err = helpers.executeWorkflow(text_workflow)

if text_result and text_result.success then
    print("✓ Text processing completed!")
    print("Final text: " .. (text_result.data and text_result.data.final_output or "N/A"))
else
    print("✗ Text processing failed: " .. tostring(err or (text_result and text_result.error)))
end

-- Summary
print("\n\n=== Basic Sequential Workflow Summary ===")
print("Key concepts demonstrated:")
print("1. Tool steps execute in order")
print("2. Step outputs available via {{step:name:output}}")
print("3. No custom functions needed - tools handle all operations")
print("4. Error handling with fail_fast (default)")
print("\nAll operations use the 33+ available tools!")

print("\n=== Example Complete ===")