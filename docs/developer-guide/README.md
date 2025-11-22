# Developer Guide

✅ **CURRENT**: Phase 13 Complete - Experimental Memory & Context Engineering
**Version**: 0.13.0 | **Crates**: 21 | **Tools**: 40+ | **Templates**: 10 | **Examples**: 60+

**Build experimental AI components with production-quality patterns for painless extraction**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [User Guide](../user-guide/) | [Technical Docs](../technical/)

---

## 🚀 Quick Start for Developers

> **📚 NOTICE**: This guide is for developers working on or extending rs-llmspell itself. For using rs-llmspell in your applications, see the [User Guide](../user-guide/).

**New contributor? Start here in 30 minutes:**

1. **Read**: [01-getting-started.md](01-getting-started.md) - Clone, build, verify (5 min)
2. **⚠️ IMPORTANT**: [07-feature-flags.md](07-feature-flags.md) - Build system (Phase 10.17.5+)
3. **Study**: One of the 60+ examples in `examples/`
4. **Test**: Run `./scripts/quality/quality-check-fast.sh`
5. **Build**: Follow patterns in [03-extending-components.md](03-extending-components.md)

> 📚 **Scripts Documentation**: See [Scripts Overview](../../scripts/) for all automation tools

---

## 📖 The 8 Numbered Guides (Linear Learning Path)

**Start at 01, progress through 08 - each builds on the previous**

### [01. Getting Started](01-getting-started.md) 📘
**Setup, architecture, first contribution**
- Clone, build options (minimal/common/full), verify setup
- Architecture overview (21 crates: Foundation + Memory + Application)
- Core concepts: BaseAgent, sync bridge, llmspell-utils, security
- First contribution paths (7 learning tracks)
- Essential patterns: parameter extraction, error building, response building
- Performance targets: <10ms tools, <2ms memory, <8ms vector search

**Time**: 15 minutes | **Prerequisites**: None

### [02. Development Workflow](02-development-workflow.md) 🔄
**Testing, quality gates, git workflow, CI/CD**
- Testing system: categorization (unit/integration/external + component)
- Test patterns: tool, agent, RAG, memory, storage backend examples
- Quality gates: minimal/fast/full checks (./scripts/quality/)
- Git workflow: branch strategy, commit format, PR checklist
- CI/CD integration: GitHub Actions, Docker multi-stage builds
- Performance validation: profiling tools, benchmarks

**Time**: 30 minutes | **Prerequisites**: 01

### [03. Extending Components](03-extending-components.md) 🔧
**Tools, agents, hooks, workflows, templates, RAG, storage backends**
- Part 1: Tool Development (37+ patterns, BaseAgent + Tool trait)
- Part 2: Agent Development (LLM integration, provider abstraction)
- Part 3: Hook Development (security, caching, cross-cutting concerns)
- Part 4: Workflow Development (4 types: sequential, parallel, conditional, multi-agent)
- Part 5: RAG Extension (pipeline builder, custom embeddings, HNSW tuning)
- Part 6: Storage Backend Extension (StorageBackend trait, Redis/PostgreSQL)
- Part 7: Template Creation (AI workflow templates, proven patterns)

**Time**: 2-6 hours (depending on component) | **Prerequisites**: 01, 02

### [04. Bridge Patterns](04-bridge-patterns.md) 🌉
**Typed Rust structs for script-to-Rust configuration**
- Core principles: typed structs, parser separation, zero serialization overhead
- Anti-patterns eliminated (JSON → HashMap chains)
- Implementation checklist with step-by-step guidance
- Common reusable parsers (ContextScope, InheritancePolicy, ModelConfig)
- Complete examples from real implementations (6 tasks)
- Testing requirements and troubleshooting guide

**Time**: 2-3 hours | **Prerequisites**: 01, 03

### [05. Production Deployment](05-production-deployment.md) 🏭
**Security, performance, scaling, monitoring**
- Part 1: Security & Multi-Tenancy (3-level model, tenant isolation)
- Part 2: Performance & Scaling (HNSW tuning, connection pooling)
- Part 3: Deployment & Operations (Docker, Kubernetes, state persistence)
- Part 4: Monitoring & Observability (Prometheus, Grafana, tracing, health checks)
- Part 5: Performance Tuning (Tokio config, system tuning, profiling, optimization)

**Targets**: <10ms tools, <50ms agents, <8ms vector search, 99.9% uptime

**Time**: 4-8 hours | **Prerequisites**: 01, 02, 03

### [06. Tracing & Debugging](06-tracing-debugging.md) 🔍
**Comprehensive instrumentation guide**
- Structured tracing patterns across all components
- Session correlation and context propagation
- Performance optimization (<2% overhead at INFO level)
- Component-specific guidelines (tools, agents, workflows)
- Testing and environment configuration

**Time**: 1-2 hours | **Prerequisites**: 01, 02

### [07. Feature Flags](07-feature-flags.md) ⚠️
**Build system and feature flags (Phase 10.17.5+)**
- Modular build system (minimal/common/full)
- Binary size reduction (19MB minimal, 25MB common, 35MB full)
- Optional tool dependencies (templates, PDF, CSV, Excel, archives, email, DB)
- CI/CD and Docker migration steps
- Feature mapping and troubleshooting

**BREAKING CHANGE**: Feature flags required for builds since Phase 10.17.5

**Time**: 15 minutes | **Prerequisites**: None (read first if building)

### [08. Operations & Performance](08-operations.md) 🏭
**Operational guide for performance, security, and deployment**
- Performance overview and benchmarks (Phases 0-13)
- Security implementation (3-level model, multi-tenant isolation)
- Performance tuning (HNSW, state persistence, memory backends)
- Security operations (STRIDE mitigations, audit logging)
- Monitoring & observability (metrics, regression detection)
- Operational checklists (deployment, production config, incident response)

**Time**: 2-4 hours (reference) | **Prerequisites**: 01, 02, 05

---

## 📚 API Reference (Rust Crate Documentation)

**Complete Rust API documentation for extending llmspell**

### Thematic Guides (Start Here)

Consolidated guides covering multiple crates by topic:

1. **[Core Traits & Foundation](reference/core-traits.md)** - BaseAgent, ExecutionContext, testing framework
2. **[Storage Backends](reference/storage-backends.md)** - Vector storage, HNSW, backends, **export/import API (Phase 13c.3.2)**
3. **[RAG Pipeline & Context Engineering](reference/rag-pipeline.md)** - Document ingestion, retrieval, knowledge graph
4. **[Memory Backends](reference/memory-backends.md)** - Episodic, semantic, procedural memory systems
5. **[Security & Multi-Tenancy](reference/security-integration.md)** - Access control, sandboxing, tenant isolation
6. **[Crate Index](reference/crate-index.md)** - Quick reference to all 21 crates

### Generated API Documentation

```bash
# Generate complete workspace documentation
cargo doc --workspace --all-features --no-deps --open

# Generate for specific crate
cargo doc --package llmspell-core --all-features --open
```

---

## 🆕 What's New in Phase 13

### Experimental Memory & Context Engineering (Complete) ⭐
- **3-Tier Memory System**: Episodic (HNSW), Semantic (SQLite/PostgreSQL graph), Procedural (patterns)
- **Hot-Swappable Backends**: InMemory (dev), HNSW (8.47x speedup), SQLite/PostgreSQL (bi-temporal graph)
- **Context Engineering**: 4 strategies (episodic, semantic, hybrid, RAG) with parallel retrieval
- **<2ms Memory Overhead**: 50x faster than target, production-quality validation at scale
- **149 Tests**: 100% pass rate, zero warnings, comprehensive validation
- **Zero Breaking Changes**: Fully backward compatible, opt-in features

### Phase 12 Achievements: Experimental AI Workflows ⭐
- **10 Experimental Templates**: 6 base + 4 advanced workflows for rapid concept exploration
- **Template System**: llmspell-templates crate (2,651 LOC, 149 tests)
- **Template CLI**: 5 subcommands (list, info, exec, search, schema) for instant productivity
- **Template Global**: 16th of 18 Lua globals with 6 methods
- **20-50x Performance**: <2ms init, <1ms lookup, <0.1ms validation

### By the Numbers
| Metric | Phase 12 | Phase 13 | Change |
|--------|----------|----------|--------|
| Crates | 18 | 21 | +3 memory crates |
| Lua Globals | 18 | 18 | Memory + Context globals |
| Templates | 0 | 10 | NEW |
| Tests | 486 | 635 | +149 tests |
| Tool Init | <10ms | <10ms | Maintained |
| Memory Ops | N/A | <2ms | NEW (50x target) |

---

## 🛠 Developer Workflow

### Essential Commands

> 📖 **Complete documentation**: See [Scripts README](../../scripts/) for all available scripts

> ⚠️ **BREAKING CHANGE (Phase 10.17.5+)**: Feature flags required for builds. See [07-feature-flags.md](07-feature-flags.md)

```bash
# Build commands (Phase 10.17.5+)
cargo build --release --features common     # Recommended (25MB, templates+PDF)
cargo build --release --features full       # All tools (35MB)
cargo build --release                       # Minimal (19MB, core only)

# Quick checks (use frequently)
./scripts/quality/quality-check-minimal.sh  # <5 seconds - format, clippy
./scripts/quality/quality-check-fast.sh     # ~1 minute - adds unit tests

# Before PR (mandatory)
./scripts/quality/quality-check.sh          # 5+ minutes - full validation

# Component testing
./scripts/testing/test-by-tag.sh unit       # Unit tests only
./scripts/testing/test-by-tag.sh rag        # RAG tests
./scripts/testing/test-by-tag.sh memory     # Memory tests
./scripts/testing/test-by-tag.sh tool       # Tool tests
```

**Script Categories:**
- [Quality & CI](../../scripts/quality/) - Code quality, CI/CD pipelines
- [Testing](../../scripts/testing/) - Test execution, coverage
- [Utilities](../../scripts/utilities/) - Helper tools, easy launcher
- [Fleet](../../scripts/fleet/) - Kernel orchestration, monitoring

### Performance Requirements
| Component | Target | Status | Measure |
|-----------|--------|--------|---------|
| Tool init | <10ms | ✅ | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | ✅ | `cargo bench -p llmspell-agents` |
| Hook overhead | <2% | ✅ | Performance tests |
| Vector search | <8ms @ 100K | ✅ | `cargo bench -p llmspell-storage` |
| Multi-tenant | 3% overhead | ✅ | Integration tests |
| Template init | <2ms | ✅ | `cargo bench -p llmspell-templates` |
| Memory ops | <2ms overhead | ✅ | `cargo bench -p llmspell-memory` |

---

## 📚 Learning Paths

### 🎓 Tool Developer (2-3 hours)
```
1. 01-getting-started.md → Core Patterns
2. 03-extending-components.md → Part 1 (Tools)
3. examples/rust-developers/custom-tool-example/
4. Implement your tool
```

### 🧠 RAG Developer (4-5 hours)
```
1. 01-getting-started.md → Architecture
2. 03-extending-components.md → Part 5 (RAG)
3. examples/script-users/getting-started/05-first-rag.lua
4. examples/script-users/cookbook/rag-multi-tenant.lua
5. Build RAG features
```

### 🏢 Production Engineer (6-8 hours)
```
1. 01-getting-started.md → Overview
2. 05-production-deployment.md → All sections
3. examples/script-users/applications/
4. Deploy with monitoring
```

### 🌉 Bridge Developer (2-3 hours)
```
1. 01-getting-started.md → Core Patterns
2. 04-bridge-patterns.md → All sections
3. Review completed examples (Tasks 11a.8.1-11a.8.6)
4. Implement typed bridge method
```

### 🎯 Template Developer (3-4 hours) - Phase 12
```
1. 01-getting-started.md → Architecture
2. 03-extending-components.md → Part 7 (Templates)
3. Review 10 built-in templates in llmspell-templates/src/builtin/
4. Implement custom template following patterns
```

---

## 🏗 Current Architecture (Phase 13)

### 21 Crates Structure
```
Foundation Layer (8 crates):
├── llmspell-core         - BaseAgent trait, types
├── llmspell-utils        - Parameters, errors, responses
├── llmspell-storage      - HNSW vectors (Phase 8)
├── llmspell-security     - 3-level model
├── llmspell-config       - Configuration
├── llmspell-rag          - RAG pipeline (Phase 8)
├── llmspell-tenancy      - Multi-tenant (Phase 8)
└── llmspell-testing      - Test utilities

Memory Layer (3 crates - Phase 13):
├── llmspell-memory       - 3-tier memory system
├── llmspell-graph        - Temporal knowledge graph
└── llmspell-context      - Context engineering

Application Layer (10 crates):
├── llmspell-kernel       - Daemon, signals, Jupyter, DAP (Phase 10)
├── llmspell-tools        - 40+ built-in tools (feature flags)
├── llmspell-templates    - 10 built-in workflow templates (Phase 12) ⭐
├── llmspell-agents       - Agent infrastructure
├── llmspell-workflows    - 4 workflow types
├── llmspell-bridge       - Script integration (18 globals)
├── llmspell-hooks        - 40+ hook points
├── llmspell-events       - Event bus
├── llmspell-providers    - LLM providers
└── llmspell-cli          - CLI interface + tool/template commands
```

### Key Patterns

#### llmspell-utils (Used Everywhere)
```rust
use llmspell_utils::params::{extract_parameters, extract_required_string};
use llmspell_utils::error_builders::llmspell::{component_error, validation_error};
use llmspell_utils::response::ResponseBuilder;
```

#### BaseAgent (Universal Interface)
```rust
impl BaseAgent for YourComponent {
    fn metadata(&self) -> &ComponentMetadata { ... }
    async fn execute(&self, input: AgentInput, ctx: ExecutionContext) -> Result<AgentOutput> { ... }
}
```

#### Sync Bridge (Script Integration)
```rust
use llmspell_bridge::sync_utils::block_on_async;
let result = block_on_async::<_, T, E>("operation", async move { ... }, timeout)?;
```

---

## 🤝 Contributing

### Before You Start
1. **Study Examples**: 60+ production examples in `examples/`
2. **Use llmspell-utils**: Never duplicate parameter/error/response handling
3. **Follow Patterns**: Use existing patterns from 40+ tools, 10 templates

### Quality Requirements
- ✅ All tests categorized: `#[cfg_attr(test_category = "unit")]`
- ✅ Use llmspell-testing helpers (no duplicates)
- ✅ Run quality checks before commit
- ✅ Zero warnings policy
- ✅ >90% test coverage, >95% doc coverage

### Submission Process
1. Write failing test first
2. Implement with existing patterns
3. Run `./scripts/quality/quality-check-fast.sh`
4. Update documentation if needed
5. Submit PR with description

---

## 🗺 Roadmap

### Phase 13 (✅ Complete)
- 3-Tier Memory System (Episodic, Semantic, Procedural)
- Hot-Swappable Backends (InMemory, HNSW via vectorlite-rs, SQLite/PostgreSQL graph)
- Context Engineering (4 strategies with parallel retrieval)
- <2ms Memory Overhead (50x faster than target)

### Phase 12 (✅ Complete)
- Experimental AI Workflow Templates
- 10 built-in templates (6 base + 4 advanced)
- Template CLI with 5 subcommands
- Template Lua API (Template global, 16th of 18)
- 20-50x performance vs targets

### Phase 11 (✅ Complete)
- Local LLM Integration (Ollama + Candle)
- Bridge Consolidation (87% compile speedup)
- API Standardization (Tool.execute consistency)

### Phase 14+ (Vision)
- Advanced Memory Consolidation (A-TKG)
- Model Context Protocol (MCP) integration
- Language Server Protocol (LSP)
- Agent-to-Agent (A2A) communication
- Plugin marketplace

---

## 📞 Getting Help

- **Examples**: 60+ in `examples/` directory
- **Questions**: GitHub Discussions
- **Bugs**: GitHub Issues
- **Quick search**: `rg "pattern" --type rust`

---

## Summary

**Phase 13 Complete** with consolidated developer documentation:

✅ **8 Numbered Guides** from setup to production (linear learning path)
✅ **21 Crates** with Memory, Graph, Context, Templates, Kernel, RAG, and multi-tenancy
✅ **40+ Tools** with feature flag modularity (19-35MB builds)
✅ **10 Built-in Templates** solving real AI workflow problems
✅ **60+ Examples** with learning paths
✅ **All Performance Targets Exceeded** (20-50x faster for templates/memory)

Start with [01-getting-started.md](01-getting-started.md) for complete onboarding.

---

**Happy contributing to rs-llmspell!** 🚀

*For architecture details: [Technical Docs](../technical/)*
*For usage: [User Guide](../user-guide/)*
