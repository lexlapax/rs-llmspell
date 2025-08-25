# LLMSpell Lua API Reference

## Overview
LLMSpell provides a comprehensive Lua API for scriptable LLM interactions. This documentation covers all available modules, functions, and patterns.

## Quick Navigation

### Core Modules
- [Agent](./agent.md) - LLM agent creation and management
- [Tool](./tool.md) - Tool invocation and management
- [Workflow](./workflow.md) - Workflow orchestration
- [State](./state.md) - State management and persistence
- [Session](./session.md) - Session management
- [Hook](./hook.md) - Hook system for extensibility
- [Event](./event.md) - Event tracking and correlation

### Configuration
- [Config](./config.md) - Configuration management
- [Provider](./provider.md) - LLM provider configuration

### Utilities
- [Utils](./utils.md) - Utility functions
- [Debug](./debug.md) - Debugging utilities

## Getting Started

### Basic Agent Creation
```lua
local agent = Agent.builder()
    :name("assistant")
    :type("llm")
    :model("openai/gpt-4o-mini")
    :build()

local response = agent:execute({
    prompt = "Hello, world!"
})
```

### Tool Invocation
```lua
local result = Tool.invoke("text-manipulator", {
    operation = "uppercase",
    text = "hello world"
})
```

### Workflow Creation
```lua
local workflow = Workflow.new("data-pipeline")
    :add_step("fetch", {type = "tool", tool = "web-fetch"})
    :add_step("process", {type = "agent", agent = agent})
    :run()
```

## API Conventions

### Return Values
- Success: Returns result data or `true`
- Failure: Returns `nil` and error message
- Async: Returns promise/future object

### Error Handling
```lua
local result, err = agent:execute(params)
if not result then
    print("Error:", err)
    return
end
```

### State Management
All stateful objects support:
- `save()` - Persist current state
- `load(id)` - Load from persistent storage
- `reset()` - Clear current state

## Version Compatibility

This documentation covers LLMSpell v0.6.0 and later.

### Breaking Changes
- v0.6.0: Agent builder pattern replaces direct construction
- v0.5.0: Tool registry centralized
- v0.4.0: State persistence added

## See Also
- [Rust API Documentation](../rust/index.html)
- [Example Applications](../../../examples/EXAMPLE-INDEX.md)
- [Configuration Guide](../../configuration/README.md)