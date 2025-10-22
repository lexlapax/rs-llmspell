# rs-llmspell Project Instructions

## Project Identity
rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

Production-ready AI workflow orchestration platform built in Rust for script-driven LLM coordination with RAG at scale.

## Critical References
- **Architecture**: `/docs/technical/master-architecture-vision.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
- **Current Status**: v0.12.0 - Phase 12 Complete, Ready for Phase 13 (Adaptive Memory System)
- **Phase 12 Design**: `/docs/in-progress/phase-12-design-doc.md` (Production Template System)
- **Template Architecture**: `/docs/technical/template-system-architecture.md` (Complete template system design)
- **Release Notes**: `/RELEASE_NOTES_v0.12.0.md` (comprehensive v0.12.0 changelog)
- **Current Work**: ALWAYS read `/TODO.md` - hierarchical checkboxes with numbered tasks

## Recent Completion Status
- ✅ **Phase 7**: Infrastructure Consolidation (536+ files refactored, centralized testing, config revolution)
- ✅ **Phase 10**: Service Integration & IDE Connectivity (kernel, Jupyter, VS Code, LSP)
- ✅ **Phase 11**: Local LLM Integration (Ollama + Candle dual-path) - COMPLETE
- ✅ **Phase 11a**: Bridge Consolidation (87% compile speedup, API standardization, docs completeness) - COMPLETE
- ✅ **Phase 11b**: Local LLM Cleanup (unified profiles, dual-architecture models, platform-aware GPU) - COMPLETE
- ✅ **Phase 12**: Production Template System (10 templates, CLI + Lua API, multi-agent workflows) - COMPLETE
- 🚧 **Phase 13**: Next - Adaptive Memory System (A-TKG temporal knowledge graph, template memory integration)

## Project-Specific Behavior Controls
- **Always Megathink**: analyze existing code, never guess,
- **Always check docs when analyzing** : check docs/in-process, docs/technical, docs/user-guide, docs/developer-guide, NOT docs/archives
- **Future-based thinking**: consider impact on phases 8-16 in implementation-phases.md
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
- `llmspell-templates`: Production AI workflow templates (NEW in v0.12.0)
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

## v0.12.0 Key Achievements (Phase 12)
- **10 Production Templates**: Turn-key AI workflows solving "0-day retention problem"
- **Template System Core**: 2,847 lines (trait-based, DashMap registry, ExecutionContext builder)
- **CLI + Lua Integration**: `template list|info|exec|search|schema` commands + Template global (16th)
- **Multi-Agent Workflows**: Real LLM integration (code-generator: 3 agents, data-analysis: 2 agents)
- **<2ms Overhead**: 50x faster than target (<100ms), production performance
- **149 Tests Passing**: 100% pass rate, zero warnings, comprehensive validation
- **3,655 Lines Docs**: Architecture + 10 user guides + examples
- **5 Template Categories**: Research, Development, Content, Productivity, Workflow

## Banned Project Behaviors
- Creating new files (edit existing)
- Adding dependencies (use existing or llmspell-utils)
- Implementing features from future phases
- Leaving TODO comments in code
- Using unwrap() in production code
- Skipping tests to meet deadlines
- When writing Lua scripts, use `docs/user-guide/api/lua/README.md` for API reference - DO NOT GUESS method names
- Never include attribution footers in git commits