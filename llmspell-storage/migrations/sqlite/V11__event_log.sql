-- Migration V11: Event Log Storage (Phase 13c.2.7)
--
-- Creates event_log table for time-series event storage with hybrid schema.
-- Optimized for EventStorage trait queries (pattern, correlation, time range).
--
-- SQLite Adaptations from PostgreSQL V11:
--   - UUID -> TEXT (36-char UUID strings)
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps)
--   - JSONB -> TEXT (JSON stored as text)
--   - BIGINT -> INTEGER (SQLite INTEGER is 64-bit signed)
--   - Partitioning -> Single table with indexes (SQLite doesn't support partitioning)
--   - Partition management functions -> Not applicable (no partitioning)
--   - RLS policies -> Application-level filtering
--   - FORCE ROW LEVEL SECURITY -> Not applicable
--   - GIN indexes -> json_extract() expression indexes
--   - COMMENT ON -> Not applicable (SQLite doesn't support column comments)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: event_log (Time-Series Event Storage)
-- ============================================================================

CREATE TABLE IF NOT EXISTS event_log (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),

    -- Tenant isolation (application-level)
    tenant_id TEXT NOT NULL,

    -- Event identification
    event_id TEXT NOT NULL,

    -- Extracted columns for efficient indexing (hot query paths)
    event_type TEXT NOT NULL,
    correlation_id TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    sequence INTEGER NOT NULL,
    language TEXT NOT NULL,

    -- Full event stored as JSON for flexibility
    payload TEXT NOT NULL, -- JSON (JSONB equivalent)

    -- Unique constraint: One event per (tenant, event_id)
    UNIQUE(tenant_id, event_id),

    -- Unique constraint: Prevent duplicate sequences per tenant
    UNIQUE(tenant_id, sequence)
);

-- ============================================================================
-- Indexes for event_log
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_event_log_tenant ON event_log(tenant_id);

-- Index for EventStorage.get_events_by_correlation_id
-- Composite index: correlation_id + timestamp DESC for ordered retrieval
CREATE INDEX IF NOT EXISTS idx_event_log_correlation
    ON event_log(correlation_id, timestamp DESC);

-- Index for EventStorage.get_events_by_pattern (event_type matching)
-- Composite index: event_type + timestamp DESC for ordered retrieval
CREATE INDEX IF NOT EXISTS idx_event_log_type
    ON event_log(event_type, timestamp DESC);

-- Index for sequence-based ordering (global event ordering)
CREATE INDEX IF NOT EXISTS idx_event_log_sequence
    ON event_log(sequence DESC);

-- Index for EventStorage.get_events_by_time_range
-- Composite index: tenant_id + timestamp for time range queries
CREATE INDEX IF NOT EXISTS idx_event_log_tenant_time
    ON event_log(tenant_id, timestamp DESC);

-- Index for timestamp range queries (cleanup old events)
CREATE INDEX IF NOT EXISTS idx_event_log_timestamp
    ON event_log(timestamp);

-- Index for language-based queries
CREATE INDEX IF NOT EXISTS idx_event_log_language
    ON event_log(language);

-- Composite index for tenant + type + time queries
CREATE INDEX IF NOT EXISTS idx_event_log_tenant_type_time
    ON event_log(tenant_id, event_type, timestamp DESC);

-- JSON indexes for payload queries (using json_extract)
-- Extract common fields from UniversalEvent payload
CREATE INDEX IF NOT EXISTS idx_event_log_payload_context
    ON event_log(json_extract(payload, '$.context'));

CREATE INDEX IF NOT EXISTS idx_event_log_payload_emitter
    ON event_log(json_extract(payload, '$.emitter'));

-- ============================================================================
-- Migration Notes
-- ============================================================================

-- PostgreSQL V11 features NOT ported to SQLite:
--   1. Monthly RANGE partitioning: SQLite doesn't support table partitioning
--      - Use single table with timestamp index for time-range queries
--      - Cleanup old events periodically via DELETE WHERE timestamp < ?
--   2. Partition management functions: Not applicable without partitioning
--      - No create_event_log_partition()
--      - No ensure_future_event_log_partitions()
--      - No cleanup_old_event_log_partitions()
--   3. RLS policies: Application code must filter by tenant_id
--   4. FORCE ROW LEVEL SECURITY: Not applicable
--   5. GIN indexes: Use json_extract() expression indexes
--   6. Column comments: SQLite doesn't support COMMENT ON COLUMN
--
-- Application responsibilities:
--   - Filter all queries with WHERE tenant_id = ?
--   - Cleanup old events periodically:
--       DELETE FROM event_log WHERE timestamp < ?
--   - Handle partition-like queries with timestamp ranges:
--       WHERE timestamp >= ? AND timestamp < ?
--   - Ensure sequence uniqueness (use atomic counter or AUTOINCREMENT)
--
-- Performance considerations:
--   - SQLite table size limit: Practical limit ~140TB (theoretical 281TB)
--   - Recommended event retention: 90 days (auto-cleanup old events)
--   - Expected event volume: 1K-10K events/day per tenant
--   - Storage estimates:
--       1K events/day * 90 days = 90K events per tenant
--       Average event size: 1-5KB (JSON payload)
--       90K events * 2.5KB = ~225MB per tenant
--       100 tenants = ~22.5GB
--   - Index overhead: ~30% of table size
--   - Total storage: ~30GB for 100 tenants with 90-day retention
--
-- Performance targets (from PostgreSQL V11):
--   - store_event: <10ms
--   - get_events_by_correlation_id: <50ms
--   - get_events_by_pattern: <100ms
--   - get_events_by_time_range: <200ms (with indexed time range)
--   - cleanup_old_events: <500ms (bulk DELETE with timestamp index)
--
-- Cleanup strategy (application-level):
--   1. Run daily cleanup job:
--      DELETE FROM event_log WHERE timestamp < strftime('%s', 'now', '-90 days')
--   2. Use VACUUM INCREMENTAL or auto_vacuum for space reclamation
--   3. Monitor table size with:
--      SELECT page_count * page_size FROM pragma_page_count(), pragma_page_size()

-- Insert V11 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (11, 'event_log', 'v11-time-series-events');
