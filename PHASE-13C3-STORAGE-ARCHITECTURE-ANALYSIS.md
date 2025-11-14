# Phase 13c.3: Storage Architecture Analysis & Centralized Trait Refactoring

**Status**: Research & Design Complete
**Date**: November 2025
**Author**: Claude (Comprehensive Analysis)
**Purpose**: Architectural analysis for centralized storage trait refactoring and PostgreSQL/SQLite alignment

---

## Executive Summary

### Current State (Post-Phase 13c.2)
- **10 storage components** fully implemented with PostgreSQL (Phase 13b) and SQLite (Phase 13c.2)
- **Fragmented trait architecture**: Traits scattered across **5 crates** (llmspell-storage, llmspell-graph, llmspell-memory, llmspell-events, llmspell-core)
- **Implementation complete**: 100% feature parity between PostgreSQL and SQLite backends
- **Problem**: Trait definitions not centralized, creating maintenance burden and circular dependency risks
- **⚠️ CRITICAL FINDING**: **11 crates** directly use storage traits (not 5!), including **llmspell-bridge** which was initially missed

### Scope Expansion After Comprehensive Analysis
Initial analysis focused on core storage crates. **Comprehensive grep-based analysis revealed**:
- **11 crates** directly depend on storage traits (not 5)
- **llmspell-bridge** has 9+ files using storage traits (MAJOR consumer, initially overlooked)
- **llmspell-rag** has 8+ files using VectorStorage
- **llmspell-tenancy** implements VectorStorage trait (decorator pattern)
- **llmspell-memory** uses all 3 storage trait categories (highest complexity)
- **50+ source files** across workspace need import updates

### Proposed Solution
**Centralized Trait Architecture** in `llmspell-core` with:
1. All storage traits in `llmspell-core/src/traits/storage/` (single source of truth)
2. Domain types in `llmspell-core/src/types/storage/` (shared across backends)
3. Backend implementations in `llmspell-storage/src/backends/{postgres,sqlite}/` (follow traits)
4. Runtime injection via config: `Infrastructure::from_config()` → instantiated backends → kernel/bridge
5. **Re-export strategy** in all 11 impacted crates (zero breaking changes)

### Revised Impact Assessment
- **+3,500 lines**: New trait definitions and domain types in llmspell-core
- **~50 files** need import updates across 11 crates (not 20 as initially estimated)
- **Zero breaking changes**: Re-exports preserve all existing import paths
- **50% reduction** in future maintenance: Single trait definition for all backends
- **Eliminates circular dependencies**: Foundation crate (llmspell-core) has no internal deps
- **Week 2 effort increase**: 11 crates to update (not 5), estimated +8 hours

---

## Table of Contents

1. [Current Architecture Analysis](#1-current-architecture-analysis)
2. [Storage Component Inventory](#2-storage-component-inventory)
3. [Trait Location Audit](#3-trait-location-audit)
4. [Backend Implementation Matrix](#4-backend-implementation-matrix)
5. [Migration Matrix (PostgreSQL ↔ SQLite)](#5-migration-matrix-postgresql--sqlite)
6. [Problem Statement](#6-problem-statement)
7. [Proposed Architecture](#7-proposed-architecture)
8. [Detailed Design](#8-detailed-design)
9. [Migration Strategy](#9-migration-strategy)
10. [Risk Analysis](#10-risk-analysis)
11. [Implementation Plan](#11-implementation-plan)
12. **[What Was Initially Missed](#12-what-was-initially-missed)** ⚠️ NEW

---

## What Was Initially Missed (Critical Findings)

### Initial Analysis Scope
The first version of this document focused on:
- llmspell-storage (backend implementations)
- llmspell-graph (KnowledgeGraph trait)
- llmspell-memory (memory traits)
- llmspell-events (EventStorage adapter)
- llmspell-kernel (StateManager usage)

**Total**: 5 crates identified

### Comprehensive Analysis Revealed
After thorough `grep` and `Cargo.toml` analysis across ALL 21 crates:

#### 1. **llmspell-bridge** (CRITICAL MISS)
**Why missed**: Initial focus on "storage" crate names, overlooked bridge layer
**Impact**: 9+ files use storage traits
- `infrastructure.rs` - Creates SqliteBackend directly (no injection)
- `rag_bridge.rs` - Uses Arc<dyn VectorStorage>
- `memory_bridge.rs` - Uses MemoryManager (storage traits)
- `globals/` - 3 files create backends inline

**Risk**: Bridge is the main entry point for Lua/JS scripts - HIGH impact on refactor!

#### 2. **llmspell-rag** (HIGH IMPACT)
**Why missed**: Assumed RAG only used via kernel, not directly
**Impact**: 8+ files use VectorStorage
- **StateAwareVectorStorage** - Wraps VectorStorage for multi-tenancy
- **HybridStorage trait** - Extends VectorStorage (trait inheritance!)
- Pipeline, retrieval, ingestion all use Arc<dyn VectorStorage>

**Risk**: RAG is a core feature - changes affect many users!

#### 3. **llmspell-tenancy** (MEDIUM IMPACT - DECORATOR PATTERN)
**Why missed**: Tenancy seemed like infrastructure, not storage consumer
**Impact**: **MultiTenantVectorManager implements VectorStorage**
- This is a **decorator pattern** wrapping any VectorStorage backend
- Trait implementation depends on VectorStorage being in scope

**Risk**: If VectorStorage moves to llmspell-core AFTER tenancy compiles, circular dependency possible!

#### 4. **llmspell-agents** (MEDIUM IMPACT)
**Why missed**: Agents seemed orthogonal to storage
**Impact**: AgentRegistry uses StorageBackend for persistence
- `registry/persistence.rs` - PersistentAgentRegistry struct

**Risk**: LOW - only uses generic StorageBackend (KV)

#### 5. **llmspell-context** (LOW IMPACT - INDIRECT)
**Why missed**: Initially assumed it was domain-only
**Impact**: Indirect dependency via llmspell-memory and llmspell-graph
- No direct storage trait usage
- Compilation will break if memory/graph break

**Risk**: LOW - no code changes needed, just verification

#### 6. **llmspell-cli** (LOW IMPACT)
**Why missed**: CLI seemed like thin wrapper
**Impact**: Uses kernel infrastructure (StateManager, etc.)
- No direct backend creation
- Just imports types for CLI commands

**Risk**: VERY LOW - minimal changes

### Summary of Missed Items

| Item | Initial Estimate | Actual Count | Delta |
|------|-----------------|--------------|-------|
| **Crates impacted** | 5 | 11 | +6 (120% increase) |
| **Files to update** | ~20 | ~50 | +30 (150% increase) |
| **Critical crates** | 2 (storage, kernel) | 3 (storage, kernel, **bridge**) | +1 |
| **Migration effort** | 4 weeks | **5 weeks** | +1 week |

### Revised Risk Assessment

**Original risks** (underestimated):
- ✅ Type parameter mismatches
- ✅ Doc test breakage
- ✅ Integration test failures

**New risks** (discovered):
- ⚠️ **Bridge layer disruption**: llmspell-bridge is Lua/JS entry point - breaks all scripts if incorrect
- ⚠️ **RAG pipeline refactor**: 8 files using VectorStorage - high user impact
- ⚠️ **Tenancy decorator pattern**: MultiTenantVectorManager depends on VectorStorage location
- ⚠️ **Test infrastructure**: Many tests create backends inline - needs factory pattern

### Lessons Learned

1. **Grep is essential**: Can't rely on crate names or mental models - must grep for trait usage
2. **Bridge layer critical**: llmspell-bridge is infrastructure creation hub - always check it
3. **Decorator patterns**: Traits implemented BY domain crates (not just used) create tight coupling
4. **Test code matters**: Test files often create backends directly - need injection pattern there too

### Mitigation Strategy

1. **Week 2 extension**: Add 1 day for llmspell-bridge updates
2. **Bridge-first approach**: Update llmspell-bridge BEFORE memory/rag to catch issues early
3. **Test factories**: Create `TestStorageFactory` in llmspell-testing for all test backends
4. **Incremental compilation**: Update crates in dependency order (core → storage → bridge → domain)

---

## 1. Current Architecture Analysis

### 1.1 Crate Dependency Graph

```
┌─────────────────┐
│  llmspell-core  │  ← Foundation crate (NO internal dependencies)
└────────┬────────┘
         │
    ┌────┴──────────────────────────────────────┐
    │                                            │
┌───▼────────────┐                    ┌─────────▼────────┐
│ llmspell-      │                    │  llmspell-graph  │
│ storage        │                    │  (KnowledgeGraph)│
└───┬────────────┘                    └──────────────────┘
    │
    ├──> postgres/   (PostgresBackend, PostgresVectorStorage, etc.)
    ├──> sqlite/     (SqliteBackend, SqliteVectorStorage, etc.)
    └──> vector/     (Legacy HNSW files - Phase 13c.2.8 removal)

┌─────────────────┐
│ llmspell-memory │  ← Depends on llmspell-storage, llmspell-graph
└─────────────────┘
    └──> traits/   (EpisodicMemory, SemanticMemory, ProceduralMemory)

┌─────────────────┐
│ llmspell-events │  ← Depends on llmspell-storage
└─────────────────┘
    └──> EventStorage trait → StorageAdapter<B: StorageBackend>

┌─────────────────┐
│ llmspell-hooks  │  ← Depends on llmspell-storage
└─────────────────┘
    └──> HookReplayManager uses StorageBackend

┌─────────────────┐
│ llmspell-kernel │  ← Uses all above crates
└─────────────────┘
    └──> StateManager uses StorageBackend
```

**⚠️ WARNING: This diagram is INCOMPLETE!** See Section 1.3 for full crate dependency analysis.

### 1.2 Complete Crate Dependency Map (All 21 Crates)

After comprehensive analysis, **11 crates** directly depend on storage traits:

| Crate | Depends On | Storage Traits Used | Direct Backend Creation | Impact Level |
|-------|-----------|---------------------|------------------------|--------------|
| **llmspell-storage** | llmspell-core, llmspell-graph | ALL (implements backends) | ✅ PostgresBackend, SqliteBackend | **CRITICAL** |
| **llmspell-kernel** | llmspell-storage, llmspell-memory | StorageBackend, VectorStorage | ✅ SqliteBackend (StateManager) | **CRITICAL** |
| **llmspell-bridge** | llmspell-storage, llmspell-memory | StorageBackend, VectorStorage | ✅ SqliteBackend (Infrastructure) | **CRITICAL** |
| **llmspell-memory** | llmspell-storage, llmspell-graph | VectorStorage, KnowledgeGraph, ProceduralMemory | ❌ (receives injected) | **HIGH** |
| **llmspell-rag** | llmspell-storage | VectorStorage | ✅ SqliteVectorStorage (tests) | **HIGH** |
| **llmspell-tenancy** | llmspell-storage | VectorStorage | ✅ SqliteVectorStorage (tests) | **MEDIUM** |
| **llmspell-agents** | llmspell-storage | StorageBackend | ❌ (receives injected) | **MEDIUM** |
| **llmspell-events** | llmspell-storage | StorageBackend (adapter pattern) | ❌ (receives injected) | **MEDIUM** |
| **llmspell-hooks** | llmspell-storage | StorageBackend | ❌ (receives injected) | **MEDIUM** |
| **llmspell-context** | llmspell-memory, llmspell-graph | Indirect (via memory/graph) | ❌ | **LOW** |
| **llmspell-cli** | llmspell-storage | StorageBackend (via kernel) | ❌ | **LOW** |

**Crates with NO storage dependencies** (10): llmspell-core, llmspell-graph, llmspell-config, llmspell-providers, llmspell-tools, llmspell-templates, llmspell-workflows, llmspell-security, llmspell-testing, llmspell-utils

### 1.3 Critical Finding: llmspell-bridge Storage Usage

**llmspell-bridge is a MAJOR storage consumer** that was missing from initial analysis:

```rust
// llmspell-bridge/src/infrastructure.rs:53
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};

impl Infrastructure {
    pub async fn from_config(config: &LLMSpellConfig) -> Result<Self> {
        // PROBLEM: Direct backend construction (no injection pattern)
        let storage_backend = Arc::new(SqliteBackend::new(config).await?);
        let vector_storage = Arc::new(SqliteVectorStorage::new(backend, 384).await?);

        // Creates StateManager, SessionManager, RAG, MemoryManager
        // All receive backends inline
    }
}
```

**Bridge crate files using storage traits**:
1. `infrastructure.rs` - Creates SqliteBackend, SqliteVectorStorage directly
2. `rag_bridge.rs` - Uses `VectorStorage` trait (Arc<dyn VectorStorage>)
3. `memory_bridge.rs` - Uses `MemoryManager` (which uses storage traits)
4. `context_bridge.rs` - Uses `MemoryManager`
5. `state_adapter.rs` - Uses `StorageBackend`
6. `artifact_bridge.rs` - Uses `ArtifactStorage` (via SessionManager)
7. `globals/rag_infrastructure.rs` - Creates SqliteVectorStorage
8. `globals/state_infrastructure.rs` - Uses StorageBackend
9. `globals/session_infrastructure.rs` - Uses StorageBackend

**Impact**: llmspell-bridge needs trait refactoring in 9+ files!

### 1.4 llmspell-rag Storage Usage

**llmspell-rag heavily uses VectorStorage trait**:

```rust
// llmspell-rag/src/pipeline/rag_pipeline.rs:28
pub struct RAGPipeline {
    storage: Arc<dyn VectorStorage>,  // Trait object injection
    // ...
}

// llmspell-rag/src/state_integration.rs:15
pub struct StateAwareVectorStorage {
    storage: Arc<dyn VectorStorage>,  // Wraps VectorStorage for multi-tenancy
    // ...
}
```

**Files using VectorStorage** (8 files):
1. `traits/hybrid.rs` - `pub trait HybridStorage: VectorStorage` (extends trait)
2. `pipeline/rag_pipeline.rs` - Main pipeline uses `Arc<dyn VectorStorage>`
3. `pipeline/retrieval_flow.rs` - Retrieval uses `Arc<dyn VectorStorage>`
4. `pipeline/builder.rs` - Builder pattern for pipeline construction
5. `pipeline/ingestion.rs` - Ingestion pipeline
6. `state_integration.rs` - **StateAwareVectorStorage wrapper**
7. `session_integration.rs` - Session-scoped RAG

**Test files creating backends**:
- 4 test files create `SqliteBackend` and `SqliteVectorStorage` directly (not injected)

**Impact**: Core RAG functionality depends on VectorStorage trait location!

### 1.5 llmspell-tenancy Storage Usage

**llmspell-tenancy wraps VectorStorage for multi-tenant isolation**:

```rust
// llmspell-tenancy/src/manager.rs:37
pub struct MultiTenantVectorManager {
    storage: Arc<dyn VectorStorage>,  // Injected backend
    // ...
}

// llmspell-tenancy/src/manager.rs:435
impl VectorStorage for MultiTenantVectorManager {
    // Delegates to underlying storage with tenant filtering
}
```

**Key finding**: MultiTenantVectorManager **implements VectorStorage trait** itself! This is a decorator pattern that adds tenant isolation on top of any VectorStorage backend.

**Impact**: VectorStorage trait must be in llmspell-core for tenancy crate to compile!

### 1.6 llmspell-agents Storage Usage

**llmspell-agents uses StorageBackend for registry persistence**:

```rust
// llmspell-agents/src/registry/persistence.rs:24
pub struct PersistentAgentRegistry {
    storage: Arc<dyn StorageBackend>,  // Generic KV storage
    // ...
}
```

**Usage**: Saves/loads agent configurations to/from storage backend.

**Impact**: LOW - Only uses StorageBackend (generic KV), no complex storage traits.

### 1.7 llmspell-memory Storage Usage

**llmspell-memory is the HIGHEST complexity storage user**:

```rust
// llmspell-memory/src/manager.rs
pub struct DefaultMemoryManager {
    episodic: Arc<dyn EpisodicMemory>,      // Uses VectorStorage internally
    semantic: Arc<dyn SemanticMemory>,      // Uses KnowledgeGraph internally
    procedural: Arc<dyn ProceduralMemory>,  // Direct storage trait
    // ...
}
```

**Memory crate defines 3 domain traits that WRAP storage traits**:
1. `EpisodicMemory` → wraps `VectorStorage`
2. `SemanticMemory` → wraps `KnowledgeGraph`
3. `ProceduralMemory` → direct storage trait (should be in llmspell-core)

**Consolidation engine uses KnowledgeGraph**:
```rust
// llmspell-memory/src/consolidation/validator.rs:21
pub struct ConsolidationValidator {
    knowledge_graph: Arc<dyn KnowledgeGraph>,
}

// llmspell-memory/src/consolidation/llm_engine.rs:108
pub struct LLMConsolidationEngine {
    knowledge_graph: Arc<dyn KnowledgeGraph>,
}
```

**Impact**: CRITICAL - Memory system deeply integrates with all 3 storage trait categories!

### 1.8 Current Trait Distribution

| Trait Name | Current Location | Line Count | Implementations |
|-----------|------------------|------------|-----------------|
| `StorageBackend` | llmspell-storage/src/traits.rs:47 | 54 | Memory, Sqlite, Postgres |
| `VectorStorage` | llmspell-storage/src/vector_storage.rs:20 | 200+ | SqliteVectorStorage, PostgresVectorStorage |
| `KnowledgeGraph` | llmspell-graph/src/traits/knowledge_graph.rs:21 | 142 | SurrealDB (legacy), SqliteGraphStorage, PostgresGraphStorage |
| `ProceduralMemory` | llmspell-memory/src/traits/procedural.rs:49 | 100 | SqliteProceduralStorage, PostgresProceduralStorage |
| `EventStorage` | llmspell-events/src/storage_adapter.rs:17 | 42 | EventStorageAdapter<B: StorageBackend> |
| `WorkflowStateStorage` | llmspell-core/src/traits/storage/workflow.rs:51 | 27 | SqliteWorkflowStateStorage (✅ already in core) |
| `SessionStorage` | llmspell-core/src/traits/storage/session.rs:52 | 35 | SqliteSessionStorage, PostgresSessionStorage (✅ already in core) |
| `ArtifactStorage` | llmspell-core/src/traits/storage/artifact.rs:60 | 30 | SqliteArtifactStorage, PostgresArtifactStorage (✅ already in core) |
| `EpisodicMemory` | llmspell-memory/src/traits/episodic.rs:47 | 80 | InMemory, HNSW, Sqlite, Postgres |
| `SemanticMemory` | llmspell-memory/src/traits/semantic.rs:66 | 60 | GraphSemanticMemory (wraps KnowledgeGraph) |

**Key Observations**:
1. **Partial Migration Already Done**: 3 traits (Workflow, Session, Artifact) already in `llmspell-core` (Phase 13c.2.0)
2. **Fragmentation**: 7 remaining traits scattered across 4 crates
3. **Circular Dependency Risk**: llmspell-storage → llmspell-graph dependency creates cycle potential

---

## 2. Storage Component Inventory

### 2.1 Complete Storage Component Map (10 Components)

| Component | PostgreSQL Migration | SQLite Migration | Backend Trait | Domain Trait | Current Status |
|-----------|---------------------|------------------|---------------|--------------|----------------|
| **V3: Vector Embeddings** | V3 (4 dimensions: 384/768/1536/3072) | V3 (vectorlite HNSW) | `VectorStorage` | `EpisodicMemory` | ✅ Complete |
| **V4: Temporal Graph** | V4 (tstzrange bi-temporal) | V4 (INTEGER start/end) | `KnowledgeGraph` | `SemanticMemory` | ✅ Complete |
| **V5: Procedural Patterns** | V5 (frequency tracking) | V5 (UPSERT pattern) | `ProceduralMemory` | - | ✅ Complete |
| **V6: Agent State** | V6 (JSONB state data) | V6 (TEXT JSON) | `StorageBackend` | - | ✅ Complete |
| **V7: KV Store** | V7 (generic key-value) | V7 (scope isolation) | `StorageBackend` | - | ✅ Complete |
| **V8: Workflow States** | V8 (lifecycle tracking) | V8 (status transitions) | `WorkflowStateStorage` ✅ | - | ✅ Complete |
| **V9: Sessions** | V9 (expiration management) | V9 (cleanup_expired) | `SessionStorage` ✅ | - | ✅ Complete |
| **V10: Artifacts** | V10 (Large Objects for >1MB) | V10 (BLOB inline) | `ArtifactStorage` ✅ | - | ✅ Complete |
| **V11: Event Log** | V11 (monthly partitioning) | V11 (time-series events) | `EventStorage` | - | ✅ Complete |
| **V13: Hook History** | V13 (correlation tracking) | V13 (replay support) | `StorageBackend` | - | ✅ Complete |

### 2.2 Implementation File Structure

```
llmspell-storage/src/backends/
├── postgres/
│   ├── backend.rs          (PostgresBackend: StorageBackend)
│   ├── vector.rs           (PostgresVectorStorage: VectorStorage)
│   ├── graph.rs            (PostgresGraphStorage: KnowledgeGraph) ← ⚠️ WRONG CRATE
│   ├── procedural.rs       (PostgresProceduralStorage: ProceduralMemory)
│   ├── agent_state.rs      (embedded in backend.rs)
│   ├── workflow_state.rs   (PostgresWorkflowStateStorage: WorkflowStateStorage)
│   ├── session.rs          (PostgresSessionStorage: SessionStorage)
│   ├── artifact.rs         (PostgresArtifactStorage: ArtifactStorage)
│   ├── event_log.rs        (PostgresEventLogStorage: EventStorage)
│   └── hook_history.rs     (PostgresHookHistoryStorage: StorageBackend)
│
└── sqlite/
    ├── backend.rs          (SqliteBackend: StorageBackend)
    ├── vector.rs           (SqliteVectorStorage: VectorStorage)
    ├── graph.rs            (SqliteGraphStorage: KnowledgeGraph) ← ⚠️ WRONG CRATE
    ├── procedural.rs       (SqliteProceduralStorage: ProceduralMemory)
    ├── agent_state.rs      (SqliteAgentStateStorage: StorageBackend)
    ├── kv_store.rs         (SqliteKVStorage: StorageBackend)
    ├── workflow_state.rs   (SqliteWorkflowStateStorage: WorkflowStateStorage)
    ├── session.rs          (SqliteSessionStorage: SessionStorage)
    ├── artifact.rs         (SqliteArtifactStorage: ArtifactStorage)
    ├── event_log.rs        (SqliteEventLogStorage: EventStorage)
    └── hook_history.rs     (SqliteHookHistoryStorage: StorageBackend)
```

**Problem**: Graph storage implementations are in `llmspell-storage` but `KnowledgeGraph` trait is in `llmspell-graph` crate!

---

## 3. Trait Location Audit

### 3.1 Traits Already in llmspell-core ✅

These were added in Phase 13c.2.0 (Task 13c.2.0):

```rust
// llmspell-core/src/traits/storage/workflow.rs
pub trait WorkflowStateStorage: Send + Sync {
    async fn save_state(&self, workflow_id: &str, state: &WorkflowState) -> Result<()>;
    async fn load_state(&self, workflow_id: &str) -> Result<Option<WorkflowState>>;
    async fn update_status(&self, workflow_id: &str, status: WorkflowStatus) -> Result<()>;
    async fn list_workflows(&self, status_filter: Option<WorkflowStatus>) -> Result<Vec<String>>;
    async fn delete_state(&self, workflow_id: &str) -> Result<()>;
}

// llmspell-core/src/traits/storage/session.rs
pub trait SessionStorage: Send + Sync {
    async fn create_session(&self, session_id: &str, data: &SessionData) -> Result<()>;
    async fn get_session(&self, session_id: &str) -> Result<Option<SessionData>>;
    async fn update_session(&self, session_id: &str, data: &SessionData) -> Result<()>;
    async fn delete_session(&self, session_id: &str) -> Result<()>;
    async fn list_active_sessions(&self) -> Result<Vec<String>>;
    async fn cleanup_expired(&self) -> Result<usize>;
}

// llmspell-core/src/traits/storage/artifact.rs
pub trait ArtifactStorage: Send + Sync {
    async fn store_artifact(&self, artifact: &Artifact) -> Result<ArtifactId>;
    async fn get_artifact(&self, id: &ArtifactId) -> Result<Option<Artifact>>;
    async fn delete_artifact(&self, id: &ArtifactId) -> Result<()>;
    async fn list_session_artifacts(&self, session_id: &str) -> Result<Vec<ArtifactId>>;
    async fn get_storage_stats(&self, session_id: &str) -> Result<SessionStorageStats>;
}
```

### 3.2 Traits to Migrate to llmspell-core

| Trait | Current Location | Lines | Migration Complexity | Reason to Move |
|-------|------------------|-------|---------------------|----------------|
| `StorageBackend` | llmspell-storage/src/traits.rs | 101 | **LOW** | Generic KV trait, no domain deps |
| `VectorStorage` | llmspell-storage/src/vector_storage.rs | 200+ | **MEDIUM** | Many helper types (VectorEntry, VectorQuery, VectorResult), but domain-agnostic |
| `KnowledgeGraph` | llmspell-graph/src/traits/knowledge_graph.rs | 142 | **MEDIUM** | Graph domain types (Entity, Relationship, TemporalQuery) need migration |
| `ProceduralMemory` | llmspell-memory/src/traits/procedural.rs | 100 | **LOW** | Simple pattern tracking, minimal types |
| `EventStorage` | llmspell-events/src/storage_adapter.rs | 42 | **LOW** | Already uses StorageBackend, can stay as adapter pattern |
| `EpisodicMemory` | llmspell-memory/src/traits/episodic.rs | 80 | **HIGH** | Memory-specific, can stay in llmspell-memory |
| `SemanticMemory` | llmspell-memory/src/traits/semantic.rs | 60 | **HIGH** | Memory-specific, wraps KnowledgeGraph, can stay |

**Recommendation**: Migrate `StorageBackend`, `VectorStorage`, `KnowledgeGraph`, `ProceduralMemory` to llmspell-core. Keep memory-specific traits (`EpisodicMemory`, `SemanticMemory`) in llmspell-memory as domain wrappers.

---

## 4. Backend Implementation Matrix

### 4.1 Feature Parity Matrix

| Feature | PostgreSQL | SQLite | Notes |
|---------|-----------|--------|-------|
| **Vector Search** | VectorChord HNSW | vectorlite-rs HNSW | Both use HNSW indexing |
| **Vector Dimensions** | 384, 768, 1536, 3072 | 384, 768, 1536, 3072 | Identical support |
| **Graph Bi-Temporal** | tstzrange (native) | INTEGER start/end | Different representation, same semantics |
| **Graph Traversal** | Recursive CTEs | Recursive CTEs | SQLite has native CTE support (since 3.8.3, 2014) |
| **Procedural Patterns** | UPSERT with ON CONFLICT | INSERT OR UPDATE | Syntax differs, functionality identical |
| **Agent State** | JSONB | TEXT (json_extract) | SQLite json1 extension equivalent |
| **Workflow States** | ENUM types | TEXT with CHECK | No ENUMs in SQLite, use TEXT |
| **Sessions** | Expiration via TIMESTAMPTZ | Expiration via INTEGER (Unix) | Both support cleanup_expired() |
| **Artifacts** | Large Objects (>1MB) | BLOB inline (all sizes) | PostgreSQL optimizes >1MB separately |
| **Event Log** | Monthly partitioning | Single table | Performance difference at scale |
| **Hook History** | Correlation tracking | Correlation tracking | Identical interface |
| **Multi-Tenancy** | RLS (Row-Level Security) | Application-level WHERE | PostgreSQL enforces at DB level |
| **Migrations** | 15 files (V1-V15) | 13 files (V1-V13, missing V2/V12/V14) | SQLite missing: test table RLS, application role enforcement, API keys |

### 4.2 Type Compatibility Matrix

| PostgreSQL Type | SQLite Type | Conversion Strategy | Lossless? |
|----------------|-------------|---------------------|-----------|
| `UUID` | `TEXT` (36 chars) | `uuid.to_string()` → `Uuid::parse_str()` | ✅ YES |
| `TIMESTAMPTZ` | `INTEGER` (Unix seconds) | `.timestamp()` → `DateTime::from_timestamp()` | ✅ YES |
| `JSONB` | `TEXT` | `serde_json::to_string()` → `serde_json::from_str()` | ✅ YES |
| `BYTEA` | `BLOB` | Direct copy | ✅ YES |
| `VECTOR(n)` | `BLOB` (f32 array) | MessagePack serialization | ✅ YES |
| `tstzrange` | `(start INTEGER, end INTEGER)` | Extract bounds → reconstruct | ✅ YES |
| `ENUM` | `TEXT` with CHECK constraint | String representation | ✅ YES |
| Large Objects (OID) | `BLOB` (inline) | Read chunks → concatenate | ✅ YES |

**Conclusion**: All PostgreSQL types have lossless SQLite equivalents. Bidirectional migration is feasible.

---

## 5. Migration Matrix (PostgreSQL ↔ SQLite)

### 5.1 Schema Equivalence

#### Example: V3 Vector Embeddings

**PostgreSQL** (`migrations/postgres/V3__vector_embeddings.sql`):
```sql
CREATE TABLE vector_embeddings_768 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    embedding VECTOR(768) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_vector_768_hnsw ON vector_embeddings_768
USING hnsw (embedding vector_cosine_ops);  -- VectorChord HNSW
```

**SQLite** (`migrations/sqlite/V3__vector_embeddings.sql`):
```sql
CREATE TABLE vec_embeddings_768 (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    tenant_id TEXT NOT NULL,
    scope TEXT NOT NULL,
    embedding BLOB NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- vectorlite-rs virtual table for HNSW search
CREATE VIRTUAL TABLE vec_search_768 USING vectorlite(
    dimension=768,
    metric='cosine',
    m=16,
    ef_construction=200
);
```

**Key Differences**:
1. **UUID**: PostgreSQL UUID type → SQLite TEXT with randomblob()
2. **VECTOR**: PostgreSQL native → SQLite BLOB (MessagePack f32 array)
3. **JSONB**: PostgreSQL JSONB → SQLite TEXT
4. **TIMESTAMPTZ**: PostgreSQL now() → SQLite strftime('%s', 'now')
5. **HNSW Index**: PostgreSQL VectorChord → SQLite vectorlite virtual table

---

## 6. Problem Statement

### 6.1 Current Issues

1. **Trait Fragmentation**: 10 storage traits spread across 5 crates makes maintenance difficult
2. **Circular Dependency Risk**: llmspell-storage implements KnowledgeGraph from llmspell-graph (wrong direction)
3. **Inconsistent Patterns**: Some traits in core (Workflow, Session, Artifact), others in domain crates
4. **Duplicate Types**: VectorEntry, Entity, Pattern types defined in multiple places
5. **No Single Source of Truth**: Changes to storage interface require updates across multiple crates

### 6.2 Specific Problems Identified

**Problem 1: Graph Trait in Wrong Crate**
- `KnowledgeGraph` trait lives in `llmspell-graph` (domain crate)
- `SqliteGraphStorage` and `PostgresGraphStorage` live in `llmspell-storage` (infrastructure crate)
- **Issue**: Infrastructure should not depend on domain! (Inverted dependency)

**Problem 2: VectorStorage Duplication**
- `llmspell-storage/src/vector_storage.rs` defines VectorStorage (200+ lines)
- `llmspell-kernel/src/state/vector_storage.rs` **duplicates** the same trait (575 lines, includes HNSWStorage)
- **Issue**: Two definitions of the same trait creates confusion and drift risk

**Problem 3: Memory Traits Not Centralized**
- `EpisodicMemory`, `SemanticMemory`, `ProceduralMemory` in llmspell-memory
- These wrap storage backends but their traits aren't in foundation layer
- **Issue**: Cannot swap memory implementations without modifying llmspell-memory

**Problem 4: No Runtime Injection Pattern**
- Backends created inline with `PostgresBackend::new()`, `SqliteBackend::new()`
- StateManager, MemoryManager, GraphBackend all construct their own backends
- **Issue**: No centralized configuration → runtime backend selection pattern

### 6.3 Phase 13c.3 Goals from TODO.md

From Phase 13c.3 Task 13c.3.2 acceptance criteria:
- [ ] Bidirectional export/import tool: `llmspell storage export/import` (PostgreSQL ↔ JSON ↔ SQLite)
- [ ] Type conversion layer in backend implementations
- [ ] Tenant isolation compatibility (PostgreSQL RLS → SQLite session variables)
- [ ] Full data roundtrip test: PostgreSQL → JSON → SQLite → JSON → PostgreSQL (zero data loss)
- [ ] Schema compatibility matrix documented

---

## 7. Proposed Architecture

### 7.1 Centralized Trait Layer

**All storage traits move to `llmspell-core/src/traits/storage/`**:

```
llmspell-core/src/traits/storage/
├── mod.rs                  (exports all storage traits)
├── backend.rs              (StorageBackend - generic KV)
├── vector.rs               (VectorStorage + helper types)
├── graph.rs                (KnowledgeGraph + Entity/Relationship types)
├── procedural.rs           (ProceduralMemory + Pattern type)
├── event.rs                (EventStorage - optional, adapter pattern OK)
├── workflow.rs             (WorkflowStateStorage) ✅ EXISTS
├── session.rs              (SessionStorage) ✅ EXISTS
└── artifact.rs             (ArtifactStorage) ✅ EXISTS
```

### 7.2 Domain Types Layer

**All storage-related types move to `llmspell-core/src/types/storage/`**:

```
llmspell-core/src/types/storage/
├── mod.rs
├── backend.rs              (StorageBackendType, StorageCharacteristics)
├── vector.rs               (VectorEntry, VectorQuery, VectorResult, DistanceMetric)
├── graph.rs                (Entity, Relationship, TemporalQuery)
├── procedural.rs           (Pattern)
├── workflow.rs             (WorkflowState, WorkflowStatus) ✅ EXISTS
├── session.rs              (SessionData, SessionStatus) ✅ EXISTS
└── artifact.rs             (Artifact, ArtifactId, ArtifactType) ✅ EXISTS
```

### 7.3 Backend Implementation Layer

**Implementations stay in `llmspell-storage/src/backends/{postgres,sqlite}/`**:

```
llmspell-storage/src/backends/
├── postgres/
│   ├── backend.rs          (impl StorageBackend for PostgresBackend)
│   ├── vector.rs           (impl VectorStorage for PostgresVectorStorage)
│   ├── graph.rs            (impl KnowledgeGraph for PostgresGraphStorage)
│   ├── procedural.rs       (impl ProceduralMemory for PostgresProceduralStorage)
│   ├── workflow_state.rs   (impl WorkflowStateStorage for PostgresWorkflowStateStorage)
│   ├── session.rs          (impl SessionStorage for PostgresSessionStorage)
│   └── artifact.rs         (impl ArtifactStorage for PostgresArtifactStorage)
│
└── sqlite/
    ├── backend.rs          (impl StorageBackend for SqliteBackend)
    ├── vector.rs           (impl VectorStorage for SqliteVectorStorage)
    ├── graph.rs            (impl KnowledgeGraph for SqliteGraphStorage)
    ├── procedural.rs       (impl ProceduralMemory for SqliteProceduralStorage)
    ├── workflow_state.rs   (impl WorkflowStateStorage for SqliteWorkflowStateStorage)
    ├── session.rs          (impl SessionStorage for SqliteSessionStorage)
    └── artifact.rs         (impl ArtifactStorage for SqliteArtifactStorage)
```

### 7.4 Runtime Injection Pattern

**Infrastructure module creates backends from config**:

```rust
// llmspell-infrastructure/src/storage.rs (or llmspell-kernel/src/infrastructure.rs)

pub struct StorageFactory;

impl StorageFactory {
    /// Create storage backends from configuration
    pub async fn from_config(config: &StorageConfig) -> Result<StorageBackends> {
        match config.backend_type {
            StorageBackendType::Sqlite => {
                let backend = Arc::new(SqliteBackend::new(config.sqlite.clone()).await?);
                Ok(StorageBackends::Sqlite(backend))
            }
            StorageBackendType::Postgres => {
                let backend = Arc::new(PostgresBackend::new(config.postgres.clone()).await?);
                Ok(StorageBackends::Postgres(backend))
            }
            StorageBackendType::Memory => {
                let backend = Arc::new(MemoryBackend::new());
                Ok(StorageBackends::Memory(backend))
            }
        }
    }
}

pub enum StorageBackends {
    Memory(Arc<MemoryBackend>),
    Sqlite(Arc<SqliteBackend>),
    #[cfg(feature = "postgres")]
    Postgres(Arc<PostgresBackend>),
}

// All backends implement all traits from llmspell-core
impl StorageBackends {
    pub fn as_storage_backend(&self) -> Arc<dyn StorageBackend> { /* ... */ }
    pub fn as_vector_storage(&self) -> Arc<dyn VectorStorage> { /* ... */ }
    pub fn as_graph_storage(&self) -> Arc<dyn KnowledgeGraph> { /* ... */ }
    pub fn as_procedural_storage(&self) -> Arc<dyn ProceduralMemory> { /* ... */ }
    // ... etc for all traits
}
```

**Usage in Kernel**:

```rust
// llmspell-kernel/src/state/manager.rs

pub struct StateManager {
    storage: Arc<dyn StorageBackend>,  // Injected via Infrastructure::from_config()
    // ...
}

impl StateManager {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self { storage }
    }
}

// llmspell-kernel/src/infrastructure.rs

pub struct Infrastructure {
    storage_backends: StorageBackends,
}

impl Infrastructure {
    pub async fn from_config(config: &Config) -> Result<Self> {
        let storage_backends = StorageFactory::from_config(&config.storage).await?;

        Ok(Self {
            storage_backends,
        })
    }

    pub fn create_state_manager(&self) -> StateManager {
        StateManager::new(self.storage_backends.as_storage_backend())
    }

    pub fn create_memory_manager(&self, embedding_service: Arc<EmbeddingService>) -> Result<MemoryManager> {
        MemoryManager::new(
            self.storage_backends.as_vector_storage(),      // Episodic
            self.storage_backends.as_graph_storage(),        // Semantic
            self.storage_backends.as_procedural_storage(),   // Procedural
            embedding_service,
        )
    }
}
```

---

## 8. Detailed Design

### 8.1 Trait Migration Checklist

| Trait | From | To | Types to Move | Estimated LOC |
|-------|------|----|--------------|--------------  |
| `StorageBackend` | llmspell-storage/src/traits.rs | llmspell-core/src/traits/storage/backend.rs | StorageBackendType, StorageCharacteristics | 120 |
| `VectorStorage` | llmspell-storage/src/vector_storage.rs | llmspell-core/src/traits/storage/vector.rs | VectorEntry, VectorQuery, VectorResult, DistanceMetric, ScoringMethod | 350 |
| `KnowledgeGraph` | llmspell-graph/src/traits/knowledge_graph.rs | llmspell-core/src/traits/storage/graph.rs | Entity, Relationship, TemporalQuery | 250 |
| `ProceduralMemory` | llmspell-memory/src/traits/procedural.rs | llmspell-core/src/traits/storage/procedural.rs | Pattern | 150 |
| **Total** | | | | **~870 lines** (traits) + **~2,500 lines** (types) = **~3,370 lines** |

### 8.2 Breaking Changes Analysis

**ZERO Breaking Changes** if done correctly:

1. **Re-export Strategy**: Old crates re-export from llmspell-core
   ```rust
   // llmspell-storage/src/traits.rs
   pub use llmspell_core::traits::storage::StorageBackend;

   // llmspell-graph/src/traits/knowledge_graph.rs
   pub use llmspell_core::traits::storage::KnowledgeGraph;
   ```

2. **Import Compatibility**: Existing code continues working
   ```rust
   // Old imports still work
   use llmspell_storage::StorageBackend;
   use llmspell_graph::KnowledgeGraph;

   // New imports also work
   use llmspell_core::traits::storage::{StorageBackend, KnowledgeGraph};
   ```

3. **Deprecation Path** (optional, post-refactor):
   ```rust
   #[deprecated(since = "0.14.0", note = "Import from llmspell_core::traits::storage instead")]
   pub use llmspell_core::traits::storage::StorageBackend;
   ```

### 8.3 Dependency Graph After Refactor

```
┌─────────────────┐
│  llmspell-core  │  ← ALL storage traits here (NO dependencies)
│  - StorageBackend
│  - VectorStorage
│  - KnowledgeGraph
│  - ProceduralMemory
│  - WorkflowStateStorage
│  - SessionStorage
│  - ArtifactStorage
└────────┬────────┘
         │
    ┌────┴──────────────────────────────────────┐
    │                                            │
┌───▼────────────┐                    ┌─────────▼────────┐
│ llmspell-      │                    │  llmspell-memory │
│ storage        │                    │  (domain layer)  │
│ (backends)     │                    └──────────────────┘
└────────────────┘                            │
    │                                         │
    ├─ PostgresBackend: StorageBackend       ├─ EpisodicMemory (uses VectorStorage)
    ├─ PostgresVectorStorage: VectorStorage  ├─ SemanticMemory (uses KnowledgeGraph)
    ├─ PostgresGraphStorage: KnowledgeGraph  └─ ProceduralMemory (direct trait)
    ├─ SqliteBackend: StorageBackend
    ├─ SqliteVectorStorage: VectorStorage
    └─ SqliteGraphStorage: KnowledgeGraph

┌─────────────────┐
│ llmspell-kernel │  ← Receives injected backends
└─────────────────┘
    │
    └─> Infrastructure::from_config() → StorageBackends → trait objects
```

**Key Improvement**: No more circular dependencies! All arrows point down.

---

## 9. Migration Strategy

### 9.1 Phase 1: Move Traits to llmspell-core (Week 1)

**Tasks**:
1. Create `llmspell-core/src/traits/storage/{backend,vector,graph,procedural}.rs`
2. Move trait definitions (copy-paste, preserve all doc comments)
3. Move domain types to `llmspell-core/src/types/storage/`
4. Update `llmspell-core/src/traits/mod.rs` and `llmspell-core/src/types/mod.rs` exports
5. Run `cargo check -p llmspell-core` (must compile with zero deps)

**Success Criteria**:
- llmspell-core compiles with all new traits
- Zero external dependencies added to llmspell-core
- All traits have comprehensive doc comments

### 9.2 Phase 2: Update Implementations (Week 2)

**Tasks - ALL 11 Impacted Crates**:

1. **llmspell-storage** (CRITICAL):
   - Update imports in `src/backends/postgres/*.rs` (10 files)
   - Update imports in `src/backends/sqlite/*.rs` (10 files)
   - Add re-exports in `src/traits.rs` and `src/vector_storage.rs`
   - Update `Cargo.toml` dependencies

2. **llmspell-kernel** (CRITICAL):
   - Update `src/state/manager.rs` imports
   - Update `src/state/backend_adapter.rs` imports
   - Update `src/state/vector_storage.rs` (remove duplication!)
   - 12+ files affected

3. **llmspell-bridge** (CRITICAL):
   - Update `src/infrastructure.rs` imports (SqliteBackend, SqliteVectorStorage)
   - Update `src/rag_bridge.rs` imports
   - Update `src/memory_bridge.rs` imports
   - Update `src/state_adapter.rs` imports
   - Update `src/globals/*.rs` (3 files)
   - **9+ files affected** ⚠️

4. **llmspell-memory** (HIGH):
   - Update `src/manager.rs` imports
   - Update `src/consolidation/*.rs` imports (2 files)
   - Update trait re-exports if ProceduralMemory moves to core
   - 5+ files affected

5. **llmspell-rag** (HIGH):
   - Update `src/traits/hybrid.rs` (extends VectorStorage)
   - Update `src/pipeline/*.rs` (4 files)
   - Update `src/state_integration.rs`
   - Update `src/session_integration.rs`
   - **8+ files affected** ⚠️

6. **llmspell-tenancy** (MEDIUM):
   - Update `src/manager.rs` (implements VectorStorage)
   - Update test files (3 files)

7. **llmspell-agents** (MEDIUM):
   - Update `src/registry/persistence.rs`
   - Update test files

8. **llmspell-events** (MEDIUM):
   - Update `src/storage_adapter.rs`
   - Already uses adapter pattern, minimal changes

9. **llmspell-hooks** (MEDIUM):
   - Update hook persistence code
   - Minimal changes (uses StorageBackend only)

10. **llmspell-graph** (LOW):
    - Add re-export for `KnowledgeGraph` trait
    - Update documentation

11. **llmspell-context** (LOW):
    - No direct changes (indirect via memory/graph)
    - Verify compilation

**Re-export Strategy** (zero breaking changes):
```rust
// llmspell-storage/src/traits.rs
#[deprecated(since = "0.14.0", note = "Import from llmspell_core::traits::storage instead")]
pub use llmspell_core::traits::storage::StorageBackend;

// llmspell-storage/src/vector_storage.rs
#[deprecated(since = "0.14.0", note = "Import from llmspell_core::traits::storage instead")]
pub use llmspell_core::traits::storage::VectorStorage;

// llmspell-graph/src/traits/knowledge_graph.rs
#[deprecated(since = "0.14.0", note = "Import from llmspell_core::traits::storage instead")]
pub use llmspell_core::traits::storage::KnowledgeGraph;

// llmspell-memory/src/traits/procedural.rs
#[deprecated(since = "0.14.0", note = "Import from llmspell_core::traits::storage instead")]
pub use llmspell_core::traits::storage::ProceduralMemory;
```

**Success Criteria**:
- All 11 crates compile successfully
- All existing imports still work (re-exports functional)
- Zero clippy warnings across workspace
- All 149+ tests passing
- Benchmark performance maintained (<5% variance)

### 9.3 Phase 3: Runtime Injection (Week 3)

**Tasks**:
1. Create `StorageFactory::from_config()` in Infrastructure module
2. Create `StorageBackends` enum with trait object accessors
3. Update `StateManager` to receive `Arc<dyn StorageBackend>` in constructor
4. Update `MemoryManager` to receive trait objects for all 3 memory types
5. Update `llmspell-kernel/src/infrastructure.rs` to use factory pattern

**Success Criteria**:
- Config-driven backend selection working
- No more inline `SqliteBackend::new()` or `PostgresBackend::new()` in kernel
- All tests passing with new injection pattern

### 9.4 Phase 4: Documentation & Cleanup (Week 4)

**Tasks**:
1. Update `docs/technical/current-architecture.md` with new trait architecture
2. Create `docs/technical/storage-trait-architecture.md` comprehensive guide
3. Update `docs/developer-guide/03-extending-components.md` with new patterns
4. Add migration guide for downstream users (if any)
5. Remove any deprecated re-exports after validation

**Success Criteria**:
- Documentation reflects new architecture
- Developer guide shows how to implement custom backends
- Zero TODOs or FIXMEs in migrated code

---

## 10. Risk Analysis

### 10.1 High Risks (Mitigation Required)

**Risk 1: Type Parameter Mismatches**
- **Impact**: Compile errors if trait bounds don't match
- **Mitigation**: Comprehensive type checking, preserve all `Send + Sync` bounds
- **Detection**: `cargo check --workspace --all-features`

**Risk 2: Doc Test Breakage**
- **Impact**: Rustdoc examples fail after imports change
- **Mitigation**: Update all doc test imports to use llmspell-core
- **Detection**: `cargo test --doc --workspace`

**Risk 3: Integration Test Failures**
- **Impact**: Tests break due to missing re-exports
- **Mitigation**: Keep re-exports until all tests pass, deprecate later
- **Detection**: `cargo test --workspace --all-features`

### 10.2 Medium Risks (Monitor)

**Risk 4: Performance Regression**
- **Impact**: Trait object overhead from dynamic dispatch
- **Mitigation**: Benchmark before/after, use `#[inline]` where needed
- **Detection**: `cargo bench --workspace`

**Risk 5: Circular Dependency Creep**
- **Impact**: New code accidentally adds deps to llmspell-core
- **Mitigation**: CI check for llmspell-core dependency count
- **Detection**: `cargo tree -p llmspell-core | grep dependencies`

### 10.3 Low Risks (Accept)

**Risk 6: Documentation Drift**
- **Impact**: Docs reference old import paths
- **Mitigation**: Deprecation warnings guide users to new imports
- **Acceptance**: Pre-1.0, breaking changes acceptable

---

## 11. Implementation Plan

### 11.1 Task Breakdown for Phase 13c.3.0

**Task 13c.3.0: Centralized Trait Architecture Design & Setup**
- **Estimated Time**: 8 hours
- **Deliverables**:
  1. Create trait files in `llmspell-core/src/traits/storage/`
  2. Create type files in `llmspell-core/src/types/storage/`
  3. Migrate `StorageBackend`, `VectorStorage`, `KnowledgeGraph`, `ProceduralMemory` trait definitions
  4. Compile llmspell-core in isolation (zero external deps)
  5. Document new architecture in this file

**Task 13c.3.1: Backend Implementation Updates**
- **Estimated Time**: 12 hours
- **Deliverables**:
  1. Update all `llmspell-storage` imports
  2. Update `llmspell-memory` imports
  3. Update `llmspell-graph` imports (keep crate, re-export trait)
  4. Add re-exports for backward compatibility
  5. Fix any compilation errors
  6. Run full test suite

**Task 13c.3.2: Runtime Injection Pattern** (Existing task, now aligned)
- **Estimated Time**: 16 hours (existing 8h + 8h for factory pattern)
- **Deliverables**:
  1. Create `StorageFactory::from_config()`
  2. Create `StorageBackends` enum
  3. Update `Infrastructure::from_config()` to use factory
  4. Update StateManager, MemoryManager constructors
  5. PostgreSQL ↔ SQLite export/import tool (existing requirement)
  6. Full data roundtrip tests

**Task 13c.3.3: Documentation & Validation** (Existing task, aligned)
- **Estimated Time**: 8 hours (existing)
- **Deliverables**:
  1. Update `current-architecture.md`
  2. Create `storage-trait-architecture.md`
  3. Update developer guide
  4. Run quality gates
  5. Benchmark validation

### 11.2 Success Metrics

- [ ] **Zero breaking changes**: All Phase 1-13 APIs work unchanged
- [ ] **Zero circular dependencies**: `cargo tree` shows clean graph
- [ ] **100% test coverage**: All 149+ tests passing
- [ ] **Zero clippy warnings**: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- [ ] **Performance maintained**: Benchmarks show <5% overhead
- [ ] **Documentation complete**: All traits documented with examples

---

## Appendices

### Appendix A: Full Trait Method Inventory

**(See detailed method signatures in sections 3.1-3.2 above)**

### Appendix B: Migration File Matrix

| PostgreSQL Migration | SQLite Migration | Schema Compatibility | Notes |
|---------------------|------------------|----------------------|-------|
| V1 (initial_setup) | V1 (initial_setup) | ✅ Compatible | Both create _migrations table |
| V2 (test_table_rls) | ❌ Missing | ⚠️ PostgreSQL-only | RLS testing table |
| V3 (vector_embeddings) | V3 (vector_embeddings) | ✅ Compatible | Different HNSW extensions |
| V4 (temporal_graph) | V4 (temporal_graph) | ✅ Compatible | tstzrange → INTEGER conversion |
| V5 (procedural_memory) | V5 (procedural_memory) | ✅ Compatible | UPSERT syntax differs |
| V6 (agent_state) | V6 (agent_state) | ✅ Compatible | JSONB → TEXT |
| V7 (kv_store) | V7 (kv_store) | ✅ Compatible | Generic key-value |
| V8 (workflow_states) | V8 (workflow_states) | ✅ Compatible | ENUM → TEXT with CHECK |
| V9 (sessions) | V9 (sessions) | ✅ Compatible | TIMESTAMPTZ → INTEGER |
| V10 (artifacts) | V10 (artifacts) | ✅ Compatible | Large Objects → BLOB |
| V11 (event_log) | V11 (event_log) | ✅ Compatible | Partitioning → single table |
| V12 (application_role_rls_enforcement) | ❌ Missing | ⚠️ PostgreSQL-only | RLS enforcement |
| V13 (hook_history) | V13 (hook_history) | ✅ Compatible | Correlation tracking |
| V14 (api_keys) | ❌ Missing | ⚠️ To be implemented | API key storage |
| V15 (bitemporal_composite_keys) | ❌ Missing | ⚠️ To be implemented | Composite key optimization |

### Appendix C: Code Examples

#### Example 1: Migrating VectorStorage Trait

**Before** (llmspell-storage/src/vector_storage.rs):
```rust
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait VectorStorage: Send + Sync {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;
    // ... 8 more methods
}
```

**After** (llmspell-core/src/traits/storage/vector.rs):
```rust
use anyhow::Result;
use async_trait::async_trait;
use crate::types::storage::{VectorEntry, VectorQuery, VectorResult};

#[async_trait]
pub trait VectorStorage: Send + Sync {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;
    // ... 8 more methods
}
```

**Backward Compatibility** (llmspell-storage/src/vector_storage.rs):
```rust
pub use llmspell_core::traits::storage::VectorStorage;
pub use llmspell_core::types::storage::{VectorEntry, VectorQuery, VectorResult};
```

#### Example 2: Runtime Injection Pattern

**Before** (kernel constructs backend inline):
```rust
// llmspell-kernel/src/state/manager.rs
impl StateManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let backend = match config.storage.backend {
            StorageBackendType::Sqlite => {
                Arc::new(SqliteBackend::new(&config.storage.sqlite).await?)
            }
            StorageBackendType::Postgres => {
                Arc::new(PostgresBackend::new(&config.storage.postgres).await?)
            }
        };

        Ok(Self {
            storage: backend,
            // ...
        })
    }
}
```

**After** (kernel receives injected backend):
```rust
// llmspell-kernel/src/state/manager.rs
impl StateManager {
    pub fn new(storage: Arc<dyn StorageBackend>) -> Self {
        Self {
            storage,
            // ...
        }
    }
}

// llmspell-kernel/src/infrastructure.rs
impl Infrastructure {
    pub async fn from_config(config: &Config) -> Result<Self> {
        let storage_backends = StorageFactory::from_config(&config.storage).await?;

        let state_manager = StateManager::new(
            storage_backends.as_storage_backend()
        );

        Ok(Self {
            storage_backends,
            state_manager,
            // ...
        })
    }
}
```

---

## Conclusion

This analysis provides a comprehensive roadmap for centralizing storage traits in `llmspell-core`, eliminating circular dependencies, and establishing a clean runtime injection pattern. The refactor maintains **zero breaking changes** to existing APIs while setting up a foundation for easier maintenance and future storage backend additions.

**Recommended Approach**:
1. **Phase 1**: Trait migration (Week 1) - Low risk, high value
2. **Phase 2**: Implementation updates (Week 2) - Medium effort, comprehensive testing
3. **Phase 3**: Runtime injection (Week 3) - Architectural improvement, enables dynamic backend selection
4. **Phase 4**: Documentation (Week 4) - Consolidate knowledge, guide future development

**Total Effort**: 4 weeks (44 hours estimated)
**Risk Level**: LOW (with proper re-exports and testing)
**Value**: HIGH (architectural clarity, maintainability, extensibility)
