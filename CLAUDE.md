# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚀 **Phase 2 - Built-in Tools Library**: IN PROGRESS (Started 2025-06-27)
- **Completed**: Phases 0, 1, and 25/25 Phase 2 tools ✅
- **Current**: Working on task 2.10.4 (Performance Benchmarking), then returning to 2.10.1 (Script Integration Tests)
- **Progress**: All 25 tools implemented, refactoring complete (Task 2.9.3 ✅)

## Key Commands

```bash
# Quality Checks (MANDATORY before commits)
cargo clippy -- -D warnings      # Zero warnings policy
cargo fmt                       # Apply formatting
cargo test --workspace          # Run all tests
./scripts/quality-check.sh      # Run all quality checks locally

# Phase 2 Specific
cargo test -p llmspell-tools    # Test tools crate
cargo bench -p llmspell-tools   # Benchmark tool performance
```

## Architecture Overview

**Core-Bridge-Script Architecture**: BaseAgent → Tool → Workflow hierarchy with scriptable interfaces.

**Tech Stack**: `rig` (LLM providers), `mlua` (scripting), `sled`/`rocksdb` (storage), comprehensive testing.

**Phase 2 Focus**: 25 self-contained tools with provider enhancements (ModelSpecifier, base URL overrides).

## Quality Requirements

- **Zero Warnings**: All code must compile without warnings
- **Test Coverage**: >90% coverage enforced in CI
- **Documentation**: >95% coverage requirement
- **CI/CD**: All quality gates implemented and enforced

## Critical Implementation Principles

- **State-First**: Agents communicate through shared state
- **Tool Composition**: Agents can be wrapped as tools
- **Security First**: Sandboxing and resource limits enforced
- **DRY Principle**: Use llmspell-utils for shared functionality

## Key Development Reminders

- **Complete Tasks Fully**: No lazy implementations, check Definition of Done
- **DRY**: Use llmspell-utils for common functionality
- **Follow TODO.md**: Stick to task hierarchy, don't jump ahead
- **Zero Warnings**: Maintain compilation without warnings
- **Update Progress**: Keep TODO.md timestamps current

## Primary Documentation

- **Architecture**: `/docs/technical/rs-llmspell-final-architecture.md`
- **Current Progress**: `/TODO.md` - Phase 2 task tracking (25/25 tools complete)
- **Phase 2 Design**: `/docs/in-progress/phase-02-design-doc.md`

## Phase 2 Progress (25/25 Tools Complete) ✅

**All Tools Implemented and Refactored**:
- ✅ Data Processing: JsonProcessor, CsvAnalyzer, HttpRequest, GraphQLQuery
- ✅ File System: FileOperations, ArchiveHandler, FileWatcher, FileConverter, FileSearch
- ✅ System Integration: EnvironmentReader, ProcessExecutor, ServiceChecker, SystemMonitor
- ✅ Media Processing: AudioProcessor, VideoProcessor, ImageProcessor
- ✅ Utilities: TemplateEngine, DataValidation, TextManipulator, UuidGenerator, HashCalculator, Base64Encoder, DiffCalculator, DateTimeHandler, Calculator
- ✅ Search: WebSearch

**Current Tasks**:
- Task 2.10.4: Documentation and Examples (IN PROGRESS - Phase 5: Testing examples)
- Task 2.10.1: Script Integration Tests (AFTER 2.10.4)

## Testing Strategy

- **Unit Tests**: Individual components
- **Integration Tests**: Tool interactions and script APIs
- **Security Tests**: Sandbox escape prevention
- **Performance**: <10ms tool initialization requirement
- **Coverage**: >90% enforced in CI