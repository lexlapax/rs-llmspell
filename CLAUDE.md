# rs-llmspell Project Instructions

## Project Identity
rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Critical References
- **Architecture Vision**: `/docs/technical/master-architecture-vision.md`
- **Current Architectur Overview**: `/docs/technical/current-architecture.md`
- **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (22-phase roadmap)
- **Current Status**: Phase 8 (check `/TODO.md` for active tasks)
- **Current Work**: ALWAYS read `/TODO.md` - hierarchical checkboxes with numbered tasks

## Project-Specific Behavior Controls
- **Always Ultrathink**: analyze existing code, never guess
- **Future-based thinking**: consider impact on phases 8-16 in implementation-phases.md
- **Update TODO.md**: mark sub-tasks complete as you do them, not after entire tasks

## Code Philosophy (Overrides Normal Practices)
- **Less code is better** - REPLACE code, don't add (breaking changes OK until 1.0)
- **Less files are better** - edit existing files, never create new ones unless critical
- **NO backward compatibility** until 1.0 - prioritize correctness and simplicity
- **NO SHORTCUTS** - holistic completion required, no TODOs for later

## Architecture Mandates
- **Traits over dependencies** - use Rust trait system for modularity
- **State-First Design** - components communicate through shared state
- **Bridge-First** - leverage existing crates vs reimplementing
- **Script API Consistency** - same surface across Lua/JS/Python

## Quality Gates (MANDATORY)
```bash
# MANDATORY before commits
./scripts/quality-check-minimal.sh     # seconds - format, clippy, compile
./scripts/quality-check-fast.sh        # 1 min - adds unit tests & docs  
./scripts/quality-check.sh             # 5+ min - full validation
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

## Banned Project Behaviors
- Creating new files (edit existing)
- Adding dependencies (use existing or llmspell-utils)
- Implementing features from future phases
- Leaving TODO comments in code
- Using unwrap() in production code
- Skipping tests to meet deadlines
- when writing lua scripts use  the `docs/user-guide/api/lua/README.md` to actually see what API/function calls to make in lua. do not guess.