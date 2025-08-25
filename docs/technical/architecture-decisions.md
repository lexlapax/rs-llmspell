# Architecture Decision Records (ADRs)

**Version**: 0.6.0  
**Last Updated**: August 2025  
**Validation**: Cross-referenced with phase design documents (phase-01 through phase-07)

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
8. [Cross-Cutting Decisions](#cross-cutting-decisions)
9. [Decision Evolution & Reversals](#decision-evolution--reversals)

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

## Cross-Cutting Decisions

### ADR-025: Three-Level Security Model

**Date**: July 2025 (Phase 3)  
**Status**: Accepted  
**Context**: Need granular security without complexity  
**Decision**: Three levels: Safe, Restricted, Privileged  
**Implementation**: SecurityLevel enum with enforcement  
**Consequences**:
- ✅ Simple to understand
- ✅ Easy to audit
- ✅ Progressive trust model

### ADR-026: mlua Over Other Lua Bindings

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Need robust Lua integration  
**Decision**: Use mlua with Lua 5.4  
**Rationale**: Most mature binding, async support  
**Consequences**:
- ✅ Stable integration
- ✅ Good documentation
- ❌ No LuaJIT (performance tradeoff)

### ADR-027: rig-core for LLM Providers

**Date**: June 2025 (Phase 1)  
**Status**: Accepted  
**Context**: Need LLM provider integration  
**Decision**: Use rig-core instead of custom implementation  
**Consequences**:
- ✅ Fast implementation
- ✅ Multiple providers supported
- ❌ External dependency
- ❌ Less control over provider details

### ADR-028: No JavaScript Implementation

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
- **Current**: 37+ tools in production

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
- **Achievement**: 35+ modules, 2.07μs/item performance

### API Naming Evolution
- **Pre-Phase 7**: Mixed (Service, Manager, retrieve, fetch)
- **Phase 7**: Standardized (Manager suffix, get() method)
- **Impact**: Consistent developer experience

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

---

## Future Decisions (Deferred)

### ADR-029: GUI Framework Selection
**Status**: Pending (Phase 8)  
**Options**: Tauri, egui, web-based  

### ADR-030: Python Integration Strategy
**Status**: Pending (Phase 9)  
**Options**: PyO3, embedded Python, subprocess  

### ADR-031: Distributed Execution
**Status**: Pending (Phase 12)  
**Options**: Custom protocol, gRPC, message queue  

---

*This document represents the consolidated architectural decisions from Phases 0-7 of LLMSpell development, validated against phase design documents.*