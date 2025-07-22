-- test-helpers.lua: Common testing utilities for tool examples
-- Provides helper functions for error handling, result validation, and performance measurement

local TestHelpers = {}

-- ANSI color codes for terminal output
local colors = {
    reset = "\27[0m",
    red = "\27[31m",
    green = "\27[32m",
    yellow = "\27[33m",
    blue = "\27[34m",
    magenta = "\27[35m",
    cyan = "\27[36m",
    white = "\27[37m",
    bold = "\27[1m"
}

-- Disable colors if not in a terminal
local function supports_color()
    local term = os.getenv("TERM")
    return term and term ~= "dumb"
end

if not supports_color() then
    for k, _ in pairs(colors) do
        colors[k] = ""
    end
end

-- Format a test result message
function TestHelpers.format_result(success, test_name, message)
    local status = success and 
        colors.green .. "✓ PASS" .. colors.reset or 
        colors.red .. "✗ FAIL" .. colors.reset
    
    local formatted = string.format("[%s] %s", status, test_name)
    if message then
        formatted = formatted .. ": " .. message
    end
    return formatted
end

-- Print a section header
function TestHelpers.print_section(title)
    print("\n" .. colors.bold .. colors.blue .. "=== " .. title .. " ===" .. colors.reset)
end

-- Print a subsection header
function TestHelpers.print_subsection(title)
    print("\n" .. colors.cyan .. "--- " .. title .. " ---" .. colors.reset)
end

-- Safe tool execution with error handling (using async-aware helper)
function TestHelpers.execute_tool(tool_name, params)
    -- Use the new async-aware helper that handles coroutines properly
    local result = Tool.executeAsync(tool_name, params)
    
    -- Parse the JSON result to get the actual tool response
    if result and result.text then
        -- Try to parse as JSON first
        local success, parsed = pcall(function()
            return JSON.parse(result.text)
        end)
        
        if success and parsed then
            return parsed
        else
            -- If JSON parsing fails, it might be plain text output
            -- Wrap it in a success structure
            return {
                success = true,
                result = result.text,
                output = result.text
            }
        end
    end
    
    -- Return error result if result is nil or has no text
    return {success = false, error = "Tool returned no output"}
end

-- Alternative: Direct tool execution (for testing sync tools)
function TestHelpers.execute_tool_direct(tool_name, params)
    local tool = Tool.get(tool_name)
    if not tool then
        return {
            success = false,
            error = "Tool not found: " .. tool_name
        }
    end
    
    local success, result = pcall(function()
        return tool:execute(params or {})
    end)
    
    if not success then
        return {
            success = false,
            error = tostring(result)
        }
    end
    
    return result
end

-- Measure execution time of a function
function TestHelpers.measure_time(fn, ...)
    local start_time = os.clock()
    local results = {fn(...)}
    local end_time = os.clock()
    local duration = (end_time - start_time) * 1000 -- Convert to milliseconds
    
    return duration, table.unpack(results)
end

-- Run a test with timing and error handling
function TestHelpers.run_test(test_name, test_fn)
    TestHelpers.print_subsection(test_name)
    
    local duration, success, result = TestHelpers.measure_time(pcall, test_fn)
    
    if success then
        print(TestHelpers.format_result(true, test_name, 
            string.format("completed in %.2fms", duration)))
        return true, result
    else
        print(TestHelpers.format_result(false, test_name, tostring(result)))
        if result then
            print(colors.red .. "Error details: " .. tostring(result) .. colors.reset)
        end
        return false, result
    end
end

-- Validate tool output structure
function TestHelpers.validate_output(output, expected_fields)
    if not output then
        return false, "Output is nil"
    end
    
    if type(output) ~= "table" then
        return false, "Output is not a table"
    end
    
    -- Check for error in output
    if output.error then
        return false, "Tool returned error: " .. tostring(output.error)
    end
    
    -- Check for expected fields
    for _, field in ipairs(expected_fields or {}) do
        if output[field] == nil then
            return false, "Missing expected field: " .. field
        end
    end
    
    return true, nil
end

-- Pretty print a table (for debugging)
function TestHelpers.print_table(t, indent)
    indent = indent or 0
    local spacing = string.rep("  ", indent)
    
    if type(t) ~= "table" then
        print(spacing .. tostring(t))
        return
    end
    
    for k, v in pairs(t) do
        if type(v) == "table" then
            print(spacing .. tostring(k) .. ":")
            TestHelpers.print_table(v, indent + 1)
        else
            print(spacing .. tostring(k) .. ": " .. tostring(v))
        end
    end
end

-- Assert helper with descriptive error messages
function TestHelpers.assert(condition, message)
    if not condition then
        error(message or "Assertion failed", 2)
    end
end

-- Check if a tool exists
function TestHelpers.tool_exists(tool_name)
    local tools = Tool.list()
    for _, name in ipairs(tools) do
        if name == tool_name then
            return true
        end
    end
    return false
end

-- Get available tools by category (based on naming patterns)
function TestHelpers.get_tools_by_category()
    local tools = Tool.list()
    local categories = {
        utility = {},
        file_system = {},
        system = {},
        data = {},
        media = {}
    }
    
    for _, tool_name in ipairs(tools) do
        if tool_name:match("base64") or tool_name:match("calculator") or 
           tool_name:match("hash") or tool_name:match("uuid") or 
           tool_name:match("text") or tool_name:match("date") or
           tool_name:match("diff") or tool_name:match("template") or
           tool_name:match("validation") then
            table.insert(categories.utility, tool_name)
        elseif tool_name:match("file") or tool_name:match("archive") or
               tool_name:match("directory") then
            table.insert(categories.file_system, tool_name)
        elseif tool_name:match("environment") or tool_name:match("process") or
               tool_name:match("service") or tool_name:match("system") then
            table.insert(categories.system, tool_name)
        elseif tool_name:match("json") or tool_name:match("csv") or
               tool_name:match("http") or tool_name:match("graphql") then
            table.insert(categories.data, tool_name)
        elseif tool_name:match("audio") or tool_name:match("video") or
               tool_name:match("image") then
            table.insert(categories.media, tool_name)
        end
    end
    
    return categories
end

-- Parse tool output JSON string
function TestHelpers.parse_tool_output(result)
    if not result or not result.success then
        return nil, result and result.error or "Tool execution failed"
    end
    
    if not result.output then
        return nil, "Tool returned no output"
    end
    
    -- Parse JSON string to Lua table
    local success, parsed = pcall(function()
        return JSON.parse(result.output)
    end)
    
    if not success then
        return nil, "Failed to parse JSON output: " .. tostring(parsed)
    end
    
    return parsed, nil
end

-- Helper to safely get nested values from parsed output
function TestHelpers.get_nested_value(table, ...)
    local value = table
    for _, key in ipairs({...}) do
        if type(value) ~= "table" then
            return nil
        end
        value = value[key]
    end
    return value
end

-- Create a test summary
function TestHelpers.create_summary(results)
    local total = 0
    local passed = 0
    local failed = 0
    
    for _, result in ipairs(results) do
        total = total + 1
        if result.success then
            passed = passed + 1
        else
            failed = failed + 1
        end
    end
    
    return {
        total = total,
        passed = passed,
        failed = failed,
        success_rate = total > 0 and (passed / total * 100) or 0
    }
end

-- Print test summary
function TestHelpers.print_summary(summary)
    print("\n" .. colors.bold .. "Test Summary:" .. colors.reset)
    print(string.format("Total tests: %d", summary.total))
    print(string.format(colors.green .. "Passed: %d" .. colors.reset, summary.passed))
    print(string.format(colors.red .. "Failed: %d" .. colors.reset, summary.failed))
    print(string.format("Success rate: %.1f%%", summary.success_rate))
    
    if summary.failed == 0 then
        print("\n" .. colors.green .. colors.bold .. "All tests passed! 🎉" .. colors.reset)
    else
        print("\n" .. colors.red .. colors.bold .. "Some tests failed. Please review the output above." .. colors.reset)
    end
end

-- Performance benchmark helper
function TestHelpers.benchmark(name, fn, iterations)
    iterations = iterations or 100
    local times = {}
    
    print(string.format("\nBenchmarking %s (%d iterations)...", name, iterations))
    
    for i = 1, iterations do
        local duration = TestHelpers.measure_time(fn)
        table.insert(times, duration)
    end
    
    -- Calculate statistics
    table.sort(times)
    local sum = 0
    for _, time in ipairs(times) do
        sum = sum + time
    end
    
    local avg = sum / iterations
    local min = times[1]
    local max = times[#times]
    local median = times[math.floor(#times / 2)]
    
    print(string.format("  Average: %.3fms", avg))
    print(string.format("  Median:  %.3fms", median))
    print(string.format("  Min:     %.3fms", min))
    print(string.format("  Max:     %.3fms", max))
    
    return {
        avg = avg,
        median = median,
        min = min,
        max = max
    }
end

return TestHelpers