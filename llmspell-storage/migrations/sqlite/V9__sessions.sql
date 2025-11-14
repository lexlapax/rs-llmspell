-- Migration V9: Session Storage (Phase 13c.2.7)
--
-- Creates sessions table for session snapshot persistence with lifecycle tracking and expiration.
-- Sessions track full session state including metadata, configuration, state items, and artifact references.
--
-- SQLite Adaptations from PostgreSQL V9:
--   - UUID -> TEXT (36-char UUID strings)
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps via strftime('%s', 'now'))
--   - JSONB -> TEXT (JSON stored as text, indexed via json_extract())
--   - Triggers -> Removed (updated_at/last_accessed_at handled in application code)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--   - Composite PK -> Single TEXT PK with UNIQUE(tenant_id, session_id)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: sessions (session snapshots with lifecycle tracking)
-- ============================================================================

CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    -- Tenant isolation (application-level)
    tenant_id TEXT NOT NULL,

    -- Session identifier
    session_id TEXT NOT NULL,

    -- Full session snapshot (SessionData serialized as JSON)
    session_data TEXT NOT NULL, -- JSON (JSONB equivalent)

    -- Extracted fields for efficient queries
    status TEXT NOT NULL DEFAULT 'active',
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_accessed_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    expires_at INTEGER, -- NULL = no expiration
    artifact_count INTEGER NOT NULL DEFAULT 0,

    -- Metadata timestamps
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One session per (tenant, session_id)
    UNIQUE(tenant_id, session_id),

    -- Status constraint: Must be valid session status
    CHECK (status IN ('active', 'archived', 'expired')),

    -- Artifact count constraint: Must be non-negative
    CHECK (artifact_count >= 0)
);

-- ============================================================================
-- Indexes for sessions
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_sessions_tenant ON sessions(tenant_id);

-- Index for session lookup by tenant and ID
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_session ON sessions(tenant_id, session_id);

-- Index for status-based queries (find all active sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_status ON sessions(status);

-- Index for expiration queries (cleanup expired sessions)
-- Partial index: only index rows with non-null expires_at
CREATE INDEX IF NOT EXISTS idx_sessions_expires
    ON sessions(expires_at)
    WHERE expires_at IS NOT NULL;

-- Index for recent sessions (created_at DESC)
CREATE INDEX IF NOT EXISTS idx_sessions_created ON sessions(created_at DESC);

-- Index for activity tracking (last_accessed_at DESC)
CREATE INDEX IF NOT EXISTS idx_sessions_accessed ON sessions(last_accessed_at DESC);

-- Composite index for tenant + status queries (find tenant's active sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_status ON sessions(tenant_id, status);

-- Composite index for expiration cleanup by tenant
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_expires
    ON sessions(tenant_id, expires_at)
    WHERE expires_at IS NOT NULL;

-- JSON indexes for session_data queries (using json_extract)
-- Note: SQLite doesn't have GIN indexes, use expression indexes instead
CREATE INDEX IF NOT EXISTS idx_sessions_data_status
    ON sessions(json_extract(session_data, '$.status'));

CREATE INDEX IF NOT EXISTS idx_sessions_data_artifact_count
    ON sessions(json_extract(session_data, '$.artifact_count'));

-- ============================================================================
-- Migration Notes
-- ============================================================================

-- PostgreSQL V9 features NOT ported to SQLite:
--   1. RLS policies: Application code must filter by tenant_id
--   2. Triggers for auto-updating timestamps: Application code must update updated_at/last_accessed_at
--   3. JSONB GIN indexes: Use json_extract() expression indexes instead
--   4. Function-based triggers: No update_sessions_updated_at() / update_sessions_accessed_at()
--
-- Application responsibilities:
--   - Filter all queries with WHERE tenant_id = ?
--   - Update updated_at = strftime('%s', 'now') on every update
--   - Update last_accessed_at = strftime('%s', 'now') on get_session (throttled to 1 minute in code)
--   - Cleanup expired sessions periodically via DELETE WHERE expires_at < strftime('%s', 'now')
--
-- Storage estimates (from PostgreSQL V9):
--   - Average session size: 1-5KB (JSONB session_data)
--   - 10K sessions: ~10-50MB
--   - 100K sessions: ~100-500MB
--
-- Performance targets:
--   - create_session: <10ms
--   - get_session: <5ms (indexed lookup)
--   - list_active_sessions: <50ms (tenant+status composite index)
--   - cleanup_expired: <100ms (partial index on expires_at)

-- Insert V9 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (9, 'sessions', 'v9-session-storage');
