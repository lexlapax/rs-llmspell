# rs-llmspell User Guide

⚠️ **EVOLVING DOCUMENTATION**: This user guide reflects the current Phase 3.3 implementation and planned future features. Features are marked with implementation status:
- ✅ **Fully Available**: Tested and production-ready (Phases 0-3.2)
- 🚧 **In Development**: Currently being implemented (Phase 3.3)  
- 📋 **Planned Feature**: Designed but not yet available (Phase 4+)
- ❌ **Not Available**: Future or removed features

**Current Project Status**: Phase 3.3 - Agent Infrastructure & Basic Multi-Agent Coordination

**🔗 Navigation**: [← Documentation Hub](../README.md) | [Project Home](../../README.md) | [Developer Guide](../developer-guide/) | [Technical Docs](../technical/)

Welcome to the rs-llmspell user documentation! This guide will help you write powerful LLM automation scripts using Lua.

## 📚 Documentation Index

### Getting Started
- **[Getting Started Guide](getting-started.md)** - Start here! Learn about globals, basic patterns, and error handling
  - Pre-injected globals (Agent, Tool, Workflow, State, JSON, etc.)
  - Basic usage patterns
  - Error handling best practices
  - Migration from require() syntax
  - Common troubleshooting

### Core APIs
- **[Agent API](agent-api.md)** - Create and interact with LLM agents
  - Creating agents with different providers (OpenAI, Anthropic, etc.)
  - Executing prompts and getting completions
  - Tool integration with agents
  - Performance characteristics

- **[Workflow API](workflow-api.md)** - Build multi-step automations with advanced patterns
  - Sequential workflows for step-by-step execution
  - Conditional branching based on results
  - Loop patterns for iterative processing
  - Parallel execution for concurrent tasks
  - Advanced tool integration patterns

- **[External Tools Guide](external-tools-guide.md)** - Phase 3.1 external integration tools
  - Web tools (scraper, API tester, webhook caller)
  - Communication tools (email, database)
  - Quick reference and examples
  - Security and performance tips

### State & Data Management
- **[State Management](state-management.md)** - Share data between script components
  - Thread-safe in-memory storage
  - Workflow state sharing patterns
  - Common patterns (counters, caching, progress tracking)
  - Future persistence features (Phase 5)

### Configuration
- **[Configuration Guide](configuration/configuration.md)** - Configure security, resources, and deployment
  - Security settings and best practices
  - Resource limits and quotas
  - Tool-specific configurations
  - Deployment configurations (dev/staging/prod)
  - Monitoring and logging setup
  - Incident response procedures

- **[LLM Provider Configuration](providers.md)** - Configure LLM providers (OpenAI, Anthropic, etc.)
  - Provider/model hierarchical syntax
  - Supported providers and models
  - Configuration methods and best practices
  - Provider-specific features
  - Troubleshooting common issues

- **[API Setup Guides](configuration/api-setup-guides.md)** - Set up external API providers
  - Provider-specific setup instructions
  - API key management
  - Rate limiting configuration

### Hooks and Events (Phase 4)
- **[Hooks & Events Overview](hooks-events-overview.md)** - Introduction to extensibility systems
  - Understanding hooks vs events
  - Architecture and performance characteristics
  - Quick start examples
  - When to use each system

- **[Hooks Guide](hooks-guide.md)** - Comprehensive hook system documentation
  - All 40+ hook points with descriptions
  - 9 HookResult types for control flow
  - Priority system and execution order
  - CircuitBreaker protection (<5% overhead)
  - Practical examples and patterns

- **[Events Guide](events-guide.md)** - Master the event system
  - UniversalEvent format and pattern subscriptions
  - FlowController and backpressure handling
  - 90K+ events/sec throughput
  - Event persistence and replay
  - Cross-language event propagation

- **[Built-in Hooks Reference](builtin-hooks-reference.md)** - Production-ready hooks
  - 18+ built-in hooks for common use cases
  - Security, caching, rate limiting, monitoring
  - Configuration and performance impact
  - Combining hooks for complex scenarios

- **[Hook Patterns](hook-patterns.md)** - Common patterns and recipes
  - Composite hooks (Sequential, Parallel, Voting)
  - Cross-component coordination
  - Error handling and recovery
  - Performance monitoring patterns

- **[Cross-Language Integration](cross-language-integration.md)** - Multi-language support
  - Hook and event system across Lua/JS/Rust
  - Language adapters and serialization
  - Performance considerations
  - Security and sandboxing

### Advanced Topics
- **[Performance Tips](advanced/performance-tips.md)** - Optimization strategies
  - Script optimization techniques
  - Resource usage patterns
  - Caching strategies

### Tutorials & Examples
- **[Tutorial: Agents & Workflows](tutorial-agents-workflows.md)** - Step-by-step tutorial
  - Learn agents from basics to advanced patterns
  - Master all 4 workflow types with examples
  - Combine agents and workflows effectively
  - Performance optimization techniques

- **[Examples Directory](../../examples/)** - Working code examples
  - 10+ agent examples
  - 13+ tool examples  
  - 9+ workflow examples
  - Complete demos and benchmarks

### Reference
- **[Tool Reference](tool-reference.md)** - Complete reference for all 34 tools
  - Detailed documentation for every tool
  - Usage examples and parameters
  - Security levels and best practices
  - Performance characteristics

- **[Agent & Workflow API Reference](api-reference-agents-workflows.md)** - Comprehensive API docs
  - Complete method signatures
  - All parameters and options
  - Template syntax and references
  - Integration patterns

- **[API Reference](api-reference.md)** - Quick reference for all globals
  - Global objects (Agent, Tool, Workflow, State, etc.)
  - Quick method lookup
  - Common patterns

## 🚀 Quick Start

```lua
-- Your first rs-llmspell script
local agent = Agent.create({
    provider = "openai",
    model = "gpt-4"
})

local result = agent:complete("Hello, world!")
print(result)
```

## 📖 Reading Order

1. **New Users**: Start with [Getting Started](getting-started.md) to understand the basics
2. **LLM Tasks**: Learn about [Agents](agent-api.md) for AI-powered interactions
3. **Automation**: Explore [Workflows](workflow-api.md) for complex multi-step processes
4. **Data Sharing**: Use [State Management](state-management.md) for coordination
5. **Deployment**: Configure with the [Configuration Guide](configuration.md)
6. **Reference**: Keep the [API Reference](api-reference.md) handy while coding

## 🔍 Finding Help

- **Examples**: Check the `/examples` directory for working scripts
- **API Questions**: See the [API Reference](api-reference.md)
- **Configuration Issues**: Check the [Configuration Guide](configuration.md)
- **Troubleshooting**: Each guide includes a troubleshooting section
- **GitHub Issues**: Report bugs and feature requests in the project repository

## 📋 Prerequisites

- Basic Lua knowledge (syntax, tables, functions)
- Understanding of LLM concepts (prompts, completions, tokens)
- Access to LLM API keys (OpenAI, Anthropic, etc.)
- rs-llmspell installed and configured

## 🎯 Common Use Cases

### Text Processing
```lua
local agent = Agent.create({provider = "openai", model = "gpt-4"})
local summary = agent:complete("Summarize this text: " .. long_text)
```

### Data Pipeline
```lua
local workflow = Workflow.sequential({
    name = "data_processor",
    steps = {
        {name = "fetch", tool = "http_client", input = {url = api_url}},
        {name = "parse", tool = "json_parser", input = "$fetch.output"},
        {name = "analyze", agent = agent, prompt = "Analyze: $parse.output"}
    }
})
local results = workflow:execute()
```

### Tool Discovery
```lua
local tools = Tool.list()
for _, tool in ipairs(tools) do
    print(tool.name .. ": " .. tool.description)
end
```

## 🛠️ Available Tools

rs-llmspell includes **34 built-in tools** across 9 categories:
- File operations (read, write, search)
- Network tools (HTTP, webhooks, scrapers)
- Data processing (JSON, CSV, text)
- System utilities (process execution, environment)
- And many more!

Use `Tool.list()` to discover all available tools and their capabilities.

## 🔒 Security Note

rs-llmspell includes comprehensive security features:
- Sandboxed execution environment
- Resource limits and quotas
- Path traversal prevention
- Rate limiting
- See the [Configuration Guide](configuration.md) for security settings

## 📈 Performance

- Agent creation: ~10ms
- Tool execution: <10ms overhead
- Workflow operations: <20ms overhead
- State access: <1ms (in-memory)

## 🗺️ Roadmap

- **Current**: Phase 3 - Enhanced tools and agent infrastructure
- **Phase 4**: Hook system for lifecycle management
- **Phase 5**: Persistent state storage
- **Phase 6**: Session and artifact management
- **Phase 7+**: Vector storage, advanced features

Happy scripting with rs-llmspell! 🚀