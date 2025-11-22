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

### Trait Refactor Performance Impact

**Phase 13c.3.1.x trait centralization** (Nov 2024) showed that proper connection and transaction management dominates performance, not vtable dispatch overhead.

#### Measured Overhead (Post-Optimization)

| Operation | Baseline | After Trait Refactor | Overhead | Status |
|-----------|----------|---------------------|----------|--------|
| insert/100 | 1.25ms | 1.24ms | **-0.8%** | ✅ GREEN |
| search/100 | 821µs | 850µs | **+3.5%** | ✅ GREEN |
| search/1000 | 885µs | 977µs | **+10.4%** | ⚠️ YELLOW |
| batch_insert/10 | 12.2ms | 2.2ms | **-82%** | ✅ GREEN |
| batch_insert/100 | 119ms | 13.8ms | **-88%** | ✅ GREEN |

**Key Findings**:
- Trait indirection overhead: **<5% for small operations** (within noise)
- Connection/transaction management: **15-88% impact** when optimized
- Async trait dispatch: Minimal overhead compared to database I/O
- Batch operations: **5-9x faster** due to explicit transaction handling

#### Optimization Best Practices

**For VectorStorage Trait Implementers**:

1. **Connection Management** (CRITICAL):
   ```rust
   // ❌ BAD: Get connection inside loop
   for entry in entries {
       let conn = self.backend.get_connection().await?;
       conn.execute("INSERT ...", params).await?;
   }

   // ✅ GOOD: Single connection + explicit transaction
   let conn = self.backend.get_connection().await?;
   conn.execute("BEGIN IMMEDIATE", ()).await?;
   for entry in entries {
       conn.execute("INSERT ...", params).await?;
   }
   conn.execute("COMMIT", ()).await?;
   ```

2. **Transaction Boundaries** (CRITICAL):
   - Use explicit `BEGIN IMMEDIATE`/`COMMIT` for batch operations
   - Single operations benefit from explicit transactions (15-17% faster)
   - libsql doesn't auto-wrap writes—always be explicit

3. **Arc Cloning** (MINOR):
   - Arc cloning overhead is negligible compared to I/O
   - Prefer borrowing `&self` but don't over-optimize Arc usage
   - Profile database I/O before optimizing memory allocations

4. **Inline Hints** (MARGINAL):
   - `#[inline]` on small trait methods (<10 lines) may help
   - Measure before applying—impact typically <1%
   - Focus on connection/transaction optimization first

#### Trade-offs Documented

**search/1000 (+10.4% regression)**:
- **Cause**: Async trait overhead accumulates on larger result sets (1000 items)
- **Justification**: Acceptable for maintaining trait-based extensibility
- **Mitigation**: Users can bypass trait with direct SqliteBackend usage for performance-critical paths
- **Alternative**: Enum dispatch would eliminate overhead but breaks extensibility

**When to Use Direct Backend**:
```rust
// Trait-based (extensible, 10% overhead on large operations)
let results = vector_storage.search_scoped(query, scope, 1000).await?;

// Direct backend (non-extensible, maximum performance)
let conn = sqlite_backend.get_connection().await?;
conn.execute("BEGIN IMMEDIATE", ()).await?;
let rows = conn.query("SELECT ...", params).await?;
conn.execute("COMMIT", ()).await?;
// Manual result parsing...
```

#### Performance Validation

See `benchmark_comparison.md` for full performance analysis from Phase 13c.3.1.16 optimization work.

**Benchmarks**:
- `benches/sqlite_vector_bench.rs`: Vector storage operations
- `benches/memory_operations.rs`: Semantic backend operations

**Baseline Preservation**:
- Run benchmarks before trait API changes
- Document any >5% regressions with justification
- Connection/transaction patterns matter more than trait design

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

## Export/Import Support (Phase 13c.3.2)

**Bidirectional Migration**: SQLite ↔ PostgreSQL lossless migration for all vector embeddings

### Export Process

Vector embeddings are exported by dimension table with all metadata preserved:

```rust
// Export structure
pub struct VectorEmbeddingExport {
    pub id: String,              // UUID
    pub tenant_id: String,       // Tenant isolation
    pub scope: String,           // Namespace (user:xyz, session:abc)
    pub embedding: Vec<f32>,     // Full-precision vector
    pub metadata: Value,         // JSON metadata
    pub created_at: i64,         // Unix timestamp
    pub updated_at: i64,         // Unix timestamp
}

// Exported by dimension
pub struct ExportData {
    pub vector_embeddings: HashMap<usize, Vec<VectorEmbeddingExport>>,
    // 384 → Vec<VectorEmbeddingExport>
    // 768 → Vec<VectorEmbeddingExport>
    // 1536 → Vec<VectorEmbeddingExport>
    // 3072 → Vec<VectorEmbeddingExport>
    ...
}
```

**Export Query** (per dimension):

```sql
SELECT
    vm.id,
    vm.tenant_id,
    vm.scope,
    ve.embedding,  -- JSON array from vec0
    vm.metadata,
    vm.created_at,
    vm.updated_at
FROM vector_metadata vm
INNER JOIN vec_embeddings_768 ve ON ve.rowid = vm.rowid
WHERE vm.dimension = 768
ORDER BY vm.created_at;
```

**Key Features**:
- ✅ **Full-precision vectors**: No quantization or compression
- ✅ **Dimension preservation**: Vectors exported by dimension (384/768/1536/3072)
- ✅ **Metadata included**: JSON metadata, scopes, tenant IDs preserved
- ✅ **HNSW indices NOT exported**: Rebuilt on import (ensures consistency)

### Import Process

Import restores vectors to SQLite with automatic HNSW index rebuild:

```rust
async fn import_vectors(dimension: usize, vectors: Vec<VectorEmbeddingExport>) {
    let conn = backend.get_connection().await?;
    conn.execute("BEGIN IMMEDIATE", ()).await?;

    for vector in vectors {
        // 1. Insert into vec0 virtual table
        conn.execute(
            format!("INSERT INTO vec_embeddings_{} (embedding) VALUES (?)", dimension),
            &[serde_json::to_string(&vector.embedding)?]
        ).await?;

        // 2. Get rowid
        let rowid = conn.last_insert_rowid();

        // 3. Insert metadata
        conn.execute(
            "INSERT INTO vector_metadata (rowid, id, tenant_id, scope, dimension, metadata, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![rowid, vector.id, vector.tenant_id, vector.scope, dimension,
                    serde_json::to_string(&vector.metadata)?, vector.created_at, vector.updated_at]
        ).await?;
    }

    conn.execute("COMMIT", ()).await?;

    // 4. HNSW indices rebuilt lazily on first search per namespace
}
```

**Import Behavior**:
- **Transaction-safe**: All-or-nothing import with rollback on error
- **HNSW rebuild**: Indices rebuilt lazily on first search (not during import)
- **Dimension validation**: Import fails if dimension mismatch detected
- **Duplicate handling**: Import fails on duplicate UUIDs (primary key violation)

### HNSW Index Rebuild After Import

After importing vectors, HNSW indices are rebuilt automatically on first search:

```rust
async fn get_or_create_index(namespace: &str) -> Result<Arc<RwLock<HnswIndex>>> {
    // Check cache
    if let Some(cached) = self.hnsw_indices.get(namespace) {
        return Ok(cached.clone());
    }

    // Check disk persistence (.hnsw files NOT migrated)
    let persistence_path = self.persistence_path
        .join(format!("{}_{}_{}_{}.hnsw", namespace, self.dimension, self.metric, uuid));

    let index = if persistence_path.exists() {
        // Load from disk (rare: only if .hnsw files manually copied)
        load_hnsw_from_disk(&persistence_path).await?
    } else {
        // Rebuild from vector_metadata + vec_embeddings (common after import)
        build_hnsw_from_database(namespace).await?
    };

    self.hnsw_indices.insert(namespace.to_string(), Arc::new(RwLock::new(Some(index))));
    Ok(index)
}
```

**Rebuild Performance** (Phase 13c.3.2 benchmarks):

| Vectors | Dimension | Rebuild Time | Search Time (after rebuild) |
|---------|-----------|--------------|------------------------------|
| 1K | 768 | ~200ms | <5ms |
| 10K | 768 | ~2s | <10ms |
| 100K | 768 | ~25s | <15ms |

**Why Not Export HNSW Indices?**

1. **Serialization Complexity**: hnsw_rs doesn't support serde (custom MessagePack format required)
2. **Cross-Backend Compatibility**: PostgreSQL uses VectorChord (different HNSW implementation)
3. **Parameter Drift**: HNSW parameters (m, ef_construction) may differ between environments
4. **Rebuild Speed**: <30s for 100K vectors (acceptable for migration)
5. **Correctness**: Rebuilding ensures HNSW graph matches actual data

### Migration Examples

**Example 1: SQLite → PostgreSQL (Development to Production)**

```bash
# 1. Export from SQLite (includes 10K vectors across 4 dimensions)
llmspell storage export --backend sqlite --output dev-data.json

# 2. Verify export structure
jq '.data.vector_embeddings | to_entries | map({dim: .key, count: (.value | length)})' dev-data.json
# Output:
# [
#   {"dim": "384", "count": 2500},
#   {"dim": "768", "count": 5000},
#   {"dim": "1536", "count": 2000},
#   {"dim": "3072", "count": 500}
# ]

# 3. Import to PostgreSQL (vectors inserted into dimension-specific tables)
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"
llmspell storage import --backend postgres --input dev-data.json

# 4. PostgreSQL creates HNSW indices automatically (VectorChord)
# SQLite rebuilds HNSW indices lazily on first search
```

**Example 2: PostgreSQL → SQLite (Production to Development)**

```bash
# 1. Export from PostgreSQL
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"
llmspell storage export --backend postgres --output prod-data.json

# 2. Import to SQLite
llmspell storage import --backend sqlite --input prod-data.json

# 3. First search per namespace triggers HNSW rebuild
# Subsequent searches use cached HNSW index
```

### Roundtrip Verification

Vector embeddings preserve full precision across SQLite ↔ PostgreSQL migrations:

```bash
# Export from SQLite
llmspell storage export --backend sqlite --output export1.json

# Import to PostgreSQL
export DATABASE_URL="postgresql://user:pass@localhost/llmspell"
llmspell storage import --backend postgres --input export1.json

# Export from PostgreSQL
llmspell storage export --backend postgres --output export2.json

# Compare vector embeddings (should be identical)
diff <(jq -S '.data.vector_embeddings' export1.json) \
     <(jq -S '.data.vector_embeddings' export2.json)
# No output = zero data loss
```

**Verified Preservation**:
- ✅ Vector dimensions (384/768/1536/3072)
- ✅ Embedding values (full f32 precision)
- ✅ Metadata (JSON)
- ✅ Scopes (namespace strings)
- ✅ Tenant IDs
- ✅ Timestamps (created_at, updated_at)

### Troubleshooting Export/Import

**Issue: HNSW Index Slow After Import**

**Symptom**: First search takes 30+ seconds after importing 100K vectors

**Cause**: HNSW index rebuilt from database on first search per namespace

**Solution**: Pre-warm indices after import:

```rust
// Pre-warm all namespaces
let namespaces = get_all_namespaces().await?;
for namespace in namespaces {
    vector_storage.search_scoped(dummy_query, &namespace, 1).await?;
    // Triggers HNSW rebuild, subsequent searches fast
}
```

**Issue: Dimension Mismatch After Import**

**Symptom**: `Error: Vector dimension 768 does not match table dimension 384`

**Cause**: Importing vectors into wrong dimension table

**Diagnosis**:

```bash
# Check export structure
jq '.data.vector_embeddings | keys' export.json
# Should show: ["384", "768", "1536", "3072"]
```

**Solution**: Import preserves dimension mapping automatically. If error persists, export file may be corrupted.

**Issue: Missing Vectors After Import**

**Symptom**: Vector count differs between source and target

**Diagnosis**:

```sql
-- SQLite: Check vector counts by dimension
SELECT dimension, COUNT(*) as count
FROM vector_metadata
GROUP BY dimension;

-- PostgreSQL: Check vector counts by dimension
SELECT 384 as dimension, COUNT(*) FROM llmspell.vector_embeddings_384
UNION ALL
SELECT 768, COUNT(*) FROM llmspell.vector_embeddings_768
UNION ALL
SELECT 1536, COUNT(*) FROM llmspell.vector_embeddings_1536
UNION ALL
SELECT 3072, COUNT(*) FROM llmspell.vector_embeddings_3072;
```

**Solution**: Import is transaction-safe and rolls back on error. Check import logs for constraint violations.

### Performance Considerations

**Export Performance** (Phase 13c.3.2):

- **Small datasets** (<1K vectors): <100ms
- **Medium datasets** (10K vectors): ~2s
- **Large datasets** (100K vectors): ~20s
- **Bottleneck**: JSON serialization of f32 arrays

**Import Performance**:

- **Small datasets** (<1K vectors): <200ms
- **Medium datasets** (10K vectors): ~5s (includes HNSW rebuild)
- **Large datasets** (100K vectors): ~60s (includes HNSW rebuild)
- **Bottleneck**: HNSW index construction (not database writes)

**Optimization Tips**:

1. **Batch imports**: Import uses explicit transactions (already optimized)
2. **Lazy HNSW rebuild**: Don't pre-warm indices unless needed
3. **Compression**: Use `gzip export.json` for large datasets (10:1 compression typical)
4. **Parallel export**: Export dimensions in parallel (not implemented, future optimization)

### See Also

- [Storage Migration Internals](storage-migration-internals.md) - Technical deep dive on export/import architecture
- [User Guide: Data Migration](../user-guide/11-data-migration.md) - Complete migration workflows
- [PostgreSQL Guide: Data Migration](postgresql-guide.md#data-migration-postgresql--sqlite---phase-13c32) - PostgreSQL-specific migration details

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
