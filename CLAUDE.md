# rs-llmspell Project Instructions

## Project Identity
rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

Production-ready AI workflow orchestration platform built in Rust for script-driven LLM coordination with RAG at scale.

## Critical References
- **Architecture**: `/docs/technical/master-architecture-vision.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
- **Current Status**: v0.11.1 - Phase 11a Complete, Ready for Phase 12 (Adaptive Memory System)
- **Phase 11a Design**: `/docs/in-progress/phase-11a-design-doc.md` (1,685-line consolidation doc)
- **Release Notes**: `/RELEASE_NOTES_v0.11.1.md` (comprehensive v0.11.1 changelog)
- **Current Work**: ALWAYS read `/TODO.md` - hierarchical checkboxes with numbered tasks

## Recent Completion Status
- ✅ **Phase 7**: Infrastructure Consolidation (536+ files refactored, centralized testing, config revolution)
- ✅ **Phase 10**: Service Integration & IDE Connectivity (kernel, Jupyter, VS Code, LSP)
- ✅ **Phase 11**: Local LLM Integration (Ollama + Candle dual-path) - COMPLETE
- ✅ **Phase 11a**: Bridge Consolidation (87% compile speedup, API standardization, docs completeness) - COMPLETE
- 🚧 **Phase 12**: Starting - Adaptive Memory System (A-TKG temporal knowledge graph)

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

## v0.11.1 Key Achievements (Phase 11a)
- **87% Compile Speedup**: Bridge-only builds 38s→5s via Cargo feature gates (ADR-042)
- **API Standardization**: Tool.execute() consistent across all 40+ tools
- **Workflow Introspection**: WorkflowResult.agent_outputs for debugging (ADR-043)
- **Documentation Completeness**: Security 40%→95%, Env Vars 0%→100% (41+ variables)
- **Code Simplification**: 876 lines removed (StepType::Custom cleanup)
- **Lua API Documentation**: 100% accurate against llmspell-bridge implementation

## Banned Project Behaviors
- Creating new files (edit existing)
- Adding dependencies (use existing or llmspell-utils)
- Implementing features from future phases
- Leaving TODO comments in code
- Using unwrap() in production code
- Skipping tests to meet deadlines
- When writing Lua scripts, use `docs/user-guide/api/lua/README.md` for API reference - DO NOT GUESS method names
- Never include attribution footers in git commits