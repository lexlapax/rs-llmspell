-- Migration V10: Artifact Storage (Phase 13c.2.7)
--
-- Creates artifact_content and artifacts tables for content-addressed artifact storage
-- with deduplication and dual storage strategy (BLOB for all sizes in SQLite).
--
-- SQLite Adaptations from PostgreSQL V10:
--   - UUID -> TEXT (36-char UUID strings)
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps)
--   - JSONB -> TEXT (JSON stored as text)
--   - BYTEA/Large Object -> BLOB (SQLite stores all BLOBs efficiently)
--   - OID -> Not needed (no Large Object system)
--   - Triggers -> Removed (reference counting handled in application code)
--   - RLS policies -> Application-level filtering
--   - Composite PK -> Single TEXT PK with UNIQUE constraints
--   - TEXT[] (arrays) -> TEXT (JSON array: '["tag1","tag2"]')
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)
--   - V9: Sessions table (foreign key reference)

-- ============================================================================
-- Table: artifact_content (Content-Addressed with Deduplication)
-- ============================================================================

CREATE TABLE IF NOT EXISTS artifact_content (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    -- Tenant isolation (application-level)
    tenant_id TEXT NOT NULL,

    -- Content-addressed key (blake3 hash, 64 hex chars)
    content_hash TEXT NOT NULL,

    -- Content storage (BLOB for all sizes - SQLite handles efficiently)
    data BLOB NOT NULL,

    -- Content metadata
    size_bytes INTEGER NOT NULL,
    is_compressed INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false, 1=true
    original_size_bytes INTEGER,

    -- Deduplication tracking (managed by application code)
    reference_count INTEGER NOT NULL DEFAULT 1,

    -- Lifecycle timestamps
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_accessed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One content per (tenant, hash)
    UNIQUE(tenant_id, content_hash),

    -- Reference count constraint
    CHECK (reference_count > 0),

    -- Size constraint (max 100MB)
    CHECK (size_bytes <= 104857600)
);

-- ============================================================================
-- Table: artifacts (Metadata and References)
-- ============================================================================

CREATE TABLE IF NOT EXISTS artifacts (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    -- Tenant isolation (application-level)
    tenant_id TEXT NOT NULL,

    -- Artifact identifier (format: "{session_id}:{sequence}:{content_hash}")
    artifact_id TEXT NOT NULL,

    -- Session reference (extracted from artifact_id)
    session_id TEXT NOT NULL,

    -- Sequence number within session
    sequence INTEGER NOT NULL,

    -- Content reference (blake3 hash)
    content_hash TEXT NOT NULL,

    -- Artifact metadata (full ArtifactMetadata as JSON)
    metadata TEXT NOT NULL, -- JSON (JSONB equivalent)

    -- Extracted fields for efficient queries
    name TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    created_by TEXT,

    -- Version tracking
    version INTEGER NOT NULL DEFAULT 1,
    parent_artifact_id TEXT,

    -- Tags for searching (JSON array: '["tag1","tag2"]')
    tags TEXT,

    -- Lifecycle timestamps
    stored_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One artifact per (tenant, artifact_id)
    UNIQUE(tenant_id, artifact_id),

    -- Unique constraint: One (tenant, session, sequence)
    UNIQUE(tenant_id, session_id, sequence),

    -- Foreign key to content table
    FOREIGN KEY (tenant_id, content_hash)
        REFERENCES artifact_content(tenant_id, content_hash)
        ON DELETE RESTRICT, -- Cannot delete content while artifacts reference it

    -- Foreign key to sessions table
    FOREIGN KEY (tenant_id, session_id)
        REFERENCES sessions(tenant_id, session_id)
        ON DELETE CASCADE, -- Delete artifacts when session is deleted

    -- Version constraint
    CHECK (version > 0),

    -- Sequence constraint
    CHECK (sequence >= 0)
);

-- ============================================================================
-- Indexes for artifact_content
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_artifact_content_tenant ON artifact_content(tenant_id);

-- Index for content hash lookup (deduplication queries)
CREATE INDEX IF NOT EXISTS idx_artifact_content_hash ON artifact_content(tenant_id, content_hash);

-- Index for reference count queries (cleanup zero-ref content)
CREATE INDEX IF NOT EXISTS idx_artifact_content_refcount
    ON artifact_content(reference_count)
    WHERE reference_count = 0;

-- Index for access tracking
CREATE INDEX IF NOT EXISTS idx_artifact_content_accessed ON artifact_content(last_accessed_at DESC);

-- ============================================================================
-- Indexes for artifacts
-- ============================================================================

-- Index for tenant-based queries
CREATE INDEX IF NOT EXISTS idx_artifacts_tenant ON artifacts(tenant_id);

-- Index for session-based queries (list all artifacts in session)
CREATE INDEX IF NOT EXISTS idx_artifacts_session ON artifacts(tenant_id, session_id, created_at DESC);

-- Index for artifact type queries
CREATE INDEX IF NOT EXISTS idx_artifacts_type ON artifacts(artifact_type, created_at DESC);

-- Index for content hash lookup (find all artifacts with same content)
CREATE INDEX IF NOT EXISTS idx_artifacts_content ON artifacts(tenant_id, content_hash);

-- Index for name searches
CREATE INDEX IF NOT EXISTS idx_artifacts_name ON artifacts(name);

-- Index for created_at range queries
CREATE INDEX IF NOT EXISTS idx_artifacts_created ON artifacts(created_at DESC);

-- Index for size-based queries
CREATE INDEX IF NOT EXISTS idx_artifacts_size ON artifacts(size_bytes DESC);

-- Composite index for tenant + type queries
CREATE INDEX IF NOT EXISTS idx_artifacts_tenant_type ON artifacts(tenant_id, artifact_type, created_at DESC);

-- JSON indexes for metadata queries (using json_extract)
CREATE INDEX IF NOT EXISTS idx_artifacts_metadata_type
    ON artifacts(json_extract(metadata, '$.type'));

CREATE INDEX IF NOT EXISTS idx_artifacts_metadata_created_by
    ON artifacts(json_extract(metadata, '$.created_by'));

-- ============================================================================
-- Migration Notes
-- ============================================================================

-- PostgreSQL V10 features NOT ported to SQLite:
--   1. RLS policies: Application code must filter by tenant_id
--   2. Triggers for reference counting: Application code must:
--      - Increment refcount on INSERT into artifacts
--      - Decrement refcount on DELETE from artifacts
--      - Update last_accessed_at on SELECT from artifact_content
--   3. Large Object storage: SQLite BLOB handles all sizes efficiently
--   4. JSONB GIN indexes: Use json_extract() expression indexes instead
--   5. TEXT[] arrays: Store as JSON array text '["tag1","tag2"]'
--   6. Auto-update triggers: Application must update updated_at manually
--
-- Application responsibilities:
--   - Filter all queries with WHERE tenant_id = ?
--   - Reference counting:
--       INSERT artifacts: UPDATE artifact_content SET reference_count = reference_count + 1
--       DELETE artifacts: UPDATE artifact_content SET reference_count = reference_count - 1
--   - Cleanup zero-ref content: DELETE FROM artifact_content WHERE reference_count = 0
--   - Update updated_at = strftime('%s', 'now') on every update
--   - Update last_accessed_at when content is accessed (throttled to 1 minute)
--
-- Deduplication strategy:
--   1. Compute blake3 hash of content
--   2. Check if content_hash exists in artifact_content
--   3. If exists: reuse content, increment reference_count
--   4. If not: insert new content with reference_count = 1
--   5. Always insert artifact metadata in artifacts table
--
-- Storage estimates (from PostgreSQL V10):
--   - Small artifacts (<1KB): ~1.5KB per artifact (metadata overhead)
--   - Medium artifacts (100KB): ~100KB per unique content
--   - Large artifacts (10MB): ~10MB per unique content
--   - Deduplication saves: 50-90% with high content reuse
--   - 10K artifacts (50% unique): ~50-500MB depending on average size
--
-- Performance targets:
--   - store_artifact: <20ms (deduplication check + 2 INSERTs)
--   - get_artifact: <10ms (JOIN artifacts + artifact_content)
--   - list_session_artifacts: <50ms (indexed session lookup)
--   - delete_artifact: <15ms (DELETE + refcount decrement)
