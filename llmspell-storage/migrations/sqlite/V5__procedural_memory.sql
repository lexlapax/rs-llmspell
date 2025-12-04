-- Migration V5: Procedural Memory Patterns (Phase 13c.2.5)
--
-- Creates procedural_patterns table for tracking learned state transition patterns.
-- Procedural memory stores patterns from repeated interactions:
--   - State transition patterns (workflow sequences)
--   - When a transition `scope:key → value` occurs ≥3 times, it becomes a learned pattern
--
-- SQLite Adaptations from PostgreSQL V5:
--   - UUID -> TEXT (36-char UUID strings via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps via strftime('%s', 'now'))
--   - Auto-update trigger -> Manual pattern (SQLite trigger syntax differs)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: procedural_patterns (state transition pattern tracking)
-- ============================================================================

CREATE TABLE IF NOT EXISTS procedural_patterns (
    pattern_id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    tenant_id TEXT NOT NULL,

    -- Pattern identity: (scope, key, value) uniquely identifies a transition
    scope TEXT NOT NULL,
    key TEXT NOT NULL,
    value TEXT NOT NULL,

    -- Pattern tracking metrics
    frequency INTEGER NOT NULL DEFAULT 1,
    first_seen INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    last_seen INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One pattern per (tenant, scope, key, value) combination
    UNIQUE(tenant_id, scope, key, value),

    -- Frequency must be positive
    CHECK (frequency > 0)
);

-- ============================================================================
-- Indexes for procedural_patterns
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_tenant ON procedural_patterns(tenant_id);

-- Index for pattern queries by scope and key
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_scope_key ON procedural_patterns(scope, key);

-- Index for frequency-based queries (top patterns, ORDER BY frequency DESC)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_frequency ON procedural_patterns(frequency DESC);

-- Index for time-based queries (pattern aging, cleanup)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_last_seen ON procedural_patterns(last_seen DESC);

-- Composite partial index for learned pattern queries (frequency >= 3)
-- Accelerates: WHERE tenant_id = ? AND scope = ? AND key = ? AND value = ? AND frequency >= 3
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_lookup
    ON procedural_patterns(tenant_id, scope, key, value)
    WHERE frequency >= 3;

-- Insert V5 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (5, 'procedural_memory', 'v5-procedural-patterns');

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Pattern Recording (UPSERT with frequency increment):
--   INSERT INTO procedural_patterns (tenant_id, scope, key, value, frequency, last_seen)
--   VALUES (?1, ?2, ?3, ?4, 1, strftime('%s', 'now'))
--   ON CONFLICT(tenant_id, scope, key, value) DO UPDATE SET
--     frequency = frequency + 1,
--     last_seen = strftime('%s', 'now'),
--     updated_at = strftime('%s', 'now')
--   RETURNING frequency;
--
-- Pattern Retrieval (learned patterns, frequency >= 3):
--   SELECT scope, key, value, frequency, first_seen, last_seen
--   FROM procedural_patterns
--   WHERE tenant_id = ? AND frequency >= 3
--   ORDER BY frequency DESC;
--
-- Pattern Frequency Lookup (specific transition):
--   SELECT frequency FROM procedural_patterns
--   WHERE tenant_id = ? AND scope = ? AND key = ? AND value = ?;
--
-- Tenant Isolation:
--   - No RLS in SQLite (PostgreSQL feature)
--   - Application enforces tenant_id filtering in WHERE clauses
--   - SqliteBackend sets tenant context, SqliteProceduralStorage uses it
--
-- Type Mappings (PostgreSQL -> SQLite):
--   - UUID -> TEXT (stored as 32-char lowercase hex via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix epoch seconds via strftime('%s', 'now'))
--   - now() -> strftime('%s', 'now')
--
-- Performance Characteristics:
--   - UPSERT pattern uses ON CONFLICT for atomic frequency increment
--   - Partial index (frequency >= 3) accelerates learned pattern queries
--   - Composite index (scope, key) for pattern lookup by state key
--   - Target: <5ms pattern insert, <10ms pattern query
