# LLMSpell User Guide

**Learn to build powerful LLM-driven applications with production-ready rs-llmspell**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [Examples](../../examples/) | [API Reference](api/)

---

## Overview

> **📚 Central Hub**: Your starting point for all LLMSpell documentation. Everything you need is organized into 8 essential documents, plus comprehensive API references for both Lua and Rust. Now with production daemon support and IDE integration!

**Version**: 0.9.0 | **Status**: Phase 10 Complete | **Last Updated**: December 2024

## 📖 Essential Documentation (8 Files)

### 1. [Getting Started](getting-started.md)
**Quick start in under 10 minutes**
- Installation and setup
- Progressive learning path (6 examples)
- RAG setup and first knowledge base
- Running your first daemon

### 2. [Core Concepts](concepts.md)
**Understand LLMSpell architecture including Phase 10 features**
- Component model (BaseAgent trait)
- Agents, Tools, Workflows
- RAG (Retrieval-Augmented Generation) ⭐
- Vector Storage & HNSW algorithm ⭐
- Multi-Tenancy with resource quotas ⭐
- Integrated Kernel Architecture (Phase 10) ⭐
- State management and sessions (unified in kernel)
- Hooks, Events, and Security model

### 3. [Configuration](configuration.md)
**Complete configuration guide including daemon setup**
- LLM providers (OpenAI, Anthropic, Ollama, Groq)
- RAG Configuration (HNSW, embeddings, chunking) ⭐
- Multi-Tenancy (isolation, quotas, billing) ⭐
- Daemon configuration (PID files, logging) ⭐
- State & Sessions persistence
- Security settings and deployment profiles
- Environment variables

### 4. [Service Deployment](service-deployment.md) ⭐ NEW
**Production deployment with system services**
- systemd deployment (Linux)
- launchd deployment (macOS)
- Daemon management (double-fork, signals)
- PID file handling
- Log rotation and monitoring
- Security best practices

### 5. [IDE Integration](ide-integration.md) ⭐ NEW
**Connect your IDE to LLMSpell kernel**
- VS Code setup with Jupyter & DAP
- Jupyter Lab configuration
- vim/neovim LSP integration
- Debug Adapter Protocol (DAP)
- Multi-client support
- Connection file format

### 6. [API Documentation](api/README.md)
**Comprehensive API reference**
- **[Lua API](api/lua/README.md)** - All 17 globals with 200+ methods
- **[Rust API](api/rust/README.md)** - 17 crates (consolidated from 20) with traits, builders, and extension guide
- Unified `llmspell-kernel` crate with state, sessions, and debugging

### 7. [Troubleshooting](troubleshooting.md)
**Solutions to common problems**
- Common issues and fixes
- Debugging techniques
- Performance optimization
- Error messages explained
- Daemon troubleshooting

### 8. [Examples](../../examples/EXAMPLE-INDEX.md)
**Learn by doing**
- 60+ working examples
- 6 Getting Started → 9 Applications progression
- RAG-powered applications and patterns
- Service deployment examples
- Best practices demonstrated

## 🚀 Quick Start

```bash
# Install and build
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell
cargo build --release

# Set API key
export OPENAI_API_KEY="sk-..."

# Run your first script (embedded kernel mode)
./target/release/llmspell exec '
  local agent = Agent.builder()
    :model("openai/gpt-4o-mini")
    :build()
  print(agent:execute({prompt = "Hello!"}).response)
'

# Use --trace flag for debugging (Phase 9)
./target/release/llmspell --trace debug run script.lua

# Start kernel as service (Phase 9-10)
./target/release/llmspell kernel start --port 9555

# Start as daemon (Phase 10)
./target/release/llmspell kernel start --daemon --port 9555

# Install as system service (Phase 10)
./target/release/llmspell kernel install-service
sudo systemctl start llmspell-kernel  # Linux
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist  # macOS
```

## 🆕 Phase 9-10 Features

### Phase 9: Kernel Architecture & Debug Support
- **Integrated Kernel**: Unified `llmspell-kernel` crate consolidating state/sessions/debug
- **Global IO Runtime**: Eliminates "dispatch task is gone" errors permanently
- **--trace Flag**: Replaces `--debug`/`--verbose` with unified logging control
- **Debug Adapter Protocol (DAP)**: Full IDE debugging with breakpoints and stepping
- **Event Correlation**: Track request flow with correlation IDs
- **Multi-Protocol Support**: Jupyter, DAP, LSP, REPL in single kernel
- **Message Router**: Handle multiple concurrent clients

### Phase 10: Production Daemon & Service
- **System Services**: Deploy as systemd (Linux) or launchd (macOS)
- **Daemon Mode**: Double-fork technique with proper TTY detachment
- **Signal Handling**: SIGTERM/SIGINT graceful shutdown, SIGHUP reload, SIGUSR1/2
- **PID Management**: Proper process control for service managers
- **kernel install-service**: Auto-generate and install service files
- **Fleet Management**: Run multiple kernel instances with load balancing
- **Health Monitoring**: HTTP endpoints for health checks and metrics
- **Log Rotation**: Automatic log management with compression

### IDE Integration
- **VS Code**: Full Jupyter notebook and debugging support
- **Jupyter Lab**: Native kernel integration with 5-channel architecture
- **vim/neovim**: LSP and DAP support
- **Connection Files**: Jupyter-compatible kernel discovery
- **Multi-Client**: Handle concurrent IDE connections

## 🧩 Available Globals (17)

All globals are pre-injected - no `require()` needed!

| Global | Purpose | Example |
|--------|---------|---------|
| **Agent** | LLM interactions | `Agent.builder():model("openai/gpt-4"):build()` |
| **Tool** | Execute tools (37+ available) | `Tool.invoke("web-search", {query = "..."})` |
| **Workflow** | Orchestration | `Workflow.sequential({steps = {...}})` |
| **State** | Data persistence | `State.set("key", value)` |
| **Session** | Session management | `Session.create({name = "..."})` |
| **Artifact** | Content storage | `Artifact.store(session_id, type, name, content)` |
| **Hook** | Intercept execution (40+ points) | `Hook.register("BeforeAgentExecution", handler)` |
| **Event** | Async notifications | `Event.publish("user.action", data)` |
| **Config** | Configuration access | `Config.get("providers.openai")` |
| **Provider** | Provider management | `Provider.list()` |
| **Debug** | Debugging utilities | `Debug.info("message", "module")` |
| **JSON** | JSON operations | `JSON.parse(string)` |
| **Streaming** | Stream handling | `Streaming.create()` |
| **Replay** | Event replay | `Replay.start()` |
| **RAG** | Vector search & retrieval | `RAG.search(query, {k = 5})` |
| **Metrics** | Performance metrics | `Metrics.get("kernel.uptime")` |
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

### Build RAG Application
```lua
-- Ingest documents
RAG.ingest(document, {collection = "knowledge"})

-- Search with vector similarity
local results = RAG.search(query, {
    k = 5,
    collection = "knowledge"
})

-- Use results in agent
local agent = Agent.builder()
    :model("openai/gpt-4")
    :build()
local response = agent:execute({
    prompt = "Based on: " .. results[1].text .. "\nAnswer: " .. question
})
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

### Connect from IDE
```bash
# Start kernel with connection file
./target/release/llmspell kernel start \
  --daemon \
  --connection-file ~/.llmspell/kernel.json

# In VS Code: Connect to existing Jupyter server
# Use the connection file path
```

## 📊 Key Metrics

| Operation | Performance | Limit |
|-----------|------------|-------|
| Agent creation | ~10ms | - |
| Tool execution | <10ms overhead | - |
| State read/write | <1ms / <5ms | - |
| Event throughput | 90K/sec | - |
| Message handling | <5ms (Phase 10) | - |
| Debug stepping | <20ms (Phase 10) | - |
| Daemon startup | <2s (Phase 10) | - |
| Memory overhead | <50MB (kernel) | 512MB default |
| Script timeout | - | 5 minutes |

## 🏗️ Architecture Overview

### Crate Structure (17 Total)
```
llmspell-kernel (Phase 10 - Unified)
├── State Management (merged)
├── Session Management (merged)
├── Debug Infrastructure
├── Daemon Support
├── Protocol Servers (Jupyter, DAP)
└── Global IO Runtime

Core Layer (3 crates)
├── llmspell-core (traits)
├── llmspell-utils (utilities)
└── llmspell-testing (test framework)

Execution Layer (5 crates)
├── llmspell-agents
├── llmspell-tools (37+ tools)
├── llmspell-workflows
├── llmspell-hooks (40+ points)
└── llmspell-events

Storage & RAG (3 crates)
├── llmspell-storage (HNSW vectors)
├── llmspell-rag (pipeline)
└── llmspell-tenancy (multi-tenant)

Integration Layer (3 crates)
├── llmspell-bridge (Lua/JS)
├── llmspell-config
└── llmspell-cli

Security & Providers (2 crates)
├── llmspell-security
└── llmspell-providers
```

## 🔍 Learning Path

1. **Beginners** → [Getting Started](getting-started.md) (5 min)
2. **Understanding** → [Core Concepts](concepts.md) (10 min)
3. **Building** → [Examples](../../examples/EXAMPLE-INDEX.md) (hands-on)
4. **Configuring** → [Configuration](configuration.md) (as needed)
5. **Deploying** → [Service Deployment](service-deployment.md) (production)
6. **IDE Setup** → [IDE Integration](ide-integration.md) (development)
7. **Debugging** → [Troubleshooting](troubleshooting.md) (when stuck)
8. **Reference** → [API Docs](api/README.md) (lookup)

## 🆘 Need Help?

- **Issues?** Check [Troubleshooting](troubleshooting.md)
- **Questions?** Review [Examples](../../examples/EXAMPLE-INDEX.md)
- **Bugs?** Report on [GitHub](https://github.com/yourusername/rs-llmspell/issues)
- **API Details?** See [Lua API](api/lua/README.md) or [Rust API](api/rust/README.md)
- **Deployment?** See [Service Deployment](service-deployment.md)
- **IDE Setup?** See [IDE Integration](ide-integration.md)

---

**Version 0.9.0** | Phase 10 - Production Kernel with Daemon Support | [Changelog](../../CHANGELOG.md)