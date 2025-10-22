# Developer Guide

✅ **CURRENT**: Phase 12 Complete - Production-Ready AI Agent Templates
**Version**: 0.12.0 | **Crates**: 18 | **Tools**: 40+ | **Templates**: 10 | **Examples**: 60+ | **Feature Flags**: Modular builds (19-35MB)

**Build and extend rs-llmspell with comprehensive developer documentation**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [User Guide](../user-guide/) | [Technical Docs](../technical/)

---

## 🚀 Quick Start for Developers

> **📚 NOTICE**: This guide is for developers working on or extending rs-llmspell itself. For using rs-llmspell in your applications, see the [User Guide](../user-guide/).

**New contributor? Start here in 30 minutes:**

1. **Read**: [developer-guide.md](developer-guide.md) - Complete onboarding guide
2. **⚠️ IMPORTANT**: [feature-flags-migration.md](feature-flags-migration.md) - Build system changes (Phase 10.17.5+)
3. **Study**: One of the 60+ examples in `examples/`
4. **Test**: Run `./scripts/quality/quality-check-fast.sh`
5. **Build**: Follow patterns in [extending-llmspell.md](extending-llmspell.md)

> 📚 **Scripts Documentation**: See [Scripts Overview](../../scripts/) for all automation tools

---

## 📖 The 8 Essential Guides (Fully Consolidated)

### 0. **[Feature Flags Migration](feature-flags-migration.md)** ⚠️
**BREAKING CHANGES - Read First (Phase 10.17.5+)**
- ✅ Modular build system (minimal/common/full)
- ✅ Binary size reduction (19MB minimal, 25MB common, 35MB full)
- ✅ Optional tool dependencies (templates, PDF, CSV, Excel, archives, email, DB)
- ✅ CI/CD and Docker migration steps
- ✅ Feature mapping and troubleshooting

*Essential for all developers: Build commands changed from Phase 10.17.5*

### 1. **[Developer Guide](developer-guide.md)** 📘
**Foundation - Start Here (Phase 12 updated)**
- ✅ Quick start setup in 5 minutes
- ✅ Phase 12 architecture (18 crates including Templates)
- ✅ Core patterns: BaseAgent, sync bridge, **llmspell-utils**
- ✅ Testing with llmspell-testing helpers
- ✅ Common tasks and workflows
- ✅ Performance requirements (<10ms tools, <8ms vector search, <2ms templates)

*Consolidates: main guide + synchronous-api-patterns + test-development-guide*

### 2. **[Extending LLMSpell](extending-llmspell.md)** 🔧
**Build Components (60% new content)**
- ✅ **Part 1**: Tool development (37+ patterns, sandbox requirements)
- ✅ **Part 2**: Agent development with providers
- ✅ **Part 3**: Hook system (logging, security, caching)
- ✅ **Part 4**: Workflow orchestration (4 types + multi-agent)
- ✅ **Part 5**: RAG system extension (ALL NEW)
  - RAGPipelineBuilder patterns
  - Custom embedding providers
  - VectorStorage trait implementation
  - HNSW tuning (m, ef_construction, ef_search)
  - Multi-tenant patterns with StateScope

*Consolidates: tool-development + hook-development + workflow-bridge + NEW RAG content*

### 3. **[Production Guide](production-guide.md)** 🏭
**Deploy with Confidence (50% new content)**
- ✅ **Part 1**: Security & Multi-Tenancy
  - 3-level security model (Safe/Restricted/Privileged)
  - StateScope::Custom("tenant:id") with 3% overhead
  - Input validation and secret management
- ✅ **Part 2**: Performance & Scaling
  - HNSW configurations for different scales
  - Memory management and connection pooling
  - 80% embedding cache hit rate patterns
- ✅ **Part 3**: Deployment & Operations
  - Docker and Kubernetes configurations
  - State persistence with backup strategies
- ✅ **Part 4**: Monitoring & Observability
  - Metrics, distributed tracing, health checks
  - Production checklist

*Consolidates: security-guide + session-artifact-implementation + NEW production patterns*

### 4. **[Examples Reference](examples-reference.md)** 📚
**Learn from 60+ Examples**
- ✅ Standards for writing examples
- ✅ Complete catalog with descriptions
- ✅ Learning paths (Beginner → RAG → Production)
- ✅ Pattern library from cookbook
- ✅ Running and debugging examples

*Updates: examples-standards with full catalog and learning paths*

### 5. **[Tracing Best Practices](tracing-best-practices.md)** 🔍
**Comprehensive Instrumentation Guide (Supplemental)**
- ✅ Structured tracing patterns across all components
- ✅ Session correlation and context propagation
- ✅ Performance optimization (<2% overhead at INFO level)
- ✅ Component-specific guidelines (tools, agents, workflows)
- ✅ Testing and environment configuration

*Essential for maintaining consistent, performant instrumentation across the codebase*

### 6. **[Template Creation Guide](template-creation.md)** 🎯
**Build Production-Ready AI Workflow Templates (Phase 12.10-12.13)**
- ✅ Quick start: minimum viable template in 50 LOC
- ✅ 4 proven patterns (code-review, content-generation, file-classification, knowledge-management)
- ✅ Best practices: validation, agent creation, error handling, testing
- ✅ Template categories and documentation requirements

*New in Phase 12: Comprehensive guide for creating custom templates with real-world patterns*

### 7. **[Bridge Pattern Guide](bridge-pattern-guide.md)** 🌉
**Typed Rust Structs for Script-to-Rust Configuration (Phase 11a.8)**
- ✅ Core principles: typed structs, parser separation, zero serialization overhead
- ✅ Anti-patterns eliminated (JSON → HashMap chains)
- ✅ Implementation checklist with step-by-step guidance
- ✅ Common reusable parsers (ContextScope, InheritancePolicy, ModelConfig)
- ✅ Complete examples from real implementations (6 tasks)
- ✅ Testing requirements and troubleshooting guide
- ✅ Design decisions reference (when to reuse vs create types)

*Critical for all bridge development: Eliminates JSON anti-patterns, ensures type safety, provides compile-time validation*

---

## 🆕 What's New in Phase 12

### Production-Ready AI Agent Templates (Complete) ⭐
- **10 Built-in Templates**: 6 base + 4 advanced workflows solving real AI problems
- **Template System**: llmspell-templates crate (2,651 LOC, 149 tests)
- **Template CLI**: 5 subcommands (list, info, exec, search, schema) for instant productivity
- **Template Global**: 16th of 18 Lua globals with 6 methods
- **TemplateRegistry**: DashMap-based concurrent template storage with Arc sharing
- **ExecutionContext**: Builder pattern for infrastructure dependency injection
- **Parameter Validation**: Declarative schema with type-safe constraints
- **Cost Estimation**: Pre-execution LLM usage budget planning
- **20-50x Performance**: <2ms init, <1ms lookup, <0.1ms validation

### By the Numbers
| Metric | Phase 10 | Phase 12 | Change |
|--------|----------|----------|--------|
| Crates | 17 | 18 | +1 templates crate |
| Lua Globals | 17 | 18 | +1 Template global |
| Templates | 0 | 10 | NEW |
| Tests | 486 | 635 | +149 template tests |
| Template Init | N/A | <2ms | NEW |
| Template Execute Overhead | N/A | <2ms | NEW (50x target) |
| Parameter Validation | N/A | <0.1ms | NEW (50x target) |

### Phase 10 Highlights (Service Integration & IDE Connectivity)
- **Unix Daemon Infrastructure**: Double-fork daemonization, PID management, log rotation, graceful shutdown
- **Signal Handling**: SIGTERM/SIGINT → Jupyter messages, atomic operations, resource cleanup
- **Tool CLI Commands**: 5 subcommands (list, info, invoke, search, test) for direct tool access
- **Fleet Management**: Bash/Python managers, Docker orchestration, OS-level process isolation
- **Enhanced Logging**: Rotating log files, structured tracing, <1ms overhead, multi-output support
- **Jupyter Protocol**: Wire protocol v5.3 with 5-channel ZeroMQ, message correlation, heartbeat
- **Debug Adapter Protocol**: 10 DAP commands (kernel-side), execution state machine, breakpoint support
- **Feature Flags**: Modular build system with minimal/common/full configurations (Phase 10.17.5+)

---

## 🛠 Developer Workflow

### Essential Commands

> 📖 **Complete documentation**: See [Scripts README](../../scripts/) for all available scripts

> ⚠️ **BREAKING CHANGE (Phase 10.17.5+)**: Feature flags required for builds. See [feature-flags-migration.md](feature-flags-migration.md)

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
./scripts/testing/test-by-tag.sh tool       # Tool tests
./scripts/testing/test-by-tag.sh kernel     # Kernel tests (Phase 10)
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
| Template init ⭐ | <2ms | ✅ | `cargo bench -p llmspell-templates` |
| Template execute | <2ms overhead | ✅ | `cargo bench -p llmspell-templates` |
| Parameter validation ⭐ | <0.1ms | ✅ | Unit tests |

---

## 📚 Learning Paths

### 🎓 Tool Developer (2-3 hours)
```
1. developer-guide.md → Core Patterns
2. extending-llmspell.md → Part 1 (Tools)
3. examples/rust-developers/custom-tool-example/
4. Implement your tool
```

### 🧠 RAG Developer (4-5 hours)
```
1. developer-guide.md → RAG System section
2. extending-llmspell.md → Part 5 (RAG)
3. examples/script-users/getting-started/05-first-rag.lua
4. examples/script-users/cookbook/rag-multi-tenant.lua
5. Build RAG features
```

### 🏢 Production Engineer (6-8 hours)
```
1. production-guide.md → All sections
2. examples/script-users/applications/
3. Deploy with monitoring
```

### 🌉 Bridge Developer (2-3 hours)
```
1. developer-guide.md → Core Patterns
2. bridge-pattern-guide.md → All sections
3. Review completed examples (Tasks 11a.8.1-11a.8.6)
4. Implement typed bridge method
```

### 🎯 Template Developer (3-4 hours) ⭐ Phase 12
```
1. developer-guide.md → Core Patterns
2. template-creation.md → All sections
3. Review 10 built-in templates in llmspell-templates/src/builtin/
4. Implement custom template following patterns
5. Add to TemplateRegistry and test with CLI/Lua
```

---

## 🏗 Current Architecture (Phase 12)

### 18 Crates Structure
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

Application Layer (10 crates): ⭐
├── llmspell-kernel       - Daemon, signals, Jupyter, DAP (Phase 10)
├── llmspell-tools        - 40+ built-in tools (feature flags)
├── llmspell-templates    - 10 built-in workflow templates (Phase 12) ⭐ NEW
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
3. **Follow Patterns**: Use existing patterns from 37 tools

### Quality Requirements
- ✅ All tests categorized: `#[cfg_attr(test_category = "unit")]`
- ✅ Use llmspell-testing helpers (no duplicates)
- ✅ Run quality checks before commit
- ✅ Zero warnings policy
- ✅ >90% test coverage, >95% doc coverage

### Submission Process
1. Write failing test first
2. Implement with existing patterns
3. Run `./scripts/quality-check-fast.sh`
4. Update documentation if needed
5. Submit PR with description

---

## 🗺 Roadmap

### Phase 10 (✅ Complete)
- Unix daemon infrastructure with double-fork
- Signal handling (SIGTERM/SIGINT → Jupyter messages)
- Tool CLI commands (direct invocation)
- Fleet management (OS-level processes)
- Jupyter Protocol v5.3 (5-channel ZeroMQ)
- Debug Adapter Protocol (DAP)
- Feature flags for modular builds (Phase 10.17.5+)

### Phase 11 (✅ Complete)
- Local LLM Integration (Ollama + Candle)
- Bridge Consolidation (87% compile speedup)
- API Standardization (Tool.execute consistency)
- Documentation Completeness (Security 40%→95%, Env Vars 0%→100%)

### Phase 12 (✅ Complete)
- Production-Ready AI Agent Templates
- 10 built-in templates (6 base + 4 advanced)
- Template CLI with 5 subcommands
- Template Lua API (Template global, 16th of 18)
- 20-50x performance vs targets

### Phase 13+ (Vision)
- Adaptive Memory System (A-TKG)
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

**Phase 12 Complete** with comprehensive developer documentation:

✅ **8 Essential Guides** including template creation and bridge pattern guides
✅ **18 Crates** with Templates, Kernel, RAG, storage, and multi-tenancy
✅ **40+ Tools** with feature flag modularity (19-35MB builds)
✅ **10 Built-in Templates** solving real AI workflow problems ⭐
✅ **60+ Examples** with learning paths
✅ **All Performance Targets Exceeded** (20-50x faster for templates)

Start with [developer-guide.md](developer-guide.md) for complete onboarding.

---

**Happy contributing to rs-llmspell!** 🚀

*For architecture details: [Technical Docs](../technical/)*
*For usage: [User Guide](../user-guide/)*