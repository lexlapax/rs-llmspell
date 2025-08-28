# LLMSpell API Documentation

**Complete API reference for scripting and extending LLMSpell**

**🔗 Navigation**: [← User Guide](../) | [Project Home](../../../) | [Examples](../../../examples/)

---

## Overview

> **🔍 API Hub**: Comprehensive documentation for both Lua scripting APIs and Rust extension APIs. Choose your path based on whether you're writing scripts or building components.

**Version**: 0.8.0 | **Status**: Phase 8 Complete | **Last Updated**: December 2024

## Documentation Structure

### 📘 [Lua API](./lua/README.md)
**Complete Lua API Reference** - Comprehensive documentation for all Lua scripting capabilities.

**Coverage:**
- 17 Global objects (Agent, Tool, Workflow, State, Session, Hook, Event, Config, Provider, Debug, JSON, Args, Streaming, Artifact, Replay, RAG, Migration)
- 200+ Methods with full type signatures
- Complete parameter and return type documentation
- Error conditions and handling patterns
- Practical examples for every API method

**Key Globals:**
- `Agent` - LLM agent creation and execution (25+ methods)
- `Tool` - Tool invocation and management (15+ methods)
- `Workflow` - Workflow orchestration patterns (20+ methods)
- `RAG` - Retrieval-Augmented Generation (9+ methods) **Phase 8**
- `State` - Persistent state management (15+ methods)
- `Session` - Session and artifact handling (20+ methods)
- `Hook` - Lifecycle hooks and interception (10+ methods)
- `Event` - Event emission and subscription (15+ methods)

### 📙 [Rust API](./rust/README.md)
**Complete Rust API Reference** - Comprehensive documentation for extending LLMSpell with Rust.

**Coverage:**
- 19 crates fully documented with traits, implementations, and examples
- Core traits (BaseAgent, Executable, Agent, Tool, Workflow)
- Builder patterns for all components
- Component Registry system
- Complete error type hierarchy
- Bridge APIs for language integration
- Testing utilities and macros
- Performance optimization guidelines

**Key Crates by Phase:**

**Phase 8 - RAG & Multi-Tenancy:**
- `llmspell-storage` - HNSW vector storage for RAG
- `llmspell-rag` - Retrieval-Augmented Generation pipeline
- `llmspell-tenancy` - Multi-tenant isolation and quotas

**Core Infrastructure:**
- `llmspell-core` - Foundation traits and types
- `llmspell-utils` - Security and utilities
- `llmspell-testing` - Test framework

**State & Sessions:**
- `llmspell-state-persistence` - Persistent state with migration
- `llmspell-state-traits` - State trait definitions
- `llmspell-sessions` - Session management with artifacts
- `llmspell-security` - Security and access control

**AI & Execution:**
- `llmspell-agents` - Agent framework
- `llmspell-providers` - LLM provider integrations
- `llmspell-workflows` - Workflow orchestration
- `llmspell-tools` - 100+ built-in tools
- `llmspell-hooks` - Lifecycle hooks
- `llmspell-events` - Event bus (90K events/sec)

**Integration:**
- `llmspell-bridge` - Lua/script integration (<1% overhead)
- `llmspell-config` - Configuration system
- `llmspell-cli` - Command-line interface

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