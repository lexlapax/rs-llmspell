-- tools-run-all.lua: Execute all tool examples and generate a summary report
-- This script runs all tool example files and collects results

-- Load test helpers (assuming it's in the same directory)
local TestHelpers = dofile("test-helpers.lua")

-- List of example files to run (in order)
local example_files = {
    -- Working reference
    "tools-utility-reference.lua",
    
    -- Core examples
    "tools-showcase.lua",
    "tools-utility.lua", 
    "tools-filesystem.lua",
    "tools-system.lua",
    
    -- Category examples
    "tools-data.lua",
    "tools-media.lua",
    "tools-security.lua",
    -- "tools-workflow.lua",
    -- "tools-performance.lua"
}

-- Results storage
local results = {}
local start_time = os.clock()

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
    local success, error_msg = pcall(function()
        dofile(filename)
    end)
    local file_duration = (os.clock() - file_start) * 1000
    
    local result = {
        file = filename,
        success = success,
        duration = file_duration,
        error = not success and error_msg or nil
    }
    
    table.insert(results, result)
    
    if success then
        print(TestHelpers.format_result(true, filename, 
            string.format("completed in %.2fms", file_duration)))
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

if #failed_files > 0 then
    print("\nFailed examples:")
    for _, file in ipairs(failed_files) do
        print("  - " .. file)
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
if failed_count == 0 then
    print(TestHelpers.format_result(true, "TEST SUITE", "All examples passed!"))
else
    print(TestHelpers.format_result(false, "TEST SUITE", 
        string.format("%d examples failed", failed_count)))
end
print(string.rep("=", 50))

-- Exit with appropriate code
os.exit(failed_count > 0 and 1 or 0)