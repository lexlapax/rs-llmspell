# SQLite Vector Storage Architecture

**Status**: ✅ Implemented (Phase 13c.2.3)
**Version**: 0.13.1
**Author**: Memory Team
**Last Updated**: 2025-11-10

## Overview

SqliteVectorStorage implements the VectorStorage trait using a hybrid architecture that combines SQLite's durability with HNSW's fast approximate nearest neighbor search.

**Key Design Decisions**:
- **Persistence**: vec0 virtual tables + vector_metadata regular table
- **Search**: In-memory HNSW indices (vectorlite-rs) per namespace
- **Isolation**: Scope-based namespaces (user:xyz, session:xyz, global)
- **Fallback**: Graceful degradation if HNSW unavailable

## Architecture

### Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│ SqliteVectorStorage                                         │
├─────────────────────────────────────────────────────────────┤
│  backend: Arc<SqliteBackend>                                │
│  dimension: usize (384/768/1536/3072)                       │
│  metric: DistanceMetric                                     │
│  hnsw_indices: DashMap<String, Arc<RwLock<Option<Hnsw>>>>  │
│  persistence_path: PathBuf                                  │
│  m, ef_construction, ef_search, max_elements: usize         │
└─────────────────────────────────────────────────────────────┘
         │                                   │
         │ libsql queries                    │ HNSW search
         ▼                                   ▼
┌──────────────────────┐          ┌─────────────────────────┐
│ SQLite Tables        │          │ In-Memory HNSW Indices  │
├──────────────────────┤          ├─────────────────────────┤
│ vec_embeddings_768   │          │ "__global__": HnswIndex │
│   (vec0 virtual)     │          │ "user:xyz": HnswIndex   │
│ - rowid              │          │ "session:abc": HnswIndex│
│ - embedding          │          └─────────────────────────┘
│                      │                      │
│ vector_metadata      │                      │ MessagePack
│ - rowid (FK)         │                      ▼
│ - id (UUID)          │          ┌─────────────────────────┐
│ - tenant_id          │          │ Persistence Files       │
│ - scope              │          ├─────────────────────────┤
│ - dimension          │          │ __global___768_Cosine.  │
│ - metadata (JSON)    │          │   hnsw                  │
│ - created_at         │          │ user:xyz_768_Cosine.    │
│ - updated_at         │          │   hnsw                  │
└──────────────────────┘          └─────────────────────────┘
```

### Data Flow

#### Insert Operation
```
insert(VectorEntry) →
  1. Validate dimension
  2. INSERT into vec_embeddings_768 (embedding as JSON)
  3. Get last_insert_rowid
  4. INSERT into vector_metadata (rowid, id, scope, ...)
  5. Get/create HNSW index for namespace
  6. Insert into HNSW index (rowid, embedding)
  ← Return vector ID
```

#### Search Operation
```
search_scoped(query, scope) →
  1. Get HNSW index for namespace
  2. HNSW K-NN search → [(rowid, distance), ...]
  3. For each rowid:
     SELECT id, tenant_id, scope, metadata
     FROM vector_metadata WHERE rowid = ?
  4. Parse results
  ← Return Vec<VectorResult>
```

#### Delete Operation
```
delete(ids) →
  1. For each id:
     SELECT rowid, scope FROM vector_metadata WHERE id = ?
  2. DELETE FROM vector_metadata WHERE id = ?
  3. DELETE FROM vec_embeddings_768 WHERE rowid = ?
  4. Remove HNSW index for namespace (forces rebuild)
  ← OK
```

## Implementation Details

### Namespace Isolation

Scopes are mapped to namespaces for HNSW index isolation:

```rust
fn scope_to_namespace(scope: &StateScope) -> String {
    match scope {
        StateScope::Global => "__global__",
        StateScope::User(id) => format!("user:{}", id),
        StateScope::Session(id) => format!("session:{}", id),
        StateScope::Agent(id) => format!("agent:{}", id),
        StateScope::Tool(id) => format!("tool:{}", id),
        StateScope::Workflow(id) => format!("workflow:{}", id),
        StateScope::Hook(id) => format!("hook:{}", id),
        StateScope::Custom(s) => {
            if s.starts_with("tenant:") => s.clone(),
            else => format!("custom:{}", s)
        }
    }
}
```

Each namespace gets its own HNSW index in `DashMap<String, Arc<RwLock<Option<HnswIndex>>>>`.

### HNSW Index Management

**Lazy Loading**: Indices built on first access per namespace
**Persistence**: Serialized to `.hnsw` files (MessagePack format)
**Rebuild Strategy**: On delete/update, clear index and rebuild from tables

```rust
async fn get_or_create_index(namespace: &str) -> Result<Arc<RwLock<Option<HnswIndex>>>> {
    if cached → return cached

    if .hnsw file exists:
        load_index_from_disk()
    else:
        build_index_from_table(namespace)

    cache + return
}
```

### HNSW Persistence

vectorlite-rs HnswIndex doesn't support direct serde (hnsw_rs limitation), so we:
1. Serialize raw vectors + metadata via `HnswPersistence` struct
2. On load, deserialize and rebuild HNSW graph from vectors
3. This is slower than native serde but ensures correctness

```rust
// HnswPersistence struct
{
    dimension: usize,
    metric: DistanceMetric,
    max_elements, m, ef_construction: usize,
    vectors: HashMap<i64, Vec<f32>>  // rowid → embedding
}
```

### Thread Safety

**RwLock Pattern**: Each HNSW index wrapped in `Arc<RwLock<Option<HnswIndex>>>`
**Lock Management**: Drop guards before `.await` to avoid Send issues
**DashMap**: Lock-free concurrent hash map for namespace cache

```rust
// Correct pattern (drop guard before await)
let neighbors = {
    let index_ref = self.get_or_create_index(&namespace).await?;
    let index_guard = index_ref.read();
    index_guard.as_ref().unwrap().search(...)
    // index_guard dropped here
};
let conn = self.backend.get_connection().await?;  // OK
```

## Performance Characteristics

### Complexity

| Operation | SQLite | HNSW | Combined |
|-----------|--------|------|----------|
| Insert | O(log N) | O(log N) | O(log N) |
| Search | O(N) | O(log N) | **O(log N)** |
| Delete | O(1) | N/A (rebuild) | O(N) |
| Update Metadata | O(1) | N/A | O(1) |

### Expected Latency (10K vectors)

- **Insert**: <1ms (SQLite write + HNSW insert)
- **Search**: <10ms (HNSW search + metadata JOIN)
- **Delete**: ~100ms (rebuild HNSW index)
- **Stats**: <5ms (COUNT queries)

### Memory Usage

- **Per Vector**: ~12 bytes (rowid + pointers)
- **HNSW Index**: 2-4KB per vector (m=16, nb_layers calculated)
- **10K vectors**: ~20-40MB per namespace

### Speedup vs sqlite-vec

- **Brute Force (vec0)**: O(N) search, ~50-500ms for 10K-100K vectors
- **HNSW (vectorlite-rs)**: O(log N) search, ~5-10ms for 10K-100K vectors
- **Expected**: **3-100x faster** depending on dataset size

## Configuration

### HNSW Parameters

```rust
SqliteVectorStorage::new(backend, 768)?
    .configure_hnsw(
        m: 16,                // Links per node (16-64 typical)
        ef_construction: 200, // Build accuracy (100-400 typical)
        ef_search: 50,        // Search accuracy (10-200 typical)
        max_elements: 100_000 // Max vectors per index
    )
```

**Tuning**:
- **Higher M**: Better recall, more memory (2-4KB/vector per increase)
- **Higher ef_construction**: Slower build, better index quality
- **Higher ef_search**: Slower search, better recall

### Supported Dimensions

- 384: sentence-transformers/all-MiniLM-L6-v2
- 768: OpenAI ada-002, BERT-base
- 1536: OpenAI text-embedding-3-small
- 3072: OpenAI text-embedding-3-large

## Migration V3 Schema

```sql
-- vec0 virtual tables (one per dimension)
CREATE VIRTUAL TABLE vec_embeddings_768 USING vec0(embedding float[768]);

-- Metadata table
CREATE TABLE vector_metadata (
    rowid INTEGER PRIMARY KEY,  -- Maps to vec_embeddings rowid
    id TEXT NOT NULL UNIQUE,    -- UUID
    tenant_id TEXT NOT NULL,    -- Tenant isolation
    scope TEXT NOT NULL,        -- Namespace (user:xyz, session:abc)
    dimension INTEGER NOT NULL CHECK (dimension IN (384, 768, 1536, 3072)),
    metadata TEXT NOT NULL DEFAULT '{}',  -- JSON
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- Indexes
CREATE INDEX idx_vector_metadata_tenant_scope ON vector_metadata(tenant_id, scope);
CREATE INDEX idx_vector_metadata_id ON vector_metadata(id);
CREATE INDEX idx_vector_metadata_dimension ON vector_metadata(dimension);
```

## Error Handling

- **Dimension Mismatch**: Return error immediately (don't insert)
- **HNSW Unavailable**: Warn + return empty results (graceful degradation)
- **Index Load Failure**: Warn + rebuild from table
- **Delete Non-Existent**: Silently skip (idempotent)
- **Namespace Empty**: Return None (no index built)

## Testing Strategy

### Unit Tests (TODO)
- ✅ Compilation passes
- [ ] Insert/search/delete basic operations
- [ ] Namespace isolation
- [ ] HNSW persistence (save/load)
- [ ] Dimension validation
- [ ] Scope parsing (namespace ↔ StateScope)
- [ ] Error cases (missing index, wrong dimension)

### Integration Tests (TODO)
- [ ] With SqliteBackend (migrations applied)
- [ ] With MemoryManager (end-to-end)
- [ ] Multi-namespace concurrent access
- [ ] Large dataset (10K+ vectors)
- [ ] Persistence across restarts

### Benchmarks (TODO)
- [ ] Insert latency (target: <1ms)
- [ ] Search latency (target: <10ms for 10K vectors)
- [ ] Speedup vs sqlite-vec brute-force (target: 3-100x)
- [ ] Memory usage (target: <50MB for 10K vectors)

## Limitations

1. **No Incremental Delete**: Deleting vectors clears entire HNSW index (rebuild required)
2. **No Cross-Namespace Search**: Each namespace searched independently
3. **No Metadata Filtering in HNSW**: Must filter results after K-NN
4. **No Hybrid Search**: No combination of HNSW + full-text search yet
5. **Fixed Dimensions**: Cannot change dimension after table creation

## Future Improvements

- **Incremental HNSW Updates**: Avoid full rebuild on delete
- **Metadata Pre-Filtering**: Filter before HNSW search (requires index integration)
- **Hybrid Search**: Combine HNSW with FTS5 for text + vector search
- **Multi-Dimension Support**: Query across dimensions with automatic routing
- **Compression**: Store embeddings as f16 or int8 for 2-4x space savings
- **Distributed Sharding**: Split large namespaces across multiple HNSW indices

## References

- **hnsw_rs**: https://github.com/jean-pierreBoth/hnswlib-rs
- **vectorlite-rs**: llmspell-rs/vectorlite-rs (pure Rust HNSW SQLite extension)
- **sqlite-vec**: https://github.com/asg017/sqlite-vec (brute-force baseline)
- **Migration V3**: llmspell-storage/migrations/sqlite/V3__vector_embeddings.sql
- **VectorStorage Trait**: llmspell-storage/src/vector_storage.rs

## Files

- **Implementation**: `llmspell-storage/src/backends/sqlite/vector.rs` (806 lines)
- **Tests**: `llmspell-storage/tests/sqlite_vector_tests.rs` (TODO)
- **Benchmarks**: `llmspell-storage/benches/vector_search.rs` (TODO)
- **Migration**: `llmspell-storage/migrations/sqlite/V3__vector_embeddings.sql` (121 lines)
