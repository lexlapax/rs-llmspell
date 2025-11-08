# 01: Getting Started with rs-llmspell Development

✅ **CURRENT**: Phase 13 Complete - Experimental Memory & Context Engineering
**Version**: 0.13.0 | **Crates**: 21 | **Tools**: 40+ | **Templates**: 10 | **Examples**: 60+

**Quick Navigation**: [Setup](#quick-start-5-minutes) | [Architecture](#architecture-21-crates) | [First Contribution](#your-first-contribution) | [Next Steps](#next-steps)

---

## Overview

rs-llmspell is a **rapid AI experimentation platform** built with production-quality engineering. This guide gets you up and running as a contributor in 5 minutes.

**Target Audience**: Developers working on or extending rs-llmspell itself. For using rs-llmspell in your applications, see the [User Guide](../user-guide/).

---

## Quick Start (5 minutes)

### 1. Clone & Build

```bash
# Clone repository
git clone <repository-url> && cd rs-llmspell

# Choose your build configuration:

# OPTION A: Minimal Build (19MB) - Core functionality
cargo build --release --bin llmspell

# OPTION B: Common Build (25MB) - Recommended for most developers
cargo build --release --bin llmspell --features common

# OPTION C: Full Build (35MB) - All features for complete development
cargo build --release --bin llmspell --features full
```

### 2. Verify Setup

```bash
# Quick quality checks (MANDATORY before commits)
./scripts/quality/quality-check-minimal.sh  # <5 seconds - format, clippy, compile
./scripts/quality/quality-check-fast.sh     # ~1 min - adds unit tests & docs
./scripts/quality/quality-check.sh          # 5+ min - full validation (before PR)

# Run example to verify
./target/release/llmspell run examples/script-users/getting-started/00-hello-world.lua
```

### 3. Feature Flags Reference

| Build Configuration | Binary Size | Tools Included | Use Case |
|---------------------|-------------|----------------|----------|
| **Minimal** (default) | 19MB | Core + essential tools | Production, containers |
| **Common** (`--features common`) | 25MB | + Templates, PDF | Most development |
| **Full** (`--features full`) | 35MB | + CSV, Excel, DB, Email | Complete toolkit |

**Individual Features**: `templates`, `pdf`, `csv-parquet`, `excel`, `json-query`, `archives`, `email`, `database`

**BREAKING CHANGE (Phase 10.17.5+)**: Feature flags required for builds. See [07-feature-flags.md](07-feature-flags.md) for migration guide.

---

## Architecture (21 Crates)

### Layered Structure

```
┌─────────────────────────────────────────────────────────────┐
│                   Lua/JS Scripts (Sync)                     │
│            60+ Examples in examples/*                       │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│          Bridge Layer (sync_utils::block_on_async)          │
│                    llmspell-bridge                          │
└─────────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────────┐
│              Rust Core (Async/Await)                        │
│                  21 Specialized Crates                      │
└─────────────────────────────────────────────────────────────┘
```

### Foundation Layer (8 crates)

| Crate | Purpose | Key Exports |
|-------|---------|-------------|
| **llmspell-core** | BaseAgent trait, types | `BaseAgent`, `Tool`, `AgentInput` |
| **llmspell-utils** | Parameter extraction, error builders | `extract_parameters`, `component_error` |
| **llmspell-storage** | HNSW vector storage | `VectorStorage`, `HNSWConfig` |
| **llmspell-security** | 3-level security model | `SecurityLevel`, `FileSandbox` |
| **llmspell-config** | Configuration management | `LLMSpellConfig` |
| **llmspell-rag** | RAG pipeline (Phase 8) | `RAGPipelineBuilder` |
| **llmspell-tenancy** | Multi-tenant isolation | `StateScope`, `TenantManager` |
| **llmspell-testing** | Centralized test utilities | `tool_helpers`, `agent_helpers` |

### Memory Layer (3 crates - Phase 13)

| Crate | Purpose | Key Exports |
|-------|---------|-------------|
| **llmspell-memory** | 3-tier memory system | `MemoryManager`, `EpisodicMemory` |
| **llmspell-graph** | Temporal knowledge graph | `TemporalKnowledgeGraph` |
| **llmspell-context** | Context assembly | `ContextBridge`, `HybridRetriever` |

### Application Layer (10 crates)

| Crate | Purpose | Key Exports |
|-------|---------|-------------|
| **llmspell-kernel** | Daemon, signals, Jupyter | `KernelServer`, `JupyterProtocol` |
| **llmspell-tools** | 40+ built-in tools | `FileReader`, `WebScraper` |
| **llmspell-templates** | 10 workflow templates | `Template`, `TemplateRegistry` |
| **llmspell-agents** | Agent infrastructure | `AgentConfig`, `AgentRegistry` |
| **llmspell-workflows** | 4 workflow types | `Workflow::sequential()` |
| **llmspell-bridge** | Script integration (18 globals) | `Tool`, `Agent`, `Template` (Lua) |
| **llmspell-hooks** | 40+ hook points | `Hook`, `HookManager` |
| **llmspell-events** | Event bus system | `EventBus` |
| **llmspell-providers** | LLM provider integration | `OpenAIProvider`, `OllamaProvider` |
| **llmspell-cli** | CLI interface | `tool` commands, `template` commands |

---

## Core Concepts (Must Know)

1. **BaseAgent Trait**: Everything implements BaseAgent (tools, agents, workflows)
2. **Sync Bridge Pattern**: Lua/JS are sync, Rust is async - bridge with `block_on_async()`
3. **llmspell-utils Patterns**: Parameter extraction, error builders, response builders
4. **Security Levels**: Safe, Restricted, Privileged - every tool must declare
5. **Test Categories**: unit, integration, external - always categorize
6. **RAG System (Phase 8)**: Vector storage, embeddings, multi-tenant isolation
7. **Memory System (Phase 13)**: Episodic (HNSW), Semantic (graph), Procedural (patterns)

---

## Your First Contribution

### Choose Your Path

| I want to... | Start here | Example to Study | Time |
|-------------|------------|------------------|------|
| Add a new tool | [03-extending-components.md](03-extending-components.md) → Part 1 | `examples/rust-developers/custom-tool-example/` | 30 min |
| Create an agent | [03-extending-components.md](03-extending-components.md) → Part 2 | `examples/rust-developers/custom-agent-example/` | 45 min |
| Build RAG features | [03-extending-components.md](03-extending-components.md) → Part 5 | `examples/script-users/cookbook/rag-multi-tenant.lua` | 60 min |
| Fix a bug | [02-development-workflow.md](02-development-workflow.md) → Bug Fix | Test patterns in `llmspell-testing` | 15 min |
| Add tests | [02-development-workflow.md](02-development-workflow.md) → Testing | `reference/core-traits.md` | 10 min |
| Add a template | [03-extending-components.md](03-extending-components.md) → Templates | `llmspell-templates/src/builtin/` | 3 hrs |
| Implement storage backend | [03-extending-components.md](03-extending-components.md) → Part 6 | `llmspell-storage/src/postgres/` | 6 hrs |

### Standard Development Workflow

```bash
# 1. Read relevant guide + TODO.md
cat TODO.md | grep "Phase"

# 2. Write failing test using llmspell-testing helpers
cd llmspell-tools  # or relevant crate
cargo test -p llmspell-tools --lib  # Should fail

# 3. Implement minimal solution
# (Edit source files)

# 4. Run quality checks (MANDATORY)
./scripts/quality/quality-check-fast.sh

# 5. Update TODO.md sub-tasks as completed
# (Mark checkboxes with [x])
```

---

## Essential Patterns

### llmspell-utils (Foundation for ALL Code)

**Parameter Extraction**:
```rust
use llmspell_utils::params::{extract_parameters, extract_required_string};

let params = extract_parameters(&input)?;
let operation = extract_required_string(params, "operation")?;
let timeout = extract_optional_u64(params, "timeout_ms").unwrap_or(30000);
```

**Error Building**:
```rust
use llmspell_utils::error_builders::llmspell::{component_error, validation_error};

return Err(validation_error("Path cannot be empty", Some("path".to_string())));
return Err(component_error(format!("Failed to connect: {}", e)));
```

**Response Building**:
```rust
use llmspell_utils::response::ResponseBuilder;

let response = ResponseBuilder::success("tool-name")
    .with_result(json!(result))
    .with_metadata("operation", json!(operation))
    .with_duration_ms(elapsed.as_millis() as u64)
    .build();
```

### Synchronous Bridge Pattern

```rust
use llmspell_bridge::sync_utils::block_on_async;

let result = block_on_async::<_, AgentInstance, LLMSpellError>(
    "agent_create",  // Operation name for debugging
    async move {
        bridge.create_agent(model_spec, config_json).await
    },
    Some(Duration::from_secs(30)),  // Optional timeout
)?;
```

### Test Categorization (MANDATORY)

```rust
#[tokio::test]
#[cfg_attr(test_category = "unit")]        // Speed: unit/integration/external
#[cfg_attr(test_category = "tool")]        // Component: tool/agent/workflow/etc
async fn test_my_feature() {
    use llmspell_testing::tool_helpers::create_test_tool_input;

    let tool = MyTool::new(Default::default());
    let input = create_test_tool_input(vec![
        ("operation", "test"),
    ]);

    let result = tool.execute(input, Default::default()).await;
    assert!(result.is_ok());
}
```

---

## Project Philosophy

### Code Principles (Overrides Normal Practices)

- **Less code is better** - REPLACE code, don't add (breaking changes OK until 1.0)
- **Less files are better** - Edit existing files, never create new ones unless critical
- **NO backward compatibility** until 1.0 - Prioritize correctness and simplicity
- **NO SHORTCUTS** - Holistic completion required, no TODOs for later
- **NO CLIPPY ALLOWS** - Fix properly if possible

### Architecture Mandates

- **Traits over dependencies** - Use Rust trait system for modularity
- **State-First Design** - Components communicate through shared state
- **Bridge-First** - Leverage existing crates vs reimplementing
- **Script API Consistency** - Same surface across Lua/JS/Python

### Quality Gates (MANDATORY)

```bash
# Before every commit
./scripts/quality/quality-check-minimal.sh     # seconds - format, clippy, compile

# Before PR (runs in CI)
./scripts/quality/quality-check-fast.sh        # 1 min - adds unit tests & docs
./scripts/quality/quality-check.sh             # 5+ min - full validation
```

### Performance Requirements

| Component | Target | Status | Measure |
|-----------|--------|--------|---------|
| Tool init | <10ms | ✅ | `cargo bench -p llmspell-tools` |
| Agent creation | <50ms | ✅ | `cargo bench -p llmspell-agents` |
| Hook overhead | <2% | ✅ | Performance tests |
| Vector search | <8ms @ 100K | ✅ | `cargo bench -p llmspell-storage` |
| Multi-tenant | 3% overhead | ✅ | Integration tests |
| Template init | <2ms | ✅ | `cargo bench -p llmspell-templates` |
| Memory operations | <2ms overhead | ✅ | `cargo bench -p llmspell-memory` |

---

## Next Steps

### Learning Paths

**🎓 Tool Developer** (2-3 hours):
```
1. This guide (01-getting-started.md)
2. [03-extending-components.md](03-extending-components.md) → Part 1 (Tools)
3. examples/rust-developers/custom-tool-example/
4. Implement your tool
```

**🧠 RAG Developer** (4-5 hours):
```
1. This guide (01-getting-started.md)
2. [03-extending-components.md](03-extending-components.md) → Part 5 (RAG)
3. examples/script-users/getting-started/05-first-rag.lua
4. examples/script-users/cookbook/rag-multi-tenant.lua
5. Build RAG features
```

**🏢 Production Engineer** (6-8 hours):
```
1. This guide (01-getting-started.md)
2. [05-production-deployment.md](05-production-deployment.md) → All sections
3. examples/script-users/applications/
4. Deploy with monitoring
```

**🌉 Bridge Developer** (2-3 hours):
```
1. This guide (01-getting-started.md)
2. [04-bridge-patterns.md](04-bridge-patterns.md) → All sections
3. Review completed examples (Tasks 11a.8.1-11a.8.6)
4. Implement typed bridge method
```

**🎯 Template Developer** (3-4 hours) - Phase 12:
```
1. This guide (01-getting-started.md)
2. [03-extending-components.md](03-extending-components.md) → Templates
3. Review 10 built-in templates in llmspell-templates/src/builtin/
4. Implement custom template following patterns
```

### The 7 Numbered Guides

**Linear path from setup to production**:

1. **[01-getting-started.md](01-getting-started.md)** (You are here) - Setup, architecture, first steps
2. **[02-development-workflow.md](02-development-workflow.md)** - Testing, quality gates, CI/CD, git workflow
3. **[03-extending-components.md](03-extending-components.md)** - Tools, Agents, Hooks, Workflows, Templates, RAG, Storage
4. **[04-bridge-patterns.md](04-bridge-patterns.md)** - Typed bridge patterns, Lua integration
5. **[05-production-deployment.md](05-production-deployment.md)** - Scaling, monitoring, performance, deployment
6. **[06-tracing-debugging.md](06-tracing-debugging.md)** - Logging, tracing, debugging
7. **[07-feature-flags.md](07-feature-flags.md)** - Build system, feature flags

### API Reference

**Thematic Guides** (Start Here):
- [Core Traits & Foundation](reference/core-traits.md) - BaseAgent, ExecutionContext, testing
- [Storage Backends](reference/storage-backends.md) - Vector storage, HNSW, backends
- [RAG Pipeline & Context](reference/rag-pipeline.md) - Document ingestion, retrieval
- [Memory Backends](reference/memory-backends.md) - Episodic, semantic, procedural
- [Security & Multi-Tenancy](reference/security-integration.md) - Access control, sandboxing
- [Crate Index](reference/crate-index.md) - Quick reference to all 21 crates

**Generated Docs**:
```bash
# Generate complete workspace documentation
cargo doc --workspace --all-features --no-deps --open
```

---

## Getting Help

- **Examples**: 60+ in `examples/` directory
- **Questions**: GitHub Discussions
- **Bugs**: GitHub Issues
- **Quick search**: `rg "pattern" --type rust`
- **Scripts Docs**: See [Scripts README](../../scripts/) for all automation tools

---

## Summary

**The Fast Path** (what most developers need):

1. ✅ Clone, build, run quality checks
2. ✅ Study relevant examples (60+ available)
3. ✅ Use llmspell-utils patterns (params, errors, responses)
4. ✅ Test with proper categorization
5. ✅ Run quality checks before commit
6. ✅ Submit PR

**Key Phase 13 Additions**:
- 21 crates (not 18) - added llmspell-memory, llmspell-graph, llmspell-context
- 3-tier memory system (episodic, semantic, procedural)
- Hot-swappable backends (InMemory, HNSW, SurrealDB)
- <2ms memory overhead (50x faster than target)
- 149 tests passing, zero warnings

**Remember**:
- Everything is BaseAgent
- Lua/JS sync ↔ Rust async via block_on_async
- llmspell-utils for ALL parameter/error/response handling
- Categorize ALL tests
- Study examples first

---

**Ready to contribute?** → Start with [02-development-workflow.md](02-development-workflow.md) for testing and quality gates.

**Want to extend?** → Jump to [03-extending-components.md](03-extending-components.md) for component development patterns.
