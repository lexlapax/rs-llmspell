# Rs-LLMSpell Documentation Hub

**Complete documentation for experimental AI platform enabling rapid concept exploration with production-ready foundations**

**🔗 Navigation**: [← Project Home](../README.md) | [Examples](../examples/) | [Contributing](../CONTRIBUTING.md)

> **📖 Documentation Hub**: All documentation for rs-llmspell v0.13.0 (Phase 13 Complete - Experimental Memory & Context Engineering). Comprehensive guides for rapid AI experimentation with script-first velocity and clear production extraction path. **Learn → Experiment → Validate → Extract**.

---

## Documentation Structure (Phase 13 Complete - Experimental Infrastructure)

### 📘 [User Guide](user-guide/) - *For Experimenters*
**Purpose**: Practical guides for rapid AI concept exploration via scripts.

**Status**: ✅ Consolidated (Phase 13b.18) - 10 numbered guides + appendix
**Structure**: **Linear learning path** (01 → 10) + comprehensive appendix
**Contents**: 10 numbered guides + 1 appendix covering complete user journey from installation to production deployment

**The 10 Numbered Guides** (Start here for linear learning):
1. **[Getting Started](user-guide/01-getting-started.md)** - Installation and first experiments (10 min)
2. **[Core Concepts](user-guide/02-core-concepts.md)** - Architecture, memory, RAG, multi-tenancy
3. **[Configuration](user-guide/03-configuration.md)** - Providers, memory, security, feature flags
4. **[Lua Scripting](user-guide/04-lua-scripting.md)** - Essentials (18 globals, common patterns)
5. **[CLI Reference](user-guide/05-cli-reference.md)** - All 16 command groups
6. **[Templates & Workflows](user-guide/06-templates-and-workflows.md)** - 10 AI workflows
7. **[Storage Setup](user-guide/07-storage-setup.md)** - PostgreSQL quick start
8. **[Deployment](user-guide/08-deployment.md)** - systemd/launchd + IDE integration
9. **[Security](user-guide/09-security.md)** - Sandbox, permissions, multi-tenancy
10. **[Troubleshooting](user-guide/10-troubleshooting.md)** - Debug, profile, diagnose

**Appendix** (Reference lookups):
- **[Lua API Reference](user-guide/appendix/lua-api-reference.md)** - Complete API (3,729 lines, 200+ methods)

**Additional Resources**:
- **[Templates](user-guide/templates/)** - Detailed template documentation (11 files)
- **[Developer Guide Reference](developer-guide/reference/)** - Rust API by theme (6 guides)

**Phase 13 Additions**: 3-tier memory (episodic/semantic/procedural), context engineering, Memory + Context globals (17th/18th)
**Phase 12 Additions**: 10 experimental templates, template CLI, Template global (16th)
**Phase 11 Additions**: Local LLM support (Ollama + Candle), model management

**Start here if**: You want to experiment with AI concepts via Lua scripts, use experimental workflows, explore memory patterns, or deploy production services

---

### 🔧 [Developer Guide](developer-guide/) - *For Contributors*
**Purpose**: Technical guides for developers contributing to or extending rs-llmspell.

**Status**: ✅ Consolidated (Phase 13b.19) - 7 numbered guides + 6 thematic API references
**Structure**: **Linear learning path** (01 → 07) + thematic API documentation
**Contents**: 7 numbered guides + 6 thematic references + examples guide = 14 total files

**The 7 Numbered Guides** (Start here for linear learning):
1. **[Getting Started](developer-guide/01-getting-started.md)** - Setup, architecture, first contribution (15 min)
2. **[Development Workflow](developer-guide/02-development-workflow.md)** - Testing, quality gates, git workflow (30 min)
3. **[Extending Components](developer-guide/03-extending-components.md)** - Tools, agents, hooks, workflows, RAG, storage, templates (2-6 hrs)
4. **[Bridge Patterns](developer-guide/04-bridge-patterns.md)** - Typed structs for script config (2-3 hrs)
5. **[Production Deployment](developer-guide/05-production-deployment.md)** - Security, performance, scaling, monitoring (4-8 hrs)
6. **[Tracing & Debugging](developer-guide/06-tracing-debugging.md)** - Instrumentation, session correlation (1-2 hrs)
7. **[Feature Flags](developer-guide/07-feature-flags.md)** - Build system, modular builds (15 min)

**Thematic API References** (Consolidated by topic):
- [Core Traits](developer-guide/reference/core-traits.md) - BaseAgent, ExecutionContext, testing
- [Storage Backends](developer-guide/reference/storage-backends.md) - Vector storage, HNSW
- [RAG Pipeline](developer-guide/reference/rag-pipeline.md) - Document ingestion, retrieval, context
- [Memory Backends](developer-guide/reference/memory-backends.md) - Episodic, semantic, procedural
- [Security Integration](developer-guide/reference/security-integration.md) - Access control, multi-tenancy
- [Crate Index](developer-guide/reference/crate-index.md) - Quick reference to all 21 crates

**Additional Resources**:
- **[Examples Reference](developer-guide/examples-reference.md)** - 60+ production examples

**Phase 13 Additions**: Memory backend patterns, context engineering, knowledge graph integration
**Phase 12 Additions**: Template creation patterns, TemplateRegistry, ExecutionContext builder
**Phase 11 Additions**: Local provider patterns, typed bridge pattern, GGUF model handling

**Start here if**: You want to build experimental components with production-quality code for future extraction

---

### 🏗️ [Technical](technical/) - *For Architects*
**Purpose**: Core architectural documentation and implementation decisions.

**Status**: ✅ Complete for Phase 13 with 13+ documents
**Contents**: 6 core guides + 7 supplementary docs covering architecture, protocols, performance, benchmarking, stress testing, protocol compliance, and dependency analysis
**Key Files**: `current-architecture.md`, `kernel-protocol-architecture.md`, `debug-dap-architecture.md`, `cli-command-architecture.md`, `performance-baseline.md`, `benchmarking-guide.md`, `stress-test-results.md`, `protocol-compliance-report.md`, `mlua-upgrade-analysis.md`
**Phase 12 Additions**: Template system architecture (TemplateRegistry, ExecutionContext), 10 built-in templates, 20-50x performance targets, parameter validation schema
**Phase 11 Additions**: Local provider architecture, GGUF inference pipeline, dual-backend design (Ollama via rig + Candle embedded)
**Start here if**: You need to understand system architecture, protocols, template system, local LLM integration, performance characteristics, or debugging infrastructure

---

### 📚 [Archives](archives/) - *Historical Reference*
**Purpose**: Preserve historical documents for reference.

**Status**: 📦 100+ documents archived
**Contents**: Phase handoff packages, superseded technical docs, consolidated guides, research notes
**Note**: These documents may be outdated but provide historical context

---

### 🚧 [In-Progress](in-progress/) - *Development Tracking*
**Purpose**: Track planning and implementation toward version 1.0.

**Status**: 📋 Phase 13 Complete, Phase 14 Planning
**Contents**: Phase completion documents (PHASE00-12 DONE), implementation roadmaps, design documents
**Key Files**: `implementation-phases.md` (23-phase roadmap), phase-specific design docs
**For**: Core team tracking progress

---

## What Rs-LLMSpell Actually Is

**Experimental Platform** with **Production-Quality Foundations** (v0.13.0):
- ✅ **Adaptive Memory System** with 3-tier architecture (episodic/semantic/procedural) (Phase 13) ⭐
- ✅ **Memory + Context Globals** (17th/18th Lua globals) for experimental memory patterns ⭐
- ✅ **Hot-Swappable Backends** (InMemory/HNSW/SurrealDB) with 8.47x speedup ⭐
- ✅ **10 Experimental Workflows** with 6 base + 4 advanced templates (Phase 12)
- ✅ **Template CLI** (5 subcommands: list, info, exec, search, schema)
- ✅ **Template Lua API** (Template global, 16th of 18, with 6 methods)
- ✅ **TemplateRegistry** with DashMap concurrent storage and Arc sharing ⭐
- ✅ **ExecutionContext** builder for infrastructure dependency injection ⭐
- ✅ **Template Performance** 20-50x faster than targets (<2ms init, <1ms lookup) ⭐
- ✅ **Local LLM Integration** with dual-backend support (Ollama via rig + Candle embedded inference)
- ✅ **GGUF Model Support** with HuggingFace downloads, quantization (Q4_K_M, Q5_K_M, Q8_0), and chat templates
- ✅ **Privacy-First Workflows** with offline-capable local model inference
- ✅ **Model Management CLI** (7 subcommands: `llmspell model list/pull/remove/info/available/status/install-ollama`)
- ✅ **Model Management API** (LocalLLM Lua global with list, pull, info, status methods for script access)
- ✅ **Kernel Protocol Extension** (model_request/model_reply for remote kernel model management)
- ✅ **Unix Daemon Infrastructure** with double-fork, TTY detachment, session leadership
- ✅ **Signal Handling** (SIGTERM/SIGINT → Jupyter shutdown, atomic operations)
- ✅ **Tool CLI Commands** (5 subcommands: list, info, invoke, search, test)
- ✅ **Fleet Management** with OS-level process isolation, Bash/Python managers, Docker orchestration
- ✅ **Feature Flags** for modular builds (minimal 19MB, common 25MB, full 35MB)
- ✅ **Enhanced Logging** with rotation (10MB/7 days), structured tracing, <1ms overhead
- ✅ **PID File Management** with lifecycle tracking and stale cleanup
- ✅ **systemd/launchd Integration** for production service deployment
- ✅ **Integrated Kernel Architecture** with global IO runtime (no "dispatch task is gone")
- ✅ **Debug Adapter Protocol (DAP)** with 10 essential commands, IDE integration
- ✅ **Multi-Protocol Support** (Jupyter v5.3, DAP, LSP, REPL)
- ✅ **40+ tools** with optional dependencies (templates, PDF, CSV, Excel, archives, email, DB)
- ✅ **21 crates** with unified kernel, template system, and memory system
- ✅ **Lua scripting** with 18 zero-import globals (Agent, Tool, Template, Memory, Context, RAG, etc.)
- ✅ **--trace flag** with unified logging control
- ✅ **Event correlation** for request tracking with correlation IDs
- ✅ **Multi-client support** with message routing
- ✅ **Connection file discovery** for IDE attachment (Jupyter-compatible)
- ✅ **Agent infrastructure** with BaseAgent trait and builder patterns
- ✅ **4 workflow patterns** (Sequential, Parallel, Conditional, Loop)
- ✅ **RAG system** with HNSW vector search (8ms @ 100K vectors, 20% faster than target)
- ✅ **Multi-tenant architecture** with StateScope::Custom isolation (3% overhead, 40% better)
- ✅ **State persistence** unified in kernel
- ✅ **Hook system** with 40+ points and circuit breakers
- ✅ **Event bus** with 90K+ events/sec throughput
- ✅ **64+ production examples** across 7 categories (including templates and local LLM)

**Phase 13 Achievements** (v0.13.0 - Experimental Infrastructure):
- ✅ **3-Tier Memory System**: Episodic (HNSW), Semantic (SurrealDB graph), Procedural (patterns)
- ✅ **Hot-Swappable Backends**: InMemory (dev), HNSW (8.47x speedup), SurrealDB (bi-temporal graph)
- ✅ **Context Engineering**: 4 strategies (episodic, semantic, hybrid, RAG) with parallel retrieval
- ✅ **<2ms Memory Overhead**: 50x faster than target, production-quality validation at scale
- ✅ **149 Tests**: 100% pass rate, zero warnings, comprehensive validation
- ✅ **Zero Breaking Changes**: Fully backward compatible, opt-in features
- ✅ **1,300+ Lines API Docs**: llmspell-memory, llmspell-graph, llmspell-context

**Phase 12 Achievements** (v0.12.0 - Experimental Workflows):
- ✅ **10 Experimental Templates**: 6 base workflows + 4 advanced patterns for rapid exploration
- ✅ **Template System**: llmspell-templates crate with TemplateRegistry, ExecutionContext
- ✅ **20-50x Performance**: <2ms init, <1ms lookup, production-quality engineering
- ✅ **149 Tests**: 100% passing, comprehensive validation
- ✅ **3,655 Lines Docs**: Complete user guides for all 10 templates

**Phase 11 Achievements** (v0.11.0):
- ✅ **Dual-Backend Architecture**: Ollama via rig (REST API) + Candle (embedded GGUF inference)
- ✅ **CLI Model Commands**: 7 subcommands (list, pull, remove, info, available, status, install-ollama) - 468 lines
- ✅ **Kernel Protocol Extension**: model_request/model_reply messages for remote model management
- ✅ **GGUF Integration**: Direct GGUF file loading with HuggingFace downloads, tokenizer fallback
- ✅ **Chat Template Support**: TinyLlama-Chat and similar models with proper formatting
- ✅ **Model Management API**: LocalLLM Lua global with list, pull, info, status methods
- ✅ **Provider Factory**: ModelSpecifier parsing with @ollama/@candle backend selection
- ✅ **Dual Interface**: Both CLI and Lua API for maximum user flexibility
- ✅ **Testing**: 10/10 integration tests passing (5 Ollama + 5 Candle with RUN_EXPENSIVE_TESTS)
- ✅ **Documentation**: Comprehensive user guide (320 lines) + 4 production examples (260 lines)
- ✅ **Zero Warnings**: Clean cargo doc build, all clippy warnings resolved

**Phase 10 Achievements** (v0.10.0):
- ✅ **Unix Daemon Infrastructure**: 1.8s startup (10% faster), double-fork, signal handling
- ✅ **Tool CLI**: 5 subcommands for direct tool access, 7ms initialization (30% faster)
- ✅ **Fleet Management**: OS-level isolation, Bash/Python managers, Docker orchestration
- ✅ **Feature Flags**: Modular builds reducing size 43% (minimal) to 26% (common)
- ✅ **Enhanced Logging**: Rotating logs with 78ms rotation (22% faster), <1ms overhead
- ✅ **Performance**: All 10 targets exceeded by 10-40% (message handling 3.8ms, 24% faster)
- ✅ **Testing**: 486 tests total (kernel:57, bridge:334, CLI:57, fleet:38)
- ✅ **Production Ready**: systemd/launchd, graceful shutdown, health monitoring

**What it doesn't do**:
- ❌ GUI or web interface (CLI and library only)
- ❌ JavaScript support in kernel (Lua only currently)
- ❌ Python kernel support (planned for Phase 13+)
- ❌ Distributed execution (planned for Phase 13+)

---

## Quick Start Paths

### 🚀 **I want to experiment with AI concepts**
1. **[Getting Started](user-guide/getting-started.md)** - 5-minute experimental setup
2. **[Experimental Workflows](user-guide/templates/README.md)** ⭐ - 10 templates for rapid concept exploration
3. **[Memory System](user-guide/memory-configuration.md)** ⭐ - Explore adaptive memory patterns
3. **[Core Concepts](user-guide/concepts.md)** - Understand kernel, tools (40+), agents, workflows, templates
4. **Template CLI** ⭐ - `llmspell template list`, `exec`, `info`, `search`, `schema` commands
5. **Tool CLI** - `llmspell tool list`, `invoke`, `info`, `search`, `test` commands
6. **[Service Deployment](user-guide/service-deployment.md)** - Deploy as daemon with systemd/launchd
7. **[IDE Integration](user-guide/ide-integration.md)** - Connect VS Code or Jupyter
8. **[Lua API Reference](user-guide/appendix/lua-api-reference.md)** - Complete API documentation (18 globals)
9. **[Examples](../examples/script-users/)** - 60+ working examples including templates and tool CLI

### 🔨 **I want to build experimental components**
1. **[Developer Guide](developer-guide/developer-guide.md)** - Complete onboarding for 21 crates
2. **[Template Creation](developer-guide/template-creation.md)** - Build experimental workflows
3. **[Feature Flags Migration](developer-guide/feature-flags-migration.md)** - Build system changes (Phase 10.17.5+)
4. **[Extending LLMSpell](developer-guide/extending-llmspell.md)** - Build tools, agents, protocols
5. **[Production Guide](developer-guide/production-guide.md)** - Deploy to production
6. **[Kernel Architecture](technical/kernel-protocol-architecture.md)** - Daemon, protocols, fleet

### 🏛️ **I need architectural understanding**
1. **[Current Architecture](technical/current-architecture.md)** - 21 crates, Phase 13 experimental infrastructure
2. **[Kernel Protocol Architecture](technical/kernel-protocol-architecture.md)** - Daemon, protocols, transport
3. **[Debug DAP Architecture](technical/debug-dap-architecture.md)** - 10 DAP commands, IDE integration
4. **[Performance Baseline](technical/performance-baseline.md)** - Phase 12 metrics (20-50x for templates)
5. **[Master Vision](technical/master-architecture-vision.md)** - 23-phase roadmap

### 🛠️ **I want to validate at scale / extract to production**
1. **[Service Deployment](user-guide/service-deployment.md)** - Daemon for scale validation, systemd/launchd when extracting
2. **[Configuration Guide](user-guide/configuration.md)** - Daemon, feature flags, fleet config
3. **[Performance Tuning](user-guide/performance-tuning.md)** - Optimization for production
4. **[Troubleshooting](user-guide/troubleshooting.md)** - General issues
5. **[Phase 10 Troubleshooting](user-guide/troubleshooting-phase10.md)** - Daemon, signals, PID, fleet
6. **[Production Guide](developer-guide/production-guide.md)** - Best practices

---

## Phase 12 Documentation Achievements (v0.12.0)

### Template System Documentation
- **Template User Guide**: Comprehensive guide with all 10 built-in templates documented
- **Template CLI Documentation**: Complete reference for all 5 template subcommands
- **Template Lua API**: Full documentation of Template global with 6 methods
- **Template Creation Guide**: Developer guide for building custom templates (50 LOC minimum)
- **3,655 Lines Template Docs**: User guides, API reference, and examples

### Template Examples
- **10 Built-in Templates**: Complete documentation for all templates with parameter schemas
- **Template Categories**: Research, Chat, Analysis, CodeGen, Document, Workflow documented
- **ExecutionContext Patterns**: Dependency injection examples for all infrastructure types
- **Parameter Validation**: Declarative schema patterns with constraints

### API Documentation
- **Zero Warnings**: Clean cargo doc build for llmspell-templates crate
- **Template Trait**: Complete documentation of Template trait with 5 methods
- **TemplateRegistry**: DashMap-based concurrent storage documentation
- **ExecutionContext**: Builder pattern with 6 infrastructure types documented

### Documentation Structure Updates
- **User Guide**: Updated from 11 to 12 essential documents (added templates/README.md)
- **Developer Guide**: Added template-creation.md (8th essential guide)
- **Technical Docs**: Updated all with Phase 12 template system architecture
- **Navigation**: Updated all README files with Phase 12 status and 18 crates/globals
- **API Updates**: Lua API 17→18 globals, Rust API 17→18 crates

---

## Phase 11 Documentation Achievements (v0.11.0)

### Local LLM Integration Documentation
- **Local LLM User Guide**: Comprehensive 320-line guide covering both Ollama and Candle backends
- **Quick Start Sections**: Separate quick starts for each backend (Ollama REST API, Candle embedded)
- **Model Management**: Complete documentation for list, pull, info, status commands
- **Configuration Examples**: Both backends with GGUF settings, quantization, chat templates
- **Troubleshooting**: 6 common scenarios (Ollama connection, model downloads, GGUF errors, tokenizer issues)
- **Performance Comparison**: Ollama vs Candle characteristics and use cases

### Example Applications
- **4 Production Examples**: 260 lines of runnable Lua scripts (status, chat, comparison, model info)
- **LocalLLM Global API**: Demonstrated usage of all LocalLLM methods
- **Agent Integration**: Examples showing local models with Agent.create()
- **Error Handling**: Proper status checks and fallbacks

### API Documentation
- **Zero Warnings**: Clean cargo doc build for all Phase 11 packages
- **GGUF Documentation**: Complete docs for GGUF loading, tokenization, inference
- **Provider Patterns**: LocalProviderInstance trait, factory pattern, backend selection

### Documentation Structure Updates
- **User Guide**: Updated from 10 to 11 essential documents
- **Navigation**: Updated all README files with Phase 11 status
- **Examples Index**: Added local LLM examples to example categories

---

## Phase 10 Documentation Achievements (v0.10.0)

### Core Infrastructure Documentation
- **Unix Daemon Guide**: Double-fork, TTY detachment, session leadership, PID lifecycle
- **Signal Handling**: SIGTERM/SIGINT → Jupyter messages, atomic operations, resource cleanup
- **Tool CLI Architecture**: 5 subcommands, kernel message protocol, ComponentRegistry access
- **Fleet Management**: OS-level isolation, Bash/Python managers, Docker orchestration patterns
- **Feature Flags Migration**: Modular builds (19-35MB), dependency mapping, troubleshooting

### Performance & Testing Documentation
- **Performance Baseline**: 10 metrics with targets vs actuals (all 10-40% faster)
- **Benchmarking Guide**: Automated kernel benchmarking, Criterion setup, regression detection
- **Stress Test Results**: >24h uptime, 42MB stable memory, multi-client scenarios
- **Protocol Compliance**: Jupyter v5.3 wire protocol, ZeroMQ 5-channel validation
- **MLua Analysis**: Upgrade impact assessment (0.9.9 → 0.11), revert rationale

### User & Developer Documentation
- **User Guide**: 10 essential documents (added Phase 10 troubleshooting, performance tuning)
- **Developer Guide**: 6 essential guides (added feature flags migration, tracing best practices)
- **Technical Docs**: 13 documents (6 core + 7 supplementary)
- **Troubleshooting**: General + Phase 10 specific (daemon, signals, PID, fleet, tool CLI)

### Consolidation Results
- **Documentation Growth**: 8 → 10 user docs, 4 → 6 developer docs, 8 → 13 technical docs
- **New Guides**: Phase 10 troubleshooting, performance tuning, feature flags migration, tracing best practices
- **Performance Data**: Complete baseline with 10 Phase 10 metrics
- **Testing Coverage**: 486 tests documented (kernel:57, bridge:334, CLI:57, fleet:38)

---

## Documentation Quality Standards

### Accuracy ✅
- All code examples tested with v0.12.0
- API documentation matches Phase 12 implementation
- Performance metrics from actual Phase 12 measurements (20-50x faster for templates)
- Architecture validated against 18 crates with daemon infrastructure, local LLM support, and template system

### Organization ✅
- Clear separation: User (usage) vs Developer (contributing) vs Technical (architecture)
- Template system with 10 built-in workflows and creation guide
- Service deployment with daemon, tool CLI, template CLI, and fleet management
- IDE integration documented independently
- Phase 10 troubleshooting separate from general issues
- Cross-references updated for all Phase 12 features

### Maintenance 📋
- Version tracking (v0.12.0)
- Phase status clearly marked (Phase 12 Complete - Production-Ready AI Agent Templates)
- Update dates: October 2025
- Template system documentation for Phase 12
- Feature flags migration guide for Phase 10.17.5+ builds
- Local LLM integration guide for Phase 11
- Deprecation notices for old patterns

---

## Kernel Architecture Overview

### 🎯 Execution Modes
1. **Embedded Mode**: Kernel runs within CLI process (default)
2. **Service Mode**: Kernel listens for external connections
3. **Daemon Mode**: Kernel runs as background system service

### 🔌 Protocol Support
- **Jupyter Protocol**: 5-channel architecture for notebooks
- **Debug Adapter Protocol**: IDE debugging with VS Code
- **Language Server Protocol**: Code intelligence (future)
- **REPL Protocol**: Interactive command-line interface

### 🚀 Key Features
- **Global IO Runtime**: Single Tokio runtime for all operations
- **Message Router**: Multi-client message distribution
- **Event Correlation**: Track requests across components
- **Connection Files**: Jupyter-compatible kernel discovery
- **Health Monitoring**: HTTP endpoints for metrics/health

---

## Examples Structure

### 📚 [Script Users](../examples/script-users/) - 64+ Lua Examples
- **Getting Started**: 6 progressive tutorials (hello world → kernel → RAG)
- **Local LLM**: 4 examples (status, chat, comparison, model info) using Ollama + Candle
- **Kernel Examples**: Service mode, daemon deployment, multi-client, tool CLI
- **Debug Examples**: DAP integration, breakpoints, tracing
- **Features**: Agent, tool (40+), workflow, state, provider examples
- **Cookbook**: 11 patterns (caching, RAG multi-tenant, error handling, tool CLI)
- **Applications**: 9 full applications (webapp creator, knowledge base, etc.)
- **Configs**: 15+ configuration examples including daemon, feature flags, fleet configs
- **Tool CLI**: Direct tool invocation examples (list, info, invoke, search, test)

### 🦀 [Rust Developers](../examples/rust-developers/) - 10+ Reference Projects
- Kernel integration patterns (daemon, signals, PID management)
- Protocol implementation (Jupyter, DAP)
- Custom transport layers
- Debug hook implementation
- Service wrapper creation
- Global IO runtime usage
- Event correlation patterns
- Multi-client handling
- Tool CLI integration
- Fleet management patterns
- Feature flags configuration

---

## Performance Metrics (Phase 10 Actual)

| Component | Metric | Target | Achieved | Status |
|-----------|--------|--------|----------|--------|
| Daemon Startup | Cold start | <2s | 1.8s | ✅ 10% faster |
| Message Handling | Latency | <5ms | 3.8ms | ✅ 24% faster |
| Signal Response | SIGTERM/SIGINT | <100ms | 85ms | ✅ 15% faster |
| Tool Initialization | Startup time | <10ms | 7ms | ✅ 30% faster |
| Log Rotation | File rotation | <100ms | 78ms | ✅ 22% faster |
| PID File Check | Validation | <10ms | 6ms | ✅ 40% faster |
| Memory Overhead | Kernel daemon | <50MB | 42MB | ✅ 16% better |
| Heartbeat Latency | ZeroMQ ping | <1ms | 0.8ms | ✅ 20% faster |
| Vector Search | 100K vectors | <10ms | 8ms | ✅ 20% faster |
| Multi-Tenant | Isolation overhead | <5% | 3% | ✅ 40% better |

---

## Getting Help

**📋 Documentation Issues**: File GitHub issues for corrections
**❓ General Questions**: Check [Troubleshooting](user-guide/troubleshooting.md)
**⚙️ Phase 10 Issues**: See [Phase 10 Troubleshooting](user-guide/troubleshooting-phase10.md) for daemon, signals, PID, fleet
**🚀 Performance**: Review [Performance Tuning](user-guide/performance-tuning.md) for optimization
**🏗️ Feature Flags**: Check [Feature Flags Migration](developer-guide/feature-flags-migration.md) for build issues
**🐛 Bug Reports**: Use GitHub issues with reproduction steps
**💡 Feature Requests**: Review [roadmap](in-progress/implementation-phases.md) first
**🤝 Contributing**: Start with [Developer Guide](developer-guide/)
**🚀 Deployment Help**: See [Service Deployment](user-guide/service-deployment.md)
**🔧 Debug Issues**: Check [Debug Architecture](technical/debug-dap-architecture.md)

---

**Last Updated**: January 2025 | **Version**: 0.13.0 (Phase 13 Complete - Experimental Memory & Context Engineering) | **Next**: Phase 14 (Template Composition)