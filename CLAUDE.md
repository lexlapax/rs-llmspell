# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

rs-llmspell: **Scriptable LLM interactions** via Lua, JavaScript - Cast scripting spells to animate LLM golems

## Current Status

🚀 **Phase 2 - Built-in Tools Library**: IN PROGRESS (Started 2025-06-27)
- **Completed**: All architectural research phases (1-13) ✅
- **Completed**: Phase 0 implementation - All 37 tasks ✅
- **Completed**: Phase 1 implementation - All 21 tasks ✅
- **Current**: Phase 2 implementation - Built-in tools library 🔄
- **Next**: 12+ core tools with provider enhancements

### Phase 0 Achievements
- ✅ **12-crate workspace** with zero compiler warnings
- ✅ **165 comprehensive tests** (unit, integration, property, doc tests)
- ✅ **Complete CI/CD pipeline** with 7 jobs and quality gates
- ✅ **Professional documentation** (>95% coverage, GitHub Pages ready)
- ✅ **Architecture enhanced** with streaming and multimodal support

### Phase 1 Achievements
- ✅ **13th crate**: `llmspell-utils` for shared utilities
- ✅ **Streaming support**: BaseAgent and Tool traits extended
- ✅ **Multimodal types**: MediaContent (Image, Audio, Video, Binary)
- ✅ **Lua runtime**: Basic script execution with agent APIs through ScriptEngineBridge
- ✅ **CLI enhancement**: Streaming output with progress indicators
- ✅ **Provider integration**: Rig provider wrapper with capability detection
- ✅ **Performance**: All targets exceeded (startup <100μs, streaming <50μs)

### Phase 2 Implementation Focus
- 🔄 **ModelSpecifier**: Parse `provider/model` syntax (e.g., "openai/gpt-4")
- 🔄 **Base URL overrides**: Agent-level configuration for custom endpoints
- 🔄 **12+ Built-in Tools**: Web search, JSON processing, HTTP requests, etc.
- 🔄 **Tool Registry**: Discovery by capability with validation
- 🔄 **Security Sandbox**: Prevent unauthorized file/network access

## Phase 2 Development Commands

**Current Focus**: Built-in Tools Library (10 working days)

```bash
# Phase 2 Development Commands
cargo check --workspace          # Verify workspace compilation
cargo build --workspace          # Build all crates including tools
cargo test --workspace           # Run all tests including tools
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
npm install -g markdown-link-check # Install markdown link validator (npm package)
cargo deadlinks --dir target/doc # Check internal documentation links
markdown-link-check README.md    # Validate README links

# Phase 2 Specific Tasks
cargo test -p llmspell-tools     # Test tools crate
cargo test -p llmspell-providers # Test provider enhancements
cargo bench -p llmspell-tools    # Benchmark tool performance

# CI/CD Pipeline (IMPLEMENTED)
.github/workflows/ci.yml         # Automated quality checks
.github/QUALITY_GATES.md         # Branch protection and quality standards
.github/CI_VALIDATION_REPORT.md  # CI/CD pipeline validation results
.github/PHASE0_COMPLETION_REPORT.md # Phase 0 completion validation
.github/PHASE1_COMPLETION_REPORT.md # Phase 1 completion validation
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
- **No Shortcuts or simplification**: Web Research or ask user if complex coding issue
- **Do not jump ahead** Stick to task hierarchy in TODO.md, re-read it after tasks again to make sure you didn't miss steps.
- **Maintain Bridge Philosophy**: Use existing crates, don't reinvent
- **State Over Messages**: Agent handoff via shared state
- **Tool Composition**: Agents as composable tools
- **No Backward Compatibility**: Breaking changes encouraged until v1.0.0
- **Update Documentation**: Keep TODO.md current with timestamps as tasks progress

## Primary Documentation

**🎯 Complete Architecture**: `/docs/technical/rs-llmspell-final-architecture.md`
- Standalone 15,034+ line comprehensive guide
- All architectural decisions and implementation details
- Production-ready specifications with examples
- No external references required

**Phase Implementation Documents**:
- `/docs/in-progress/phase-02-design-doc.md` - Detailed Phase 2 specifications
- `/docs/in-progress/PHASE02-TODO.md` - 22 specific implementation tasks
- `/docs/in-progress/implementation-phases.md` - Complete 16-phase roadmap (updated)
- `/TODO.md` - Current Phase 2 task tracking

**Research Archive**: `/docs/technical/`
- 30+ research documents from architectural phases
- Deep dives into specific technical decisions
- Historical context for architectural choices

## Phase 2 Specific Tasks

### Task 2.1: Provider Enhancement (ModelSpecifier)
1. Implement ModelSpecifier to parse "provider/model" syntax
2. Update ProviderManager with create_agent_from_spec()
3. Update script APIs to support new syntax
4. **Acceptance**: Scripts can use "openai/gpt-4" syntax

### Task 2.2: Core Tool Infrastructure
1. Enhance Tool trait with streaming and security methods
2. Implement Tool Registry with discovery and validation
3. Create Security Sandbox for safe execution
4. **Acceptance**: Tools can be registered and discovered

### Task 2.3-2.5: Built-in Tools Implementation
1. Search Tools: WebSearch, SemanticSearch, CodeSearch
2. Data Tools: JsonProcessor, CsvAnalyzer, XmlTransformer
3. API Tools: HttpRequest, GraphQLQuery
4. File Tools: FileOperations, ArchiveHandler
5. Utility Tools: TemplateEngine, DataValidation
6. **Acceptance**: All 12+ tools functional with tests

### Task 2.6: Integration and Documentation
1. Script integration tests for all tools
2. Security validation and penetration testing
3. Performance optimization (<10ms init)
4. Comprehensive documentation and examples
5. **Acceptance**: Ready for Phase 3 handoff

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