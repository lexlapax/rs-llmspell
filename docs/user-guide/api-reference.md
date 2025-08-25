# API Quick Reference

**📋 Implementation Status**: Current Phase 7 - Production Ready

> **📚 Complete Documentation Available**: For comprehensive API documentation designed for both humans and LLM coding assistants, see:
> - **[Lua API Reference](api/lua/README.md)** - Complete Lua scripting API with all globals, methods, and examples
> - **[Rust API Reference](api/rust/README.md)** - Complete Rust extension API with traits, builders, and patterns

This is a quick reference for all rs-llmspell APIs. All globals are pre-injected - no require() needed!

## Agent Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Agent.create(config)` | `config`: {model, provider, temperature, max_tokens, ...} | Agent instance | Creates a new agent |
| `agent:execute(input, options)` | `input`: string, `options`: table (optional) | string | Execute agent with input |
| `agent:complete(prompt)` | `prompt`: string | string | Get completion for prompt |
| `Agent.register(name, config)` | `name`: string, `config`: table | nil | Register agent globally |
| `Agent.get(name)` | `name`: string | Agent or nil | Get registered agent |
| `Agent.list()` | none | {string} | List registered agents |

### Common Agent Config
```lua
{
    model = "gpt-4",              -- Required
    provider = "openai",          -- Optional (inferred from model)
    temperature = 0.7,            -- Optional (0.0-2.0)
    max_tokens = 1000,            -- Optional
    system_prompt = "...",        -- Optional
    tools = {"tool1", "tool2"}    -- Optional tool names
}
```

## Workflow Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Workflow.sequential(config)` | See below | Workflow | Create step-by-step workflow |
| `Workflow.conditional(config)` | See below | Workflow | Create branching workflow |
| `Workflow.loop(config)` | See below | Workflow | Create iterating workflow |
| `Workflow.parallel(config)` | See below | Workflow | Create concurrent workflow |
| `workflow:execute(input)` | `input`: table (optional) | table | Execute workflow |
| `workflow:validate()` | none | {valid, errors} | Validate configuration |
| `workflow:getInfo()` | none | {id, name, type} | Get workflow metadata |
| `workflow:getMetrics()` | none | {duration, steps, ...} | Get execution metrics |
| `workflow:setState(key, value)` | `key`: string, `value`: any | nil | Set workflow state |
| `workflow:getState(key)` | `key`: string | any | Get workflow state |
| `Workflow.list()` | none | {info} | List all workflows |
| `Workflow.get(id)` | `id`: string | Workflow or nil | Get workflow by ID |

### Workflow Configs

**Sequential**
```lua
{
    name = "my_workflow",
    steps = {
        {name = "step1", tool = "tool_name", input = {...}},
        {name = "step2", agent = agent, prompt = "..."},
        {name = "step3", tool = "tool2", input = "$step1.output"}
    }
}
```

**Conditional**
```lua
{
    name = "conditional_flow",
    branches = {
        {
            condition = function(input, state) return input.value > 10 end,
            workflow = success_workflow
        },
        {
            condition = function(input, state) return true end,
            workflow = default_workflow
        }
    }
}
```

**Loop**
```lua
{
    name = "loop_flow",
    body = workflow_to_repeat,
    condition = function(input, state, iteration)
        return iteration < 5 and state.continue
    end,
    max_iterations = 10
}
```

**Parallel**
```lua
{
    name = "parallel_flow",
    branches = {
        {name = "branch1", workflow = workflow1},
        {name = "branch2", workflow = workflow2}
    },
    max_concurrency = 5,
    join_strategy = "all"  -- or "any", "none"
}
```

## State Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `State.set(key, value)` | `key`: string, `value`: any | nil | Store value |
| `State.get(key)` | `key`: string | any or nil | Retrieve value |
| `State.delete(key)` | `key`: string | nil | Delete value |
| `State.list()` | none | {string} | List all keys |

## Tool Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Tool.get(name)` | `name`: string | Tool or nil | Get specific tool |
| `tool:execute(params)` | `params`: table | table | Execute tool |
| `Tool.list()` | none | {string} | List all tools |
| `Tool.categories()` | none | {string} | Get tool categories |

### Common Tool Usage
```lua
local file_tool = Tool.get("file_operations")
local result = file_tool:execute({
    operation = "read",
    path = "/path/to/file.txt"
})
```

## JSON Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `JSON.parse(string)` | `string`: JSON string | table | Parse JSON to table |
| `JSON.stringify(object)` | `object`: table | string | Convert table to JSON |

## Logger Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Logger.info(msg, data)` | `msg`: string, `data`: table (opt) | nil | Log info |
| `Logger.debug(msg, data)` | `msg`: string, `data`: table (opt) | nil | Log debug |
| `Logger.warn(msg, data)` | `msg`: string, `data`: table (opt) | nil | Log warning |
| `Logger.error(msg, data)` | `msg`: string, `data`: table (opt) | nil | Log error |

## Config Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Config.get(key, default)` | `key`: string, `default`: any (opt) | any | Get config value |
| `Config.has(key)` | `key`: string | boolean | Check if key exists |

## Utils Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Utils.uuid()` | none | string | Generate UUID |
| `Utils.timestamp()` | none | number | Current timestamp |
| `Utils.hash(data)` | `data`: string | string | Hash data |
| `Utils.sleep(seconds)` | `seconds`: number | nil | Sleep (blocks) |

## Hook Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Hook.register(point, handler, priority)` | `point`: string, `handler`: function, `priority`: string/number (opt) | HookHandle | Register a hook |
| `Hook.unregister(handle)` | `handle`: HookHandle | boolean | Unregister a hook |
| `Hook.list(filter)` | `filter`: string/table (optional) | {hook_info} | List registered hooks |
| `Hook.enable_builtin(name, config)` | `name`: string/table, `config`: table (opt) | nil | Enable built-in hooks |
| `Hook.disable_builtin(name)` | `name`: string/table | nil | Disable built-in hooks |

### Hook Registration
```lua
-- Register with priority
local handle = Hook.register("BeforeAgentExecution", function(context)
    print("Agent executing:", context.component_id.name)
    return "continue"  -- or {action = "modified", ...}
end, "high")  -- Priority: highest, high, normal, low, lowest

-- Unregister
Hook.unregister(handle)
-- or
handle:unregister()
```

### Hook Context
```lua
context = {
    hook_point = "BeforeAgentExecution",
    component_id = {id = "...", name = "...", component_type = "..."},
    correlation_id = "...",
    data = {input = {...}, output = {...}, ...},
    metadata = {...},
    language = "lua",
    state = {}
}
```

### Hook Results
```lua
-- Continue execution
return "continue"

-- Modify data
return {
    action = "modified",
    modified_data = {input = {text = "modified text"}}
}

-- Cancel execution
return {
    action = "cancel",
    reason = "Validation failed"
}

-- Retry with backoff
return {
    action = "retry",
    max_attempts = 3,
    backoff_ms = 1000
}
```

## Session Global 🚧

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Session.create(options)` | `options`: {name, description, tags, metadata, parent_session_id} | string (session_id) | Create new session |
| `Session.get(session_id)` | `session_id`: string | table (metadata) | Get session metadata |
| `Session.suspend(session_id)` | `session_id`: string | nil | Suspend active session |
| `Session.resume(session_id)` | `session_id`: string | nil | Resume suspended session |
| `Session.complete(session_id)` | `session_id`: string | nil | Complete session (removes from active) |
| `Session.delete(session_id)` | `session_id`: string | nil | Delete session entirely |
| `Session.list(filters)` | `filters`: {status, tags, limit, sort_by} (optional) | {sessions} | List sessions |
| `Session.save(session_id)` | `session_id`: string | nil | Save to persistent storage |
| `Session.load(session_id)` | `session_id`: string | string (session_id) | Load from storage |
| `Session.getCurrent()` | none | string or nil | Get current thread-local session |
| `Session.setCurrent(session_id)` | `session_id`: string or nil | nil | Set current thread-local session |

### Session Config Example
```lua
{
    name = "Customer Support",          -- Required
    description = "Support ticket #123", -- Optional
    tags = {"support", "priority-high"}, -- Optional
    metadata = {custom = "data"},        -- Optional custom data
    parent_session_id = "parent-123"     -- Optional parent session
}
```

## Artifact Global 🚧

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Artifact.store(session_id, type, name, content, metadata)` | See below | {content_hash, session_id, sequence} | Store artifact |
| `Artifact.get(session_id, artifact_id)` | `session_id`: string, `artifact_id`: table | {content, metadata} | Retrieve artifact |
| `Artifact.list(session_id)` | `session_id`: string or "" (uses current) | {artifacts} | List session artifacts |
| `Artifact.delete(session_id, artifact_id)` | `session_id`: string, `artifact_id`: table | nil | Delete artifact |
| `Artifact.storeFile(session_id, path, type, metadata)` | See below | {content_hash, session_id, sequence} | Store file as artifact |

### Artifact Parameters
- `session_id`: Session to store in
- `type`: "tool_result", "agent_output", "user_input", "system_generated", or custom
- `name`: Artifact name (e.g., "analysis.txt")
- `content`: String content (can be binary)
- `metadata`: Optional table with:
  - `mime_type`: Content MIME type
  - `tags`: Array of tags
  - Custom fields

### Artifact Usage
```lua
-- Store text artifact
local id = Artifact.store(
    session_id,
    "tool_result",
    "output.txt",
    "Results here...",
    {mime_type = "text/plain", tags = {"important"}}
)

-- Store binary data
local image_data = io.open("chart.png", "rb"):read("*a")
local img_id = Artifact.store(
    session_id,
    "agent_output",
    "chart.png",
    image_data,
    {mime_type = "image/png"}
)

-- Retrieve artifact
local artifact = Artifact.get(session_id, id)
print(artifact.content)
print(artifact.metadata.name)
```

## Event Global

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Event.publish(event_type, data, options)` | `event_type`: string, `data`: table, `options`: table (opt) | nil | Publish an event |
| `Event.subscribe(pattern)` | `pattern`: string/table | Subscription | Subscribe to event pattern(s) |
| `Event.receive(subscription, timeout_ms)` | `subscription`: Subscription, `timeout_ms`: number | event or nil | Receive next event |
| `Event.receive_batch(sub, options)` | `sub`: Subscription, `options`: table | {events} | Receive multiple events |
| `Event.unsubscribe(subscription)` | `subscription`: Subscription | nil | Unsubscribe |
| `Event.list_subscriptions()` | none | {sub_info} | List active subscriptions |
| `Event.subscription_stats(sub)` | `sub`: Subscription | {stats} | Get subscription statistics |

### Event Publishing
```lua
-- Simple publish
Event.publish("user.action.completed", {
    action = "search",
    results = 10
})

-- With options
Event.publish("system.alert", {
    level = "warning",
    message = "High CPU usage"
}, {
    language = "lua",
    correlation_id = "req-123",
    ttl_seconds = 3600
})
```

### Event Subscription
```lua
-- Subscribe to pattern
local sub = Event.subscribe("user.*")  -- Wildcards supported

-- Receive events
local event = Event.receive(sub, 1000)  -- 1 second timeout
if event then
    print("Event:", event.event_type)
    print("Data:", event.data)
end

-- Clean up
Event.unsubscribe(sub)
```

### Event Format
```lua
event = {
    id = "uuid",
    event_type = "user.action.completed",
    timestamp = "2025-07-25T10:30:45.123Z",
    version = "1.0",
    source = {
        component = "script",
        instance_id = "...",
        language = "lua"
    },
    data = {...},
    metadata = {...}
}
```

## Error Handling

Always use `pcall` for safe execution:
```lua
local success, result = pcall(function()
    return agent:complete("Hello")
end)

if success then
    print("Result: " .. result)
else
    Logger.error("Failed", {error = result})
end
```

## Variable References in Workflows

Use `$` prefix to reference previous step outputs:
- `$stepName` - Full output of step
- `$stepName.field` - Specific field from step output
- `$$` - Full input to workflow
- `$$.field` - Field from workflow input

## Performance Notes

- Agent creation: ~10ms
- Tool execution: <10ms overhead  
- State access: <1ms
- Workflow step transition: <5ms

## Available Tools (34 Total)

**Categories**: API (2), Communication (2), Data Processing (2), File System (5), Media (3), Search (1), System (4), Utility (9), Web (6)

Use `Tool.list()` to discover all available tools and their capabilities.

---

*Note: This reference covers the current API. Hooks and Events are placeholders for Phase 4.*