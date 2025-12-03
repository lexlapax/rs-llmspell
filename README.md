# rs-llmspell

**Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust

**🚀 Version 0.13.1 - Production Storage Infrastructure & Documentation Consolidation**

**🔗 Quick Links**: [📘 Documentation Hub](docs/) | [🚀 Quick Start](#-quick-start) | [🎯 What This Is](#what-is-rs-llmspell) | [🏗️ Experiment → Production](#from-experiment-to-production) | [📖 Release Notes](RELEASE_NOTES_v0.13.1.md) | [🛠️ Examples](examples/) | [🔧 Contributing](CONTRIBUTING.md)

---

> **📝 Note**: rs-llmspell builds upon concepts from numerous open-source projects and owes special acknowledgment to [go-llms](https://github.com/lexlapax/go-llms), which was instrumental in rapidly prototyping early ideas. This Rust implementation supersedes go-llms, leveraging Rust's native compilation and zero-cost abstractions for experimental velocity with production-ready foundations.

---

## What is rs-llmspell?

rs-llmspell is an **experimental platform for rapid AI concept exploration**.

**The Experiment-Extract Workflow**:
1. **Explore**: Script AI concepts in Lua/JS - iterate in minutes
2. **Validate**: Test ideas with production-grade performance
3. **Extract**: Move proven patterns to Rust when ready
4. **Scale**: Production deployment with minimal refactoring

Built with **production-quality engineering** (architecture, performance, testing, observability) to make the transition from experiment to production as painless as possible. We use Rust not because we're production-ready, but because proven patterns deserve solid foundations for extraction.

**Current Status**: v0.13.1 complete. Production-ready PostgreSQL storage backend (10 unified backends), Row-Level Security multi-tenancy, cross-platform support (Linux + macOS), self-contained kernel architecture, and 52% documentation consolidation (111 → 53 files). Previous experimental features include adaptive memory (episodic/semantic/procedural), context engineering, 10 workflow templates, local LLM support (Ollama + Candle), and RAG with HNSW vector search. See [Documentation Hub](docs/) for comprehensive guides.

---

## From Experiment to Production

### Why Experimental + Rust?

rs-llmspell prioritizes **rapid experimentation** while building **production-ready foundations**.

**The Philosophy**:
- **Script Velocity**: Lua/JS for minute-level iteration on AI ideas
- **Concept Exploration**: Play with LLMs, transformers, diffusion, memory, learning
- **Validation at Scale**: Production-quality performance for thorough testing
- **Painless Extraction**: Clear path from validated experiments to Rust production code

### Production-Quality Foundations (While Experimental)

Although experimental, rs-llmspell is built with production-grade engineering:

- **Performance**: <2ms memory overhead, 8.47x HNSW speedup, <100ms context assembly
- **Architecture**: Modular (21 crates), trait-based, SOLID principles, clear boundaries
- **Scalability**: Designed for growth (async-first, resource limits, multi-tenancy ready)
- **Testing**: >90% coverage (784 tests passing), zero warnings policy
- **Documentation**: >95% API docs (50+ guides across user/dev/technical)
- **Observability**: Full tracing with <2% overhead, structured logging

**Result**: When your experiment succeeds, transitioning to production is **engineering work, not research work**.

### What This Is

✅ Experimental AI concept playground
✅ Script-first rapid iteration
✅ Production-quality engineering
✅ Clear extraction path to Rust
✅ Learning platform for AI patterns

### What This Is NOT

❌ Production-ready out of the box
❌ Enterprise deployment platform
❌ Guaranteed stable APIs (pre-1.0)
❌ Support contracts or SLAs

---

## ✨ Experimentation Capabilities

### 🧠 Adaptive Memory & Context Engineering (v0.13.0)
**Latest experimental infrastructure for rapid memory pattern exploration**
- **3-Tier Memory**: Episodic (conversation), Semantic (knowledge graph), Procedural (patterns)
- **Hot-Swappable Backends**: InMemory (dev), HNSW (production), SurrealDB (graph)
- **Context Assembly**: 4 strategies (episodic, semantic, hybrid, RAG) with parallel retrieval
- **Performance**: <2ms memory overhead (50x faster than target), 8.47x HNSW speedup
- **CLI + Lua API**: Memory global (17th), Context global (18th) for script access
- **149 Tests**: 100% pass rate, zero warnings, comprehensive validation
- **See**: [Memory Configuration Guide](docs/user-guide/memory-configuration.md)

### 🎯 Workflow Templates (Experimental)
**10 experimental workflows for rapid AI concept exploration (v0.12.0)**
- **Research**: research-assistant (4-phase workflow), knowledge-management (RAG integration)
- **Development**: code-generator (3 agents), code-review (multi-aspect analysis)
- **Content**: content-generation (quality-driven), document-processor (PDF/OCR)
- **Productivity**: interactive-chat (session-based), file-classification (scan-classify-act)
- **Workflow**: workflow-orchestrator (custom composition), data-analysis (CSV/Excel/JSON)
- **CLI Access**: `template list|info|exec|search|schema`
- **Lua API**: Template global (16th) with 6 methods
- **Performance**: <2ms overhead (50x faster than target)

### 🔒 Local LLM Exploration (Zero Cost)
**Experiment offline with 100+ models (v0.11.0-v0.11.2)**
- **Dual Backend**: Ollama (REST API, 100+ models) + Candle (embedded GGUF inference)
- **Zero API Keys**: No cloud accounts needed for experimentation
- **Model Management**: `llmspell model list|pull|info|status`
- **Platform-Aware GPU**: Metal (macOS) + CUDA (Linux) with CPU fallback
- **Performance**: 40 tok/s throughput, 150ms first token, <5GB memory
- **10 Builtin Profiles**: LLaMA, T5, Qwen2, Phi-2 ready-to-use configs

### 📦 Modular Builds for Fast Iteration
**Feature flags for rapid development cycles (v0.10.0)**
- **Minimal**: 19MB (core only, fast compile)
- **Common**: 25MB (+templates, PDF)
- **Full**: 35MB (all experimental tools)
- **87% Compile Speedup**: Bridge-only builds 38s→5s

### 🧠 RAG System (Experimental Retrieval)
**Vector search and hybrid retrieval for concept validation (v0.8.0)**
- **HNSW Vector Storage**: <8ms @ 100K vectors, <35ms @ 1M vectors
- **Hybrid Search**: Vector + keyword + BM25 reranking
- **Multi-Tenant**: StateScope isolation with 3% overhead
- **RAGPipelineBuilder**: Fluent API for custom pipelines
- **Embedding Providers**: OpenAI, Cohere, HuggingFace, local models

### 🛠️ 40+ Experimental Tools
**Modular tools for rapid prototyping**
- **Core**: File ops, web search, calculator, HTTP client
- **Common** (`--features common`): Templates (Tera/Handlebars), PDF processing
- **Full** (`--features full`): Excel, CSV, archives, email, database
- **Direct CLI**: `llmspell tool list|info|invoke|search|test`
- **Sandboxed**: Secure execution with automatic feature detection

### 🎯 Service Infrastructure (Scale Validation)
**When validating at scale or extracting to production (v0.10.0)**
- **Unix Daemon**: Double-fork daemonization (1.8s startup)
- **Signal Handling**: SIGTERM/SIGINT → graceful shutdown
- **systemd/launchd**: Service deployment when concepts are proven
- **Fleet Management**: Multi-kernel orchestration for load testing
- **Log Rotation**: Automatic rotation (78ms, size/age policies)
- **PID Management**: Lifecycle tracking (6ms validation)

### 🤖 Multi-Agent Orchestration
**Coordinate 2-20+ agents for complex workflows**
- Sequential, parallel, conditional execution patterns
- Real-time state sharing between agents
- Automatic error recovery and retry logic
- Session-aware context management

### 🔒 Security Sandbox (For Safe Experiments)
**Experiment safely with isolated tool execution**
- 3-level security model (Safe/Restricted/Privileged)
- Mandatory sandboxing for all tool executions
- Policy-based access control
- Resource boundaries (CPU, memory, I/O)

---

## 🚀 Quick Start

### Installation (Choose Your Build)

```bash
# Clone repository
git clone https://github.com/lexlapax/rs-llmspell
cd rs-llmspell

# Choose your build:
cargo build --release                      # Minimal: 19MB (core, fast compile)
cargo build --release --features common    # Common: 25MB (+templates, PDF)
cargo build --release --features full      # Full: 35MB (all tools)

# Set API key (or use local LLMs)
export OPENAI_API_KEY="sk-..."  # Optional for cloud models
```

### Your First Experiment (60 seconds)

```bash
# Simple agent interaction
./target/release/llmspell exec '
  local agent = Agent.create({model = "openai/gpt-4o-mini"})
  print(agent:execute({prompt = "Explain Rust ownership in 2 sentences"}).response)
'

# Or use local LLM (zero cost)
./target/release/llmspell exec '
  local agent = Agent.create({model = "local/llama3.1:8b@ollama"})
  print(agent:execute({prompt = "What is async Rust?"}).response)
'
```

### Experimental Workflows

```bash
# Research workflow with memory
./target/release/llmspell template exec research-assistant \
  --param topic="Rust async patterns" \
  --param max_sources=10 \
  --param memory_enabled=true

# Code generation experiment
./target/release/llmspell template exec code-generator \
  --param description="Binary search tree in Rust" \
  --param language="rust" \
  --param model="ollama/llama3.2:3b"

# Memory exploration
./target/release/llmspell exec examples/script-users/getting-started/06-episodic-memory-basic.lua
```

### Explore More

- **[60+ Examples](examples/)** - Learning by doing
- **[User Guide](docs/user-guide/)** - Comprehensive experimentation guide
- **[Template Guides](docs/user-guide/templates/)** - 10 workflow templates explained
- **[Local LLM Setup](docs/user-guide/local-llm.md)** - Zero-cost exploration
- **[Developer Guide](docs/developer-guide/)** - Build your own experimental components

---

## 📊 Performance Validation (At Scale)

**Phase 13 Memory System** (Experimental Infrastructure):
| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Memory add | <10ms | 0.248ms | ✅ 40x faster |
| Context assembly | <100ms | ~8ms | ✅ 12x faster |
| HNSW speedup | >5x | 8.47x | ✅ 70% better |
| Memory overhead | <100ms | <2ms | ✅ 50x faster |

**Phase 12 Template System** (Experimental Workflows):
| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Template list | <10ms | 0.5ms | ✅ 20x faster |
| Execute overhead | <100ms | <2ms | ✅ 50x faster |
| Parameter validation | <5ms | 0.1ms | ✅ 50x faster |

**Phase 10-11 Infrastructure** (Service & Local LLM):
| Metric | Target | Achieved | Status |
|--------|--------|----------|--------|
| Daemon startup | <2s | 1.8s | ✅ 10% faster |
| Tool init | <10ms | 7ms | ✅ 30% faster |
| Vector search @ 100K | <10ms | 8ms | ✅ 20% faster |
| Candle throughput | 30 tok/s | 40 tok/s | ✅ 33% faster |

---

## 🎯 Development Phases (Experimental Infrastructure Evolution)

**Completed (13/13 major phases)**:
- ✅ **Phase 0-6**: Foundation (traits, tools, hooks, state, sessions)
- ✅ **Phase 7**: Infrastructure consolidation (536+ files refactored)
- ✅ **Phase 8**: RAG system (HNSW vectors, multi-tenant)
- ✅ **Phase 9**: REPL & debugging (interactive development)
- ✅ **Phase 10**: Service integration (daemon, tool CLI, fleet)
- ✅ **Phase 11**: Local LLM (Ollama + Candle dual backend)
- ✅ **Phase 11a**: Bridge consolidation (87% compile speedup)
- ✅ **Phase 11b**: LLM cleanup (unified profiles, T5 support)
- ✅ **Phase 12**: Workflow templates (10 experimental templates)
- ✅ **Phase 13**: Adaptive memory (3-tier, hot-swap backends, context engineering)

**Upcoming Experimental Features** (Phases 14+):
- **Phase 14**: Template composition (custom workflow building)
- **Phase 15**: Model Context Protocol (external tool integration)
- **Phase 16**: Advanced orchestration patterns
- **Phase 17-18**: Distributed execution, cloud platform

*Note: All phases build experimental infrastructure with production-quality engineering. When concepts are proven, extraction to production is straightforward.*

---

## 📘 Documentation

**For comprehensive guides, see [Documentation Hub](docs/)**

### Quick Links
- **[Getting Started](docs/user-guide/01-getting-started.md)** - 5-minute experimental setup
- **[Core Concepts](docs/user-guide/02-core-concepts.md)** - Understand the architecture
- **[Configuration](docs/user-guide/03-configuration.md)** - LLM providers, memory, storage, security
- **[Templates](docs/user-guide/templates/)** - 10 workflow templates
- **[Storage Setup](docs/user-guide/07-storage-setup.md)** - PostgreSQL deployment guide
- **[Lua API](docs/user-guide/appendix/lua-api-reference.md)** - 18 globals, 200+ methods
- **[Developer Guide](docs/developer-guide/)** - Build experimental components
- **[Technical Docs](docs/technical/)** - Architecture & design decisions

---

## 🛠️ Scripts & Automation

```bash
# Quality checks before committing
./scripts/quality/quality-check-minimal.sh  # Fast: format, clippy
./scripts/quality/quality-check-fast.sh     # 1 min: + unit tests

# Run experimental workflows
./scripts/utilities/llmspell-easy.sh         # Interactive launcher
./scripts/testing/test-by-tag.sh memory      # Test memory system
```

See [Scripts Overview](scripts/) for all automation tools.

---

## 🤝 Contributing

**New Contributors**: Start with [README-DEVEL.md](README-DEVEL.md) for complete development environment setup.

Building experimental AI components? See [Developer Guide](docs/developer-guide/) for:
- Rapid iteration patterns
- Production-quality code for future extraction
- Testing with llmspell-testing helpers
- 60+ examples to learn from

Read [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines and workflow.

---

## 📞 Community

- **Issues**: [GitHub Issues](https://github.com/lexlapax/rs-llmspell/issues)
- **Discussions**: [GitHub Discussions](https://github.com/lexlapax/rs-llmspell/discussions)
- **Examples**: [60+ working examples](examples/)

---

## 📜 License

Apache License, Version 2.0. See [LICENSE-APACHE](LICENSE-APACHE) for details.

---

## 🚀 Latest Release

**v0.13.1 - Production Storage Infrastructure & Documentation Consolidation**

Production-ready PostgreSQL storage backend with 10 unified backends, Row-Level Security for database-enforced multi-tenancy, cross-platform compilation support, self-contained kernel architecture, and comprehensive documentation consolidation.

**Key Achievements**:
- 🗄️ 1 new crate (llmspell-storage) with 10 PostgreSQL backends
- 🐘 PostgreSQL 18 + VectorChord (5x faster than pgvector, 26x cheaper)
- 🔒 Row-Level Security with <5% overhead (4.9% measured)
- 🐧 Cross-platform support (Linux + macOS)
- 🏗️ Self-contained kernel (630+ lines deleted, 82% fewer API methods)
- 📖 52% documentation consolidation (111 → 53 files)
- 📊 379 PostgreSQL tests passing (100% pass rate, zero warnings)
- 🎯 Zero breaking changes (opt-in PostgreSQL, existing backends preserved)

See [Release Notes](RELEASE_NOTES_v0.13.1.md) for complete details.

---

**📘 Full Documentation**: See [docs/](docs/) for comprehensive user guides, technical architecture, and developer resources.
