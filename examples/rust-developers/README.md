# Rust Developer Examples

Examples for developers embedding LLMSpell in Rust applications.

## 📚 Categories

### [Getting Started](getting-started/)
Learn how to embed LLMSpell in your Rust application.

**Learning Path:**
1. `00-embed-llmspell.rs` - Basic embedding
2. `01-custom-tool.rs` - Creating custom tools
3. `02-custom-agent.rs` - Building custom agents
4. `03-workflows.rs` - Workflow integration
5. `04-testing.rs` - Testing patterns

### [API Usage](api-usage/)
Comprehensive API demonstrations and patterns.

**Topics Covered:**
- Agent management
- Tool registration
- Workflow builders
- State persistence
- Event handling
- Hook integration
- Session management

### [Patterns](patterns/)
Design patterns and best practices.

**Patterns Include:**
- Dependency injection
- Factory patterns
- Registry patterns
- Observer patterns
- Strategy patterns
- Builder patterns

### [Extensions](extensions/)
Creating custom components and extending LLMSpell.

**Extension Points:**
- Custom tools
- Custom agents
- Provider implementations
- Storage backends
- Hook implementations

## 🚀 Running Examples

```bash
# Run from crate examples
cargo run --example basic_agent

# Run standalone example
cd examples/rust-developers/getting-started
cargo run --bin embed-llmspell

# Run with features
cargo run --features "openai anthropic" --example custom_agent
```

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
1. Start with [getting-started](getting-started/) examples
2. Review [api-usage](api-usage/) for your use case
3. Study [patterns](patterns/) for best practices

### Experienced Rust Developer
1. Jump to [api-usage](api-usage/) for API overview
2. Review [patterns](patterns/) for architecture
3. Explore [extensions](extensions/) for customization

### Building Production Systems
1. Focus on [patterns](patterns/) for robust design
2. Study error handling and testing examples
3. Review performance optimization patterns

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