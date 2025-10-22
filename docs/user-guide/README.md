# LLMSpell User Guide

**Learn to build powerful LLM-driven applications with production-ready rs-llmspell**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [Examples](../../examples/) | [API Reference](api/)

---

## Overview

> **📚 Central Hub**: Your starting point for all LLMSpell documentation. Everything you need is organized into 12 essential documents, plus comprehensive API references for both Lua and Rust. Now with Unix daemon infrastructure, tool CLI commands, fleet management, feature flags, and 10 production-ready AI agent templates!

**Version**: 0.12.0 | **Status**: Phase 12 Complete - Production-Ready AI Agent Templates | **Last Updated**: October 2025

## 📖 Essential Documentation (12 Files)

### 1. [Getting Started](getting-started.md)
**Quick start in under 10 minutes**
- Installation and setup with feature flags
- Progressive learning path (6 examples)
- RAG setup and first knowledge base
- Running your first daemon with tool CLI

### 2. [Core Concepts](concepts.md)
**Understand LLMSpell architecture including Phase 10 & 11 features**
- Component model (BaseAgent trait)
- Agents, Tools (40+ with feature flags), Workflows
- RAG (Retrieval-Augmented Generation) ⭐
- Vector Storage & HNSW algorithm ⭐
- Multi-Tenancy with resource quotas ⭐
- Integrated Kernel Architecture (Phase 10) ⭐
- Local LLM Integration (Ollama + Candle, Phase 11) ⭐
- Tool CLI and direct tool invocation ⭐
- Fleet management and process isolation ⭐
- State management and sessions (unified in kernel)
- Hooks, Events, and Security model

### 3. [Configuration](configuration.md)
**Complete configuration guide including daemon and fleet setup**
- LLM providers (OpenAI, Anthropic, Ollama, Groq, local backends)
- RAG Configuration (HNSW, embeddings, chunking) ⭐
- Multi-Tenancy (isolation, quotas, billing) ⭐
- Daemon configuration (PID files, log rotation, signals) ⭐
- Feature flags for modular builds (19-35MB) ⭐
- Fleet management and multi-kernel orchestration ⭐
- State & Sessions persistence
- Security settings and deployment profiles
- Environment variables

### 4. [Security & Permissions](security-and-permissions.md) ⭐ Phase 11a.13
**Comprehensive security sandbox guide for safe tool execution**
- Three-level security model (Safe/Restricted/Privileged)
- Sandbox system (FileSandbox, NetworkSandbox, IntegratedSandbox)
- Permission configuration (network access, process execution, file system)
- Tool-specific security settings via config.toml
- Troubleshooting permission errors with solutions
- Security best practices (least privilege, allowlists, monitoring)
- Common scenarios (enable curl, API access, file permissions)

### 5. [Local LLM Integration](local-llm.md) ⭐ Phase 11
**Use local LLM models via Ollama or Candle**
- Quick start for Ollama and Candle
- Model management (list, pull, info, status)
- Configuration for both backends
- Performance characteristics and comparison
- 6 troubleshooting scenarios
- 4 complete example scripts

### 6. [AI Agent Templates](templates/README.md) ⭐ Phase 12
**Production-ready AI workflows - Installation to productive AI in <5 minutes**
- 10 built-in templates (6 base + 4 advanced)
- Research Assistant: 4-phase research workflow (web search, analysis, synthesis, validation)
- Interactive Chat, Data Analysis, Code Generator, Document Processor, Workflow Orchestrator
- Code Review, Content Generation, File Classification, Knowledge Management (advanced)
- CLI commands: `template list|info|exec|search|schema`
- Lua API: Template global (16th of 18 globals) with 6 methods
- 20-50x faster than performance targets
- Complete user guides for all 10 templates

### 7. [Service Deployment](service-deployment.md) ⭐ Phase 10
**Production deployment with system services**
- systemd deployment (Linux)
- launchd deployment (macOS)
- Daemon management (double-fork, TTY detachment, signals)
- PID file handling and lifecycle tracking
- Log rotation with size/age policies
- Fleet management for multi-kernel deployments
- Security best practices

### 8. [IDE Integration](ide-integration.md) ⭐ Phase 10
**Connect your IDE to LLMSpell kernel**
- VS Code setup with Jupyter & DAP
- Jupyter Lab configuration
- vim/neovim LSP integration
- Debug Adapter Protocol (10 DAP commands)
- Multi-client support
- Connection file format and kernel discovery

### 9. [API Documentation](api/README.md)
**Comprehensive API reference**
- **[Lua API](api/lua/README.md)** - All 18 globals with 200+ methods
- **[Rust API](api/rust/README.md)** - 18 crates with traits, builders, and extension guide
- Unified `llmspell-kernel` crate with daemon, state, sessions, and debugging
- New `llmspell-templates` crate with Template trait and registry
- Feature flags documentation for modular builds

### 10. [Troubleshooting](troubleshooting.md)
**Solutions to common problems**
- Common issues and fixes
- Debugging techniques
- Error messages explained
- Daemon and service troubleshooting
- Feature flag issues

### 11. [Phase 10 Troubleshooting](troubleshooting-phase10.md) ⭐ Phase 10
**Phase 10 specific issues**
- Daemon startup problems
- Signal handling issues
- PID file conflicts
- Log rotation configuration
- Fleet management debugging
- Tool CLI troubleshooting

### 12. [Performance Tuning](performance-tuning.md) ⭐ Phase 10
**Optimize for production**
- Daemon performance tuning
- Memory optimization
- Log rotation settings
- Fleet scaling strategies
- HNSW parameter tuning
- Multi-tenant resource allocation

### 13. [Examples](../../examples/EXAMPLE-INDEX.md)
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

# Use AI Agent Templates (Phase 12) - Instant productive AI ⭐
./target/release/llmspell template list
./target/release/llmspell template exec research-assistant \
  --param topic="Rust async programming" \
  --param max_sources=10

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

## 🆕 Phase 12 Features (Complete) ⭐

### AI Agent Templates - Turn-key Workflows
- **10 Built-in Templates**: 6 base templates + 4 advanced workflows
- **Research Assistant**: 4-phase research workflow with web search, analysis, synthesis, validation
- **Interactive Chat**: Session-based conversation with context and memory
- **Data Analysis**: CSV/Excel/JSON analysis with visualizations
- **Code Generator**: Multi-language code generation with tests
- **Document Processor**: PDF/OCR extraction and transformation
- **Workflow Orchestrator**: Custom agent/tool/template composition
- **Code Review**: Multi-aspect code analysis (structure, security, performance, best practices)
- **Content Generation**: Quality-driven iterative content creation
- **File Classification**: Scan-classify-act pattern for file organization
- **Knowledge Management**: Document ingestion, RAG integration, query interface

### Template CLI Commands (5 Subcommands)
- **`llmspell template list`**: Discover all 10 built-in templates with category filtering
- **`llmspell template info <name>`**: Get detailed template documentation and parameters
- **`llmspell template exec <name> --param key=value`**: Direct template execution
- **`llmspell template search <query>`**: Find templates by keyword
- **`llmspell template schema <name>`**: Get parameter schema and validation rules

### Template Lua API (Template Global - 16th of 18 Globals)
- **`Template.list([category])`**: List templates with optional category filter
- **`Template.info(name, [with_schema])`**: Get template metadata and optional schema
- **`Template.execute(name, params)`**: Execute template with parameters
- **`Template.search(query, [category])`**: Search templates by keyword
- **`Template.schema(name)`**: Get template configuration schema
- **`Template.estimate_cost(name, params)`**: Pre-execution cost estimation

### Template System Architecture
- **TemplateRegistry**: DashMap-based concurrent template storage with Arc sharing
- **Template Trait**: Metadata, config schema, cost estimation, async execute
- **ExecutionContext**: Builder pattern for infrastructure (Tools, Agents, Workflows, RAG, Providers)
- **Parameter Validation**: Declarative schema with constraints (required, type, min/max, pattern, etc.)
- **4-Layer Bridge**: Core → TemplateBridge → TemplateGlobal → Lua scripts
- **Performance**: 10-50x faster than targets (0.5ms list, 2ms execute overhead, 0.1ms validation)

### Quality Metrics (Phase 12)
- **149 Tests**: 122 unit + 27 integration, 100% passing
- **Zero Warnings**: Clippy clean with `-D warnings` across workspace
- **>90% Coverage**: All modules tested with mocks
- **3,655 Lines Docs**: Complete user guides for all 10 templates
- **Production Quality**: Format 100%, API docs >95%, comprehensive architecture docs

## 🧩 Available Globals (18)

All globals are pre-injected - no `require()` needed!

| Global | Purpose | Example |
|--------|---------|---------|
| **Agent** | LLM interactions | `Agent.builder():model("openai/gpt-4"):build()` |
| **Tool** | Execute tools (40+ available) | `Tool.execute("web-search", {query = "..."})` |
| **Workflow** | Orchestration | `Workflow.sequential({steps = {...}})` |
| **Template** | AI workflow templates ⭐ Phase 12 | `Template.execute("research-assistant", {topic = "..."})` |
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

### Use AI Agent Templates ⭐ Phase 12
```lua
-- Research a topic
local result = Template.execute("research-assistant", {
    topic = "Rust async programming",
    max_sources = 10,
    enable_validation = true
})
print(result.result)

-- Access generated artifacts
for _, artifact in ipairs(result.artifacts) do
    print("Generated: " .. artifact.filename)
end

-- Or use CLI directly
-- llmspell template exec research-assistant --param topic="..." --param max_sources=10
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

## 📊 Key Metrics (Phase 12 Actual)

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
| **Template list** ⭐ | <10ms | 0.5ms | ✅ 20x faster |
| **Template execute overhead** ⭐ | <100ms | 2ms | ✅ 50x faster |
| **Parameter validation** ⭐ | <5ms | 0.1ms | ✅ 50x faster |
| Script timeout | - | - | 5 minutes default |

## 🏗️ Architecture Overview

### Crate Structure (18 Total)
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

Execution Layer (6 crates) ⭐ Phase 12
├── llmspell-agents
├── llmspell-tools (40+ tools with feature flags)
├── llmspell-workflows
├── llmspell-templates (10 built-in templates) ⭐ NEW
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
3. **Quick Win** → [AI Agent Templates](templates/README.md) ⭐ (<5 min to productive AI)
4. **Building** → [Examples](../../examples/EXAMPLE-INDEX.md) (hands-on)
5. **Configuring** → [Configuration](configuration.md) (as needed)
6. **Deploying** → [Service Deployment](service-deployment.md) (production)
7. **IDE Setup** → [IDE Integration](ide-integration.md) (development)
8. **Debugging** → [Troubleshooting](troubleshooting.md) (when stuck)
9. **Phase 10 Issues** → [Phase 10 Troubleshooting](troubleshooting-phase10.md) (daemon/fleet)
10. **Optimizing** → [Performance Tuning](performance-tuning.md) (production)
11. **Reference** → [API Docs](api/README.md) (lookup)

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

**Version 0.12.0** | Phase 12 Complete - Production-Ready AI Agent Templates | [Release Notes](../../RELEASE_NOTES_v0.12.0.md) | [Changelog](../../CHANGELOG.md)