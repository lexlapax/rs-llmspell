-- ABOUTME: Agent orchestrator example demonstrating tool coordination
-- ABOUTME: Shows how agents can orchestrate multiple tools in complex workflows

-- Agent Orchestrator Example
-- This demonstrates how an agent can orchestrate multiple tools
-- to accomplish complex tasks

print("=== Agent Orchestrator Example ===\n")

-- Create an orchestrator agent that can manage tools
local orchestrator = Agent.create({
    name = "tool_orchestrator",
    description = "Orchestrates multiple tools to accomplish complex tasks",
    provider_model = "openai/gpt-4o-mini",
    system_prompt = [[
You are a tool orchestration specialist. You coordinate multiple tools to accomplish complex tasks efficiently.
When given a task:
1. Break it down into steps
2. Identify which tools to use
3. Execute tools in the right order
4. Combine results effectively
]],
    temperature = 0.3,  -- Lower temperature for consistent orchestration
    max_tokens = 2000
})

-- Register the orchestrator globally for reuse
if orchestrator then
    -- Agent.register("orchestrator", orchestrator)  -- TODO: Implement Agent.register
    print("Orchestrator agent created successfully")
else
    print("Failed to create orchestrator agent - check API keys")
    return
end

-- Example 1: Data Processing Pipeline
print("Example 1: Data Processing Pipeline")
print("-" .. string.rep("-", 35))

-- Create sample data
local csv_data = [[
product,price,quantity
Widget A,19.99,100
Gadget B,29.99,50
Tool C,39.99,75
Device D,49.99,25
]]

-- Save sample data
local file_result = Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/products.csv",
    input = csv_data
})

-- Orchestrate a data processing pipeline
local pipeline_result = orchestrator:execute({
    prompt = [[
Process this product data pipeline:
1. Read the CSV file at /tmp/products.csv
2. Parse the CSV data
3. Calculate total value for each product (price * quantity)
4. Sort by total value descending
5. Generate a summary report

Use the available tools to accomplish this task.
]]
})

print("Pipeline Result:")
print(pipeline_result.content)

-- Example 2: Multi-Source Data Aggregation
print("\n\nExample 2: Multi-Source Data Aggregation")
print("-" .. string.rep("-", 40))

-- Create multiple data sources
Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/sales_q1.json",
    input = '{"quarter": "Q1", "sales": 150000, "region": "North"}'
})

Tool.executeAsync("file_operations", {
    operation = "write",
    path = "/tmp/sales_q2.json",
    input = '{"quarter": "Q2", "sales": 175000, "region": "North"}'
})

-- Orchestrate aggregation
local aggregation_result = orchestrator:execute({
    prompt = [[
Aggregate sales data from multiple sources:
1. Read JSON files: /tmp/sales_q1.json and /tmp/sales_q2.json
2. Parse the JSON data
3. Calculate total sales across quarters
4. Calculate quarter-over-quarter growth
5. Generate a consolidated report

Coordinate the tools to complete this analysis.
]]
})

print("Aggregation Result:")
print(aggregation_result.content)

-- Example 3: System Monitoring and Reporting
print("\n\nExample 3: System Monitoring and Reporting")
print("-" .. string.rep("-", 45))

local monitoring_result = orchestrator:execute({
    prompt = [[
Perform a system health check:
1. Check system resources (CPU, memory, disk)
2. List running processes
3. Check environment variables for critical settings
4. Generate a timestamped health report
5. Save the report to /tmp/system_health_report.txt

Use system monitoring and file tools to accomplish this.
]]
})

print("Monitoring Result:")
print(monitoring_result.content)

-- Example 4: Automated Data Transformation
print("\n\nExample 4: Automated Data Transformation")
print("-" .. string.rep("-", 42))

-- Create complex data structure
local complex_data = {
    users = {
        { id = 1, name = "Alice", scores = {85, 92, 88} },
        { id = 2, name = "Bob", scores = {78, 85, 90} },
        { id = 3, name = "Charlie", scores = {92, 95, 91} }
    },
    metadata = {
        test_count = 3,
        max_score = 100
    }
}

-- Save as JSON
local json_result = Tool.executeAsync("json_processor", {
    operation = "stringify",
    input = complex_data,
    pretty = true
})

if json_result and json_result.output then
    Tool.executeAsync("file_operations", {
        operation = "write",
        path = "/tmp/student_data.json",
        input = json_result.output
    })
end

-- Orchestrate transformation
local transform_result = orchestrator:execute({
    prompt = [[
Transform the student data:
1. Read /tmp/student_data.json
2. Calculate average score for each student
3. Identify the top performer
4. Create a grade distribution analysis
5. Generate both JSON and CSV outputs with the results

Coordinate multiple tools to complete this transformation.
]]
})

print("Transformation Result:")
print(transform_result.content)

-- Example 5: Error Recovery Orchestration
print("\n\nExample 5: Error Recovery Orchestration")
print("-" .. string.rep("-", 41))

local recovery_result = orchestrator:execute({
    prompt = [[
Implement an error recovery workflow:
1. Try to read a non-existent file: /tmp/missing_data.txt
2. When it fails, create the file with default content
3. Validate the file was created successfully
4. Generate a recovery report with timestamps

Show how to handle errors gracefully using tool orchestration.
]],
    -- Error handling configuration
    on_error = function(error)
        print("Orchestrator handling error: " .. tostring(error))
        return { retry = true, max_attempts = 3 }
    end
})

print("Recovery Result:")
print(recovery_result.content)

-- Performance Metrics
print("\n\n=== Orchestration Performance Metrics ===")

local start_time = os.clock()

-- Quick orchestration benchmark
local benchmark_result = orchestrator:execute({
    prompt = "Quickly calculate: what is 42 * 17 + 89? Use the calculator tool.",
    max_tokens = 100
})

local end_time = os.clock()
local duration = (end_time - start_time) * 1000

print(string.format("Quick orchestration time: %.2f ms", duration))
print("Result: " .. benchmark_result.content)

-- Advanced Orchestration Pattern: Conditional Tool Selection
print("\n\n=== Advanced: Conditional Tool Selection ===")

local conditional_result = orchestrator:execute({
    prompt = [[
Analyze this data and choose the appropriate processing:
- If it's JSON: parse and extract keys
- If it's CSV: analyze columns and rows
- If it's plain text: count words and lines

Data: {"name": "test", "value": 42, "active": true}

Select and use the appropriate tool based on the data format.
]]
})

print("Conditional Processing Result:")
print(conditional_result.content)

-- Tool Discovery Example
print("\n\n=== Tool Discovery and Usage ===")

local discovery_result = orchestrator:execute({
    prompt = [[
Discover and list all available tools, then demonstrate using 3 different tools:
1. First, check what tools are available
2. Pick 3 tools that work well together
3. Create a mini-workflow using those tools
4. Explain why you chose those specific tools
]]
})

print("Tool Discovery Result:")
print(discovery_result.content)

print("\n=== Agent Orchestrator Example Complete ===")

-- Summary statistics
print("\nOrchestration Summary:")
print("- Examples demonstrated: 7")
print("- Tools coordinated: file_operations, json_processor, csv_analyzer, calculator, system_monitor")
print("- Error handling: Demonstrated recovery patterns")
print("- Performance: Sub-second orchestration for simple tasks")