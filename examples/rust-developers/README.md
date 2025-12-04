# Rust Developer Examples

**Comprehensive Rust integration patterns for LLMSpell**

**🔗 Navigation**: [← Examples](../) | [Project Home](../../) | [Developer Guide](../../docs/developer-guide/) | [API Docs](../../docs/user-guide/api/rust/)

## 📚 Examples

Three core examples demonstrating LLMSpell Rust integration patterns. For advanced patterns like async execution, builder patterns, and extension architecture, see API documentation and developer guides.

**Complete Learning Path:**
1. **[custom-tool-example](custom-tool-example/)** - Creating custom tools with BaseAgent + Tool traits
2. **[custom-agent-example](custom-agent-example/)** - Building agents with personalities and specializations
3. **[integration-test-example](integration-test-example/)** - Comprehensive testing strategies and patterns

**Key Concepts Covered:**
- BaseAgent trait implementation patterns
- Tool trait with categories and security levels
- AgentInput/AgentOutput structured communication
- Parameter validation and error handling
- Unit testing, integration testing, and mocking

**Advanced Patterns (See API Docs):**
- **Async Patterns**: See doc tests in [llmspell-core BaseAgent](https://docs.rs/llmspell-core) - concurrent execution, timeouts, select patterns
- **Builder Pattern**: See doc tests in [llmspell-tools](https://docs.rs/llmspell-tools) - fluent APIs and tool configuration
- **Extension Architecture**: See [Extension Architecture Guide](../../docs/developer-guide/extension-architecture.md) - plugin/extension systems

## 🚀 Running Examples

Each example is a standalone Rust project with its own Cargo.toml:

```bash
# Run any example (from rust-developers/)
cd custom-tool-example && cargo run
cd custom-agent-example && cargo run
cd integration-test-example && cargo run

# Run tests for integration-test-example
cd integration-test-example && cargo test

# Check compilation for all examples
cd custom-tool-example && cargo check
cd custom-agent-example && cargo check
cd integration-test-example && cargo check
```

**Compilation Time:** ~20 seconds first build, <1 second subsequent builds
**Execution Time:** <5 seconds per example
**Total Learning Time:** ~15 minutes for all 3 core examples

## 📖 Prerequisites

### Development Environment
- Rust 1.70+ installed
- Cargo configured
- LLMSpell dependencies

### For Agent Examples
- API keys configured
- Provider dependencies enabled

### For Extension Examples
- Understanding of trait systems
- Async Rust knowledge
- Error handling patterns

## 🎯 Learning Recommendations

### New to LLMSpell
1. **Start here:** [custom-tool-example](custom-tool-example/) - Learn BaseAgent + Tool fundamentals
2. **Then:** [custom-agent-example](custom-agent-example/) - Understand agent personalities and specialization
3. **Finally:** [integration-test-example](integration-test-example/) - Learn testing patterns
4. **Advanced:** Read [Extension Architecture Guide](../../docs/developer-guide/extension-architecture.md) for extensible systems

### Experienced Rust Developer
1. **Quick start:** [custom-tool-example](custom-tool-example/) - API overview in 5 minutes
2. **Advanced patterns:** See [llmspell-core doc tests](https://docs.rs/llmspell-core) for async execution patterns
3. **Architecture:** [Extension Architecture Guide](../../docs/developer-guide/extension-architecture.md) for plugin system design

### Building Production Systems
1. **Testing:** [integration-test-example](integration-test-example/) - Professional testing strategies
2. **Configuration:** See [llmspell-tools doc tests](https://docs.rs/llmspell-tools) for builder patterns
3. **Performance:** See [llmspell-core doc tests](https://docs.rs/llmspell-core) for concurrent execution optimization

### Specific Use Cases
- **Need extensible tools?** → [Extension Architecture Guide](../../docs/developer-guide/extension-architecture.md)
- **Complex configuration?** → [llmspell-tools builder pattern doc tests](https://docs.rs/llmspell-tools)
- **High performance?** → [llmspell-core async pattern doc tests](https://docs.rs/llmspell-core)
- **Agent personalities?** → [custom-agent-example](custom-agent-example/)

## 📝 Example Standards

All examples follow Rust best practices:
- Comprehensive error handling
- Proper use of Result types
- Documentation comments
- Unit tests included
- No unwrap() in production code
- Idiomatic Rust patterns

## 🔗 Related Resources

- [Script User Examples](../script-users/) - Lua scripting
- [Developer Guide](../../docs/developer-guide/) - Architecture docs
- [Extension Architecture](../../docs/developer-guide/extension-architecture.md) - Plugin system design
- [API Reference](https://docs.rs/llmspell/) - Rust API docs (includes async/builder pattern doc tests)
- [Contributing Guide](../../CONTRIBUTING.md) - Contribution guidelines