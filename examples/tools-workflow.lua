-- tools-workflow.lua: Multi-tool workflow demonstrations
-- Shows how to chain tools together for complex operations

-- Load test helpers for consistent formatting
local helpers = dofile("examples/test-helpers.lua")

print("🔗 Multi-Tool Workflow Examples")
print("=================================")
print()

-- Helper function to show workflow steps
local function show_workflow(name, description)
    print(string.format("[1m[35m=== %s ===[0m", name))
    print(description)
    print()
end

-- Workflow 1: Data Processing Pipeline
show_workflow("Data Processing Pipeline", "Fetch data → Process → Validate → Transform")

-- Step 1: Generate test data
local uuid_gen = Tool.get("uuid_generator")
local test_id = uuid_gen.execute({format = "standard"})
local uuid_value = nil
if test_id.success and test_id.output then
    -- Parse JSON output to get UUID
    uuid_value = test_id.output:match('"uuid"%s*:%s*"([^"]+)"')
end
print("Step 1 - Generated test ID:", uuid_value or "failed")

-- Step 2: Create JSON data
local json_data = {
    id = uuid_value or "unknown",
    name = "Test Workflow",
    timestamp = os.date("!%Y-%m-%dT%H:%M:%SZ"),
    values = {10, 20, 30, 40, 50}
}

-- Step 3: Process with JSON tool
local json_tool = Tool.get("json_processor")
-- First convert Lua table to JSON string
local json_string = string.format(
    '{"id":"%s","name":"%s","timestamp":"%s","values":[%s]}',
    json_data.id,
    json_data.name,
    json_data.timestamp,
    table.concat(json_data.values, ",")
)
-- Parse and pretty-print using jq
local json_result = json_tool.execute({
    operation = "query",
    input = json_string,
    query = "."  -- Identity query to pretty-print
})
print("\nStep 2 - Formatted JSON:")
if json_result.success then
    print(json_result.output)
else
    print("Error:", json_result.error)
end

-- Step 4: Calculate checksum
local hash_tool = Tool.get("hash_calculator")
local checksum = hash_tool.execute({
    operation = "hash",
    algorithm = "sha256",
    input = type(json_result.output) == "string" and json_result.output or tostring(json_result.output)
})
local hash_value = nil
if checksum.success and checksum.output then
    hash_value = checksum.output:match('"hash"%s*:%s*"([^"]+)"')
end
print("\nStep 3 - Data checksum:", hash_value or "failed")

-- Step 5: Encode for transmission
local base64_tool = Tool.get("base64_encoder")
local to_encode = json_result.success and json_result.output or json_string
local encoded = base64_tool.execute({
    operation = "encode",
    input = to_encode
})
if encoded.success and encoded.output then
    -- Extract the base64 string from JSON output
    local b64_value = encoded.output:match('"output"%s*:%s*"([^"]+)"')
    if b64_value then
        print("\nStep 4 - Base64 encoded (first 50 chars):", b64_value:sub(1, 50) .. "...")
    else
        print("\nStep 4 - Base64 encoding failed")
    end
else
    print("\nStep 4 - Base64 encoding error:", encoded.error)
end

print("\n✅ Data processing pipeline complete!")

-- Workflow 2: File Analysis Pipeline
show_workflow("File Analysis Workflow", "Create file → Analyze → Transform → Archive")

-- Step 1: Create test file content
local template_tool = Tool.get("template_engine")
local file_content = template_tool.execute({
    engine = "handlebars",
    template = [[
# Analysis Report {{report_id}}
Generated: {{timestamp}}

## Data Summary
Total Items: {{total}}
Average Value: {{average}}

## Items
{{#each items}}
- Item {{@index}}: {{this}}
{{/each}}
]],
    context = {
        report_id = "WF-2025-001",
        timestamp = os.date("!%Y-%m-%d %H:%M:%S UTC"),
        total = 5,
        average = 30,
        items = {10, 20, 30, 40, 50}
    }
})
print("Step 1 - Generated report content")

-- Step 2: Convert to different formats
local text_tool = Tool.get("text_manipulator")
local uppercase_report = text_tool.execute({
    operation = "uppercase",
    text = file_content.output
})
print("\nStep 2 - Converted to uppercase (first line):")
print(uppercase_report.output:match("^[^\n]+"))

-- Step 3: Calculate diff between versions
local diff_tool = Tool.get("diff_calculator")
local diff_result = diff_tool.execute({
    operation = "text_diff",
    old_text = file_content.output,
    new_text = uppercase_report.output,
    format = "unified",
    context_lines = 1
})
print("\nStep 3 - Generated diff (", diff_result.output:match("@@ .-@@"), ")")

print("\n✅ File analysis workflow complete!")

-- Workflow 3: System Monitoring Pipeline
show_workflow("System Monitoring Workflow", "Read env → Check system → Process data → Report")

-- Step 1: Read environment
local env_tool = Tool.get("environment_reader")
local env_data = env_tool.execute({
    operation = "get_all",
    filter = "PATH|HOME|USER"
})
print("Step 1 - Read environment variables")

-- Step 2: Get system info
local system_tool = Tool.get("system_monitor")
local system_info = system_tool.execute({
    operation = "info"
})
print("\nStep 2 - Retrieved system information")

-- Step 3: Create monitoring report
local calc_tool = Tool.get("calculator")
local memory_percent = calc_tool.execute({
    expression = "100 * 0.75"  -- Example: 75% memory usage
})
local calc_result = nil
if memory_percent.success and memory_percent.output then
    calc_result = memory_percent.output:match('"result"%s*:%s*([^,}]+)')
end
print("\nStep 3 - Calculated metrics:", (calc_result or "N/A") .. "% memory usage")

-- Step 4: Generate timestamp
local datetime_tool = Tool.get("date_time_handler")
local timestamp = datetime_tool.execute({
    operation = "now",
    format = "%Y-%m-%d %H:%M:%S"
})
local timestamp_value = nil
if timestamp.success and timestamp.output then
    timestamp_value = timestamp.output:match('"datetime"%s*:%s*"([^"]+)"')
end
print("\nStep 4 - Report timestamp:", timestamp_value or "N/A")

print("\n✅ System monitoring workflow complete!")

-- Workflow 4: Data Validation Pipeline
show_workflow("Data Validation Workflow", "Generate → Validate → Transform → Store")

-- Step 1: Generate test data
local uuid_data = {}
for i = 1, 3 do
    local id = uuid_gen.execute({format = "standard"})
    if id.success and id.output then
        local uuid = id.output:match('"uuid"%s*:%s*"([^"]+)"')
        if uuid then
            table.insert(uuid_data, uuid)
        end
    end
end
print("Step 1 - Generated", #uuid_data, "UUIDs")

-- Step 2: Create CSV data
local csv_content = "id,name,value\n"
for i, uuid in ipairs(uuid_data) do
    csv_content = csv_content .. string.format("%s,Item-%d,%d\n", uuid, i, i * 100)
end

-- Step 3: Analyze CSV
local csv_tool = Tool.get("csv_analyzer")
local csv_analysis = csv_tool.execute({
    operation = "analyze",
    input = csv_content
})
print("\nStep 2 - CSV analysis complete")

-- Step 4: Transform data
local hash_result = hash_tool.execute({
    operation = "hash",
    algorithm = "md5",
    input = csv_content
})
local data_hash = nil
if hash_result.success and hash_result.output then
    data_hash = hash_result.output:match('"hash"%s*:%s*"([^"]+)"')
end

local transform_data = {
    source = "workflow",
    timestamp = os.date("!%Y-%m-%dT%H:%M:%SZ"),
    records = #uuid_data,
    data_hash = data_hash or "error"
}

-- Step 5: Format final output
-- Convert Lua table to JSON string for processing
local meta_json = string.format(
    '{"source":"%s","timestamp":"%s","records":%d,"data_hash":"%s"}',
    transform_data.source,
    transform_data.timestamp,
    transform_data.records,
    transform_data.data_hash
)
local final_json = json_tool.execute({
    operation = "query",
    input = meta_json,
    query = "."  -- Pretty print
})
print("\nStep 3 - Final metadata:")
if final_json.success then
    print(final_json.output)
else
    print("Error:", final_json.error)
end

print("\n✅ Data validation workflow complete!")

-- Workflow 5: Cross-Tool Error Handling
show_workflow("Error Handling Workflow", "Demonstrate error propagation and recovery")

-- Test invalid operations
print("Testing error handling across tools:")

-- Invalid hash algorithm
local hash_error = hash_tool.execute({
    operation = "hash",
    algorithm = "invalid-algo",
    input = "test"
})
print("\n1. Invalid hash algorithm:", hash_error.success and "Success" or "Failed")
if hash_error.error then
    print("   Error:", hash_error.error)
end

-- Invalid JSON
local json_error = json_tool.execute({
    operation = "parse",
    input = "{invalid json"
})
print("\n2. Invalid JSON parsing:", json_error.success and "Success" or "Failed")
if json_error.error then
    print("   Error:", json_error.error:match("^[^:]+"))
end

-- Recovery: Use fallback
if not json_error.success then
    local fallback = json_tool.execute({
        operation = "format",
        input = {error = "Invalid input", fallback = true},
        pretty = true
    })
    print("\n3. Fallback recovery:", fallback.success and "Success" or "Failed")
end

print("\n✅ Error handling workflow complete!")

-- Summary
print("\n" .. string.rep("=", 50))
print("📊 Workflow Examples Summary")
print(string.rep("=", 50))
print()
print("Demonstrated workflows:")
print("  1. Data Processing Pipeline - Chain data transformation")
print("  2. File Analysis Workflow - Content generation and analysis")
print("  3. System Monitoring - Environment and system data collection")
print("  4. Data Validation - Generate, validate, and transform data")
print("  5. Error Handling - Graceful error recovery across tools")
print()
print("Key concepts shown:")
print("  - Tool output chaining")
print("  - Error propagation and recovery")
print("  - Data transformation pipelines")
print("  - Cross-tool integration patterns")

-- Return summary for test runner
return {
    status = "success",
    workflows = 5,
    concepts = {
        "output_chaining",
        "error_handling",
        "data_pipelines",
        "tool_integration"
    }
}