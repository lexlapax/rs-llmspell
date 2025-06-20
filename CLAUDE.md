# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🔄 **Phase 12 - Task 12.3 IN PROGRESS**: Manual Review of Final Documentation
- **Completed**: All research and documentation phases (1-12.2)
- **Delivered**: `/docs/rs-llmspell-complete-architecture.md` - 15,034+ line standalone guide
- **Current**: Task 12.3 - Reviewing architecture through Q&A (completed 10 questions)
- **Next**: Complete Task 12.3 review, then Phase 13 - Implementation roadmap

## Build and Development Commands

```bash
# Build the project
cargo build
cargo build --release

# Run tests
cargo test
cargo test --all-features
cargo test --workspace

# Run a specific test
cargo test test_name

# Linting and formatting
cargo clippy -- -D warnings
cargo fmt --check
cargo fmt

# Documentation
cargo doc --open
cargo doc --no-deps

# Benchmarks (when implemented)
cargo bench
cargo bench --bench benchmark_name

# Clean build artifacts
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

## Implementation Workflow

1. **Bridge-first Development**: Always wrap existing crates rather than reimplementing
2. **TDD Mandatory**: Write tests before implementation - no exceptions
3. **Async-aware Design**: Implement cooperative scheduling for script engines
4. **Track Progress**: Update TODO.md with timestamps for completed tasks
5. **Quality Checks**: Run `cargo test && cargo clippy && cargo fmt` before commits

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
- **Update Documentation**: Keep TODO.md current with timestamps

## Primary Documentation

**🎯 Complete Architecture**: `/docs/rs-llmspell-complete-architecture.md`
- Standalone 15,034+ line comprehensive guide
- All architectural decisions and implementation details
- Production-ready specifications with examples
- No external references required

**Technical Documents**: `/docs/technical/`
- 30+ research documents organized by phase
- Deep dives into specific architectural aspects
- Implementation patterns and best practices

**Key Documents**:
- `/docs/technical/architecture.md` - Original architecture specification
- `/docs/technical/final_architecture_synthesis.md` - Complete integration
- `/docs/technical/component_ecosystem_design.md` - Built-in components
- `/docs/technical/build_vs_buy_decision_matrix.md` - Technology choices

## Common Development Tasks

### Adding a New Tool
1. Implement the `Tool` trait in `llmspell-tools/src/category/`
2. Add schema validation and parameter handling
3. Write comprehensive tests including error cases
4. Update tool registry and documentation

### Creating an Agent Template
1. Extend `BaseAgent` in `llmspell-agents/src/templates/`
2. Define specialized system prompts and behaviors
3. Implement state management and tool integration
4. Add hooks for monitoring and debugging

### Implementing a Workflow Pattern
1. Create workflow type in `llmspell-workflows/src/patterns/`
2. Define execution strategy (sequential/parallel/conditional)
3. Implement state passing between steps
4. Add error handling and recovery logic

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
```