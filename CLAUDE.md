# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚀 **Phase 1 - Core Execution Runtime**: IN PROGRESS (Started 2025-06-26)
- **Completed**: All architectural research phases (1-13) ✅
- **Completed**: Phase 0 implementation - All 37 tasks ✅
- **Completed**: Architecture updates for streaming/multimodal ✅
- **Current**: Phase 1 implementation - Core runtime with Lua 🔄
- **Next**: Working Lua scripts calling LLM agents

### Phase 0 Achievements
- ✅ **12-crate workspace** with zero compiler warnings
- ✅ **165 comprehensive tests** (unit, integration, property, doc tests)
- ✅ **Complete CI/CD pipeline** with 7 jobs and quality gates
- ✅ **Professional documentation** (>95% coverage, GitHub Pages ready)
- ✅ **Architecture enhanced** with streaming and multimodal support

### Phase 1 Implementation Focus
- 🔄 **13th crate**: `llmspell-utils` for shared utilities
- 🔄 **Streaming support**: BaseAgent and Tool traits extended
- 🔄 **Multimodal types**: MediaContent (Image, Audio, Video, Binary)
- 🔄 **Lua runtime**: Basic script execution with agent APIs
- 🔄 **CLI enhancement**: Streaming output with progress indicators

## Phase 1 Development Commands

**Current Focus**: Core Execution Runtime (10 working days)

```bash
# Phase 1 Development Commands
cargo check --workspace          # Verify workspace compilation
cargo build --workspace          # Build all foundation crates
cargo test --workspace           # Run foundation tests
cargo doc --workspace --no-deps  # Generate documentation

# Quality Assurance (MANDATORY before commits)
cargo clippy -- -D warnings      # Zero warnings policy
cargo fmt --check               # Formatting validation
cargo fmt                       # Apply formatting
cargo test --workspace          # Run all tests

# Local Quality Check Script (RECOMMENDED)
./scripts/quality-check.sh       # Run all quality checks locally (matches CI)

# Documentation Validation Tools
cargo install cargo-deadlinks    # Install documentation link checker
cargo install markdown-link-check # Install markdown link validator
cargo deadlinks --dir target/doc # Check internal documentation links
markdown-link-check README.md    # Validate README links

# Phase 0 Specific Tasks
cargo metadata                   # Verify workspace structure
cargo tree                      # Check dependency graph

# CI/CD Pipeline (IMPLEMENTED - Phase 0 Complete)
.github/workflows/ci.yml         # Automated quality checks
.github/QUALITY_GATES.md         # Branch protection and quality standards
.github/CI_VALIDATION_REPORT.md  # CI/CD pipeline validation results
.github/PHASE0_COMPLETION_REPORT.md # Phase 0 completion validation
.markdown-link-check.json        # Link validation configuration

# Clean workspace
cargo clean
```

## Architecture Overview

Rs-LLMSpell is a **production-ready scriptable LLM interaction framework** that revolutionizes AI application development through a unique Core-Bridge-Script architecture.

### Component Hierarchy

```
BaseAgent ← Agent ← SpecializedAgent (Research, Analysis, etc.)
    ↑
  Tool ← ToolWrappedAgent (Agents as Tools)
    ↑  
Workflow ← SequentialWorkflow, ParallelWorkflow, ConditionalWorkflow
```

### Key Design Elements

1. **BaseAgent**: Foundation trait providing tool-handling capabilities, state management, and hook integration
2. **Agent**: LLM wrapper extending BaseAgent with specialized prompts and provider integration
3. **Tool**: LLM-callable functions that can wrap agents for composition
4. **Workflow**: Deterministic orchestration patterns (sequential, parallel, conditional, loop)
5. **Built-in Library**: 40+ tools across 8 categories, 6 agent templates, 6 workflow types
6. **Hook System**: 20+ hook points for logging, metrics, security, and custom behavior
7. **Event Bus**: Async event emission/subscription for real-time coordination

### Technology Stack

- **LLM Providers**: `rig` (multi-provider) + `candle` (local models)
- **Scripting**: `mlua` (Lua 5.4), `boa`/`quickjs` (JavaScript), `pyo3` (Python - future)
- **Storage**: `sled` (development) / `rocksdb` (production) behind trait abstractions
- **Events**: `tokio-stream` + `crossbeam` hybrid for async/sync patterns
- **Testing**: `mockall` + `proptest` + `criterion` comprehensive stack
- **Observability**: `tracing` + `metrics-rs` + optional `opentelemetry`

### Async Patterns

- **Lua**: Coroutine-based cooperative scheduling with Promise-like abstractions
- **JavaScript**: Native Promises with controlled concurrency and backpressure
- **Unified Interface**: Consistent async patterns across all scripting languages
- **Cooperative Yielding**: Non-blocking execution for long-running operations

## Phase 0 Implementation Workflow

### CRITICAL Phase 0 Requirements
1. **Zero Warnings Policy**: All code must compile without warnings
2. **Documentation First**: Every trait/type documented before implementation
3. **TDD Foundation**: Core traits tested before implementation begins
4. **CI/CD Ready**: Pipeline validates every commit from day one
5. **Track Progress**: Update TODO.md with task completion timestamps

### Development Process
1. **Workspace First**: Set up complete 12-crate workspace structure
2. **Core Traits**: Implement BaseAgent/Agent/Tool/Workflow trait hierarchy
3. **Error Handling**: Comprehensive error system with categorization
4. **Testing Infrastructure**: mockall + proptest + criterion setup
5. **CI/CD Pipeline**: GitHub Actions with quality gates
6. **Documentation**: >95% coverage requirement

### Quality Gates (MANDATORY - All Implemented in CI)
- `cargo check --workspace` - Zero errors/warnings ✅
- `cargo test --workspace` - >90% test coverage (enforced in CI) ✅
- `cargo clippy -- -D warnings` - Zero clippy warnings ✅
- `cargo fmt --check` - Consistent formatting ✅
- `cargo doc --workspace` - Documentation builds successfully (>95% coverage) ✅
- `./scripts/quality-check.sh` - Local validation matching CI requirements ✅
- `cargo deadlinks --dir target/doc` - Internal documentation links valid ✅
- `markdown-link-check` - External documentation links valid ✅

## Critical Implementation Principles

### State-First Architecture
- Agents communicate through shared state, not direct message passing
- State preservation across agent handoffs
- Debugging and observability through state inspection

### Tool-Wrapped Agents Pattern
```rust
// Any agent can be wrapped as a tool
let research_tool = AgentAsTool::new(research_agent);
workflow.add_tool(research_tool);
```

### Hook Integration
```rust
// Hooks at every execution point
self.hooks.execute(HookPoint::BeforeExecution, &input).await?;
// ... execution ...
self.hooks.execute(HookPoint::AfterExecution, &result).await?;
```

### Production-First Design
- Circuit breakers on every agent
- Resource limits enforced
- Comprehensive error handling
- Security sandboxing built-in

## Key Development Reminders

- **Complete Tasks Fully**: No lazy implementations or deferrals
- **Do not jump ahead** Stick to task hierarchy in TODO.md, re-read it after tasks again to make sure you didn't miss steps.
- **Maintain Bridge Philosophy**: Use existing crates, don't reinvent
- **State Over Messages**: Agent handoff via shared state
- **Tool Composition**: Agents as composable tools
- **No Backward Compatibility**: Breaking changes allowed until v1.0.0
- **Update Documentation**: Keep TODO.md current with timestamps as tasks progress

## Primary Documentation

**🎯 Complete Architecture**: `/docs/technical/rs-llmspell-final-architecture.md`
- Standalone 15,034+ line comprehensive guide
- All architectural decisions and implementation details
- Production-ready specifications with examples
- No external references required

**Phase 0 Implementation Documents**:
- `/docs/in-progress/phase-00-design-doc.md` - Detailed Phase 0 specifications
- `/docs/in-progress/PHASE00-TODO.md` - 37 specific implementation tasks
- `/docs/in-progress/implementation-phases.md` - Complete 16-phase roadmap
- `/TODO.md` - Current Phase 0 task tracking

**Research Archive**: `/docs/technical/`
- 30+ research documents from architectural phases
- Deep dives into specific technical decisions
- Historical context for architectural choices

## Phase 1 Specific Tasks

### Task 1.0: Create llmspell-utils Crate
1. Create new crate directory with Cargo.toml
2. Add to workspace members in root Cargo.toml
3. Implement utility modules (async_utils, file_utils, etc.)
4. **Acceptance**: All utilities tested with >90% coverage

### Task 1.1: Enhanced Core Types
1. Add streaming types (AgentStream, AgentChunk, ChunkContent)
2. Add multimodal types (MediaContent, ImageFormat, etc.)
3. Update AgentInput/AgentOutput with media support
4. **Acceptance**: All types serialize/deserialize correctly

### Task 1.2: Script Runtime Foundation
1. Create ScriptRuntime struct with Lua integration
2. Inject Agent API into Lua environment
3. Implement coroutine-based streaming support
4. **Acceptance**: Can execute Lua scripts with agents

### Task 1.3: Provider Integration
1. Create provider abstraction layer
2. Implement rig provider wrapper
3. Add capability detection for streaming/multimodal
4. **Acceptance**: LLM calls work from scripts

### Task 1.4: CLI Implementation
1. Create basic CLI structure with clap
2. Add streaming output support with progress
3. Implement configuration loading
4. **Acceptance**: CLI executes scripts with streaming output

## Testing Strategy

- **Unit Tests**: Test individual components in isolation
- **Integration Tests**: Test component interactions
- **Script Tests**: Test Lua/JavaScript API functionality
- **Property Tests**: Use proptest for invariant testing
- **Benchmarks**: Track performance with criterion

Run tests with increasing scope:
```bash
cargo test --lib                    # Unit tests only
cargo test --test '*'               # Integration tests
cargo test --all-features           # All features enabled
cargo test --workspace              # Entire workspace
cargo clippy -- -D warnings         # Zero warnings policy 
```