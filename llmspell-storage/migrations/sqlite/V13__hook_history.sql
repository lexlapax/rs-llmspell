-- Migration V13: Hook Execution History (Phase 13c.2.7)
--
-- Creates hook_history table for storing hook executions with replay capabilities.
-- Supports compressed context storage, execution metrics, and retention policies.
--
-- SQLite Adaptations from PostgreSQL V13:
--   - UUID -> TEXT (36-char UUID strings)
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps)
--   - BYTEA -> BLOB (binary data for compressed contexts)
--   - JSONB -> TEXT (JSON stored as text)
--   - TEXT[] (arrays) -> TEXT (JSON array: '["tag1","tag2"]')
--   - INTEGER (Postgres) -> INTEGER (SQLite, both 64-bit)
--   - RLS policies -> Application-level filtering
--   - FORCE ROW LEVEL SECURITY -> Not applicable
--   - GIN indexes -> json_extract() expression indexes
--   - Functions (cleanup_old_hook_executions, get_hook_history_stats) -> Application code
--   - GRANT permissions -> Not applicable (no role system)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: hook_history (Hook Execution History with Compression)
-- ============================================================================

CREATE TABLE IF NOT EXISTS hook_history (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    -- Primary identifiers
    execution_id TEXT NOT NULL,
    tenant_id TEXT NOT NULL,

    -- Hook identification
    hook_id TEXT NOT NULL,
    hook_type TEXT NOT NULL,
    correlation_id TEXT NOT NULL,

    -- Execution data (compressed hook_context stored as BLOB)
    hook_context BLOB NOT NULL,      -- Compressed serialized HookContext (lz4/zstd)
    result_data TEXT NOT NULL,        -- Serialized HookResult as JSON

    -- Execution metrics
    timestamp INTEGER NOT NULL,
    duration_ms INTEGER NOT NULL,

    -- Component information (from HookMetadata)
    triggering_component TEXT NOT NULL,
    component_id TEXT NOT NULL,
    modified_operation INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false, 1=true

    -- Categorization and retention (tags stored as JSON array)
    tags TEXT DEFAULT '[]',  -- JSON array: '["tag1","tag2"]'
    retention_priority INTEGER NOT NULL DEFAULT 0,

    -- Storage metadata
    context_size INTEGER NOT NULL,
    contains_sensitive_data INTEGER NOT NULL DEFAULT 0, -- Boolean: 0=false, 1=true
    metadata TEXT NOT NULL DEFAULT '{}', -- JSON object

    -- Timestamps
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One execution per (tenant, execution_id)
    UNIQUE(tenant_id, execution_id)
);

-- ============================================================================
-- Indexes for hook_history
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_hook_history_tenant ON hook_history(tenant_id);

-- Index for hook execution history queries (most common pattern)
CREATE INDEX IF NOT EXISTS idx_hook_history_hook_time
    ON hook_history(hook_id, timestamp DESC);

-- Index for correlation ID lookups (replay scenarios)
CREATE INDEX IF NOT EXISTS idx_hook_history_correlation
    ON hook_history(correlation_id);

-- Index for hook type filtering
CREATE INDEX IF NOT EXISTS idx_hook_history_type
    ON hook_history(hook_type);

-- Index for tenant isolation + temporal queries
CREATE INDEX IF NOT EXISTS idx_hook_history_tenant_time
    ON hook_history(tenant_id, timestamp DESC);

-- Index for retention policy queries (cleanup old executions)
CREATE INDEX IF NOT EXISTS idx_hook_history_retention
    ON hook_history(retention_priority, timestamp);

-- Index for execution ID lookup
CREATE INDEX IF NOT EXISTS idx_hook_history_execution_id
    ON hook_history(execution_id);

-- Composite index for tenant + hook queries
CREATE INDEX IF NOT EXISTS idx_hook_history_tenant_hook
    ON hook_history(tenant_id, hook_id, timestamp DESC);

-- Composite index for tenant + type queries
CREATE INDEX IF NOT EXISTS idx_hook_history_tenant_type
    ON hook_history(tenant_id, hook_type, timestamp DESC);

-- Index for sensitive data filtering
CREATE INDEX IF NOT EXISTS idx_hook_history_sensitive
    ON hook_history(contains_sensitive_data)
    WHERE contains_sensitive_data = 1;

-- JSON indexes for metadata queries (using json_extract)
CREATE INDEX IF NOT EXISTS idx_hook_history_metadata_triggering
    ON hook_history(json_extract(metadata, '$.triggering_component'));

-- ============================================================================
-- Migration Notes
-- ============================================================================

-- PostgreSQL V13 features NOT ported to SQLite:
--   1. RLS policies: Application code must filter by tenant_id
--   2. FORCE ROW LEVEL SECURITY: Not applicable
--   3. GIN indexes: Use json_extract() expression indexes instead
--   4. TEXT[] arrays: Store as JSON array text '["tag1","tag2"]'
--   5. Functions:
--      - cleanup_old_hook_executions(): Implement in application code
--      - get_hook_history_stats(): Implement in application code
--   6. GRANT permissions: Not applicable (SQLite doesn't have role system)
--   7. COMMENT ON FUNCTION: Not applicable
--
-- Application responsibilities:
--   - Filter all queries with WHERE tenant_id = ?
--   - Cleanup old executions periodically:
--       DELETE FROM hook_history
--       WHERE timestamp < ? AND retention_priority <= ?
--   - Calculate statistics (equivalent to get_hook_history_stats):
--       SELECT COUNT(*), SUM(context_size), MIN(timestamp), MAX(timestamp), AVG(duration_ms)
--       FROM hook_history WHERE tenant_id = ?
--   - Handle tag queries with JSON functions:
--       WHERE json_extract(tags, '$') LIKE '%"tag1"%'
--   - Compression/decompression of hook_context BLOB (use lz4 or zstd)
--
-- Compression strategy:
--   - Use lz4_flex crate for compression (fast, good ratio)
--   - Compress HookContext before storing in hook_context BLOB
--   - Decompress on retrieval
--   - Expected compression ratio: 3-10x for typical contexts
--   - context_size stores original uncompressed size
--
-- Storage estimates:
--   - Average hook context: 1-10KB (compressed to 200-2KB)
--   - Average result_data: 500B-5KB (JSON)
--   - 10K executions/day * 90 days = 900K executions per tenant
--   - 900K * 2.5KB avg = ~2.25GB per tenant
--   - 100 tenants = ~225GB
--   - With compression: ~50-75GB
--
-- Performance targets (from PostgreSQL V13):
--   - store_execution: <10ms
--   - load_execution: <5ms (primary key lookup + decompression)
--   - get_executions_by_correlation_id: <50ms
--   - get_executions_by_hook_id: <100ms (indexed query)
--   - cleanup_old_executions: <500ms (bulk DELETE with indexed timestamp)
--
-- Retention policy (application-level):
--   1. retention_priority levels:
--      - 0: Default (90 days retention)
--      - 1-5: Medium priority (180 days retention)
--      - 6-10: High priority (1 year retention)
--   2. Cleanup job runs daily:
--      DELETE FROM hook_history
--      WHERE timestamp < strftime('%s', 'now', '-90 days')
--        AND retention_priority = 0
--   3. Sensitive data handling:
--      - Flag with contains_sensitive_data = 1
--      - Shorter retention (30 days) regardless of priority
--      - Optional encryption of hook_context BLOB

-- Insert V13 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (13, 'hook_history', 'v13-hook-execution-replay');
