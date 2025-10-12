# LLMSpell Features - Core Capabilities

**Level**: INTERMEDIATE  
**Time**: 30 minutes  
**Prerequisites**: Completed getting-started examples

## 📚 Overview

This directory contains 5 essential feature demonstrations that bridge the gap between basic getting-started examples and advanced production patterns.

## 🎯 Learning Progression

```
getting-started/ → features/ (YOU ARE HERE) → advanced-patterns/ → cookbook/ → applications/
   BEGINNER        INTERMEDIATE                   ADVANCED          EXPERT      PROFESSIONAL
```

## 📖 Feature Examples

### 1. agent-basics.lua
**Core agent functionality**
- Agent.builder() pattern
- execute() method (standard API)
- Provider flexibility
- Agent discovery

```bash
# Requires API key (set OPENAI_API_KEY or ANTHROPIC_API_KEY)
llmspell -p providers run examples/script-users/features/agent-basics.lua

# Or with debug logging:
llmspell -p development run examples/script-users/features/agent-basics.lua
```

### 2. tool-basics.lua
**Essential tool operations**
- Tool.execute() for file operations
- UUID generation, encoding, hashing
- Tool discovery
- Error handling patterns

```bash
# No API key needed
./target/debug/llmspell run examples/script-users/features/tool-basics.lua
```

### 3. workflow-basics.lua
**Workflow orchestration patterns**
- Workflow.builder() construction
- Sequential vs parallel execution
- Data flow between steps
- Parameterized workflows

```bash
# No API key needed
./target/debug/llmspell run examples/script-users/features/workflow-basics.lua
```

### 4. state-persistence.lua
**State management features**
- State.save() with scopes (global, custom, workflow, agent, tool)
- State.load() and State.delete()
- Atomic operations
- Conflict resolution

```bash
# Requires state persistence enabled
llmspell -p state run examples/script-users/features/state-persistence.lua

# Or with sessions (includes state + hooks + events):
llmspell -p sessions run examples/script-users/features/state-persistence.lua
```

### 5. provider-info.lua
**Provider discovery and configuration**
- List available providers
- Check provider capabilities
- Model enumeration
- Configuration validation

```bash
# Works without API keys (shows available providers)
llmspell run examples/script-users/features/provider-info.lua

# Or with providers profile to show configured details:
llmspell -p providers run examples/script-users/features/provider-info.lua
```

## 🔑 Key Concepts

### Execution Model
- **Synchronous API**: All operations block until complete
- **Single execution method**: Use `agent:execute()` not `invoke()`
- **Structured responses**: Returns tables with `text` and metadata

### Error Handling
```lua
local success, result = pcall(function()
    return agent:execute({text = "Hello"})
end)
if success then
    print(result.text)
else
    print("Error: " .. tostring(result))
end
```

### Builder Pattern
All major components use fluent builder pattern:
```lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-3.5-turbo")
    :temperature(0.7)
    :build()
```

## 🚀 Next Steps

After mastering these features:
1. Explore **advanced-patterns/** for complex orchestration
2. Study **cookbook/** for production-ready patterns
3. Review **applications/** for complete systems

## 📝 Common Issues

### API Key Not Set
Set environment variables and use providers profile:
```bash
export OPENAI_API_KEY="your-key-here"
export ANTHROPIC_API_KEY="your-key-here"

# Then run with providers profile:
llmspell -p providers run examples/script-users/features/agent-basics.lua
```

### State Not Available
Use state or sessions builtin profile:
```bash
# Basic state:
llmspell -p state run examples/script-users/features/state-persistence.lua

# Full sessions (state + hooks + events):
llmspell -p sessions run examples/script-users/features/state-persistence.lua
```

### Wrong Method Name
- ✅ Use: `agent:execute({text = "..."})`
- ❌ Don't use: `agent:invoke()` or `agent:execute({prompt = "..."})`

### State API Requires Scope
- ✅ Use: `State.save("global", "key", value)`
- ❌ Don't use: `State.save("key", value)`

## 📊 Execution Times

| Example | Time | API Key Required |
|---------|------|-----------------|
| agent-basics.lua | 8s | Yes |
| tool-basics.lua | 3s | No |
| workflow-basics.lua | 2s | No |
| state-persistence.lua | 5s | No |
| provider-info.lua | 1s | No |

## 🔗 Related Documentation

- [Lua API Reference](../../../docs/user-guide/api/lua/README.md)
- [Getting Started](../getting-started/README.md)
- [Advanced Patterns](../advanced-patterns/README.md)
- [Cookbook](../cookbook/README.md)