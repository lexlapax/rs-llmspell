-- Migration V3: Vector Embeddings Storage (Phase 13c.2.2 - Task 13c.2.2)
-- Adds sqlite-vec integration for brute-force vector search
--
-- Dependencies:
-- - vec0.dylib/vec0.so extension must be loaded (handled by SqliteBackend::new())
-- - sqlite-vec v0.1.7-alpha.2 or later
--
-- Architecture:
-- - vec_embeddings_* virtual tables: One per dimension (384, 768, 1536, 3072)
-- - vector_metadata: Tenant/scope/timestamps (vec0 only stores rowid + embedding)
-- - No HNSW index (brute-force search, O(N) complexity)
-- - Future: vectorlite-rs (Task 13c.2.2a) will add HNSW for 3-100x speedup

-- =============================================================================
-- Vector Embeddings Virtual Tables (sqlite-vec vec0 module)
-- =============================================================================
-- NOTE: Task 13c.2.8.16 - vec0 tables creation moved to runtime (optional)
-- These tables are only created if vec0 extension is available
-- Default vector storage now uses vectorlite-rs (HNSW-indexed, Task 13c.2.2a)
--
-- vec0 tables are created at runtime by SqliteVectorStorage when needed
-- This allows tests to run without vec0.so extension file

-- =============================================================================
-- Vector Metadata Table
-- =============================================================================
-- vec0 virtual tables only store rowid + embedding blob
-- Metadata table provides tenant isolation, scope filtering, timestamps, and JSON metadata

CREATE TABLE IF NOT EXISTS vector_metadata (
    -- Primary key (maps to vec0 rowid)
    rowid INTEGER PRIMARY KEY,

    -- Vector identifier (UUID)
    id TEXT NOT NULL UNIQUE,

    -- Tenant isolation
    tenant_id TEXT NOT NULL,

    -- Scope filtering (session:xxx, user:xxx, global)
    scope TEXT NOT NULL,

    -- Vector dimension (384, 768, 1536, 3072)
    -- Determines which vec_embeddings_* table to use
    dimension INTEGER NOT NULL CHECK (dimension IN (384, 768, 1536, 3072)),

    -- JSON metadata (searchable via json_extract)
    metadata TEXT NOT NULL DEFAULT '{}',

    -- Timestamps
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

-- =============================================================================
-- Indexes for Query Performance
-- =============================================================================

-- Tenant + scope filtering (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_vector_metadata_tenant_scope
ON vector_metadata(tenant_id, scope);

-- UUID lookup
CREATE INDEX IF NOT EXISTS idx_vector_metadata_id
ON vector_metadata(id);

-- Dimension filtering (for routing to correct vec_embeddings_* table)
CREATE INDEX IF NOT EXISTS idx_vector_metadata_dimension
ON vector_metadata(dimension);

-- Timestamp range queries
CREATE INDEX IF NOT EXISTS idx_vector_metadata_created
ON vector_metadata(created_at);

-- =============================================================================
-- Migration Notes
-- =============================================================================
-- 1. Performance: Brute-force search O(N), suitable for <100K vectors
--    - 10K vectors: ~10-50ms search latency
--    - 100K vectors: ~100-500ms search latency
--    - For HNSW indexing (3-100x faster), see Task 13c.2.2a (vectorlite-rs)
--
-- 2. K-NN Search Pattern (application code):
--    SELECT m.id, m.metadata, v.distance
--    FROM vec_embeddings_768 v
--    JOIN vector_metadata m ON v.rowid = m.rowid
--    WHERE m.tenant_id = ?1 AND m.scope = ?2
--      AND v.embedding MATCH ?3
--    ORDER BY v.distance
--    LIMIT ?4
--
-- 3. Dimension Routing (application logic):
--    - Query vector_metadata.dimension to determine table
--    - Route to vec_embeddings_{384|768|1536|3072}
--    - Single query cannot search across dimensions
--
-- 4. Storage Estimates:
--    - 384-dim: ~1.5KB per vector + metadata
--    - 768-dim: ~3KB per vector + metadata
--    - 1536-dim: ~6KB per vector + metadata
--    - 3072-dim: ~12KB per vector + metadata
--
-- 5. Extension Loading:
--    - SqliteBackend::new() loads vec0 extension automatically
--    - If extension missing, vector tables won't be accessible
--    - Build instructions in error message when load fails
