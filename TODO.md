# Phase 1: Core Execution Runtime - TODO List

**Version**: 1.0  
**Date**: June 2025  
**Status**: IN PROGRESS 🚀  
**Started**: 2025-06-26 (Evening)  
**Phase**: 1 (Core Execution Runtime)  
**Timeline**: Weeks 3-4 (10 working days)  
**Priority**: CRITICAL (MVP Core)
**Arch-Document**: docs/technical/rs-llmspell-final-architecture.md
**All-Phases-Document**: docs/in-progress/implementation-phases.md
**Design-Document**: docs/in-progress/phase-01-design-doc.md

> **📢 UPDATE**: Phase 0 complete! Phase 1 implementation started. Architecture enhanced with streaming and multimodal support.

> **📋 Actionable Task List**: This document breaks down Phase 1 implementation into specific, measurable tasks with clear acceptance criteria.

---

## Overview

**Goal**: Implement core execution engine with basic Lua scripting, streaming support, and multimodal content types.

**Success Criteria Summary:**
- [ ] `llmspell-utils` crate provides common utilities to all crates
- [ ] Can execute simple Lua scripts with Agent/Tool APIs
- [ ] LLM providers can be called from scripts
- [ ] Basic tool execution works
- [ ] Streaming methods defined and functional (stub implementation acceptable)
- [ ] Multimodal types compile and are accessible from scripts
- [ ] Error propagation from scripts to CLI
- [ ] Memory usage stays under 50MB for simple scripts

---

## Phase 1.0: Utilities Crate (Days 1-2) ✅ COMPLETE

### Task 1.0.1: Create llmspell-utils Crate Structure
**Priority**: CRITICAL  
**Estimated Time**: 2 hours  
**Assignee**: Core Team Lead
**Dependencies**: Workspace setup from Phase 0

**Description**: Create the new llmspell-utils crate with proper structure and dependencies.

**Acceptance Criteria:**
- [x] `llmspell-utils` directory created with Cargo.toml
- [x] Added to workspace members in root Cargo.toml
- [x] Basic module structure created (lib.rs with submodules)
- [x] Dependencies configured (tokio, tracing, etc.)
- [x] `cargo check -p llmspell-utils` passes

**Implementation Steps:**
1. Create `llmspell-utils/` directory
2. Create `llmspell-utils/Cargo.toml` with workspace dependencies
3. Create `llmspell-utils/src/lib.rs` with module declarations
4. Add to workspace members array in root Cargo.toml
5. Create empty module files for each utility module

**Definition of Done:**
- [x] Crate compiles without warnings
- [x] Module structure matches design document
- [x] All dependencies resolve correctly
- [x] Basic documentation in lib.rs

**Completed**: 2025-06-26

### Task 1.0.2: Implement Async Utilities Module
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Core Team
**Dependencies**: Task 1.0.1

**Description**: Implement retry logic, timeout helpers, and other async utilities.

**Acceptance Criteria:**
- [x] `retry_async` function with configurable attempts and backoff
- [x] `timeout_with_default` helper function
- [x] `concurrent_map` for parallel async operations
- [x] Comprehensive unit tests for each utility (12 tests)
- [x] Documentation with usage examples

**Implementation Steps:**
1. Create `src/async_utils.rs`
2. Implement retry logic with exponential backoff
3. Add timeout wrapper functions
4. Create concurrent execution helpers
5. Write unit tests for each function
6. Add rustdoc with examples

**Definition of Done:**
- [x] All functions have tests
- [x] Test coverage >90%
- [x] No clippy warnings
- [x] Documentation complete

**Completed**: 2025-06-26

### Task 1.0.3: Implement File Utilities Module
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Core Team
**Dependencies**: Task 1.0.1

**Description**: Create safe file operations and path manipulation utilities.

**Acceptance Criteria:**
- [x] Path normalization functions
- [x] Safe file read/write with proper error handling
- [x] Directory creation with parent handling
- [x] Atomic file operations support
- [x] Cross-platform path handling

**Implementation Steps:**
1. Create `src/file_utils.rs`
2. Implement path normalization for cross-platform support
3. Add safe file operations with error context
4. Create atomic write functionality
5. Add directory utilities
6. Write comprehensive tests

**Definition of Done:**
- [x] Works on Windows, macOS, and Linux
- [x] Handles edge cases (permissions, missing dirs)
- [x] Tests cover error scenarios (18 unit tests + 3 property tests)
- [x] Performance benchmarks included

**Completed**: 2025-06-26

### Task 1.0.4: Implement Remaining Utility Modules
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: Core Team
**Dependencies**: Task 1.0.1

**Description**: Implement string_utils, system_info, error_builders, id_generator, and serialization modules.

**Acceptance Criteria:**
- [x] String manipulation and formatting utilities
- [x] System/OS detection helpers
- [x] Error builder patterns for common errors
- [x] UUID generation with prefixes
- [x] JSON/TOML serialization helpers

**Implementation Steps:**
1. Create module files for each utility type
2. Implement core functionality for each module
3. Add tests for each module
4. Ensure consistent API design across modules
5. Document all public functions

**Definition of Done:**
- [x] All modules compile and pass tests
- [x] Consistent naming conventions
- [x] >80% test coverage overall
- [x] Examples in documentation

**Completed**: 2025-06-26

---

## Phase 1.1: Enhanced Core Types (Days 2-3)

### Task 1.1.1: Add Streaming Types to Core
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Core Team Lead
**Dependencies**: llmspell-utils completion

**Description**: Add streaming-related types to llmspell-core.

**Acceptance Criteria:**
- [x] `AgentStream` type alias defined
- [x] `AgentChunk` struct with all fields
- [x] `ChunkContent` enum with variants
- [x] `ChunkMetadata` struct
- [x] Serialization/deserialization support

**Implementation Steps:**
1. Create `src/types/streaming.rs` in llmspell-core
2. Define all streaming-related types
3. Implement Display and Debug traits
4. Add serde derives
5. Write unit tests for serialization
6. Export from types module

**Definition of Done:**
- [x] All types serialize/deserialize correctly
- [x] Comprehensive Debug implementations
- [x] Tests for all type variants
- [x] Documentation complete

**Completed**: 2025-06-26

### Task 1.1.2: Add Multimodal Content Types
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Core Team
**Dependencies**: llmspell-utils completion

**Description**: Implement MediaContent enum and related types.

**Acceptance Criteria:**
- [x] `MediaContent` enum with all variants
- [x] Format enums (ImageFormat, AudioFormat, VideoFormat)
- [x] Metadata structs for each media type
- [x] Size validation helpers
- [x] Type conversion utilities

**Implementation Steps:**
1. Create `src/types/media.rs` in llmspell-core
2. Define MediaContent enum and variants
3. Add format enums for each media type
4. Create metadata structures
5. Implement validation methods
6. Add conversion helpers
7. Write comprehensive tests

**Definition of Done:**
- [x] All media types properly defined
- [x] Validation logic works correctly (100MB/500MB/5GB limits)
- [x] Binary data handling tested
- [x] Memory-efficient implementations

**Completed**: 2025-06-26

### Task 1.1.3: Enhance AgentInput/AgentOutput Types ✅ COMPLETE
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Core Team
**Dependencies**: Task 1.1.2

**Description**: Update AgentInput and AgentOutput to support multimodal content.

**Acceptance Criteria:**
- [x] AgentInput includes media Vec
- [x] AgentOutput includes media Vec
- [x] Builder patterns for construction
- [x] Backward compatibility maintained
- [x] Helper methods for common operations

**Implementation Steps:**
1. Update AgentInput struct in core types
2. Update AgentOutput struct
3. Add builder implementations
4. Create convenience constructors
5. Update existing tests
6. Add new multimodal tests

**Definition of Done:**
- [x] Existing code still compiles
- [x] New fields properly integrated
- [x] Builder patterns documented
- [x] Migration guide written

**Completed**: 2025-06-26

### Task 1.1.4: Update BaseAgent Trait with Streaming ✅ COMPLETE
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Core Team Lead
**Dependencies**: Task 1.1.1

**Description**: Add streaming methods to BaseAgent trait.

**Acceptance Criteria:**
- [x] `stream_execute` method added with default impl
- [x] `supports_streaming` method added
- [x] `supports_multimodal` method added
- [x] `supported_media_types` method added
- [x] Trait still object-safe

**Implementation Steps:**
1. Update BaseAgent trait in traits module
2. Add streaming method with NotImplemented default
3. Add capability detection methods
4. Update documentation
5. Verify trait object safety
6. Update mock implementations

**Definition of Done:**
- [x] Trait compiles without breaking changes
- [x] Default implementations work
- [x] Mock implementations updated
- [x] Documentation explains streaming

**Completed**: 2025-06-26

---

## Phase 1.2: Script Runtime Foundation (Days 3-5)

### Task 1.2.1: Create Basic ScriptRuntime Structure
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Bridge Team Lead
**Dependencies**: Enhanced core types

**Description**: Implement the core ScriptRuntime struct with Lua integration.

**Acceptance Criteria:**
- [ ] ScriptRuntime struct with all fields
- [ ] Lua initialization with restricted stdlib
- [ ] Component registry integration
- [ ] Provider manager integration
- [ ] Basic execute_script method

**Implementation Steps:**
1. Create runtime module in llmspell-bridge
2. Define ScriptRuntime struct
3. Implement new() with Lua setup
4. Add component registry field
5. Add provider manager field
6. Implement basic script execution

**Definition of Done:**
- [ ] Runtime initializes without errors
- [ ] Can execute simple Lua scripts
- [ ] Proper error handling
- [ ] Thread-safe implementation

### Task 1.2.2: Inject Agent API into Lua
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Bridge Team
**Dependencies**: Task 1.2.1

**Description**: Create Lua bindings for Agent creation and execution.

**Acceptance Criteria:**
- [ ] Agent.create() function in Lua
- [ ] agent:execute() method works
- [ ] agent:get_config() method
- [ ] Error handling from Lua to Rust
- [ ] Type conversions work correctly

**Implementation Steps:**
1. Create lua/agent_api.rs module
2. Implement Agent userdata type
3. Add create function to globals
4. Implement execute method
5. Add configuration access
6. Write Lua integration tests

**Definition of Done:**
- [ ] Can create agents from Lua
- [ ] Execute method returns results
- [ ] Errors propagate correctly
- [ ] Tests pass with mock agents

### Task 1.2.3: Implement Lua Streaming Support
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Bridge Team
**Dependencies**: Task 1.2.2

**Description**: Add coroutine-based streaming to Lua API.

**Acceptance Criteria:**
- [ ] stream_execute returns Lua coroutine
- [ ] Coroutines yield chunks properly
- [ ] Error handling in coroutines
- [ ] Memory management correct
- [ ] Examples demonstrate usage

**Implementation Steps:**
1. Create lua/streaming.rs module
2. Implement coroutine creation helpers
3. Add stream iteration support
4. Handle chunk conversion to Lua
5. Test with mock streaming agent
6. Create usage examples

**Definition of Done:**
- [ ] Streaming works end-to-end
- [ ] No memory leaks
- [ ] Performance acceptable
- [ ] Documentation complete

### Task 1.2.4: Add Tool and Basic Workflow APIs
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Bridge Team
**Dependencies**: Task 1.2.2

**Description**: Create Lua bindings for Tool usage and basic workflows.

**Acceptance Criteria:**
- [ ] Tool.get() function in Lua
- [ ] tool:execute() method works
- [ ] Basic workflow creation
- [ ] State passing between tools
- [ ] Error propagation works

**Implementation Steps:**
1. Create lua/tool_api.rs module
2. Implement Tool userdata type
3. Add tool registry access
4. Create workflow stubs
5. Test tool execution
6. Document API usage

**Definition of Done:**
- [ ] Tools callable from Lua
- [ ] Results properly converted
- [ ] Workflow stubs compile
- [ ] Integration tests pass

---

## Phase 1.3: Provider Integration (Days 5-6)

### Task 1.3.1: Create Provider Abstraction Layer
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Provider Team Lead
**Dependencies**: Core types complete

**Description**: Design and implement provider capability detection system.

**Acceptance Criteria:**
- [ ] ProviderCapabilities trait defined
- [ ] ProviderManager struct implemented
- [ ] Provider registration system
- [ ] Capability queries work
- [ ] Configuration loading

**Implementation Steps:**
1. Create abstraction.rs in llmspell-providers
2. Define ProviderCapabilities trait
3. Implement ProviderManager
4. Add configuration structures
5. Create provider registry
6. Write unit tests

**Definition of Done:**
- [ ] Clean abstraction design
- [ ] Extensible for new providers
- [ ] Configuration documented
- [ ] Tests cover all paths

### Task 1.3.2: Implement Rig Provider Wrapper
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Provider Team
**Dependencies**: Task 1.3.1

**Description**: Create wrapper for rig crate with streaming support.

**Acceptance Criteria:**
- [ ] RigProvider implements ProviderInstance
- [ ] Text completion works
- [ ] Streaming stub implemented
- [ ] Error handling complete
- [ ] Configuration from environment

**Implementation Steps:**
1. Create providers/rig.rs module
2. Implement ProviderInstance trait
3. Wrap rig completion calls
4. Add streaming stub (can return NotImplemented)
5. Handle API key configuration
6. Test with mock responses

**Definition of Done:**
- [ ] Basic completion works
- [ ] Errors handled gracefully
- [ ] Configuration flexible
- [ ] Ready for streaming later

### Task 1.3.3: Integrate Providers with ScriptRuntime
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Bridge Team
**Dependencies**: Task 1.3.2, Task 1.2.1

**Description**: Connect provider system to script runtime.

**Acceptance Criteria:**
- [ ] Providers accessible from scripts
- [ ] Default provider configuration
- [ ] Provider switching support
- [ ] Error messages helpful
- [ ] Performance acceptable

**Implementation Steps:**
1. Add provider manager to runtime
2. Configure default provider
3. Add provider access from Lua
4. Test provider calls
5. Add error context
6. Benchmark performance

**Definition of Done:**
- [ ] LLM calls work from scripts
- [ ] Configuration documented
- [ ] Errors are clear
- [ ] Performance <100ms overhead

---

## Phase 1.4: CLI Implementation (Days 7-8)

### Task 1.4.1: Create Basic CLI Structure
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: CLI Team Lead
**Dependencies**: Script runtime complete

**Description**: Implement CLI entry point with run command.

**Acceptance Criteria:**
- [ ] CLI parsing with clap
- [ ] Run subcommand works
- [ ] Help text comprehensive
- [ ] Error messages friendly
- [ ] Version information correct

**Implementation Steps:**
1. Set up clap in llmspell-cli
2. Define CLI structure
3. Implement run command
4. Add help documentation
5. Handle errors gracefully
6. Add version from cargo

**Definition of Done:**
- [ ] CLI compiles and runs
- [ ] Help text is clear
- [ ] Errors show context
- [ ] Version matches cargo

### Task 1.4.2: Add Streaming Output Support
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: CLI Team
**Dependencies**: Task 1.4.1

**Description**: Implement streaming output display with progress indicators.

**Acceptance Criteria:**
- [ ] --stream flag enables streaming
- [ ] Progress spinner/bar shows
- [ ] Output displayed incrementally
- [ ] Clean output formatting
- [ ] Ctrl+C handling works

**Implementation Steps:**
1. Add streaming flag to CLI
2. Integrate indicatif for progress
3. Implement streaming display loop
4. Handle output formatting
5. Add signal handling
6. Test with mock streams

**Definition of Done:**
- [ ] Streaming looks professional
- [ ] No display artifacts
- [ ] Interruption handled cleanly
- [ ] Performance smooth

### Task 1.4.3: Add Configuration Loading
**Priority**: MEDIUM  
**Estimated Time**: 3 hours  
**Assignee**: CLI Team
**Dependencies**: Task 1.4.1

**Description**: Load configuration from files and environment.

**Acceptance Criteria:**
- [ ] Config file discovery works
- [ ] Environment overrides supported
- [ ] Default configuration sensible
- [ ] Config errors helpful
- [ ] --config flag works

**Implementation Steps:**
1. Add config module to CLI
2. Implement file discovery
3. Add environment loading
4. Create default config
5. Merge configurations
6. Validate configuration

**Definition of Done:**
- [ ] Config loading tested
- [ ] Precedence documented
- [ ] Errors guide fixes
- [ ] Examples provided

---

## Phase 1.5: Testing and Integration (Days 8-9)

### Task 1.5.1: Unit Test Suite
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Full Team
**Dependencies**: All implementation tasks

**Description**: Ensure comprehensive unit test coverage.

**Acceptance Criteria:**
- [ ] Utils crate >90% coverage
- [ ] Core types fully tested
- [ ] Bridge components tested
- [ ] Provider abstractions tested
- [ ] CLI parsing tested

**Implementation Steps:**
1. Review test coverage reports
2. Add missing unit tests
3. Test error conditions
4. Test edge cases
5. Verify test quality
6. Document test patterns

**Definition of Done:**
- [ ] Coverage targets met
- [ ] All tests pass
- [ ] Tests are maintainable
- [ ] CI runs all tests

### Task 1.5.2: Integration Test Suite
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: Full Team
**Dependencies**: Task 1.5.1

**Description**: Create end-to-end integration tests.

**Acceptance Criteria:**
- [ ] Script execution tests
- [ ] Streaming tests work
- [ ] Provider integration tested
- [ ] CLI commands tested
- [ ] Error scenarios covered

**Implementation Steps:**
1. Create tests/integration directory
2. Write script execution tests
3. Add streaming tests
4. Test provider calls
5. Test CLI commands
6. Add performance tests

**Definition of Done:**
- [ ] Integration tests comprehensive
- [ ] Tests run in CI
- [ ] Performance benchmarks included
- [ ] Flaky tests eliminated

### Task 1.5.3: Memory and Performance Validation
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Performance Team
**Dependencies**: Task 1.5.2

**Description**: Validate memory usage and performance targets.

**Acceptance Criteria:**
- [ ] Memory usage <50MB verified
- [ ] No memory leaks detected
- [ ] Script startup <100ms
- [ ] Streaming latency <50ms
- [ ] Benchmarks automated

**Implementation Steps:**
1. Create memory benchmarks
2. Add performance benchmarks
3. Test for memory leaks
4. Profile hot paths
5. Document findings
6. Add to CI pipeline

**Definition of Done:**
- [ ] All targets met
- [ ] Benchmarks in CI
- [ ] No memory leaks
- [ ] Performance acceptable

---

## Phase 1.6: Documentation and Handoff (Days 9-10)

### Task 1.6.1: API Documentation
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Full Team
**Dependencies**: All code complete

**Description**: Ensure all public APIs are documented.

**Acceptance Criteria:**
- [ ] All public items have rustdoc
- [ ] Examples for main features
- [ ] Streaming usage documented
- [ ] Multimodal usage shown
- [ ] No documentation warnings

**Implementation Steps:**
1. Run cargo doc and review
2. Add missing documentation
3. Write usage examples
4. Document error types
5. Add module overviews
6. Generate and review

**Definition of Done:**
- [ ] Documentation complete
- [ ] Examples compile
- [ ] No warnings
- [ ] Looks professional

### Task 1.6.2: User Guide and Examples
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Documentation Team
**Dependencies**: Task 1.6.1

**Description**: Create user-facing guides and examples.

**Acceptance Criteria:**
- [ ] Getting started guide
- [ ] Streaming example script
- [ ] Multimodal example (stub)
- [ ] Error handling guide
- [ ] Performance tips

**Implementation Steps:**
1. Write getting started guide
2. Create example scripts
3. Document common patterns
4. Add troubleshooting section
5. Create performance guide
6. Test all examples

**Definition of Done:**
- [ ] Guides are clear
- [ ] Examples work
- [ ] Common issues covered
- [ ] Easy to follow

### Task 1.6.3: Phase 2 Handoff Package
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Team Lead
**Dependencies**: All tasks complete

**Description**: Prepare comprehensive handoff materials.

**Acceptance Criteria:**
- [ ] Summary of deliverables
- [ ] Known issues documented
- [ ] API stability notes
- [ ] Performance baselines
- [ ] Next phase preparation

**Implementation Steps:**
1. Document completed features
2. List known limitations
3. Provide performance data
4. Create transition guide
5. Update phase documents
6. Schedule handoff meeting

**Definition of Done:**
- [ ] Package complete
- [ ] Phase 2 team ready
- [ ] No blocking issues
- [ ] Clean handoff

---

## Summary Dashboard

### Critical Path
1. **Days 1-2**: Utils crate (foundation for everything)
2. **Days 2-3**: Core types (streaming, multimodal)
3. **Days 3-5**: Script runtime (Lua integration)
4. **Days 5-6**: Provider integration
5. **Days 7-8**: CLI implementation
6. **Days 8-10**: Testing, documentation, handoff

### Resource Allocation
- **Core Team**: Utils, core types, trait updates
- **Bridge Team**: Script runtime, Lua APIs
- **Provider Team**: Provider abstraction and integration
- **CLI Team**: Command-line interface
- **All**: Testing and documentation

### Risk Areas
1. **Lua Streaming Complexity**: Have fallback plan
2. **Memory Constraints**: Monitor early and often
3. **Provider Abstraction**: Keep simple initially
4. **Schedule**: 10 days is aggressive, prioritize MVP

### Success Metrics
- ✅ All crates compile without warnings
- ✅ Can execute Lua scripts with agents
- ✅ Streaming methods defined (stub OK)
- ✅ Multimodal types accessible
- ✅ Memory usage <50MB
- ✅ >80% test coverage
- ✅ Documentation complete