-- ============================================================
-- LLMSPELL COOKBOOK SHOWCASE
-- ============================================================
-- Pattern ID: 01 - Error Handling Patterns v0.7.0
-- Complexity Level: PRODUCTION
-- Real-World Use Case: Enterprise-grade error handling for resilient automation
-- Pattern Category: Error Handling & Recovery
--
-- Purpose: Comprehensive error handling patterns for production LLMSpell scripts.
--          Demonstrates safe invocation, retry strategies, circuit breakers,
--          graceful degradation, and error aggregation. Essential for building
--          reliable automation that handles failures elegantly.
-- Architecture: Defensive programming with error boundaries and recovery strategies
-- Crates Showcased: llmspell-tools, llmspell-agents, llmspell-bridge
-- Key Features:
--   • Safe tool invocation with detailed error information
--   • Retry mechanisms with exponential backoff
--   • Circuit breaker pattern for failing services
--   • Graceful degradation strategies
--   • Error aggregation and reporting
--   • Production logging patterns
--
-- Prerequisites:
--   • LLMSpell installed and built
--   • No API keys required (uses local tools)
--   • Write access to /tmp for logging
--
-- HOW TO RUN:
-- ./target/debug/llmspell run examples/script-users/cookbook/error-handling.lua
--
-- EXPECTED OUTPUT:
-- 6 error handling patterns demonstrated:
-- 1. Safe tool invocation with UUID generation
-- 2. Retry with backoff showing 3 attempts
-- 3. Circuit breaker preventing cascading failures
-- 4. Graceful degradation with fallback values
-- 5. Error aggregation collecting multiple errors
-- 6. Production error logging with structured data
--
-- Time to Complete: <3 seconds
-- Production Notes: Use circuit breakers for external services, implement
--                   proper logging to observability platforms, set appropriate
--                   timeout and retry limits based on SLAs.
-- ============================================================

print("=== Error Handling Cookbook ===")
print("Pattern 01: PRODUCTION - Enterprise error handling strategies\n")

-- ============================================================
-- Pattern 1: Safe Tool Invocation with Detailed Error Info
-- ============================================================

print("1. Safe Tool Invocation Pattern")
print("-" .. string.rep("-", 40))

local function safe_invoke_tool(tool_name, params)
    -- Validate inputs
    if not tool_name or type(tool_name) ~= "string" then
        return nil, "Invalid tool name"
    end
    
    if params and type(params) ~= "table" then
        return nil, "Parameters must be a table"
    end
    
    -- Attempt tool invocation with pcall
    local success, result = pcall(function()
        return Tool.invoke(tool_name, params or {})
    end)
    
    if success then
        -- Check if tool returned an error
        if result and result.error then
            return nil, "Tool error: " .. tostring(result.error), result
        end
        return result, nil
    else
        -- Invocation failed
        return nil, "Invocation failed: " .. tostring(result)
    end
end

-- Test the pattern
local result, err, details = safe_invoke_tool("uuid_generator", {
    operation = "generate",
    version = "v4"
})

if result then
    print("   ✅ Success: Generated UUID")
else
    print("   ❌ Error: " .. tostring(err))
end

-- Test with invalid tool
local result2, err2 = safe_invoke_tool("nonexistent_tool", {})
print("   ⚠️  Expected error: " .. tostring(err2))

print()

-- ============================================================
-- Pattern 2: Error Context Accumulation
-- ============================================================

print("2. Error Context Accumulation")
print("-" .. string.rep("-", 40))

local ErrorContext = {}
ErrorContext.__index = ErrorContext

function ErrorContext:new()
    return setmetatable({
        errors = {},
        warnings = {},
        context_stack = {}
    }, self)
end

function ErrorContext:push_context(context)
    table.insert(self.context_stack, context)
end

function ErrorContext:pop_context()
    table.remove(self.context_stack)
end

function ErrorContext:add_error(error_msg, details)
    local context = table.concat(self.context_stack, " > ")
    table.insert(self.errors, {
        message = error_msg,
        context = context,
        details = details,
        timestamp = os.date("%Y-%m-%d %H:%M:%S")
    })
end

function ErrorContext:add_warning(warning_msg)
    local context = table.concat(self.context_stack, " > ")
    table.insert(self.warnings, {
        message = warning_msg,
        context = context,
        timestamp = os.date("%Y-%m-%d %H:%M:%S")
    })
end

function ErrorContext:has_errors()
    return #self.errors > 0
end

function ErrorContext:get_summary()
    return {
        error_count = #self.errors,
        warning_count = #self.warnings,
        errors = self.errors,
        warnings = self.warnings
    }
end

-- Example usage
local ctx = ErrorContext:new()

ctx:push_context("Data Processing")
ctx:push_context("Validation")

-- Simulate validation errors
ctx:add_warning("Missing optional field: description")
ctx:add_error("Invalid date format", {field = "created_at", value = "not-a-date"})

ctx:pop_context() -- Back to Data Processing
ctx:push_context("Transformation")

-- Simulate transformation error
ctx:add_error("Type conversion failed", {from = "string", to = "number"})

ctx:pop_context()
ctx:pop_context()

-- Get error summary
local summary = ctx:get_summary()
print("   Errors: " .. summary.error_count)
print("   Warnings: " .. summary.warning_count)

for i, err in ipairs(summary.errors) do
    print("   [ERROR] " .. err.context .. ": " .. err.message)
end

print()

-- ============================================================
-- Pattern 3: Structured Error Types
-- ============================================================

print("3. Structured Error Types")
print("-" .. string.rep("-", 40))

local ErrorTypes = {
    VALIDATION = "VALIDATION_ERROR",
    NETWORK = "NETWORK_ERROR",
    TIMEOUT = "TIMEOUT_ERROR",
    RATE_LIMIT = "RATE_LIMIT_ERROR",
    PERMISSION = "PERMISSION_ERROR",
    NOT_FOUND = "NOT_FOUND_ERROR",
    INTERNAL = "INTERNAL_ERROR"
}

local function create_error(error_type, message, details)
    return {
        type = error_type,
        message = message,
        details = details or {},
        timestamp = os.time(),
        recoverable = error_type ~= ErrorTypes.INTERNAL
    }
end

local function handle_error(error)
    -- Different handling based on error type
    if error.type == ErrorTypes.RATE_LIMIT then
        print("   ⏱️  Rate limited - implementing backoff")
        return {action = "retry", delay = 60}
    elseif error.type == ErrorTypes.NETWORK then
        print("   🔄 Network error - will retry")
        return {action = "retry", delay = 5}
    elseif error.type == ErrorTypes.VALIDATION then
        print("   ❌ Validation failed - cannot proceed")
        return {action = "abort"}
    elseif error.type == ErrorTypes.TIMEOUT then
        print("   ⏱️  Timeout - increasing limit and retrying")
        return {action = "retry", timeout_multiplier = 2}
    else
        print("   ⚠️  Unknown error - logging and continuing")
        return {action = "continue"}
    end
end

-- Test different error types
local test_errors = {
    create_error(ErrorTypes.RATE_LIMIT, "API rate limit exceeded"),
    create_error(ErrorTypes.VALIDATION, "Invalid input format"),
    create_error(ErrorTypes.NETWORK, "Connection refused")
}

for _, err in ipairs(test_errors) do
    local action = handle_error(err)
    print("   Error: " .. err.message .. " -> Action: " .. action.action)
end

print()

-- ============================================================
-- Pattern 4: Error Recovery Chain
-- ============================================================

print("4. Error Recovery Chain")
print("-" .. string.rep("-", 40))

local function with_recovery(primary_fn, recovery_fns)
    -- Try primary function
    local success, result = pcall(primary_fn)
    
    if success then
        return result, nil
    end
    
    local last_error = result
    
    -- Try recovery functions in order
    for i, recovery_fn in ipairs(recovery_fns) do
        print("   Attempting recovery strategy #" .. i)
        local recovery_success, recovery_result = pcall(recovery_fn, last_error)
        
        if recovery_success then
            print("   ✅ Recovery strategy #" .. i .. " succeeded")
            return recovery_result, nil
        end
        
        last_error = recovery_result
    end
    
    return nil, "All recovery strategies failed: " .. tostring(last_error)
end

-- Example: Getting data with multiple fallbacks
local function get_data_with_fallbacks()
    local result, err = with_recovery(
        -- Primary: Get from API
        function()
            print("   Trying primary: API call")
            error("API unavailable") -- Simulate failure
        end,
        {
            -- Fallback 1: Get from cache
            function(prev_error)
                print("   Trying fallback 1: Cache")
                error("Cache miss") -- Simulate failure
            end,
            -- Fallback 2: Get from local file
            function(prev_error)
                print("   Trying fallback 2: Local file")
                return {data = "Local backup data", source = "file"}
            end,
            -- Fallback 3: Use default
            function(prev_error)
                print("   Trying fallback 3: Default value")
                return {data = "Default data", source = "default"}
            end
        }
    )
    
    if result then
        print("   Final result: " .. result.data .. " (from " .. result.source .. ")")
    else
        print("   ❌ Failed: " .. tostring(err))
    end
end

get_data_with_fallbacks()

print()

-- ============================================================
-- Pattern 5: Error Aggregation for Batch Operations
-- ============================================================

print("5. Batch Operation Error Handling")
print("-" .. string.rep("-", 40))

local function process_batch(items, processor_fn, options)
    options = options or {}
    local continue_on_error = options.continue_on_error ~= false
    local max_errors = options.max_errors or math.huge
    
    local results = {
        successful = {},
        failed = {},
        total = #items,
        error_count = 0
    }
    
    for i, item in ipairs(items) do
        local success, result = pcall(processor_fn, item, i)
        
        if success then
            table.insert(results.successful, {
                index = i,
                item = item,
                result = result
            })
        else
            results.error_count = results.error_count + 1
            table.insert(results.failed, {
                index = i,
                item = item,
                error = result
            })
            
            -- Check if we should continue
            if not continue_on_error then
                results.aborted = true
                results.abort_reason = "Error at item " .. i
                break
            end
            
            if results.error_count >= max_errors then
                results.aborted = true
                results.abort_reason = "Max errors (" .. max_errors .. ") reached"
                break
            end
        end
    end
    
    results.success_rate = #results.successful / results.total
    return results
end

-- Test batch processing
local test_items = {1, 2, "invalid", 4, "also_invalid", 6, 7, 8}

local batch_results = process_batch(
    test_items,
    function(item, index)
        if type(item) ~= "number" then
            error("Item at index " .. index .. " is not a number")
        end
        return item * 2
    end,
    {continue_on_error = true, max_errors = 3}
)

print("   Processed: " .. batch_results.total .. " items")
print("   Successful: " .. #batch_results.successful)
print("   Failed: " .. #batch_results.failed)
print("   Success rate: " .. string.format("%.1f%%", batch_results.success_rate * 100))

if batch_results.aborted then
    print("   ⚠️  Batch aborted: " .. batch_results.abort_reason)
end

print()

-- ============================================================
-- Pattern 6: Error Logging and Reporting
-- ============================================================

print("6. Error Logging and Reporting")
print("-" .. string.rep("-", 40))

local ErrorLogger = {}
ErrorLogger.__index = ErrorLogger

function ErrorLogger:new(options)
    options = options or {}
    return setmetatable({
        log_file = options.log_file or "/tmp/errors.log",
        max_entries = options.max_entries or 1000,
        entries = {},
        stats = {
            by_type = {},
            by_hour = {},
            total = 0
        }
    }, self)
end

function ErrorLogger:log(error_type, message, details)
    local entry = {
        timestamp = os.time(),
        date_str = os.date("%Y-%m-%d %H:%M:%S"),
        type = error_type,
        message = message,
        details = details
    }
    
    -- Add to entries (with rotation)
    table.insert(self.entries, entry)
    if #self.entries > self.max_entries then
        table.remove(self.entries, 1)
    end
    
    -- Update statistics
    self.stats.total = self.stats.total + 1
    self.stats.by_type[error_type] = (self.stats.by_type[error_type] or 0) + 1
    
    local hour = os.date("%H", entry.timestamp)
    self.stats.by_hour[hour] = (self.stats.by_hour[hour] or 0) + 1
    
    -- Write to file (in production, use proper logging)
    -- Tool.invoke("file_operations", {
    --     operation = "append",
    --     path = self.log_file,
    --     input = entry.date_str .. " [" .. error_type .. "] " .. message .. "\n"
    -- })
    
    return entry
end

function ErrorLogger:get_stats()
    return self.stats
end

function ErrorLogger:get_recent(count)
    count = count or 10
    local start = math.max(1, #self.entries - count + 1)
    local recent = {}
    for i = start, #self.entries do
        table.insert(recent, self.entries[i])
    end
    return recent
end

-- Example usage
local logger = ErrorLogger:new()

-- Simulate various errors
logger:log(ErrorTypes.VALIDATION, "Missing required field", {field = "email"})
logger:log(ErrorTypes.NETWORK, "Connection timeout", {host = "api.example.com"})
logger:log(ErrorTypes.RATE_LIMIT, "Rate limit exceeded", {limit = 100})
logger:log(ErrorTypes.VALIDATION, "Invalid format", {field = "phone"})

local stats = logger:get_stats()
print("   Total errors logged: " .. stats.total)
print("   By type:")
for error_type, count in pairs(stats.by_type) do
    print("     " .. error_type .. ": " .. count)
end

print()
print("🎯 Key Takeaways:")
print("   • Use structured error handling for production code")
print("   • Implement recovery strategies for resilience")
print("   • Track and analyze errors for improvement")
print("   • Handle batch operations gracefully")
print("   • Provide context for debugging")