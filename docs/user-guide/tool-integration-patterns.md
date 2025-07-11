# Tool Integration Patterns

This guide covers best practices and patterns for integrating tools in rs-llmspell scripts.

## Table of Contents
- [Basic Tool Usage](#basic-tool-usage)
- [Parameter Handling](#parameter-handling)
- [Response Parsing](#response-parsing)
- [Error Handling](#error-handling)
- [Async Execution](#async-execution)
- [Tool Chaining](#tool-chaining)
- [Advanced Patterns](#advanced-patterns)

## Basic Tool Usage

Tools in rs-llmspell follow a consistent API pattern across all script languages.

### Discovering Available Tools

```lua
-- List all available tools
local tools = Tool.list()
for _, toolName in ipairs(tools) do
    print("Available tool: " .. toolName)
end
```

### Getting a Tool Instance

```lua
-- Get a specific tool
local hashTool = Tool.get("hash_calculator")
assert(hashTool, "Tool not found")

-- Get tool schema/metadata
local schema = hashTool:getSchema()
print("Tool description: " .. schema.description)
```

## Parameter Handling

Tools accept parameters through a structured table/object.

### Simple Parameters

```lua
-- Basic parameter passing
local result = hashTool:execute({
    operation = "hash",
    algorithm = "sha256", 
    data = "Hello, World!"
})
```

### Complex Parameters

```lua
-- Tool with multiple parameter types
local fileResult = fileTool:execute({
    operation = "write",
    path = "/tmp/data.json",
    content = JSON.stringify({
        name = "Test",
        values = {1, 2, 3}
    }),
    mode = "overwrite"
})
```

### Optional Parameters and Defaults

```lua
-- Many parameters have defaults
local result = hashTool:execute({
    operation = "hash",
    data = "test"
    -- algorithm defaults to "sha256"
    -- format defaults to "hex"
})
```

## Response Parsing

Tools return JSON-formatted responses that need to be parsed.

### Standard Response Format

Most tools follow this response structure:

```lua
local result = tool:execute(params)
local response = JSON.parse(result.output)

-- Standard success response
-- {
--   "success": true,
--   "operation": "hash",
--   "result": {
--     "hash": "abc123...",
--     "algorithm": "sha256"
--   }
-- }

-- Standard error response  
-- {
--   "success": false,
--   "operation": "hash",
--   "error": "Missing required parameter: data"
-- }
```

### Direct Response Format

Some tools (like file_operations read) return data directly:

```lua
local readResult = fileTool:execute({
    operation = "read",
    path = "/tmp/data.txt"
})

local response = JSON.parse(readResult.output)
-- Response contains content directly:
-- {
--   "content": "file contents here",
--   "operation": "read",
--   "path": "/tmp/data.txt",
--   "size": 123
-- }
```

## Error Handling

Tools can report errors in two ways:

### 1. Exceptions (for critical errors)

```lua
-- Missing required parameters throw exceptions
local success, err = pcall(function()
    return tool:execute({})  -- Missing required 'operation'
end)

if not success then
    print("Error: " .. tostring(err))
end
```

### 2. Error Responses (for operational failures)

```lua
-- Invalid operations return error responses
local result = tool:execute({
    operation = "invalid_op"
})

local response = JSON.parse(result.output)
if not response.success then
    print("Operation failed: " .. response.error)
end
```

### Robust Error Handling Pattern

```lua
-- Helper function for safe tool execution
function executeTool(tool, params)
    local success, result = pcall(function()
        return tool:execute(params)
    end)
    
    if not success then
        return { success = false, error = tostring(result) }
    end
    
    local ok, response = pcall(function()
        return JSON.parse(result.output)
    end)
    
    if not ok then
        return { success = false, error = "Invalid JSON response" }
    end
    
    return response
end

-- Usage
local response = executeTool(hashTool, {
    operation = "hash",
    data = "test"
})

if response.success then
    print("Hash: " .. response.result.hash)
else
    print("Error: " .. response.error)
end
```

## Async Execution

All tools execute asynchronously through the Lua bridge, but this is transparent to scripts.

### Sequential Execution

```lua
-- Tools appear to execute synchronously from Lua's perspective
local result1 = tool1:execute(params1)  -- Completes before next line
local result2 = tool2:execute(params2)  -- Executes after tool1
```

### Concurrent-Like Patterns

```lua
-- Execute multiple tools rapidly
local results = {}

-- These execute very quickly due to async underlying implementation
for i = 1, 10 do
    results[i] = hashTool:execute({
        operation = "hash",
        data = "test" .. i
    })
end

-- All results are available immediately
for i, result in ipairs(results) do
    local response = JSON.parse(result.output)
    print("Hash " .. i .. ": " .. response.result.hash)
end
```

### Coroutine Integration

```lua
-- Tools work seamlessly with Lua coroutines
function processWithCoroutine()
    local co = coroutine.create(function()
        for i = 1, 3 do
            local result = tool:execute({
                operation = "generate",
                version = "v4"
            })
            coroutine.yield(result)
        end
    end)
    
    local results = {}
    while coroutine.status(co) ~= "dead" do
        local ok, result = coroutine.resume(co)
        if ok and result then
            table.insert(results, result)
        end
    end
    
    return results
end
```

## Tool Chaining

Passing output from one tool as input to another.

### Simple Chain

```lua
-- Hash some data, then encode the hash
local hashResult = hashTool:execute({
    operation = "hash",
    algorithm = "sha256",
    data = "important data"
})

local hashResponse = JSON.parse(hashResult.output)
assert(hashResponse.success, "Hash failed")

local encodeResult = base64Tool:execute({
    operation = "encode",
    input = hashResponse.result.hash
})

local encodeResponse = JSON.parse(encodeResult.output)
print("Encoded hash: " .. encodeResponse.result.output)
```

### Multi-Step Pipeline

```lua
-- Read file -> Process JSON -> Transform -> Write result
function processDataPipeline(inputPath, outputPath)
    -- Step 1: Read file
    local readResult = fileTool:execute({
        operation = "read",
        path = inputPath
    })
    local fileContent = JSON.parse(readResult.output).content
    
    -- Step 2: Process as JSON
    local jsonResult = jsonTool:execute({
        operation = "query",
        input = JSON.parse(fileContent),
        query = ".users[] | select(.active == true)"
    })
    local activeUsers = JSON.parse(jsonResult.output)
    
    -- Step 3: Transform data
    local csvResult = csvTool:execute({
        operation = "create",
        data = activeUsers,
        columns = ["id", "name", "email"]
    })
    local csvContent = JSON.parse(csvResult.output).result.csv
    
    -- Step 4: Write result
    local writeResult = fileTool:execute({
        operation = "write",
        path = outputPath,
        content = csvContent
    })
    
    return JSON.parse(writeResult.output).success
end
```

### Error Propagation in Chains

```lua
-- Chain with proper error handling
function safeChain(operations)
    local lastResult = nil
    
    for i, op in ipairs(operations) do
        local success, result = pcall(function()
            -- Use previous result if specified
            if op.usePrevious and lastResult then
                op.params.input = lastResult
            end
            
            return op.tool:execute(op.params)
        end)
        
        if not success then
            return {
                success = false,
                error = "Step " .. i .. " failed: " .. tostring(result),
                step = i
            }
        end
        
        local response = JSON.parse(result.output)
        if not response.success then
            return {
                success = false,
                error = "Step " .. i .. " error: " .. response.error,
                step = i
            }
        end
        
        lastResult = response.result
    end
    
    return { success = true, result = lastResult }
end

-- Usage
local result = safeChain({
    {
        tool = hashTool,
        params = { operation = "hash", data = "test" }
    },
    {
        tool = base64Tool,
        params = { operation = "encode" },
        usePrevious = true  -- Will use hash from previous step
    }
})
```

## Advanced Patterns

### Tool Factory Pattern

```lua
-- Create specialized tool wrappers
function createSecureFileTool()
    local fileTool = Tool.get("file_operations")
    
    return {
        readSecure = function(path)
            -- Add validation
            if not path:match("^/tmp/") then
                error("Security: Only /tmp files allowed")
            end
            
            return fileTool:execute({
                operation = "read",
                path = path
            })
        end,
        
        writeSecure = function(path, content)
            if not path:match("^/tmp/") then
                error("Security: Only /tmp files allowed")
            end
            
            return fileTool:execute({
                operation = "write",
                path = path,
                content = content,
                mode = "overwrite"
            })
        end
    }
end
```

### Batch Processing Pattern

```lua
-- Process multiple items with consistent error handling
function batchProcess(items, processor)
    local results = {
        success = {},
        failed = {},
        total = #items
    }
    
    for i, item in ipairs(items) do
        local success, result = pcall(processor, item)
        
        if success then
            table.insert(results.success, {
                index = i,
                item = item,
                result = result
            })
        else
            table.insert(results.failed, {
                index = i,
                item = item,
                error = tostring(result)
            })
        end
    end
    
    results.successRate = #results.success / results.total
    return results
end

-- Usage
local files = {"/tmp/a.txt", "/tmp/b.txt", "/tmp/c.txt"}
local results = batchProcess(files, function(path)
    local result = fileTool:execute({
        operation = "read",
        path = path
    })
    return JSON.parse(result.output)
end)

print(string.format("Processed %d files, %.0f%% success rate", 
    results.total, results.successRate * 100))
```

### Retry Pattern

```lua
-- Retry operations with exponential backoff
function retryOperation(operation, maxRetries, initialDelay)
    maxRetries = maxRetries or 3
    initialDelay = initialDelay or 100  -- milliseconds
    
    local lastError = nil
    local delay = initialDelay
    
    for attempt = 1, maxRetries do
        local success, result = pcall(operation)
        
        if success then
            return result
        end
        
        lastError = result
        
        if attempt < maxRetries then
            -- Simple sleep simulation (in production, use proper async waiting)
            local start = os.clock()
            while os.clock() - start < delay / 1000 do
                -- busy wait
            end
            
            delay = delay * 2  -- exponential backoff
        end
    end
    
    error("Operation failed after " .. maxRetries .. " attempts: " .. tostring(lastError))
end

-- Usage
local result = retryOperation(function()
    return httpTool:execute({
        operation = "request",
        method = "GET",
        url = "https://api.example.com/data"
    })
end)
```

### Caching Pattern

```lua
-- Cache tool results for expensive operations
local toolCache = {}

function cachedExecute(tool, params, cacheKey, ttl)
    ttl = ttl or 300  -- 5 minutes default
    
    local now = os.time()
    local cached = toolCache[cacheKey]
    
    if cached and (now - cached.timestamp) < ttl then
        return cached.result
    end
    
    local result = tool:execute(params)
    
    toolCache[cacheKey] = {
        result = result,
        timestamp = now
    }
    
    return result
end

-- Usage
local result = cachedExecute(
    httpTool,
    {
        operation = "request",
        method = "GET", 
        url = "https://api.example.com/expensive-data"
    },
    "expensive-api-call",
    600  -- 10 minute cache
)
```

## Best Practices

1. **Always parse tool output**: Use `JSON.parse()` to handle structured responses
2. **Handle both error types**: Use pcall for exceptions and check response.success
3. **Validate parameters**: Check required fields before execution
4. **Use descriptive variable names**: Make tool chains readable
5. **Cache expensive operations**: Especially for HTTP requests or large file operations
6. **Implement timeouts**: For operations that might hang
7. **Log operations**: For debugging and audit trails
8. **Test error paths**: Ensure graceful degradation

## Performance Considerations

- Tools initialize in <10ms (verified by benchmarks)
- Simple operations complete in <50ms
- File operations are sandboxed and may have overhead
- HTTP operations depend on network latency
- Batch operations when possible to reduce overhead

## Security Notes

- File operations are sandboxed by default
- Path traversal attempts are blocked
- Process execution requires appropriate permissions
- Always validate user input before passing to tools
- Use security levels (safe/restricted/privileged) appropriately

## See Also

- [Getting Started Guide](./getting-started.md)
- [Error Handling Guide](./error-handling.md)
- [Performance Tips](./performance-tips.md)
- [Tool API Reference](/docs/api/tools.md)