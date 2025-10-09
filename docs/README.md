# Rs-LLMSpell Documentation Hub

**Complete documentation for production-ready scriptable LLM interactions with service integration**

**🔗 Navigation**: [← Project Home](../README.md) | [Examples](../examples/) | [Contributing](../CONTRIBUTING.md)

> **📖 Documentation Hub**: All documentation for rs-llmspell v0.11.0 (Phase 11 Complete - Local LLM Integration). Fully consolidated with integrated kernel architecture, Unix daemon infrastructure, tool CLI commands, fleet management, local LLM support (Ollama + Candle), and feature flags for modular builds (19-35MB). **Phase 11 delivers local model inference with dual-backend support for privacy-first and offline-capable LLM workflows.**

---

## Documentation Structure (Phase 11 Complete)

### 📘 [User Guide](user-guide/) - *For Script Writers*
**Purpose**: Practical guides for using rs-llmspell to build LLM-driven scripts and workflows.

**Status**: ✅ Updated with Phase 11 local LLM integration
**Contents**: 11 essential documents including getting started, concepts, configuration, local LLM integration, API reference (Lua + 17 Rust crates), troubleshooting, Phase 10 troubleshooting, performance tuning, service deployment, IDE integration
**Key Files**: `getting-started.md`, `concepts.md`, `configuration.md`, `local-llm.md`, `service-deployment.md`, `ide-integration.md`, `troubleshooting-phase10.md`, `performance-tuning.md`, `api/lua/README.md`
**Phase 11 Additions**: Local LLM support (Ollama + Candle backends), model management (list, pull, info, status), privacy-first workflows, offline inference
**Start here if**: You want to write Lua scripts, use local LLM models, use tool CLI directly, or deploy as production services

---

### 🔧 [Developer Guide](developer-guide/) - *For Contributors*
**Purpose**: Technical guides for developers contributing to or extending rs-llmspell.

**Status**: ✅ Updated with Phase 11 local provider patterns and bridge pattern guide
**Contents**: 7 essential guides including developer guide, extending guide, production guide, examples reference, feature flags migration, tracing best practices, bridge pattern guide
**Key Files**: `developer-guide.md`, `extending-llmspell.md`, `production-guide.md`, `examples-reference.md`, `feature-flags-migration.md`, `tracing-best-practices.md`, `bridge-pattern-guide.md`
**Phase 11 Additions**: Local provider implementation patterns, GGUF model handling, dual-backend architecture (Ollama + Candle), typed bridge pattern (Phase 11a.8)
**Start here if**: You want to create custom tools, contribute code, implement local providers, work on bridge layer, or understand modular build system

---

### 🏗️ [Technical](technical/) - *For Architects*
**Purpose**: Core architectural documentation and implementation decisions.

**Status**: ✅ Complete for Phase 11 with 13 documents
**Contents**: 6 core guides + 7 supplementary docs covering architecture, protocols, performance, benchmarking, stress testing, protocol compliance, and dependency analysis
**Key Files**: `current-architecture.md`, `kernel-protocol-architecture.md`, `debug-dap-architecture.md`, `cli-command-architecture.md`, `performance-baseline.md`, `benchmarking-guide.md`, `stress-test-results.md`, `protocol-compliance-report.md`, `mlua-upgrade-analysis.md`
**Phase 11 Additions**: Local provider architecture, GGUF inference pipeline, dual-backend design (Ollama via rig + Candle embedded)
**Start here if**: You need to understand system architecture, protocols, local LLM integration, performance characteristics, or debugging infrastructure

---

### 📚 [Archives](archives/) - *Historical Reference*
**Purpose**: Preserve historical documents for reference.

**Status**: 📦 100+ documents archived
**Contents**: Phase handoff packages, superseded technical docs, consolidated guides, research notes
**Note**: These documents may be outdated but provide historical context

---

### 🚧 [In-Progress](in-progress/) - *Development Tracking*
**Purpose**: Track planning and implementation toward version 1.0.

**Status**: 📋 Phase 11 Complete, Phase 12 Planning
**Contents**: Phase completion documents (PHASE00-11 DONE), implementation roadmaps, design documents
**Key Files**: `implementation-phases.md` (16-phase roadmap), phase-specific design docs
**For**: Core team tracking progress

---

## What Rs-LLMSpell Actually Is

**Production-Ready Features** (v0.11.0):
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
- ✅ **17 crates** with unified kernel
- ✅ **Lua scripting** with 17 zero-import globals (Agent, Tool, RAG, Debug, etc.)
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
- ✅ **64+ production examples** across 7 categories (including local LLM examples)

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
- ❌ Python kernel support (planned for Phase 12)
- ❌ Distributed execution (planned for Phase 13)

---

## Quick Start Paths

### 🚀 **I want to use rs-llmspell**
1. **[Getting Started](user-guide/getting-started.md)** - 5-minute setup with feature flags
2. **[Core Concepts](user-guide/concepts.md)** - Understand kernel, tools (40+), agents, workflows
3. **Tool CLI** - `llmspell tool list`, `invoke`, `info`, `search`, `test` commands
4. **[Service Deployment](user-guide/service-deployment.md)** - Deploy as daemon with systemd/launchd
5. **[IDE Integration](user-guide/ide-integration.md)** - Connect VS Code or Jupyter
6. **[Lua API Reference](user-guide/api/lua/README.md)** - Complete API documentation
7. **[Examples](../examples/script-users/)** - 60+ working examples including tool CLI

### 🔨 **I want to extend rs-llmspell**
1. **[Developer Guide](developer-guide/developer-guide.md)** - Complete onboarding for 17 crates
2. **[Feature Flags Migration](developer-guide/feature-flags-migration.md)** - Build system changes (Phase 10.17.5+)
3. **[Extending LLMSpell](developer-guide/extending-llmspell.md)** - Build tools, agents, protocols
4. **[Production Guide](developer-guide/production-guide.md)** - Deploy to production
5. **[Kernel Architecture](technical/kernel-protocol-architecture.md)** - Daemon, protocols, fleet

### 🏛️ **I need architectural understanding**
1. **[Current Architecture](technical/current-architecture.md)** - 17 crates, Phase 10 achievements
2. **[Kernel Protocol Architecture](technical/kernel-protocol-architecture.md)** - Daemon, protocols, transport
3. **[Debug DAP Architecture](technical/debug-dap-architecture.md)** - 10 DAP commands, IDE integration
4. **[Performance Baseline](technical/performance-baseline.md)** - Phase 10 metrics (10-40% faster)
5. **[Master Vision](technical/master-architecture-vision.md)** - 16-phase roadmap

### 🛠️ **I want to deploy in production**
1. **[Service Deployment](user-guide/service-deployment.md)** - systemd/launchd, daemon mode, fleet
2. **[Configuration Guide](user-guide/configuration.md)** - Daemon, feature flags, fleet config
3. **[Performance Tuning](user-guide/performance-tuning.md)** - Optimization for production
4. **[Troubleshooting](user-guide/troubleshooting.md)** - General issues
5. **[Phase 10 Troubleshooting](user-guide/troubleshooting-phase10.md)** - Daemon, signals, PID, fleet
6. **[Production Guide](developer-guide/production-guide.md)** - Best practices

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
- All code examples tested with v0.11.0
- API documentation matches Phase 11 implementation
- Performance metrics from actual Phase 10 measurements (10-40% faster)
- Architecture validated against 17 crates with daemon infrastructure and local LLM support

### Organization ✅
- Clear separation: User (usage) vs Developer (contributing) vs Technical (architecture)
- Service deployment with daemon, tool CLI, and fleet management
- IDE integration documented independently
- Phase 10 troubleshooting separate from general issues
- Cross-references updated for all Phase 10 features

### Maintenance 📋
- Version tracking (v0.11.1)
- Phase status clearly marked (Phase 11a Complete - Bridge Consolidation)
- Update dates: October 2025
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

**Last Updated**: October 2025 | **Version**: 0.11.1 (Phase 11a Complete - Bridge Consolidation) | **Next**: Phase 12 (Adaptive Memory System)