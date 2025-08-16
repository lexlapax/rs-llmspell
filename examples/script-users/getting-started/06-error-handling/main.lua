-- Example: Error Handling - Building Robust Scripts
-- Purpose: Learn how to handle errors gracefully in your scripts
-- Audience: Script Users (Beginners)
-- Prerequisites: Completed 05-state-persistence
-- Expected Output: Error handling demonstration
-- Version: 0.7.0
-- Tags: getting-started, errors, error-handling, robustness, beginner

print("=== Error Handling ===")
print("")

-- Errors are a normal part of programming. Good error handling makes
-- your scripts more reliable and easier to debug.

print("1. Basic Error Handling with pcall")
print("")

-- pcall (protected call) runs a function and catches any errors
local success, result = pcall(function()
    -- This will succeed
    return 10 / 2
end)

if success then
    print("   ✅ Operation succeeded: " .. tostring(result))
else
    print("   ❌ Operation failed: " .. tostring(result))
end

-- Now let's cause an error
local success2, error_msg = pcall(function()
    -- This will fail (division by zero)
    return 10 / 0
end)

if success2 then
    print("   Result: " .. tostring(error_msg))
else
    print("   ⚠️  Caught error: " .. tostring(error_msg))
end

print("")

-- Example 2: Handling Tool Errors
print("2. Handling Tool Invocation Errors")
print("")

-- Try to read a file that doesn't exist
local read_success, read_result = pcall(function()
    return Tool.invoke("file_operations", {
        operation = "read",
        path = "/tmp/nonexistent_file_12345.txt"
    })
end)

if read_success then
    if read_result and read_result.success then
        print("   File contents: " .. (read_result.text or "empty"))
    else
        print("   ⚠️  Tool returned error: " .. (read_result and read_result.error or "Unknown error"))
        print("   This is expected - the file doesn't exist!")
    end
else
    -- Tool.invoke can throw errors for missing files
    print("   ⚠️  Caught error when reading non-existent file")
    print("   This is expected - the file doesn't exist!")
end

print("")

-- Example 3: Fallback Strategies
print("3. Using Fallback Strategies")
print("")

-- Try multiple approaches until one works
local data = nil

-- Approach 1: Try to read from cache
local cache_success, cache_result = pcall(function()
    return Tool.invoke("file_operations", {
        operation = "read",
        path = "/tmp/cache_data.txt"
    })
end)

if cache_success and cache_result and cache_result.text then
    data = cache_result.text
    print("   ✅ Loaded data from cache")
else
    print("   ⚠️  Cache miss, generating new data...")
    
    -- Approach 2: Generate new data
    local uuid_result = Tool.invoke("uuid_generator", {
        operation = "generate",
        version = "v4"
    })
    
    if uuid_result and uuid_result.text then
        data = "Generated data: " .. uuid_result.text
        
        -- Save to cache for next time
        Tool.invoke("file_operations", {
            operation = "write",
            path = "/tmp/cache_data.txt",
            input = data
        })
        
        print("   ✅ Generated new data and cached it")
    else
        -- Approach 3: Use default
        data = "Default data: " .. os.date()
        print("   ⚠️  Using default fallback data")
    end
end

print("   Data: " .. tostring(data))
print("")

-- Example 4: Retry Logic
print("4. Implementing Retry Logic")
print("")

local function retry_operation(func, max_attempts, delay)
    max_attempts = max_attempts or 3
    delay = delay or 1
    
    for attempt = 1, max_attempts do
        local success, result = pcall(func)
        
        if success and result then
            return true, result
        end
        
        print("   Attempt " .. attempt .. " failed")
        
        if attempt < max_attempts then
            print("   Retrying in " .. delay .. " second(s)...")
            -- In real scripts, you might use a sleep function here
            -- For demo, we'll just continue
        end
    end
    
    return false, "All attempts failed"
end

-- Simulate an operation that might fail
local retry_success, retry_result = retry_operation(function()
    -- This simulates a 50% chance of success
    local random_num = math.random()
    if random_num > 0.3 then  -- 70% success rate
        return "Operation succeeded!"
    else
        error("Random failure occurred")
    end
end, 3, 0)

if retry_success then
    print("   ✅ " .. tostring(retry_result))
else
    print("   ❌ " .. tostring(retry_result))
end

print("")

-- Example 5: Validation and Guards
print("5. Input Validation and Guards")
print("")

local function safe_divide(a, b)
    -- Guard against nil values
    if a == nil or b == nil then
        return nil, "Missing parameters"
    end
    
    -- Guard against non-numbers
    if type(a) ~= "number" or type(b) ~= "number" then
        return nil, "Parameters must be numbers"
    end
    
    -- Guard against division by zero
    if b == 0 then
        return nil, "Division by zero"
    end
    
    return a / b, nil
end

-- Test the safe function
local test_cases = {
    {10, 2},      -- Valid
    {10, 0},      -- Division by zero
    {10, nil},    -- Missing parameter
    {"10", 2},    -- Wrong type
}

for i, case in ipairs(test_cases) do
    local result, error = safe_divide(case[1], case[2])
    if error then
        print("   Case " .. i .. ": ⚠️  " .. error)
    else
        print("   Case " .. i .. ": ✅ Result = " .. tostring(result))
    end
end

print("")

-- Example 6: Logging Errors
print("6. Error Logging")
print("")

local error_log = {}

local function log_error(context, error_msg)
    table.insert(error_log, {
        timestamp = os.date("%Y-%m-%d %H:%M:%S"),
        context = context,
        error = error_msg
    })
end

-- Simulate some operations with error logging
local operations = {
    {name = "database_connect", will_fail = false},
    {name = "fetch_data", will_fail = true},
    {name = "process_data", will_fail = false},
    {name = "save_results", will_fail = true},
}

for _, op in ipairs(operations) do
    if op.will_fail then
        log_error(op.name, "Simulated failure in " .. op.name)
        print("   ❌ " .. op.name .. " failed (logged)")
    else
        print("   ✅ " .. op.name .. " succeeded")
    end
end

print("")
print("   Error Log Summary:")
for _, entry in ipairs(error_log) do
    print("   [" .. entry.timestamp .. "] " .. entry.context .. ": " .. entry.error)
end

print("")

-- Example 7: Cleanup on Error
print("7. Cleanup on Error (finally pattern)")
print("")

local function operation_with_cleanup()
    local temp_file = "/tmp/temp_working_file.txt"
    
    -- Create a temporary file
    Tool.invoke("file_operations", {
        operation = "write",
        path = temp_file,
        input = "Temporary working data"
    })
    print("   Created temporary file")
    
    local success, result = pcall(function()
        -- Do some work that might fail
        local random = math.random()
        if random < 0.5 then
            error("Simulated processing error")
        end
        return "Processing completed"
    end)
    
    -- Cleanup - always runs (wrapped in pcall for safety)
    local cleanup_success, cleanup_result = pcall(function()
        return Tool.invoke("file_operations", {
            operation = "delete",
            path = temp_file
        })
    end)
    
    if cleanup_success and cleanup_result and cleanup_result.success then
        print("   ✅ Cleaned up temporary file")
    else
        -- File might not exist if creation failed
        print("   ⚠️  Cleanup skipped (file may not exist)")
    end
    
    if success then
        print("   ✅ Operation result: " .. tostring(result))
    else
        print("   ❌ Operation failed: " .. tostring(result))
    end
end

operation_with_cleanup()

print("")
print("🎉 Congratulations! You've successfully:")
print("   - Learned to use pcall for protected function calls")
print("   - Handled tool invocation errors")
print("   - Implemented fallback strategies")
print("   - Created retry logic for unreliable operations")
print("   - Added input validation and guards")
print("   - Logged errors for debugging")
print("   - Ensured cleanup with finally-like patterns")
print("")
print("💡 Key Concepts:")
print("   - Always expect and handle errors gracefully")
print("   - Use pcall to catch errors without crashing")
print("   - Provide fallbacks for critical operations")
print("   - Validate inputs before processing")
print("   - Clean up resources even when errors occur")
print("   - Log errors for debugging and monitoring")
print("")
print("🚀 You've completed the Getting Started series!")
print("   You now have the foundation to build robust LLMSpell scripts.")
print("   Explore the advanced examples to learn more!")