# Agent API Reference

## Overview

✅ **Phase 3.3 Implementation Status**: Agent creation, execution, and registry are fully functional. Advanced agent features (tool integration, multi-agent coordination) are in active development.

The Agent API provides a synchronous interface for creating and interacting with LLM agents in LLMSpell. As of version 0.3.0, the API has been simplified to remove the need for Lua coroutines, making agent creation straightforward and predictable.

## Creating Agents

### Basic Usage

```lua
-- Create an agent with minimal configuration
local agent = Agent.create({
    model = "openai/gpt-4",                    -- Required: provider/model format
    system_prompt = "You are a helpful assistant"  -- Note: system_prompt, not prompt
})

-- Create an agent with Anthropic
local agent = Agent.create({
    model = "anthropic/claude-3-sonnet",
    system_prompt = "You are a code review expert"
})
```

### Advanced Configuration

```lua
local agent = Agent.create({
    name = "data_analyst",                        -- Optional: auto-generated if omitted
    model = "openai/gpt-4",                      -- Required: provider/model format  
    description = "Data analysis expert",         -- Optional: agent description
    system_prompt = "You are a data analysis expert",  -- Note: system_prompt, not prompt
    temperature = 0.7,                           -- Optional: 0.0-2.0
    max_tokens = 2000,                           -- Optional: response token limit
    max_conversation_length = 100,               -- Optional: conversation history limit
    base_url = "https://custom-endpoint.com/v1", -- Optional: override provider endpoint
    api_key = "custom-api-key"                   -- Optional: override default API key
})
```

## Executing Agents

```lua
-- Simple execution (requires table input)
local response = agent:execute({
    prompt = "What is 2 + 2?"
})
print(response)

-- Execution with options
local response = agent:execute({
    prompt = "Analyze this data",
    temperature = 0.3,  -- Override agent's default temperature
    max_tokens = 500    -- Limit response length for this execution
})
```

## Synchronous API Design

### Why Synchronous?

The Agent API is synchronous to provide:
- **Simplicity**: No need to manage coroutines or async/await patterns
- **Predictability**: Direct execution without yielding complexities
- **Compatibility**: Works seamlessly with Lua's execution model
- **Performance**: <10ms overhead for agent operations

### Implementation Details

Internally, the synchronous API uses Rust's `tokio::runtime::Handle::block_on()` to handle asynchronous operations transparently. This means:
- Agent creation is immediate
- No coroutine wrapping required
- No "attempt to yield from outside a coroutine" errors
- Clean stack traces for debugging

### Migration from Async API

If you have code using the old async API:

```lua
-- OLD (deprecated in v0.3.0)
local agent = coroutine.wrap(function()
    return Agent.createAsync({
        model = "gpt-4",
        prompt = "Hello"
    })
end)()

-- NEW (v0.3.0+)
local agent = Agent.create({
    model = "gpt-4", 
    prompt = "Hello"
})
```

## Agent Registry

Agents can be registered for global access:

```lua
-- Register an agent
Agent.register("assistant", {
    model = "gpt-4o-mini",
    prompt = "You are a helpful assistant"
})

-- Get a registered agent
local agent = Agent.get("assistant")
if agent then
    local response = agent:execute("Hello!")
end

-- List all registered agents
local agents = Agent.list()
for _, name in ipairs(agents) do
    print("Registered agent:", name)
end
```

## Tool Integration

Agents can use tools for enhanced capabilities:

```lua
local agent = Agent.create({
    model = "gpt-4",
    prompt = "You are a research assistant",
    tools = {
        "web_search",
        "calculator",
        "file_reader"
    }
})

-- Agent can now use these tools in responses
local response = agent:execute("Search for recent AI developments and calculate the growth rate")
```

## Error Handling

```lua
-- Wrap agent creation in pcall for error handling
local success, agent_or_error = pcall(function()
    return Agent.create({
        model = "invalid/model",
        prompt = "Test"
    })
end)

if not success then
    print("Error creating agent:", agent_or_error)
    return
end

-- Handle execution errors
local success, response = pcall(function()
    return agent:execute("Process this request")
end)

if not success then
    print("Execution error:", response)
end
```

## Performance Characteristics

- **Agent Creation**: <50ms including provider initialization
- **Tool Invocation**: <10ms overhead per tool call
- **Memory Usage**: Efficient with automatic cleanup
- **Concurrency**: Thread-safe for multiple agents

## Best Practices

1. **Reuse Agents**: Create agents once and reuse for multiple executions
2. **Error Handling**: Always wrap agent operations in pcall
3. **Resource Management**: Agents are automatically cleaned up by Lua's GC
4. **Provider Configuration**: Set API keys via environment variables
5. **Tool Selection**: Only include tools the agent needs

## Future Roadmap

While the current API is synchronous, future versions may introduce:
- Callback-based async execution for streaming
- Promise/Future patterns for concurrent operations
- Event-driven agent interactions

These will be additive features that maintain backward compatibility with the synchronous API.

## API Reference

### Agent.create(config)
Creates a new agent instance.

**Parameters:**
- `config` (table): Agent configuration
  - `model` (string): Model identifier or "provider/model" syntax
  - `prompt` (string): System prompt for the agent
  - `temperature` (number, optional): Sampling temperature (0.0-2.0)
  - `max_tokens` (number, optional): Maximum response tokens
  - `tools` (array, optional): Tool names to enable
  - `base_url` (string, optional): Override provider endpoint
  - `api_key` (string, optional): Override API key

**Returns:**
- Agent instance or throws error

### agent:execute(input, options)
Execute the agent with given input.

**Parameters:**
- `input` (string): User input/query
- `options` (table, optional): Execution options

**Returns:**
- Response string or throws error

### Agent.register(name, config)
Register an agent globally.

**Parameters:**
- `name` (string): Agent identifier
- `config` (table): Agent configuration

### Agent.get(name)
Get a registered agent.

**Parameters:**
- `name` (string): Agent identifier

**Returns:**
- Agent instance or nil

### Agent.list()
List all registered agents.

**Returns:**
- Array of agent names