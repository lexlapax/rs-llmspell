# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚀 **Phase 0 - Foundation Infrastructure**: READY TO START IMPLEMENTATION
- **Completed**: All architectural research phases (1-13) ✅
- **Delivered**: Complete architecture and implementation roadmap ✅
- **Current**: Phase 0 implementation - Foundation infrastructure 
- **Next**: Implement core traits, workspace setup, and CI/CD pipeline

### Architecture Readiness
- ✅ **15,034+ line complete architecture document**
- ✅ **16-phase implementation roadmap** with clear success criteria
- ✅ **Technology stack selection** and integration strategy
- ✅ **Phase 0 detailed design** with 37 specific implementation tasks
- ✅ **Complete built-in library specification** (40+ tools, agent templates, workflows)

## Phase 0 Development Commands

**Current Focus**: Foundation Infrastructure Implementation

```bash
# Phase 0 Workspace Setup
cargo check --workspace          # Verify workspace compilation
cargo build --workspace          # Build all foundation crates
cargo test --workspace           # Run foundation tests
cargo doc --workspace --no-deps  # Generate documentation

# Quality Assurance (MANDATORY before commits)
cargo clippy -- -D warnings      # Zero warnings policy
cargo fmt --check               # Formatting validation
cargo fmt                       # Apply formatting
cargo test --workspace          # Run all tests

# Phase 0 Specific Tasks
cargo metadata                   # Verify workspace structure
cargo tree                      # Check dependency graph

# CI/CD Pipeline (when implemented)
.github/workflows/ci.yml         # Automated quality checks

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

### Quality Gates (MANDATORY)
- `cargo check --workspace` - Zero errors/warnings
- `cargo test --workspace` - >90% test coverage
- `cargo clippy -- -D warnings` - Zero clippy warnings
- `cargo fmt --check` - Consistent formatting
- `cargo doc --workspace` - Documentation builds successfully

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

## Phase 0 Specific Tasks

### Task 0.1: Workspace Setup
1. Create root `Cargo.toml` with 12-crate workspace
2. Generate individual crate structures and manifests
3. Verify workspace compilation: `cargo check --workspace`
4. **Acceptance**: Zero compilation errors, clean dependency graph

### Task 0.2: Core Traits Definition
1. Implement `ComponentId`, `Version`, `ComponentMetadata` in `llmspell-core`
2. Define `BaseAgent` trait with full method signatures
3. Implement `Agent`, `Tool`, `Workflow` traits extending `BaseAgent`
4. **Acceptance**: All traits compile with comprehensive documentation

### Task 0.3: Error Handling System
1. Create `LLMSpellError` enum with all error variants
2. Implement error categorization and retryability detection
3. Add error convenience macros for consistent usage
4. **Acceptance**: Comprehensive error handling with 100% test coverage

### Task 0.4: Testing Infrastructure
1. Configure `mockall` for trait mocking
2. Set up `proptest` for property-based testing
3. Configure `criterion` for performance benchmarks
4. **Acceptance**: All testing frameworks operational with examples

### Task 0.5: CI/CD Pipeline
1. Create `.github/workflows/ci.yml`
2. Configure quality gates (tests, clippy, formatting, docs)
3. Set up GitHub Pages documentation deployment
4. **Acceptance**: Pipeline passes on clean codebase, <10min runtime

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