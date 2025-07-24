-- ABOUTME: Agent data processor example demonstrating ETL and data transformation
-- ABOUTME: Shows how agents can process, transform, and analyze complex data

-- Agent Data Processor Example
-- Demonstrates intelligent data processing, transformation, and analysis

print("=== Agent Data Processor Example ===\n")

-- Create a data processing agent directly
print("Creating data processor agent...")
local success, processor = pcall(function()
    return Agent.create({
        name = "data_processor",
        model = "openai/gpt-4o-mini",
        system_prompt = [[
You are a data processing expert. You excel at:
1. Extracting insights from raw data
2. Transforming data between formats
3. Cleaning and validating data
4. Identifying patterns and anomalies
5. Optimizing data structures

Always ensure data quality and provide clear transformations.
]],
        temperature = 0.3,
        max_tokens = 500
    })
end)

if not success or not processor then
    print("Failed to create processor agent: " .. tostring(processor))
    return
end

print("✓ Data processor agent created\n")

-- Example 1: Process CSV data
print("1. Processing CSV Sales Data")
local csv_data = [[
Date,Product,Sales,Region
2024-01-01,Widget A,150,North
2024-01-01,Widget B,200,South
2024-01-02,Widget A,175,North
2024-01-02,Widget B,225,South
]]

local success1, result1 = pcall(function()
    return processor:invoke({
        text = "Analyze this CSV sales data and provide insights:\n" .. csv_data
    })
end)

if success1 and result1 then
    print("Analysis Result:")
    print(result1.text or result1.output or "No response")
else
    print("Failed to process CSV: " .. tostring(result1))
end

-- Example 2: Transform JSON data
print("\n2. Transforming JSON User Data")
local json_data = [[
{
    "users": [
        {"id": 1, "name": "Alice", "orders": 5, "total_spent": 250.50},
        {"id": 2, "name": "Bob", "orders": 3, "total_spent": 150.25},
        {"id": 3, "name": "Charlie", "orders": 8, "total_spent": 425.00}
    ]
}
]]

local success2, result2 = pcall(function()
    return processor:invoke({
        text = "Transform this JSON data to show average order value per user:\n" .. json_data
    })
end)

if success2 and result2 then
    print("Transformation Result:")
    print(result2.text or result2.output or "No response")
else
    print("Failed to transform JSON: " .. tostring(result2))
end

-- Example 3: Clean messy data
print("\n3. Cleaning and Validating Messy Data")
local messy_data = [[
Name,Email,Phone
John Doe,john@example.com,555-1234
Jane Smith,invalid-email,555-5678
Bob Johnson,bob@example.com,(555) 9012
Alice Brown,,555-3456
]]

local success3, result3 = pcall(function()
    return processor:invoke({
        text = "Clean and validate this contact data, identify any issues:\n" .. messy_data
    })
end)

if success3 and result3 then
    print("Data Cleaning Result:")
    print(result3.text or result3.output or "No response")
else
    print("Failed to clean data: " .. tostring(result3))
end

-- Example 4: Statistical analysis
print("\n4. Performing Statistical Analysis")
local grades_data = [[
Student,Math,Science,English
Alice,85,92,88
Bob,78,85,90
Charlie,92,88,85
Diana,88,90,92
Eve,75,82,78
]]

local success4, result4 = pcall(function()
    return processor:invoke({
        text = "Perform statistical analysis on these student grades (mean, median, std dev):\n" .. grades_data
    })
end)

if success4 and result4 then
    print("Statistical Analysis:")
    print(result4.text or result4.output or "No response")
else
    print("Failed to analyze grades: " .. tostring(result4))
end

-- Example 5: Data aggregation
print("\n5. Data Aggregation Pipeline")
local success5, result5 = pcall(function()
    return processor:invoke({
        text = [[Create a data aggregation pipeline design for:
- Input: Daily sales transactions
- Required aggregations: Daily totals, Weekly summaries, Monthly trends
- Output format: Executive dashboard data
Provide the pipeline steps and transformations needed.]]
    })
end)

if success5 and result5 then
    print("Pipeline Design:")
    print(result5.text or result5.output or "No response")
else
    print("Failed to design pipeline: " .. tostring(result5))
end

print("\n=== Data Processor Example Complete ===")
print("\nKey Demonstrated Capabilities:")
print("- CSV data analysis and insights")
print("- JSON data transformation")
print("- Data cleaning and validation")
print("- Statistical analysis")
print("- Pipeline design and aggregation")
print("\nNote: This example uses the synchronous Agent API")
print("All agent methods are now synchronous - no coroutines needed")