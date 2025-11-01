# GEMINI.md

## Project Overview

**Project Identity:** rs-llmspell is a **Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust.

This project, `rs-llmspell`, is an experimental platform for rapid AI concept exploration via Lua/JavaScript scripting. It enables quick iteration on AI ideas (LLMs, transformers, diffusion, memory, learning) with production-quality engineering (architecture, performance, testing) to ease the transition from validated experiments to Rust production code.

The platform is highly modular (21 crates) with feature flags for flexible builds. It supports multiple LLM providers and includes experimental multi-tenancy and security features.

## Key Documents

*   **Architecture**: `/docs/technical/master-architecture-vision.md`
*   **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
*   **Current Status**: v0.13.0 - Phase 13 Complete (Experimental Memory & Context Engineering)
*   **Phase 12 Design**: `/docs/in-progress/phase-12-design-doc.md` (Experimental Template System)
*   **Phase 13 Design**: `/docs/in-progress/phase-13-design-doc.md` (Experimental Memory System)
*   **Template Architecture**: `/docs/technical/template-system-architecture.md` (Template system design)
*   **Release Notes**: `/RELEASE_NOTES_v0.13.0.md` (comprehensive v0.13.0 changelog)
*   **Current Work**: Always read `/TODO.md` for active tasks.

## Recent Completion Status (Experimental Infrastructure Evolution)

*   ✅ **Phase 7**: Infrastructure Consolidation (536+ files refactored, centralized testing, config revolution)
*   ✅ **Phase 10**: Service Integration & IDE Connectivity (kernel, Jupyter, VS Code, LSP)
*   ✅ **Phase 11**: Local LLM Integration (Ollama + Candle dual-path) - COMPLETE
*   ✅ **Phase 11a**: Bridge Consolidation (87% compile speedup, API standardization, docs completeness) - COMPLETE
*   ✅ **Phase 11b**: Local LLM Cleanup (unified profiles, dual-architecture models, platform-aware GPU) - COMPLETE
*   ✅ **Phase 12**: Experimental Template System (10 workflows, CLI + Lua API, multi-agent patterns) - COMPLETE
*   ✅ **Phase 13**: Experimental Memory & Context Engineering (3-tier memory, hot-swap backends, context assembly) - COMPLETE

## Development Philosophy

*   **Experimental iteration focus**: Script-first rapid prototyping → validation → Rust extraction
*   **Less code is better**: REPLACE code, don't add (breaking changes are acceptable until version 1.0)
*   **Less files are better**: Edit existing files; do not create new ones unless critical
*   **NO backward compatibility until 1.0**: Prioritize correctness and simplicity
*   **NO SHORTCUTS**: Holistic completion is required; no TODOs for later
*   **Production-quality code**: For painless extraction when experiments are validated
*   Do not leave TODO comments in the code
*   Do not use `unwrap()` in production code

### Architecture Mandates

*   **Traits over dependencies**: Use the Rust trait system for modularity.
*   **State-First Design**: Components communicate through shared state.
*   **Bridge-First**: Leverage existing crates versus reimplementing.
*   **Script API Consistency**: The same surface across Lua/JS/Python.

## Crate Structure

*   `llmspell-core`: Core traits and types
*   `llmspell-tools`: Tool implementations
*   `llmspell-agents`: Agent infrastructure
*   `llmspell-workflows`: Workflow patterns
*   `llmspell-templates`: Experimental AI workflow templates (Phase 12)
*   `llmspell-memory`: Adaptive memory system (Phase 13)
*   `llmspell-graph`: Temporal knowledge graph (Phase 13)
*   `llmspell-context`: Context engineering pipeline (Phase 13)
*   `llmspell-bridge`: Script language integration
*   `llmspell-utils`: Shared utilities (use for ALL shared code)
*   `llmspell-state-persistence`: State with persistence
*   `llmspell-hooks`: Hook system with replay
*   `llmspell-events`: Event system with correlation
*   `llmspell-sessions`: Session management with artifacts

## Building and Running

### Building the Project

The project can be built using `cargo`. There are three main build configurations:

*   **Minimal (19MB)**: Includes core functionality with Lua scripting and essential tools.
    ```bash
    cargo build --release
    ```
*   **Common (25MB)**: Adds template engines and PDF processing.
    ```bash
    cargo build --release --features common
    ```
*   **Full (35MB)**: Includes all optional components, such as CSV/Parquet, Excel, archives, email, and database support.
    ```bash
    cargo build --release --features full
    ```

### Running Applications

The project includes a script for easily running applications, which is the recommended way for new users to get started.

```bash
./scripts/utilities/llmspell-easy.sh <application-name>
```

For example, to run the `file-organizer` application:

```bash
./scripts/utilities/llmspell-easy.sh file-organizer
```

**Advanced Users:**

You can also run applications directly with the `llmspell` binary:

```bash
llmspell app run <application-name>
```

### Using Templates (Experimental Workflows - v0.12.0)

Templates provide experimental AI workflows for rapid concept exploration, accessible via CLI or Lua:

**CLI Usage:**
```bash
# List available templates
llmspell template list

# Generate code with 3-agent pipeline
llmspell template exec code-generator \
  --param description="Fibonacci function" \
  --param language="rust" \
  --param model="ollama/llama3.2:3b"

# Research assistant with RAG
llmspell template exec research-assistant \
  --param topic="Rust async patterns" \
  --param max_sources=10
```

**Lua Usage:**
```lua
local result = Template.execute("code-generator", {
    description = "Calculate prime numbers",
    language = "rust",
    model = "ollama/llama3.2:3b"
})
```

## Development Workflow

1.  Read the relevant phase documentation and `TODO.md`.
2.  Write a failing test using `llmspell-testing` helpers.
3.  Implement the minimal solution.
4.  Run `./scripts/quality/quality-check-fast.sh`.
5.  Update sub-tasks in `TODO.md` as they are completed.

## Testing and Quality

### Quality Gates

The following scripts are mandatory to run before committing:

*   `./scripts/quality/quality-check-minimal.sh`: Runs in seconds (format, clippy, compile).
*   `./scripts/quality/quality-check-fast.sh`: Takes about 1 minute (adds unit tests & docs).
*   `./scripts/quality/quality-check.sh`: Takes 5+ minutes (full validation).

### Testing System

*   Use helpers from the **`llmspell-testing`** crate ONLY.
*   Testing is feature-based via Cargo features.
*   Target **>90% test coverage** and **>95% API documentation coverage**.

### Performance Targets

*   **Tool initialization**: <10ms
*   **Agent creation**: <50ms
*   **Hook overhead**: <1%
*   **State operations**: <5ms write, <1ms read
*   **Zero warnings policy**: `cargo clippy --workspace --all-target --all-features`

## v0.13.0 Key Achievements (Phase 13 - Experimental Infrastructure)

*   **3-Tier Memory System**: Episodic (HNSW), Semantic (SurrealDB graph), Procedural (patterns)
*   **Hot-Swappable Backends**: InMemory (dev), HNSW (8.47x speedup), SurrealDB (bi-temporal graph)
*   **Context Engineering**: 4 strategies (episodic, semantic, hybrid, RAG) with parallel retrieval
*   **<2ms Memory Overhead**: 50x faster than target, production-quality validation at scale
*   **149 Tests Passing**: 100% pass rate, zero warnings, comprehensive validation
*   **Zero Breaking Changes**: Fully backward compatible, opt-in features
*   **1,300+ Lines API Docs**: llmspell-memory, llmspell-graph, llmspell-context
*   **Memory + Context Globals**: 17th and 18th Lua globals for script access

## v0.12.0 Key Achievements (Phase 12 - Experimental Workflows)

*   **10 Experimental Templates**: Rapid AI concept exploration workflows
*   **Template System Core**: 2,847 lines (trait-based, DashMap registry, ExecutionContext builder)
*   **CLI + Lua Integration**: `template list|info|exec|search|schema` commands + Template global (16th global)
*   **Multi-Agent Workflows**: Real LLM integration (code-generator: 3 agents, data-analysis: 2 agents)
*   **<2ms Overhead**: 50x faster than target, production-quality engineering
*   **3,655 Lines Docs**: Complete architecture + 10 user guides + examples
*   **5 Template Categories**: Research, Development, Content, Productivity, Workflow
*   **Template Examples**: research-assistant, code-generator, content-generation, workflow-orchestrator, etc.

## Important Notes

*   When writing Lua scripts, always refer to `docs/user-guide/api/lua/README.md` for accurate API method names. **Do not guess**.
*   All tool calls use `Tool.execute(name, params)` - no alternative methods.
*   Template system (Experimental v0.12.0): Use `Template.execute(name, params)` for experimental workflows. See `docs/user-guide/templates/` for all 10 template guides.
*   Memory system (Experimental v0.13.0): Use `Memory` and `Context` globals for adaptive memory exploration. See `docs/user-guide/memory-configuration.md` for details.
*   Environment variables are now fully documented in `docs/user-guide/configuration.md` and `docs/user-guide/security-and-permissions.md`.