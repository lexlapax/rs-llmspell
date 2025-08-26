# LLMSpell API Documentation

**Complete API reference for Lua and Rust interfaces**

**🔗 Navigation**: [← User Guide](../) | [Docs Hub](../../) | [Lua API](lua/) | [Rust API](rust/)

---

## Overview
LLMSpell provides comprehensive APIs for both Rust developers and script users. The platform offers identical functionality across all supported languages with consistent patterns and behaviors.

## Documentation Structure

### 📘 [Lua API](./lua/README.md)
**Complete Lua API Reference** - Comprehensive documentation for all Lua scripting capabilities.

**Coverage:**
- 15 Global objects (Agent, Tool, Workflow, State, Session, Hook, Event, Config, Provider, Debug, JSON, Args, Streaming, Artifact, Replay)
- 100+ Methods with full type signatures
- Complete parameter and return type documentation
- Error conditions and handling patterns
- Practical examples for every API method

**Key Globals:**
- `Agent` - LLM agent creation and execution
- `Tool` - Tool invocation and management
- `Workflow` - Workflow orchestration patterns
- `State` - Persistent state management
- `Session` - Session and artifact handling
- `Hook` - Lifecycle hooks and interception
- `Event` - Event emission and subscription

### 📙 [Rust API](./rust/README.md)
**Complete Rust API Reference** - Comprehensive documentation for extending LLMSpell with Rust.

**Coverage:**
- Core traits (BaseComponent, Executable, Agent, Tool, Workflow)
- Builder patterns for all components
- Component Registry system
- Complete error type hierarchy
- Bridge APIs for language integration
- Testing utilities and macros
- Performance optimization guidelines

**Key Crates:**
- `llmspell-core` - Core traits and types
- `llmspell-agents` - Agent infrastructure
- `llmspell-tools` - Tool implementations
- `llmspell-workflows` - Workflow orchestration
- `llmspell-bridge` - Script language integration
- `llmspell-state-persistence` - State management
- `llmspell-hooks` - Hook system
- `llmspell-events` - Event system

## Quick Start

### Rust Development
```rust
use llmspell_agents::{Agent, AgentBuilder};
use llmspell_tools::ToolRegistry;

let agent = AgentBuilder::new()
    .name("assistant")
    .model("openai/gpt-4")
    .build()?;

let response = agent.execute("Hello, world!").await?;
```

### Lua Scripting
```lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-4")
    :build()

local response = agent:execute({
    prompt = "Hello, world!"
})
```

## API Parity

Both Rust and Lua APIs provide identical functionality:

| Feature | Rust API | Lua API |
|---------|----------|---------|
| Agent Creation | `AgentBuilder` | `Agent.builder()` |
| Tool Invocation | `ToolRegistry::invoke()` | `Tool.invoke()` |
| Workflows | `WorkflowBuilder` | `Workflow.new()` |
| State Management | `StateManager` | `State` module |
| Async Operations | `async/await` | Promises/callbacks |
| Error Handling | `Result<T, E>` | `nil, error` returns |

## Design Principles

### 1. Consistency
- Same concepts and patterns across languages
- Predictable naming conventions
- Uniform error handling

### 2. Safety
- Type safety in Rust
- Runtime validation in Lua
- Comprehensive error messages

### 3. Performance
- Zero-cost abstractions in Rust
- Minimal overhead in script bridge
- Efficient state management

### 4. Extensibility
- Trait-based architecture
- Plugin system support
- Custom tool registration

## Common Patterns

### Builder Pattern
Both APIs use builder pattern for complex object creation:

**Rust:**
```rust
let agent = AgentBuilder::new()
    .name("agent")
    .temperature(0.7)
    .build()?;
```

**Lua:**
```lua
local agent = Agent.builder()
    :name("agent")
    :temperature(0.7)
    :build()
```

### Error Handling
Consistent error handling across languages:

**Rust:**
```rust
match agent.execute(prompt).await {
    Ok(response) => println!("{}", response),
    Err(e) => eprintln!("Error: {}", e),
}
```

**Lua:**
```lua
local response, err = agent:execute({prompt = prompt})
if not response then
    print("Error:", err)
end
```

### Async Operations
Both APIs support asynchronous operations:

**Rust:**
```rust
let response = agent.execute_async(prompt).await?;
```

**Lua:**
```lua
agent:execute_async({prompt = prompt})
    :then(function(response)
        print(response)
    end)
```

## Migration Guide

### From v0.5 to v0.6
- Agent creation now uses builder pattern
- Tool registry centralized
- State persistence added

See [CHANGELOG.md](../../../CHANGELOG.md) for detailed migration instructions.

## Performance Considerations

### Rust API
- Zero-cost abstractions
- Compile-time optimizations
- Direct memory management

### Lua API
- JIT compilation with LuaJIT
- Minimal bridge overhead (<1%)
- Efficient C bindings

## Testing

### Rust Testing
```bash
cargo test --workspace --all-features
```

### Lua Testing
```lua
-- Run test suite
require("llmspell.test").run_all()
```

## Examples

### Complete Examples
- [Rust Examples](../../../examples/rust/)
- [Lua Examples](../../../examples/lua/)
- [Application Examples](../../../examples/script-users/applications/)

### Code Snippets
Both API documentations include extensive code examples for every function and pattern.

## Support

### Resources
- [User Guide](../../README.md)
- [Configuration Guide](../../configuration/)
- [Troubleshooting](../../troubleshooting/)

### Community
- GitHub Issues: Report bugs and request features
- Discussions: Ask questions and share experiences
- Contributing: See [CONTRIBUTING.md](../../../CONTRIBUTING.md)

## Version Compatibility

| LLMSpell Version | Rust Edition | Lua Version | API Stability |
|-----------------|--------------|-------------|---------------|
| 0.6.x | 2021 | 5.1+ / LuaJIT | Beta |
| 0.5.x | 2021 | 5.1+ | Alpha |
| 0.4.x | 2021 | 5.1+ | Alpha |

## License

See [LICENSE](../../../LICENSE) for licensing information.