# rs-llmspell Project Instructions

These instructions apply to every agent harness. `CLAUDE.md` and `GEMINI.md` point here.

## Project Identity
rs-llmspell: **Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust

Experimental platform for rapid AI concept exploration via Lua/JavaScript scripting. Built with production-quality engineering (architecture, performance, testing) to make the transition from validated experiments to Rust production code as painless as possible.

## Critical References
- **Architecture**: `/docs/technical/master-architecture-vision.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
- **Phase records**: `/docs/in-progress/phase-NN-design-doc.md` (design) and `/docs/in-progress/PHASENN-DONE.md` (detailed implementation notes) for each phase
- **Latest phase**: `/docs/in-progress/phase-14-design-doc.md` and `/docs/in-progress/PHASE14-DONE.md` (Web Interface)
- **Successor design**: `/docs/rs-aikit-docs/` (design notes for a streamlined `rs-aikit`; no implementation yet)
- **Current Work**: read `/TODO.md` when it exists - hierarchical checkboxes with numbered tasks. It exists only while a phase is open; when a phase closes it is renamed to that phase's `PHASENN-DONE.md`. When there is no `/TODO.md`, no phase is open and the newest `PHASENN-DONE.md` is the record of the last one.

## Current Status
- **Released**: v0.14.1 (2025-12-13) - Phase 14, Web Interface ("Mission Control"): HTTP API, WebSocket streaming, embedded React UI, daemon mode. See `CHANGELOG.md` and `RELEASE_NOTES_v0.14.1.md`.
- **Phases 0-14**: complete. Phase 13c (v0.14.0) consolidated local storage on SQLite, added `vectorlite-rs`, centralized traits in `llmspell-core` and introduced layer-based profiles.
- **No phase is open.** The only work after v0.14.1 is the `rs-aikit` design notes. `implementation-phases.md` still labels Phase 14 as next; the changelog and `PHASE14-DONE.md` are the current record.

## Project-Specific Behavior Controls
- **Experimental mindset**: This is a rapid iteration platform for AI exploration, not production deployment tool
- **Script-first philosophy**: Lua/JS for velocity, Rust for proven patterns extraction
- **Extraction focus**: Document how experiments could transition to production when validated
- **Always Megathink**: analyze existing code, never guess
- **Always check docs when analyzing**: check docs/in-progress, docs/technical, docs/user-guide, docs/developer-guide, NOT docs/archives
- **Future-based thinking**: consider impact on later phases in implementation-phases.md
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
- Zero warnings policy: `cargo clippy --workspace --all-targets --all-features`

## Project-Specific Commands
- **"attack complexity"** = use megathink mode, no "for now" solutions
- **"phase impact?"** = analyze how change affects later phases
- **"trait implications?"** = consider trait hierarchy and dependencies
- **"benchmark this"** = check against performance targets above

## Crate Structure
The workspace members are listed in `Cargo.toml`. The main ones:
- `llmspell-core`: Core traits and types
- `llmspell-tools`: Tool implementations
- `llmspell-agents`: Agent infrastructure
- `llmspell-workflows`: Workflow patterns
- `llmspell-templates`: Experimental AI workflow templates (Phase 12)
- `llmspell-memory`: Adaptive memory system (Phase 13)
- `llmspell-graph`: Temporal knowledge graph (Phase 13)
- `llmspell-context`: Context engineering pipeline (Phase 13)
- `llmspell-bridge`: Script language integration
- `llmspell-kernel`: Script execution kernel
- `llmspell-storage`: State management and persistence (Phase 13c)
- `llmspell-web`: HTTP/WebSocket API server with embedded React UI (Phase 14)
- `llmspell-utils`: Shared utilities (use for ALL shared code)
- `llmspell-hooks`: Hook system with replay
- `llmspell-events`: Event system with correlation
- `vectorlite-rs`: Pure Rust HNSW vector search SQLite extension (Phase 13c.2.2a)

## Development Workflow
1. Read relevant phase doc + TODO.md
2. Write failing test using llmspell-testing helpers
3. Run ./scripts/quality/quality-check-fast.sh
4. Update TODO.md sub-tasks as completed

## Recent Releases
- **v0.14.1**: Web interface (HTTP API, WebSocket streaming, React UI, daemon mode), database path configuration, kernel execution mode refactoring
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
- Never include agent attribution (Claude or any other tool) in git commits or in code
