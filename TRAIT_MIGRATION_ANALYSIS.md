# Task 13c.3.0: Storage Traits Migration Analysis

**Date**: November 14, 2025
**Task**: Identify and locate all storage traits for migration to llmspell-core
**Status**: Complete - All 4 traits identified and cataloged

---

## Executive Summary

This document provides a complete analysis of the 4 core storage traits that must be migrated to `llmspell-core` as part of Phase 13c.3.0 (Trait Architecture Consolidation). 

**Total API Surface**: 34 methods across 4 traits + 15 supporting types
**Scope**: All code analysis performed with zero implementation changes

### Traits Located

1. **StorageBackend** - `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-storage/src/traits.rs` (lines 45-101)
2. **VectorStorage** - `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-storage/src/vector_storage.rs` (lines 18-304)
3. **KnowledgeGraph** - `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-graph/src/traits/knowledge_graph.rs` (lines 10-142)
4. **ProceduralMemory** - `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-memory/src/traits/procedural.rs` (lines 44-104)

---

## Trait 1: StorageBackend

### Location
- **File**: `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-storage/src/traits.rs`
- **Lines**: 45-101 (trait definition)
- **Lines**: 9-43 (supporting types)
- **Lines**: 103-121 (helper trait)

### Methods (11 total)

All methods are async and return `Result<T>`:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `get` | `async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>` | Retrieve single value |
| `set` | `async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>` | Store single KV pair |
| `delete` | `async fn delete(&self, key: &str) -> Result<()>` | Delete single key |
| `exists` | `async fn exists(&self, key: &str) -> Result<bool>` | Check key existence |
| `list_keys` | `async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>` | Enumerate keys by prefix |
| `get_batch` | `async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>` | Multi-get |
| `set_batch` | `async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>` | Multi-set |
| `delete_batch` | `async fn delete_batch(&self, keys: &[String]) -> Result<()>` | Multi-delete |
| `clear` | `async fn clear(&self) -> Result<()>` | Wipe all data |
| `backend_type` | `fn backend_type(&self) -> StorageBackendType` | Get backend kind |
| `characteristics` | `fn characteristics(&self) -> StorageCharacteristics` | Get backend metadata |
| `run_migrations` | `async fn run_migrations(&self) -> Result<()>` | Apply schema migrations |
| `migration_version` | `async fn migration_version(&self) -> Result<usize>` | Get current schema version |

### Associated Types

#### StorageBackendType (Enum)
```rust
pub enum StorageBackendType {
    Memory,                                    // in-memory only
    Sqlite,                                    // SQLite database
    #[cfg(feature = "postgres")]
    Postgres,                                  // PostgreSQL + VectorChord
}
```
- **Derives**: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
- **Usage**: Identifies backend implementation type
- **Note**: Postgres variant is feature-gated

#### StorageCharacteristics (Struct)
```rust
pub struct StorageCharacteristics {
    pub persistent: bool,                      // Does it survive process restart?
    pub transactional: bool,                   // ACID transactions?
    pub supports_prefix_scan: bool,            // list_keys implementation?
    pub supports_atomic_ops: bool,             // Multi-operation atomicity?
    pub avg_read_latency_us: u64,             // Estimated read latency (microseconds)
    pub avg_write_latency_us: u64,            // Estimated write latency (microseconds)
}
```
- **Derives**: Debug, Clone, Serialize, Deserialize
- **Usage**: Query backend capabilities for adaptive behavior
- **Example Values**:
  - Memory: persistent=false, transactional=false, latency=<100us
  - SQLite: persistent=true, transactional=true, latency=<2ms
  - Postgres: persistent=true, transactional=true, latency=<5ms

#### StorageSerialize (Trait)
```rust
pub trait StorageSerialize: Sized {
    fn to_storage_bytes(&self) -> Result<Vec<u8>>;
    fn from_storage_bytes(bytes: &[u8]) -> Result<Self>;
}
```
- **Blanket Implementation**: All `Serialize + Deserialize` types
- **Serialization**: Uses serde_json
- **Usage**: Type-safe serialization for KV values
- **Note**: Automatically implemented for all types supporting serde

### Dependencies
- `anyhow::Result` - Error handling
- `async_trait` - Async trait syntax
- `serde::{Deserialize, Serialize}` - Type serialization
- `std::collections::HashMap` - Batch operations container

### Design Principles
- Unified KV interface across all backends
- Capabilities discovery via characteristics
- Batch operations for efficiency
- Backend-specific migrations
- Serialization abstraction

### Tests
- General storage validation (exact location TBD in implementations)

---

## Trait 2: VectorStorage + HNSWStorage

### Location
- **File**: `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-storage/src/vector_storage.rs`
- **Lines**: 18-304 (trait and types)
- **Lines**: 573-599 (HNSWStorage extension trait)
- **Lines**: 737-881 (comprehensive test suite)

### VectorStorage Methods (9 total)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `insert` | `async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>>` | Batch insert vectors with IDs |
| `search` | `async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>>` | ANN search without scope restriction |
| `search_scoped` | `async fn search_scoped(&self, query: &VectorQuery, scope: &StateScope) -> Result<Vec<VectorResult>>` | ANN with tenant isolation |
| `update_metadata` | `async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()>` | Update metadata without re-indexing |
| `delete` | `async fn delete(&self, ids: &[String]) -> Result<()>` | Delete vectors by ID |
| `delete_scope` | `async fn delete_scope(&self, scope: &StateScope) -> Result<usize>` | Bulk delete by scope (returns count) |
| `stats` | `async fn stats(&self) -> Result<StorageStats>` | Global statistics |
| `stats_for_scope` | `async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats>` | Scope-specific statistics |
| `save` | `async fn save(&self) -> Result<()>` | Persist to disk (default impl = no-op) |
| `load` | `async fn load(&self) -> Result<()>` | Load from disk (default impl = no-op) |

### HNSWStorage Extension (8 methods)
```rust
pub trait HNSWStorage: VectorStorage {
    fn configure_hnsw(&mut self, config: HNSWConfig);
    async fn build_index(&self) -> Result<()>;
    async fn create_namespace(&self, namespace: &str) -> Result<()>;
    async fn delete_namespace(&self, namespace: &str) -> Result<()>;
    fn hnsw_params(&self) -> &HNSWConfig;
    async fn optimize_index(&self) -> Result<()>;
    async fn namespace_stats(&self, namespace: &str) -> Result<NamespaceStats>;
    async fn save(&self) -> Result<()>;
}
```

### Associated Types - VectorEntry

```rust
pub struct VectorEntry {
    pub id: String,                                    // Unique identifier
    pub embedding: Vec<f32>,                           // Vector embedding
    pub metadata: HashMap<String, Value>,              // Flexible metadata
    pub scope: StateScope,                             // Multi-tenant scope (from llmspell-core)
    pub created_at: SystemTime,                        // Ingestion timestamp
    pub updated_at: SystemTime,                        // Last update timestamp
    pub event_time: Option<SystemTime>,                // Event time (bi-temporal)
    pub expires_at: Option<SystemTime>,                // Expiration time
    pub ttl_seconds: Option<u64>,                      // TTL duration
    pub tenant_id: Option<String>,                     // Explicit tenant ID
}
```

**Builder Methods**:
- `new(id: String, embedding: Vec<f32>) -> Self`
- `with_scope(scope: StateScope) -> Self`
- `with_metadata(metadata: HashMap<String, Value>) -> Self`
- `with_expiration(expires_at: SystemTime) -> Self`
- `with_event_time(event_time: SystemTime) -> Self`
- `with_ttl(ttl_seconds: u64) -> Self`
- `is_expired() -> bool`
- `update(&mut self)` - Updates `updated_at` to now

**Key Features**:
- Bi-temporal support (event_time vs ingestion time)
- TTL with auto-expiration calculation
- Flexible metadata via serde_json::Value
- Multi-tenant awareness via StateScope

### Associated Types - VectorQuery

```rust
pub struct VectorQuery {
    pub vector: Vec<f32>,                                           // Query vector
    pub k: usize,                                                   // Number of results
    pub filter: Option<HashMap<String, Value>>,                    // Metadata filters
    pub scope: Option<StateScope>,                                 // Scope restriction
    pub threshold: Option<f32>,                                    // Similarity threshold (0.0-1.0)
    pub include_metadata: bool,                                    // Include metadata in results
    pub event_time_range: Option<(SystemTime, SystemTime)>,       // Bi-temporal: filter by event time
    pub ingestion_time_range: Option<(SystemTime, SystemTime)>,  // Bi-temporal: filter by ingestion time
    pub exclude_expired: bool,                                     // Skip expired entries
}
```

**Builder Methods**:
- `new(vector: Vec<f32>, k: usize) -> Self`
- `with_scope(scope: StateScope) -> Self`
- `with_event_time_range(start: SystemTime, end: SystemTime) -> Self`
- `with_ingestion_time_range(start: SystemTime, end: SystemTime) -> Self`
- `exclude_expired(exclude: bool) -> Self`
- `with_filter(filter: HashMap<String, Value>) -> Self`
- `with_threshold(threshold: f32) -> Self`

### Associated Types - VectorResult

```rust
pub struct VectorResult {
    pub id: String,                                    // Vector ID
    pub score: f32,                                    // Similarity score (higher = more similar)
    pub vector: Option<Vec<f32>>,                      // Vector data (optional)
    pub metadata: Option<HashMap<String, Value>>,      // Metadata if requested
    pub distance: f32,                                 // Distance metric value
}
```

### Associated Types - StorageStats

```rust
pub struct StorageStats {
    pub total_vectors: usize,                          // Total vectors in storage
    pub storage_bytes: usize,                          // Total storage size
    pub namespace_count: usize,                        // Number of namespaces
    pub index_build_time_ms: Option<u64>,             // Index build time
    pub avg_query_time_ms: Option<f32>,               // Average query latency
    pub dimensions: Option<usize>,                     // Vector dimensions
}
```

### Associated Types - ScopedStats

```rust
pub struct ScopedStats {
    pub scope: StateScope,                             // The scope (from llmspell-core)
    pub vector_count: usize,                           // Vectors in this scope
    pub storage_bytes: usize,                          // Storage used by scope
    pub query_count: usize,                            // Queries executed
    pub tokens_processed: usize,                       // Total tokens embedded
    pub estimated_cost: f64,                           // Cost estimate in USD
}
```

### Associated Types - HNSWConfig

```rust
pub struct HNSWConfig {
    pub m: usize,                                      // Connections per node (16-64)
    pub ef_construction: usize,                        // Candidate list size (~200)
    pub ef_search: usize,                              // Search candidate size (~50-200)
    pub max_elements: usize,                           // Storage capacity (1M default)
    pub seed: Option<u64>,                             // RNG seed for reproducibility
    pub metric: DistanceMetric,                        // Distance metric (default: Cosine)
    pub allow_replace_deleted: bool,                   // Reuse deleted slots
    pub num_threads: Option<usize>,                    // Parallelism
    pub nb_layers: Option<usize>,                      // Hierarchical layers
    pub parallel_batch_size: Option<usize>,            // Batch size (default: 128)
    pub enable_mmap: bool,                             // Memory mapping (future)
    pub mmap_sync_interval: Option<u64>,              // Sync frequency (seconds)
}
```

**Preset Constructors**:
- `fast()` - m=12, ef_construction=100, ef_search=50
- `accurate()` - m=32, ef_construction=400, ef_search=200
- `balanced()` - m=16, ef_construction=200, ef_search=100 (default)

### Associated Types - DistanceMetric

```rust
pub enum DistanceMetric {
    #[default]
    Cosine,                                            // Cosine similarity
    Euclidean,                                         // L2 distance
    InnerProduct,                                      // Dot product
    Manhattan,                                         // L1 distance
}
```

### Associated Types - NamespaceStats

```rust
pub struct NamespaceStats {
    pub namespace: String,                             // Namespace identifier
    pub vector_count: usize,                           // Vectors in namespace
    pub memory_bytes: usize,                           // Memory used
    pub avg_connections: f32,                          // Avg node connections
    pub build_time_ms: Option<u64>,                    // Index build time
    pub last_optimized: Option<SystemTime>,            // Last optimization
}
```

### Dependencies
- `anyhow::Result` - Error handling
- `async_trait` - Async trait syntax
- `llmspell_core::state::StateScope` - Multi-tenant scoping (2 uses: VectorEntry.scope, VectorQuery.scope)
- `serde::{Deserialize, Serialize}` - Serialization
- `serde_json::Value` - Flexible metadata
- `std::collections::HashMap` - Metadata and batch containers
- `std::time::SystemTime` - Temporal support

### Design Principles
- Multi-tenant aware (StateScope isolation)
- Bi-temporal support (event_time vs ingestion_time)
- TTL and expiration handling
- HNSW extension for advanced indexing
- Comprehensive statistics for monitoring
- Builder pattern for queries and entries
- Namespace isolation for multi-tenant scenarios

### Tests
- Comprehensive test suite: lines 737-881 (~145 lines)
- Tests cover:
  - Builder pattern methods
  - Temporal fields
  - TTL mechanism
  - Expiration checking
  - Entry updates
  - Temporal query filters
  - Bi-temporal support
  - HNSW config presets
  - Distance metric defaults

### Performance Targets
- Average search latency: <2ms
- Query time for stats: <5ms

---

## Trait 3: KnowledgeGraph

### Location
- **File**: `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-graph/src/traits/knowledge_graph.rs`
- **Lines**: 10-142 (trait definition)
- **Supporting Types**: `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-graph/src/types.rs` (lines 1-206)

### Methods (9 total)

| Method | Signature | Purpose |
|--------|-----------|---------|
| `add_entity` | `async fn add_entity(&self, entity: Entity) -> Result<String>` | Create entity, return ID |
| `update_entity` | `async fn update_entity(&self, id: &str, changes: HashMap<String, Value>) -> Result<()>` | Update properties, preserving history |
| `get_entity` | `async fn get_entity(&self, id: &str) -> Result<Entity>` | Get current entity version |
| `get_entity_at` | `async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>` | Time-travel query |
| `add_relationship` | `async fn add_relationship(&self, relationship: Relationship) -> Result<String>` | Create relationship, return ID |
| `get_related` | `async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>>` | Find related entities by type |
| `get_relationships` | `async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>` | Get all relationships |
| `query_temporal` | `async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>` | Complex temporal queries |
| `delete_before` | `async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>` | Data retention cleanup |
| `traverse` | `async fn traverse(&self, start_entity: &str, relationship_type: Option<&str>, max_depth: usize, at_time: Option<DateTime<Utc>>) -> Result<Vec<(Entity, usize, String)>>` | Multi-hop graph traversal |

### Key Algorithm: traverse() Method

**Signature**: `async fn traverse(&self, start_entity: &str, relationship_type: Option<&str>, max_depth: usize, at_time: Option<DateTime<Utc>>) -> Result<Vec<(Entity, usize, String)>>`

**Parameters**:
- `start_entity`: Starting entity ID
- `relationship_type`: Optional filter (None = traverse all types)
- `max_depth`: Maximum traversal depth (1-4 recommended, 10 max)
- `at_time`: Optional temporal point for bi-temporal queries (None = current)

**Returns**: `Vec<(Entity, depth, path_json)>` where:
- Entity: The discovered entity
- depth: Distance from start (0 = start entity)
- path_json: JSON array string of entity IDs traversed

**Features**:
- Cycle prevention
- Relationship type filtering
- Temporal point queries (bi-temporal)

**Performance**:
- 1-hop: O(k) where k = avg relationships per node
- N-hop: O(k^N) worst case, O(k*N) with cycle prevention
- Target: <50ms for 4-hop traversal on 100K node graph

**Example Usage**:
```rust
// Find all entities within 2 hops via "knows" relationships
let results = graph.traverse("entity-1", Some("knows"), 2, None).await?;
for (entity, depth, path) in results {
    println!("Found {} at depth {} via path {}", entity.name, depth, path);
}
```

### Associated Types - Entity

```rust
pub struct Entity {
    pub id: String,                                    // Unique UUID
    pub name: String,                                  // Entity label
    pub entity_type: String,                           // Category (e.g., "person", "concept")
    pub properties: Value,                             // Flexible properties (JSON)
    pub event_time: Option<DateTime<Utc>>,            // When event occurred (bi-temporal)
    pub ingestion_time: DateTime<Utc>,                // When we learned about it (bi-temporal)
}
```

**Constructor Methods**:
- `new(name: String, entity_type: String, properties: Value) -> Self` - Auto-generates UUID
- `with_event_time(event_time: DateTime<Utc>) -> Self` - Builder
- `with_id(id: String) -> Self` - Builder

**Temporal Semantics**:
- `event_time`: When the real-world event occurred (can be None if unknown)
- `ingestion_time`: When we learned about it (always set to Utc::now())

### Associated Types - Relationship

```rust
pub struct Relationship {
    pub id: String,                                    // Unique UUID
    pub from_entity: String,                           // Source entity ID
    pub to_entity: String,                             // Target entity ID
    pub relationship_type: String,                     // Type (e.g., "has_feature", "works_at")
    pub properties: Value,                             // Flexible properties (JSON)
    pub event_time: Option<DateTime<Utc>>,            // When relationship formed (bi-temporal)
    pub ingestion_time: DateTime<Utc>,                // When we learned about it (bi-temporal)
}
```

**Constructor Methods**:
- `new(from: String, to: String, type: String, props: Value) -> Self` - Auto-generates UUID
- `with_event_time(event_time: DateTime<Utc>) -> Self` - Builder
- `with_id(id: String) -> Self` - Builder

### Associated Types - TemporalQuery

```rust
pub struct TemporalQuery {
    pub entity_type: Option<String>,                   // Filter by type
    pub event_time_start: Option<DateTime<Utc>>,      // Event time range start
    pub event_time_end: Option<DateTime<Utc>>,        // Event time range end
    pub ingestion_time_start: Option<DateTime<Utc>>,  // Ingestion time range start
    pub ingestion_time_end: Option<DateTime<Utc>>,    // Ingestion time range end
    pub property_filters: Vec<(String, Value)>,       // Property filters
    pub limit: Option<usize>,                         // Result limit
}
```

**Constructor Methods**:
- `new() -> Self` - Empty query
- `with_entity_type(type: String) -> Self`
- `with_event_time_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Self`
- `with_ingestion_time_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Self`
- `with_property(key: String, value: Value) -> Self`
- `with_limit(limit: usize) -> Self`

### Dependencies
- `async_trait` - Async trait syntax
- `chrono::{DateTime, Utc}` - Bi-temporal timestamps
- `serde::{Deserialize, Serialize}` - Serialization
- `serde_json::Value` - Flexible properties
- `uuid::Uuid` - Entity/relationship IDs
- `crate::error::Result` - Error type from llmspell-graph

### Design Principles
- Bi-temporal data model (event_time vs ingestion_time)
- Property-based filtering via JSON
- Relationship-typed directed graph
- Multi-hop traversal with cycle prevention
- Time-travel queries (what did we know at time T?)
- Data retention via delete_before()
- Path tracking for audit trails

### Performance Targets
- 4-hop traversal on 100K nodes: <50ms

---

## Trait 4: ProceduralMemory

### Location
- **File**: `/Users/spuri/projects/lexlapax/rs-llmspell/llmspell-memory/src/traits/procedural.rs`
- **Lines**: 44-104 (trait definition and type)

### Methods (5 total)

**Active Methods (Phase 13.7.4)**:

| Method | Signature | Purpose |
|--------|-----------|---------|
| `record_transition` | `async fn record_transition(&self, scope: &str, key: &str, from_value: Option<&str>, to_value: &str) -> Result<u32>` | Record state transition, return frequency |
| `get_pattern_frequency` | `async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32>` | Query transition frequency |
| `get_learned_patterns` | `async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<Pattern>>` | Get patterns above threshold |

**Placeholder Methods (Phase 13.3)**:

| Method | Signature | Status |
|--------|-----------|--------|
| `get_pattern` | `async fn get_pattern(&self, id: &str) -> Result<()>` | Deprecated, to be replaced |
| `store_pattern` | `async fn store_pattern(&self, pattern_data: &str) -> Result<String>` | Deprecated, to be replaced |

### Associated Type - Pattern

```rust
pub struct Pattern {
    pub scope: String,              // State scope (e.g., "global", "session:xyz")
    pub key: String,                // State key (e.g., "config.theme")
    pub value: String,              // Transition value (e.g., "dark")
    pub frequency: u32,             // Number of times transition occurred
    pub first_seen: u64,            // Milliseconds since epoch
    pub last_seen: u64,             // Milliseconds since epoch
}
```

**Constraints**:
- Minimum frequency threshold: 3 occurrences (must occur ≥3 times to be considered a pattern)
- Bi-temporal tracking via first_seen and last_seen timestamps

### Implementation Timeline

| Phase | Status | Features |
|-------|--------|----------|
| 13.7.4 (CURRENT) | Implemented | State transition pattern tracking |
| 13.3 | Pending | Full pattern storage and retrieval |
| 14 | Planned | Pattern learning from state transitions |
| 15 | Planned | Skill optimization and transfer learning |

### Dependencies
- `async_trait` - Async trait syntax
- `serde::{Deserialize, Serialize}` - Serialization
- `crate::error::Result` - Error type from llmspell-memory

### Design Principles
- Minimal Phase 13 scope (state transitions only)
- Frequency-based pattern learning
- Scope-based pattern isolation
- Key-value state tracking
- Extensible for future phases
- Bi-temporal support (first_seen, last_seen)

### Performance Targets
- Pattern lookup: <1ms

---

## Migration Summary

### File Structure

```
CURRENT STATE:
llmspell-storage/src/
  ├─ traits.rs (StorageBackend + types)
  └─ vector_storage.rs (VectorStorage + HNSWStorage)

llmspell-graph/src/
  ├─ traits/knowledge_graph.rs (KnowledgeGraph)
  └─ types.rs (Entity, Relationship, TemporalQuery)

llmspell-memory/src/
  └─ traits/procedural.rs (ProceduralMemory + Pattern)

TARGET STATE (after migration):
llmspell-core/src/
  ├─ storage/
  │   ├─ backend.rs (StorageBackend + types)
  │   └─ vector.rs (VectorStorage + HNSWStorage)
  ├─ graph/
  │   ├─ trait.rs (KnowledgeGraph)
  │   └─ types.rs (Entity, Relationship, TemporalQuery)
  └─ memory/
      └─ procedural.rs (ProceduralMemory + Pattern)

With re-exports from:
  - llmspell-storage/src/lib.rs
  - llmspell-graph/src/lib.rs
  - llmspell-memory/src/lib.rs
```

### Total API Surface

| Category | Count |
|----------|-------|
| Traits | 4 (StorageBackend, VectorStorage, KnowledgeGraph, ProceduralMemory) |
| Extension Traits | 1 (HNSWStorage extends VectorStorage) |
| Methods | 34 (11+9+9+5) |
| Supporting Structs | 11 (VectorEntry, VectorQuery, VectorResult, StorageStats, ScopedStats, HNSWConfig, NamespaceStats, Entity, Relationship, TemporalQuery, Pattern) |
| Enums | 2 (StorageBackendType, DistanceMetric) |
| Helper Traits | 1 (StorageSerialize) |

### Dependency Analysis

**No circular dependencies**:
- VectorStorage depends on `llmspell_core::state::StateScope` (acceptable)
- KnowledgeGraph has no core dependencies
- ProceduralMemory has no core dependencies
- StorageBackend has no core dependencies

**All dependencies are external or from llmspell-core itself** (which already exists):
- async_trait
- serde
- chrono
- uuid
- anyhow

---

## Implementation Order

1. **StorageBackend** (no dependencies)
   - Migrate traits.rs content
   - Migrate supporting types
   - Update imports in llmspell-storage

2. **VectorStorage** (depends on StateScope)
   - Migrate vector_storage.rs content
   - Ensure StateScope is accessible
   - Preserve all tests
   - Update imports in llmspell-storage

3. **KnowledgeGraph** (independent)
   - Migrate knowledge_graph.rs trait
   - Migrate types.rs supporting types
   - Update imports in llmspell-graph

4. **ProceduralMemory** (independent)
   - Migrate procedural.rs trait and Pattern
   - Update imports in llmspell-memory

5. **Integration**
   - Update all Cargo.toml dependencies
   - Verify re-exports work
   - Run full test suite
   - Update documentation

---

## Key Implementation Notes

1. **Feature Flags**: Preserve `#[cfg(feature = "postgres")]` on StorageBackendType::Postgres

2. **Documentation**: All traits have extensive doc comments with examples - preserve completely

3. **Tests**: Move all unit tests with their code

4. **Default Implementations**: VectorStorage has default impls for `save()` and `load()`

5. **Builder Patterns**: Extensively used in VectorEntry, VectorQuery, Entity, Relationship, TemporalQuery

6. **Re-exports**: Ensure public re-exports from crate roots to minimize import changes in dependent code

7. **Temporal Support**: All major types support bi-temporal semantics - critical for queries and auditing

---

## Verification Checklist

- [ ] All 4 traits located and documented
- [ ] All 11 supporting types identified
- [ ] All method signatures captured
- [ ] All dependencies cataloged
- [ ] No circular dependencies detected
- [ ] Test locations identified
- [ ] Doc comment examples preserved in analysis
- [ ] Builder pattern methods documented
- [ ] Temporal semantics understood
- [ ] Performance targets documented

**Status**: COMPLETE - Ready for migration implementation

---

**Document Generated**: November 14, 2025
**Scope**: Analysis only - no code changes
**Next Step**: Use this analysis for Task 13c.3.1 (Trait Migration Implementation)
