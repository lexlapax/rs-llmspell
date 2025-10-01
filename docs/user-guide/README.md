# LLMSpell User Guide

**Learn to build powerful LLM-driven applications with production-ready rs-llmspell**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [Examples](../../examples/) | [API Reference](api/)

---

## Overview

> **📚 Central Hub**: Your starting point for all LLMSpell documentation. Everything you need is organized into 10 essential documents, plus comprehensive API references for both Lua and Rust. Now with Unix daemon infrastructure, tool CLI commands, fleet management, and feature flags!

**Version**: 0.10.0 | **Status**: Phase 10 Complete - Service Integration & IDE Connectivity | **Last Updated**: January 2025

## 📖 Essential Documentation (10 Files)

### 1. [Getting Started](getting-started.md)
**Quick start in under 10 minutes**
- Installation and setup with feature flags
- Progressive learning path (6 examples)
- RAG setup and first knowledge base
- Running your first daemon with tool CLI

### 2. [Core Concepts](concepts.md)
**Understand LLMSpell architecture including Phase 10 features**
- Component model (BaseAgent trait)
- Agents, Tools (40+ with feature flags), Workflows
- RAG (Retrieval-Augmented Generation) ⭐
- Vector Storage & HNSW algorithm ⭐
- Multi-Tenancy with resource quotas ⭐
- Integrated Kernel Architecture (Phase 10) ⭐
- Tool CLI and direct tool invocation ⭐
- Fleet management and process isolation ⭐
- State management and sessions (unified in kernel)
- Hooks, Events, and Security model

### 3. [Configuration](configuration.md)
**Complete configuration guide including daemon and fleet setup**
- LLM providers (OpenAI, Anthropic, Ollama, Groq)
- RAG Configuration (HNSW, embeddings, chunking) ⭐
- Multi-Tenancy (isolation, quotas, billing) ⭐
- Daemon configuration (PID files, log rotation, signals) ⭐
- Feature flags for modular builds (19-35MB) ⭐
- Fleet management and multi-kernel orchestration ⭐
- State & Sessions persistence
- Security settings and deployment profiles
- Environment variables

### 4. [Service Deployment](service-deployment.md) ⭐ Phase 10
**Production deployment with system services**
- systemd deployment (Linux)
- launchd deployment (macOS)
- Daemon management (double-fork, TTY detachment, signals)
- PID file handling and lifecycle tracking
- Log rotation with size/age policies
- Fleet management for multi-kernel deployments
- Security best practices

### 5. [IDE Integration](ide-integration.md) ⭐ Phase 10
**Connect your IDE to LLMSpell kernel**
- VS Code setup with Jupyter & DAP
- Jupyter Lab configuration
- vim/neovim LSP integration
- Debug Adapter Protocol (10 DAP commands)
- Multi-client support
- Connection file format and kernel discovery

### 6. [API Documentation](api/README.md)
**Comprehensive API reference**
- **[Lua API](api/lua/README.md)** - All 17 globals with 200+ methods
- **[Rust API](api/rust/README.md)** - 17 crates with traits, builders, and extension guide
- Unified `llmspell-kernel` crate with daemon, state, sessions, and debugging
- Feature flags documentation for modular builds

### 7. [Troubleshooting](troubleshooting.md)
**Solutions to common problems**
- Common issues and fixes
- Debugging techniques
- Error messages explained
- Daemon and service troubleshooting
- Feature flag issues

### 8. [Phase 10 Troubleshooting](troubleshooting-phase10.md) ⭐ NEW
**Phase 10 specific issues**
- Daemon startup problems
- Signal handling issues
- PID file conflicts
- Log rotation configuration
- Fleet management debugging
- Tool CLI troubleshooting

### 9. [Performance Tuning](performance-tuning.md) ⭐ NEW
**Optimize for production**
- Daemon performance tuning
- Memory optimization
- Log rotation settings
- Fleet scaling strategies
- HNSW parameter tuning
- Multi-tenant resource allocation

### 10. [Examples](../../examples/EXAMPLE-INDEX.md)
**Learn by doing**
- 60+ working examples
- 6 Getting Started → 9 Applications progression
- RAG-powered applications and patterns
- Service deployment examples
- Tool CLI usage examples
- Best practices demonstrated

## 🚀 Quick Start

```bash
# Install and build (Phase 10.17.5+ with feature flags)
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell

# Choose your build:
cargo build --release                      # Minimal (19MB, core only)
cargo build --release --features common    # Common (25MB, templates + PDF)
cargo build --release --features full      # Full (35MB, all tools)

# Set API key
export OPENAI_API_KEY="sk-..."

# Run your first script (embedded kernel mode)
./target/release/llmspell exec '
  local agent = Agent.builder()
    :model("openai/gpt-4o-mini")
    :build()
  print(agent:execute({prompt = "Hello!"}).response)
'

# Use tool CLI directly (Phase 10)
./target/release/llmspell tool list
./target/release/llmspell tool invoke calculator --params '{"expression": "2+2"}'

# Use --trace flag for debugging
./target/release/llmspell --trace debug run script.lua

# Start kernel as daemon (Phase 10)
./target/release/llmspell kernel start --daemon --port 9555

# Install as system service (Phase 10)
./target/release/llmspell kernel install-service
sudo systemctl start llmspell-kernel  # Linux
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist  # macOS
```

## 🆕 Phase 10 Features (Complete)

### Unix Daemon Infrastructure
- **Double-Fork Daemonization**: Proper TTY detachment and session leadership
- **Signal Handling**: SIGTERM/SIGINT → Jupyter shutdown messages, atomic operations
- **PID File Management**: Lifecycle tracking with stale file cleanup
- **Log Rotation**: Size (10MB) and age (7 days) based policies with automatic rotation
- **Graceful Shutdown**: Resource cleanup guarantees on all exit paths
- **systemd/launchd Integration**: Production service deployment on Linux/macOS

### Tool CLI Commands (5 Subcommands)
- **`llmspell tool list`**: Discover all 40+ available tools with filtering
- **`llmspell tool info <name>`**: Get detailed tool documentation
- **`llmspell tool invoke <name> --params <json>`**: Direct tool execution
- **`llmspell tool search <query>`**: Find tools by keyword
- **`llmspell tool test <name>`**: Validate tool functionality
- **Kernel Message Protocol**: Tools execute in kernel via protocol messages
- **Runtime Discovery**: Automatic tool availability detection based on feature flags

### Fleet Management
- **OS-Level Process Isolation**: Multi-kernel orchestration with true process boundaries
- **Bash Fleet Manager**: `llmspell-fleet` for spawn/stop/list/health operations
- **Python Advanced Monitoring**: psutil integration for detailed metrics
- **Docker Orchestration**: docker-compose.yml for containerized deployments
- **Standard Tooling**: Compatible with ps, kill, docker, systemd workflows
- **Configuration-Driven**: Different configs = different processes

### Enhanced Logging & Observability
- **Rotating Log Files**: Automatic rotation, compression, and retention
- **Structured Tracing**: JSON output support with correlation IDs
- **<1ms Overhead**: Lock-free tracing paths for hot code
- **Multi-Output**: File, stderr, and syslog support
- **Session Tracking**: Full request lifecycle visibility

### Feature Flags (Phase 10.17.5+)
- **Modular Builds**: Choose minimal (19MB), common (25MB), or full (35MB)
- **Optional Dependencies**: Templates, PDF, CSV, Excel, archives, email, database
- **Zero Runtime Overhead**: Compile-time feature selection
- **Backward Compatible**: Existing scripts work with appropriate feature flags

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
| **Tool** | Execute tools (40+ available) | `Tool.invoke("web-search", {query = "..."})` |
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

## 📊 Key Metrics (Phase 10 Actual)

| Operation | Target | Achieved | Status |
|-----------|--------|----------|--------|
| Daemon startup | <2s | 1.8s | ✅ 10% faster |
| Message handling | <5ms | 3.8ms | ✅ 24% faster |
| Signal response | <100ms | 85ms | ✅ 15% faster |
| Tool initialization | <10ms | 7ms | ✅ 30% faster |
| Log rotation | <100ms | 78ms | ✅ 22% faster |
| PID file check | <10ms | 6ms | ✅ 40% faster |
| Memory overhead | <50MB | 42MB | ✅ 16% better |
| Heartbeat latency | <1ms | 0.8ms | ✅ 20% faster |
| Vector search (100K) | <10ms | 8ms | ✅ 20% faster |
| Multi-tenant overhead | <5% | 3% | ✅ 40% better |
| Script timeout | - | - | 5 minutes default |

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
├── llmspell-tools (40+ tools with feature flags)
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
8. **Phase 10 Issues** → [Phase 10 Troubleshooting](troubleshooting-phase10.md) (daemon/fleet)
9. **Optimizing** → [Performance Tuning](performance-tuning.md) (production)
10. **Reference** → [API Docs](api/README.md) (lookup)

## 🆘 Need Help?

- **General Issues?** Check [Troubleshooting](troubleshooting.md)
- **Phase 10 Issues?** See [Phase 10 Troubleshooting](troubleshooting-phase10.md) for daemon, signals, PID, fleet
- **Performance?** See [Performance Tuning](performance-tuning.md) for optimization
- **Questions?** Review [Examples](../../examples/EXAMPLE-INDEX.md)
- **Bugs?** Report on [GitHub](https://github.com/yourusername/rs-llmspell/issues)
- **API Details?** See [Lua API](api/lua/README.md) or [Rust API](api/rust/README.md)
- **Deployment?** See [Service Deployment](service-deployment.md)
- **IDE Setup?** See [IDE Integration](ide-integration.md)

---

**Version 0.10.0** | Phase 10 Complete - Service Integration & IDE Connectivity | [Changelog](../../CHANGELOG.md)