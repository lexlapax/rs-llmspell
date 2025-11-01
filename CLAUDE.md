# rs-llmspell Project Instructions

## Project Identity
rs-llmspell: **Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust

Experimental platform for rapid AI concept exploration via Lua/JavaScript scripting. Built with production-quality engineering (architecture, performance, testing) to make the transition from validated experiments to Rust production code as painless as possible.

## Critical References
- **Architecture**: `/docs/technical/master-architecture-vision.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
- **Current Status**: v0.13.0 - Phase 13 Complete (Experimental Memory & Context Engineering)
- **Phase 12 Design**: `/docs/in-progress/phase-12-design-doc.md` (Experimental Template System)
- **Phase 13 Design**: `/docs/in-progress/phase-13-design-doc.md` (Experimental Memory System)
- **Release Notes**: `/RELEASE_NOTES_v0.13.0.md` (comprehensive v0.13.0 changelog)
- **Current Work**: ALWAYS read `/TODO.md` - hierarchical checkboxes with numbered tasks

## Recent Completion Status (Experimental Infrastructure Evolution)
- ✅ **Phase 7**: Infrastructure Consolidation (536+ files refactored, centralized testing, config revolution)
- ✅ **Phase 10**: Service Integration & IDE Connectivity (kernel, Jupyter, VS Code, LSP)
- ✅ **Phase 11**: Local LLM Integration (Ollama + Candle dual-path) - COMPLETE
- ✅ **Phase 11a**: Bridge Consolidation (87% compile speedup, API standardization, docs completeness) - COMPLETE
- ✅ **Phase 11b**: Local LLM Cleanup (unified profiles, dual-architecture models, platform-aware GPU) - COMPLETE
- ✅ **Phase 12**: Experimental Template System (10 workflows, CLI + Lua API, multi-agent patterns) - COMPLETE
- ✅ **Phase 13**: Experimental Memory & Context Engineering (3-tier memory, hot-swap backends, context assembly) - COMPLETE

## Project-Specific Behavior Controls
- **Experimental mindset**: This is a rapid iteration platform for AI exploration, not production deployment tool
- **Script-first philosophy**: Lua/JS for velocity, Rust for proven patterns extraction
- **Extraction focus**: Document how experiments could transition to production when validated
- **Always Megathink**: analyze existing code, never guess
- **Always check docs when analyzing**: check docs/in-progress, docs/technical, docs/user-guide, docs/developer-guide, NOT docs/archives
- **Future-based thinking**: consider impact on phases 14+ in implementation-phases.md
- **Update TODO.md**: mark sub-tasks complete as you do them, not after entire tasks

## Code Philosophy (Overrides Normal Practices)
- **Less code is better** - REPLACE code, don't add (breaking changes OK until 1.0)
- **Less files are better** - edit existing files, never create new ones unless critical
- **NO backward compatibility** until 1.0 - prioritize correctness and simplicity
- **NO SHORTCUTS** - holistic completion required, no TODOs for later
- **NO CLIPPY ALLOWS** - fix properly if possible

## Architecture Mandates
- **Traits over dependencies** - use Rust trait system for modularity
- **State-First Design** - components communicate through shared state
- **Bridge-First** - leverage existing crates vs reimplementing
- **Script API Consistency** - same surface across Lua/JS/Python

## Quality Gates (MANDATORY)
```bash
# MANDATORY before commits
./scripts/quality/quality-check-minimal.sh     # seconds - format, clippy, compile
./scripts/quality/quality-check-fast.sh        # 1 min - adds unit tests & docs  
./scripts/quality/quality-check.sh             # 5+ min - full validation
```

## Testing System
- Use **llmspell-testing** crate helpers ONLY
- **Feature-based testing** via Cargo features
- **>90% test coverage, >95% API documentation coverage**

## Performance Targets
- Tool initialization: <10ms
- Agent creation: <50ms  
- Hook overhead: <1%
- State operations: <5ms write, <1ms read
- Zero warnings policy: `cargo clippy --workspace --all-target --all-features`

## Project-Specific Commands
- **"attack complexity"** = use megathink mode, no "for now" solutions
- **"phase impact?"** = analyze how change affects phases 8-16
- **"trait implications?"** = consider trait hierarchy and dependencies
- **"benchmark this"** = check against performance targets above

## Crate Structure
- `llmspell-core`: Core traits and types
- `llmspell-tools`: Tool implementations
- `llmspell-agents`: Agent infrastructure
- `llmspell-workflows`: Workflow patterns
- `llmspell-templates`: Experimental AI workflow templates (Phase 12)
- `llmspell-memory`: Adaptive memory system (Phase 13)
- `llmspell-graph`: Temporal knowledge graph (Phase 13)
- `llmspell-context`: Context engineering pipeline (Phase 13)
- `llmspell-bridge`: Script language integration
- `llmspell-utils`: Shared utilities (use for ALL shared code)
- `llmspell-state-persistence`: State with persistence
- `llmspell-hooks`: Hook system with replay
- `llmspell-events`: Event system with correlation
- `llmspell-sessions`: Session management with artifacts

## Development Workflow
1. Read relevant phase doc + TODO.md
2. Write failing test using llmspell-testing helpers
3. Implement minimal solution
4. Run ./scripts/quality-check-fast.sh
5. Update TODO.md sub-tasks as completed

## v0.13.0 Key Achievements (Phase 13 - Experimental Infrastructure)
- **3-Tier Memory System**: Episodic (HNSW), Semantic (SurrealDB graph), Procedural (patterns)
- **Hot-Swappable Backends**: InMemory (dev), HNSW (8.47x speedup), SurrealDB (bi-temporal graph)
- **Context Engineering**: 4 strategies (episodic, semantic, hybrid, RAG) with parallel retrieval
- **<2ms Memory Overhead**: 50x faster than target, production-quality validation at scale
- **149 Tests Passing**: 100% pass rate, zero warnings, comprehensive validation
- **Zero Breaking Changes**: Fully backward compatible, opt-in features
- **1,300+ Lines API Docs**: llmspell-memory, llmspell-graph, llmspell-context
- **Memory + Context Globals**: 17th and 18th Lua globals for script access

## v0.12.0 Key Achievements (Phase 12 - Experimental Workflows)
- **10 Experimental Templates**: Rapid AI concept exploration workflows
- **Template System Core**: 2,847 lines (trait-based, DashMap registry, ExecutionContext builder)
- **CLI + Lua Integration**: `template list|info|exec|search|schema` commands + Template global (16th)
- **Multi-Agent Workflows**: Real LLM integration (code-generator: 3 agents, data-analysis: 2 agents)
- **<2ms Overhead**: 50x faster than target, production-quality engineering
- **3,655 Lines Docs**: Architecture + 10 user guides + examples

## Banned Project Behaviors
- Creating new files (edit existing)
- Adding dependencies (use existing or llmspell-utils)
- Implementing features from future phases
- Leaving TODO comments in code
- Using unwrap() in production code
- Skipping tests to meet deadlines
- When writing Lua scripts, use `docs/user-guide/api/lua/README.md` for API reference - DO NOT GUESS method names
- Never include attribution footers in git commits