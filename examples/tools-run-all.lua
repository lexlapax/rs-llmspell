-- tools-run-all.lua: Execute all tool examples and generate a summary report
-- This script runs all tool example files and collects results

-- Load test helpers
local TestHelpers = dofile("examples/test-helpers.lua")

-- Auto-discover all tools-*.lua files in the examples directory
local function discover_tool_examples()
    local files = {}
    local handle = io.popen("ls examples/tools-*.lua 2>/dev/null")
    if handle then
        for file in handle:lines() do
            -- Skip tools-run-all.lua itself
            if not file:match("tools%-run%-all%.lua$") then
                table.insert(files, file)
            end
        end
        handle:close()
    end
    
    -- Sort files for consistent ordering
    table.sort(files)
    
    -- Put reference file first if it exists
    local ref_index = nil
    for i, file in ipairs(files) do
        if file:match("tools%-utility%-reference%.lua$") then
            ref_index = i
            break
        end
    end
    
    if ref_index then
        local ref_file = table.remove(files, ref_index)
        table.insert(files, 1, ref_file)
    end
    
    return files
end

-- List of example files to run (auto-discovered)
local example_files = discover_tool_examples()

if #example_files == 0 then
    print("ERROR: No tools-*.lua example files found!")
    os.exit(1)
end

print("\nDiscovered " .. #example_files .. " example files:")
for i, file in ipairs(example_files) do
    print(string.format("  %2d. %s", i, file))
end

-- Results storage
local results = {}
local start_time = os.clock()
local total_operations = 0
local failed_operations = 0

-- Main header
print(TestHelpers.format_result(true, "LLMSpell Tool Examples Test Suite", "Starting..."))
print("Date: " .. os.date("%Y-%m-%d %H:%M:%S"))
print("Available tools: " .. #Tool.list())

-- List available tools by category
TestHelpers.print_section("Available Tools by Category")
local categories = TestHelpers.get_tools_by_category()
for category, tools in pairs(categories) do
    if #tools > 0 then
        print(string.format("\n%s (%d):", category:gsub("_", " "):gsub("^%l", string.upper), #tools))
        for _, tool in ipairs(tools) do
            print("  - " .. tool)
        end
    end
end

-- Run each example file
TestHelpers.print_section("Running Examples")

for _, filename in ipairs(example_files) do
    TestHelpers.print_subsection("Running " .. filename)
    
    local file_start = os.clock()
    
    -- Capture stdout to analyze test results
    local original_print = print
    local captured_output = {}
    local operation_count = 0
    local failure_count = 0
    local error_details = {}
    
    -- Override print to capture output
    print = function(...)
        local args = {...}
        local line = table.concat(args, "\t")
        table.insert(captured_output, line)
        
        -- Count operations and failures
        if line:match("✅") then
            operation_count = operation_count + 1
        elseif line:match("❌") then
            failure_count = failure_count + 1
            -- Capture error details
            local error_msg = line:match("❌%s*(.+)")
            if error_msg then
                table.insert(error_details, error_msg)
            end
        end
        
        -- Still print to console
        original_print(...)
    end
    
    local success, error_msg = pcall(function()
        dofile(filename)
    end)
    
    -- Restore original print
    print = original_print
    
    local file_duration = (os.clock() - file_start) * 1000
    
    local result = {
        file = filename,
        success = success,
        duration = file_duration,
        error = not success and error_msg or nil,
        operations = operation_count,
        failures = failure_count,
        error_details = error_details,
        output = captured_output
    }
    
    table.insert(results, result)
    total_operations = total_operations + operation_count
    failed_operations = failed_operations + failure_count
    
    if success then
        if failure_count > 0 then
            print(TestHelpers.format_result(false, filename, 
                string.format("completed with %d failures in %.2fms", failure_count, file_duration)))
            -- Show first few error details
            for i = 1, math.min(3, #error_details) do
                print("  " .. error_details[i])
            end
            if #error_details > 3 then
                print("  ... and " .. (#error_details - 3) .. " more errors")
            end
        else
            print(TestHelpers.format_result(true, filename, 
                string.format("completed in %.2fms (%d operations)", file_duration, operation_count)))
        end
    else
        print(TestHelpers.format_result(false, filename, "failed to execute"))
        if error_msg then
            print("Error: " .. tostring(error_msg))
        end
    end
end

-- Calculate total duration
local total_duration = (os.clock() - start_time) * 1000

-- Generate summary
TestHelpers.print_section("Summary Report")

local passed_count = 0
local failed_count = 0
local failed_files = {}

for _, result in ipairs(results) do
    if result.success then
        passed_count = passed_count + 1
    else
        failed_count = failed_count + 1
        table.insert(failed_files, result.file)
    end
end

print(string.format("\nTotal examples run: %d", #results))
print(string.format("Passed: %d", passed_count))
print(string.format("Failed: %d", failed_count))
print(string.format("Total duration: %.2fms", total_duration))
print(string.format("Success rate: %.1f%%", 
    #results > 0 and (passed_count / #results * 100) or 0))

print(string.format("\nTotal operations tested: %d", total_operations))
print(string.format("Failed operations: %d", failed_operations))
if total_operations > 0 then
    print(string.format("Operation success rate: %.1f%%", 
        (total_operations - failed_operations) / total_operations * 100))
end

if #failed_files > 0 then
    print("\nFailed examples:")
    for _, file in ipairs(failed_files) do
        print("  - " .. file)
    end
end

-- Show examples with operation failures
local examples_with_failures = {}
for _, result in ipairs(results) do
    if result.success and result.failures > 0 then
        table.insert(examples_with_failures, result)
    end
end

if #examples_with_failures > 0 then
    print("\nExamples with failed operations:")
    for _, result in ipairs(examples_with_failures) do
        print(string.format("  - %s: %d failures", result.file, result.failures))
        for i = 1, math.min(2, #result.error_details) do
            print("      " .. result.error_details[i])
        end
    end
end

-- Performance summary
TestHelpers.print_section("Performance Summary")

local fastest = nil
local slowest = nil

for _, result in ipairs(results) do
    if result.success then
        if not fastest or result.duration < fastest.duration then
            fastest = result
        end
        if not slowest or result.duration > slowest.duration then
            slowest = result
        end
    end
end

if fastest then
    print(string.format("Fastest: %s (%.2fms)", fastest.file, fastest.duration))
end
if slowest then
    print(string.format("Slowest: %s (%.2fms)", slowest.file, slowest.duration))
end

-- Tool coverage analysis
TestHelpers.print_section("Tool Coverage Analysis")

-- This will be populated as we fix the examples
local tools_demonstrated = {}
local all_tools = Tool.list()

print(string.format("Total tools available: %d", #all_tools))
print(string.format("Tools demonstrated: %d", #tools_demonstrated))
print(string.format("Coverage: %.1f%%", 
    #all_tools > 0 and (#tools_demonstrated / #all_tools * 100) or 0))

-- Final status
print("\n" .. string.rep("=", 50))
if failed_count == 0 and failed_operations == 0 then
    print(TestHelpers.format_result(true, "TEST SUITE", "All examples and operations passed!"))
elseif failed_count > 0 then
    print(TestHelpers.format_result(false, "TEST SUITE", 
        string.format("%d examples failed", failed_count)))
elseif failed_operations > 0 then
    print(TestHelpers.format_result(false, "TEST SUITE", 
        string.format("All examples ran but %d operations failed", failed_operations)))
end
print(string.rep("=", 50))

-- Exit with appropriate code (fail if any operations failed)
os.exit((failed_count > 0 or failed_operations > 0) and 1 or 0)