# rs-llmspell

**Production-Ready AI Workflow Orchestration Platform** - Script-driven LLM coordination at scale

**🎉 Version 0.7.0 - First MVP Release**

**🔗 Quick Links**: [📘 User Guide](docs/user-guide/) | [🔧 Developer Guide](docs/developer-guide/) | [📚 Examples](examples/) | [🚀 Get Started](#-quick-start)

---

## 🌟 Production Ready

rs-llmspell v0.7.0 is our **first production-ready MVP**, capable of orchestrating complex AI workflows with enterprise-grade reliability. Successfully validated through WebApp Creator - orchestrating **20 AI agents** to generate complete applications in **4.5 minutes**.

## ✨ Key Features

### 🤖 Multi-Agent Orchestration
- Coordinate 2-20+ AI agents in complex workflows
- Sequential, parallel, and conditional execution patterns
- Real-time state sharing between agents
- Automatic error recovery and retry logic

### 🧠 RAG & Vector Search (Phase 8)
- **Production HNSW**: <10ms vector search across 1M+ vectors with >95% recall
- **Multi-Tenant RAG**: Complete data isolation with tenant-scoped operations
- **Intelligent Chunking**: Semantic, fixed-size, and recursive document processing
- **Hybrid Retrieval**: Vector similarity + keyword search with reranking
- **Session-Aware**: Conversation memory and cross-session context retention

### 🛠️ 34+ Built-in Tools
- File operations, web search, data processing
- JSON/YAML manipulation, text transformation
- API testing, webhook calling, database connectivity
- All tools run in secure sandboxes

### 📦 7 Production Applications (RAG-Enhanced)
Progressive complexity from Universal to Expert:
- **file-organizer** (2 agents): Smart file organization with content analysis
- **content-creator** (4 agents): Multi-format content generation  
- **communication-manager** (5 agents): Business communication with template knowledge
- **code-review-assistant** (8 agents): Code review with codebase RAG knowledge
- **webapp-creator** (20 agents): AI-driven development with pattern library knowledge

### 🔒 Enterprise Security & Multi-Tenancy
- **Multi-Tenant Isolation**: Complete data separation with zero cross-tenant access
- **Advanced Access Control**: Policy-based authorization with row-level security
- **Mandatory Sandboxing**: All tool executions in isolated environments
- **Resource Boundaries**: Configurable CPU, memory, and I/O limits per tenant
- **Audit Compliance**: Complete audit trails with security event correlation

### ⚡ Blazing Performance
- Agent creation: **2-3ms** (94% faster than target)
- Tool initialization: **1-2ms**
- State operations: **<1ms**
- **Vector search: <10ms** across 1M+ vectors (Phase 8)
- **RAG retrieval: <5ms** with context assembly (Phase 8)
- **Multi-tenant overhead: <3%** per tenant (Phase 8)
- WebApp generation: **4.5 minutes** for 20 agents

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS 15.7 (ARM64) | ✅ Fully Tested | All features working on Apple Silicon |
| Linux | ⏳ Testing Pending | Expected to work, formal testing in progress |
| Windows | ⏳ Testing Pending | Expected to work, formal testing in progress |

> **Note**: v0.7.0 has been thoroughly tested on macOS 15.7 (Darwin 24.6.0, ARM64). Linux and Windows testing is in progress. Please report any platform-specific issues.

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
- Multi-tenant isolation with complete data separation
- Advanced access control and security policies
- 7 RAG-enhanced applications
- <10ms vector search across 1M+ vectors

### Upcoming Feature Additions (Phases 9-16)

#### Near Term (Q4 2025)
- **Phase 9**: Visual Workflow Designer - Drag-and-drop UI for complex workflows
- **Phase 10**: Distributed Execution - Multi-node orchestration with load balancing
- **Phase 11**: LLM Router - Intelligent model selection and cost optimization

#### Medium Term (Q4 2025)
- **Phase 11**: Fine-tuning Integration - Custom model training
- **Phase 12**: JavaScript Bridge - Full JS/TypeScript support
- **Phase 13**: IDE Plugins - VSCode and IntelliJ integration

#### Long Term (2026)
- **Phase 14**: Cloud Platform - Managed service offering
- **Phase 15**: Mobile SDKs - iOS and Android libraries
- **Phase 16**: Python Bridge - Complete Python integration

*Note: From v0.7.0 onwards, updates will primarily add features rather than breaking existing functionality.*

## Documentation

- **[Quick Start Guide](docs/user-guide/getting-started.md)** - Get started in 5 minutes
- **[Documentation Hub](docs/README.md)** - Complete documentation index
- **[Tool Reference](docs/user-guide/tool-reference.md)** - All 34 tools documented
- **[Examples](examples/)** - Working code examples for all features

## Development

```bash
# Run quality checks before committing
./scripts/quality-check-minimal.sh

# Run full test suite
cargo test --workspace

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

**🎉 v0.8.0 Released**: Production RAG & Multi-Tenancy with HNSW vector search, complete tenant isolation, and 7 enhanced applications. See [Release Notes](RELEASE_NOTES_v0.8.0.md) for details.