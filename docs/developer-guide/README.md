# Developer Guide

✅ **CURRENT**: Phase 8 Complete - RAG & Multi-Tenancy Operational
**Version**: 0.8.0 | **Crates**: 20 | **Tools**: 37+ | **Examples**: 60+

**Build and extend rs-llmspell with comprehensive developer documentation**

**🔗 Navigation**: [← Docs Hub](../) | [Project Home](../../) | [User Guide](../user-guide/) | [Technical Docs](../technical/)

---

## 🚀 Quick Start for Developers

> **📚 NOTICE**: This guide is for developers working on or extending rs-llmspell itself. For using rs-llmspell in your applications, see the [User Guide](../user-guide/).

**New contributor? Start here in 30 minutes:**

1. **Read**: [developer-guide.md](developer-guide.md) - Complete onboarding guide
2. **Study**: One of the 60+ examples in `examples/`
3. **Test**: Run `./scripts/quality/quality-check-fast.sh`
4. **Build**: Follow patterns in [extending-llmspell.md](extending-llmspell.md)

> 📚 **Scripts Documentation**: See [Scripts Overview](../../scripts/) for all automation tools

---

## 📖 The 4 Comprehensive Guides (Fully Consolidated)

### 1. **[Developer Guide](developer-guide.md)** 📘
**Foundation - Start Here (40% new content)**
- ✅ Quick start setup in 5 minutes
- ✅ Phase 8 architecture (20 crates including RAG)
- ✅ Core patterns: BaseAgent, sync bridge, **llmspell-utils** (NEW)
- ✅ Testing with llmspell-testing helpers
- ✅ Common tasks and workflows
- ✅ Performance requirements (<10ms tools, <8ms vector search)

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

---

## 🆕 What's New in Phase 8

### RAG System (Complete)
- **llmspell-rag**: RAG pipeline with embeddings
- **llmspell-storage**: HNSW vector storage (<8ms @ 100K vectors)
- **llmspell-tenancy**: Multi-tenant isolation (3% overhead)
- **Performance**: 80% embedding cache hit rate, 70% cost reduction

### By the Numbers
| Metric | Phase 7 | Phase 8 | Change |
|--------|---------|---------|--------|
| Crates | 17 | 20 | +3 RAG crates |
| Tools | 26 | 37+ | +11 tools |
| Examples | 40 | 60+ | +20 examples |
| Vector Search | N/A | <8ms @ 100K | NEW |
| Multi-tenant | N/A | 3% overhead | NEW |

---

## 🛠 Developer Workflow

### Essential Commands

> 📖 **Complete documentation**: See [Scripts README](../../scripts/) for all available scripts

```bash
# Quick checks (use frequently)
./scripts/quality/quality-check-minimal.sh  # <5 seconds - format, clippy
./scripts/quality/quality-check-fast.sh     # ~1 minute - adds unit tests

# Before PR (mandatory)
./scripts/quality/quality-check.sh          # 5+ minutes - full validation

# Component testing
./scripts/testing/test-by-tag.sh unit       # Unit tests only
./scripts/testing/test-by-tag.sh rag        # RAG tests
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

---

## 🏗 Current Architecture (Phase 8)

### 20 Crates Structure
```
Foundation Layer (10 crates):
├── llmspell-core         - BaseAgent trait, types
├── llmspell-utils        - Parameters, errors, responses
├── llmspell-storage      - HNSW vectors (Phase 8)
├── llmspell-security     - 3-level model
├── llmspell-config       - Configuration
├── llmspell-state-traits - State abstractions
├── llmspell-state-persistence - Persistence
├── llmspell-rag          - RAG pipeline (Phase 8)
├── llmspell-tenancy      - Multi-tenant (Phase 8)
└── llmspell-testing      - Test utilities

Application Layer (10 crates):
├── llmspell-tools        - 37+ built-in tools
├── llmspell-agents       - Agent infrastructure
├── llmspell-workflows    - 4 workflow types
├── llmspell-bridge       - Script integration
├── llmspell-hooks        - 40+ hook points
├── llmspell-events       - Event bus
├── llmspell-sessions     - Session management
├── llmspell-providers    - LLM providers
├── llmspell-cli          - CLI interface
└── llmspell-examples     - Example utilities
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

### Phase 9 (Next)
- Enhanced observability
- Advanced workflow patterns
- Extended provider support
- Performance optimizations

### Phase 10+ (Vision)
- Distributed execution
- Advanced caching
- Plugin marketplace
- Visual development tools

---

## 📞 Getting Help

- **Examples**: 60+ in `examples/` directory
- **Questions**: GitHub Discussions
- **Bugs**: GitHub Issues
- **Quick search**: `rg "pattern" --type rust`

---

## Summary

**Phase 8 Complete** with comprehensive developer documentation:

✅ **4 Consolidated Guides** covering everything developers need
✅ **20 Crates** with RAG, storage, and multi-tenancy
✅ **37+ Tools** with standardized patterns
✅ **60+ Examples** with learning paths
✅ **All Performance Targets Met**

Start with [developer-guide.md](developer-guide.md) for complete onboarding.

---

**Happy contributing to rs-llmspell!** 🚀

*For architecture details: [Technical Docs](../technical/)*
*For usage: [User Guide](../user-guide/)*