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

#### 1. Agent Global
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

#### 2. Tool Global
Access and execute the 33+ built-in tools.

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

#### 3. Workflow Global
Create and execute complex multi-step workflows.

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

#### 4. JSON Global
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

#### 5. State Global (In-Memory)
Temporary state storage within script execution.

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

#### 6. Hook Global (Placeholder)
Lifecycle event registration (full implementation in Phase 4).

```lua
-- Register a hook (placeholder)
Hook.register("agent_complete", function(data)
    print("Agent finished:", data.agent_id)
end)

-- List registered hooks
local hooks = Hook.list()
```

#### 7. Event Global (Placeholder)
Event emission and subscription (full implementation in Phase 4).

```lua
-- Emit an event (placeholder)
Event.emit("custom_event", {data = "test"})

-- Subscribe to events (placeholder)
Event.subscribe("custom_event", function(data)
    print("Event received:", data)
end)
```

### Configuration Globals

#### 8. Logger Global
Structured logging functionality.

```lua
-- Log at different levels
Logger.info("Application started")
Logger.debug("Debug information", {user_id = 123})
Logger.error("An error occurred", {error = "Connection failed"})
```

#### 9. Config Global
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

#### 10. Utils Global
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

## Future Enhancements

### Phase 4 (Hooks & Events)
- Full hook system with 20+ lifecycle points
- Event bus with async subscriptions
- Performance monitoring hooks

### Phase 5 (Persistent State)
- State backed by sled/rocksdb
- Automatic persistence and recovery
- State migrations and versioning

### Phase 15 (JavaScript)
- Full JavaScript global implementations
- Cross-language state sharing
- Unified debugging tools

---

## Examples

See the `/examples` directory for complete examples:
- `global_injection_demo.lua` - Basic usage of all globals
- `agent_workflow_integration.lua` - Advanced multi-agent workflows

---

## API Reference

For detailed API documentation of each global, see:
- [Agent Global API](./api/agent_global.md)
- [Tool Global API](./api/tool_global.md)
- [Workflow Global API](./api/workflow_global.md)
- [JSON Global API](./api/json_global.md)
- [State Global API](./api/state_global.md)