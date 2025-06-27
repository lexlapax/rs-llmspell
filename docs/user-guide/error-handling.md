# Error Handling in LLMSpell

This guide covers error handling patterns and best practices when writing LLMSpell scripts.

## Table of Contents

1. [Error Types](#error-types)
2. [Lua Error Handling](#lua-error-handling)
3. [Runtime Errors](#runtime-errors)
4. [Provider Errors](#provider-errors)
5. [Best Practices](#best-practices)

## Error Types

LLMSpell can encounter several types of errors:

### 1. Script Errors
- Syntax errors in Lua code
- Runtime errors (nil access, type mismatches)
- Logic errors in your script

### 2. System Errors
- Memory limit exceeded
- Execution timeout
- Security violations

### 3. Provider Errors
- API key missing or invalid
- Network connectivity issues
- Rate limiting
- Invalid requests

## Lua Error Handling

### Basic Error Handling with pcall

Use `pcall` (protected call) to catch errors:

```lua
-- error-handling.lua
local function risky_operation()
    error("Something went wrong!")
end

-- Call function safely
local success, result = pcall(risky_operation)

if success then
    print("Operation succeeded:", result)
else
    print("Operation failed:", result)
end

-- Return status
return {
    success = success,
    error = not success and result or nil
}
```

### Using xpcall for Better Error Messages

`xpcall` allows custom error handlers:

```lua
-- xpcall-example.lua
local function error_handler(err)
    return {
        error = err,
        traceback = debug.traceback()
    }
end

local function divide(a, b)
    if b == 0 then
        error("Division by zero!")
    end
    return a / b
end

-- Call with error handler
local success, result = xpcall(
    function() return divide(10, 0) end,
    error_handler
)

if not success then
    print("Error occurred:")
    print("  Message:", result.error)
    print("  Traceback:", result.traceback)
end
```

### Assertions

Use assertions for preconditions:

```lua
-- assertions.lua
local function process_data(data)
    assert(type(data) == "table", "Data must be a table")
    assert(data.id, "Data must have an id field")
    assert(data.id > 0, "ID must be positive")
    
    -- Process the data
    return {
        processed = true,
        id = data.id
    }
end

-- This will work
local result1 = process_data({id = 123})

-- This will fail with clear error
-- local result2 = process_data({})  -- Error: Data must have an id field
```

## Runtime Errors

### Memory Limit Errors

When scripts exceed memory limits:

```lua
-- memory-safe.lua
local function create_large_data()
    local data = {}
    
    -- Check periodically in loops
    for i = 1, 1000000 do
        if i % 10000 == 0 then
            -- In real implementation, this would check actual memory
            print(string.format("Processing: %d items", i))
        end
        
        -- Create data carefully
        data[i] = {
            index = i,
            value = math.random()
        }
    end
    
    return data
end

-- Use pcall to catch memory errors
local success, result = pcall(create_large_data)
if not success then
    print("Memory error:", result)
    return { error = "Out of memory" }
end
```

### Timeout Handling

For long-running operations:

```lua
-- timeout-aware.lua
local function long_operation()
    local start_time = os.time()
    local timeout_seconds = 5
    local results = {}
    
    for i = 1, 1000000 do
        -- Check timeout periodically
        if i % 1000 == 0 then
            local elapsed = os.time() - start_time
            if elapsed > timeout_seconds then
                print("Operation timed out")
                break
            end
        end
        
        -- Do work
        results[i] = math.sqrt(i)
    end
    
    return results
end
```

## Provider Errors

### Handling Missing Providers

```lua
-- provider-safety.lua
local function safe_provider_list()
    -- Check if Provider API exists
    if not Provider then
        return {
            error = "Provider API not available",
            providers = {}
        }
    end
    
    -- Safely call Provider.list()
    local success, providers = pcall(Provider.list)
    
    if not success then
        return {
            error = "Failed to list providers: " .. tostring(providers),
            providers = {}
        }
    end
    
    return {
        error = nil,
        providers = providers or {}
    }
end

local result = safe_provider_list()
if result.error then
    print("Error:", result.error)
else
    print("Found", #result.providers, "providers")
end
```

## Best Practices

### 1. Always Validate Input

```lua
local function process_user_data(input)
    -- Validate input type
    if type(input) ~= "table" then
        return nil, "Input must be a table"
    end
    
    -- Validate required fields
    local required = {"name", "email"}
    for _, field in ipairs(required) do
        if not input[field] then
            return nil, "Missing required field: " .. field
        end
    end
    
    -- Process validated input
    return {
        processed = true,
        name = input.name,
        email = input.email
    }
end
```

### 2. Use Meaningful Error Messages

```lua
local function load_config(filename)
    if not filename then
        return nil, "Configuration filename is required"
    end
    
    -- In Phase 1, file access is typically disabled
    -- This is a demonstration
    local config = {}
    
    if not config then
        return nil, string.format(
            "Failed to load configuration from '%s': File not found or access denied",
            filename
        )
    end
    
    return config, nil
end
```

### 3. Create Error Wrappers

```lua
-- error-wrapper.lua
local ErrorHandler = {}

function ErrorHandler.wrap(func, context)
    return function(...)
        local success, result = pcall(func, ...)
        
        if success then
            return result
        else
            return {
                success = false,
                error = result,
                context = context,
                timestamp = os.date()
            }
        end
    end
end

-- Usage
local safe_divide = ErrorHandler.wrap(
    function(a, b) return a / b end,
    "division operation"
)

local result = safe_divide(10, 0)
if result.success == false then
    print("Error in", result.context, ":", result.error)
end
```

### 4. Log Errors Appropriately

```lua
local function log_error(error_info)
    local log_entry = string.format(
        "[%s] ERROR: %s\n  Context: %s\n  Stack: %s",
        os.date("%Y-%m-%d %H:%M:%S"),
        error_info.message or "Unknown error",
        error_info.context or "No context",
        error_info.stack or debug.traceback()
    )
    
    print(log_entry)
    
    -- Return structured error for further processing
    return {
        logged = true,
        timestamp = os.time(),
        error = error_info
    }
end
```

### 5. Graceful Degradation

```lua
-- graceful-degradation.lua
local function get_provider_info()
    local info = {
        available = false,
        providers = {},
        fallback_used = false
    }
    
    -- Try to get provider list
    if Provider and Provider.list then
        local success, providers = pcall(Provider.list)
        if success then
            info.available = true
            info.providers = providers
            return info
        end
    end
    
    -- Fallback behavior
    info.fallback_used = true
    info.message = "Using fallback: Provider API not available"
    
    return info
end
```

## Common Error Patterns

### Pattern 1: Resource Cleanup

```lua
local function with_resource(callback)
    local resource = acquire_resource()
    
    local success, result = pcall(callback, resource)
    
    -- Always cleanup, even on error
    release_resource(resource)
    
    if not success then
        error(result)
    end
    
    return result
end
```

### Pattern 2: Retry Logic

```lua
local function retry_operation(func, max_attempts)
    max_attempts = max_attempts or 3
    local last_error = nil
    
    for attempt = 1, max_attempts do
        local success, result = pcall(func)
        
        if success then
            return result
        end
        
        last_error = result
        print(string.format(
            "Attempt %d/%d failed: %s",
            attempt,
            max_attempts,
            tostring(last_error)
        ))
        
        -- Wait before retry (if not last attempt)
        if attempt < max_attempts then
            -- In real implementation, this would be async sleep
            print("Retrying...")
        end
    end
    
    error("All attempts failed: " .. tostring(last_error))
end
```

## Security Error Handling

Scripts may encounter security restrictions:

```lua
-- security-aware.lua
local function check_file_access()
    -- File operations might be restricted
    local operations = {
        {
            name = "file_read",
            test = function() 
                -- This would attempt file read
                return false, "File access disabled by security policy"
            end
        },
        {
            name = "network_access",
            test = function()
                -- Network might be allowed for providers
                return true, "Network access allowed for providers"
            end
        }
    }
    
    local results = {}
    for _, op in ipairs(operations) do
        local allowed, message = op.test()
        results[op.name] = {
            allowed = allowed,
            message = message
        }
    end
    
    return results
end
```

## Summary

Effective error handling in LLMSpell involves:

1. **Prevention**: Validate inputs and check preconditions
2. **Detection**: Use pcall/xpcall to catch errors
3. **Recovery**: Implement retry logic and fallbacks
4. **Reporting**: Provide clear, actionable error messages
5. **Learning**: Log errors for debugging and improvement

Remember that in Phase 1, certain operations (like file I/O) may be restricted by security policies. Always handle these cases gracefully and provide meaningful feedback to users.