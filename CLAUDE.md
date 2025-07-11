# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚀 **Phase 2 - Built-in Tools Library**: IN PROGRESS (Started 2025-06-27)
- **Completed**: Phases 0, 1, and 25/25 Phase 2 tools ✅
- **Current**: Task 2.10.1 Script Integration Tests (8/9 subtasks complete)
- **Progress**: All 25 tools implemented, JSON API added, provider enhancements tested
- **Next**: Complete integration documentation (2.10.1.9), then proceed to 2.10.2 Security Validation

## Key Commands

```bash
# Quality Checks (MANDATORY before commits)
cargo clippy -- -D warnings            # Zero warnings policy
cargo fmt --all                        # Apply formatting
./scripts/quality-check-minimal.sh     # Quick check (seconds) - formatting, clippy, compilation
./scripts/quality-check-fast.sh        # Fast check (~1 min) - adds unit tests & docs
./scripts/quality-check.sh             # Full check (5+ min) - all tests & coverage

# Individual Checks
cargo test --workspace                 # Run all tests (can be slow)
cargo test --lib --all                 # Run only unit tests (faster)
cargo check --workspace                # Quick compilation check

# Phase 2 Specific
cargo test -p llmspell-tools          # Test tools crate
cargo bench -p llmspell-tools         # Benchmark tool performance
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

### Quality Check Scripts

Three levels of quality validation are available:

1. **Minimal Check** (`quality-check-minimal.sh`) - Runs in seconds
   - Code formatting verification
   - Clippy lints with zero warnings
   - Compilation check
   
2. **Fast Check** (`quality-check-fast.sh`) - Runs in ~1 minute
   - All minimal checks
   - Unit tests only
   - Documentation build verification
   
3. **Full Check** (`quality-check.sh`) - Runs in 5+ minutes
   - All fast checks
   - Full integration test suite
   - Optional coverage analysis (if cargo-tarpaulin installed)
   - Security audit (if cargo-audit installed)

**Recommendation**: Use minimal check before commits, fast check before pushing, and full check before PRs.

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