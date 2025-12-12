# rs-llmspell Project Instructions

## Project Identity
rs-llmspell: **Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust

Experimental platform for rapid AI concept exploration via Lua/JavaScript scripting. Built with production-quality engineering (architecture, performance, testing) to make the transition from validated experiments to Rust production code as painless as possible.

## Critical References
- **Architecture**: `/docs/technical/master-architecture-vision.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
- **Current Status**: v0.14.x - Phase 14 In Progress (Web Interface - through 14.6.7)
- **Phase 13c Design**: `/docs/in-progress/phase-13c-design-doc.md` (Storage Consolidation)
- **Phase 13c Completion**: `/docs/in-progress/PHASE13c-DONE.md` (Detailed implementation notes)
- **Phase 14 Design**: `/docs/in-progress/phase-14-design-doc.md` (Web Interface)
- **Current Work**: ALWAYS read `/TODO.md` - hierarchical checkboxes with numbered tasks

## Current Status (Phase 14 - In Progress)
- ✅ **Phases 7-13c**: Complete
- ✅ **Phase 13c**: Storage Consolidation & Usability → v0.14.0 COMPLETE
  - ✅ 13c.1: Dependencies cleanup (10+ deps removed)
  - ✅ 13c.2: SQLite Unified Local Storage (10 storage components)
    - vectorlite-rs (1,098 lines, pure Rust HNSW)
    - 10 SQLite backends (V3-V13): Vector, Graph, Procedural, Agent, KV, Workflow, Sessions, Artifacts, Events, Hooks
  - ✅ 13c.3: Trait Centralization (3,714 lines to llmspell-core)
  - ✅ 13c.4: Profile System Overhaul (18 layers + 20 presets)
  - ✅ 13c.5-13c.8: Examples, Validation, Documentation, Release
- ⏳ **Phase 14**: Web Interface (HTTP API + Web UI + WebSocket) (IN PROGRESS)
  - ✅ 14.1: Foundation & Crate Setup (llmspell-web crate, Axum server)
  - ✅ 14.2: HTTP Backend Implementation (REST API, WebSocket streaming)
  - ✅ 14.3: Frontend Integration (React/Vite, Monaco editor, embedded UI)
  - ✅ 14.4: Security & Daemon Integration (API Key/JWT auth, CLI lifecycle)
  - ✅ 14.5: Testing & Documentation (Integration tests, real config management)
  - ✅ 14.6.4.1: Scripts Execution Tab (Web console output)
  - ✅ 14.6.4.2: Fix Web Output Display (Real-time Lua print streaming)
  - ✅ 14.6.5: Kernel Execution Mode Refactoring (Eliminate dual waste)
  - ✅ 14.6.6: Fix Verbose Debug Output (rig-core 0.25→0.26 upgrade)
  - ✅ 14.6.7: Proper Database Path Configuration (WAL mode, home directory fallback)

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
- **Use `--test-threads=1`** if hitting "Too many open files" (OS limit)

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
- `llmspell-web`: HTTP/WebSocket API server with embedded React UI (Phase 14)
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
- **v0.14.x** (In Progress): Web interface (HTTP API, WebSocket streaming, React UI, daemon mode), database path configuration, kernel execution mode refactoring
- **v0.14.0**: Storage consolidation (SQLite unified), vectorlite-rs HNSW, layer-based profiles, 635+ tests
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