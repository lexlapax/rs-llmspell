# GEMINI.md

## Project Overview

**Project Identity:** rs-llmspell is a **Rapid AI Experimentation Platform** - Cast scripting spells to explore AI concepts, extract proven patterns to production-ready Rust.

This project, `rs-llmspell`, is an experimental platform for rapid AI concept exploration via Lua/JavaScript scripting. It enables quick iteration on AI ideas (LLMs, transformers, diffusion, memory, learning) with production-quality engineering (architecture, performance, testing) to ease the transition from validated experiments to Rust production code.

The platform is highly modular (21 crates) with feature flags for flexible builds. It supports multiple LLM providers and includes experimental multi-tenancy and security features.

## Key Documents

*   **Architecture**: `/docs/technical/master-architecture-vision.md`
*   **Implementation Phases**: `/docs/in-progress/implementation-phases.md` (23+ phase roadmap)
*   **Current Status**: v0.14.0 - Phase 13c Complete (Storage Consolidation & Usability)
*   **Phase 13c Design**: `/docs/in-progress/phase-13c-design-doc.md`
*   **Current Work**: Always read `/TODO.md` for active tasks.

## Recent Completion Status

*   ✅ **Phase 11**: Local LLM Integration (Ollama + Candle dual-path) - COMPLETE
*   ✅ **Phase 12**: Experimental Template System (10 workflows, CLI + Lua API) - COMPLETE
*   ✅ **Phase 13**: Experimental Memory & Context Engineering (3-tier memory) - COMPLETE
*   ✅ **Phase 13c**: Storage Consolidation & Usability (v0.14.0) - COMPLETE
    *   Dependencies cleanup (10+ deps removed)
    *   SQLite Unified Local Storage (10 storage components)
    *   Trait Centralization (3,714 lines to llmspell-core)
    *   Profile System Overhaul (18 layers + 20 presets)

## Current Focus: Phase 14 (Web Interface)

*   **Goal**: Unified, single-binary web interface (HTTP API + Web UI + WebSocket).
*   **Status**: Implementation In Progress.

## Development Philosophy & Behavior Controls

*   **Experimental mindset**: Rapid iteration platform, not just production deployment.
*   **Script-first philosophy**: Lua/JS for velocity, Rust for proven patterns.
*   **Extraction focus**: Document transitions to production.
*   **Always Megathink**: Analyze existing code, never guess.
*   **Future-based thinking**: Consider impact on future phases.
*   **Update TODO.md**: Mark sub-tasks complete as you do them.

### Strict Project Behaviors
*   **Less code is better**: REPLACE code, don't add.
*   **Less files are better**: Edit existing files, never create new ones unless critical.
*   **NO backward compatibility until 1.0**: Prioritize correctness.
*   **NO SHORTCUTS**: Holistic completion required.
*   **NO CLIPPY ALLOWS**: Fix properly.
*   **Never include attribution footers** (Google/Gemini/Antigravity) in git commits or code.

### Architecture Mandates
*   **Traits over dependencies**: Use Rust trait system.
*   **State-First Design**: Components communicate through shared state.
*   **Bridge-First**: Leverage existing crates.
*   **Script API Consistency**: Same surface across Lua/JS/Python.

## Crate Structure

*   `llmspell-core`: Core traits and types
*   `llmspell-tools`: Tool implementations
*   `llmspell-agents`: Agent infrastructure
*   `llmspell-workflows`: Workflow patterns
*   `llmspell-templates`: Experimental AI workflow templates
*   `llmspell-memory`: Adaptive memory system
*   `llmspell-graph`: Temporal knowledge graph
*   `llmspell-context`: Context engineering pipeline
*   `llmspell-bridge`: Script language integration
*   `llmspell-storage`: State management and persistence (Phase 13c)
*   `llmspell-utils`: Shared utilities
*   `llmspell-state-persistence`: State with persistence
*   `llmspell-hooks`: Hook system with replay
*   `llmspell-events`: Event system with correlation
*   `llmspell-sessions`: Session management with artifacts
*   `vectorlite-rs`: Pure Rust HNSW vector search SQLite extension

## Quality Gates (MANDATORY)

*   `./scripts/quality/quality-check-minimal.sh`: Runs in seconds.
*   `./scripts/quality/quality-check-fast.sh`: Takes about 1 minute.
*   `./scripts/quality/quality-check.sh`: Takes 5+ minutes.

## Testing System

*   Use `llmspell-testing` crate helpers ONLY.
*   Target >90% test coverage.
*   Run tests with `--test-threads=1` if you encounter "Too many open files" (OS resource limit).

## Performance Targets

*   Tool initialization: <10ms
*   Agent creation: <50ms
*   Hook overhead: <1%
*   State operations: <5ms write, <1ms read
*   Zero warnings policy.