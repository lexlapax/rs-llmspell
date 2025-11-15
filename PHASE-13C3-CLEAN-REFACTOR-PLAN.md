# Phase 13c.3: CLEAN Refactor Plan - Zero Re-exports, Breaking Changes OK

**Status**: Comprehensive Analysis Complete
**Date**: November 2025
**Approach**: Clean architectural refactor with NO re-exports
**Breaking Changes**: ACCEPTED (pre-1.0)

---

## Executive Summary

### Clean Refactor Philosophy

**NO HALF-MEASURES**: Instead of re-exports that maintain backward compatibility, this plan implements a clean break where:
1. All storage traits move to `llmspell-core` (single source of truth)
2. Every import statement updates to use `llmspell_core::traits::storage::*`
3. Zero re-exports in old locations - clean removal
4. All documentation, tests, and examples updated
5. Breaking changes throughout - clean slate for v0.14.0

### Comprehensive Scope

After exhaustive analysis across **all 1,141 Rust source files**:

| Category | Count | Needs Update |
|----------|-------|--------------|
| **Rust source files** (total) | 1,141 | - |
| **Files with storage imports** | 149 | ✅ ALL |
| - Source files (non-test) | 86 | ✅ ALL |
| - Test files | 77 | ✅ ALL |
| **Total import statements** | 374 | ✅ ALL |
| **Markdown files with traits** | 48 | ✅ ALL |
| **Crate README files** | 18 | ✅ 11 (storage-related) |
| **Example scripts** | 69 | ⚠️ Review (Lua/JS/Python) |
| **Rustdoc comments** | 20+ | ✅ ALL |

### Effort Estimation

| Phase | Duration | Files Changed | Lines Changed |
|-------|----------|---------------|---------------|
| 1. Trait Migration | 3 days | 10 | ~3,500 (new code) |
| 2. Backend Imports | 5 days | 86 | ~500 (imports) |
| 3. Test Updates | 4 days | 77 | ~250 (imports) |
| 4. Documentation | 3 days | 48 | ~200 (examples) |
| 5. README Updates | 2 days | 11 | ~150 (examples) |
| 6. Rustdoc Fixes | 2 days | 30+ | ~50 (doc comments) |
| 7. Integration Testing | 3 days | - | - (validation) |
| **TOTAL** | **22 days (~4.5 weeks)** | **~250 files** | **~4,700 lines** |

---

## Table of Contents

### Part I: Architectural Analysis
1. [Detailed Scope Analysis](#1-detailed-scope-analysis)
2. [Storage Component Inventory](#2-storage-component-inventory)
3. [Backend Implementation Matrix](#3-backend-implementation-matrix)
4. [Trait Method Inventory](#4-trait-method-inventory)
5. [Runtime Injection Architecture](#5-runtime-injection-architecture)

### Part II: Execution Plan
6. [File-by-File Breakdown](#6-file-by-file-breakdown)
7. [Import Pattern Changes](#7-import-pattern-changes)
8. [Documentation Updates](#8-documentation-updates)
9. [Test Infrastructure](#9-test-infrastructure)
10. [Migration Sequence](#10-migration-sequence)
11. [Validation Strategy](#11-validation-strategy)
12. [Risk Mitigation](#12-risk-mitigation)
13. [Success Metrics](#13-success-metrics)

---

## 1. Detailed Scope Analysis

### 1.1 Source Files by Crate (Non-Test)

| Crate | Files with Imports | Traits Used | Complexity |
|-------|-------------------|-------------|------------|
| **llmspell-storage** | 20 | All (implements) | CRITICAL |
| **llmspell-kernel** | 12 | StorageBackend, VectorStorage | CRITICAL |
| **llmspell-bridge** | 9 | StorageBackend, VectorStorage, MemoryManager | CRITICAL |
| **llmspell-memory** | 15 | VectorStorage, KnowledgeGraph, ProceduralMemory | HIGH |
| **llmspell-rag** | 8 | VectorStorage | HIGH |
| **llmspell-tenancy** | 3 | VectorStorage | MEDIUM |
| **llmspell-agents** | 2 | StorageBackend | MEDIUM |
| **llmspell-events** | 3 | StorageBackend | MEDIUM |
| **llmspell-hooks** | 1 | StorageBackend | LOW |
| **llmspell-context** | 3 | Indirect (via memory/graph) | LOW |
| **llmspell-graph** | 5 | KnowledgeGraph (defines) | MEDIUM |
| **llmspell-templates** | 2 | Via MemoryManager | LOW |
| **llmspell-testing** | 1 | StorageBackend | LOW |
| **TOTAL** | **86** | - | - |

### 1.2 Test Files by Category

| Category | Count | Import Lines | Effort |
|----------|-------|--------------|--------|
| **Unit tests** (llmspell-storage) | 38 | ~120 | HIGH |
| **Integration tests** (llmspell-bridge) | 12 | ~35 | HIGH |
| **Memory system tests** | 10 | ~25 | MEDIUM |
| **RAG tests** | 5 | ~15 | MEDIUM |
| **Tenancy tests** | 4 | ~12 | LOW |
| **Other crate tests** | 8 | ~23 | LOW |
| **TOTAL** | **77** | **~230** | - |

### 1.3 Documentation Files

| Type | Count | Needs Update | Examples to Fix |
|------|-------|--------------|-----------------|
| **Technical docs** (docs/technical/) | 15 | ✅ ALL | ~30 code blocks |
| **Developer guide** (docs/developer-guide/) | 8 | ✅ ALL | ~20 code blocks |
| **User guide** (docs/user-guide/) | 3 | ⚠️ Some | ~5 code blocks |
| **In-progress docs** (docs/in-progress/) | 12 | ⚠️ Archive | ~25 code blocks |
| **Phase design docs** | 6 | ⚠️ Historical | ~15 code blocks |
| **Crate READMEs** | 11 | ✅ ALL | ~40 code blocks |
| **Root README** | 1 | ✅ YES | ~5 code blocks |
| **TOTAL** | **56** | ~40 critical | **~140 code blocks** |

### 1.4 Example Scripts (Lua/JS/Python)

**Finding**: 69 example scripts exist, but storage traits are NOT directly exposed to scripts!

**Analysis**:
- Lua/JS/Python use **bridge layer** APIs (globals like `Memory`, `RAG`, `State`)
- Bridge internals use storage traits, but scripts don't import them
- **Impact**: ZERO changes to example scripts (bridge layer abstracts storage)

**Validation**: ✅ Confirmed by checking `llmspell-bridge/src/globals/` - no trait exposure

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

**Key Observations**:
- **10 components** with 100% PostgreSQL/SQLite feature parity
- **3 traits already in llmspell-core** (Workflow, Session, Artifact) ✅
- **4 traits to migrate** (StorageBackend, VectorStorage, KnowledgeGraph, ProceduralMemory)
- **3 traits stay in domain crates** (EpisodicMemory, SemanticMemory, EventStorage)

### 2.2 Implementation File Structure

```
llmspell-storage/src/backends/
├── postgres/
│   ├── backend.rs          (PostgresBackend: StorageBackend)
│   ├── vector.rs           (PostgresVectorStorage: VectorStorage)
│   ├── graph.rs            (PostgresGraphStorage: KnowledgeGraph)
│   ├── procedural.rs       (PostgresProceduralStorage: ProceduralMemory)
│   ├── workflow_state.rs   (PostgresWorkflowStateStorage: WorkflowStateStorage)
│   ├── session.rs          (PostgresSessionStorage: SessionStorage)
│   ├── artifacts.rs        (PostgresArtifactStorage: ArtifactStorage)
│   ├── event_log.rs        (PostgresEventLogStorage: EventStorage)
│   └── hook_history.rs     (PostgresHookHistoryStorage: StorageBackend)
│
└── sqlite/
    ├── backend.rs          (SqliteBackend: StorageBackend)
    ├── vector.rs           (SqliteVectorStorage: VectorStorage)
    ├── graph.rs            (SqliteGraphStorage: KnowledgeGraph)
    ├── procedural.rs       (SqliteProceduralStorage: ProceduralMemory)
    ├── agent_state.rs      (SqliteAgentStateStorage: StorageBackend)
    ├── kv_store.rs         (SqliteKVStorage: StorageBackend)
    ├── workflow_state.rs   (SqliteWorkflowStateStorage: WorkflowStateStorage)
    ├── session.rs          (SqliteSessionStorage: SessionStorage)
    ├── artifact.rs         (SqliteArtifactStorage: ArtifactStorage)
    ├── event_log.rs        (SqliteEventLogStorage: EventStorage)
    └── hook_history.rs     (SqliteHookHistoryStorage: StorageBackend)
```

**Current state**: These backend implementations are in `llmspell-storage`, but the traits they implement (`KnowledgeGraph`, `ProceduralMemory`) are currently scattered across other crates. The refactor will centralize all traits in `llmspell-core`.


---

## 3. Backend Implementation Matrix

### 3.1 Feature Parity Matrix

| Feature | PostgreSQL | SQLite | Notes |
|---------|-----------|--------|-------|
| **Vector Search** | VectorChord HNSW | vectorlite-rs HNSW | Both use HNSW indexing |
| **Vector Dimensions** | 384, 768, 1536, 3072 | 384, 768, 1536, 3072 | Identical support |
| **Graph Bi-Temporal** | tstzrange (native) | INTEGER start/end | Different representation, same semantics |
| **Graph Traversal** | Recursive CTEs | Recursive CTEs | SQLite has native CTE support |
| **Procedural Patterns** | UPSERT with ON CONFLICT | INSERT OR REPLACE | Syntax differs, functionality identical |
| **Agent State** | JSONB | TEXT (json_extract) | SQLite json1 extension equivalent |
| **Workflow States** | ENUM types | TEXT with CHECK | No ENUMs in SQLite, use TEXT |
| **Sessions** | Expiration via TIMESTAMPTZ | Expiration via INTEGER (Unix) | Both support cleanup_expired() |
| **Artifacts** | Large Objects (>1MB) | BLOB inline (all sizes) | PostgreSQL optimizes >1MB separately |
| **Event Log** | Monthly partitioning | Single table | Performance difference at scale |
| **Hook History** | Correlation tracking | Correlation tracking | Identical interface |
| **Multi-Tenancy** | RLS (Row-Level Security) | Application-level WHERE | PostgreSQL enforces at DB level |

**Conclusion**: **100% feature parity** achieved in Phase 13c.2!

---

## 4. Trait Method Inventory

### 4.1 StorageBackend Trait (Generic KV Storage)

**Location**: `llmspell-core/src/traits/storage/backend.rs` (after migration)

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    // Core KV operations
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn exists(&self, key: &str) -> Result<bool>;

    // Batch operations
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;
    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>;
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;

    // Administrative
    async fn clear(&self) -> Result<()>;
    fn backend_type(&self) -> StorageBackendType;
    fn characteristics(&self) -> StorageCharacteristics;

    // Migration support
    async fn run_migrations(&self) -> Result<()>;
    async fn migration_version(&self) -> Result<usize>;
}
```

**Methods**: 13 total
**Complexity**: LOW - simple KV interface
**Implementations**: MemoryBackend, SqliteBackend, PostgresBackend

### 4.2 VectorStorage Trait (Semantic Search)

**Location**: `llmspell-core/src/traits/storage/vector.rs` (after migration)

```rust
#[async_trait]
pub trait VectorStorage: Send + Sync {
    // Core vector operations
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>;
    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>;
    async fn get(&self, id: &str) -> Result<Option<VectorEntry>>;
    async fn delete(&self, id: &str) -> Result<()>;

    // Metadata operations
    async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()>;

    // Tenant operations
    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>;
    async fn delete_scope(&self, scope: &StateScope) -> Result<usize>;

    // Bulk operations
    async fn clear(&self) -> Result<()>;
    async fn count(&self) -> Result<usize>;
    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<VectorEntry>>;
}
```

**Methods**: 10 total
**Complexity**: MEDIUM - vector search with metadata
**Implementations**: SqliteVectorStorage, PostgresVectorStorage
**Decorators**: StateAwareVectorStorage (llmspell-rag), MultiTenantVectorManager (llmspell-tenancy)

### 4.3 KnowledgeGraph Trait (Bi-Temporal Graph)

**Location**: `llmspell-core/src/traits/storage/graph.rs` (after migration)

```rust
#[async_trait]
pub trait KnowledgeGraph: Send + Sync {
    // Entity operations
    async fn add_entity(&self, entity: Entity) -> Result<String>;
    async fn update_entity(&self, id: &str, changes: HashMap<String, Value>) -> Result<()>;
    async fn get_entity(&self, id: &str) -> Result<Entity>;
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>;

    // Relationship operations
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;
    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>>;
    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    // Query operations
    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;
    async fn traverse(
        &self,
        start_entity: &str,
        relationship_type: Option<&str>,
        max_depth: usize,
        at_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(Entity, usize, String)>>;

    // Cleanup
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;
}
```

**Methods**: 10 total
**Complexity**: HIGH - bi-temporal graph with traversal
**Implementations**: SqliteGraphStorage, PostgresGraphStorage, SurrealDBBackend (legacy)

### 4.4 ProceduralMemory Trait (Pattern Learning)

**Location**: `llmspell-core/src/traits/storage/procedural.rs` (after migration)

```rust
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    // Pattern tracking
    async fn record_transition(
        &self,
        scope: &str,
        key: &str,
        from_value: Option<&str>,
        to_value: &str,
    ) -> Result<u32>;

    async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32>;
    async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<Pattern>>;

    // Legacy placeholders (deprecated)
    async fn get_pattern(&self, id: &str) -> Result<()>;
    async fn store_pattern(&self, pattern: &str) -> Result<String>;
}
```

**Methods**: 5 total
**Complexity**: LOW - simple frequency tracking
**Implementations**: SqliteProceduralStorage, PostgresProceduralStorage

---

## 5. Runtime Injection Architecture

### 5.1 Current Problem: Inline Backend Creation

**Current pattern** (scattered across codebase):
```rust
// llmspell-bridge/src/infrastructure.rs (creates backend inline)
let config = SqliteConfig::new("./data/storage.db");
let backend = Arc::new(SqliteBackend::new(config).await?);
let vector_storage = Arc::new(SqliteVectorStorage::new(backend, 384).await?);

// llmspell-kernel/src/state/manager.rs (creates backend inline)
let backend = match config.backend {
    BackendType::Sqlite => Arc::new(SqliteBackend::new(&config.sqlite).await?),
    BackendType::Postgres => Arc::new(PostgresBackend::new(&config.postgres).await?),
};
```

**Problems**:
1. Backend creation logic duplicated across crates
2. No centralized configuration
3. Difficult to swap backends at runtime
4. Test code duplicates production setup

### 5.2 Proposed Solution: StorageFactory Pattern

**Create centralized factory** in `llmspell-storage` or `llmspell-kernel`:

```rust
// llmspell-storage/src/factory.rs (NEW)
use llmspell_core::traits::storage::{StorageBackend, VectorStorage, KnowledgeGraph, ProceduralMemory};
use std::sync::Arc;

pub struct StorageFactory;

impl StorageFactory {
    /// Create all storage backends from configuration
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

/// Enum wrapping all backend types
pub enum StorageBackends {
    Memory(Arc<MemoryBackend>),
    Sqlite(Arc<SqliteBackend>),
    #[cfg(feature = "postgres")]
    Postgres(Arc<PostgresBackend>),
}

impl StorageBackends {
    /// Get StorageBackend trait object
    pub fn as_storage_backend(&self) -> Arc<dyn StorageBackend> {
        match self {
            Self::Memory(b) => Arc::clone(b) as Arc<dyn StorageBackend>,
            Self::Sqlite(b) => Arc::clone(b) as Arc<dyn StorageBackend>,
            #[cfg(feature = "postgres")]
            Self::Postgres(b) => Arc::clone(b) as Arc<dyn StorageBackend>,
        }
    }

    /// Get VectorStorage trait object
    pub async fn as_vector_storage(&self, dimension: usize) -> Result<Arc<dyn VectorStorage>> {
        match self {
            Self::Sqlite(b) => {
                Ok(Arc::new(SqliteVectorStorage::new(Arc::clone(b), dimension).await?) as Arc<dyn VectorStorage>)
            }
            #[cfg(feature = "postgres")]
            Self::Postgres(b) => {
                Ok(Arc::new(PostgresVectorStorage::new(Arc::clone(b), dimension).await?) as Arc<dyn VectorStorage>)
            }
            Self::Memory(_) => {
                Ok(Arc::new(InMemoryVectorStorage::new(dimension)) as Arc<dyn VectorStorage>)
            }
        }
    }

    /// Get KnowledgeGraph trait object
    pub fn as_knowledge_graph(&self) -> Arc<dyn KnowledgeGraph> {
        match self {
            Self::Sqlite(b) => Arc::new(SqliteGraphStorage::new(Arc::clone(b))) as Arc<dyn KnowledgeGraph>,
            #[cfg(feature = "postgres")]
            Self::Postgres(b) => Arc::new(PostgresGraphStorage::new(Arc::clone(b))) as Arc<dyn KnowledgeGraph>,
            Self::Memory(_) => Arc::new(InMemoryGraphStorage::new()) as Arc<dyn KnowledgeGraph>,
        }
    }

    /// Get ProceduralMemory trait object
    pub fn as_procedural_memory(&self) -> Arc<dyn ProceduralMemory> {
        match self {
            Self::Sqlite(b) => Arc::new(SqliteProceduralStorage::new(Arc::clone(b))) as Arc<dyn ProceduralMemory>,
            #[cfg(feature = "postgres")]
            Self::Postgres(b) => Arc::new(PostgresProceduralStorage::new(Arc::clone(b))) as Arc<dyn ProceduralMemory>,
            Self::Memory(_) => Arc::new(InMemoryProceduralStorage::new()) as Arc<dyn ProceduralMemory>,
        }
    }
}
```

### 5.3 Usage in Kernel (After Refactor)

```rust
// llmspell-kernel/src/infrastructure.rs
pub struct Infrastructure {
    storage_backends: StorageBackends,
    state_manager: Arc<StateManager>,
    memory_manager: Option<Arc<MemoryManager>>,
}

impl Infrastructure {
    pub async fn from_config(config: &Config) -> Result<Self> {
        // Single creation point
        let storage_backends = StorageFactory::from_config(&config.storage).await?;

        // Inject into components
        let state_manager = StateManager::new(
            storage_backends.as_storage_backend()
        );

        let memory_manager = if config.memory.enabled {
            Some(Arc::new(MemoryManager::new(
                storage_backends.as_vector_storage(384).await?,  // Episodic
                storage_backends.as_knowledge_graph(),            // Semantic
                storage_backends.as_procedural_memory(),          // Procedural
                embedding_service,
            )?))
        } else {
            None
        };

        Ok(Self {
            storage_backends,
            state_manager: Arc::new(state_manager),
            memory_manager,
        })
    }
}
```

### 5.4 Benefits of Factory Pattern

| Benefit | Description |
|---------|-------------|
| **Single creation point** | All backend instantiation in one place |
| **Config-driven** | Runtime backend selection via config |
| **Test-friendly** | Easy to inject test backends |
| **Type-safe** | Enum ensures all backends implement all traits |
| **Centralized** | Easy to add new backends in the future |

---

## 6. File-by-File Breakdown

### 6.1 Critical Path: llmspell-storage (20 files)

#### Backend Implementations

| File | Current Imports | New Imports | Lines Changed |
|------|----------------|-------------|---------------|
| `src/backends/postgres/backend.rs` | `use crate::traits::StorageBackend` | `use llmspell_core::traits::storage::StorageBackend` | 3-5 |
| `src/backends/postgres/vector.rs` | `use crate::vector_storage::*` | `use llmspell_core::traits::storage::*` | 5-8 |
| `src/backends/postgres/graph.rs` | `use llmspell_graph::KnowledgeGraph` | `use llmspell_core::traits::storage::KnowledgeGraph` | 2-3 |
| `src/backends/sqlite/backend.rs` | `use crate::traits::StorageBackend` | `use llmspell_core::traits::storage::StorageBackend` | 3-5 |
| `src/backends/sqlite/vector.rs` | `use crate::vector_storage::*` | `use llmspell_core::traits::storage::*` | 5-8 |
| `src/backends/sqlite/graph.rs` | `use llmspell_graph::KnowledgeGraph` | `use llmspell_core::traits::storage::KnowledgeGraph` | 2-3 |
| `src/backends/sqlite/procedural.rs` | `use llmspell_memory::ProceduralMemory` | `use llmspell_core::traits::storage::ProceduralMemory` | 2-3 |
| `src/backends/sqlite/workflow_state.rs` | `use llmspell_core::traits::storage::*` | ✅ Already correct! | 0 |
| `src/backends/sqlite/session.rs` | `use llmspell_core::traits::storage::*` | ✅ Already correct! | 0 |
| `src/backends/sqlite/artifact.rs` | `use llmspell_core::traits::storage::*` | ✅ Already correct! | 0 |
| ... | (10 more files) | Similar patterns | ~30 total |

#### Type Exports

| File | Action | Complexity |
|------|--------|------------|
| `src/traits.rs` | **DELETE** (moved to core) | HIGH - remove 120 lines |
| `src/vector_storage.rs` | **DELETE** (moved to core) | HIGH - remove 600+ lines |
| `src/lib.rs` | Update exports | MEDIUM - remove old, add new re-exports from core |

### 6.2 Critical Path: llmspell-bridge (9+ files)

| File | Current Pattern | New Pattern | Impact |
|------|----------------|-------------|--------|
| `src/infrastructure.rs` | `use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage}` | `use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage}`<br>`use llmspell_core::traits::storage::VectorStorage` | Import split |
| `src/rag_bridge.rs` | `use llmspell_storage::{VectorEntry, VectorResult, VectorStorage}` | `use llmspell_core::traits::storage::VectorStorage`<br>`use llmspell_core::types::storage::{VectorEntry, VectorResult}` | Import reorganization |
| `src/memory_bridge.rs` | `use llmspell_memory::{...}` | Indirect - memory crate updates | Low direct impact |
| `src/globals/rag_infrastructure.rs` | Similar to infrastructure.rs | Same pattern | Import reorganization |

### 6.3 High-Impact: llmspell-memory (15 files)

| File | Traits Used | Change Type |
|------|-------------|-------------|
| `src/manager.rs` | EpisodicMemory, SemanticMemory, ProceduralMemory | Indirect (memory crate wraps storage) |
| `src/episodic/sqlite_backend.rs` | VectorStorage | Direct import change |
| `src/consolidation/validator.rs` | KnowledgeGraph | `use llmspell_core::traits::storage::KnowledgeGraph` |
| `src/consolidation/llm_engine.rs` | KnowledgeGraph | Same |
| `src/semantic.rs` | KnowledgeGraph | Same |
| ... | (10 more files) | Similar patterns |

### 6.4 Test Files - Storage Crate (38 files!)

**Pattern**: All test files create backends directly

```rust
// BEFORE (38 test files with similar pattern)
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_storage::{VectorEntry, VectorQuery, VectorStorage};

// AFTER
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_core::traits::storage::VectorStorage;
use llmspell_core::types::storage::{VectorEntry, VectorQuery};
```

**Files** (partial list):
- `tests/postgres_vector_tests.rs`
- `tests/postgres_knowledge_graph_tests.rs`
- `tests/sqlite_vector_tests.rs`
- `tests/sqlite_workflow_state_tests.rs`
- `tests/postgres_procedural_memory_tests.rs`
- ... (33 more files)

**Effort**: 2-3 import lines per file × 38 files = ~100 lines changed

---

## 7. Import Pattern Changes

### 7.1 Before → After Patterns

#### Pattern 1: VectorStorage Trait
```rust
// BEFORE
use llmspell_storage::VectorStorage;
use llmspell_storage::{VectorEntry, VectorQuery, VectorResult};

// AFTER
use llmspell_core::traits::storage::VectorStorage;
use llmspell_core::types::storage::{VectorEntry, VectorQuery, VectorResult};
```

#### Pattern 2: KnowledgeGraph Trait
```rust
// BEFORE
use llmspell_graph::KnowledgeGraph;
use llmspell_graph::types::{Entity, Relationship};

// AFTER
use llmspell_core::traits::storage::KnowledgeGraph;
use llmspell_core::types::storage::{Entity, Relationship, TemporalQuery};
```

#### Pattern 3: StorageBackend Trait
```rust
// BEFORE
use llmspell_storage::{StorageBackend, StorageBackendType};

// AFTER
use llmspell_core::traits::storage::StorageBackend;
use llmspell_core::types::storage::StorageBackendType;
```

#### Pattern 4: ProceduralMemory Trait
```rust
// BEFORE
use llmspell_memory::ProceduralMemory;
use llmspell_memory::types::Pattern;

// AFTER
use llmspell_core::traits::storage::ProceduralMemory;
use llmspell_core::types::storage::Pattern;
```

#### Pattern 5: Combined Imports (Common in Tests)
```rust
// BEFORE
use llmspell_storage::{
    VectorStorage, VectorEntry, VectorQuery, VectorResult,
    StorageBackend, MemoryBackend
};

// AFTER
use llmspell_core::traits::storage::{VectorStorage, StorageBackend};
use llmspell_core::types::storage::{VectorEntry, VectorQuery, VectorResult};
use llmspell_storage::backends::memory::MemoryBackend; // Backend stays in storage crate
```

### 7.2 Crate Re-exports (Domain Crates Keep Wrappers)

**llmspell-memory** (keeps EpisodicMemory, SemanticMemory - domain wrappers):
```rust
// llmspell-memory/src/lib.rs
// These are DOMAIN traits that WRAP storage traits - they stay!
pub use crate::traits::episodic::EpisodicMemory;
pub use crate::traits::semantic::SemanticMemory;

// But ProceduralMemory moves to core (it's a storage trait, not domain wrapper)
#[deprecated(since = "0.14.0", note = "Use llmspell_core::traits::storage::ProceduralMemory")]
pub use llmspell_core::traits::storage::ProceduralMemory;
```

**llmspell-storage** (REMOVES all trait re-exports):
```rust
// llmspell-storage/src/lib.rs
// BEFORE
pub use crate::traits::StorageBackend;
pub use crate::vector_storage::{VectorStorage, VectorEntry, VectorQuery};

// AFTER - NO RE-EXPORTS, just backends
pub mod backends {
    pub mod memory;
    #[cfg(feature = "sqlite")]
    pub mod sqlite;
    #[cfg(feature = "postgres")]
    pub mod postgres;
}

// Traits come from llmspell-core now - NO re-export
```

**llmspell-graph** (REMOVES KnowledgeGraph re-export):
```rust
// llmspell-graph/src/lib.rs
// BEFORE
pub use crate::traits::KnowledgeGraph;

// AFTER - NO re-export
pub mod extraction; // Domain logic stays
pub mod backends;   // SurrealDB backend stays

// KnowledgeGraph trait is in llmspell-core now
```

---

## 8. Documentation Updates

### 8.1 Critical Documentation Files (Must Update)

#### Technical Documentation

**docs/technical/current-architecture.md**:
- 10+ code examples with `use llmspell_storage::VectorStorage`
- UPDATE: All import statements
- UPDATE: Architecture diagrams showing trait locations

**docs/technical/master-architecture-vision.md**:
- 20+ code examples with storage traits
- UPDATE: All examples
- NOTE: This is the master vision document - critical!

**docs/technical/postgresql-guide.md**:
- 8+ code examples with PostgreSQL backend
- UPDATE: Import statements

**docs/technical/sqlite-vector-storage-architecture.md**:
- 12+ code examples with SQLite backend
- UPDATE: Import statements

**docs/technical/rag-system-guide.md**:
- 6+ code examples with VectorStorage
- UPDATE: All examples

#### Developer Guide

**docs/developer-guide/03-extending-components.md**:
- "PART 6: Storage Backend Extension" section
- Examples show how to implement StorageBackend
- UPDATE: Import statements in all examples

**docs/developer-guide/reference/storage-backends.md**:
- Complete storage backend reference
- UPDATE: All import examples

#### Crate READMEs (11 files)

**llmspell-storage/README.md**:
- Lines 26-96: 4 code examples with imports
- UPDATE: All examples

**llmspell-memory/README.md**:
- Examples using MemoryManager with storage backends
- UPDATE: Import statements

**llmspell-graph/README.md**:
- Examples using KnowledgeGraph trait
- UPDATE: Trait import

**llmspell-rag/README.md**:
- Examples using VectorStorage
- UPDATE: Import statements

**llmspell-tenancy/README.md**:
- Examples wrapping VectorStorage
- UPDATE: Import statements

... (6 more crate READMEs)

### 8.2 Rustdoc Comments (20+ occurrences)

**Pattern**: Trait definitions have doc comment examples

```rust
// BEFORE (llmspell-storage/src/vector_storage.rs)
/// # Examples
///
/// ```rust
/// use llmspell_storage::{VectorEntry, VectorStorage};
/// use llmspell_core::state::StateScope;
/// ```

// AFTER (llmspell-core/src/traits/storage/vector.rs)
/// # Examples
///
/// ```rust
/// use llmspell_core::traits::storage::VectorStorage;
/// use llmspell_core::types::storage::VectorEntry;
/// use llmspell_core::state::StateScope;
/// ```
```

**Files with rustdoc examples**:
- All trait definition files (moving to llmspell-core)
- Backend implementation files (updating imports)
- Manager/wrapper files in domain crates

---

## 9. Test Infrastructure

### 9.1 Test Helper Pattern (Create `TestStorageFactory`)

**Problem**: 77 test files create backends inline with duplicate code

**Solution**: Create test factory in `llmspell-testing`

```rust
// llmspell-testing/src/storage.rs (NEW)
use llmspell_core::traits::storage::{StorageBackend, VectorStorage};
use llmspell_storage::backends::memory::MemoryBackend;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use std::sync::Arc;

pub struct TestStorageFactory;

impl TestStorageFactory {
    /// Create in-memory backend for fast tests
    pub fn memory_backend() -> Arc<dyn StorageBackend> {
        Arc::new(MemoryBackend::new())
    }

    /// Create temporary SQLite backend for integration tests
    pub async fn temp_sqlite_backend() -> Arc<SqliteBackend> {
        let config = SqliteConfig::temp().unwrap();
        Arc::new(SqliteBackend::new(config).await.unwrap())
    }

    /// Create temporary SQLite vector storage
    pub async fn temp_vector_storage(dimension: usize) -> Arc<dyn VectorStorage> {
        let backend = Self::temp_sqlite_backend().await;
        Arc::new(SqliteVectorStorage::new(backend, dimension).await.unwrap())
    }

    // ... more factory methods
}
```

**Usage in tests**:
```rust
// BEFORE (duplicated in 77 test files)
let config = SqliteConfig::temp().unwrap();
let backend = Arc::new(SqliteBackend::new(config).await.unwrap());
let storage = Arc::new(SqliteVectorStorage::new(backend, 384).await.unwrap());

// AFTER (using factory)
use llmspell_testing::storage::TestStorageFactory;
let storage = TestStorageFactory::temp_vector_storage(384).await;
```

**Impact**: Simplifies 77 test files, reduces duplication by ~200 lines

### 9.2 Test Files by Type

| Test Type | Count | Update Effort | Factory Usage |
|-----------|-------|---------------|---------------|
| **Unit tests** (simple) | 25 | LOW | Use `memory_backend()` |
| **Integration tests** (SQLite) | 30 | MEDIUM | Use `temp_sqlite_backend()` |
| **Integration tests** (PostgreSQL) | 15 | MEDIUM | Requires Docker |
| **E2E tests** (full stack) | 7 | HIGH | Complex setup |
| **TOTAL** | **77** | - | ~60 can use factory |

---

## 10. Migration Sequence

### 10.1 Week 1: Foundation (Days 1-3)

**Day 1**: Trait Migration to llmspell-core
- Create `llmspell-core/src/traits/storage/` directory
- Move `StorageBackend` trait (120 lines)
- Move `VectorStorage` trait (350 lines)
- Move `KnowledgeGraph` trait (250 lines)
- Move `ProceduralMemory` trait (150 lines)
- Create `llmspell-core/src/types/storage/` directory
- Move all domain types (2,500+ lines)
- **Validation**: `cargo check -p llmspell-core` (must compile in isolation)

**Day 2**: Update llmspell-storage Backends
- Update `src/backends/postgres/*.rs` (10 files, ~30 imports)
- Update `src/backends/sqlite/*.rs` (10 files, ~30 imports)
- Remove `src/traits.rs` (delete file)
- Remove `src/vector_storage.rs` (delete file)
- Update `src/lib.rs` (remove re-exports)
- **Validation**: `cargo check -p llmspell-storage`

**Day 3**: Update llmspell-graph
- Update `src/backends/*.rs` imports
- Remove `src/traits/knowledge_graph.rs` (moved to core)
- Update `src/lib.rs` (remove re-export)
- **Validation**: `cargo check -p llmspell-graph`

### 10.2 Week 2: Critical Crates (Days 4-8)

**Day 4**: llmspell-kernel (12 files)
- Update `src/state/manager.rs`
- Update `src/state/backend_adapter.rs`
- **DELETE** `src/state/vector_storage.rs` (duplicate of llmspell-storage version!)
- Update 9 other files
- **Validation**: `cargo check -p llmspell-kernel`

**Day 5**: llmspell-bridge (9+ files) - CRITICAL
- Update `src/infrastructure.rs` (backend creation hub)
- Update `src/rag_bridge.rs`
- Update `src/memory_bridge.rs`
- Update `src/globals/*.rs` (3 files)
- Update 3 more files
- **Validation**: `cargo check -p llmspell-bridge`
- **CRITICAL**: Run bridge integration tests!

**Day 6-7**: llmspell-memory (15 files)
- Update `src/manager.rs`
- Update `src/episodic/*.rs` (3 files)
- Update `src/consolidation/*.rs` (4 files)
- Update `src/semantic.rs`
- Update 6 other files
- **Validation**: `cargo check -p llmspell-memory`

**Day 8**: llmspell-rag (8 files)
- Update `src/traits/hybrid.rs` (extends VectorStorage)
- Update `src/pipeline/*.rs` (4 files)
- Update `src/state_integration.rs`
- Update `src/session_integration.rs`
- **Validation**: `cargo check -p llmspell-rag`

### 10.3 Week 3: Medium-Impact Crates (Days 9-11)

**Day 9**: llmspell-tenancy, llmspell-agents, llmspell-events
- Tenancy: 3 files, implements VectorStorage
- Agents: 2 files, uses StorageBackend
- Events: 3 files, adapter pattern
- **Validation**: `cargo check -p llmspell-tenancy llmspell-agents llmspell-events`

**Day 10**: llmspell-context, llmspell-templates, llmspell-testing
- Context: 3 files, indirect dependencies
- Templates: 2 files, uses MemoryManager
- Testing: 1 file + create TestStorageFactory
- **Validation**: `cargo check -p llmspell-context llmspell-templates llmspell-testing`

**Day 11**: Workspace Compilation
- **CRITICAL**: `cargo check --workspace --all-features`
- Fix any remaining compilation errors
- Verify zero dependency cycles

### 10.4 Week 4: Tests (Days 12-15)

**Day 12**: llmspell-storage tests (38 files!)
- Update imports in all PostgreSQL tests (20 files)
- Update imports in all SQLite tests (15 files)
- Update imports in other tests (3 files)
- **Effort**: ~100 import lines changed
- **Validation**: `cargo test -p llmspell-storage`

**Day 13**: llmspell-bridge tests (12 files)
- Update integration tests
- Update RAG tests
- Update memory tests
- **Validation**: `cargo test -p llmspell-bridge`

**Day 14**: Other crate tests (27 files)
- Memory tests (10 files)
- RAG tests (5 files)
- Tenancy tests (4 files)
- Other tests (8 files)
- **Validation**: `cargo test --workspace`

**Day 15**: Test Factory Integration
- Refactor tests to use `TestStorageFactory`
- Remove duplicated test setup code
- **Target**: Convert ~60 tests to use factory

### 10.5 Week 5: Documentation (Days 16-18)

**Day 16**: Technical Documentation
- Update `current-architecture.md` (10 examples)
- Update `master-architecture-vision.md` (20 examples)
- Update `postgresql-guide.md` (8 examples)
- Update `sqlite-vector-storage-architecture.md` (12 examples)
- Update `rag-system-guide.md` (6 examples)
- **Total**: ~56 code blocks

**Day 17**: Developer Guide + Crate READMEs
- Update `docs/developer-guide/03-extending-components.md`
- Update `docs/developer-guide/reference/storage-backends.md`
- Update 11 crate README files
- **Total**: ~50 code blocks

**Day 18**: Rustdoc Comments
- Update doc comments in trait definitions (now in llmspell-core)
- Update doc comments in backend implementations
- Fix broken doc links
- **Validation**: `cargo doc --workspace --no-deps --all-features`

### 10.6 Week 6: Validation & Polish (Days 19-22)

**Day 19**: Comprehensive Testing
- Run full test suite: `cargo test --workspace --all-features`
- Run benchmarks: `cargo bench --workspace`
- Verify performance (no regressions >5%)

**Day 20**: Quality Gates
- Zero clippy warnings: `cargo clippy --workspace --all-features --all-targets -- -D warnings`
- Format check: `cargo fmt --all -- --check`
- Doc tests: `cargo test --doc --workspace`
- Quality script: `./scripts/quality/quality-check.sh`

**Day 21**: Integration Validation
- Test Lua script examples (bridge layer)
- Test Python script examples (if any)
- Verify CLI commands work
- Run E2E tests

**Day 22**: Release Preparation
- Update CHANGELOG.md
- Update version to 0.14.0 in all Cargo.toml
- Write migration guide for downstream users
- Create GitHub release notes

---

## 11. Validation Strategy

### 11.1 Continuous Validation

**After each crate update**:
```bash
# Compile just this crate
cargo check -p llmspell-<crate>

# Run this crate's tests
cargo test -p llmspell-<crate>

# Check for clippy warnings
cargo clippy -p llmspell-<crate> -- -D warnings
```

**After each week**:
```bash
# Full workspace check
cargo check --workspace --all-features

# Full test suite
cargo test --workspace --all-features

# Dependency tree validation
cargo tree -p llmspell-core | grep -E "llmspell-(storage|graph|memory)"
# Should show ZERO dependencies (llmspell-core is foundation)
```

### 11.2 Compilation Checkpoints

| Checkpoint | Command | Expected Result |
|------------|---------|-----------------|
| **Day 1** | `cargo check -p llmspell-core` | ✅ Compiles (no deps) |
| **Day 3** | `cargo check -p llmspell-storage llmspell-graph` | ✅ Compiles |
| **Day 8** | `cargo check --workspace --lib` | ✅ All libs compile |
| **Day 11** | `cargo check --workspace --all-features` | ✅ Everything compiles |
| **Day 15** | `cargo test --workspace --lib` | ✅ All lib tests pass |
| **Day 18** | `cargo doc --workspace --no-deps` | ✅ All docs build |
| **Day 22** | `./scripts/quality/quality-check.sh` | ✅ All quality gates pass |

### 11.3 Performance Benchmarks

**Baseline** (before refactor):
```bash
cargo bench --bench memory_operations > baseline.txt
cargo bench --bench sqlite_vector_bench > baseline_vector.txt
cargo bench --bench graph_bench > baseline_graph.txt
```

**Post-refactor** (after Day 19):
```bash
cargo bench --bench memory_operations > refactor.txt
cargo bench --bench sqlite_vector_bench > refactor_vector.txt
cargo bench --bench graph_bench > refactor_graph.txt

# Compare (should be <5% variance)
diff baseline.txt refactor.txt
```

---

## 12. Risk Mitigation

### 12.1 High Risks

**Risk 1: Bridge Layer Breakage**
- **Impact**: ALL Lua/JS scripts fail
- **Mitigation**: Update llmspell-bridge early (Day 5)
- **Validation**: Run all bridge integration tests before proceeding
- **Rollback**: Git tag before bridge update

**Risk 2: Forgotten Import**
- **Impact**: Compilation failure late in process
- **Mitigation**: Grep for old imports after each week
- **Detection**:
  ```bash
  # Check for old imports
  rg "use llmspell_storage::(VectorStorage|StorageBackend)" llmspell-*/src/
  rg "use llmspell_graph::KnowledgeGraph" llmspell-*/src/
  rg "use llmspell_memory::ProceduralMemory" llmspell-*/src/
  ```

**Risk 3: Test Failures**
- **Impact**: Tests fail after import changes
- **Mitigation**: Test after each crate update
- **Recovery**: Fix imports immediately, don't accumulate failures

**Risk 4: Documentation Drift**
- **Impact**: Docs show old imports, confuse users
- **Mitigation**: Dedicated documentation week (Week 5)
- **Validation**: Manual review of all code examples

### 12.2 Medium Risks

**Risk 5: Performance Regression**
- **Impact**: Trait object overhead from `Arc<dyn Trait>`
- **Mitigation**: Benchmark before/after
- **Threshold**: <5% variance acceptable
- **Optimization**: Use `#[inline]` if needed

**Risk 6: Circular Dependencies**
- **Impact**: Compilation fails due to dep cycle
- **Mitigation**: llmspell-core has ZERO internal deps (verify daily)
- **Detection**: `cargo tree -p llmspell-core`

### 12.3 Mitigation Tools

**Git Workflow**:
```bash
# Create feature branch
git checkout -b refactor/phase-13c3-clean

# Tag before major milestones
git tag week1-foundation
git tag week2-critical-crates
git tag week3-medium-crates
git tag week4-tests
git tag week5-docs

# Rollback if needed
git reset --hard week2-critical-crates
```

**Automated Checks** (run daily):
```bash
#!/bin/bash
# scripts/refactor-check.sh

echo "=== Checking for old imports ==="
rg "use llmspell_storage::(VectorStorage|StorageBackend)" llmspell-*/src/ && exit 1
rg "use llmspell_graph::KnowledgeGraph" llmspell-*/src/ && exit 1

echo "=== Checking llmspell-core dependencies ==="
cargo tree -p llmspell-core | grep -E "llmspell-(storage|graph|memory)" && exit 1

echo "=== Compiling workspace ==="
cargo check --workspace --all-features || exit 1

echo "✅ All checks passed!"
```

---

## 13. Success Metrics & Breaking Changes

### 13.1 Import Changes (ALL Code)

**Every file using storage traits must update imports**:

| Old Import | New Import |
|-----------|------------|
| `use llmspell_storage::VectorStorage;` | `use llmspell_core::traits::storage::VectorStorage;` |
| `use llmspell_storage::StorageBackend;` | `use llmspell_core::traits::storage::StorageBackend;` |
| `use llmspell_graph::KnowledgeGraph;` | `use llmspell_core::traits::storage::KnowledgeGraph;` |
| `use llmspell_memory::ProceduralMemory;` | `use llmspell_core::traits::storage::ProceduralMemory;` |
| `use llmspell_storage::{VectorEntry, VectorQuery};` | `use llmspell_core::types::storage::{VectorEntry, VectorQuery};` |
| `use llmspell_graph::types::{Entity, Relationship};` | `use llmspell_core::types::storage::{Entity, Relationship};` |

### 13.2 Removed Exports

**llmspell-storage**: NO trait re-exports (only backend modules exported)
**llmspell-graph**: NO KnowledgeGraph re-export
**llmspell-memory**: ProceduralMemory re-export removed (EpisodicMemory/SemanticMemory stay)

### 13.3 Downstream Impact

**For users with custom backends**:
- Must update trait imports to `llmspell_core::traits::storage::*`
- Backend implementations stay in `llmspell_storage` crate (no change to `impl` statements)

**For users using traits**:
- All trait imports must change
- Type imports must change
- Backend instantiation unchanged

### 13.4 Success Metrics

| Metric | Target | Validation |
|--------|--------|------------|
| **Zero compilation errors** | ✅ | `cargo check --workspace --all-features` |
| **Zero clippy warnings** | ✅ | `cargo clippy --workspace --all-targets -- -D warnings` |
| **All tests passing** | ✅ 149+ tests | `cargo test --workspace --all-features` |
| **All docs building** | ✅ | `cargo doc --workspace --no-deps --all-features` |
| **Performance maintained** | <5% variance | Benchmark comparison |
| **Zero circular deps** | ✅ | `cargo tree -p llmspell-core` shows no llmspell-* deps |
| **Old imports removed** | 0 occurrences | `rg "use llmspell_(storage\|graph)::(VectorStorage\|KnowledgeGraph)"` |
| **Quality gates passing** | ✅ | `./scripts/quality/quality-check.sh` |

---

## Summary

**Total Effort**: 22 days (~4.5 weeks) for clean refactor with NO re-exports

**Files Changed**: ~250 files across workspace
**Lines Changed**: ~4,700 lines (3,500 new code in llmspell-core + 1,200 import updates)

**Breaking Changes**: ACCEPTED - clean break for v0.14.0 (pre-1.0)

**Benefits**:
- ✅ Single source of truth for all storage traits
- ✅ Clean architecture (no re-exports)
- ✅ Zero circular dependencies
- ✅ Foundation for future storage backends
- ✅ Easier maintenance (50% reduction in trait duplication)

**Risks**: Managed through incremental validation and automated checks
