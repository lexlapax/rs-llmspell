-- Migration V6: Agent State Storage (Phase 13c.2.6)
--
-- Creates agent_states table for persistent agent state with versioning and checksum validation.
-- Agent states track execution state, configuration, and metadata for long-running agents.
--
-- SQLite Adaptations from PostgreSQL V6:
--   - UUID -> TEXT (36-char UUID strings via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps via strftime('%s', 'now'))
--   - JSONB -> TEXT (JSON stored as text, indexed via json_extract())
--   - Auto-update trigger -> Manual pattern (SQLite trigger syntax differs)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: agent_states (agent execution state with versioning)
-- ============================================================================

CREATE TABLE IF NOT EXISTS agent_states (
    state_id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    tenant_id TEXT NOT NULL,
    agent_id TEXT NOT NULL,
    agent_type TEXT NOT NULL,
    state_data TEXT NOT NULL, -- JSON (JSONB equivalent)
    schema_version INTEGER NOT NULL DEFAULT 1,
    data_version INTEGER NOT NULL DEFAULT 1,
    checksum TEXT NOT NULL, -- SHA256 hex string
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One state per (tenant, agent)
    UNIQUE(tenant_id, agent_id),

    -- Validation constraints
    CHECK (schema_version > 0),
    CHECK (data_version > 0),
    CHECK (length(checksum) = 64) -- SHA256 produces 64 hex chars
);

-- ============================================================================
-- Indexes for agent_states
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_agent_states_tenant ON agent_states(tenant_id);

-- Index for agent type queries
CREATE INDEX IF NOT EXISTS idx_agent_states_type ON agent_states(agent_type);

-- Index for recently updated agents
CREATE INDEX IF NOT EXISTS idx_agent_states_updated ON agent_states(updated_at DESC);

-- JSON functional indexes (SQLite 3.9+)
-- Index for execution state queries (e.g., WHERE json_extract(state_data, '$.state.execution_state') = 'running')
CREATE INDEX IF NOT EXISTS idx_agent_states_execution_state
    ON agent_states(json_extract(state_data, '$.state.execution_state'));

-- Index for agent name queries
CREATE INDEX IF NOT EXISTS idx_agent_states_metadata_name
    ON agent_states(json_extract(state_data, '$.metadata.name'));

-- ============================================================================
-- Trigger: Auto-increment data_version on state updates
-- ============================================================================

CREATE TRIGGER IF NOT EXISTS trigger_agent_state_version
AFTER UPDATE ON agent_states
FOR EACH ROW
WHEN NEW.state_data != OLD.state_data
BEGIN
    UPDATE agent_states
    SET data_version = data_version + 1,
        updated_at = strftime('%s', 'now')
    WHERE state_id = NEW.state_id;
END;

-- Insert V6 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (6, 'agent_state', 'v6-agent-state-versioning');

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Agent State UPSERT Pattern:
--   INSERT INTO agent_states (tenant_id, agent_id, agent_type, state_data, checksum)
--   VALUES (?1, ?2, ?3, ?4, ?5)
--   ON CONFLICT(tenant_id, agent_id) DO UPDATE SET
--     state_data = excluded.state_data,
--     checksum = excluded.checksum,
--     updated_at = strftime('%s', 'now');
--
-- Agent State Retrieval:
--   SELECT state_id, agent_id, agent_type, state_data, schema_version, data_version,
--          checksum, created_at, updated_at
--   FROM agent_states
--   WHERE tenant_id = ? AND agent_id = ?;
--
-- Checksum Validation:
--   SELECT checksum FROM agent_states WHERE tenant_id = ? AND agent_id = ?;
--   -- Compute SHA256(state_data) and compare
--
-- Tenant Isolation:
--   - No RLS in SQLite (PostgreSQL feature)
--   - Application enforces tenant_id filtering in WHERE clauses
--   - SqliteBackend sets tenant context, SqliteAgentStateStorage uses it
--
-- Type Mappings (PostgreSQL -> SQLite):
--   - UUID -> TEXT (stored as 32-char lowercase hex via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix epoch seconds via strftime('%s', 'now'))
--   - JSONB -> TEXT (parsed via json_extract() for functional indexes)
--   - now() -> strftime('%s', 'now')
--
-- Performance Characteristics:
--   - UPSERT pattern uses ON CONFLICT for atomic state updates
--   - JSON functional indexes accelerate common query patterns
--   - Trigger auto-increments data_version on every state_data change
--   - Target: <10ms write, <5ms read
