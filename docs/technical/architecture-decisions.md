# Architecture Decision Records (ADRs)

**Version**: 0.11.0
**Last Updated**: October 2025
**Validation**: Cross-referenced with phase design documents (phase-01 through phase-11)

> **📋 Decision Log**: Consolidated record of all significant architecture decisions made throughout LLMSpell development, showing how decisions evolved and sometimes reversed across phases.

---

## Table of Contents

1. [Phase 0-1: Foundation Decisions](#phase-0-1-foundation-decisions)
2. [Phase 2: Tool & Provider Decisions](#phase-2-tool--provider-decisions)
3. [Phase 3: Infrastructure Decisions](#phase-3-infrastructure-decisions)
4. [Phase 4: Hook & Event Decisions](#phase-4-hook--event-decisions)
5. [Phase 5: State Management Decisions](#phase-5-state-management-decisions)
6. [Phase 6: Session Management Decisions](#phase-6-session-management-decisions)
7. [Phase 7: API Standardization Decisions](#phase-7-api-standardization-decisions)
8. [Phase 8: RAG System Decisions](#phase-8-rag-system-decisions)
9. [Phase 11: API Refinement Decisions](#phase-11-api-refinement-decisions)
10. [Cross-Cutting Decisions](#cross-cutting-decisions)
11. [Decision Evolution & Reversals](#decision-evolution--reversals)

---

## Phase 0-1: Foundation Decisions

### ADR-001: BaseAgent as Universal Foundation

**Date**: June 2025 (Phase 1)  
**Status**: Accepted & Validated  
**Context**: Need unified interface for all components  
**Decision**: All components (agents, tools, workflows) implement `BaseAgent` trait  
**Implementation**:
```rust
pub trait BaseAgent: Send + Sync {
    fn metadata(&self) -> &ComponentMetadata;
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput>;
}
```
**Consequences**: 
- ✅ Uniform execution model across all components
- ✅ Enabled composable architecture
- ✅ Single error handling path
- ❌ Some overhead for simple tools (accepted tradeoff)

### ADR-002: llmspell-utils Crate Creation

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Phase 1 recognized need for shared utilities to avoid duplication  
**Decision**: Create `llmspell-utils` crate before implementing features  
**Rationale**: DRY principle, reduce code duplication across crates  
**Consequences**:
- ✅ Consistent utility functions across all crates
- ✅ Single source of truth for common operations
- ✅ Easier maintenance and updates
- ❌ Additional crate dependency (minimal impact)

### ADR-003: Async-First Implementation

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: LLM calls and I/O are inherently async  
**Decision**: All core traits use `async_trait`, tokio runtime  
**Consequences**:
- ✅ Non-blocking I/O throughout system
- ✅ Better resource utilization
- ❌ Complexity bridging to sync Lua (solved by ADR-004)

### ADR-004: Synchronous Script Bridge

**Date**: July 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Lua is synchronous, Rust core is async  
**Decision**: Use `block_on()` to bridge async Rust to sync scripts  
**Implementation**:
```rust
pub fn execute_sync(input: LuaValue) -> LuaResult<LuaValue> {
    let handle = tokio::runtime::Handle::current();
    handle.block_on(async { execute_async(input).await })
}
```
**Consequences**:
- ✅ Scripts remain simple without async complexity
- ✅ No need for Lua coroutine complexity
- ❌ Potential for deadlocks (mitigated with careful design)

---

## Phase 2: Tool & Provider Decisions

### ADR-005: Global Injection Over Require

**Date**: July 2025 (Phase 2)  
**Status**: Accepted  
**Context**: Lua's require() is complex and error-prone  
**Decision**: Pre-inject all 15 globals (Agent, Tool, Workflow, etc.)  
**Performance**: 2-4ms injection time  
**Consequences**:
- ✅ Zero-import scripts
- ✅ Better performance than dynamic loading
- ✅ Simpler user experience
- ❌ Larger initial memory footprint (acceptable)

### ADR-006: Provider/Model Syntax

**Date**: July 2025 (Phase 2)  
**Status**: Accepted  
**Context**: Need intuitive model specification  
**Decision**: Support "provider/model" syntax (e.g., "openai/gpt-4")  
**Implementation**: ModelSpecifier parser in providers crate  
**Consequences**:
- ✅ Intuitive API for users
- ✅ Consistent with industry patterns
- ✅ Supports base URL overrides

### ADR-007: Self-Contained Tools First

**Date**: July 2025 (Phase 2)  
**Status**: Accepted  
**Context**: External dependencies add complexity  
**Decision**: Phase 2 implements only self-contained tools (26 tools)  
**Rationale**: Prove system works before adding external APIs  
**Consequences**:
- ✅ Faster initial development
- ✅ No external API dependencies
- ✅ Easier testing
- ❌ Limited functionality (addressed in Phase 3)

---

## Phase 3: Infrastructure Decisions

### ADR-008: Clean Break Strategy

**Date**: July 2025 (Phase 3)  
**Status**: Accepted  
**Context**: Tool parameter inconsistency discovered  
**Decision**: Direct upgrade without migration tools (pre-1.0 freedom)  
**Rationale**: Save 1 week of development time for better features  
**Consequences**:
- ✅ Faster development velocity
- ✅ Cleaner codebase without legacy support
- ✅ Better final architecture
- ❌ Breaking changes (acceptable pre-1.0)

### ADR-009: Tool Parameter Standardization

**Date**: July 2025 (Phase 3)  
**Status**: Accepted  
**Context**: 26 tools had inconsistent parameter names  
**Decision**: Standardize on `input`, `path`, `operation`  
**Changes Applied**:
- `text`, `content`, `data` → `input`
- `file_path`, `archive_path` → `path`
- All multi-function tools use `operation`
**Consequences**:
- ✅ 95% parameter consistency (from 60%)
- ✅ Predictable API
- ❌ Breaking changes (accepted via ADR-008)

### ADR-010: Factory Pattern for Components

**Date**: July 2025 (Phase 3.3)  
**Status**: Accepted  
**Context**: Complex component creation logic  
**Decision**: Implement factory pattern with builders  
**Implementation**: AgentFactory, WorkflowFactory  
**Consequences**:
- ✅ Type-safe creation
- ✅ Validation at build time
- ✅ Clear component types

---

## Phase 4: Hook & Event Decisions

### ADR-011: Event-Driven Hook System

**Date**: July 2025 (Phase 4)  
**Status**: Accepted  
**Context**: Overlap between hooks and events  
**Decision**: Unify into event-driven hook system  
**Performance**: <5% overhead with circuit breakers  
**Consequences**:
- ✅ Single system instead of two
- ✅ Automatic performance protection
- ✅ Event correlation built-in

### ADR-012: Cross-Language Hook Support

**Date**: July 2025 (Phase 4)  
**Status**: Accepted  
**Context**: Future support for JS/Python planned  
**Decision**: Add HookAdapter trait for language abstraction  
**Implementation**: Adapters for Lua, stubs for JS/Python  
**Consequences**:
- ✅ Future-proof design
- ✅ Consistent hook behavior across languages
- ❌ Additional abstraction layer

### ADR-013: Circuit Breaker for Hooks

**Date**: July 2025 (Phase 4)  
**Status**: Accepted  
**Context**: Bad hooks shouldn't break system  
**Decision**: Add automatic circuit breaker (5 failures → open)  
**Thresholds**: 5 failures, 60 second reset  
**Consequences**:
- ✅ System resilience
- ✅ Automatic recovery
- ❌ Hooks might not execute (acceptable for stability)

---

## Phase 5: State Management Decisions

### ADR-014: Multi-Backend State Persistence

**Date**: July 2025 (Phase 5)  
**Status**: Accepted  
**Context**: Different deployment scenarios need different storage  
**Decision**: Support Memory, Sled, RocksDB backends  
**Implementation**: StorageBackend trait abstraction  
**Consequences**:
- ✅ Flexible deployment options
- ✅ Zero external dependencies with Sled
- ✅ High performance with RocksDB
- ❌ More complexity in abstraction

### ADR-015: State Scoping Hierarchy

**Date**: July 2025 (Phase 5)  
**Status**: Accepted  
**Context**: Components need isolated state with controlled sharing  
**Decision**: 4-level scope hierarchy (Global, Session, Workflow, Component)  
**Implementation**: StateScope enum with resolution logic  
**Consequences**:
- ✅ Clear isolation boundaries
- ✅ Prevents state pollution
- ✅ Enables parallel execution
- ❌ More complex state resolution

### ADR-016: Migration Speed Over Safety

**Date**: July 2025 (Phase 5)  
**Status**: Accepted  
**Context**: State migrations need to be fast  
**Decision**: Optimize for speed (483K items/sec)  
**Tradeoffs**: Batch operations, minimal validation  
**Consequences**:
- ✅ Fast migrations (2.07μs/item)
- ✅ Minimal downtime
- ❌ Less validation (requires backups)

### ADR-017: Deferred Custom Transformers

**Date**: July 2025 (Phase 5)  
**Status**: Deferred  
**Context**: Complex field transformations discovered during implementation  
**Decision**: Implement basic Copy/Default/Remove, defer custom logic  
**Rationale**: Scope management, time constraints  
**Consequences**:
- ✅ Phase 5 completed on schedule
- ✅ Basic migrations work
- ❌ 4 integration tests marked #[ignore]

---

## Phase 6: Session Management Decisions

### ADR-018: Blake3 for Content Addressing

**Date**: August 2025 (Phase 6)  
**Status**: Accepted  
**Context**: Need fast content hashing for artifacts  
**Decision**: Use blake3 instead of SHA256  
**Performance**: 10x faster than SHA256  
**Consequences**:
- ✅ Faster artifact storage
- ✅ Lower CPU usage
- ✅ Maintains cryptographic security

### ADR-019: LZ4 for Artifact Compression

**Date**: August 2025 (Phase 6)  
**Status**: Accepted  
**Context**: Large artifacts need compression  
**Decision**: Use lz4_flex (pure Rust) for >10KB artifacts  
**Rationale**: Fast compression, pure Rust implementation  
**Consequences**:
- ✅ Reduced storage size
- ✅ Fast compression/decompression
- ✅ No C dependencies

### ADR-020: ReplayableHook Integration

**Date**: August 2025 (Phase 6)  
**Status**: Accepted  
**Context**: Session replay needs hook history  
**Decision**: Leverage Phase 4's ReplayableHook trait  
**Implementation**: Session-specific hook adapters  
**Consequences**:
- ✅ Reuses existing infrastructure
- ✅ Consistent replay behavior
- ✅ No duplicate implementation

---

## Phase 7: API Standardization Decisions

### ADR-021: Service → Manager Naming

**Date**: August 2025 (Phase 7)  
**Status**: Accepted  
**Context**: Inconsistent naming (Service, Manager, Registry)  
**Decision**: Standardize on "Manager" suffix  
**Changes**:
- ProviderService → ProviderManager
- StateService → StateManager
- HookService → HookManager
**Consequences**:
- ✅ Consistent naming
- ✅ Clearer purpose
- ❌ Breaking API change

### ADR-022: Universal Builder Pattern

**Date**: August 2025 (Phase 7)  
**Status**: Accepted  
**Context**: Inconsistent object creation patterns  
**Decision**: All complex objects use builder pattern  
**Example**:
```rust
let agent = AgentBuilder::new()
    .name("assistant")
    .model("gpt-4")
    .build()?;
```
**Consequences**:
- ✅ Consistent API
- ✅ Compile-time validation
- ❌ More verbose for simple cases

### ADR-023: retrieve() → get() Standardization

**Date**: August 2025 (Phase 7)  
**Status**: Accepted  
**Context**: Mix of retrieve(), get(), fetch() methods  
**Decision**: Standardize on get() for reads  
**Mapping**: retrieve/fetch/load/query → get/list  
**Consequences**:
- ✅ Predictable API
- ✅ Follows Rust conventions
- ❌ Breaking change

### ADR-024: Feature-Based Testing

**Date**: August 2025 (Phase 7)  
**Status**: Accepted  
**Context**: cfg_attr test categorization broke compilation  
**Decision**: Use Cargo features for test categories  
**Implementation**: llmspell-testing crate with feature flags  
**Consequences**:
- ✅ Working test categorization
- ✅ Selective test execution
- ✅ CI/CD optimization

---

## Phase 8: RAG System Decisions

### ADR-025: HNSW-Based Vector Storage

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: Need high-performance vector search for RAG at scale  
**Decision**: Use HNSW algorithm via hnsw_rs crate (not hnswlib-rs)  
**Rationale**: 
- Pure Rust implementation (no C++ dependencies)
- Sub-10ms search on 100K+ vectors
- Configurable trade-offs (speed vs accuracy)
**Implementation**: llmspell-storage/backends/vector/hnsw.rs  
**Performance**: 8ms search for 100K vectors, 450MB memory  
**Consequences**:
- ✅ Excellent search performance
- ✅ Predictable memory usage
- ✅ No external dependencies
- ❌ Higher memory than inverted index (acceptable)

### ADR-026: Namespace Multi-Tenancy Pattern

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: RAG system needs secure multi-tenant isolation  
**Decision**: Use `StateScope::Custom("tenant:id")` pattern for isolation  
**Implementation**: Namespace prefixes in vector storage  
**Overhead**: 3% performance impact  
**Consequences**:
- ✅ Complete tenant isolation
- ✅ No cross-tenant data leakage
- ✅ Reuses existing StateScope infrastructure
- ✅ Minimal performance impact

### ADR-027: Separate Storage Crate

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: Vector storage is distinct from state persistence  
**Decision**: Create llmspell-storage crate for vector operations  
**Rationale**: 
- Clear separation of concerns
- Different performance characteristics
- Independent scaling and optimization
**Consequences**:
- ✅ Clean architecture boundaries
- ✅ Specialized optimizations possible
- ✅ Easier to swap implementations
- ❌ Additional crate to maintain

### ADR-028: Multi-Tenant First Design

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: Enterprise RAG requires tenant isolation by default  
**Decision**: All RAG operations tenant-aware from the start  
**Implementation**: TenantManager in llmspell-tenancy  
**Features**:
- Usage tracking per tenant
- Cost calculation and limits
- Resource quotas
- Audit logging
**Consequences**:
- ✅ Enterprise-ready from day one
- ✅ Built-in compliance features
- ❌ Slight complexity for single-tenant use

### ADR-029: Simplified Two-Parameter Lua API

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: Complex RAG operations need simple script interface  
**Decision**: All RAG functions take (primary, options) pattern  
**Example**: `RAG.search(query, {k=10, tenant_id="acme"})`  
**Rationale**: Consistent with existing Tool.execute pattern  
**Consequences**:
- ✅ Intuitive API for users
- ✅ Consistent across all operations
- ✅ Optional parameters via table
- ✅ Future extensibility

### ADR-030: Configuration-Driven RAG

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: RAG features should work without compile flags  
**Decision**: Use runtime configuration for all RAG features  
**Implementation**: TOML configuration with sensible defaults  
**Example**:
```toml
[rag]
enabled = true
embedding_provider = "openai"
[rag.vector_storage.hnsw]
m = 16
ef_construction = 200
```
**Consequences**:
- ✅ No recompilation for changes
- ✅ Easy A/B testing
- ✅ Production tuning without rebuilds
- ❌ Slightly larger binary (all features included)

### ADR-031: OpenAI-Only Embeddings (Phase 8)

**Date**: December 2024 (Phase 8)  
**Status**: Accepted (Temporary)  
**Context**: Limited time, need production-quality embeddings  
**Decision**: Implement only OpenAI text-embedding-3-small  
**Rationale**: 
- Most widely used in production
- Well-documented API
- Good price/performance (384 dimensions)
**Future**: Add local models in Phase 9+  
**Consequences**:
- ✅ Faster Phase 8 delivery
- ✅ Production-ready embeddings
- ❌ Vendor lock-in (temporary)
- ❌ Requires API keys

### ADR-032: Session-Scoped RAG

**Date**: December 2024 (Phase 8)  
**Status**: Accepted  
**Context**: RAG data often session-specific  
**Decision**: Support session-scoped vectors with TTL  
**Implementation**: Integration with llmspell-sessions  
**Features**:
- Automatic cleanup on session end
- TTL-based expiration
- Session artifact integration
**Consequences**:
- ✅ Natural session workflows
- ✅ Automatic resource cleanup
- ✅ Prevents data accumulation
- ❌ Additional scope complexity

---

## Phase 11: API Refinement Decisions

### ADR-042: Unified execute() Method Naming

**Date**: October 2025 (Phase 11a.11)
**Status**: Accepted
**Context**: API method naming was inconsistent between Rust core traits and script language bindings
**Problem**:
- Rust `BaseAgent` trait: `execute()` only
- Rust `Tool` trait: inherits `execute()` only
- Rust `Workflow` trait: `execute()` only
- Lua Tool binding: `invoke()` only (inconsistent)
- Lua Agent binding: both `invoke()` and `execute()` (confusing)
- Documentation: mixed references to both methods

**Decision**: Standardize all component execution methods on `execute()` naming across all language bindings

**Rationale**:
1. **Consistency with Core**: Rust core traits universally use `execute()`
2. **Semantic Clarity**: "Execute a component" is clearer than "invoke a component"
3. **Future-Proof**: Ensures Python/JavaScript bindings follow same pattern
4. **Cognitive Load**: Single method name reduces mental overhead
5. **Documentation**: Easier to document and teach with uniform API

**Implementation** (Phase 11a.11):
- Lua: `Tool.invoke()` → `Tool.execute()`
- Lua: Removed `agent:invoke()` (kept `agent:execute()` only)
- Lua: Updated 20 example files (66 replacements total)
- JavaScript: Updated stub comments to reference `execute()`
- Documentation: 7 user guide files + 1 technical doc updated

**Breaking Changes**:
- `Tool.invoke(name, params)` → `Tool.execute(name, params)`
- `agent:invoke(input)` removed (use `agent:execute(input)`)

**Migration Path**:
- No deprecation period (pre-1.0, breaking changes acceptable per project policy)
- All examples updated atomically to prevent confusion

**Performance Impact**: None (API rename only, implementation unchanged)

**Consequences**:
- ✅ Consistent API across all components (Tool, Agent, Workflow)
- ✅ Matches Rust core trait naming conventions
- ✅ Clearer mental model: "execute a component instance"
- ✅ Future-proof for Python/JavaScript bindings (Phase 12+)
- ✅ Reduced documentation burden (single method to document)
- ✅ Easier for new users to learn
- ❌ Breaking change for existing Lua scripts (acceptable pre-1.0)
- ❌ Lost semantic distinction between registry-based vs instance-based calls (accepted trade-off)

**Related ADRs**:
- ADR-023: retrieve() → get() Standardization (similar API naming standardization in Phase 7)
- ADR-001: BaseAgent as Universal Foundation (defines core execute() method)

**Validation**:
- 66 method call updates across 20 Lua example files
- All workspace tests passing (1,832+ tests)
- Zero clippy warnings
- Examples validated across beginner/intermediate/advanced levels

---

### ADR-043: Removal of Custom Workflow Steps

**Date**: October 2025 (Phase 11a.12)
**Status**: Accepted
**Context**: Custom workflow steps (StepType::Custom) existed in codebase but were incomplete

**Problem**:
1. **Mock Implementation**: execute_custom_step() only returned hardcoded strings
2. **No Real Functionality**: 15 hardcoded function names, no user extension mechanism
3. **Documentation Lies**: Rust docs showed CustomStep trait that didn't exist
4. **API Confusion**: Exposed via Lua API but didn't work as expected
5. **Architectural Obsolescence**: Phase 3 replaced all custom functions with tools/agents

**Decision**: Remove StepType::Custom variant entirely, educate users on tool/agent/workflow patterns

**Rationale**:
1. **Tools Provide Superiority**: Tools are reusable, testable, discoverable, documented
2. **Agents Handle Reasoning**: Complex logic better suited to LLM-based agents
3. **Workflows Enable Composition**: Conditional/loop/nested workflows cover orchestration
4. **Zero Real Functionality Lost**: Custom steps were 100% mock implementation
5. **Code Quality**: Removes 200+ lines of dead/misleading code
6. **User Clarity**: Eliminates confusion about unimplemented features

**Implementation** (Phase 11a.12):
- Removed StepType::Custom variant from traits.rs (7 lines)
- Removed execute_custom_step() mock method (72 lines)
- Removed all Custom match arms from step_executor.rs (~70 lines)
- Removed custom step parsing from Lua bindings (18 lines)
- Updated 9 test files to use Tool/Agent steps
- Fixed Rust API documentation (llmspell-workflows.md)
- Added 240-line migration guide to Lua API docs
- Fixed misleading example comment in 03-first-workflow.lua

**Breaking Changes**:
- `StepType::Custom { function_name, parameters }` removed
- Lua API: `{ type = "custom", function = "...", parameters = {...} }` removed
- **Impact**: ZERO - Feature was never functional

**Migration Path**:
- Custom transformations → Create tools with Tool.register()
- Custom reasoning → Create agents with Agent.create()
- Custom branching → Use Workflow.conditional()
- Custom iteration → Use Workflow.loop()
- Custom composition → Use nested workflows

**Alternatives Considered**:
1. **Implement CustomStep trait** - Would duplicate tool/agent functionality (rejected)
2. **Document as unimplemented** - Keeps dead code, doesn't address root cause (rejected)
3. **Deprecation period** - Unnecessary since feature never worked (rejected)

**Consequences**:
- ✅ Cleaner codebase (-200 lines dead code)
- ✅ No user confusion about unimplemented features
- ✅ Aligns with Phase 3 architectural decision
- ✅ Documentation accuracy restored
- ✅ Users learn superior patterns (tools/agents/workflows)
- ✅ Future maintainability improved
- ✅ Zero breaking changes (feature never worked)

**Performance Impact**: None (mock execution was already negligible)

**Related ADRs**:
- ADR-001: BaseAgent foundation (agents as primary reasoning primitive)
- ADR-004: Synchronous Script Bridge (tools/agents bridge to Lua)
- ADR-042: Unified execute() naming (consistent API across components)

**Validation**:
- 71 workflow tests pass (including tracing tests migrated to Tool steps)
- 0 clippy warnings in llmspell-workflows
- Migration guide demonstrates 6 patterns
- All examples execute successfully

---

## Cross-Cutting Decisions

### ADR-033: Three-Level Security Model

**Date**: July 2025 (Phase 3)  
**Status**: Accepted  
**Context**: Need granular security without complexity  
**Decision**: Three levels: Safe, Restricted, Privileged  
**Implementation**: SecurityLevel enum with enforcement  
**Consequences**:
- ✅ Simple to understand
- ✅ Easy to audit
- ✅ Progressive trust model

### ADR-034: mlua Over Other Lua Bindings

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Need robust Lua integration  
**Decision**: Use mlua with Lua 5.4  
**Rationale**: Most mature binding, async support  
**Consequences**:
- ✅ Stable integration
- ✅ Good documentation
- ❌ No LuaJIT (performance tradeoff)

### ADR-035: rig-core for LLM Providers

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Need LLM provider integration  
**Decision**: Use rig-core instead of custom implementation  
**Consequences**:
- ✅ Fast implementation
- ✅ Multiple providers supported
- ❌ External dependency
- ❌ Less control over provider details

### ADR-036: No JavaScript Implementation

**Date**: August 2025 (Phase 7)  
**Status**: Deferred  
**Context**: JavaScript support complex with V8/QuickJS  
**Decision**: Focus on Lua, defer JavaScript  
**Rationale**: Limited resources, Lua working well  
**Consequences**:
- ✅ Faster Phase 7 completion
- ✅ More stable Lua implementation
- ❌ No JS support
- ❌ Smaller potential audience

---

## Decision Evolution & Reversals

### Tool Count Evolution
- **Phase 2**: 26 self-contained tools
- **Phase 3.0**: Standardized to 26 tools
- **Phase 3.1**: Added 8 external tools (34 total)
- **Phase 3.2**: Optimized all 34 tools
- **Phase 7**: 37+ tools in production
- **Phase 8**: 37+ tools + RAG system (not counted as tool)

### Parameter Naming Evolution
- **Phase 2**: Inconsistent (text, content, data, input)
- **Phase 3**: Standardized (input, path, operation)
- **Impact**: 95% consistency from 60%

### Test Infrastructure Evolution
- **Initial**: cfg_attr approach (broken)
- **Phase 7**: Feature-based testing
- **Result**: 536+ test files consolidated

### State Persistence Evolution
- **Phase 3**: Basic state management
- **Phase 5**: Full persistence with migrations
- **Phase 6**: Session integration with artifacts
- **Phase 8**: StateScope extended for multi-tenant RAG
- **Achievement**: 35+ modules, 2.07μs/item performance

### API Naming Evolution
- **Pre-Phase 7**: Mixed (Service, Manager, retrieve, fetch)
- **Phase 7**: Standardized (Manager suffix, get() method)
- **Phase 8**: Two-parameter pattern for RAG (primary, options)
- **Impact**: Consistent developer experience

### RAG System Evolution
- **Phase 7**: Basic RAG mock/stub
- **Phase 8**: Complete RAG implementation
  - HNSW vector storage (100K+ vectors)
  - Multi-tenant isolation (3% overhead)
  - OpenAI embeddings (384 dimensions)
  - Session-scoped vectors with TTL
- **Achievement**: 8ms search on 100K vectors, enterprise-ready

---

## Lessons Learned

1. **Start with traits**: BaseAgent foundation proved invaluable
2. **Utils crate early**: Phase 1 decision to create llmspell-utils paid off
3. **Bridge complexity worth it**: Sync bridge keeps scripts simple
4. **Clean breaks acceptable**: Pre-1.0 freedom enabled better architecture
5. **Performance targets help**: Guided optimization decisions
6. **Defer complex features**: Phase 5 custom transformers deferred correctly
7. **Standardization matters**: Phase 7 API consistency improved usability
8. **Test infrastructure critical**: Phase 7 test reorganization was necessary
9. **Multi-tenant first**: Phase 8 tenant isolation from start was right choice
10. **Configuration over compilation**: Runtime RAG config enables production tuning

---

## Future Decisions (Deferred)

### ADR-037: GUI Framework Selection
**Status**: Pending (Phase 9+)  
**Options**: Tauri, egui, web-based  

### ADR-038: Python Integration Strategy
**Status**: Pending (Phase 9+)  
**Options**: PyO3, embedded Python, subprocess  

### ADR-039: Distributed Execution
**Status**: Pending (Phase 12)  
**Options**: Custom protocol, gRPC, message queue

### ADR-040: Local Embedding Models
**Status**: Pending (Phase 9+)
**Options**: Candle integration, ONNX runtime, native implementations
**Models**: BGE-M3, E5, ColBERT v2

### ADR-041: Hybrid Search Implementation
**Status**: Pending (Phase 9+)
**Options**: BM25 + vector, late interaction models
**Considerations**: Performance vs accuracy trade-offs  

---

*This document represents the consolidated architectural decisions from Phases 0-11 of LLMSpell development, validated against phase design documents.*