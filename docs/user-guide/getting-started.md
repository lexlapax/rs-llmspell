# Global Object Injection System Guide

**Version**: 1.0  
**Date**: July 2025  
**Status**: Production Ready  

> **📋 Complete Guide**: This document explains the global object injection system that provides all rs-llmspell functionality through pre-injected globals without require() statements.

---

## Overview

The global object injection system is a core feature of rs-llmspell that provides seamless access to all functionality through pre-injected global objects. This eliminates the need for require() statements and provides a consistent API across different scripting languages.

### Key Benefits

- **Zero Configuration**: All globals are automatically available in scripts
- **Language Agnostic**: Same API across Lua, JavaScript (Phase 15), and Python (future)
- **Performance Optimized**: <5ms injection time with caching
- **Type Safe**: Comprehensive type conversion between script and native types
- **Dependency Managed**: Automatic resolution of inter-global dependencies

---

## Available Globals

### Core Functionality Globals

#### 1. Agent Global ✅ **Fully Available**
Provides access to LLM agent creation and execution.

```lua
-- Create an agent
local agent = Agent.create({
    name = "assistant",
    provider = "openai",
    model = "gpt-4",
    system_prompt = "You are a helpful assistant."
})

-- Execute the agent
local response = agent:execute({
    prompt = "Explain quantum computing"
})

-- List all agents
local agents = Agent.list()
```

#### 2. Tool Global ✅ **Fully Available**
Access and execute the 34 built-in tools.

```lua
-- Get a specific tool
local calc = Tool.get("calculator")

-- Execute the tool
local result = calc:execute({
    operation = "add",
    a = 10,
    b = 20
})

-- List all tools
local tools = Tool.list()

-- Get tool categories
local categories = Tool.categories()
```

#### 3. Workflow Global 🚧 **In Development**
Create and execute complex multi-step workflows. Basic patterns available, advanced features in development.

```lua
-- Sequential workflow
local workflow = Workflow.sequential({
    name = "data_pipeline",
    steps = {
        {name = "fetch", tool = "web_scraper", parameters = {...}},
        {name = "process", tool = "data_processor", parameters = {...}},
        {name = "store", tool = "database", parameters = {...}}
    }
})

-- Execute workflow
local result = Workflow.execute(workflow)

-- Other workflow types
local conditional = Workflow.conditional({...})
local parallel = Workflow.parallel({...})
local loop = Workflow.loop({...})
```

### Utility Globals

#### 4. JSON Global ✅ **Fully Available**
Parse and stringify JSON data.

```lua
-- Parse JSON string
local data = JSON.parse('{"name": "test", "value": 42}')

-- Stringify object
local json_str = JSON.stringify({
    message = "hello",
    timestamp = os.date()
})
```

#### 5. State Global ✅ **Fully Available** (In-Memory Only)
Temporary state storage within script execution. Persistent storage coming in Phase 5.

```lua
-- Set state
State.set("user_id", 12345)
State.set("config", {theme = "dark", lang = "en"})

-- Get state
local user_id = State.get("user_id")
local config = State.get("config")

-- List all keys
local keys = State.list()

-- Delete state
State.delete("temp_data")
```

**Note**: Currently in-memory only. Persistent storage coming in Phase 5.

#### 6. Hook Global 📋 **Phase 4 Feature**
*Lifecycle event registration will be available in Phase 4. Use State global for current coordination needs.*

```lua
-- ❌ NOT YET AVAILABLE - Phase 4 feature
-- Hook.register("agent_complete", function(data)
--     print("Agent finished:", data.agent_id)
-- end)

-- ✅ CURRENT WORKAROUND - Use State for coordination
State.set("agent_completion_callbacks", {})
local callbacks = State.get("agent_completion_callbacks") or {}
```

#### 7. Event Global 📋 **Phase 4 Feature**
*Event emission and subscription will be available in Phase 4. Use State global for current event coordination.*

```lua
-- ❌ NOT YET AVAILABLE - Phase 4 feature
-- Event.emit("custom_event", {data = "test"})
-- Event.subscribe("custom_event", function(data)
--     print("Event received:", data)
-- end)

-- ✅ CURRENT WORKAROUND - Use State for event-like coordination
State.set("custom_events", {})
local events = State.get("custom_events") or {}
table.insert(events, {event = "custom_event", data = "test"})
State.set("custom_events", events)
```

### Configuration Globals

#### 8. Logger Global ✅ **Fully Available**
Structured logging functionality.

```lua
-- Log at different levels
Logger.info("Application started")
Logger.debug("Debug information", {user_id = 123})
Logger.error("An error occurred", {error = "Connection failed"})
```

#### 9. Config Global ✅ **Fully Available**
Access configuration values.

```lua
-- Get config values
local api_key = Config.get("api_key")
local timeout = Config.get("timeout", 30)  -- with default

-- Check if config exists
if Config.has("feature_flag") then
    -- Feature is enabled
end
```

#### 10. Utils Global ✅ **Fully Available**
Common utility functions.

```lua
-- Generate UUID
local id = Utils.uuid()

-- Get timestamp
local now = Utils.timestamp()

-- Hash data
local hash = Utils.hash("my data")
```

---

## Architecture

### Language-Agnostic Design

The global injection system uses a three-layer architecture:

1. **Global Object Layer**: Language-agnostic trait definitions
2. **Language Binding Layer**: Language-specific implementations
3. **Injection Layer**: Efficient injection with dependency resolution

```
┌─────────────────────────────────────┐
│         Script (Lua/JS/Python)      │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│      Global Objects (Injected)      │
│  Agent │ Tool │ Workflow │ JSON │   │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│    Language Bindings (Lua/JS/Py)   │
└─────────────────┬───────────────────┘
                  │
┌─────────────────▼───────────────────┐
│    Native Rust Implementation      │
└─────────────────────────────────────┘
```

### Dependency Resolution

Globals can depend on each other. The system automatically resolves dependencies using topological sorting:

```rust
GlobalMetadata {
    name: "Agent",
    dependencies: vec!["Logger", "Config"],
    // ...
}
```

### Type Conversion

Automatic bidirectional type conversion between script and native types:

- **Primitives**: bool, number, string, nil/null
- **Collections**: arrays, objects/tables
- **Complex Types**: AgentInput, ToolParameters, WorkflowConfig
- **Binary Data**: Base64 encoded for media content

---

## Performance Characteristics

- **Injection Time**: <5ms for all globals
- **Memory Overhead**: ~2MB per script context
- **Caching**: LRU cache for frequently used conversions
- **Lazy Loading**: Globals initialized only when accessed

---

## Usage Patterns

### 1. Simple Tool Execution
```lua
local result = Tool.get("calculator"):execute({
    operation = "multiply",
    a = 7,
    b = 8
})
```

### 2. Agent with State
```lua
-- Store agent config
State.set("agent_config", {
    temperature = 0.7,
    max_tokens = 150
})

-- Create agent with stored config
local config = State.get("agent_config")
local agent = Agent.create({
    name = "smart_agent",
    provider = "anthropic",
    model = "claude-3",
    temperature = config.temperature
})
```

### 3. Workflow with Dynamic Steps
```lua
local steps = {}

-- Build steps dynamically
for i = 1, 5 do
    table.insert(steps, {
        name = "step_" .. i,
        tool = "logger",
        parameters = {message = "Processing item " .. i}
    })
end

local workflow = Workflow.sequential({
    name = "dynamic_workflow",
    steps = steps
})
```

### 4. Error Handling
```lua
-- Safe tool execution
local success, result = pcall(function()
    return Tool.get("web_scraper"):execute({url = "https://example.com"})
end)

if success then
    State.set("last_scrape", result)
else
    Logger.error("Scraping failed", {error = result})
end
```

---

## Migration Guide

### From require() to Globals

Before (old style):
```lua
local agent = require("llmspell.agent")
local tool = require("llmspell.tool")

local my_agent = agent.create({...})
local my_tool = tool.get("calculator")
```

After (with globals):
```lua
-- No requires needed!
local my_agent = Agent.create({...})
local my_tool = Tool.get("calculator")
```

### Platform-Specific Code

The globals work identically across platforms, but you can check:

```lua
if _VERSION then
    -- Lua specific code
elseif typeof ~= nil then
    -- JavaScript specific code
end
```

---

## Best Practices

1. **Check Tool Existence**
   ```lua
   local tool = Tool.get("my_tool")
   if tool then
       -- Tool exists, safe to use
   end
   ```

2. **State Namespacing**
   ```lua
   -- Use prefixes to avoid collisions
   State.set("myapp:user_id", 123)
   State.set("myapp:session", {...})
   ```

3. **Error Propagation**
   ```lua
   local function safe_execute(tool_name, params)
       local tool = Tool.get(tool_name)
       if not tool then
           return nil, "Tool not found: " .. tool_name
       end
       return pcall(tool.execute, tool, params)
   end
   ```

4. **Workflow Composition**
   ```lua
   -- Store reusable workflows
   State.set("workflows:data_pipeline", pipeline_workflow)
   State.set("workflows:report_gen", report_workflow)
   
   -- Compose them later
   local master = Workflow.sequential({
       name = "master",
       steps = {
           {workflow = State.get("workflows:data_pipeline")},
           {workflow = State.get("workflows:report_gen")}
       }
   })
   ```

---

## Troubleshooting

### Global Not Available
- Ensure script engine was properly initialized
- Check that llmspell runtime has inject_apis() called
- Verify feature flags (lua/javascript) are enabled

### Type Conversion Errors
- Use JSON.stringify() to inspect complex objects
- Ensure parameters match tool schemas
- Check for nil/null values in required fields

### Performance Issues
- Minimize State.get() calls in loops
- Cache frequently accessed tools
- Use workflow batching for multiple operations

---

## What's Coming Next

### 🔮 Phase 4: Hook and Event System (Weeks 17-18)
Transform the current workarounds into a robust system:
- **Hook System**: 20+ lifecycle hooks for agents, workflows, and tools
- **Event Bus**: Async event emission and subscription 
- **Integration**: Replace State-based coordination with proper events
- **Monitoring**: Built-in performance and error tracking hooks

### 🗄️ Phase 5: Persistent State Management (Weeks 19-20)  
Upgrade the current in-memory State global:
- **Persistent Backend**: State backed by sled/rocksdb
- **Automatic Recovery**: Scripts resume with saved state
- **Migrations**: Seamless state schema upgrades
- **Cross-Session**: State survives script restarts

### 🌐 Phase 15: JavaScript Engine Support (Weeks 39-40)
Expand beyond Lua:
- **JavaScript Global APIs**: Same interface, different language
- **Cross-Language State**: Shared state between Lua and JS scripts
- **Unified Tooling**: Debug and monitor both engines together

*For complete phase details, see `/docs/in-progress/implementation-phases.md`*

---

## Error Handling Best Practices

### Using pcall for Safe Execution

Always use `pcall` (protected call) to catch errors when executing tools, agents, or workflows:

```lua
-- Safe tool execution
local success, result = pcall(function()
    return Tool.get("web_scraper"):execute({url = "https://example.com"})
end)

if success then
    State.set("last_scrape", result)
    Logger.info("Scraping successful", {url = "https://example.com"})
else
    Logger.error("Scraping failed", {error = result})
end

-- Safe agent execution
local success, response = pcall(function()
    return agent:execute({prompt = "Analyze this data"})
end)

if not success then
    Logger.error("Agent execution failed", {error = response})
    return {error = "Processing failed: " .. tostring(response)}
end
```

### Error Handling Patterns

#### 1. Input Validation
```lua
local function process_with_agent(agent_name, prompt)
    -- Validate inputs
    if type(agent_name) ~= "string" then
        return nil, "Agent name must be a string"
    end
    
    if not prompt or type(prompt) ~= "string" then
        return nil, "Prompt is required and must be a string"
    end
    
    -- Get agent safely
    local agent = Agent.get(agent_name)
    if not agent then
        return nil, "Agent not found: " .. agent_name
    end
    
    -- Execute with error handling
    local success, result = pcall(agent.execute, agent, {prompt = prompt})
    if not success then
        return nil, "Execution failed: " .. tostring(result)
    end
    
    return result, nil
end
```

#### 2. Retry Logic
```lua
local function retry_operation(func, max_attempts, delay_ms)
    max_attempts = max_attempts or 3
    delay_ms = delay_ms or 1000
    local last_error = nil
    
    for attempt = 1, max_attempts do
        local success, result = pcall(func)
        
        if success then
            return result
        end
        
        last_error = result
        Logger.warn(string.format(
            "Attempt %d/%d failed: %s",
            attempt,
            max_attempts,
            tostring(last_error)
        ))
        
        -- Wait before retry (if not last attempt)
        if attempt < max_attempts then
            Utils.sleep(delay_ms / 1000)  -- Convert to seconds
        end
    end
    
    error("All attempts failed: " .. tostring(last_error))
end

-- Usage
local result = retry_operation(function()
    return Tool.get("api_tester"):execute({
        url = "https://api.example.com/data",
        method = "GET"
    })
end, 3, 2000)
```

#### 3. Error Aggregation in Workflows
```lua
local function validate_data(data)
    local errors = {}
    
    -- Run multiple validations
    local validators = {
        {name = "format", tool = "json_validator"},
        {name = "schema", tool = "schema_validator"},
        {name = "business", tool = "business_rules"}
    }
    
    for _, validator in ipairs(validators) do
        local tool = Tool.get(validator.tool)
        if tool then
            local success, result = pcall(tool.execute, tool, {input = data})
            if not success then
                table.insert(errors, {
                    validator = validator.name,
                    error = tostring(result)
                })
            end
        end
    end
    
    if #errors > 0 then
        return false, errors
    end
    
    return true, nil
end
```

### Common Error Types and Handling

#### Resource Limit Errors
```lua
-- Handle memory or timeout errors gracefully
local success, result = pcall(function()
    return Tool.get("data_processor"):execute({
        operation = "transform",
        data = large_dataset
    })
end)

if not success and string.find(tostring(result), "memory") then
    Logger.error("Memory limit exceeded", {
        dataset_size = #large_dataset,
        error = result
    })
    -- Try with smaller batches
    return process_in_batches(large_dataset, 1000)
end
```

#### Missing Dependencies
```lua
-- Check for required tools/agents before workflow
local required_tools = {"file_reader", "data_processor", "file_writer"}
local missing = {}

for _, tool_name in ipairs(required_tools) do
    if not Tool.get(tool_name) then
        table.insert(missing, tool_name)
    end
end

if #missing > 0 then
    error("Missing required tools: " .. table.concat(missing, ", "))
end
```

### Graceful Degradation
```lua
local function get_data_with_fallback(primary_source, fallback_source)
    -- Try primary source
    local primary_tool = Tool.get(primary_source)
    if primary_tool then
        local success, data = pcall(primary_tool.execute, primary_tool, {})
        if success then
            return data
        end
        Logger.warn("Primary source failed, trying fallback", {
            source = primary_source,
            error = data
        })
    end
    
    -- Try fallback
    local fallback_tool = Tool.get(fallback_source)
    if fallback_tool then
        local success, data = pcall(fallback_tool.execute, fallback_tool, {})
        if success then
            State.set("used_fallback", true)
            return data
        end
    end
    
    error("Both primary and fallback sources failed")
end
```

### Error Logging and Monitoring
```lua
local function log_error(error_info)
    -- Structure error for monitoring
    local error_data = {
        timestamp = Utils.timestamp(),
        error = tostring(error_info.error),
        context = error_info.context or "unknown",
        user_id = State.get("user_id"),
        session_id = State.get("session_id")
    }
    
    -- Log to appropriate level
    if error_info.severity == "critical" then
        Logger.error("Critical error occurred", error_data)
        -- Could emit event for monitoring (when available)
        -- Event.emit("critical_error", error_data)
    else
        Logger.warn("Error occurred", error_data)
    end
    
    -- Store for later analysis
    local errors = State.get("errors") or {}
    table.insert(errors, error_data)
    State.set("errors", errors)
    
    return error_data
end
```

---

## Examples

See the `/examples` directory for complete examples:
- `global_injection_demo.lua` - Basic usage of all globals
- `agent_workflow_integration.lua` - Advanced multi-agent workflows
- `error_handling_patterns.lua` - Error handling best practices

---

## API Reference

For detailed API documentation of each global, see:
- [Agent Global API](./api/agent_global.md)
- [Tool Global API](./api/tool_global.md)
- [Workflow Global API](./api/workflow_global.md)
- [JSON Global API](./api/json_global.md)
- [State Global API](./api/state_global.md)