# API Quick Reference

**📋 Implementation Status**: Current Phase 3.3 APIs - Features marked 📋 are planned for Phase 4+

This is a comprehensive quick reference for all rs-llmspell APIs. All globals are pre-injected - no require() needed!

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

## Hook Global 📋 **Phase 4 Feature**

*Hook system will be available in Phase 4*

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Hook.register(name, fn)` 📋 | `name`: string, `fn`: function | nil | Register hook (Phase 4) |
| `Hook.list()` 📋 | none | {string} | List hooks (Phase 4) |

## Event Global 📋 **Phase 4 Feature**

*Event system will be available in Phase 4*

| Method | Parameters | Returns | Description |
|--------|------------|---------|-------------|
| `Event.emit(name, data)` 📋 | `name`: string, `data`: table | nil | Emit event (Phase 4) |
| `Event.subscribe(name, fn)` 📋 | `name`: string, `fn`: function | nil | Subscribe to event (Phase 4) |

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