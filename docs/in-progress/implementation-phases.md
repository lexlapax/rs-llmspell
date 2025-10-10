# Rs-LLMSpell Implementation Phases

**Version**: 1.0  
**Date**: June 2025  
**Status**: Implementation Roadmap  

> **📋 Complete Implementation Guide**: This document defines all implementation phases for rs-llmspell, from MVP foundation through advanced production features. Currently includes 18 core phases with additional future enhancements.

---

## Overview

Rs-LLMSpell follows a carefully structured 23+ phase implementation approach that prioritizes core functionality while building toward production readiness. Each phase has specific goals, components, and measurable success criteria. The roadmap includes Phase 9 for Interactive REPL and Debugging Infrastructure, Phase 10 for Service Integration & IDE Connectivity, Phase 11 for Local LLM Integration, and Phase 12 for Adaptive Memory System, essential for agent intelligence.

### Phase Categories

- **MVP Foundation** (Phases 0-2): Core functionality with comprehensive tools for minimal viable product
- **MVP Completion** (Phase 3): Tool enhancement, agent infrastructure, and bridge integration
- **Production Infrastructure** (Phases 4-6): Hook system, state management, and sessions
- **Infrastructure Consolidation** (Phase 7): Foundational solidification for production readiness
- **Advanced Features** (Phase 8): Vector storage and search infrastructure
- **Developer Experience** (Phase 9): Interactive REPL and debugging infrastructure
- **Service & IDE Integration** (Phase 10): External service layer and IDE connectivity
- **Local LLM Support** (Phase 11): Ollama and Candle integration for offline operation
- **Advanced AI** (Phase 12): Adaptive memory system with temporal knowledge graphs
- **Protocol Support** (Phases 13-14): MCP client and server integration
- **Language Extensions** (Phase 15): JavaScript engine support
- **Platform Support** (Phases 16-17): Library mode and cross-platform support
- **Production Optimization** (Phase 18): Performance and security hardening
- **Future Enhancements** (Phases 19+): Extended tools, A2A protocols, multimodal, and AI/ML tools

---

## MVP Foundation Phases

### **Phase 0: Foundation Infrastructure (Weeks 1-2)**

**Goal**: Establish core project infrastructure and build system  
**Priority**: CRITICAL (MVP Prerequisite)

**Components**:
- Cargo workspace setup with all crates (`llmspell-core`, `llmspell-agents`, `llmspell-tools`, etc.)
- Basic trait definitions (`BaseAgent`, `Agent`, `Tool`, `Workflow`)
- Error handling system with `LLMSpellError` types
- Logging infrastructure with `tracing`
- CI/CD pipeline setup

**Success Criteria**:
- [x] All crates compile without warnings
- [x] Basic trait hierarchy compiles
- [x] CI runs successfully on Linux
- [x] Documentation builds without errors
- [x] `cargo test` passes for foundation tests

**Testing Requirements**:
- Unit tests for all trait definitions
- Basic error handling validation
- CI pipeline smoke tests
- Documentation generation tests

---

### **Phase 1: Core Execution Runtime (Weeks 3-4)**

**Goal**: Implement core execution engine and basic Lua scripting  
**Priority**: CRITICAL (MVP Core)

**MVP Scope**:
- `ScriptEngineBridge` trait implementation (language-agnostic foundation)
- `LuaEngine` as first concrete implementation of ScriptEngineBridge
- `ComponentLifecycleManager` with basic 5-phase initialization
- Language-agnostic API injection through bridge pattern
- Factory pattern for runtime creation with different engines
- Lua coroutine-based streaming support through bridge
- Simple LLM provider integration using `rig`
- Provider capability detection for multimodal support
- Provider abstraction layer for future multimodal providers (mistral.rs, etc.)
- In-memory state management

**Essential Components**:
- `llmspell-utils` crate with shared utilities
- `ScriptEngineBridge` trait for language abstraction
- `LuaEngine` implementing ScriptEngineBridge
- Engine factory pattern for future extensibility
- Language-agnostic ScriptRuntime using Box<dyn ScriptEngineBridge>
- `BaseAgent` trait implementation with streaming support
- `Agent` trait with basic LLM calling and multimodal types
- `Tool` trait with schema validation and streaming interface
- Basic CLI entry point (`llmspell run script.lua --engine lua`) with streaming output support
- Multimodal content types (`MediaContent`, enhanced `AgentInput`/`AgentOutput`)

**Success Criteria**:
- [x] `llmspell-utils` crate provides common utilities to all crates
- [x] ScriptEngineBridge abstraction works (not just Lua integration)
- [x] Engine factory pattern functional
- [x] Directory structure supports multi-language from day one
- [x] API injection is language-agnostic (ready for Phase 14)
- [x] Can execute simple Lua scripts through ScriptEngineBridge abstraction
- [x] LLM providers can be called from scripts
- [x] Basic tool execution works
- [x] Streaming methods defined (implementation can be stub)
- [x] Multimodal types compile and are accessible from scripts
- [x] Error propagation from scripts to CLI
- [x] Runtime can switch between engines (even with only Lua implemented)
- [x] Third-party engine plugin interface defined
- [x] Memory usage stays under 50MB for simple scripts

**Testing Requirements**:
- ScriptEngineBridge trait behavior tests
- Engine factory pattern validation
- Cross-engine API consistency framework (ready for Phase 14)
- Script execution integration tests
- Language-agnostic API injection testing
- Bridge abstraction unit tests
- Engine implementation compliance tests
- LLM provider mock testing
- Memory usage benchmarks
- Error handling validation
- CLI command testing

---

### **Phase 2: Self-Contained Tools Library (Weeks 5-8)** ✅ COMPLETE

**Goal**: Implement all self-contained tools without external dependencies
**Priority**: CRITICAL (MVP Essential)
**Status**: COMPLETE - All 25 self-contained tools implemented, including basic WebSearchTool

**Expanded Scope** (All Self-Contained Tools):
- **File System** (8 tools): `FileOperationsTool`, `ArchiveHandlerTool`, `file_watcher`, `file_converter`, `file_search`
- **Data Processing** (4 tools): `JsonProcessorTool`, `CsvAnalyzerTool`, `DataValidationTool`, `TemplateEngineTool`
- **Utilities & Helpers** (8 tools): `calculator`, `text_manipulator`, `date_time_handler`, `uuid_generator`, `hash_calculator`, `base64_encoder`, `diff_calculator`
- **System Integration** (4 tools): `environment_reader`, `process_executor`, `service_checker`, `system_monitor`
- **Simple Media** (3 tools): `image_processor`, `audio_processor`, `video_processor` (basic operations only)

**Essential Components**:
- Tool registry and discovery system
- Tool schema validation with media type support
- Streaming tool interface implementation
- Security sandboxing for tools
- Tool error handling and timeout management
- **Provider Enhancement Features** (moved from Phase 1):
  - `ModelSpecifier` implementation for provider abstraction
  - `ProviderManager` to parse "provider/model" syntax (e.g., "openai/gpt-4", "anthropic/claude-3")
  - Base URL overrides in agent configuration for custom endpoints
  - CLI support for new model specification syntax
  - Built-in tools examples using convenience syntax
- **JSON API for Script Bridge**:
  - Language-agnostic JSON parsing/stringifying in bridge layer
  - `JsonApiDefinition` added to `ApiSurface` in engine types
  - `inject_json_api()` function for each script engine
  - `JSON.parse()` and `JSON.stringify()` available in all scripts
  - Enables proper handling of tool outputs (which return JSON strings)

**Success Criteria**:
- [x] 26+ self-contained tools functional across all categories
- [x] All utilities (calculator, uuid, hash, etc.) working
- [x] All file system tools (watcher, converter, search) implemented
- [x] All system integration tools (env reader, process executor, etc.) working
- [x] Simple media tools (basic image/audio/video operations) functional
- [x] Tool security sandbox prevents unauthorized access
- [x] Tool execution timeout enforcement works
- [x] Tools can be called from both Agent and direct script context
- [x] ModelSpecifier parsing works for all supported providers
- [x] CLI accepts both full configuration and "provider/model" syntax
- [x] Base URL overrides function correctly for custom endpoints
- [x] JSON API available in all script environments (JSON.parse/stringify)
- [x] Tool outputs (JSON strings) can be parsed to native objects
- [x] Scripts can work with structured data from tool results

**Testing Requirements**:
- Individual tool unit tests
- Tool registry integration tests
- Security sandbox validation
- Timeout enforcement tests
- Cross-tool compatibility tests
- JSON API functionality tests (parse/stringify roundtrip)
- Tool output parsing validation
- Cross-language JSON consistency tests (same behavior in all engines)

---

### **Phase 3: Tool Enhancement & Agent Infrastructure (Weeks 9-16)**

**Goal**: Standardize tools, add external integrations, implement agent infrastructure, bridge integration, and basic multi-agent coordination  
**Priority**: HIGH (MVP Completion)
**Duration**: 8 weeks (expanded from original 2 weeks)

#### **Phase 3.0: Critical Tool Fixes (Weeks 9-10)**
**Goal**: Standardize existing 26 tools before adding new ones

**Scope**:
- Tool signature standardization (ResponseBuilder pattern)
- Consistent parameter naming across all tools
- DRY principle enforcement (use shared validators, retry logic)
- Breaking changes documentation and migration tools

**Success Criteria**:
- [ ] 95% parameter consistency (from 60%)
- [ ] ResponseBuilder pattern adopted by all tools
- [ ] Migration guide and tools complete
- [ ] All shared utilities extracted to llmspell-utils

#### **Phase 3.1: External Integration Tools (Weeks 11-12)**
**Goal**: Add external integration tools following new standards

**Scope** (8 External Integration Tools):
- **Web & Network** (7 tools): 
  - `WebSearchTool` enhancement (add Google, Brave, DuckDuckGo, SerpApi, SerperDev APIs)
  - `web_scraper`, `url_analyzer`, `api_tester`, `webhook_caller`, `webpage_monitor`, `sitemap_crawler`
- **Communication** (1 tool): `email_sender`, `database_connector`

**Success Criteria**:
- [ ] WebSearchTool enhanced with 5 real API implementations (DuckDuckGo, Google, Brave, SerpApi, SerperDev)
- [ ] 7 new external tools functional
- [ ] All tools follow Phase 3.0 standards from day one
- [ ] Rate limiting and authentication working

#### **Phase 3.2: Security & Performance (Weeks 13-14)**
**Goal**: Harden security and optimize performance for all 33 tools

**Scope**:
- Security hardening (calculator DoS protection, symlink prevention)
- Resource limit enforcement across all tools
- Performance optimization (shared resource pools, caching)
- Comprehensive security test suite

**Success Criteria**:
- [ ] All security vulnerabilities addressed
- [ ] Resource limits enforced (memory, CPU, file sizes)
- [ ] Performance maintained at 52,600x target
- [ ] Security test coverage >95%

#### **Phase 3.3: Agent Infrastructure & Basic Workflows (Weeks 15-16)**
**Goal**: Implement agent infrastructure and basic multi-agent coordination patterns
**Status**: IN PROGRESS - Core infrastructure complete, bridge integration partial, provider architecture enhancement added

**Scope**:
- ✅ Enhanced agent lifecycle management (Tasks 3.3.3-3.3.4)
- ✅ Agent registry and discovery (Tasks 3.3.1-3.3.2)
- ✅ BaseAgent tool integration infrastructure (Task 3.3.2)
- ⚠️ Script-to-agent integration bridge (Task 3.3.9 - PARTIAL)
- ❌ **Script-to-workflow integration bridge** (Task 3.3.16)
- ✅ Agent-as-tool wrapping support (infrastructure exists)
- ✅ Tool composition patterns (infrastructure exists)
- ✅ Agent composition patterns (Task 3.3.6)
- ❌ **Basic Workflow Patterns** (Sequential, Conditional, Loop, Parallel)
- ❌ **Workflow-Agent Integration**
- ❌ **CLI Integration for Agents and Workflows**
- ⚠️ Lua agent examples and documentation (basic only)
- ✅ Multi-agent coordination primitives (Task 3.3.7)
- 🆕 **Provider Architecture Enhancement** (Task 3.3.23)
  - Add `provider_type` field to ProviderConfig
  - Implement hierarchical provider naming (e.g., `rig/openai/gpt-4`)
  - Fix "Unsupported provider: rig" error

**Bridge Integration Strategy**:
1. **Tool Bridge**: Pre-existing infrastructure ✅
2. **Agent Bridge**: Script-to-agent integration (Task 3.3.9) ⚠️ PARTIAL
   - ✅ Basic agent creation and execution from Lua
   - ✅ Agent discovery (list types, templates)
   - ❌ Agent-to-tool invocation through bridge
   - ❌ Monitoring/lifecycle/composition access
   - ❌ Enhanced ExecutionContext support
3. **Workflow Bridge**: Script-to-workflow integration (Task 3.3.16) ❌ NOT STARTED
4. **Unified Bridge APIs**: Consistent patterns across all component types ❌ INCOMPLETE

**Success Criteria**:
- [x] Agent lifecycle hooks implemented (Task 3.3.3)
- [x] Agent registry and discovery functional (Tasks 3.3.1-3.3.2)
- [x] BaseAgent tool integration functional (Task 3.3.2)
- [❌] Agents can discover and invoke tools from 33+ tool ecosystem **via bridge**
- [x] Agent-as-tool wrapping works seamlessly (infrastructure only)
- [x] Tool composition patterns demonstrated (infrastructure only)
- [x] Agent composition patterns demonstrated (Task 3.3.6)
- [❌] **Basic workflow patterns functional** (Sequential, Conditional, Loop, Parallel)
- [❌] **Workflow-agent integration operational**
- [❌] **CLI updated with agent and workflow commands**
- [❌] **Multi-agent coordination via workflows demonstrated**
- [⚠️] Agents accessible from scripts via bridges (basic access only)
- [⚠️] Script-to-agent calling mechanism functional (limited to text I/O)
- [❌] **Script-to-workflow bridge operational**
- [❌] **Workflow discovery from scripts functional**
- [⚠️] Lua agent examples working and documented (basic examples only)
- [⚠️] Agent and workflow discovery from scripts operational (agents yes, workflows no)
- [❌] **All four basic workflow patterns accessible from Lua**
- [ ] **Provider architecture supports type separation** (Task 3.3.23)
- [ ] **Hierarchical provider naming implemented** (e.g., `rig/openai/gpt-4`)

**Current Bridge Limitations (Task 3.3.9)**:
- Only ~20% of agent infrastructure exposed to scripts
- No tool discovery/invocation from script-created agents
- No monitoring, lifecycle, or composition pattern access
- Limited to basic text I/O (no multimodal/streaming)
- Missing workflow integration entirely

**Testing Requirements**:
- Tool standardization validation tests
- Migration tool testing
- External API integration tests
- Security vulnerability tests
- Workflow integration tests with all tools

---
## Production Features Phases

### **Phase 4: Hook and Event System (Weeks 17-18.5)** 

**Goal**: Implement comprehensive hooks and events system with cross-language support and production patterns  
**Priority**: HIGH (Production Essential)
**Dependencies**: Requires Phase 3.3 Agent Infrastructure
**Timeline Note**: Extended by 2-3 days for future-proofing, saves 2+ weeks in later phases

**Components**:
- Hook execution framework with 20+ hook points and **HookAdapter trait for language flexibility**
- Event bus using `tokio-stream` + `crossbeam` with **FlowController for backpressure handling**
- Built-in hooks (logging, metrics, debugging, **caching, rate limiting, retry, cost tracking, security**)
- Script-accessible hook registration with **language-specific adapters (Lua sync, JS promises, Python async)**
- Agent lifecycle hooks integration with **ReplayableHook trait for Phase 5 persistence**
- **CircuitBreaker for automatic performance protection (<5% overhead guaranteed)**
- **UniversalEvent format for cross-language event propagation**
- **DistributedHookContext for future A2A protocol support (Phase 18-19)**
- **CompositeHook patterns (Sequential, Parallel, FirstMatch, Voting)**
- **Enhanced HookResult enum (Continue, Modified, Cancel, Redirect, Replace, Retry, Fork, Cache, Skipped)**

**Success Criteria**:
- [ ] Pre/post execution hooks work for agents and tools with **automatic circuit breaking**
- [ ] Hook execution works for 6 agent states, 34 tools, and 4 workflow patterns
- [ ] Event emission and subscription functional with **backpressure handling**
- [ ] Built-in logging, metrics, **caching, rate limiting, retry, cost tracking, and security** hooks operational
- [ ] Scripts can register custom hooks in **Lua (sync), JavaScript (promises), and Python (async) patterns**
- [ ] Hook execution doesn't significantly impact performance (<5% overhead **enforced by CircuitBreaker**)
- [ ] **Cross-language event propagation works (Lua→JS, JS→Lua, etc.)**
- [ ] **ReplayableHook trait enables hook persistence for Phase 5**
- [ ] **Performance monitoring integrated with automatic hook disabling**

**Testing Requirements**:
- Hook execution order validation
- Event bus performance tests with **backpressure scenarios**
- Script hook registration tests **across languages**
- Performance impact measurements
- Performance regression testing from day 1 to ensure <5% overhead
- Hook error handling tests
- **CircuitBreaker triggering and recovery tests**
- **Cross-language event propagation tests**
- **Composite hook execution tests**

---

### **Phase 5: Persistent State Management (Weeks 19-20)**

**Goal**: Implement persistent state storage with sled/rocksdb  
**Priority**: MEDIUM (Production Important)
**Dependencies**: Requires Phase 4 Hook System for state change notifications
**Phase 4 Integration**: This phase leverages the ReplayableHook trait and HookContext serialization from Phase 4 to enable state replay and hook history persistence.

**Note**: This phase will leverage the `llmspell-storage` infrastructure implemented in Phase 3.3, extending it for general state management beyond agent registry persistence.

**Components**:
- `StateManager` with persistent backend (using llmspell-storage)
- Agent state serialization/deserialization (extending StorageSerialize trait)
- State migration and versioning
- Backup and recovery mechanisms
- Hook integration for state change events
- **Hook history persistence using ReplayableHook trait from Phase 4**
- **State replay with hook execution reconstruction**
- **Event correlation for state timeline visualization**

**Success Criteria**:
- [ ] Agent state persists across application restarts
- [ ] State can be serialized and restored correctly
- [ ] Multiple agents can have independent state
- [ ] State migrations work for schema changes
- [ ] Backup/restore operations functional
- [ ] **Hook history is persisted and replayable**
- [ ] **State changes trigger appropriate hooks**
- [ ] **Event correlation IDs link state changes**

**Testing Requirements**:
- State persistence integration tests
- Serialization roundtrip tests
- Migration pathway validation
- Backup/restore functionality tests
- Multi-agent state isolation tests
- **Hook replay functionality tests**
- **State timeline reconstruction tests**

---

### **Phase 6: Session and Artifact Management (Weeks 21-22)**

**Goal**: Implement session management and artifact storage  
**Priority**: MEDIUM (Production Enhancement)
**Dependencies**: Requires Phase 5 Persistent State Management
**Phase 4 Integration**: Session boundaries are managed through hooks (session:start, session:end) with automatic artifact collection and event correlation.

**Note**: Session and artifact storage will use the `llmspell-storage` backend infrastructure for consistent persistence patterns across the system.

**Components**:
- Session lifecycle management **with built-in hooks**
- Artifact storage and retrieval system (using llmspell-storage)
- Session context preservation **via HookContext**
- Artifact versioning and metadata
- Session replay capabilities **using ReplayableHook**
- Integration with state management
- **Automatic artifact collection hooks**
- **Cross-session event correlation via UniversalEvent**

**Success Criteria**:
- [ ] Sessions can be created, saved, and restored
- [ ] Artifacts are stored with proper metadata
- [ ] Session context preserved across restarts
- [ ] Artifact versioning and history tracking works
- [ ] Session replay functionality operational
- [ ] **Session hooks fire at appropriate boundaries**
- [ ] **Artifacts are automatically collected via hooks**
- [ ] **Event correlation links session activities**

**Testing Requirements**:
- Session lifecycle tests
- Artifact storage and retrieval tests
- Session context preservation validation
- Artifact versioning tests
- Session replay functionality tests
- **Session hook integration tests**
- **Artifact collection hook tests**

---

### **Phase 7: Infrastructure Consolidation and Foundational Solidification (Weeks 23-29)**

**Goal**: Consolidate and solidify all framework infrastructure to enable production-ready AI workflow orchestration at scale  
**Priority**: CRITICAL (Framework Foundation - Prerequisite for Production)
**Dependencies**: Requires Phase 6 Session Management completion
**Status**: Implementation Complete - Scope expanded from API polish to comprehensive infrastructure overhaul
**Timeline Note**: Extended by 2 weeks due to discovery of critical architectural gaps requiring immediate resolution

**Components**:
- **Test Infrastructure Revolution (536+ files refactored)**:
  - Centralized `llmspell-testing` crate replacing scattered test helpers across all crates
  - Feature-based test categorization replacing broken cfg_attr syntax
  - MockBaseAgent implementation with comprehensive test fixtures
  - Test execution scripts enabling targeted testing by category (<5s unit, <30s integration)
- **Configuration Architecture Revolution (2,700+ lines)**:
  - EnvRegistry system managing 45+ standardized environment variables
  - Hierarchical TOML configuration with schema validation
  - Configuration migration from scattered settings to centralized management
  - Environment variable precedence and validation framework
- **Security Architecture Revolution**:
  - Mandatory bridge-provided sandbox for 7 security-sensitive tools
  - Permission validation preventing privilege escalation
  - Configuration injection prevention with strict validation
  - Security threat modeling and mitigation implementation
- **Bridge Architecture Revolution (Fix for "Missing Link Problem")**:
  - ComponentLookup trait enabling real component execution (not mock data)
  - StepExecutor fixed with registry field for actual tool/agent access
  - StateWorkflowAdapter implementing Google ADK patterns
  - Single execution path eliminating mock vs real execution divergence
- **API Standardization Foundation (50+ APIs)**:
  - BaseAgent trait unification with execute_impl() pattern
  - Consistent builder patterns across all configuration objects
  - Standardized error handling and result types
  - Method naming consistency (get_*, set_*, with_*, *Manager)
- **Production Validation via WebApp Creator**:
  - 20-agent orchestration proving infrastructure readiness
  - 170-second end-to-end execution demonstrating performance
  - Complex workflow validation across all infrastructure components
  - Real-world application proving framework production readiness

**Major Discoveries and Resolutions**:
- **Critical Discovery**: StepExecutor couldn't execute ANY real components (all returned mock data)
- **Root Cause**: Missing ComponentRegistry access in workflow execution chain
- **Resolution**: Complete bridge architecture overhaul with ComponentLookup trait
- **Validation**: WebApp Creator successfully orchestrates 20 agents in production scenario

**Success Criteria**:
- [x] Test infrastructure centralized with feature-based categorization working
- [x] Configuration system handles 45+ environment variables with validation
- [x] Security sandbox mandatory for all sensitive tool operations
- [x] Bridge architecture enables real component execution (no mock fallbacks)
- [x] API standardization complete with consistent patterns across 50+ APIs
- [x] WebApp Creator validates 20-agent orchestration in <3 minutes
- [x] All infrastructure supports multi-language bridge expansion (JS, Python ready)
- [x] Performance targets met (<10ms tool init, <5% hook overhead, >90K events/sec)
- [x] Migration tools enable automated infrastructure upgrades
- [x] Production deployment patterns established with Kubernetes manifests

**Testing Requirements**:
- Infrastructure integration tests validating all components work together
- Performance benchmarking suite with automated regression detection
- Security penetration testing for sandbox escape prevention
- Configuration validation tests for all 45+ environment variables
- Bridge architecture tests proving real execution (not mock)
- WebApp Creator end-to-end validation across multiple scenarios
- Migration tool testing for automated codebase upgrades
- Load testing with concurrent agent execution (20+ agents)
- Memory stress testing for long-running workflows
- Component lookup performance tests (>1500 lookups/second)

---

### **Phase 8: Vector Storage and Search Infrastructure (Weeks 28-29)**

**Goal**: Implement vector storage backends and advanced search capabilities as foundation for memory system
**Priority**: MEDIUM (Advanced Features - Critical for Phase 9 Memory)
**Dependencies**: Requires Phase 6 Session Management for search context
**Phase 4 Integration**: High-frequency embedding events handled by FlowController, with CachingHook for embedding reuse and performance monitoring.

**Components**:
- `VectorStorageBackend` trait implementations (memory, disk, external)
- `llmspell-rag` crate with RAG patterns and document chunking
- `SemanticSearchTool` implementation using vector storage
- `CodeSearchTool` implementation with tree-sitter integration
- **Integration with CachingHook for embedding caching**
- **Event-driven vector indexing with backpressure control**
- **Performance monitoring for vector operations**

**Essential Components**:
- **HNSW Indexing**: Hierarchical Navigable Small World graphs for <300ms P95 latency
  - Parallel index building (85% build time reduction)
  - Dynamic dataset support for insert/delete without rebuild
  - Hybrid HNSW-IF for billion-scale datasets
- **Embedding Strategy**: 
  - BGE-M3 model support (8192 token context, multi-lingual)
  - ColBERT v2 late interaction for fine-grained relevance
  - Hybrid retrieval: dense + sparse + multi-vector
- **Chunking Strategies**:
  - SentenceSplitter (best performance per benchmarks)
  - Landmark Embeddings for chunking-free approach
  - Adaptive chunking based on document structure
- **Optimization Techniques**:
  - d-HNSW disaggregated memory architecture
  - Representative index caching for compute efficiency
  - Dimensional reduction for performance
  - Software-level caching for hot paths

**Memory System Preparation**:
- Episodic memory foundation via vector embeddings
- Temporal metadata support in vector storage
- Event capture infrastructure for memory ingestion
- Hybrid search API (vector + keyword + future graph)

**Success Criteria**:
- [ ] Vector storage backends operational (memory, disk-based)
- [ ] HNSW index with <300ms P95 query latency
- [ ] BGE-M3 embedding model integrated
- [ ] Semantic search with embeddings functional
- [ ] Code search with AST parsing and symbol extraction working
- [ ] RAG pipeline patterns implemented
- [ ] Performance acceptable for 100k+ vectors
- [ ] **Embedding cache hit rate >80% for repeated content**
- [ ] **Vector indexing handles high-frequency updates via backpressure**
- [ ] Temporal metadata stored with vectors for Phase 9

**Testing Requirements**:
- Vector similarity search accuracy tests
- HNSW index performance benchmarks
- BGE-M3 embedding quality validation
- RAG pipeline integration tests
- Code parsing and search validation
- Performance benchmarks for vector operations
- **Embedding cache effectiveness tests**
- **Backpressure handling under load tests**
- Temporal metadata persistence tests

**Research Notes**:
- Zep achieves 94.8% accuracy using BGE-M3 embeddings
- HNSW with disaggregated memory reduces latency by 85%
- ColBERT v2 + SentenceSplitter best for RAG retrieval
- Hybrid retrieval (dense + sparse + multi-vector) critical for accuracy

---

### **Phase 9: Interactive REPL and Debugging Infrastructure (Weeks 30-32)**

**Goal**: Implement interactive REPL for development with comprehensive debugging capabilities
**Priority**: HIGH (Developer Experience - Critical for adoption)
**Dependencies**: Requires Phase 8 Vector Storage for search context
**Evolution**: Combines original REPL concept with advanced debugging infrastructure from Phase-9 branch learnings

**Core Architecture**:
- **REPL Engine**: Interactive command-line interface with state persistence
- **Debug Infrastructure**: Enhanced error reporting, breakpoints, and variable inspection
- **Development Tools**: Hot reload, script validation, and performance profiling
- **Integration**: Hook introspection, event stream visualization, LSP support

**Phase 9.1: Core REPL Infrastructure (Week 30)**:
- `llmspell repl` command implementation
- State persistence between REPL commands
- Multi-line input handling with syntax highlighting
- Tab completion for APIs and variable names
- Command history with search (Ctrl+R style)
- REPL-specific commands (`.help`, `.save`, `.load`, `.exit`)
- Streaming output display with progress indicators
- Media file input/output support
- Integration with existing runtime and bridge

**Phase 9.2: Enhanced Error Reporting (Week 31 - Part 1)**:
- **Error Context Enhancement**:
  - Source mapping to original files
  - Line and column number tracking
  - Variable state at error point
  - Execution path visualization
- **Beautiful Error Formatting**:
  - Rust-style error messages with colors
  - Source context with line highlighting
  - Suggested fixes based on error patterns
  - Stack traces with local variables
- **Common Error Detection**:
  - Nil value access patterns
  - Type mismatches
  - Missing function detection
  - Automatic suggestions

**Phase 9.3: Interactive Debugging (Week 31 - Part 2)**:
- **Breakpoint System**:
  - Set breakpoints via `.break file:line` command
  - Conditional breakpoints with expressions
  - Hit counts and ignore counts
  - Breakpoint management (list, enable, disable, delete)
- **Step Debugging**:
  - `.step` - Step into functions
  - `.next` - Step over functions
  - `.continue` - Resume execution
  - `.up`/`.down` - Navigate call stack
- **Variable Inspection**:
  - `.locals` - Show local variables
  - `.watch expr` - Watch expressions
  - `.print var` - Inspect specific variables
  - Deep object inspection with formatting

**Phase 9.4: Development Experience (Week 32 - Part 1)**:
- **Hot Reload Support**:
  - File watcher for automatic reload
  - State preservation across reloads
  - Validation before reload
  - Error recovery without losing session
- **Script Validation**:
  - `.validate` command for syntax checking
  - Static analysis for undefined variables
  - Type inference warnings
  - Common mistake detection
- **Performance Profiling**:
  - `.profile` command to start/stop profiling
  - Flame graph generation
  - Memory usage tracking
  - Execution time analysis

**Phase 9.5: Advanced Features (Week 32 - Part 2)**:
- **Hook Integration**:
  - `.hooks list` - List active hooks
  - `.hooks trace` - Trace hook execution
  - Real-time event stream visualization
  - Performance monitoring with circuit breaker status
- **Remote Debugging**:
  - Debug server mode for remote attachment
  - Debug Adapter Protocol (DAP) support
  - Session recording and replay
  - Distributed tracing integration
- **IDE Support**:
  - Language Server Protocol (LSP) implementation
  - VS Code extension integration
  - Completion and hover support
  - Diagnostic integration

**Success Criteria**:
- [ ] REPL starts and accepts commands interactively
- [ ] State persists between REPL commands
- [ ] Multi-line scripts can be entered and executed
- [ ] Tab completion works for APIs and variables
- [ ] Command history is saved and restored
- [ ] Enhanced error messages show source context and suggestions
- [ ] Breakpoints can be set and hit during execution
- [ ] Step debugging works (step, next, continue)
- [ ] Variables can be inspected at any point
- [ ] Hot reload works without losing state
- [ ] Script validation catches errors before execution
- [ ] Hook introspection commands functional
- [ ] Event stream can be monitored in real-time
- [ ] Performance profiling generates useful insights
- [ ] Remote debugging via DAP protocol works
- [ ] LSP integration provides IDE support

**Testing Requirements**:
- REPL command execution tests
- State persistence validation across sessions
- User interaction simulation tests
- Tab completion functionality tests
- History management tests
- Error enhancement accuracy tests
- Breakpoint hit detection tests
- Step debugging functionality tests
- Variable inspection correctness tests
- Hot reload state preservation tests
- Script validation accuracy tests
- Hook introspection command tests
- Event stream display tests
- Performance profiling accuracy tests
- Remote debugging connectivity tests
- LSP protocol compliance tests

**Implementation Notes**:
- REPL serves as the primary development interface
- Debugging is not a separate mode but integrated into REPL
- All debug commands are REPL meta-commands (start with `.`)
- State persistence uses existing Phase 5 infrastructure
- Hook introspection leverages Phase 4 hook system
- Performance monitoring uses Phase 4 CircuitBreaker
- Error enhancement improves developer productivity by 80%+

---

### **Phase 10: Service Integration & IDE Connectivity (Weeks 33-36)**

**Goal**: Implement external service layer for client connectivity and IDE integration
**Priority**: HIGH (Critical for Developer Experience and External Tool Integration)
**Duration**: 4 weeks (optimized from 6 weeks by combining shared infrastructure)
**Dependencies**:
- ✅ **Phase 9** (Kernel Integration): Kernel foundation with message loop and DAP bridge
- ✅ **Phase 4** (Hook System): FlowController and CircuitBreaker for service stability

**Rationale**: Service mode and IDE integration share 60-70% of infrastructure (transport layers, connection management, multi-client support). Combining them accelerates time-to-value by 6 weeks and enables immediate IDE connectivity after Phase 9.

#### **10.1: Service Infrastructure Foundation (Week 33)**

**Core Service Layer**:
- **`llmspell serve` Command Framework**:
  ```bash
  llmspell serve --jupyter --port 8888     # Jupyter Lab connectivity
  llmspell serve --dap --port 8889         # VS Code debugging
  llmspell serve --lsp --port 8890         # Language server
  llmspell serve --repl                    # Interactive REPL
  llmspell serve --all                     # Multi-protocol mode
  ```

- **Multi-Protocol Service Manager**:
  ```rust
  pub struct ServiceManager {
      jupyter_service: Option<JupyterService>,
      dap_service: Option<DAPService>,
      lsp_service: Option<LSPService>,
      repl_service: Option<REPLService>,
      client_registry: ClientConnectionRegistry,
      session_manager: MultiClientSessionManager,
  }
  ```

- **Connection Registry & Discovery**:
  - Service announcement via mDNS/DNS-SD
  - Connection file generation (Jupyter kernel.json)
  - Port management and conflict resolution
  - Client authentication and authorization
  - Session isolation and resource limits

- **Transport Layer Implementation**:
  - ZeroMQ for Jupyter 5-channel protocol
  - TCP/WebSocket for DAP and LSP
  - Unix domain sockets for local connections
  - TLS 1.3 for secure remote connections

#### **10.2: Jupyter Lab Integration (Week 34 - Part 1)**

**ZeroMQ Transport Implementation**:
- **5-Channel Architecture**:
  ```rust
  pub struct JupyterTransport {
      shell: zmq::Socket,      // Execute requests
      iopub: zmq::Socket,      // Output publication
      stdin: zmq::Socket,      // Input requests
      control: zmq::Socket,    // Control commands
      heartbeat: zmq::Socket,  // Keep-alive
  }
  ```

- **Kernel Registration & Discovery**:
  - Generate kernel.json with connection info
  - Install kernel spec for Jupyter discovery
  - Support custom kernel display names
  - Handle multiple kernel instances

- **Message Processing Integration**:
  - Connect ZeroMQ transport to IntegratedKernel
  - Message signing and verification (HMAC)
  - Parent header tracking for message correlation
  - Comm channel support for widgets

- **Notebook Features**:
  - Rich display support (HTML, images, plots)
  - Magic commands (%time, %debug, etc.)
  - Tab completion and introspection
  - Inline documentation and help

#### **10.2: VS Code Integration (Week 34 - Part 2)**

**Debug Adapter Protocol Server**:
- **DAP 1.0 Protocol Implementation**:
  ```rust
  pub struct DAPServer {
      tcp_listener: TcpListener,
      dap_bridge: Arc<Mutex<DAPBridge>>,  // From Phase 9
      protocol_handler: DAPProtocolHandler,
      client_sessions: HashMap<ClientId, DAPSession>,
  }
  ```

- **Essential DAP Commands**:
  - initialize/launch/attach
  - setBreakpoints/setExceptionBreakpoints
  - continue/next/stepIn/stepOut/pause
  - stackTrace/scopes/variables/evaluate
  - disconnect/terminate

- **VS Code Extension**:
  - One-click debugging with auto-kernel start
  - Integrated terminal with REPL
  - Syntax highlighting for Lua/JavaScript
  - Breakpoint management UI
  - Variable watch and hover inspection
  - Call stack visualization
  - Debug console with expression evaluation

#### **10.3: Language Server Protocol (Week 35 - Part 1)**

**LSP Implementation**:
- **Core LSP Features**:
  ```rust
  pub struct LSPServer {
      tcp_listener: TcpListener,
      kernel: Arc<IntegratedKernel>,
      document_store: DocumentStore,
      symbol_index: SymbolIndex,
  }
  ```

- **Language Intelligence**:
  - Code completion from runtime context
  - Go-to-definition using kernel state
  - Find references across scripts
  - Real-time diagnostics from execution
  - Hover information with type details
  - Code actions and quick fixes
  - Document formatting and refactoring

- **Multi-Language Support**:
  - Lua language server features
  - JavaScript support (future-proofing)
  - Custom DSL highlighting
  - Mixed-language documents

#### **10.3: Interactive REPL Service (Week 35 - Part 2)**

**Enhanced REPL Implementation**:
- **REPL Server Features**:
  - Multi-client REPL sessions
  - Session persistence and replay
  - Command history across sessions
  - Tab completion from kernel state
  - Inline documentation
  - Debug commands integration

- **Advanced REPL Capabilities**:
  - Hot code reloading
  - Watch mode for file changes
  - Performance profiling commands
  - Memory inspection tools
  - State visualization

#### **10.4: Service Deployment & Management (Week 36 - Part 1)**

**System Service Integration**:
- **Service Management**:
  ```bash
  # systemd integration
  systemctl start llmspell.service
  systemctl enable llmspell.service

  # launchd integration (macOS)
  launchctl load ~/Library/LaunchAgents/com.llmspell.plist
  ```

- **Production Features**:
  - Automatic restart on failure
  - Health checks and monitoring
  - Resource limits and quotas
  - Log rotation and management
  - Metrics export (Prometheus)

- **Scheduler Integration**:
  - Cron-like job scheduling
  - Interval-based task execution
  - Event-driven triggers
  - Task queue management

#### **10.4: Multi-Client & Security (Week 36 - Part 2)**

**Multi-Client Architecture**:
- **Session Management**:
  - Client isolation and sandboxing
  - Shared kernel with separate contexts
  - Collaborative debugging support
  - Session migration between clients

- **Security Layer**:
  - TLS 1.3 for all remote connections
  - Certificate-based authentication
  - OAuth2/OIDC integration
  - RBAC for operations
  - Audit logging
  - Secret masking in output

**Success Criteria**:
- ✅ **`llmspell serve` launches multi-protocol server**
- ✅ **Jupyter Lab connects and executes notebooks**
- ✅ **VS Code debugging works with <20ms stepping**
- ✅ **LSP provides code intelligence in any IDE**
- ✅ **Multiple clients connect simultaneously**
- ✅ **Service runs as systemd/launchd daemon**
- ✅ **Session persistence across restarts**
- ✅ **Health monitoring and auto-restart functional**
- ✅ **Performance: <5ms message handling maintained**
- ✅ **Security: TLS + auth for remote connections**

**Testing Requirements**:
- Multi-protocol server startup tests
- Jupyter kernel protocol compliance
- DAP protocol compliance validation
- LSP protocol compliance tests
- Multi-client synchronization tests
- Service stability (24-hour run)
- Security penetration testing
- Performance benchmarks under load
- Session persistence validation
- Crash recovery testing

---

### **Phase 11: Local LLM Integration (Weeks 37-41)**

**Goal**: Implement dual-path local LLM support via Ollama (external process) and Candle (embedded Rust) with unified model management
**Priority**: CRITICAL (Enables offline and cost-free LLM operations)
**Timeline**: 20 working days (both Ollama and Candle)
**Dependencies**: Phase 10 Service Integration ✅

**Architecture Overview**:
- **Dual Implementation**: Both Ollama and Candle support in single phase
- **Unified UX**: Same script API regardless of backend (Ollama vs Candle)
- **Model Agnostic**: Support LLaMA 3.1, Mistral, Phi-3, Gemma 2 via both paths
- **CLI-First**: Model download/install managed through `llmspell model` subcommand
- **Provider Abstraction**: Extend existing ProviderInstance trait for local providers

**Phase 11.1: Ollama Integration (Days 1-10)**:
- Fast path to local LLM (mature ecosystem)
- Uses ollama-rs crate (v0.3.2+)
- External process dependency at localhost:11434
- Rich model library via `ollama pull`
- Streaming support built-in
- Auto-start capability

**Phase 11.2: Candle Integration (Days 11-20)**:
- Pure Rust embedded inference
- Uses candle-core + candle-transformers
- GGUF format from HuggingFace
- GPU acceleration (CUDA/Metal)
- Q4_K_M quantization default
- No external dependencies

**Unified Model Management**:
- Single CLI: `llmspell model` for both backends
- Consistent syntax: `local/<model>:<variant>[@backend]`
- Auto-detection: Prefers Ollama, falls back to Candle
- Examples:
  - `local/llama3.1:8b` (auto-detect)
  - `local/phi3:3.8b@ollama` (force Ollama)
  - `local/mistral:7b@candle` (force Candle)

**Recommended Models**:
1. **Phi-3 Mini (3.8B)** - Smallest, fastest (~2.4GB)
2. **Mistral 7B** - Best quality/size ratio (~4.1GB)
3. **LLaMA 3.1 8B** - Most capable (~4.7GB)
4. **Gemma 2 9B** - Google's efficient model (~5.4GB)

**Script API** (`LocalLLM` global):
```lua
-- Check status
local status = LocalLLM.status()

-- List local models
local models = LocalLLM.list()

-- Create agent with local model (backend auto-detected)
local agent = Agent.create({
    model = "local/llama3.1:8b",
    temperature = 0.7
})

-- Download model from script
LocalLLM.pull("ollama/llama3.1:8b")
```

**CLI Commands**:
```bash
# Check status
llmspell model status

# List local models
llmspell model list [--backend ollama|candle]

# Download models
llmspell model pull ollama/phi3:3.8b
llmspell model pull candle/mistral:7b --quantization Q4_K_M

# View available models
llmspell model available [--recommended]

# Install Ollama
llmspell model install-ollama
```

**Performance Targets**:
- **First token latency**: <100ms (Ollama), <200ms (Candle)
- **Throughput**: >20 tokens/second for 7B models
- **Memory**: <5GB RAM for Q4_K_M quantized models
- **Model load**: <5 seconds
- **Cold start**: <10 seconds total

**Example Applications**:
Created 5 example applications in `examples/script-users/applications/local-chat/`:
1. `local_chat.lua` - Simple chat with auto-detection
2. `ollama_chat.lua` - Ollama-specific example
3. `candle_inference.lua` - Candle-specific with benchmarking
4. `backend_comparison.lua` - Compare Ollama vs Candle performance
5. `llmspell.toml` - Configuration template

**Tracing Integration**:
All code includes comprehensive tracing with trace!, debug!, info!, warn!, error! macros following Phase 10 patterns:
- info!() for lifecycle events (provider init, model loading)
- debug!() for operational details (config values, timing)
- trace!() for fine-grained flow (tokenization, file loading)
- warn!() for recoverable issues (model not found, fallback)
- error!() for critical failures (GGUF parse errors, API failures)

**Success Criteria**:
- [ ] `llmspell model list` shows available local models (both Ollama and Candle)
- [ ] `llmspell model pull ollama/llama3.1:8b` downloads via Ollama
- [ ] `llmspell model pull candle/mistral:7b` downloads GGUF for Candle
- [ ] Scripts use `model = "local/llama3.1:8b"` syntax (backend auto-detected)
- [ ] Ollama provider functional with streaming support
- [ ] Candle provider functional with GGUF loading
- [ ] Performance: <100ms first token, >20 tokens/sec for 7B models
- [ ] Memory: <5GB RAM for Q4_K_M quantized models
- [ ] Examples demonstrate both Ollama and Candle usage
- [ ] Comprehensive test coverage for both providers
- [ ] Documentation covers installation, configuration, and usage

**Testing Requirements**:
- Ollama provider unit tests (health check, list models, completion)
- Candle provider unit tests (model loading, GGUF parsing, inference)
- Integration tests (full workflow from pull to completion)
- Backend switching tests (agent can use both backends)
- Performance benchmarks (meet latency/throughput targets)
- Memory usage validation (<5GB for Q4_K_M models)
- Example application validation (all 5 examples working)

**Research References**:
- Ollama: Go-based LLM server (localhost:11434) with easy model management
- ollama-rs: Rust client library (v0.3.2+) for Ollama API
- rig-core: Rust LLM framework (v0.20.0) with native Ollama provider support
- Candle: HuggingFace's pure Rust ML framework for embedded LLM inference
- GGUF Format: Quantized model format for efficient inference (Q4_K_M quantization)

**Design Document**: `/docs/in-progress/phase-11-design-doc.md` (2,500+ lines)

---

### **Phase 11a: Bridge Consolidation & Documentation Completeness (Weeks 41.5-44.5)**

**Goal**: Consolidate bridge layer, standardize APIs, and achieve documentation completeness before Phase 12
**Priority**: HIGH (Foundation for Phase 12, MCP, Agent-to-Agent)
**Timeline**: 3-4 weeks (October 2025)
**Dependencies**: Phase 11 Local LLM Integration ✅
**Type**: CONSOLIDATION (Quality, Performance, Documentation)
**Status**: ✅ COMPLETE

**Consolidation Philosophy**:
Phase 11a addresses the gap between major feature phases. Phase 11 delivered dual local LLM support (Ollama + Candle), Phase 12 requires solid foundation for Adaptive Memory (A-TKG), and Phase 11a ensures clean handoff: fast builds, clear docs, consistent APIs.

**Core Principles**:
- **Consolidation over Innovation**: Strengthen existing capabilities, not add new features
- **Developer Experience First**: 87% faster bridge-only compile, zero ambiguity in APIs
- **Documentation as Code**: 40%→95% coverage, environment variables 0%→100%
- **API Consistency**: Eliminate parallel methods, standardize naming (Tool.execute)
- **Code Simplification**: Remove unused code (StepType::Custom), clean abstractions
- **Foundation for Scale**: Enable Phase 12 (Memory), Phase 13 (MCP), Phase 14 (A2A)

**Phase 11a Sub-phases** (8 total):

**11a.1-7: Feature Gate Architecture (87% Compile Speedup)**:
- Cargo feature flags isolate language runtimes (lua, javascript)
- Bridge-only builds: 38s → 5s (87% improvement)
- Pattern extends to future runtimes (Python, Ruby, MCP)
- Zero runtime performance impact

```toml
# llmspell-bridge features
[features]
default = ["lua", "javascript"]
lua = ["mlua"]
javascript = ["boa_engine"]
```

```bash
# Fast bridge-only compile
cargo build -p llmspell-bridge --no-default-features  # 5s vs 38s
```

**11a.10: Workflow Output Collection (Agent Introspection)**:
- Add agent_outputs to WorkflowResult for debugging multi-step workflows
- Collect agent outputs: `Vec<(String, serde_json::Value)>`
- Enable workflow introspection without custom logging
- Foundation for Phase 14 (Agent-to-Agent) result passing

```lua
local result = workflow:execute({})

-- Debug agent outputs
for i, output in ipairs(result.agent_outputs) do
    print("Agent " .. output[1] .. ":", output[2])
end
```

**11a.11: API Method Naming Standardization**:
- Standardize on Tool.execute() across all 40+ tools
- Eliminate parallel methods (call, invoke)
- Update examples, documentation, tests
- Zero ambiguity for users

```lua
-- Consistent across all tools
local result = Tool.execute("file-operations", {operation = "read", path = "data.txt"})
```

**11a.12: Custom Steps Removal (Code Simplification)**:
- Remove unused StepType::Custom variant
- Delete 5 custom step files (876 lines)
- Simplify workflow abstractions (Tool | Agent only)
- Cleaner maintenance burden

**11a.13: Security Sandbox Documentation (40%→95% Coverage)**:
- Create security-and-permissions.md user guide (371 lines)
- Fix configuration.md TOML schema (remove fake [security.sandboxing])
- Document 3 security levels (Safe/Restricted/Privileged)
- Add 4 common scenarios + 5 troubleshooting guides
- Fix critical Config global bug (empty stub → full implementation)

**11a.14: Environment Variables Documentation (0%→100% Coverage)**:
- Document 50+ security environment variables (41+ documented)
- Add to configuration.md, security-and-permissions.md, getting-started.md
- CI/CD patterns: GitHub Actions, GitLab CI
- Container patterns: Docker, Docker Compose
- Service patterns: systemd configuration
- Total new content: 405 lines across 3 files

**Success Criteria** - ✅ ALL COMPLETE:
- [x] Bridge compile time <5s (was 38s) via feature gates
- [x] Agent outputs accessible in WorkflowResult.agent_outputs
- [x] Tool.execute() unified across all invocation patterns
- [x] StepType::Custom enum variant removed
- [x] Security documentation 40%→95% with user guide
- [x] Environment variables 0%→100% documented (41+ vars)
- [x] Zero compiler warnings across workspace
- [x] All quality gates passing (format, clippy, compile, test, doc)
- [x] Config global bug fixed (empty stub → full implementation)
- [x] ADR-042 (Feature Gates) and ADR-043 (Workflow Outputs) documented

**Quality Metrics Achieved**:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Bridge compile time | 38s | 5s | 87% faster |
| Security docs coverage | 40% | 95% | +55% |
| Env vars documentation | 0% | 100% | +100% |
| API consistency (tools) | 60% | 100% | +40% |
| TOML schema accuracy | 30% | 95% | +65% |
| Code removed | 0 | 876 lines | Simplification |
| Documentation lines | baseline | +1,866 lines | Comprehensive |

**Environment Variables Documented** (41+ security vars):
- **Runtime Security**: ALLOW_FILE_ACCESS, ALLOW_NETWORK_ACCESS, ALLOW_PROCESS_SPAWN
- **File Operations**: ALLOWED_PATHS, MAX_FILE_SIZE, BLOCKED_EXTENSIONS (7 vars)
- **Web Search**: ALLOWED_DOMAINS, BLOCKED_DOMAINS, RATE_LIMIT (5 vars)
- **HTTP Request**: ALLOWED_HOSTS, BLOCKED_HOSTS, TIMEOUT, VERIFY_SSL (8 vars)
- **System/Process**: ALLOW_PROCESS_EXEC, ALLOWED_COMMANDS, TIMEOUT (8 vars)
- **Network Config**: TIMEOUT, RETRIES, VERIFY_SSL (3 vars)
- **State Persistence**: ENABLED, PATH, AUTO_SAVE, AUTO_LOAD (4 vars)

**Deployment Patterns Documented**:
- ✅ GitHub Actions workflow examples
- ✅ GitLab CI configuration
- ✅ Docker container (Dockerfile)
- ✅ Docker Compose multi-service
- ✅ systemd service units
- ✅ Single command overrides (quick testing)

**Testing**:
- Feature gate tests (lua/javascript isolation)
- Workflow output collection tests (5 unit tests)
- Tool.execute() validation across all tools
- Config global verification tests
- Quality check scripts passing

**Impact on Future Phases**:
- **Phase 12 (Memory)**: Fast iteration (87% compile), workflow debugging, security docs
- **Phase 13 (MCP)**: Feature gates extend to MCP backends, Tool.execute for MCP tools
- **Phase 14 (A2A)**: Workflow introspection for A2A result passing, security isolation
- **Phase 15 (Dynamic Workflows)**: Simplified StepType enum easier to generate

**Design Document**: `/docs/in-progress/phase-11a-design-doc.md` (comprehensive)

---

### **Phase 12: Adaptive Memory System (Weeks 42-46)**

**Goal**: Implement Adaptive Temporal Knowledge Graph (A-TKG) memory architecture with IDE visualization
**Priority**: HIGH (Core AI Capability)
**Dependencies**:
- Phase 8: Vector Storage as foundation
- Phase 10: IDE integration for memory visualization
- Phase 11: Local LLM support for knowledge extraction
**Research Foundation**: Based on Zep/Graphiti (94.8% DMR accuracy) and Mem0 (26% improvement over OpenAI)

**Memory Architecture Overview**:
- **Working Memory**: Immediate session context (managed by `llmspell-state`)
- **Episodic Memory**: Raw interactions indexed by vectors (built on Phase 8 `llmspell-rag`)
- **Semantic Memory**: Temporal Knowledge Graph storing facts, entities, relationships
- **Adaptive Consolidation**: LLM-driven memory management (add/update/delete logic)

**Phase 12.1: Foundational Episodic Memory (Week 42)**:
- Create `llmspell-memory` crate with core data structures
- Implement `InteractionLog` and `MemoryItem` types
- Integrate with `llmspell-events` for interaction capture
- Asynchronous ingestion pipeline via hooks
- Basic vector retrieval using Phase 8 infrastructure
- Memory persistence via `llmspell-storage`

**Phase 12.2: Temporal Knowledge Graph Foundation (Weeks 43-44)**:
- **New Crate: `llmspell-graph`**
  - Bi-temporal data model (event time + ingestion time)
  - Node/Edge structures with temporal validity intervals
  - Entity resolution and deduplication
  - Incremental graph updates without full rebuild
- **Graph Storage Backend**:
  - Embedded Rust solution (primary)
  - Neo4j adapter (enterprise)
  - Storage trait abstraction via `llmspell-storage`
- **Knowledge Extraction Pipeline**:
  - LLM-driven entity/relationship extraction
  - Temporal information parsing
  - Contradiction detection and resolution

**Phase 12.3: Hybrid Retrieval System (Week 45)**:
- **Memory Orchestrator** in `llmspell-memory`:
  - Unified API for all memory types
  - Query planning and routing logic
  - Result fusion and re-ranking
- **Hybrid Search Strategy**:
  - Vector search for semantic similarity (episodic)
  - Graph traversal for relationships (semantic)
  - BM25 keyword search for exact matches
  - Temporal filtering for point-in-time queries
- **Performance Targets**:
  - P95 latency <300ms (matching Zep benchmark)
  - No LLM calls during retrieval
  - Support for 1M+ memory items

**Phase 12.4: Adaptive Consolidation (Week 45)**:
- **Memory Consolidation Pipeline** (Mem0-inspired):
  - Periodic review of memory items
  - LLM-driven decisions: Add/Update/Delete/Ignore
  - Importance scoring based on usage patterns
  - Conflict resolution for contradictions
- **Episodic Summarization**:
  - Compress old interactions into summaries
  - Extract key facts into TKG
  - Prune detailed events beyond threshold
- **Adaptive Feedback Loops**:
  - Track memory item usage via hooks
  - Adjust importance scores based on outcomes
  - Self-improving relevance ranking

**Phase 12.5: Integration and Polish (Week 46)**:
- **Script API** (`MemoryGlobal`):
  - `Memory.store()` - Store new memories
  - `Memory.search()` - Semantic search
  - `Memory.graphQuery()` - Graph traversal
  - `Memory.buildContext()` - Unified context assembly
- **Agent Integration**:
  - Automatic memory injection into agent context
  - Memory-aware tool selection
  - Cross-session continuity
- **IDE Visualization** (NEW - leverages Phase 10):
  - VS Code memory graph explorer
  - Jupyter notebook memory inspection
  - Real-time memory state via DAP
  - Memory performance profiler
- **Observability**:
- **Script API** (`MemoryGlobal`):
  - `Memory.store()` - Store new memories
  - `Memory.search()` - Semantic search
  - `Memory.graphQuery()` - Graph traversal
  - `Memory.buildContext()` - Unified context assembly
- **Agent Integration**:
  - Automatic memory injection into agent context
  - Memory-aware tool selection
  - Cross-session continuity
- **Observability**:
  - Memory growth metrics
  - Retrieval performance monitoring
  - Consolidation effectiveness tracking

**Success Criteria**:
- [ ] A-TKG architecture fully operational
- [ ] 94%+ accuracy on memory benchmarks (target: Zep level)
- [ ] P95 retrieval latency <300ms
- [ ] Bi-temporal queries working correctly
- [ ] Memory consolidation reduces storage by >50%
- [ ] Cross-session agent continuity functional
- [ ] Graph supports 100k+ entities, 1M+ relationships
- [ ] Hybrid retrieval outperforms vector-only by >15%

**Testing Requirements**:
- Memory accuracy benchmarks (DMR, LongMemEval)
- Temporal reasoning test suite
- Graph consistency validation
- Consolidation effectiveness tests
- Cross-session continuity tests
- Performance stress tests (1M+ items)
- Hybrid retrieval accuracy comparison

**Research References**:
- Zep/Graphiti: Temporal Knowledge Graph Architecture (arXiv:2501.13956)
- Mem0: Scalable Long-Term Memory (arXiv:2504.19413)
- Graph RAG vs Vector RAG benchmarks showing 80% vs 50.83% accuracy
- BGE-M3 + ColBERT v2 for optimal retrieval performance

---

## Advanced Integration Phases


### **Phase 13: MCP Tool Integration (Weeks 47-48)**

**Goal**: Support Model Control Protocol for external tools
**Priority**: LOW (Advanced Integration)

**Components**:
- MCP client implementation
- External tool discovery and registration
- MCP protocol compliance
- Connection management and retries

**Success Criteria**:
- [ ] Can connect to MCP servers
- [ ] External tools are discoverable and callable
- [ ] MCP protocol messages handled correctly
- [ ] Connection failures handled gracefully
- [ ] Tool schemas from MCP servers validated

**Testing Requirements**:
- MCP protocol compliance tests
- External tool integration tests
- Connection failure recovery tests
- Tool schema validation tests
- Multi-server connection tests

---

### **Phase 14: MCP Server Mode (Weeks 49-50)**

**Goal**: Expose rs-llmspell tools and agents via MCP protocol
**Priority**: LOW (Advanced Integration)

**Components**:
- MCP server implementation
- Tool and agent exposure via MCP
- Multi-client support
- Protocol compliance and testing

**Success Criteria**:
- [ ] MCP server accepts client connections
- [ ] Built-in tools are accessible via MCP
- [ ] Agents can be called as MCP tools
- [ ] Multiple clients can connect simultaneously
- [ ] Protocol compliance verified with test suite

**Testing Requirements**:
- MCP server compliance tests
- Multi-client connection tests
- Tool/agent exposure validation
- Protocol message handling tests
- Load testing with multiple clients

---

### **Phase 15: JavaScript Engine Support (Weeks 51-52)**

**Goal**: Add JavaScript support via bridge pattern established in Phase 1
**Priority**: MEDIUM (Multi-Language Support)
**Dependencies**: Phase 1 ScriptEngineBridge foundation

**Components**:
- JavaScript engine selection and integration (V8, QuickJS, or Boa)
- `JavaScriptEngine` implementing `ScriptEngineBridge`
- JavaScript-specific API bindings
- Node.js compatibility layer
- npm package support (selective)

**Success Criteria**:
- [ ] JavaScript scripts execute correctly
- [ ] Bridge abstraction maintains <5% overhead
- [ ] Mixed Lua/JavaScript execution works
- [ ] Basic npm packages can be used
- [ ] Memory usage acceptable (<100MB overhead)

**Testing Requirements**:
- JavaScript execution tests
- Bridge overhead benchmarks
- Cross-language interop tests
- npm integration tests
- Memory usage monitoring

---

### **Phase 16: Library Mode Support (Weeks 53-54)**

**Goal**: Agent-to-Agent communication as client
**Priority**: LOW (Advanced Networking)

**Components**:
- A2A protocol client implementation
- Agent discovery and delegation
- Network communication and retries
- Task distribution patterns

**Success Criteria**:
- [ ] Can discover remote agents via A2A protocol
- [ ] Task delegation to remote agents works
- [ ] Network failures handled with retries
- [ ] Agent capabilities properly negotiated
- [ ] Distributed task execution functional

**Testing Requirements**:
- A2A protocol client tests
- Agent discovery validation
- Network failure simulation tests
- Task delegation integration tests
- Distributed execution tests

---

### **Phase 17: Cross-Platform Support (Weeks 55-56)**

**Goal**: Expose local agents via A2A protocol
**Priority**: LOW (Advanced Networking)

**Components**:
- A2A protocol server implementation
- Agent exposure and capability advertisement
- Multi-agent coordination
- Load balancing and failover

**Success Criteria**:
- [ ] A2A server accepts agent connections
- [ ] Local agents discoverable by remote clients
- [ ] Multi-agent coordination works
- [ ] Load is distributed across available agents
- [ ] Failover mechanisms handle agent failures

**Testing Requirements**:
- A2A server implementation tests
- Agent exposure validation
- Multi-agent coordination tests
- Load balancing functionality tests
- Failover mechanism tests

---

## Production Optimization Phase

### **Phase 18: Production Optimization (Weeks 57-58)**

**Goal**: Support usage as native module in external runtimes
**Priority**: MEDIUM (Alternative Usage Mode)

**Components**:
- C API layer for FFI
- `RuntimeMode::Library` implementation
- `SelectiveInitStrategy` for partial initialization
- Native module packaging (LuaRock, NPM)

**Success Criteria**:
- [ ] Can be compiled as shared library
- [ ] C API allows external lua_State injection
- [ ] Selective initialization works (tools-only, agents-only)
- [ ] Native modules can be required in external scripts
- [ ] Memory management safe in external runtimes

**Testing Requirements**:
- C API functionality tests
- External runtime integration tests
- Memory safety validation
- Selective initialization tests
- Native module packaging tests

---

### **Phase 19: A2A Client Support (Weeks 59-60)**

**Goal**: Agent-to-Agent communication as client
**Priority**: LOW (Advanced Networking)

**Components**:
- A2A protocol client implementation
- Agent discovery and delegation
- Network communication and retries
- Task distribution patterns

**Success Criteria**:
- [ ] Can discover remote agents via A2A protocol
- [ ] Task delegation to remote agents works
- [ ] Network failures handled with retries
- [ ] Agent capabilities properly negotiated
- [ ] Distributed task execution functional

**Testing Requirements**:
- A2A protocol client tests
- Agent discovery validation
- Network failure simulation tests
- Task delegation integration tests
- Distributed execution tests

---

### **Phase 20: A2A Server Support (Weeks 61-62)**

**Goal**: Expose local agents via A2A protocol  
**Priority**: LOW (Advanced Networking)

**Components**:
- A2A protocol server implementation
- Agent exposure and capability advertisement
- Multi-agent coordination
- Load balancing and failover

**Success Criteria**:
- [ ] A2A server accepts agent connections
- [ ] Local agents discoverable by remote clients
- [ ] Multi-agent coordination works
- [ ] Load is distributed across available agents
- [ ] Failover mechanisms handle agent failures

**Testing Requirements**:
- A2A server implementation tests
- Agent exposure validation
- Multi-agent coordination tests
- Load balancing functionality tests
- Failover mechanism tests

---

### **Phase 21: Multimodal Tools Implementation (Weeks 63-64)**

**Goal**: Implement comprehensive multimodal processing tools  
**Priority**: MEDIUM (Feature Enhancement)
**Dependencies**: Requires Phase 8 Vector Storage and Phase 12 Memory System
**Phase 4 Integration**: Media processing hooks enable dynamic parameter adjustment, progress tracking, and cost monitoring for expensive operations.

**Components**:
- Image processing tools (resize, crop, format conversion)
- OCR tool with multiple language support
- Video processing tools (frame extraction, thumbnail generation)
- Audio transcription tool (stub with interface)
- Media format conversion utilities
- **PDF creation tools (separate from Phase 7 extraction)**:
  - **PdfCreatorTool** using `krilla` library (high-level, ergonomic PDF creation)
  - Operations: `create_document`, `render_from_html`, `add_text`, `add_image`, `merge_pdfs`
  - Template-based PDF generation with JSON data
  - Support for accessible PDFs (tagged PDF, PDF/A compliance)
  - Alternative: `papermake` for high-volume document generation
  - **Architecture Note**: Intentionally separate from PdfProcessorTool (Phase 7) which uses `pdf-extract` for reading PDFs. Different libraries optimize for different use cases (reading vs writing)
  - **CRITICAL TODO**: Replace Phase 7's `pdf-extract` with `lopdf` (async support) or `pdfium-render` (robust extraction) due to blocking/hanging issues on complex PDFs
- Integration with multimodal workflows
- **Hook-based parameter optimization for large media files**
- **Progress hooks for long-running operations**
- **Cost tracking for AI-powered media analysis**

**Success Criteria**:
- [ ] Image processing tools handle common formats (PNG, JPEG, WebP)
- [ ] OCR tool extracts text from images accurately
- [ ] Video tools can extract frames and generate thumbnails
- [ ] Audio transcription interface defined (implementation can be stub)
- [ ] **PDF creation tools generate valid PDFs** (separate from Phase 7 PDF extraction):
  - [ ] PdfCreatorTool creates PDFs from templates and JSON data
  - [ ] HTML-to-PDF rendering works with CSS styling
  - [ ] PDF merging combines multiple PDFs correctly
  - [ ] Generated PDFs pass validation in multiple viewers
  - [ ] Accessible PDF features work (tagged PDF, PDF/A)
- [ ] Tools integrate smoothly with streaming workflows
- [ ] Media type validation works correctly
- [ ] **Hooks automatically optimize processing for large files**
- [ ] **Progress events emitted for operations >1 second**
- [ ] **Cost tracking accurate for AI operations**

**Testing Requirements**:
- Individual tool functionality tests
- Media format compatibility tests
- **PDF creation tests (separate from Phase 7 extraction tests)**:
  - PDF generation from templates
  - HTML-to-PDF rendering validation
  - PDF merging functionality
  - Cross-viewer compatibility tests (Adobe, Chrome, Firefox, Preview)
  - Accessibility compliance validation (PDF/A, tagged PDF)
- Integration tests with workflows
- Performance benchmarks for media processing
- Error handling for invalid media
- **Hook-based optimization tests**
- **Progress tracking accuracy tests**

---

### **Phase 22: AI/ML Complex Tools (Weeks 65-66)**

**Goal**: Implement AI and ML dependent complex tools  
**Priority**: MEDIUM (Advanced AI Features)
**Phase 4 Integration**: CostTrackingHook, RateLimitHook, and RetryHook from Phase 4 are essential for production AI/ML tool deployment.

**Components**:
- **AI/ML Tools** (6 tools): text_summarizer, sentiment_analyzer, language_detector, text_classifier, named_entity_recognizer, embedding_generator
- **Advanced Multimodal** (8 tools): image_analyzer, ocr_extractor, audio_transcriber, image_generator, media_converter, face_detector, scene_analyzer
- Model loading and inference infrastructure
- Local model support and optimization
- **AI/ML Tools with automatic cost tracking via Phase 4 hooks**
- **Rate limiting with backoff for API quota management**
- **Retry mechanisms for transient AI service failures**

**Success Criteria**:
- [ ] All AI/ML tools functional with local models
- [ ] Advanced multimodal tools handle complex operations
- [ ] Model loading and caching optimized
- [ ] Performance acceptable for production use
- [ ] Integration with vector storage (from Phase 8)
- [ ] **Cost tracking accurate for all AI operations**
- [ ] **Rate limiting prevents API quota exhaustion**
- [ ] **Automatic retry handles transient failures**

**Testing Requirements**:
- AI/ML tool accuracy validation
- Model loading performance tests
- Advanced multimodal integration tests
- Memory usage optimization tests
- Production deployment validation

---

### **Phase 23: Advanced Workflow Features (Weeks 67-70)**
**Goal**: Implement sophisticated workflow orchestration capabilities and advanced patterns
**Priority**: LOW (Advanced Enhancement)
**Dependencies**: Builds on Phase 3 (Workflows), Phase 8 (Vector Storage), Phase 12 (Memory System), and Phase 10 (Service Integration)

**Advanced Workflow Patterns**:
```rust
// Conditional Branching with Complex Logic
pub struct ConditionalWorkflow {
    condition_evaluator: Box<dyn ConditionEvaluator>,
    branches: HashMap<String, Workflow>,
    default_branch: Option<Box<Workflow>>,
    state_machine: WorkflowStateMachine,
}

// Parallel Execution with Synchronization
pub struct ParallelWorkflow {
    parallel_branches: Vec<Workflow>,
    synchronization_strategy: SyncStrategy,
    merge_handler: Box<dyn MergeHandler>,
    resource_pool: WorkflowResourcePool,
}

// Dynamic Workflow Generation
pub struct DynamicWorkflow {
    workflow_generator: Box<dyn WorkflowGenerator>,
    runtime_compiler: WorkflowCompiler,
    validation_engine: WorkflowValidator,
}

// Event-Driven Workflows
pub struct EventDrivenWorkflow {
    event_handlers: HashMap<EventType, WorkflowHandler>,
    event_bus: EventBus,
    correlation_engine: EventCorrelator,
}
```

**Components**:
- **Advanced Pattern Library**:
  - Saga patterns for distributed transactions
  - Circuit breaker workflows for fault tolerance
  - Retry with exponential backoff and jitter
  - Bulkhead isolation patterns
  - Pipeline with back-pressure handling
  - Fork-join with custom merge strategies
  - Map-reduce workflow patterns
  - State machine workflows with persistence

- **Workflow Composition Engine**:
  - Runtime workflow composition from templates
  - Workflow inheritance and extension
  - Mixin patterns for reusable workflow components
  - Workflow versioning and migration
  - Hot-reload of workflow definitions

- **Optimization Engine**:
  - Workflow execution plan optimization
  - Resource allocation and scheduling
  - Parallel execution optimization
  - Caching strategies for workflow results
  - Predictive pre-computation based on patterns

- **Monitoring and Observability**:
  - Workflow execution tracing with OpenTelemetry
  - Performance profiling per workflow step
  - Resource usage tracking and alerts
  - Workflow analytics and insights
  - Visual workflow execution timeline

- **Integration Features**:
  - Workflow-as-a-Service API
  - REST/GraphQL workflow triggers
  - Webhook integration for external events
  - Message queue integration (AMQP, Kafka)
  - Workflow federation across instances

**Advanced Capabilities**:
```rust
// Workflow Template System
pub struct WorkflowTemplateEngine {
    template_registry: TemplateRegistry,
    parameter_resolver: ParameterResolver,
    validation_rules: ValidationRuleSet,
    instantiation_context: Context,
}

impl WorkflowTemplateEngine {
    pub fn instantiate_from_template(
        &self,
        template_id: &str,
        parameters: Parameters,
    ) -> Result<Workflow> {
        let template = self.template_registry.get(template_id)?;
        let resolved = self.parameter_resolver.resolve(&template, parameters)?;
        self.validation_rules.validate(&resolved)?;
        Ok(resolved)
    }
}

// Workflow Scheduling System
pub struct WorkflowScheduler {
    schedule_store: ScheduleStore,
    trigger_engine: TriggerEngine,
    execution_queue: PriorityQueue<ScheduledWorkflow>,
    resource_manager: ResourceManager,
}

// Workflow Testing Framework
pub struct WorkflowTestHarness {
    mock_registry: MockServiceRegistry,
    simulation_engine: SimulationEngine,
    assertion_framework: AssertionFramework,
    coverage_analyzer: CoverageAnalyzer,
}
```

**Success Criteria**:
- [ ] All advanced workflow patterns implemented and tested
- [ ] Dynamic workflow composition working with validation
- [ ] Parallel execution with proper synchronization
- [ ] Event-driven workflows with correlation support
- [ ] Workflow versioning and migration tools complete
- [ ] Performance optimization showing 30%+ improvement
- [ ] Monitoring integration with full observability
- [ ] Template system with 50+ built-in templates
- [ ] Testing framework with simulation capabilities
- [ ] Documentation with pattern cookbook

**Testing Requirements**:
- Complex workflow pattern integration tests
- Parallel execution stress tests
- Event correlation accuracy tests
- Template instantiation validation
- Performance benchmarks vs baseline
- Resource utilization tests
- Failure recovery scenarios
- Load testing with 1000+ concurrent workflows
- End-to-end workflow federation tests

**Performance Targets**:
- Workflow compilation: <100ms for complex workflows
- Parallel branch overhead: <5ms per branch
- Event correlation: <10ms for 10,000 events
- Template instantiation: <50ms
- Workflow state persistence: <20ms
- Resource scheduling decisions: <5ms

---

## MVP Definition

### Minimal Viable Product (MVP)

**MVP includes Phases 0-3** (Foundation through Agent Infrastructure)

**Essential Traits**:
- `BaseAgent` - Foundation trait with tool-handling capabilities
- `Agent` - LLM wrapper with specialized prompts
- `Tool` - LLM-callable functions with schema validation
- `Workflow` - Deterministic orchestration patterns
- `ScriptEngineBridge` - Language abstraction for script engines

**Essential Components**:
- `ScriptRuntime` - Central execution orchestrator
- `ScriptEngineBridge` - Language abstraction layer
- `LuaEngine` - First concrete engine implementation
- Engine factory pattern - Runtime creation with different engines
- Comprehensive built-in tools - 33+ tools across categories (25 from Phase 2, 8 from Phase 3.1)
- Standardized tool interfaces - ResponseBuilder pattern and consistent parameters
- Security hardening - DoS protection, resource limits
- Agent infrastructure - Lifecycle, communication, registry, composition

**Essential Features**:
- Lua scripting with Agent/Tool APIs
- LLM provider calling via `rig`
- Tool execution with security sandboxing
- Agent infrastructure and coordination
- CLI interface (`llmspell run script.lua`)

### MVP Success Criteria

- [ ] Can run Lua scripts that use agents and tools
- [ ] Can call LLM providers from scripts
- [ ] Has 33+ essential built-in tools across all categories
- [ ] All tools follow standardized interfaces (95% consistency)
- [ ] Security vulnerabilities addressed
- [ ] Agent infrastructure operational
- [ ] External integration tools functional
- [ ] Migration guide for breaking changes complete
- [x] Runs on Linux with stable performance
- [x] Memory usage under 50MB for simple scripts
- [x] Complete test coverage for all MVP components
- [x] Documentation covers all MVP features
- [x] External integration tools functional (web, APIs, databases)
- [x] Self-contained tools provide comprehensive utility coverage

---

## Implementation Strategy

### Priority Order

1. **Immediate Priority** (Phases 0-2.5): MVP foundation with comprehensive tools
   - Phase 1.2 MUST implement ScriptEngineBridge foundation
   - NO direct Lua coupling allowed in ScriptRuntime
   - Bridge pattern implementation is CRITICAL for future phases
2. **High Priority** (Phases 3-4, 7, 11-12, 18): Agent infrastructure, hooks, API consistency, local LLM support, memory system, and production optimization
3. **Medium Priority** (Phases 5-6, 8-10, 15-17, 21-23): State management, sessions, vector storage, REPL, IDE integration, platform support, and AI/ML tools
   - Phase 15 (JavaScript) becomes much simpler due to existing bridge infrastructure
   - Additional engines can be added as medium priority features
4. **Low Priority** (Phases 13-14, 19-20): MCP protocols and A2A support

### Breaking Changes Strategy

- **Pre-1.0**: Breaking changes allowed between any phases
- **Phase 1.2 Exception**: ScriptEngineBridge API must be stable before Phase 2
- **Engine Interface Stability**: ScriptEngineBridge API frozen after Phase 1.2
- **Post-1.0**: Breaking changes only at major version boundaries
- **Engine Plugin API**: Stable interface for third-party engines from Phase 1.2
- **Migration tooling**: Provide migration tools for configuration and state
- **Deprecation cycle**: 2-phase deprecation cycle for API changes

### Testing Milestones

Each phase must pass:
1. **Unit Tests**: All components tested in isolation
2. **Integration Tests**: Component interactions validated
3. **Performance Tests**: Performance requirements met
4. **Security Tests**: Security requirements validated
5. **Cross-Platform Tests**: Platform compatibility verified (when applicable)

### Dependencies and Prerequisites

- **Phase 0**: No prerequisites
- **Phases 1-2.5**: Sequential dependency (each depends on previous)
- **Phase 3**: Depends on comprehensive tools (Phase 2.5)
- **Phases 4+**: Depends on MVP completion (Phases 0-3)
- **Phase 4**: Hook System depends on Phase 3.3 Agent Infrastructure
- **Phase 5**: State Management depends on Phase 4 Hook System **(specifically ReplayableHook trait)**
- **Phase 6**: Session Management depends on Phase 5 State Management
- **Phase 7**: API Consistency depends on Phase 6 Session Management completion
- **Phase 8**: Vector Storage depends on Phase 7 API Standardization
- **Phase 9**: REPL and Debugging depends on Phase 8 Vector Storage for search context
- **Phase 10**: Service & IDE Integration depends on Phase 9 (Kernel Integration)
- **Phase 11**: Local LLM Integration depends on Phase 10 Service Integration (for model management service)
- **Phase 12**: Adaptive Memory System depends on Phase 8 Vector Storage, Phase 10 IDE for visualization, and Phase 11 Local LLM for knowledge extraction
- **Phase 13**: MCP Tool Integration depends on Phase 10 Service infrastructure
- **Phase 19-20**: A2A Protocol depends on Phase 4 **(DistributedHookContext required)**
- **Phase 15**: JavaScript Engine Support depends on MVP completion + ScriptEngineBridge foundation from Phase 1.2 **(greatly simplified by Phase 4 JavaScriptHookAdapter)**
- **Phase 16**: Library Mode depends on Phase 4 **(SelectiveHookRegistry needed)**
- **Phase 23**: Advanced Workflow Features depends on Phase 8 Vector Storage and Phase 12 Memory System
- **Phase 21**: Multimodal Tools depends on Phase 12 Memory System
- **Phase 22**: AI/ML Tools depends on Phase 4 **(CostTrackingHook essential)**
- **Cross-language testing**: Can begin in Phase 1 with bridge abstraction tests
- **Engine implementations**: Can be developed in parallel once ScriptEngineBridge is stable
- **Third-party engines**: Can be added after Phase 1.2 completion using bridge pattern
- **Cross-cutting features**: Can be developed in parallel where dependencies allow

---

## Timeline and Resources

### Estimated Timeline

- **MVP Foundation**: ✅ COMPLETE (Phases 0-2, delivered in 8 weeks)
- **MVP with External Tools & Agent Infrastructure**: 16 weeks (Phases 0-3)
- **Production Infrastructure**: 22 weeks (Phases 0-6, includes enhanced hooks +3 days)
- **Pre-1.0 Polish**: 29 weeks (Phases 0-7, API consistency and documentation)
- **Advanced Features**: 32 weeks (Phases 0-8, includes vector storage and search)
- **Developer Experience**: 32 weeks (Phases 0-9, REPL and debugging infrastructure)
- **Service & IDE Integration**: 36 weeks (Phases 0-10, external connectivity and IDE support)
- **Local LLM Support**: 41 weeks (Phases 0-11, Ollama and Candle integration)
- **Advanced AI**: 46 weeks (Phases 0-12, adaptive memory system with IDE visualization)
- **Protocol Support**: 50 weeks (Phases 0-14, MCP client and server)
- **Multi-Language Ready**: 52 weeks (Phases 0-15, JavaScript support)
- **Library & Platform Support**: 56 weeks (Phases 0-17, library mode and cross-platform)
- **Production Ready**: 58 weeks (Phases 0-18, optimization and hardening)
- **Distributed Computing**: 62 weeks (Phases 0-20, A2A protocol support)
- **Advanced Tools**: 66 weeks (Phases 0-22, multimodal and AI/ML tools)
- **Full Feature Set**: 70 weeks (All 23 phases)

### Resource Requirements

- **Core Development**: 1-2 full-time developers
- **Bridge Architecture**: 0.5 FTE dedicated to ScriptEngineBridge design in Phase 1.2
- **Engine Implementation**: Can parallelize after bridge foundation
- **Testing and QA**: 0.5 full-time equivalent
- **Cross-Engine Testing**: Additional 0.25 FTE for multi-language validation
- **Documentation**: 0.25 full-time equivalent
- **DevOps/Infrastructure**: 0.25 full-time equivalent

### Risk Mitigation

- **Dependency risks**: Alternative crate selections identified
  - **mlua alternatives**: If mlua doesn't work with bridge pattern, have alternatives ready
  - **JavaScript engine selection**: Choose engine that works well with bridge pattern
  - **Bridge trait design**: Get trait design right in Phase 1.2, hard to change later
- **Performance risks**: Early benchmarking and optimization
  - **Performance Risk**: Bridge abstraction must not add significant overhead
  - **Performance Risk**: CircuitBreaker in Phase 4 guarantees <5% overhead across all phases
- **Complexity risks**: Phased approach allows course correction
  - **Bridge Abstraction Complexity**: Start simple, ensure it works with Lua first
  - **API Injection Complexity**: Design language-agnostic APIs carefully
- **Integration risks**: Comprehensive testing at each phase
- **Architecture Risk**: CRITICAL - implement bridge pattern correctly in Phase 1.2 or face major refactoring in Phase 14
- **Architecture Risk**: Phase 4 hook system designed with future phases in mind, preventing Phase 3-style rework
- **Cross-Language Risk**: UniversalEvent and language adapters prepared in Phase 4
- **Distributed Risk**: DistributedHookContext ready for Phase 18-19 A2A protocol
- **Cost Risk**: Built-in cost tracking hooks prevent runaway AI/ML expenses

---

## Phase-Specific Implementation Strategy

### Documentation Approach: Individual Phase Guides

After analyzing implementation complexity and team collaboration needs, we recommend creating **focused, phase-specific implementation documents** rather than one monolithic design document.

### **Why Phase-Specific Documentation?**

**1. Implementation Reality Check**
- Real implementation reveals gaps, edge cases, and design issues not visible in theoretical planning
- Each phase teaches us something that should inform the next phase design
- Better to discover and fix architectural issues early than propagate them through all phases

**2. Focus and Cognitive Load Management**
- Developers working on Phase 1 don't need Phase 11 distractions
- The main architecture document is already 15,034+ lines - adding detailed implementation specs would make it unmanageable
- Focused documents are easier to review, approve, and implement effectively

**3. Learning and Iterative Improvement**
- Each phase becomes a learning laboratory for the next
- Can incorporate real performance data, developer feedback, and discovered edge cases
- Design evolution based on actual implementation experience vs theoretical planning

**4. Enhanced Team Collaboration**
- Different team members can own different phases
- Easier to get stakeholder sign-off on focused, implementable chunks
- Better for code reviews and technical discussions
- Parallel development where dependencies allow

### **Recommended Documentation Structure**

```
docs/
├── technical/
│   ├── master-architecture-vision.md     # High-level architectural reference (existing)
│   ├── implementation-phases.md              # Roadmap overview (this document)
│   └── phase-implementations/
│       ├── phase-0-foundation-guide.md       # Detailed implementation guide
│       ├── phase-1-core-runtime-guide.md     # Created after Phase 0 learnings
│       ├── phase-2-tools-library-guide.md    # Created after Phase 1 learnings
│       ├── phase-3-workflow-guide.md         # Created after Phase 2 learnings
│       └── ...                               # Future phases
```

### **Phase Implementation Document Template**

Each phase-specific document should include:

**1. Implementation Specifications**
- Detailed technical specifications with complete code examples
- Architecture decisions specific to this phase with rationale
- Performance targets and constraints with measurement criteria
- Security considerations and threat model for this phase

**2. Step-by-Step Implementation Guidance**
- Ordered implementation approach with dependencies
- Code patterns, best practices, and architectural guidelines
- Common pitfalls identified from previous phases and how to avoid them
- Testing strategies with specific acceptance criteria

**3. Integration and Transition Planning**
- How this phase integrates with previous phases
- What changes and prepares for the next phase
- Migration considerations and compatibility requirements
- Breaking changes documentation with migration guides
- Handoff specifications to next phase development team

**4. Post-Implementation Review** (added after completion)
- What worked well vs what didn't in implementation
- Performance insights and bottlenecks discovered
- Design decisions that should be reconsidered
- Concrete recommendations for next phase based on learnings

### **Implementation Workflow**

**Phase 0 Start:**
1. Create detailed `phase-0-foundation-guide.md` with complete implementation specs
2. Implement Phase 0 according to the guide
3. Conduct post-implementation review and lessons learned
4. Update Phase 0 guide with learnings and create Phase 1 guide

**Subsequent Phases:**
1. Use learnings from previous phase to create next phase guide
2. Incorporate performance data, edge cases, and design improvements
3. Implement phase according to updated understanding
4. Continue iterative improvement cycle

### **Advantages of This Approach**

- **Maintainable**: Smaller, focused documents are easier to maintain and update
- **Actionable**: Each document is immediately implementable with current knowledge
- **Evolutionary**: Design improves with each phase based on real experience
- **Collaborative**: Better for team review, approval, and parallel development
- **Realistic**: Based on actual implementation experience rather than theoretical planning
- **Quality**: Higher quality decisions based on accumulated learnings

### **Starting Point: Phase 0 Foundation Guide**

The first document to create should be `docs/technical/phase-implementations/phase-0-foundation-guide.md` with:

- Complete Cargo workspace structure and crate specifications
- Full trait definitions with method signatures and documentation
- Error handling implementation patterns and error type hierarchy
- Logging infrastructure setup with tracing integration
- CI/CD pipeline specifications with test automation
- Testing framework setup and initial test patterns

This Phase 0 guide will be comprehensive and detailed, serving as the template for subsequent phase guides while incorporating lessons learned from actual foundation implementation.

### **Evolution and Feedback Loop**

Each phase guide will be:
1. **Informed** by previous phase implementation experience
2. **Validated** through implementation and testing
3. **Refined** based on actual development challenges
4. **Enhanced** with performance and security learnings

This creates a continuous improvement loop where each phase builds on real knowledge rather than theoretical assumptions, resulting in higher quality implementation and more realistic timelines.

This implementation roadmap provides a clear path from initial foundation through production-ready deployment and additional enhancements, with specific success criteria and testing requirements for each phase, supported by focused, learnings-driven implementation guides.