# rs-llmspell

**Production-Ready AI Workflow Orchestration Platform** - Script-driven LLM coordination with RAG at scale

**🚀 Version 0.11.2 - Local LLM Cleanup & Enhancement**

**🔗 Quick Links**: [📘 User Guide](docs/user-guide/) | [🔧 Developer Guide](docs/developer-guide/) | [📚 Examples](examples/) | [🛠️ Scripts](scripts/) | [🚀 Get Started](#-quick-start) | [📖 Release Notes](RELEASE_NOTES_v0.11.0.md) | [🔧 Phase 11a](docs/in-progress/phase-11a-design-doc.md) | [🧹 Phase 11b](docs/in-progress/phase-11b-design-doc.md)

---

> **📝 Note**: rs-llmspell builds upon concepts from numerous open-source projects and owes special acknowledgment to [go-llms](https://github.com/lexlapax/go-llms), which was instrumental in rapidly prototyping early ideas. This Rust implementation supersedes go-llms, leveraging Rust's native compilation and zero-cost abstractions for production-grade performance and safety.

---

## 🌟 Production Ready with Local LLM Enhancement & Cleanup

rs-llmspell v0.11.2 delivers **Phase 11b local LLM cleanup** with single-binary architecture (removed unused llmspell-test), unified profile system (10 builtin TOML files), dual-architecture model support (LLaMA GGUF + T5 Safetensors), and platform-aware GPU detection (Metal/CUDA with graceful CPU fallback). Net code reduction of 120 LOC while adding T5 support, plus **Phase 11a bridge consolidation** with 87% compile speedup (38s→5s bridge-only), 95% security documentation coverage, and API standardization. Run agents entirely on your hardware with zero API keys, no cloud dependencies, and full data privacy—all while maintaining production Unix service infrastructure with daemon mode, tool CLI commands, and modular builds (19-35MB).

## ✨ Key Features

### 🧹 Local LLM Cleanup & Enhancement (NEW in v0.11.2)
- **Single-Binary Architecture**: Removed unused llmspell-test binary (-675 LOC, enforced philosophy)
- **Unified Profile System**: 10 builtin TOML profiles replace 100+ lines CLI mutations
- **Dual-Architecture Models**: LLaMA (GGUF) + T5 (Safetensors) support via ModelArchitecture enum
- **Platform-Aware GPU**: Metal (macOS) + CUDA (Linux) with graceful CPU fallback
- **LocalLLM Fix**: 15/15 globals now injected correctly (was 14/15)
- **Net Code Reduction**: -120 LOC (+755 new, -875 deleted) while adding T5 support
- **Config Consolidation**: 40+ Lua examples updated to modern patterns
- **Model Discovery**: HuggingFace and Ollama URLs added to help text

### 🚀 Bridge Consolidation & Quality Improvements (v0.11.1)
- **87% Compile Speedup**: Bridge-only builds 38s→5s via feature gates (ADR-042)
- **API Standardization**: Tool.execute() unified across all 40+ tools (zero ambiguity)
- **Workflow Introspection**: Agent output collection for debugging multi-step workflows (ADR-043)
- **Documentation Completeness**: Security 40%→95%, environment variables 0%→100% (41+ vars)
- **Code Simplification**: 876 LOC removed (Custom steps cleanup)
- **CI/CD Ready**: 6 deployment patterns documented (GitHub Actions, Docker, systemd)
- **Critical Bug Fixes**: Config global fixed, TOML schema corrected

### 🔒 Local LLM Integration & Privacy-First AI (v0.11.0)
- **100% Offline Inference**: All AI processing on your hardware, no cloud calls, no data leakage
- **Dual-Backend Architecture**: Ollama (100+ models) + Candle (native Rust GGUF inference)
- **Zero API Keys Required**: No OpenAI/Anthropic accounts needed for local models
- **HIPAA/GDPR Ready**: Complete data privacy with air-gap capability
- **HuggingFace Integration**: Automatic model downloads (TinyLlama, Phi-2, Qwen2, custom GGUF)
- **Production Performance**: 40 tok/s throughput, 150ms first token, <5GB memory
- **Complete Model Management**: CLI commands for list, pull, status, info operations

### 📦 Optimized Feature-Based Builds (Phase 10)
- **Minimal 19MB binary** - 43% smaller than before, perfect for containers
- **Choose your features** - Include only what you need (templates, PDF, data tools)
- **Zero runtime overhead** - Feature flags are compile-time only
- **Automatic tool discovery** - Runtime adapts to available features
- **Three preset configurations**: minimal (19MB), common (25MB), full (35MB)

### 🎯 Production Service Infrastructure (Phase 10)
- **Unix Daemon Mode**: Double-fork daemonization with 1.8s startup (10% faster than target)
- **Tool CLI Commands**: 5 subcommands for direct tool access without scripts
- **Fleet Management**: OS-level multi-kernel orchestration with Bash/Python/Docker managers
- **Signal Handling**: SIGTERM/SIGINT → graceful Jupyter shutdown with resource cleanup
- **systemd/launchd Ready**: Production service deployment on Linux/macOS
- **Log Rotation**: Automatic rotation with 78ms performance (22% faster than target)
- **PID Management**: Lifecycle tracking with 6ms validation (40% faster than target)

### 🧠 Complete RAG System (v0.8.0)
- **HNSW Vector Storage**: <8ms search @ 100K vectors, <35ms @ 1M vectors
- **Multi-Tenant Isolation**: StateScope::Custom("tenant:id") with 3% overhead
- **Embedding Pipeline**: OpenAI, Cohere, HuggingFace with 80% cache hit rate
- **RAGPipelineBuilder**: Fluent API for constructing RAG pipelines
- **Hybrid Search**: Vector + keyword with configurable weights

### 🤖 Multi-Agent Orchestration
- Coordinate 2-20+ AI agents in complex workflows
- Sequential, parallel, and conditional execution patterns
- Real-time state sharing between agents
- Automatic error recovery and retry logic

### 🛠️ 40+ Built-in Tools (Modular)
- **Core Tools** (always available): File ops, web search, calculator, HTTP client
- **Common Tools** (`--features common`): Templates (Tera/Handlebars), PDF processing
- **Full Tools** (`--features full`): Excel, CSV, archives, email (SMTP/SES), database (Postgres/MySQL/SQLite)
- **RAG Tools**: pdf-processor, document-chunker, embedding-generator, vector-search
- **Direct CLI Access**: `llmspell tool list|info|invoke|search|test` (v0.10.0)
- All tools run in secure sandboxes with automatic feature detection

### 📦 9 Production Applications
Progressive complexity with RAG capabilities:
- **file-organizer** (2 agents): Smart organization with content analysis
- **knowledge-base** (NEW): RAG-powered knowledge management
- **research-assistant** (NEW): Document analysis with citations
- **webapp-creator** (20 agents): Full-stack generation with pattern library

### 🔒 Enterprise Security & Multi-Tenancy
- **Complete Tenant Isolation**: Zero cross-tenant data leakage
- **Row-Level Security**: Policy-based access control
- **Mandatory Sandboxing**: All tool executions isolated
- **Resource Boundaries**: Per-tenant CPU, memory, I/O limits
- **Audit Compliance**: Complete trails with event correlation

### ⚡ Blazing Performance (v0.11.0)
**Phase 11 Local LLM Performance** (All targets exceeded by 25-33%):
- **Candle First Token**: 150ms (25% faster than 200ms target)
- **Candle Throughput**: 40 tok/s (33% faster than 30 tok/s target)
- **Candle Memory**: ~400MB/2K tokens (8x better than 5GB target)
- **Model Downloads**: 638MB in 20s (HuggingFace with progress tracking)
- **Ollama Integration**: Production-ready with 17+ models tested

**Phase 10 Service Infrastructure** (All targets exceeded by 10-40%):
- **Binary size**: 19MB minimal (43% smaller), 25MB common (26% smaller), 35MB full
- **Daemon startup**: 1.8s (10% faster than <2s target)
- **Message handling**: 3.8ms (24% faster than <5ms target)
- **Tool initialization**: 7ms (30% faster than <10ms target)
- **Vector search**: 8ms @ 100K (20% faster), 35ms @ 1M (30% faster)
- **Multi-tenant overhead**: 3% (40% better than <5% target)
- **Test coverage**: 800+ tests (100% pass rate)

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS 15.7 (ARM64) | ✅ Fully Tested | All features including local LLM (Ollama + Candle), daemon, tool CLI, fleet management |
| Linux | ⏳ Testing Pending | Expected to work, formal testing in progress |
| Windows | ⏳ Testing Pending | Expected to work, formal testing in progress |

> **Note**: v0.11.0 has been thoroughly tested on macOS 15.7 (Darwin 24.6.0, ARM64) with complete local LLM integration (Ollama + Candle backends), daemon infrastructure, tool CLI commands, and fleet management. Linux and Windows testing is in progress.

## Quick Example

```lua
-- Create an agent with local model (NEW in v0.11.0)
local local_agent = Agent.create({
    model = "local/llama3.1:8b@ollama",  -- 100% offline, zero API keys
    system_prompt = "You are a helpful assistant"
})

-- Or use cloud models (OpenAI, Anthropic, etc.)
local cloud_agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant"
})

local tool = Tool.get("file_operations")
local content = tool:execute({
    operation = "read",
    path = "data.txt"
})

local response = local_agent:execute({
    prompt = "Summarize this content: " .. content.output
})

Logger.info("Summary", {result = response.output})
```

### Hook and Event Example (v0.4.0)

```lua
-- Register a hook to monitor agent execution
Hook.register("agent:before_execution", function(context)
    Logger.info("Agent starting", {agent_id = context.agent_id})
    return {continue_execution = true}
end)

-- Subscribe to error events
Event.subscribe("*.error", function(event)
    Alert.send("Error occurred", event.payload)
end)

-- Emit custom events
Event.emit({
    event_type = "custom:analysis_complete",
    payload = {duration_ms = 250, records = 1000}
})
```

### Persistent State Example (v0.5.0)

```lua
-- Save state with automatic persistence
State.save("agent:gpt-4", "conversation_history", messages)

-- Load state with fallback
local config = State.load("global", "app_config") or {theme = "dark"}

-- Backup state before critical operations
local backup_id = State.backup({description = "Before update"})

-- Perform migration
State.migrate({
    from_version = 1,
    to_version = 2,
    transformations = {
        {field = "old_field", transform = "copy", to = "new_field"}
    }
})
```

### Session Management Example (v0.6.0)

```lua
-- Create a session for long-running interactions
local session = Session.create({
    name = "research_session",
    max_duration = 3600  -- 1 hour
})

-- Store artifacts within the session
local artifact_id = Artifact.store(session.id, "analysis_result", {
    summary = "Market analysis complete",
    data = {revenue = 1000000, growth = 0.15}
})

-- Retrieve artifacts later
local artifact = Artifact.get(session.id, artifact_id)

-- Suspend and resume sessions
Session.suspend(session.id)
-- ... later ...
Session.resume(session.id)

-- List all artifacts in a session
local artifacts = Artifact.list(session.id)
```

### RAG Example (v0.8.0)

```lua
-- Ingest documents into RAG
RAG.ingest({
    content = "LLMSpell is a production-ready AI orchestration platform",
    metadata = {source = "documentation", category = "overview"}
})

-- Search with hybrid retrieval
local results = RAG.search("What is LLMSpell?", {
    max_results = 5,
    hybrid_weights = {vector = 0.7, keyword = 0.3}
})

-- Multi-tenant RAG
local tenant_scope = "tenant:customer_123"
RAG.ingest({
    content = "Customer-specific knowledge",
    metadata = {tenant = "customer_123"}
}, tenant_scope)

-- Only returns tenant's data
local tenant_results = RAG.search("knowledge", {
    scope = tenant_scope
})
```

## 🚀 Quick Start

### Easy Installation (Choose Your Build)

```bash
# Clone repository
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell

# Choose your build size:
cargo build --release                   # Minimal: 19MB (core tools only)
cargo build --release --features common # Common: 25MB (+templates, PDF)
cargo build --release --features full   # Full: 35MB (all features)

# Use the friendly launcher with setup wizard
./scripts/utilities/llmspell-easy.sh

# Run your first application!
./scripts/utilities/llmspell-easy.sh file-organizer
```

💡 **Advanced User Tip**: You can also run applications directly with the `llmspell` binary:
`llmspell app run file-organizer`

💡 **Build Size Tip**: Start with minimal (19MB) for production or common (25MB) for development. See [Installation Options](#-installation-options) below for details.

### Try Production Applications

```bash
# Organize messy files (2 agents, <30s)
./scripts/utilities/llmspell-easy.sh file-organizer

# Generate content (4 agents, <1min)
./scripts/utilities/llmspell-easy.sh content-creator

# Review code (8 agents, <2min)
./scripts/utilities/llmspell-easy.sh code-review-assistant

# Build a web app (20 agents, 4.5min)
./scripts/utilities/llmspell-easy.sh webapp-creator
```

## 📦 Installation Options

rs-llmspell supports flexible installation via Cargo feature flags to control binary size and dependencies:

### Minimal Installation (19MB)
Includes core functionality with Lua scripting and essential tools:
```bash
cargo build --release --bin llmspell
# Or explicitly:
cargo build --release --bin llmspell --no-default-features --features lua
```

### Common Installation (recommended for most users)
Adds template engines and PDF processing (~25MB):
```bash
cargo build --release --bin llmspell --features common
```

### Full Installation
Includes all optional components - CSV/Parquet, Excel, archives, email, database support (~35MB):
```bash
cargo build --release --bin llmspell --features full
```

### Custom Feature Selection
Mix and match features based on your needs:
```bash
# Example: Just add template support
cargo build --release --features templates

# Example: Add CSV/Parquet and archives
cargo build --release --features csv-parquet,archives
```

**Available Features:**
- `templates` - Tera and Handlebars template engines
- `pdf` - PDF document processing
- `csv-parquet` - Apache Arrow/Parquet support for data analysis
- `excel` - Excel file reading/writing
- `json-query` - JQ-style JSON queries
- `archives` - ZIP, TAR, GZ archive handling
- `email` - Email sending via SMTP
- `email-aws` - Email via AWS SES
- `database` - SQL database connectivity (PostgreSQL, MySQL, SQLite)

## 📊 Comprehensive Feature Set

### Core Capabilities
- **Local LLM Integration**: Offline inference with Ollama (100+ models) and Candle (native GGUF) - zero API keys (v0.11.0)
- **Model Management CLI**: Direct model operations via `llmspell model list|pull|status|info` (v0.11.0)
- **40+ Production Tools**: File operations, web scraping, data processing, system utilities (modular with feature flags)
- **Tool CLI Commands**: Direct tool access via `llmspell tool list|info|invoke|search|test` (v0.10.0)
- **Unix Daemon Mode**: Production service deployment with systemd/launchd (v0.10.0)
- **Fleet Management**: Multi-kernel orchestration with OS-level process isolation (v0.10.0)
- **Multi-Agent Coordination**: Orchestrate 2-20+ LLM agents with different models and roles
- **Workflow Patterns**: Sequential, parallel, conditional, and recursive execution
- **Session Management**: Long-running sessions with suspend/resume, artifacts, and replay
- **State Persistence**: RocksDB/SQLite backends with migrations and atomic backups
- **Hook System**: 40+ extensibility points with <1% performance overhead
- **Event Bus**: Cross-language event propagation at >90K events/sec
- **Security Sandbox**: Mandatory isolation for all tool executions
- **Multi-Provider**: OpenAI, Anthropic, Ollama, Candle, and 10+ other providers

### RAG & Vector Capabilities (Phase 8)
- **Production HNSW**: 1M+ vector search with <10ms latency and >95% recall accuracy
- **Multi-Tenant Vector Storage**: Complete tenant isolation with StateScope boundaries
- **Document Processing**: Semantic, fixed-size, and recursive chunking strategies
- **Embedding Management**: OpenAI, local models with intelligent caching and fallback
- **Hybrid Retrieval**: Vector similarity combined with keyword search and reranking
- **Conversation Memory**: Session-aware RAG with context retention across interactions
- **Metadata Filtering**: Rich queries with inverted indices for complex search patterns

### Enterprise Features
- **Multi-Tenant Architecture**: Complete tenant isolation with resource quotas and billing
- **Advanced Access Control**: Policy-based authorization with row-level security filters
- **Audit Logging**: Complete execution history with replay capability and security correlation
- **Cost Tracking**: Per-agent, per-workflow, and per-tenant cost monitoring with usage analytics
- **Rate Limiting**: Global, per-resource, and per-tenant rate limits with intelligent throttling
- **Error Recovery**: Automatic retry with exponential backoff and circuit breaker patterns
- **Resource Limits**: CPU, memory, storage, and token constraints with real-time enforcement
- **Compliance**: Data retention policies, PII protection, and regulatory compliance features

## 🎯 Roadmap

### Current: v0.11.0 - Local LLM Integration ✅
- **Phase 11 Completed**: Privacy-first local inference with Ollama + Candle
- Dual-backend architecture: Ollama (100+ models) + Candle (native GGUF)
- Complete model management: list, pull, status, info CLI commands
- Zero API keys required: 100% offline AI workflows
- HuggingFace integration: Automatic model downloads
- 21 crates with 800+ tests total
- All 7 performance targets exceeded by 25-33%

### Next: Phase 12 - Multi-Agent Orchestration (Q4 2024)
- **Complex Agent Collaboration**: Multi-agent workflows with state sharing
- **Role-Based Agents**: Specialized agents with distinct capabilities
- **Agent Communication**: Direct agent-to-agent messaging
- **Workflow Templates**: Pre-built patterns for common orchestration scenarios
- **Performance Optimization**: Parallel agent execution and batching
- **Monitoring & Observability**: Agent execution tracing and debugging

### Upcoming Feature Additions (Phases 13-18)

#### Near Term (2025)
- **Phase 13**: Adaptive Memory System - Working memory, episodic memory, semantic knowledge graphs
- **Phase 14**: Model Context Protocol (MCP) - External tool integration
- **Phase 15**: Security Hardening - Advanced threat protection

#### Medium Term (2025-2026)
- **Phase 16**: Production Orchestration - Kubernetes, autoscaling, monitoring
- **Phase 17**: Distributed Execution - Multi-node orchestration
- **Phase 18**: Cloud Platform - Managed service offering

*Note: From v0.11.0 onwards, infrastructure is production-stable with local LLM support. Updates add features without breaking existing functionality. Feature flags ensure backward compatibility.*

## Documentation

- **[Quick Start Guide](docs/user-guide/getting-started.md)** - Get started in 5 minutes
- **[Documentation Hub](docs/README.md)** - Complete documentation index (10 user guides, 6 developer guides, 13 technical docs)
- **[Local LLM Guide](docs/user-guide/local-llm.md)** - Ollama + Candle setup, model management, privacy-first workflows ⭐ NEW
- **[Service Deployment](docs/user-guide/service-deployment.md)** - systemd/launchd deployment with daemon mode
- **[IDE Integration](docs/user-guide/ide-integration.md)** - VS Code, Jupyter Lab, vim/neovim setup
- **[Feature Flags Migration](docs/developer-guide/feature-flags-migration.md)** - Modular builds guide
- **[RAG System Guide](docs/technical/rag-system-guide.md)** - Complete RAG documentation
- **[Examples](examples/)** - 60+ working examples with RAG patterns, local LLM, and tool CLI

## Scripts & Automation

- **[Scripts Overview](scripts/)** - All automation tools
  - **[Quality & CI](scripts/quality/)** - Code quality, CI/CD pipelines
  - **[Testing](scripts/testing/)** - Test execution, coverage analysis
  - **[Utilities](scripts/utilities/)** - Helper tools, easy launcher
  - **[Fleet Management](scripts/fleet/)** - Kernel orchestration, monitoring

## Development

```bash
# Run quality checks before committing
./scripts/quality/quality-check-minimal.sh

# Run full test suite
./scripts/testing/test-by-tag.sh unit

# See development guide for more
cat docs/developer-guide/README.md
```

- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute
- **[Developer Documentation](docs/developer-guide/)** - Development setup and workflows
- **[Architecture](docs/technical/master-architecture-vision.md)** - System architecture

## Project Links

- **Issues**: [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lexlapax/rs-llmspell/discussions)
- **Progress Tracking**: [Phase Status](docs/in-progress/)

## License

This project is licensed under the Apache License, Version 2.0. See [LICENSE-APACHE](LICENSE-APACHE) for details.

---

**🚀 v0.11.0 Released**: Local LLM Integration & Privacy-First AI with dual backends (Ollama + Candle), 100% offline inference, zero API keys, complete model management CLI, and HuggingFace integration. All 7 performance targets exceeded by 25-33%. Phase 11 completed in 4.5 days (77% faster than planned). See [Release Notes](RELEASE_NOTES_v0.11.0.md) for complete details.