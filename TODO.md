# Phase 2: Built-in Tools Library - TODO List

**Version**: 1.0  
**Date**: June 2025  
**Status**: READY TO START  
**Started**: Not yet started  
**Phase**: 2 (Built-in Tools Library)  
**Timeline**: Weeks 5-6 (10 working days)  
**Priority**: CRITICAL (Core Functionality)
**Arch-Document**: docs/technical/rs-llmspell-final-architecture.md
**Design-Document**: docs/in-progress/phase-02-design-doc.md

> **📢 UPDATE**: Phase 1 complete! Ready to implement built-in tools library with provider enhancements.

> **📋 Actionable Task List**: This document breaks down Phase 2 implementation into specific, measurable tasks with clear acceptance criteria.

---

## Overview

**Goal**: Implement comprehensive built-in tools library with ModelSpecifier provider enhancements rolled from Phase 1.

**Success Criteria Summary:**
- [ ] ModelSpecifier parses `provider/model` syntax correctly
- [ ] Base URL overrides work at agent creation time
- [ ] 12+ built-in tools fully implemented and tested
- [ ] Tool registry with discovery and validation
- [ ] Security sandboxing prevents unauthorized access
- [ ] All tools support streaming where applicable
- [ ] Agent-tool integration works seamlessly in scripts
- [ ] >90% test coverage across all tools
- [ ] Performance: <10ms tool initialization
- [ ] Complete documentation for all tools

**Progress Update (2025-06-27):**
- [x] Task 2.1.1: Implement ModelSpecifier 2025-06-27
- [x] Task 2.1.2: Update ProviderManager 2025-06-27
- [x] Task 2.1.3: Update Script APIs 2025-06-27
- [x] Task 2.2.1: Enhance Tool Trait 2025-06-27
- [x] Task 2.2.2: Implement Tool Registry 2025-06-27
- [x] Task 2.2.3: Security Sandbox Implementation 2025-06-27
- [ ] Task 2.3.1: WebSearchTool <date>
- [ ] Task 2.3.2: SemanticSearchTool <date>
- [ ] Task 2.3.3: CodeSearchTool <date>
- [ ] Task 2.4.1: JsonProcessorTool <date>
- [ ] Task 2.4.2: CsvAnalyzerTool <date>
- [ ] Task 2.4.3: HttpRequestTool <date>
- [ ] Task 2.4.4: GraphQLQueryTool <date>
- [ ] Task 2.5.1: FileOperationsTool <date>
- [ ] Task 2.5.2: ArchiveHandlerTool <date>
- [ ] Task 2.5.3: TemplateEngineTool <date>
- [ ] Task 2.5.4: DataValidationTool <date>
- [ ] Task 2.6.1: Script Integration Tests <date>
- [ ] Task 2.6.2: Security Validation <date>
- [ ] Task 2.6.3: Performance Optimization <date>
- [ ] Task 2.6.4: Documentation and Examples <date>
- [ ] Task 2.6.5: Phase 3 Handoff Package <date>

---

## Phase 2.1: Provider Enhancement (Days 1-2)

### Task 2.1.1: Implement ModelSpecifier
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Provider Team Lead
**Dependencies**: Phase 1 complete

**Description**: Create ModelSpecifier struct to parse provider/model syntax.

**Acceptance Criteria:**
- [x] ModelSpecifier struct with provider, model, base_url fields
- [x] `parse()` method handles "provider/model" and "model" formats
- [x] `parse_with_base_url()` method for base URL overrides
- [x] Handles nested paths like "openrouter/deepseek/model"
- [x] Comprehensive unit tests for all parsing scenarios

**Implementation Steps:**
1. Create `llmspell-providers/src/model_specifier.rs`
2. Implement ModelSpecifier struct with parsing logic
3. Handle edge cases (empty strings, multiple slashes)
4. Write unit tests for all format variations
5. Add property tests for parsing robustness
6. Document parsing rules and examples

**Definition of Done:**
- [x] All parsing formats work correctly
- [x] Tests cover >95% of code paths
- [x] No panics on malformed input
- [x] Documentation complete

### Task 2.1.2: Update ProviderManager
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Provider Team
**Dependencies**: Task 2.1.1

**Description**: Enhance ProviderManager to use ModelSpecifier for agent creation.

**Acceptance Criteria:**
- [x] `create_agent_from_spec()` method implemented
- [x] Provider resolution from model string works
- [x] Base URL overrides applied correctly
- [x] Fallback to default provider when none specified
- [x] Error handling for missing providers

**Implementation Steps:**
1. Update `llmspell-providers/src/manager.rs`
2. Add create_agent_from_spec method
3. Implement provider resolution logic
4. Handle base URL override precedence
5. Update existing agent creation methods
6. Write integration tests

**Definition of Done:**
- [x] All agent creation paths tested
- [x] Provider resolution works correctly
- [x] Base URL precedence documented
- [x] Backward compatibility maintained

### Task 2.1.3: Update Script APIs
**Priority**: HIGH  
**Estimated Time**: 3 hours  
**Assignee**: Bridge Team
**Dependencies**: Task 2.1.2

**Description**: Update Lua (and prepare for JS/Python) APIs to support new syntax.

**Acceptance Criteria:**
- [x] Lua Agent.create supports model string syntax
- [x] Base URL parameter works in agent config
- [x] Examples updated to show new syntax
- [x] Old syntax still works (backward compatible)
- [x] Error messages helpful for invalid syntax

**Implementation Steps:**
1. Update `llmspell-bridge/src/lua/api/agent.rs`
2. Modify agent creation to use ModelSpecifier
3. Add base_url to configuration parsing
4. Update Lua examples and tests
5. Prepare similar updates for JS bridge
6. Document migration path

**Definition of Done:**
- [x] Lua scripts can use new syntax
- [x] Integration tests pass
- [x] Examples demonstrate both syntaxes
- [x] Migration guide written

---

## Phase 2.2: Core Tool Infrastructure (Days 3-4)

### Task 2.2.1: Enhance Tool Trait
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Core Team Lead
**Dependencies**: Provider enhancement complete

**Description**: Add streaming and security methods to Tool trait.

**Acceptance Criteria:**
- [x] `stream_execute()` method with default implementation
- [x] `security_requirements()` method added
- [x] `resource_limits()` method added
- [x] Trait remains object-safe
- [x] Default implementations sensible

**Implementation Steps:**
1. Update `llmspell-core/src/traits/tool.rs`
2. Add streaming type definitions
3. Define SecurityRequirements struct
4. Define ResourceLimits struct
5. Update mock implementations
6. Verify trait object safety

**Definition of Done:**
- [x] Enhanced trait compiles
- [x] Default implementations work
- [x] Mocks updated
- [x] Documentation complete

### Task 2.2.2: Implement Tool Registry
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: Core Team
**Dependencies**: Task 2.2.1

**Description**: Create tool registry with discovery and validation.

**Acceptance Criteria:**
- [x] ToolRegistry struct with storage and metadata
- [x] Register method validates tools
- [x] Discovery by capability works
- [x] Category-based organization
- [x] Thread-safe for concurrent access

**Implementation Steps:**
1. Create `llmspell-tools/src/registry.rs`
2. Implement storage with Arc for sharing
3. Add validation during registration
4. Implement discovery methods
5. Add category management
6. Write comprehensive tests

**Definition of Done:**
- [x] Registry operations thread-safe
- [x] Discovery returns correct tools
- [x] Validation prevents bad tools
- [x] Performance benchmarked

### Task 2.2.3: Security Sandbox Implementation
**Priority**: CRITICAL  
**Estimated Time**: 6 hours  
**Assignee**: Security Team
**Dependencies**: Task 2.2.1

**Description**: Implement security sandboxing for tool execution.

**Acceptance Criteria:**
- [x] FileSandbox restricts file access
- [x] NetworkSandbox controls network access
- [x] ResourceMonitor tracks usage
- [x] Sandbox integrates with tool execution
- [x] Security violations logged

**Implementation Steps:**
1. Create `llmspell-security/src/sandbox/mod.rs`
2. Implement FileSandbox with path restrictions
3. Implement NetworkSandbox with domain allowlists
4. Create ResourceMonitor for CPU/memory limits
5. Integrate with tool execution flow
6. Write security tests

**Definition of Done:**
- [x] Sandbox prevents unauthorized access
- [x] Resource limits enforced
- [x] Security tests comprehensive
- [x] Performance overhead <5%

---

## Phase 2.3: Search Tools Implementation (Days 4-5)

### Task 2.3.1: WebSearchTool
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Tools Team
**Dependencies**: Core infrastructure complete

**Description**: Implement web search tool with multiple providers.

**Acceptance Criteria:**
- [ ] Supports multiple search providers (Google, Bing, DuckDuckGo)
- [ ] Rate limiting implemented
- [ ] Result formatting consistent
- [ ] Streaming results supported
- [ ] API key management secure

**Implementation Steps:**
1. Create `llmspell-tools/src/search/web_search.rs`
2. Implement provider abstraction
3. Add rate limiting logic
4. Create result formatters
5. Implement streaming support
6. Write tests with mocked APIs

**Definition of Done:**
- [ ] All providers work correctly
- [ ] Rate limits respected
- [ ] Tests use mocked responses
- [ ] Documentation includes examples

### Task 2.3.2: SemanticSearchTool
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Tools Team
**Dependencies**: Core infrastructure complete

**Description**: Implement semantic search over vector stores.

**Acceptance Criteria:**
- [ ] Embedding model abstraction works
- [ ] Multiple vector stores supported
- [ ] Similarity search accurate
- [ ] Metadata filtering supported
- [ ] Performance optimized

**Implementation Steps:**
1. Create `llmspell-tools/src/search/semantic_search.rs`
2. Define EmbeddingModel trait
3. Define VectorStore trait
4. Implement similarity search
5. Add metadata filtering
6. Optimize with caching

**Definition of Done:**
- [ ] Search results relevant
- [ ] Multiple backends tested
- [ ] Performance benchmarked
- [ ] Examples provided

### Task 2.3.3: CodeSearchTool
**Priority**: MEDIUM  
**Estimated Time**: 4 hours  
**Assignee**: Tools Team
**Dependencies**: Core infrastructure complete

**Description**: Implement code search with language awareness.

**Acceptance Criteria:**
- [ ] Supports major languages (Rust, Python, JS, etc.)
- [ ] Symbol search works
- [ ] Full-text search accurate
- [ ] Git integration optional
- [ ] Results include context

**Implementation Steps:**
1. Create `llmspell-tools/src/search/code_search.rs`
2. Integrate tree-sitter parsers
3. Build search index
4. Implement symbol extraction
5. Add context extraction
6. Test with real codebases

**Definition of Done:**
- [ ] Multiple languages supported
- [ ] Search accuracy validated
- [ ] Performance acceptable
- [ ] Git integration tested

---

## Phase 2.4: Data Processing Tools (Days 6-7)

### Task 2.4.1: JsonProcessorTool
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Data Team
**Dependencies**: Core infrastructure complete

**Description**: JSON processing with jq-like syntax.

**Acceptance Criteria:**
- [ ] jq syntax support comprehensive
- [ ] Schema validation works
- [ ] Streaming large files supported
- [ ] Error messages helpful
- [ ] Common operations optimized

**Implementation Steps:**
1. Create `llmspell-tools/src/data/json_processor.rs`
2. Integrate jq engine
3. Add schema validation
4. Implement streaming parser
5. Create helpful error formatting
6. Benchmark performance

**Definition of Done:**
- [ ] jq compatibility high
- [ ] Large files handled
- [ ] Validation accurate
- [ ] Examples comprehensive

### Task 2.4.2: CsvAnalyzerTool
**Priority**: MEDIUM  
**Estimated Time**: 3 hours  
**Assignee**: Data Team
**Dependencies**: Core infrastructure complete

**Description**: CSV analysis and processing tool.

**Acceptance Criteria:**
- [ ] Encoding detection automatic
- [ ] Statistical analysis provided
- [ ] Data type inference works
- [ ] Large files streamed
- [ ] Export formats supported

**Implementation Steps:**
1. Create `llmspell-tools/src/data/csv_analyzer.rs`
2. Add encoding detection
3. Implement statistical functions
4. Add type inference logic
5. Create streaming processor
6. Test with various CSV formats

**Definition of Done:**
- [ ] Handles malformed CSV
- [ ] Statistics accurate
- [ ] Memory efficient
- [ ] Documentation complete

### Task 2.4.3: HttpRequestTool
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: API Team
**Dependencies**: Core infrastructure complete

**Description**: HTTP client with advanced features.

**Acceptance Criteria:**
- [ ] All HTTP methods supported
- [ ] Authentication methods comprehensive
- [ ] Retry logic configurable
- [ ] Rate limiting built-in
- [ ] Response parsing automatic

**Implementation Steps:**
1. Create `llmspell-tools/src/api/http_request.rs`
2. Wrap reqwest client
3. Add authentication handlers
4. Implement retry logic
5. Add rate limiting
6. Create response parsers

**Definition of Done:**
- [ ] All HTTP verbs work
- [ ] Auth methods tested
- [ ] Retries configurable
- [ ] Rate limits respected

### Task 2.4.4: GraphQLQueryTool
**Priority**: MEDIUM  
**Estimated Time**: 5 hours  
**Assignee**: API Team
**Dependencies**: Core infrastructure complete

**Description**: GraphQL client with schema introspection.

**Acceptance Criteria:**
- [ ] Query execution works
- [ ] Mutation support complete
- [ ] Schema introspection cached
- [ ] Variable substitution safe
- [ ] Error handling comprehensive

**Implementation Steps:**
1. Create `llmspell-tools/src/api/graphql_query.rs`
2. Implement GraphQL client
3. Add schema introspection
4. Create query builder
5. Add variable handling
6. Test with public APIs

**Definition of Done:**
- [ ] Queries execute correctly
- [ ] Schema caching works
- [ ] Variables handled safely
- [ ] Examples provided

---

## Phase 2.5: File System and Utility Tools (Day 8)

### Task 2.5.1: FileOperationsTool
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: System Team
**Dependencies**: Security sandbox complete

**Description**: Safe file system operations tool.

**Acceptance Criteria:**
- [ ] Read/write operations sandboxed
- [ ] Path traversal prevented
- [ ] Permissions checked
- [ ] Atomic operations supported
- [ ] Directory operations safe

**Implementation Steps:**
1. Create `llmspell-tools/src/fs/file_operations.rs`
2. Integrate with FileSandbox
3. Add path validation
4. Implement atomic writes
5. Add directory operations
6. Test security boundaries

**Definition of Done:**
- [ ] Security tests pass
- [ ] Operations atomic
- [ ] Sandbox effective
- [ ] Documentation clear

### Task 2.5.2: ArchiveHandlerTool
**Priority**: MEDIUM  
**Estimated Time**: 3 hours  
**Assignee**: System Team
**Dependencies**: FileOperationsTool

**Description**: Archive extraction and creation tool.

**Acceptance Criteria:**
- [ ] ZIP/TAR/GZ formats supported
- [ ] Extraction limits enforced
- [ ] Compression levels configurable
- [ ] Path safety maintained
- [ ] Streaming supported

**Implementation Steps:**
1. Create `llmspell-tools/src/fs/archive_handler.rs`
2. Add format detection
3. Implement extraction limits
4. Add compression support
5. Ensure path safety
6. Test with various archives

**Definition of Done:**
- [ ] Formats handled correctly
- [ ] Limits prevent bombs
- [ ] Paths sanitized
- [ ] Performance good

### Task 2.5.3: TemplateEngineTool
**Priority**: LOW  
**Estimated Time**: 3 hours  
**Assignee**: Utility Team
**Dependencies**: Core infrastructure complete

**Description**: Template rendering with multiple engines.

**Acceptance Criteria:**
- [ ] Multiple template syntaxes supported
- [ ] Custom filters work
- [ ] Context injection safe
- [ ] Error messages helpful
- [ ] Performance optimized

**Implementation Steps:**
1. Create `llmspell-tools/src/util/template_engine.rs`
2. Add template engine abstraction
3. Implement safety checks
4. Add custom filter support
5. Create error formatter
6. Benchmark rendering

**Definition of Done:**
- [ ] Templates render correctly
- [ ] Injection prevented
- [ ] Filters extensible
- [ ] Examples clear

### Task 2.5.4: DataValidationTool
**Priority**: LOW  
**Estimated Time**: 2 hours  
**Assignee**: Utility Team
**Dependencies**: Core infrastructure complete

**Description**: Data validation with custom rules.

**Acceptance Criteria:**
- [ ] Multiple validation types supported
- [ ] Custom rules definable
- [ ] Error reporting detailed
- [ ] Performance acceptable
- [ ] Schema validation included

**Implementation Steps:**
1. Create `llmspell-tools/src/util/data_validation.rs`
2. Define Validator trait
3. Implement common validators
4. Add rule composition
5. Create error reports
6. Test with various data

**Definition of Done:**
- [ ] Validators comprehensive
- [ ] Rules composable
- [ ] Errors helpful
- [ ] Performance measured

---

## Phase 2.6: Integration and Testing (Days 9-10)

### Task 2.6.1: Script Integration Tests
**Priority**: CRITICAL  
**Estimated Time**: 5 hours  
**Assignee**: Full Team
**Dependencies**: All tools implemented

**Description**: Comprehensive integration testing with scripts.

**Acceptance Criteria:**
- [ ] All tools callable from Lua
- [ ] Provider enhancement works in scripts
- [ ] Tool chaining tested
- [ ] Error propagation correct
- [ ] Performance acceptable

**Implementation Steps:**
1. Create integration test suite
2. Write Lua scripts using all tools
3. Test tool combinations
4. Verify error handling
5. Benchmark script execution
6. Document patterns

**Definition of Done:**
- [ ] All tools tested from scripts
- [ ] Common patterns documented
- [ ] Performance benchmarked
- [ ] No integration issues

### Task 2.6.2: Security Validation
**Priority**: CRITICAL  
**Estimated Time**: 4 hours  
**Assignee**: Security Team
**Dependencies**: All tools implemented

**Description**: Security audit of all tools.

**Acceptance Criteria:**
- [ ] Sandbox escape attempts fail
- [ ] Resource limits enforced
- [ ] Path traversal prevented
- [ ] Injection attacks blocked
- [ ] Audit trail complete

**Implementation Steps:**
1. Create security test suite
2. Attempt sandbox escapes
3. Test resource exhaustion
4. Try path traversal attacks
5. Test injection scenarios
6. Document findings

**Definition of Done:**
- [ ] No security vulnerabilities
- [ ] All attacks prevented
- [ ] Audit documented
- [ ] Fixes implemented

### Task 2.6.3: Performance Optimization
**Priority**: HIGH  
**Estimated Time**: 4 hours  
**Assignee**: Performance Team
**Dependencies**: All tools implemented

**Description**: Optimize tool performance.

**Acceptance Criteria:**
- [ ] Tool init <10ms achieved
- [ ] Memory usage minimized
- [ ] Caching implemented
- [ ] Bottlenecks identified
- [ ] Benchmarks automated

**Implementation Steps:**
1. Profile all tools
2. Identify bottlenecks
3. Implement caching
4. Optimize hot paths
5. Reduce allocations
6. Automate benchmarks

**Definition of Done:**
- [ ] Performance targets met
- [ ] Benchmarks in CI
- [ ] Optimizations documented
- [ ] No regressions

### Task 2.6.4: Documentation and Examples
**Priority**: HIGH  
**Estimated Time**: 5 hours  
**Assignee**: Full Team
**Dependencies**: All implementations complete

**Description**: Create comprehensive documentation.

**Acceptance Criteria:**
- [ ] Every tool documented
- [ ] Usage examples provided
- [ ] Best practices guide written
- [ ] API reference complete
- [ ] Migration guide created

**Implementation Steps:**
1. Document each tool's API
2. Create usage examples
3. Write best practices guide
4. Generate API reference
5. Create migration guide
6. Review and polish

**Definition of Done:**
- [ ] Documentation complete
- [ ] Examples runnable
- [ ] Guides helpful
- [ ] API reference accurate

### Task 2.6.5: Phase 3 Handoff Package
**Priority**: CRITICAL  
**Estimated Time**: 3 hours  
**Assignee**: Team Lead
**Dependencies**: All tasks complete

**Description**: Prepare handoff to Phase 3.

**Acceptance Criteria:**
- [ ] Feature summary complete
- [ ] Known issues documented
- [ ] Performance data provided
- [ ] Architecture updates noted
- [ ] Phase 3 prep included

**Implementation Steps:**
1. Summarize deliverables
2. Document known issues
3. Gather performance data
4. Note architecture changes
5. Prepare Phase 3 overview
6. Schedule handoff meeting

**Definition of Done:**
- [ ] Handoff package complete
- [ ] Phase 3 team briefed
- [ ] No blocking issues
- [ ] Clean transition

---

## Summary Dashboard

### Critical Path
1. **Days 1-2**: Provider enhancement (ModelSpecifier)
2. **Days 3-4**: Core infrastructure (Registry, Sandbox)
3. **Days 4-5**: Search tools (Web, Semantic, Code)
4. **Days 6-7**: Data/API tools (JSON, HTTP, GraphQL)
5. **Day 8**: File/Utility tools
6. **Days 9-10**: Integration, Security, Documentation

### Resource Allocation
- **Provider Team**: ModelSpecifier and ProviderManager updates
- **Core Team**: Tool trait, Registry implementation
- **Security Team**: Sandbox implementation and validation
- **Tools Team**: All 12+ tool implementations
- **All**: Integration testing and documentation

### Risk Areas
1. **External API Dependencies**: Use mocks extensively
2. **Security Vulnerabilities**: Continuous security testing
3. **Performance Targets**: Early profiling and optimization
4. **Tool Complexity**: Start with simpler tools
5. **Schedule**: 10 days ambitious for 12+ tools

### Success Metrics
- [ ] ModelSpecifier parsing works for all formats
- [ ] 12+ tools fully implemented and tested
- [ ] >90% test coverage achieved
- [ ] <10ms tool initialization verified
- [ ] Security sandbox prevents all escapes
- [ ] Documentation comprehensive
- [ ] Performance benchmarks in CI
- [ ] Phase 3 handoff ready