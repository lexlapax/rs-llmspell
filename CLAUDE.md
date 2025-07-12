# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems
## Phase Status
- ✅ Phase 0: Foundation Infrastructure (COMPLETE)
- ✅ Phase 1: Core Execution Runtime (COMPLETE)
- ✅ Phase 2: Self-Contained Tools Library (COMPLETE - 25 tools)
- 🚀 **Phase 3: Tool Enhancement & Workflow Orchestration** (ACTIVE - Weeks 9-16)
  - ✅ Phase 3.0: Critical Tool Fixes (Weeks 9-10) - COMPLETE 2025-07-12
  - Phase 3.1: External Integration Tools (Weeks 11-12) - STARTING
  - Phase 3.2: Security & Performance (Weeks 13-14)
  - Phase 3.3: Workflow Orchestration (Weeks 15-16)
- ⏳ Phase 4: Vector Storage and Search (Weeks 17-18)
- ⏳ Phase 5+: Future phases...

## Current Status

🚀 **Phase 3.1 - External Integration Tools**: STARTING (Phase 3.0 Complete 2025-07-12)
- **Completed**: Phases 0, 1, 2, 3.0 ✅ (26 standardized tools with consistent interfaces)
- **Current**: Phase 3.1 - External Integration Tools (16 new tools)
- **Next Task**: Task 3.1.1 - WebSearchTool Enhancement
- **Achievements**: 95% parameter consistency, 95% DRY compliance, security hardening complete

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

# Phase 3 Specific
cargo test -p llmspell-tools          # Test tools crate
cargo test -p llmspell-utils          # Test shared utilities
cargo bench -p llmspell-tools         # Benchmark tool performance
cargo test --all-features             # Test with all external integrations
```

## Architecture Overview

**Core-Bridge-Script Architecture**: BaseAgent → Tool → Workflow hierarchy with scriptable interfaces.

**Tech Stack**: `rig` (LLM providers), `mlua` (scripting), `sled`/`rocksdb` (storage), comprehensive testing.

**Phase 3 Focus**: Standardize 25 existing tools, add 16 external integration tools, security hardening, workflow orchestration.

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
- **Current Progress**: `/docs/in-progress/PHASE03-TODO.md` - Phase 3 task tracking
- **Phase 3 Design**: `/docs/in-progress/phase-03-design-doc.md`
- **Breaking Changes**: Clean break approach with comprehensive documentation

## Phase 3 Plan (41+ Tools Target)

**Phase 3.0 (Weeks 9-10)**: Critical Tool Fixes ✅ COMPLETE
- ✅ Standardized all 26 tools to consistent interfaces
- ✅ Extracted shared utilities (DRY compliance 95% achieved)
- ✅ Implemented critical security fixes (Calculator DoS, path traversal)
- ✅ Created breaking changes documentation (CHANGELOG_v0.3.0.md)

**Phase 3.1 (Weeks 11-12)**: External Integration Tools (16 new)
- Web & Network: WebSearchTool enhancement, web_scraper, url_analyzer, api_tester, webhook_caller, webpage_monitor, sitemap_crawler
- Communication: email_sender, database_connector
- Rate limiting and circuit breaker patterns

**Phase 3.2 (Weeks 13-14)**: Advanced Security & Performance
- Comprehensive security hardening for all 41 tools
- Performance optimization (maintain 52,600x target)
- Resource limit enforcement

**Phase 3.3 (Weeks 15-16)**: Workflow Orchestration
- Sequential, Conditional, Loop, Streaming workflows
- State management and error handling
- Integration with full tool library

## Phase 3 Breaking Changes

**Clean Break Approach**: As a pre-1.0 project (v0.1.0 → v0.3.0), we're making breaking changes without migration tools.

**Key Changes**:
- **Parameter Standardization**: `input` as universal primary data parameter
- **Path Parameters**: `path: PathBuf` for single files, `source_path`/`target_path` for transforms
- **ResponseBuilder Pattern**: All tools use standardized response format
- **No Migration Tools**: Clear documentation and examples instead

**Documentation**: See `/docs/in-progress/CHANGELOG_v0.3.0.md` for complete breaking changes.

## Testing Strategy

- **Unit Tests**: Individual components
- **Integration Tests**: Tool interactions and script APIs  
- **Security Tests**: DoS protection, path traversal, resource limits
- **Performance**: <10ms tool initialization requirement
- **Coverage**: >90% enforced in CI