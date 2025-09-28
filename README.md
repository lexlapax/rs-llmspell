# rs-llmspell

**Production-Ready AI Workflow Orchestration Platform** - Script-driven LLM coordination with RAG at scale

**🚀 Version 0.9.0 - Interactive Kernel & Debugging Infrastructure Complete**

**🔗 Quick Links**: [📘 User Guide](docs/user-guide/) | [🔧 Developer Guide](docs/developer-guide/) | [📚 Examples](examples/) | [🛠️ Scripts](scripts/) | [🚀 Get Started](#-quick-start) | [📖 Release Notes](RELEASE_NOTES_v0.9.0.md)

---

> **📝 Note**: rs-llmspell builds upon concepts from numerous open-source projects and owes special acknowledgment to [go-llms](https://github.com/lexlapax/go-llms), which was instrumental in rapidly prototyping early ideas. This Rust implementation supersedes go-llms, leveraging Rust's native compilation and zero-cost abstractions for production-grade performance and safety.

---

## 🌟 Production Ready with Interactive Kernel Architecture

rs-llmspell v0.9.0 delivers **unified kernel architecture** with interactive REPL capabilities, comprehensive debugging infrastructure, and multi-protocol support. Build sophisticated AI applications with Jupyter integration, DAP debugging, real-time tracing, and 100% validated application suite across all complexity layers.

## ✨ Key Features

### 🎯 Unified Kernel & Debug Infrastructure (v0.9.0)
- **Interactive REPL**: Jupyter 5-channel protocol support
- **Debug Capabilities**: DAP bridge with breakpoints and stepping
- **Global IO Runtime**: Eliminates runtime context issues
- **Session Management**: Complete lifecycle with artifacts
- **Comprehensive Tracing**: -3.99% overhead (performance gain!)

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

### 🛠️ 37+ Built-in Tools
- File operations, web search, data processing
- **NEW**: pdf-processor, document-chunker, embedding-generator
- **NEW**: vector-search, similarity-calculator, web-scraper
- JSON/YAML manipulation, text transformation
- All tools run in secure sandboxes

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

### ⚡ Blazing Performance (v0.9.0)
- **Message handling**: 3.8ms (24% faster than target)
- **Tracing overhead**: -3.99% (performance improved!)
- **Application validation**: 100% success rate
- **Vector search**: <8ms @ 100K vectors, <35ms @ 1M vectors
- **Embedding generation**: 45ms with caching (80% hit rate)
- **Multi-tenant overhead**: 3% (40% better than target)
- **Ingestion throughput**: 1.8K vectors/sec
- Agent creation: 2-3ms
- Tool initialization: 1-2ms
- State operations: <1ms
- WebApp generation: 4.5 minutes (20 agents)

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS 15.7 (ARM64) | ✅ Fully Tested | All features including RAG working |
| Linux | ⏳ Testing Pending | Expected to work, formal testing in progress |
| Windows | ⏳ Testing Pending | Expected to work, formal testing in progress |

> **Note**: v0.9.0 has been thoroughly tested on macOS 15.7 (Darwin 24.6.0, ARM64) with complete kernel architecture and debugging support. Linux and Windows testing is in progress.

## Quick Example

```lua
-- Create an agent and use tools
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are a helpful assistant"
})

local tool = Tool.get("file_operations")
local content = tool:execute({
    operation = "read",
    path = "data.txt"
})

local response = agent:execute({
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

### Easy Installation (Recommended)

```bash
# Clone and build
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell
cargo build --release

# Use the friendly launcher with setup wizard
./scripts/llmspell-easy.sh

# Run your first application!
./scripts/llmspell-easy.sh file-organizer
```

### Try Production Applications

```bash
# Organize messy files (2 agents, <30s)
./scripts/llmspell-easy.sh file-organizer

# Generate content (4 agents, <1min)
./scripts/llmspell-easy.sh content-creator

# Review code (8 agents, <2min)
./scripts/llmspell-easy.sh code-review-assistant

# Build a web app (20 agents, 4.5min)
./scripts/llmspell-easy.sh webapp-creator
```

## 📊 Comprehensive Feature Set

### Core Capabilities
- **34 Production Tools**: File operations, web scraping, data processing, system utilities
- **Multi-Agent Coordination**: Orchestrate 2-20+ LLM agents with different models and roles
- **Workflow Patterns**: Sequential, parallel, conditional, and recursive execution
- **Session Management**: Long-running sessions with suspend/resume, artifacts, and replay
- **State Persistence**: RocksDB/SQLite backends with migrations and atomic backups
- **Hook System**: 40+ extensibility points with <1% performance overhead
- **Event Bus**: Cross-language event propagation at >90K events/sec
- **Security Sandbox**: Mandatory isolation for all tool executions
- **Multi-Provider**: OpenAI, Anthropic, Ollama, and custom providers

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

### Current: v0.8.0 - RAG & Multi-Tenancy ✅
- **Phase 8 Completed**: Production RAG with HNSW vector search
- Multi-tenant isolation with StateScope::Custom
- 3 new crates: llmspell-rag, llmspell-storage, llmspell-tenancy
- 37+ tools with 11 new RAG/data tools
- <8ms vector search @ 100K vectors achieved

### Next: Phase 9 - Enhanced Observability (Q1 2025)
- **OpenTelemetry Integration**: Distributed tracing and metrics
- **Performance Analytics**: Cost tracking and optimization
- **Advanced Debugging**: Profilers and diagnostic tools
- **Developer Experience**: Enhanced error messages and documentation

### Upcoming Feature Additions (Phases 10-16)

#### Near Term (2025)
- **Phase 10**: Advanced Workflow Patterns - Complex orchestration
- **Phase 11**: LLM Router - Intelligent model selection
- **Phase 12**: JavaScript Bridge - Full JS/TypeScript support

#### Medium Term (2025-2026)
- **Phase 13**: IDE Plugins - VSCode and IntelliJ integration
- **Phase 14**: Cloud Platform - Managed service offering
- **Phase 15**: Mobile SDKs - iOS and Android libraries
- **Phase 16**: Python Bridge - Complete Python integration

*Note: From v0.8.0 onwards, infrastructure is stable. Updates will add features without breaking existing functionality.*

## Documentation

- **[Quick Start Guide](docs/user-guide/getting-started.md)** - Get started in 5 minutes
- **[Documentation Hub](docs/README.md)** - Complete documentation index
- **[RAG System Guide](docs/technical/rag-system-guide.md)** - Complete RAG documentation
- **[Examples](examples/)** - 60+ working examples with RAG patterns

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

**🚀 v0.8.0 Released**: Complete RAG & Multi-Tenant Vector Storage with <8ms search @ 100K vectors, 80% embedding cache hit rate, and 9 production applications. See [Release Notes](RELEASE_NOTES_v0.8.0.md) for details.