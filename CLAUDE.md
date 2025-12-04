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

## Current Status (Phase 13c - Storage Consolidation)
- ✅ **Phases 7-13**: Complete
- 🚧 **Phase 13c**: Storage Consolidation → v0.14.0
  - ✅ 13c.1: Dependencies (-60MB)
  - ✅ 13c.2.0: Trait Architecture (3 traits, llmspell-core)
  - ✅ 13c.2.1: libsql Foundation (SqliteBackend, migrations)
  - ✅ 13c.2.2a: vectorlite-rs (1,098 lines, HNSW)
  - ✅ 13c.2.3/3a: Vector Storage (1,174 lines, MessagePack persistence, <2ms)
  - ✅ 13c.2.4: Graph Storage (1,230 lines, bi-temporal)
  - ✅ 13c.2.5: Procedural Storage (885 lines, V5)
  - ✅ 13c.2.6: State Storage (1,619 lines, V6/V7/V8, agent/KV/workflow)
  - ✅ 13c.2.7: Auxiliary Tables (3,598 lines, V9/V10/V11/V13, 31 tests, zero warnings)
  - ⏳ 13c.2.8: Legacy Removal + Graph Traversal Enhancement (NEXT)
  - Pending: 13c.2.9-13c.2.12 (Testing, Integration, Compatibility, Docs)

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
- `llmspell-storage`: State management and persistence (Phase 13c)
- `llmspell-utils`: Shared utilities (use for ALL shared code)
- `llmspell-state-persistence`: State with persistence
- `llmspell-hooks`: Hook system with replay
- `llmspell-events`: Event system with correlation
- `llmspell-sessions`: Session management with artifacts
- `vectorlite-rs`: Pure Rust HNSW vector search SQLite extension (Phase 13c.2.2a)

## Development Workflow
1. Read relevant phase doc + TODO.md
2. Write failing test using llmspell-testing helpers
3. Run ./scripts/quality-check-fast.sh
4. Update TODO.md sub-tasks as completed

## Recent Releases
- **v0.13.0**: 3-tier memory, hot-swap backends, context engineering, 149 tests
- **v0.12.0**: 10 templates, multi-agent workflows, CLI+Lua

## Strict Project Behaviors
- Do not Create new files (edit existing if possible)
- Do Not add dependencies (use existing or llmspell-utils if possible)
- Do not Implement features from future phases
- Never Leave TODO comments in code, implement in place
- No Using unwrap() in production code
- No Skipping tests to meet deadlines
- Never guess method names when writing Lua scripts, use `docs/user-guide/appendix/lua-api-reference.md` for API reference 
- Never include claude attribution footers in git commits or in code