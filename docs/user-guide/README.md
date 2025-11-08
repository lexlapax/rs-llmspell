# LLMSpell User Guide

**Learn to experiment with AI concepts through rapid scripting with rs-llmspell**

🔗 **Navigation**: [← Docs Hub](../) | [Project Home](../../) | [Examples](../../examples/) | [Developer Guide](../developer-guide/)

---

## Overview

> **📚 Central Hub**: Your starting point for AI experimentation with LLMSpell. Everything you need is organized into **10 numbered guides** plus a comprehensive appendix with complete API references. Master the essentials quickly, then explore advanced features at your own pace!

**Version**: 0.13.0 | **Status**: Phase 13b Complete - Experimental Memory & Context Engineering | **Last Updated**: 2025-11-08

---

## 📖 The 10 Essential Guides

**Linear learning path from setup to production deployment**

### [01. Getting Started](01-getting-started.md)
**Get up and running in under 10 minutes**
- Installation and build options
- Your first agent, tool, and RAG example
- Progressive learning path (6 examples)
- Memory system quick start
- Feature flags overview

### [02. Core Concepts](02-core-concepts.md)
**Understand llmspell's experimental architecture**
- Component model (BaseAgent trait, tools, workflows)
- Agents, Tools (40+), Templates (10 workflows)
- RAG (Retrieval-Augmented Generation) with HNSW
- Memory System (episodic, semantic, procedural)
- Context Engineering (4 strategies)
- Multi-tenancy and resource quotas
- State, Sessions, Hooks, Events

### [03. Configuration](03-configuration.md)
**Complete configuration reference**
- LLM providers (OpenAI, Anthropic, Ollama, Groq, Candle)
- RAG configuration (HNSW, embeddings, chunking)
- Memory system (consolidation, daemon, profiles)
- Multi-tenancy (isolation, quotas, billing)
- Feature flags (minimal/common/full builds)
- Security settings and deployment profiles
- Environment variables

### [04. Lua Scripting Essentials](04-lua-scripting.md)
**Quick guide to writing Lua scripts**
- 18 Lua globals overview
- Quick start examples (Agent, Tool, RAG, Memory, Template)
- Common patterns (multi-agent, stateful conversation, tool chaining)
- Error handling and debugging
- Links to complete API reference

### [05. CLI Reference](05-cli-reference.md)
**Complete command-line interface documentation**
- All 16 command groups (run, exec, kernel, tool, template, memory, context, etc.)
- Global options and trace levels
- Built-in profiles and feature flags
- Examples for every command
- Quick reference table

### [06. Templates & Workflows](06-templates-and-workflows.md)
**Pre-built AI workflow templates for rapid experimentation**
- 10 experimental templates by category
- CLI usage (list, info, exec, search, schema)
- Lua API usage (Template global)
- Template examples and customization
- Template composition and integration

### [07. Storage Setup](07-storage-setup.md)
**Quick start guide for PostgreSQL storage**
- Docker Compose setup (5 minutes)
- Basic connection configuration
- Simple backup procedures
- Links to technical docs for deep dives

### [08. Deployment](08-deployment.md)
**Production deployment strategies**
- systemd deployment (Linux)
- launchd deployment (macOS)
- Daemon management and lifecycle
- IDE integration (VS Code, Jupyter, vim)
- Debug Adapter Protocol (DAP)
- Multi-client support

### [09. Security](09-security.md)
**Security sandbox and permission system**
- Three-level security model (Safe/Restricted/Privileged)
- Sandbox systems (File, Network, Integrated)
- Permission configuration
- Tool-specific security settings
- Troubleshooting permission errors
- Security best practices

### [10. Troubleshooting](10-troubleshooting.md)
**Solutions to common problems**
- Common issues and quick fixes
- Kernel and service troubleshooting
- Debugging techniques (--trace flag, RUST_LOG)
- Performance optimization
- API and provider issues
- Script errors and tool issues
- IDE integration debugging
- Diagnostic procedures

---

## 📚 Appendix: Complete API References

### [Lua API Reference](appendix/lua-api-reference.md)
**Complete documentation for all 18 Lua globals**
- 200+ methods across Agent, Tool, Workflow, Template, Memory, Context, RAG, etc.
- Organized by global with detailed examples
- Searchable reference for lookups

---

## 🚀 Quick Start

```bash
# Install and build
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell

# Choose your build:
cargo build --release                      # Minimal (19MB, core only)
cargo build --release --features common    # Common (25MB, templates + PDF)
cargo build --release --features full      # Full (35MB, all tools)

# Set API key
export OPENAI_API_KEY="sk-..."

# Run your first script
./target/release/llmspell exec '
  local agent = Agent.new({
    provider = "openai",
    model = "gpt-4o-mini"
  })
  print(agent:execute("Hello!").content)
'

# Use tool CLI directly
./target/release/llmspell tool list
./target/release/llmspell tool invoke calculator --params '{"input": "2+2"}'

# Use AI Agent Templates - Instant productive AI
./target/release/llmspell template list
./target/release/llmspell template exec research-assistant \
  --param topic="Rust async programming" \
  --param depth="comprehensive"

# Use Memory System
./target/release/llmspell memory add "PostgreSQL is a relational database" \
  --type semantic
./target/release/llmspell memory search "What is PostgreSQL?"

# Start kernel as daemon
./target/release/llmspell kernel start --daemon --port 9555
```

---

## 🆕 Phase 13 Features (Complete)

### Adaptive Memory & Context Engineering
- **Multi-Tier Memory System**: Episodic (conversation history), Semantic (knowledge graph), Procedural (patterns)
- **Hot-Swappable Backends**: HNSW (8.47x speedup), InMemory (dev), SurrealDB (bi-temporal graph)
- **Bi-Temporal Knowledge Graph**: Event time + ingestion time tracking
- **Context Engineering Pipeline**: Query understanding, multi-strategy retrieval, DeBERTa reranking, token-aware assembly
- **LLM-Driven Consolidation**: Extract entities and relationships into knowledge graph
- **CLI Commands**: `llmspell memory add|search|consolidate`, `llmspell context assemble`
- **Lua API**: Memory global (17th) + Context global (18th)
- **Performance**: <2ms episodic add, ~8ms context assembly, ~2x parallel hybrid speedup
- **149 Tests**: 100% pass rate, zero warnings

### Phase 12 Features (Complete)
- **10 AI Workflow Templates**: Research, Chat, Analysis, Code, Documents, Orchestration
- **Template CLI**: `template list|info|exec|search|schema`
- **Template Lua API**: Template global (16th) with 6 methods
- **Performance**: 20-50x faster than targets (0.5ms list, 2ms execute overhead)
- **Quality**: 149 tests, 3,655 lines docs, production-ready architecture

---

## 🧩 Available Lua Globals (18)

All globals are pre-injected - no `require()` needed!

| Global | Purpose | Example |
|--------|---------|---------|
| **Agent** | LLM interactions | `Agent.new({provider = "openai", model = "gpt-4o-mini"})` |
| **Tool** | Execute tools (40+ available) | `Tool.get("web-search"):invoke({query = "..."})` |
| **Workflow** | Orchestration | `Workflow.new({type = "sequential", steps = {...}})` |
| **Template** | AI workflow templates | `Template.execute("research-assistant", {topic = "..."})` |
| **Memory** | Multi-tier memory | `Memory.add_episodic("session", "content", {metadata})` |
| **Context** | Context engineering | `Context.assemble({query = "...", strategies = {"hybrid"}})` |
| **RAG** | Vector search & retrieval | `RAG.search(query, {k = 5})` |
| **State** | Data persistence | `State.write("key", value)` |
| **Session** | Session management | `Session.create({name = "..."})` |
| **Hook** | Intercept execution | `Hook.add("before_agent_execution", handler)` |
| **Event** | Async notifications | `Event.publish("user.action", data)` |
| **Config** | Configuration access | `Config.get("providers.openai")` |
| **Provider** | Provider management | `Provider.list()` |
| **Model** | Model management | `Model.pull("llama3.2")` |
| **Debug** | Debugging utilities | `Debug.info("message", "module")` |
| **Security** | Security controls | `Security.check_permission("file_read")` |
| **Kernel** | Kernel control | `Kernel.start({port = 9555})` |
| **Embedding** | Generate embeddings | `Embedding.generate("text", {model = "..."})` |

---

## 🎯 Common Tasks

### Chat with AI
```lua
local agent = Agent.new({
    provider = "openai",
    model = "gpt-4o-mini"
})
local response = agent:execute("Explain quantum computing")
print(response.content)
```

### Build RAG Application
```lua
-- Ingest documents
local rag = RAG.new({collection = "knowledge"})
rag:ingest("Getting started with llmspell...", {
    source = "docs/getting-started.md"
})

-- Search with vector similarity
local results = rag:search("How do I get started?", {k = 5})

-- Use results in agent
local agent = Agent.new({provider = "openai", model = "gpt-4o-mini"})
local response = agent:execute(
    "Based on: " .. results[1].content .. "\nAnswer: How do I get started?"
)
```

### Use Memory System
```lua
-- Add episodic memory (conversation)
Memory.add_episodic("session_123", "User asked about PostgreSQL", {
    role = "user",
    timestamp = os.time()
})

-- Add semantic memory (facts)
Memory.add_semantic("PostgreSQL is a relational database", {
    relations = {
        {"PostgreSQL", "is_a", "database"},
        {"PostgreSQL", "supports", "ACID"}
    }
})

-- Query memory
local memories = Memory.query_episodic("PostgreSQL setup", {
    session_id = "session_123",
    k = 3
})
```

### Use AI Agent Templates
```lua
-- Research a topic
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    depth = "comprehensive",
    sources = "10"
})
print(result.content)

-- Or use CLI directly
-- llmspell template exec research-assistant \
--   --param topic="Rust async programming" \
--   --param depth="comprehensive"
```

### Deploy as Service
```bash
# Install service
./target/release/llmspell kernel install-service --type systemd

# Start and enable
sudo systemctl start llmspell-kernel
sudo systemctl enable llmspell-kernel

# Check status
sudo systemctl status llmspell-kernel
```

---

## 📊 Key Performance Metrics

| Operation | Target | Achieved | Status |
|-----------|--------|----------|--------|
| Memory episodic add | <10ms | <2ms | ✅ 5x faster |
| Memory search (HNSW) | <100ms | ~50ms | ✅ 2x faster |
| Context assembly | <20ms | ~8ms | ✅ 2.5x faster |
| Template list | <10ms | 0.5ms | ✅ 20x faster |
| Template execute overhead | <100ms | 2ms | ✅ 50x faster |
| Tool initialization | <10ms | 7ms | ✅ 30% faster |
| Vector search (100K) | <10ms | 8ms | ✅ 20% faster |
| Message handling | <5ms | 3.8ms | ✅ 24% faster |

---

## 🔍 Learning Path

1. **Start Here** → [Getting Started](01-getting-started.md) (10 min)
2. **Understand** → [Core Concepts](02-core-concepts.md) (15 min)
3. **Quick Win** → [Templates & Workflows](06-templates-and-workflows.md) (<5 min to productive AI)
4. **Build** → [Examples](../../examples/EXAMPLE-INDEX.md) (hands-on)
5. **Configure** → [Configuration](03-configuration.md) (as needed)
6. **Script** → [Lua Scripting](04-lua-scripting.md) (essentials)
7. **Deploy** → [Deployment](08-deployment.md) (production)
8. **Secure** → [Security](09-security.md) (permissions)
9. **Debug** → [Troubleshooting](10-troubleshooting.md) (when stuck)
10. **Reference** → [Lua API](appendix/lua-api-reference.md) (lookup)

---

## 🆘 Need Help?

- **Getting Started?** See [01. Getting Started](01-getting-started.md)
- **Understanding Concepts?** See [02. Core Concepts](02-core-concepts.md)
- **Configuration Issues?** See [03. Configuration](03-configuration.md)
- **Scripting Questions?** See [04. Lua Scripting](04-lua-scripting.md)
- **CLI Commands?** See [05. CLI Reference](05-cli-reference.md)
- **Template Usage?** See [06. Templates & Workflows](06-templates-and-workflows.md)
- **Storage Setup?** See [07. Storage Setup](07-storage-setup.md)
- **Deployment?** See [08. Deployment](08-deployment.md)
- **Security?** See [09. Security](09-security.md)
- **Troubleshooting?** See [10. Troubleshooting](10-troubleshooting.md)
- **API Reference?** See [Lua API](appendix/lua-api-reference.md)
- **Examples?** See [Example Index](../../examples/EXAMPLE-INDEX.md)
- **Developer Guide?** See [Developer Guide](../developer-guide/README.md)
- **Technical Details?** See [Technical Docs](../technical/README.md)
- **Bugs?** Report on [GitHub](https://github.com/anthropics/llmspell/issues)

---

**Version**: 0.13.0 | **Phase**: 13b.18.3 | **Last Updated**: 2025-11-08
