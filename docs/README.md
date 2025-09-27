# Rs-LLMSpell Documentation Hub

**Complete documentation for production-ready scriptable LLM interactions with kernel architecture**

**🔗 Navigation**: [← Project Home](../README.md) | [Examples](../examples/) | [Contributing](../CONTRIBUTING.md)

> **📖 Documentation Hub**: All documentation for rs-llmspell v0.9.0 (Phase 10 Complete). Fully consolidated with integrated kernel architecture, production daemon support, and comprehensive debugging capabilities. **Phase 9-10 adds unified kernel, global IO runtime, DAP debugging, and service deployment.**

---

## Documentation Structure (Phase 10 Complete)

### 📘 [User Guide](user-guide/) - *For Script Writers*
**Purpose**: Practical guides for using rs-llmspell to build LLM-driven scripts and workflows.

**Status**: ✅ Updated with kernel architecture and service deployment
**Contents**: Getting started, concepts, configuration, complete API reference (Lua + 17 Rust crates), troubleshooting, service deployment, IDE integration
**Key Files**: `getting-started.md`, `concepts.md`, `configuration.md`, `service-deployment.md`, `ide-integration.md`, `api/lua/README.md`
**Phase 9-10 Additions**: Kernel modes, --trace flag, daemon deployment, DAP debugging, fleet management
**Start here if**: You want to write Lua scripts that use LLMs and tools, or deploy as services

---

### 🔧 [Developer Guide](developer-guide/) - *For Contributors*
**Purpose**: Technical guides for developers contributing to or extending rs-llmspell.

**Status**: ✅ Consolidated with kernel architecture patterns
**Contents**: Onboarding guide, extension patterns, production deployment, examples reference
**Key Files**: `developer-guide.md`, `extending-llmspell.md`, `production-guide.md`, `examples-reference.md`
**Phase 9-10 Additions**: Kernel integration patterns, global IO runtime usage, protocol implementation, daemon development
**Start here if**: You want to create custom tools, contribute code, or understand kernel architecture

---

### 🏗️ [Technical](technical/) - *For Architects*
**Purpose**: Core architectural documentation and implementation decisions.

**Status**: ✅ Updated for Phase 10 with kernel and debug architecture
**Contents**: Current architecture, master vision, kernel protocol architecture, debug DAP architecture, CLI command architecture
**Key Files**: `current-architecture.md`, `kernel-protocol-architecture.md`, `debug-dap-architecture.md`, `cli-command-architecture.md`
**Phase 9-10 Additions**: Integrated kernel design, global IO runtime, DAP bridge, multi-protocol support, event correlation
**Start here if**: You need to understand system architecture, protocols, or debugging infrastructure

---

### 📚 [Archives](archives/) - *Historical Reference*
**Purpose**: Preserve historical documents for reference.

**Status**: 📦 100+ documents archived
**Contents**: Phase handoff packages, superseded technical docs, consolidated guides, research notes
**Note**: These documents may be outdated but provide historical context

---

### 🚧 [In-Progress](in-progress/) - *Development Tracking*
**Purpose**: Track planning and implementation toward version 1.0.

**Status**: 📋 Phase 10 Complete, Phase 11 Planning
**Contents**: Phase completion documents (PHASE00-10 DONE), implementation roadmaps, design documents
**Key Files**: `implementation-phases.md` (16-phase roadmap), phase-specific design docs
**For**: Core team tracking progress

---

## What Rs-LLMSpell Actually Is

**Production-Ready Features** (v0.9.0):
- ✅ **Integrated Kernel Architecture** with global IO runtime (no "dispatch task is gone")
- ✅ **Production Daemon Mode** with systemd/launchd integration
- ✅ **Debug Adapter Protocol (DAP)** with IDE integration
- ✅ **Multi-Protocol Support** (Jupyter, DAP, LSP, REPL)
- ✅ **Fleet Management** for multiple kernel instances
- ✅ **40+ tools** across 9 categories (file, web, data, media, system)
- ✅ **17 crates** with unified kernel (consolidated from 20)
- ✅ **Lua scripting** with 17 zero-import globals (including Debug)
- ✅ **--trace flag** replacing --debug/--verbose
- ✅ **Signal handling** (SIGTERM, SIGHUP, SIGUSR1, SIGUSR2)
- ✅ **Event correlation** for request tracking
- ✅ **Multi-client support** with message routing
- ✅ **Connection file discovery** for IDE attachment
- ✅ **Agent infrastructure** with BaseAgent trait and builder patterns
- ✅ **4 workflow patterns** (Sequential, Parallel, Conditional, Loop)
- ✅ **RAG system** with HNSW vector search (<8ms @ 100K vectors)
- ✅ **Multi-tenant architecture** with StateScope::Custom isolation
- ✅ **State persistence** unified in kernel
- ✅ **Hook system** with 40+ points and circuit breakers
- ✅ **Event bus** with 90K+ events/sec throughput
- ✅ **60+ production examples** across 6 categories

**Phase 9-10 Achievements**:
- ✅ **Kernel Unification**: State, sessions, and debug merged into `llmspell-kernel`
- ✅ **Global IO Runtime**: Single Tokio runtime eliminates context issues
- ✅ **Service Deployment**: Double-fork daemon with PID management
- ✅ **IDE Integration**: VS Code, Jupyter Lab, vim/neovim support
- ✅ **Debug Infrastructure**: 10 essential DAP commands cover 95% needs
- ✅ **Production Safety**: Rate limiting, sanitization, health monitoring

**What it doesn't do**:
- ❌ GUI or web interface (CLI and library only)
- ❌ JavaScript support in kernel (Lua only currently)
- ❌ Python kernel support (planned for Phase 11)
- ❌ Distributed execution (planned for Phase 12)

---

## Quick Start Paths

### 🚀 **I want to use rs-llmspell**
1. **[Getting Started](user-guide/getting-started.md)** - 5-minute setup with kernel modes
2. **[Core Concepts](user-guide/concepts.md)** - Understand kernel, tools, agents, workflows
3. **[Service Deployment](user-guide/service-deployment.md)** - Deploy as system service
4. **[IDE Integration](user-guide/ide-integration.md)** - Connect VS Code or Jupyter
5. **[Lua API Reference](user-guide/api/lua/README.md)** - Complete API documentation
6. **[Examples](../examples/script-users/)** - 60+ working examples

### 🔨 **I want to extend rs-llmspell**
1. **[Developer Guide](developer-guide/developer-guide.md)** - Complete onboarding
2. **[Extending LLMSpell](developer-guide/extending-llmspell.md)** - Build tools, agents, protocols
3. **[Production Guide](developer-guide/production-guide.md)** - Deploy to production
4. **[Kernel Architecture](technical/kernel-protocol-architecture.md)** - Understand kernel design

### 🏛️ **I need architectural understanding**
1. **[Current Architecture](technical/current-architecture.md)** - 17 crates, Phase 10 complete
2. **[Kernel Protocol Architecture](technical/kernel-protocol-architecture.md)** - Protocols & transport
3. **[Debug DAP Architecture](technical/debug-dap-architecture.md)** - Debug infrastructure
4. **[Master Vision](technical/master-architecture-vision.md)** - 16-phase roadmap

### 🛠️ **I want to deploy in production**
1. **[Service Deployment](user-guide/service-deployment.md)** - systemd/launchd setup
2. **[Configuration Guide](user-guide/configuration.md)** - Daemon & kernel config
3. **[Troubleshooting](user-guide/troubleshooting.md)** - Debug production issues
4. **[Production Guide](developer-guide/production-guide.md)** - Best practices

---

## Phase 9-10 Documentation Achievements

### Phase 9: Kernel Architecture & Debug Support
- **Kernel Unification**: Merged state, sessions, debug into single `llmspell-kernel` crate
- **Global IO Runtime**: Documented solution to "dispatch task is gone" errors
- **DAP Bridge**: 10 essential commands with ~500 lines of code
- **Multi-Protocol**: Jupyter, DAP, LSP, REPL in single kernel
- **--trace Flag**: Unified logging control replacing --debug/--verbose
- **Event Correlation**: Request tracking with correlation IDs
- **Multi-Client**: Independent debug sessions per client

### Phase 10: Production Deployment
- **Service Deployment Guide**: Complete systemd/launchd documentation
- **Daemon Mode**: Double-fork technique with PID management
- **Signal Handling**: SIGTERM, SIGHUP, SIGUSR1, SIGUSR2 documentation
- **Fleet Management**: Multiple kernel instance coordination
- **IDE Integration Guide**: VS Code, Jupyter Lab, vim setup
- **Health Monitoring**: Prometheus-compatible metrics endpoints
- **Production Safety**: Rate limiting and data sanitization

### Consolidation Results
- **Crate Reduction**: 20 → 17 crates (15% reduction)
- **Unified Kernel**: State, sessions, debug in one place
- **Service Docs**: New deployment and IDE guides
- **Debug Docs**: Complete DAP architecture documentation
- **Examples**: Updated with kernel and service patterns

---

## Documentation Quality Standards

### Accuracy ✅
- All code examples tested with v0.9.0
- API documentation matches Phase 10 implementation
- Performance metrics from kernel measurements
- Architecture validated against unified kernel

### Organization ✅
- Clear separation: Kernel (how it works) vs Usage (how to use)
- Service deployment separate from development
- IDE integration documented independently
- Cross-references updated for Phase 10

### Maintenance 📋
- Version tracking (v0.9.0)
- Phase status clearly marked (Phase 10 Complete)
- Update dates: December 2024
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

### 📚 [Script Users](../examples/script-users/) - 50+ Lua Examples
- **Getting Started**: 6 progressive tutorials (hello world → kernel → RAG)
- **Kernel Examples**: Service mode, daemon deployment, multi-client
- **Debug Examples**: DAP integration, breakpoints, tracing
- **Features**: Agent, tool, workflow, state, provider examples
- **Cookbook**: 11 patterns (caching, RAG multi-tenant, error handling)
- **Applications**: 9 full applications (webapp creator, knowledge base, etc.)
- **Configs**: 15+ configuration examples including daemon configs

### 🦀 [Rust Developers](../examples/rust-developers/) - 10+ Reference Projects
- Kernel integration patterns
- Protocol implementation
- Custom transport layers
- Debug hook implementation
- Service wrapper creation
- Global IO runtime usage
- Event correlation patterns
- Multi-client handling

---

## Performance Metrics (Phase 10)

| Component | Metric | Target | Achieved |
|-----------|--------|--------|----------|
| Kernel Startup | Cold start | <200ms | <100ms ✅ |
| Message Processing | Latency | <10ms | <5ms ✅ |
| Debug Overhead | No breakpoints | <5% | <2% ✅ |
| DAP Commands | Response time | <5ms | <2ms ✅ |
| Multi-Client | Overhead per client | <10% | <5% ✅ |
| Event Correlation | Lookup time | <1ms | <0.5ms ✅ |
| Daemon Fork | Process creation | <50ms | <30ms ✅ |
| Signal Handling | Response time | <5ms | <2ms ✅ |
| Fleet Coordination | Sync overhead | <100ms | <50ms ✅ |

---

## Getting Help

**📋 Documentation Issues**: File GitHub issues for corrections
**❓ Usage Questions**: Check [Troubleshooting](user-guide/troubleshooting.md)
**🐛 Bug Reports**: Use GitHub issues with reproduction steps
**💡 Feature Requests**: Review [roadmap](in-progress/implementation-phases.md) first
**🤝 Contributing**: Start with [Developer Guide](developer-guide/)
**🚀 Deployment Help**: See [Service Deployment](user-guide/service-deployment.md)
**🔧 Debug Issues**: Check [Debug Architecture](technical/debug-dap-architecture.md)

---

**Last Updated**: December 2024 | **Version**: 0.9.0 (Phase 10 Complete) | **Next**: Phase 11 (Enhanced Observability & Python Support)