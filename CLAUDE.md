# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems
## Phase Status
- ✅ Phase 0: Foundation Infrastructure (COMPLETE)
- ✅ Phase 1: Core Execution Runtime (COMPLETE)
- ✅ Phase 2: Self-Contained Tools Library (COMPLETE - 26 tools)
- 🚀 **Phase 3: Tool Enhancement & Agent Infrastructure** (ACTIVE - Weeks 9-16)
  - ✅ Phase 3.0: Critical Tool Fixes (Weeks 9-10) - COMPLETE 2025-07-11
  - ✅ Phase 3.1: External Integration Tools (Weeks 11-12) - COMPLETE 2025-07-16
  - ✅ Phase 3.2: Security & Performance (Weeks 13-14) - COMPLETE 2025-07-17
  - 🚧 **Phase 3.3: Agent Infrastructure & Basic Multi-Agent Coordination (Weeks 15-16) - IN PROGRESS**
- ⏳ Phase 4: Hook and Event System (Weeks 17-18)
- ⏳ Phase 5: Persistent State Management (Weeks 19-20)
- ⏳ Phase 6: Session and Artifact Management (Weeks 21-22)
- ⏳ Phase 7: Vector Storage (Weeks 23-24)
- ⏳ Phase 8: Advanced Workflow Features (Weeks 25-26)
- ⏳ Phase 9+: Future phases...

## Current Status

🚧 **Phase 3.3 - Agent Infrastructure & Basic Multi-Agent Coordination**: IN PROGRESS
- **Completed**: Phases 0, 1, 2, 3.0, 3.1, 3.2 ✅ (33+ standardized and secured tools)
- **Current Focus**: Implementing agent infrastructure foundation
- **Next Task**: Task 3.3.1 - Agent Factory Implementation
- **Phase 3.3 Scope**: 
  - Agent factory, registry, and lifecycle management
  - BaseAgent tool integration for agent-tool composition
  - Basic workflow patterns (Sequential, Conditional, Loop)
  - Script-to-agent integration bridge
  - Multi-agent coordination via workflows
- **Achievements**: 95% parameter consistency, 95% DRY compliance, comprehensive security hardening, 33+ production-ready tools

## Key Commands

```bash
# Quality Checks (MANDATORY before commits)
cargo clippy -- -D warnings            # Zero warnings policy
cargo fmt --all                        # Apply formatting
./scripts/quality-check-minimal.sh     # Quick check (seconds) - formatting, clippy, compilation
./scripts/quality-check-fast.sh        # Fast check (~1 min) - adds unit tests & docs
./scripts/quality-check.sh             # Full check (5+ min) - all tests & coverage

# Test Runners (See scripts/README.md for full documentation)
./scripts/test-by-tag.sh unit         # Run only unit tests
./scripts/test-by-tag.sh tool         # Run tool tests
./scripts/test-by-tag.sh external     # Run external/network tests
./scripts/list-tests-by-tag.sh all    # List test categories
SKIP_SLOW_TESTS=true ./scripts/quality-check.sh  # Skip slow tests

# Phase 3 Specific
cargo test -p llmspell-tools          # Test tools crate
cargo test -p llmspell-utils          # Test shared utilities
cargo bench -p llmspell-tools         # Benchmark tool performance

# Phase 3.3 Agent Development
cargo test -p llmspell-agents         # Test agents crate (when created)
cargo test -p llmspell-workflows      # Test workflows crate (when created)
cargo run --example agent-basic       # Run basic agent example
cargo run --example workflow-sequential # Run sequential workflow example
```

## Architecture Overview

**Core-Bridge-Script Architecture**: BaseAgent → Agent → Tool → Workflow hierarchy with full composition support.

**Tech Stack**: `rig` (LLM providers), `mlua` (scripting), `sled`/`rocksdb` (storage), comprehensive testing.

**Phase 3.3 Agent Infrastructure Focus**:
- **Agent Factory**: Flexible agent creation with configuration builders
- **Agent Registry**: Centralized discovery and management
- **BaseAgent Tool Integration**: Agents can discover and invoke tools
- **Basic Workflows**: Sequential, Conditional, and Loop patterns for multi-agent coordination
- **Script Bridge**: Lua/JavaScript access to agents and workflows
- **Composition**: Agents as tools, workflows using agents, full bidirectional integration

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

**Phase 3.3 (Weeks 15-16)**: Agent Infrastructure & Basic Multi-Agent Coordination ✅ STARTING
- Agent Factory, Registry, Lifecycle Management
- BaseAgent Tool Integration (agents can discover and use tools)
- Basic Workflow Patterns (Sequential, Conditional, Loop)
- Script-to-Agent Integration Bridge
- Multi-agent coordination via workflows

## Phase 3 Breaking Changes

**Clean Break Approach**: As a pre-1.0 project (v0.1.0 → v0.3.0), we're making breaking changes without migration tools.

**Key Changes**:
- **Parameter Standardization**: `input` as universal primary data parameter
- **Path Parameters**: `path: PathBuf` for single files, `source_path`/`target_path` for transforms
- **ResponseBuilder Pattern**: All tools use standardized response format
- **No Migration Tools**: Clear documentation and examples instead

**Documentation**: See `/docs/in-progress/CHANGELOG_v0.3.0.md` for complete breaking changes.

## Agent Implementation Guidelines (Phase 3.3)

### Core Agent Architecture
- **BaseAgent Trait**: Foundation trait that all components (Agent, Tool, Workflow) implement
- **Agent Trait**: Extends BaseAgent with LLM-specific capabilities
- **Tool Integration**: BaseAgent includes tool discovery and invocation methods
- **Composition Pattern**: Agents can wrap tools, tools can wrap agents

### Implementation Priorities
1. **Agent Factory** (Task 3.3.1): Start with flexible creation patterns
2. **Agent Registry** (Task 3.3.2): Enable discovery and management
3. **Tool Integration** (Task 3.3.3): Connect agents to 33+ existing tools
4. **Script Bridge** (Task 3.3.4): Enable Lua/JS access to agents
5. **Basic Workflows** (Tasks 3.3.12-3.3.14): Implement multi-agent coordination

### Key Design Principles
- **ADK Alignment**: Follow Google ADK patterns for multi-agent coordination
- **Tool Reuse**: Leverage all 33+ standardized tools from Phases 3.0-3.2
- **Memory-Based State**: Phase 3.3 uses in-memory state (persistence in Phase 5)
- **Script-First**: All agent capabilities must be accessible from Lua/JavaScript
- **Composition Over Inheritance**: Prefer composition patterns for flexibility

### Testing Requirements
- **Agent Unit Tests**: Test factory, registry, lifecycle independently
- **Tool Integration Tests**: Verify agents can invoke all 33+ tools
- **Workflow Tests**: Test Sequential, Conditional, Loop patterns
- **Script Integration Tests**: Verify Lua can create and invoke agents
- **Performance**: <50ms agent creation, <10ms tool invocation overhead

## Testing Strategy

- **Unit Tests**: Individual components
- **Integration Tests**: Tool interactions and script APIs  
- **Security Tests**: DoS protection, path traversal, resource limits
- **Performance**: <10ms tool initialization requirement
- **Coverage**: >90% enforced in CI