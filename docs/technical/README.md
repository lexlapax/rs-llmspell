# Technical Documentation - LLMSpell v0.13.0

**Phase 13 Complete** - Adaptive Memory & Context Engineering

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [User Guide](../user-guide/) | [Developer Guide](../developer-guide/)

---

## Overview

> **📊 Technical Reference**: Comprehensive technical documentation for LLMSpell v0.13.0. Documentation structure optimized to 6 core guides covering the complete experimental system with production-quality engineering foundations. Phase 13 experimental infrastructure includes adaptive memory system, bi-temporal knowledge graph, context engineering, Unix daemon infrastructure, signal handling, tool CLI commands, fleet management, feature flags, and experimental AI workflows.

**Version**: v0.13.0 | **Status**: Experimental Platform with Production-Quality Foundations | **Last Updated**: January 2025

---

## Core Documentation (6 Essential Guides)

### 1. 📊 [Current Architecture](current-architecture.md) ✅ v0.13.0
**Purpose**: Overview and navigation hub
**Coverage**: Complete system architecture from Phase 0-13
**Key Content**:
- Component architecture (21 crates)
- Adaptive memory system (Phase 13: episodic, semantic, procedural)
- Daemon infrastructure and service support (Phase 10)
- Template system with 10 experimental workflows (Phase 12)
- Performance characteristics (all targets exceeded 10-50x)
- API surface (20 Lua globals including Memory, Context, Template, RAG)
- Feature flags and modular builds (19-35MB)

### 2. 🚀 [Kernel Protocol Architecture](kernel-protocol-architecture.md) ✅ v0.10.0
**Purpose**: Kernel and protocol implementation details
**Coverage**: IntegratedKernel with daemon infrastructure (Phase 10)
**Key Content**:
- IntegratedKernel design with global IO runtime
- Unix daemon implementation (double-fork, TTY detachment)
- Signal handling (SIGTERM/SIGINT → Jupyter messages)
- Protocol/Transport trait abstraction
- Jupyter v5.3 and DAP protocol implementations
- Fleet management architecture

### 3. 🔧 [CLI Command Architecture](cli-command-architecture.md) ✅ v0.12.0
**Purpose**: CLI design and command structure
**Coverage**: Complete CLI with kernel, tool, and template subcommands (Phase 10-12)
**Key Content**:
- Command hierarchy (kernel, tool, template, run subcommands)
- Kernel management (start, stop, status with daemon support)
- Tool CLI (list, info, invoke, search, test)
- Template CLI (list, info, exec, search, schema) ⭐ Phase 12
- Service installation (systemd/launchd)
- PID file and log rotation management

### 4. 🎯 [Architecture Decisions](architecture-decisions.md) ✅ v0.13.0
**Purpose**: All architectural decisions and rationale
**Coverage**: 46+ ADRs from Phase 0-13
**Key Content**:
- Foundation decisions (BaseAgent, async-first, trait-based modularity)
- Phase evolution and key reversals
- RAG system architecture (Phase 8)
- Daemon and service architecture (Phase 10)
- Feature flags and modular builds (Phase 10.17.5+)
- Fleet management via OS processes vs internal runtime
- Adaptive memory system (Phase 13: ADR-044 Bi-Temporal Graph, ADR-045 Consolidation Strategy, ADR-046 LLM-Driven Consolidation)
- Phase 13 design: [Phase 13 Design Document](../in-progress/phase-13-design-doc.md)

### 5. ⚡ [Operational Guide](operational-guide.md) ✅ v0.10.0
**Purpose**: Performance, security, and operations unified
**Coverage**: Complete operational reference for production
**Key Content**:
- Performance benchmarks (10-40% faster than targets)
- Security implementation (3-level model, multi-tenant)
- Monitoring and observability (structured tracing, <1ms overhead)
- Service deployment procedures (systemd, launchd, Docker)
- Operational checklists and health monitoring

### 6. 🔍 [RAG System Guide](rag-system-guide.md) ✅ v0.8.0 (Stable)
**Purpose**: Complete RAG documentation
**Coverage**: Phase 8 RAG implementation
**Key Content**:
- HNSW vector storage configuration (m, ef_construction, ef_search)
- Multi-tenant architecture with StateScope
- Performance tuning (8ms @ 100K vectors, 80% cache hit rate)
- RAGPipelineBuilder patterns and custom providers

## Supplementary Documentation

### 7. 🐛 [Debug DAP Architecture](debug-dap-architecture.md) ✅ v0.10.0
**Purpose**: Debug Adapter Protocol implementation
**Coverage**: IDE debugging support (Phase 10)
**Key Content**:
- DAPBridge with 10 essential DAP commands
- ExecutionManager state machine for pause/resume
- Breakpoint management and script integration
- Jupyter control channel tunneling
- VS Code compatibility

### 8. 📈 [Performance Baseline](performance-baseline.md) ✅ v0.10.0
**Purpose**: Performance benchmarks and baselines
**Coverage**: Phase 10 performance measurements
**Key Content**:
- Daemon startup metrics (1.8s, 10% faster than target)
- Message handling benchmarks (3.8ms, 24% faster)
- Tool initialization performance (7ms, 30% faster)
- All performance targets exceeded by 10-40%

### 9. 🔬 [Benchmarking Guide](benchmarking-guide.md) ✅ v0.10.0
**Purpose**: How to run and interpret benchmarks
**Coverage**: Benchmark methodology and tools
**Key Content**:
- Criterion benchmark setup
- Performance testing procedures
- Regression detection
- Automated kernel benchmarking script

### 10. 💪 [Stress Test Results](stress-test-results.md) ✅ v0.10.0
**Purpose**: Stress testing outcomes
**Coverage**: Load and stability testing
**Key Content**:
- Multi-client scenarios
- Long-running daemon tests (>24h uptime)
- Resource usage under load (42MB stable)
- Signal handling and graceful shutdown validation

### 11. 📋 [Protocol Compliance Report](protocol-compliance-report.md) ✅ v0.10.0
**Purpose**: Jupyter protocol compliance verification
**Coverage**: Wire protocol v5.3 validation
**Key Content**:
- 5-channel ZeroMQ compliance
- Message format validation
- Raw protocol testing (test_raw_zmq.py)
- jupyter_client compatibility issues (upstream bug)

### 12. 🔄 [MLua Upgrade Analysis](mlua-upgrade-analysis.md) ✅ v0.10.0
**Purpose**: MLua 0.9.9 → 0.11 upgrade impact
**Coverage**: Dependency upgrade assessment
**Key Content**:
- Breaking changes analysis (5 major changes)
- Performance impact evaluation (negligible)
- Migration effort estimation (3-5 days)
- Decision to revert rationale (timing, risk vs benefit)

### 13. 📖 [Master Architecture Vision](master-architecture-vision.md) 📚 Historical
**Purpose**: Original aspirational architecture (historical reference)
**Note**: 987K+ lines of original design vision, useful for understanding project evolution

---

## Quick Start Navigation

### Understanding the System
1. **Start**: [Current Architecture](current-architecture.md) - What we built (17 crates, Phase 10)
2. **Kernel**: [Kernel Protocol Architecture](kernel-protocol-architecture.md) - Daemon, protocols, service integration
3. **CLI**: [CLI Command Architecture](cli-command-architecture.md) - Commands including tool CLI
4. **Learn**: [Architecture Decisions](architecture-decisions.md) - Why we built it this way
5. **Operate**: [Operational Guide](operational-guide.md) - Performance, security, monitoring
6. **RAG**: [RAG System Guide](rag-system-guide.md) - Vector search with HNSW

### Specialized Topics
7. **Debug**: [Debug DAP Architecture](debug-dap-architecture.md) - IDE debugging support
8. **Performance**: [Performance Baseline](performance-baseline.md) - Benchmarks and metrics
9. **Benchmarking**: [Benchmarking Guide](benchmarking-guide.md) - How to run benchmarks
10. **Testing**: [Stress Test Results](stress-test-results.md) - Load and stability validation
11. **Protocols**: [Protocol Compliance Report](protocol-compliance-report.md) - Jupyter v5.3 compliance
12. **Dependencies**: [MLua Upgrade Analysis](mlua-upgrade-analysis.md) - Upgrade impact analysis

---

## Phase 13 Experimental Infrastructure (Production-Quality Engineering)

### System Architecture
- **21 crates** in workspace (added llmspell-memory, llmspell-graph, llmspell-context)
- **~70K lines** of Rust code (3,200+ LOC Phase 13)
- **40+ tools** with feature flag modularity
- **20 Lua globals** (including Memory, Context, Template, RAG, Debug)
- **10 built-in templates** (6 base + 4 advanced)
- **3-tier memory system** (episodic, semantic, procedural)
- **IntegratedKernel** with unified runtime
- **Feature flags** for modular builds (19-35MB)

### Daemon & Service Support (Phase 10)
- **Double-fork daemonization** with proper TTY detachment and session leadership
- **Signal handling** (SIGTERM/SIGINT → Jupyter shutdown messages, atomic operations)
- **PID file management** with lifecycle tracking and stale cleanup
- **Log rotation** with size/age-based policies (10MB default, 7 days retention)
- **systemd integration** for Linux deployment
- **launchd integration** for macOS deployment
- **Graceful shutdown** with resource cleanup guarantees

### Tool CLI Commands (Phase 10)
- **5 subcommands** (list, info, invoke, search, test)
- **Direct tool execution** without script overhead
- **ComponentRegistry access** via kernel message protocol
- **Runtime tool discovery** with automatic availability detection

### Template System (Phase 12 - Experimental Workflows) ⭐
- **10 experimental workflows** for rapid AI concept exploration
- **6 template categories** (Research, Chat, Analysis, CodeGen, Document, Workflow)
- **5 template CLI subcommands** (list, info, exec, search, schema)
- **6 Template Lua API methods** (Template global, 16th of 18)
- **TemplateRegistry** with DashMap-based concurrent storage
- **ExecutionContext** for infrastructure dependency injection
- **Parameter validation** with declarative schema constraints
- **Cost estimation** for pre-execution budget planning
- **20-50x performance** vs targets (<2ms init, <1ms lookup, <0.1ms validation)

### Fleet Management (Phase 10)
- **OS-level process isolation** for multi-kernel orchestration
- **Bash fleet manager** (spawn, stop, list, health operations)
- **Python advanced monitoring** with psutil integration
- **Docker orchestration** via docker-compose
- **Standard tooling** compatible with ps, kill, systemd workflows

### Protocol Support
- **Jupyter protocol v5.3** with 5-channel ZeroMQ architecture
- **DAP integration** for IDE debugging (10 essential commands)
- **Message correlation** with parent header tracking
- **Connection file** management
- **Heartbeat monitoring** for connection health

### Performance Highlights
| Metric | Target | Phase 12 Actual | Status |
|--------|--------|-----------------|--------|
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

### Testing Coverage
- **784 tests total** (kernel: 57, bridge: 334, CLI: 57, fleet: 38, templates: 149, memory: 149)
- **149 memory tests** (episodic, semantic, context, consolidation) ⭐ Phase 13
- **149 template tests** (122 unit + 27 integration) ⭐ Phase 12
- **29 daemon tests** for full lifecycle coverage
- **11 tool CLI tests** covering all subcommands
- **10 signal handling tests** for graceful shutdown
- **8 fleet tests** for multi-kernel scenarios
- **Zero warnings policy** enforced with clippy

### Documentation Updates
- **6 core guides** covering all aspects of Phase 10-13
- **8 supplementary guides** for specialized topics
- **3 new Rust API docs** for Phase 13 crates (llmspell-memory, llmspell-graph, llmspell-context)
- **Complete technical coverage** from architecture to scale validation
- **Extraction-ready** with production-quality engineering documented
- **Feature flags migration guide** for Phase 10.17.5+ builds
- **3,655 lines template docs** covering all 10 built-in templates ⭐ Phase 12
- **Memory configuration guide** and **RAG-memory integration** ⭐ Phase 13

---

## Related Documentation

### For Experimenters
- **[User Guide](../user-guide/)** - How to experiment with LLMSpell
- **[Service Deployment](../user-guide/service-deployment.md)** - Scale validation & deployment ⭐
- **[IDE Integration](../user-guide/ide-integration.md)** - IDE setup ⭐ NEW
- **[Lua API](../user-guide/api/lua/)** - Script reference
- **[Examples](../../examples/)** - Working examples

### For Developers
- **[Developer Guide](../developer-guide/)** - Contributing guide
- **[Rust API](../user-guide/api/rust/)** - Extension reference
- **[Implementation Phases](../in-progress/implementation-phases.md)** - 16-phase roadmap
- **[TODO](../../TODO.md)** - Current task tracking

### Historical Reference
- **[Phase 13 Design](../in-progress/phase-13-design-doc.md)** - Adaptive memory & context engineering
- **[Phase 12 Design](../in-progress/phase-12-design-doc.md)** - Production templates
- **[Phase 8 Design](../in-progress/phase-08-design-doc.md)** - RAG implementation
- **[Phase 9 Design](../in-progress/PHASE09-DONE.md)** - Kernel consolidation
- **[Master Vision](master-architecture-vision.md)** - Original architecture (historical)

---

## Document Status

### Core Documentation
| Document | Version | Status | Last Updated | Phase |
|----------|---------|--------|--------------|-------|
| current-architecture.md | v0.13.0 | ✅ Current | Jan 2025 | 13 |
| kernel-protocol-architecture.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| cli-command-architecture.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| architecture-decisions.md | v0.13.0 | ✅ Current | Jan 2025 | 13 |
| operational-guide.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| rag-system-guide.md | v0.8.0 | ✅ Stable | Aug 2024 | 8 |

### Supplementary Documentation
| Document | Version | Status | Last Updated | Phase |
|----------|---------|--------|--------------|-------|
| debug-dap-architecture.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| performance-baseline.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| benchmarking-guide.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| stress-test-results.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| protocol-compliance-report.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| mlua-upgrade-analysis.md | v0.10.0 | ✅ Current | Jan 2025 | 10 |
| master-architecture-vision.md | v0.1.0 | 📚 Historical | Aug 2024 | 0 |

## Migration Notes

### From Phase 9 to Phase 10
1. **Feature Flags**: Modular build system introduced (Phase 10.17.5+) - see [Feature Flags Migration](../developer-guide/feature-flags-migration.md)
2. **Daemon Infrastructure**: Double-fork daemonization, PID management, log rotation
3. **Signal Handling**: SIGTERM/SIGINT graceful shutdown via Jupyter messages
4. **Tool CLI**: 5 new subcommands for direct tool access without scripts
5. **Fleet Management**: OS-level process isolation for multi-kernel orchestration
6. **Enhanced Logging**: Rotating logs with structured tracing, <1ms overhead
7. **Build Changes**: Default build now minimal (19MB), use `--features common` for templates/PDF (25MB)

---

## Quick Reference

### Build Commands (Phase 10.17.5+)
```bash
# Minimal build (19MB, core only)
cargo build --release

# Common build (25MB, templates + PDF)
cargo build --release --features common

# Full build (35MB, all tools)
cargo build --release --features full
```

### Start Daemon
```bash
# Basic daemon
llmspell kernel start --daemon --port 9555

# With connection file for IDE integration
llmspell kernel start --daemon --connection-file ~/.llmspell/kernel.json
```

### Tool CLI Commands
```bash
# List all available tools
llmspell tool list

# Get tool information
llmspell tool info calculator

# Invoke tool directly
llmspell tool invoke calculator --params '{"expression": "2+2"}'
```

### Template CLI Commands ⭐ Phase 12
```bash
# List all available templates
llmspell template list

# Get template information
llmspell template info research-assistant

# Execute template
llmspell template exec code-generator \
    --param description="A function to validate email" \
    --param language="rust" \
    --param model="openai/gpt-4o-mini" \
    --output text

# Search templates
llmspell template search "code" --category codegen

# Get parameter schema
llmspell template schema research-assistant
```

### Fleet Management
```bash
# Spawn multiple kernels
./scripts/fleet/llmspell-fleet spawn config/openai.toml lua
./scripts/fleet/llmspell-fleet spawn config/anthropic.toml lua

# Check health
./scripts/fleet/llmspell-fleet health kernel-9555
```

### Install Service
```bash
# systemd (Linux)
llmspell kernel install-service --type systemd
sudo systemctl enable --now llmspell-kernel

# launchd (macOS)
llmspell kernel install-service --type launchd
launchctl load ~/Library/LaunchAgents/com.llmspell.kernel.plist
```

---

*Technical documentation for LLMSpell v0.13.0 after Phase 13 experimental memory & context engineering. Experimental platform with production-quality foundations: daemon mode, tool CLI, template system, adaptive memory, fleet management, and feature flags.*