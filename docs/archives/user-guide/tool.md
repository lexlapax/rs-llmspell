# Tool Module

The Tool module provides access to the extensive tool ecosystem for various operations.

## Core Functions

### Tool.execute(name, params)
Invokes a tool with the given parameters.

**Parameters:**
- `name` (string): Tool name
- `params` (table): Tool-specific parameters

**Returns:**
- `result` (table): Tool execution result
  - `success` (boolean): Whether execution succeeded
  - `result` (any): Tool-specific result data
  - `error` (string, optional): Error message if failed

**Example:**
```lua
local result = Tool.execute("text-manipulator", {
    operation = "uppercase",
    text = "hello world"
})
-- result.result = "HELLO WORLD"
```

### Tool.list()
Lists all available tools.

**Returns:** `table` - Array of tool descriptors
- `name` (string): Tool name
- `category` (string): Tool category
- `description` (string): Tool description

**Example:**
```lua
local tools = Tool.list()
for i, tool in ipairs(tools) do
    print(tool.name, "-", tool.description)
end
```

### Tool.exists(name)
Checks if a tool exists.

**Parameters:**
- `name` (string): Tool name

**Returns:** `boolean` - Whether tool exists

### Tool.info(name)
Gets detailed information about a tool.

**Parameters:**
- `name` (string): Tool name

**Returns:** `table` - Tool information
- `name` (string): Tool name
- `category` (string): Category
- `description` (string): Description
- `parameters` (table): Parameter schema
- `examples` (table): Usage examples

## Tool Categories

### Text Manipulation
```lua
-- Text Manipulator
Tool.execute("text-manipulator", {
    operation = "replace",
    text = "Hello World",
    pattern = "World",
    replacement = "Lua"
})

-- JSON Processor
Tool.execute("json-processor", {
    operation = "parse",
    input = '{"key": "value"}'
})

-- XML Parser
Tool.execute("xml-parser", {
    operation = "parse",
    input = "<root><item>value</item></root>"
})
```

### File System
```lua
-- File Reader
Tool.execute("file-reader", {
    operation = "read",
    path = "/path/to/file.txt"
})

-- File Writer
Tool.execute("file-writer", {
    operation = "write",
    path = "/path/to/output.txt",
    content = "Hello, World!"
})

-- Directory Scanner
Tool.execute("directory-scanner", {
    operation = "scan",
    path = "/path/to/directory",
    recursive = true
})
```

### Web Tools
```lua
-- Web Search
Tool.execute("web-search", {
    query = "LLMSpell documentation",
    provider = "duckduckgo",
    limit = 10
})

-- Web Scraper
Tool.execute("web-scraper", {
    url = "https://example.com",
    selector = "h1"
})

-- HTTP Request
Tool.execute("http-request", {
    method = "GET",
    url = "https://api.example.com/data",
    headers = {["Authorization"] = "Bearer token"}
})
```

### Utility Tools
```lua
-- Calculator
Tool.execute("calculator", {
    expression = "2 + 2 * 3"
})

-- Hash Calculator
Tool.execute("hash-calculator", {
    operation = "calculate",
    algorithm = "sha256",
    input = "data to hash"
})

-- DateTime Handler
Tool.execute("datetime-handler", {
    operation = "format",
    format = "%Y-%m-%d %H:%M:%S"
})
```

### Data Tools
```lua
-- CSV Processor
Tool.execute("csv-processor", {
    operation = "parse",
    input = "name,age\nJohn,30\nJane,25"
})

-- Database Connector
Tool.execute("database-connector", {
    operation = "query",
    connection = "sqlite://data.db",
    query = "SELECT * FROM users"
})

-- Data Generator
Tool.execute("data-generator", {
    type = "user",
    count = 10
})
```

### Media Tools
```lua
-- Image Processor
Tool.execute("image-processor", {
    operation = "info",
    input = "/path/to/image.jpg"
})

-- PDF Processor
Tool.execute("pdf-processor", {
    operation = "extract_text",
    input = "/path/to/document.pdf"
})

-- Audio Analyzer
Tool.execute("audio-analyzer", {
    operation = "get_duration",
    input = "/path/to/audio.mp3"
})
```

### Communication Tools
```lua
-- Email Sender
Tool.execute("email-sender", {
    to = "user@example.com",
    subject = "Test Email",
    body = "Hello from LLMSpell!"
})

-- Webhook Caller
Tool.execute("webhook-caller", {
    url = "https://hooks.example.com/webhook",
    method = "POST",
    payload = {event = "test"}
})

-- API Tester
Tool.execute("api-tester", {
    endpoint = "https://api.example.com/test",
    method = "GET"
})
```

## Async Operations

### Tool.invoke_async(name, params)
Invokes a tool asynchronously.

**Parameters:** Same as `Tool.execute()`

**Returns:** `Promise` - Promise that resolves to result

**Example:**
```lua
Tool.invoke_async("web-search", {query = "async search"})
    :then(function(result)
        print("Found:", #result.result, "results")
    end)
```

### Tool.batch_invoke(operations)
Invokes multiple tools in parallel.

**Parameters:**
- `operations` (table): Array of operation descriptors
  - `name` (string): Tool name
  - `params` (table): Tool parameters

**Returns:** `table` - Array of results in same order

**Example:**
```lua
local results = Tool.batch_invoke({
    {name = "calculator", params = {expression = "2+2"}},
    {name = "datetime-handler", params = {operation = "now"}},
    {name = "random-generator", params = {type = "uuid"}}
})
```

## Tool Registration

### Tool.register(name, implementation)
Registers a custom tool.

**Parameters:**
- `name` (string): Tool name
- `implementation` (function): Tool implementation

**Returns:** `boolean` - Success status

**Example:**
```lua
Tool.register("custom-tool", function(params)
    -- Tool implementation
    return {
        success = true,
        result = "Processed: " .. params.input
    }
end)
```

### Tool.unregister(name)
Unregisters a tool.

**Parameters:**
- `name` (string): Tool name

**Returns:** `boolean` - Success status

## Tool Composition

### Tool.pipeline(steps)
Creates a tool pipeline where output flows to next input.

**Parameters:**
- `steps` (table): Array of pipeline steps

**Returns:** `table` - Final result

**Example:**
```lua
local result = Tool.pipeline({
    {name = "file-reader", params = {path = "data.json"}},
    {name = "json-processor", params = {operation = "parse"}},
    {name = "data-transformer", params = {transform = "normalize"}}
})
```

### Tool.parallel(operations)
Runs tools in parallel and aggregates results.

**Parameters:**
- `operations` (table): Array of operations

**Returns:** `table` - Aggregated results

## Error Handling

Tools return structured error information:

```lua
local result = Tool.execute("file-reader", {
    path = "/nonexistent/file.txt"
})

if not result.success then
    print("Error:", result.error)
    -- Handle error
end
```

## Tool Metrics

### Tool.get_metrics(name)
Gets usage metrics for a tool.

**Parameters:**
- `name` (string): Tool name

**Returns:** `table` - Metrics data
- `invocations` (number): Total invocations
- `failures` (number): Total failures
- `average_duration` (number): Average execution time (ms)

## See Also
- [Agent Module](./agent.md) - Agent integration with tools
- [Workflow Module](./workflow.md) - Tool orchestration in workflows
- [Hook Module](./hook.md) - Tool execution hooks