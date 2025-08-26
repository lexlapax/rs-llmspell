# Rust Developer Examples

**Comprehensive Rust integration patterns for LLMSpell**

**🔗 Navigation**: [← Examples](../) | [Project Home](../../) | [Developer Guide](../../docs/developer-guide/) | [API Docs](../../docs/user-guide/api/rust/)

## 📚 Examples

Six comprehensive examples demonstrating LLMSpell Rust integration patterns.

**Complete Learning Path:**
1. **[custom-tool-example](custom-tool-example/)** - Creating custom tools with BaseAgent + Tool traits
2. **[custom-agent-example](custom-agent-example/)** - Building agents with personalities and specializations
3. **[async-patterns-example](async-patterns-example/)** - Concurrent execution, streaming, timeouts, and pipelines
4. **[extension-pattern-example](extension-pattern-example/)** - Plugin/extension architecture for extensible tools
5. **[builder-pattern-example](builder-pattern-example/)** - Fluent APIs and complex tool configuration
6. **[integration-test-example](integration-test-example/)** - Comprehensive testing strategies and patterns

**Key Concepts Covered:**
- BaseAgent trait implementation patterns
- Tool trait with categories and security levels
- AgentInput/AgentOutput structured communication
- Parameter validation and error handling
- Async patterns with tokio primitives
- Extension/plugin architecture patterns
- Builder pattern for complex configuration
- Unit testing, integration testing, and mocking

## 🚀 Running Examples

Each example is a standalone Rust project with its own Cargo.toml:

```bash
# Run any example (from rust-developers/)
cd custom-tool-example && cargo run
cd custom-agent-example && cargo run
cd async-patterns-example && cargo run
cd extension-pattern-example && cargo run
cd builder-pattern-example && cargo run
cd integration-test-example && cargo run

# Run tests for integration-test-example
cd integration-test-example && cargo test

# Check compilation for all examples
cd custom-tool-example && cargo check
# ... repeat for other examples
```

**Compilation Time:** ~30 seconds first build, <1 second subsequent builds  
**Execution Time:** <5 seconds per example  
**Total Learning Time:** ~30 minutes for all 6 examples

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

### Experienced Rust Developer
1. **Quick start:** [custom-tool-example](custom-tool-example/) - API overview in 5 minutes
2. **Advanced patterns:** [async-patterns-example](async-patterns-example/) - Concurrent execution patterns
3. **Architecture:** [extension-pattern-example](extension-pattern-example/) - Plugin system design

### Building Production Systems
1. **Testing:** [integration-test-example](integration-test-example/) - Professional testing strategies
2. **Configuration:** [builder-pattern-example](builder-pattern-example/) - Flexible tool configuration
3. **Performance:** [async-patterns-example](async-patterns-example/) - Concurrent execution optimization

### Specific Use Cases
- **Need extensible tools?** → [extension-pattern-example](extension-pattern-example/)
- **Complex configuration?** → [builder-pattern-example](builder-pattern-example/)
- **High performance?** → [async-patterns-example](async-patterns-example/)
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
- [API Reference](https://docs.rs/llmspell/) - Rust API docs
- [Contributing Guide](../../CONTRIBUTING.md) - Contribution guidelines