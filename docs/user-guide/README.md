# LLMSpell User Guide

**Learn to build powerful LLM-driven applications with rs-llmspell**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [Examples](../../examples/) | [API Reference](api/)

---

## Overview

> **📚 Central Hub**: Your starting point for all LLMSpell documentation. Everything you need is organized into 7 essential documents.

**Version**: 0.6.0 | **Status**: Phase 7 Complete | **Last Updated**: August 2025

## 📖 Essential Documentation (7 Files)

### 1. [Getting Started](getting-started.md)
**Quick start in under 5 minutes**
- Installation and setup
- Your first script
- Quick examples

### 2. [Core Concepts](concepts.md)
**Understand LLMSpell architecture**
- Component model (BaseAgent trait)
- Agents, Tools, Workflows
- State management
- Execution context
- Security model

### 3. [Configuration](configuration.md)
**Complete configuration guide**
- LLM providers (OpenAI, Anthropic, Ollama, Groq)
- Security settings
- Resource limits
- External API setup
- Environment variables

### 4. [API Documentation](api/README.md)
**Comprehensive API reference**
- **[Lua API](api/lua/README.md)** - All 15 globals with 100+ methods
- **[Rust API](api/rust/README.md)** - Traits, builders, and extension guide

### 5. [Troubleshooting](troubleshooting.md)
**Solutions to common problems**
- Common issues and fixes
- Debugging techniques
- Performance optimization
- Error messages explained

### 6. [Examples](../../examples/EXAMPLE-INDEX.md)
**Learn by doing**
- 50+ working examples
- Universal → Professional progression
- Real-world applications
- Best practices demonstrated

### 7. This README
**Navigation hub** - You are here!

## 🚀 Quick Start

```bash
# Install and build
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell
cargo build --release

# Set API key
export OPENAI_API_KEY="sk-..."

# Run your first script
./target/release/llmspell exec '
  local agent = Agent.builder()
    :model("openai/gpt-4o-mini")
    :build()
  print(agent:execute({prompt = "Hello!"}).response)
'
```

## 🧩 Available Globals (15)

All globals are pre-injected - no `require()` needed!

| Global | Purpose | Example |
|--------|---------|---------|
| **Agent** | LLM interactions | `Agent.builder():model("openai/gpt-4"):build()` |
| **Tool** | Execute tools | `Tool.invoke("web-search", {query = "..."})` |
| **Workflow** | Orchestration | `Workflow.sequential({steps = {...}})` |
| **State** | Data persistence | `State.set("key", value)` |
| **Session** | Session management | `Session.create({name = "..."})` |
| **Artifact** | Content storage | `Artifact.store(session_id, type, name, content)` |
| **Hook** | Intercept execution | `Hook.register("BeforeAgentExecution", handler)` |
| **Event** | Async notifications | `Event.publish("user.action", data)` |
| **Config** | Configuration access | `Config.get("providers.openai")` |
| **Provider** | Provider management | `Provider.list()` |
| **Debug** | Debugging utilities | `Debug.info("message", "module")` |
| **JSON** | JSON operations | `JSON.parse(string)` |
| **Streaming** | Stream handling | `Streaming.create()` |
| **Replay** | Event replay | `Replay.start()` |
| **ARGS** | Script arguments | `ARGS.input` or `ARGS[1]` |

## 🎯 Common Tasks

### Chat with AI
```lua
local agent = Agent.builder()
    :model("openai/gpt-4o-mini")
    :build()
local response = agent:execute({prompt = "Explain quantum computing"})
print(response.response)
```

### Build Workflows
```lua
local workflow = Workflow.sequential({
    name = "pipeline",
    steps = {
        {name = "fetch", tool = "web-fetch", input = {url = "..."}},
        {name = "analyze", agent = agent, prompt = "Analyze: $fetch"}
    }
})
```

### Use Tools
```lua
local result = Tool.invoke("web-search", {
    query = "LLMSpell documentation",
    max_results = 10
})
```

## 📊 Key Metrics

| Operation | Performance | Limit |
|-----------|------------|-------|
| Agent creation | ~10ms | - |
| Tool execution | <10ms overhead | - |
| State read/write | <1ms / <5ms | - |
| Event throughput | 90K/sec | - |
| Memory per script | - | 512MB default |
| Script timeout | - | 5 minutes |

## 🔍 Learning Path

1. **Beginners** → [Getting Started](getting-started.md) (5 min)
2. **Understanding** → [Core Concepts](concepts.md) (10 min)
3. **Building** → [Examples](../../examples/EXAMPLE-INDEX.md) (hands-on)
4. **Configuring** → [Configuration](configuration.md) (as needed)
5. **Debugging** → [Troubleshooting](troubleshooting.md) (when stuck)
6. **Reference** → [API Docs](api/README.md) (lookup)

## 🆘 Need Help?

- **Issues?** Check [Troubleshooting](troubleshooting.md)
- **Questions?** Review [Examples](../../examples/EXAMPLE-INDEX.md)
- **Bugs?** Report on [GitHub](https://github.com/yourusername/rs-llmspell/issues)
- **API Details?** See [Lua API](api/lua/README.md) or [Rust API](api/rust/README.md)

---

**Version 0.6.0** | Phase 7 - API Standardization | [Changelog](../../CHANGELOG.md)