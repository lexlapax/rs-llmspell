# Script Users Examples

Examples for users writing Lua scripts to interact with LLMSpell.

## 📚 Categories

### [Getting Started](getting-started/)
Progressive examples from hello world to error handling. Start here if you're new to LLMSpell.

**Learning Path:**
1. `00-hello-world.lua` - Simplest possible example
2. `01-first-tool.lua` - Using your first tool
3. `02-first-agent.lua` - Creating your first agent
4. `03-first-workflow.lua` - Building your first workflow
5. `04-save-state.lua` - Persisting data between runs
6. `05-handle-errors.lua` - Proper error handling

### [Features](features/)
Comprehensive demonstrations of LLMSpell capabilities.

**Available Features:**
- Agents (OpenAI, Anthropic, local models)
- Tools (34+ built-in tools)
- Workflows (sequential, parallel, conditional, loops)
- State Management (persistence, scoping, sharing)
- Events (pub/sub, patterns, correlation)
- Hooks (lifecycle, priorities, modifications)
- Sessions (artifacts, replay, access control)

### [Cookbook](cookbook/)
Common patterns and best practices for production use.

**Recipes Include:**
- Error handling strategies
- Retry and timeout patterns
- Circuit breaker implementation
- Rate limiting
- Performance optimization
- Configuration management
- Multi-agent coordination
- State sharing patterns

### [Applications](applications/)
Complete, production-ready example applications.

**Example Applications:**
- AI Research Assistant
- Data Processing Pipeline
- Monitoring System
- Customer Support Bot
- Content Generation System

## 🚀 Running Examples

```bash
# Run any Lua example
llmspell run examples/script-users/getting-started/00-hello-world.lua

# With environment variables
OPENAI_API_KEY=your-key llmspell run examples/script-users/features/agents.lua

# With custom configuration
llmspell --config my-config.toml run examples/script-users/cookbook/rate-limiting.lua
```

## 📖 Prerequisites

### For Getting Started Examples
- LLMSpell installed (`cargo install llmspell` or build from source)
- No API keys required for basic examples

### For Agent Examples
- API key for at least one provider:
  - OpenAI: Set `OPENAI_API_KEY` environment variable
  - Anthropic: Set `ANTHROPIC_API_KEY` environment variable
  - Local models: Configure in `llmspell.toml`

### For Advanced Examples
- Understanding of Lua basics
- Familiarity with async patterns
- Knowledge of error handling

## 🎯 Learning Recommendations

### Complete Beginner
1. Start with [getting-started](getting-started/) examples in order
2. Try modifying examples to understand concepts
3. Move to [features](features/) for specific capabilities
4. Study [cookbook](cookbook/) for production patterns

### Experienced Developer
1. Skim [getting-started](getting-started/) for LLMSpell specifics
2. Jump to [features](features/) for your use case
3. Review [cookbook](cookbook/) for best practices
4. Study [applications](applications/) for architecture patterns

### Production Deployment
1. Start with [cookbook](cookbook/) patterns
2. Review [applications](applications/) for complete examples
3. Focus on error handling, monitoring, and performance
4. Test thoroughly with your specific use case

## 📝 Example Standards

All examples follow these standards:
- Clear header documentation
- Expected output documented
- Error handling included
- No hardcoded secrets
- Self-contained and runnable
- Tested in CI

See [../STANDARDS.md](../STANDARDS.md) for details.

## 🔗 Related Resources

- [Rust Developer Examples](../rust-developers/) - For embedding LLMSpell
- [User Guide](../../docs/user-guide/) - Comprehensive documentation
- [API Reference](../../docs/api-reference/) - Detailed API docs
- [Tool Reference](../../docs/user-guide/tool-reference.md) - All 34 tools