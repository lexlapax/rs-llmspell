-- Example: Workflow Basics - Sequential
-- Purpose: Basic sequential workflow example using tool steps
-- Prerequisites: None (tools work locally)
-- Expected Output: Sequential workflow execution with tool coordination
-- Version: 0.7.0
-- Tags: workflow, sequential, tools

-- ABOUTME: Basic sequential workflow example using only tool steps
-- ABOUTME: Demonstrates simple step-by-step execution without custom functions

print("=== Basic Sequential Workflow Example ===\n")

-- Example 1: Simple Tool Chain
print("Example 1: Simple Tool Chain")
print("-" .. string.rep("-", 28))

-- Create simple sequential workflow using builder pattern
local simple_workflow = Workflow.builder()
    :name("simple_tool_chain")
    :description("Execute tools in sequence")
    :type("sequential")
    :add_step({
        name = "generate_id",
        tool = "uuid_generator",
        params = { 
            operation = "generate",
            version = "v4"
        }
    })
    :add_step({
        name = "get_timestamp",
        tool = "date_time_handler",
        params = {
            operation = "now"
        }
    })
    :add_step({
        name = "format_message",
        tool = "template_engine",
        params = {
            input = "Task {{id}} started at {{timestamp}}",
            context = {
                id = "step_1_result",
                timestamp = "step_2_result"
            }
        }
    })
    :build()

print("Executing simple tool chain...")
local result = simple_workflow:run()

if result and result.success then
    print("✓ Workflow completed!")
    print("Final output: " .. tostring(result.output or "N/A"))
else
    print("✗ Workflow failed: " .. tostring(result and result.error or "Unknown error"))
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
local json_result = Tool.invoke("json_processor", {
    operation = "stringify",
    input = sample_data,
    pretty = true
})

if json_result and json_result.output then
    Tool.invoke("file_operations", {
        operation = "write",
        path = "/tmp/inventory.json",
        input = json_result.output
    })
end

-- Create data processing workflow
local data_pipeline = Workflow.builder()
    :name("data_processing_pipeline")
    :description("Process inventory data through multiple steps")
    :type("sequential")
    :add_step({
        name = "read_data",
        tool = "file_operations",
        params = {
            operation = "read",
            path = "/tmp/inventory.json"
        }
    })
    :add_step({
        name = "parse_data",
        tool = "json_processor",
        params = {
            operation = "parse",
            input = "previous_step_result"
        }
    })
    :add_step({
        name = "format_report",
        tool = "template_engine",
        params = {
            input = "Inventory Report\n================\nData processed successfully",
            context = {}
        }
    })
    :add_step({
        name = "save_report",
        tool = "file_operations",
        params = {
            operation = "write",
            path = "/tmp/inventory_report.txt",
            input = "Report generated"
        }
    })
    :build()

print("Executing data processing pipeline...")
local pipeline_result = data_pipeline:run()

if pipeline_result and pipeline_result.success then
    print("✓ Pipeline completed!")
    print("Report saved to: /tmp/inventory_report.txt")
else
    print("✗ Pipeline failed: " .. tostring(pipeline_result and pipeline_result.error or "Unknown error"))
end

-- Example 3: Mathematical Calculations
print("\n\nExample 3: Mathematical Calculations")
print("-" .. string.rep("-", 35))

-- Create mathematical workflow
local math_workflow = Workflow.builder()
    :name("math_calculations")
    :description("Perform sequential mathematical operations")
    :type("sequential")
    :add_step({
        name = "calc1",
        tool = "calculator",
        params = { 
            operation = "evaluate",
            input = "100 * 2 + 50" 
        }
    })
    :add_step({
        name = "calc2",
        tool = "calculator",
        params = { 
            operation = "evaluate",
            input = "250 / 5" 
        }
    })
    :add_step({
        name = "calc3",
        tool = "calculator",
        params = { 
            operation = "evaluate",
            input = "sqrt(50)" 
        }
    })
    :build()

print("Executing mathematical calculations...")
local math_result = math_workflow:run()

if math_result and math_result.success then
    print("✓ Calculations completed!")
    print(tostring(math_result.output or "No output"))
else
    print("✗ Calculations failed: " .. tostring(math_result and math_result.error or "Unknown error"))
end

-- Example 4: Text Processing
print("\n\nExample 4: Text Processing")
print("-" .. string.rep("-", 26))

-- Create text processing workflow
local text_workflow = Workflow.builder()
    :name("text_processing")
    :description("Process text through multiple transformations")
    :type("sequential")
    :add_step({
        name = "lowercase",
        tool = "text_manipulator",
        params = {
            operation = "lowercase",
            input = "Hello World! This is a TEST message."
        }
    })
    :add_step({
        name = "replace",
        tool = "text_manipulator",
        params = {
            operation = "replace",
            input = "hello world! this is a test message.",
            pattern = "test",
            replacement = "demo"
        }
    })
    :add_step({
        name = "uppercase",
        tool = "text_manipulator",
        params = {
            operation = "uppercase",
            input = "hello world! this is a demo message."
        }
    })
    :build()

print("Executing text processing...")
local text_result = text_workflow:run()

if text_result and text_result.success then
    print("✓ Text processing completed!")
    print("Final text: " .. tostring(text_result.output or "N/A"))
else
    print("✗ Text processing failed: " .. tostring(text_result and text_result.error or "Unknown error"))
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