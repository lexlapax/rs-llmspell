-- Migration V8: Workflow State Storage (Phase 13c.2.6)
--
-- Creates workflow_states table for workflow execution state with lifecycle tracking.
-- Workflow states track execution progress, status transitions, and timing for resumable workflows.
--
-- SQLite Adaptations from PostgreSQL V8:
--   - UUID -> TEXT (36-char UUID strings via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps via strftime('%s', 'now'))
--   - JSONB -> TEXT (JSON stored as text for state_data)
--   - Auto-update trigger -> Manual pattern (lifecycle transitions via trigger)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: workflow_states (workflow execution state with lifecycle)
-- ============================================================================

CREATE TABLE IF NOT EXISTS workflow_states (
    tenant_id TEXT NOT NULL,
    workflow_id TEXT NOT NULL,
    workflow_name TEXT,
    state_data TEXT NOT NULL, -- JSON (full PersistentWorkflowState)
    current_step INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    started_at INTEGER,
    completed_at INTEGER,
    last_updated INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Composite primary key: (tenant_id, workflow_id)
    PRIMARY KEY (tenant_id, workflow_id),

    -- Validation constraints
    CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    CHECK (current_step >= 0)
);

-- ============================================================================
-- Indexes for workflow_states
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant ON workflow_states(tenant_id);

-- Index for workflow lookup by (tenant, workflow_id)
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant_workflow ON workflow_states(tenant_id, workflow_id);

-- Index for status queries (list workflows by status)
CREATE INDEX IF NOT EXISTS idx_workflow_states_status ON workflow_states(status);

-- Index for recently started workflows
CREATE INDEX IF NOT EXISTS idx_workflow_states_started ON workflow_states(started_at DESC);

-- Partial index for completed workflows (excludes NULL completed_at)
CREATE INDEX IF NOT EXISTS idx_workflow_states_completed
    ON workflow_states(completed_at DESC)
    WHERE completed_at IS NOT NULL;

-- Composite index for tenant + status queries
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant_status ON workflow_states(tenant_id, status);

-- ============================================================================
-- Trigger: Auto-update lifecycle timestamps
-- ============================================================================

CREATE TRIGGER IF NOT EXISTS trigger_workflow_lifecycle
AFTER UPDATE OF status ON workflow_states
FOR EACH ROW
BEGIN
    -- Set started_at when transitioning from pending to running
    UPDATE workflow_states
    SET started_at = strftime('%s', 'now'),
        last_updated = strftime('%s', 'now')
    WHERE tenant_id = NEW.tenant_id
      AND workflow_id = NEW.workflow_id
      AND NEW.status = 'running'
      AND OLD.status = 'pending'
      AND started_at IS NULL;

    -- Set completed_at when transitioning to terminal state
    UPDATE workflow_states
    SET completed_at = strftime('%s', 'now'),
        last_updated = strftime('%s', 'now')
    WHERE tenant_id = NEW.tenant_id
      AND workflow_id = NEW.workflow_id
      AND NEW.status IN ('completed', 'failed', 'cancelled')
      AND OLD.status NOT IN ('completed', 'failed', 'cancelled')
      AND completed_at IS NULL;

    -- Always update last_updated on any status change
    UPDATE workflow_states
    SET last_updated = strftime('%s', 'now')
    WHERE tenant_id = NEW.tenant_id
      AND workflow_id = NEW.workflow_id
      AND NEW.status != OLD.status;
END;

-- Insert V8 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (8, 'workflow_states', 'v8-workflow-lifecycle');

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Workflow State UPSERT Pattern (save_state):
--   INSERT INTO workflow_states (tenant_id, workflow_id, workflow_name, state_data, current_step, status)
--   VALUES (?1, ?2, ?3, ?4, ?5, ?6)
--   ON CONFLICT(tenant_id, workflow_id) DO UPDATE SET
--     workflow_name = excluded.workflow_name,
--     state_data = excluded.state_data,
--     current_step = excluded.current_step,
--     status = excluded.status,
--     last_updated = strftime('%s', 'now');
--
-- Workflow State Retrieval (load_state):
--   SELECT workflow_id, workflow_name, state_data, current_step, status,
--          started_at, completed_at, last_updated, created_at
--   FROM workflow_states
--   WHERE tenant_id = ? AND workflow_id = ?;
--
-- Status Update (update_status):
--   UPDATE workflow_states
--   SET status = ?, last_updated = strftime('%s', 'now')
--   WHERE tenant_id = ? AND workflow_id = ?;
--   -- Trigger will automatically set started_at or completed_at as needed
--
-- List Workflows (list_workflows with optional status filter):
--   -- All workflows:
--   SELECT workflow_id FROM workflow_states WHERE tenant_id = ?;
--   -- By status:
--   SELECT workflow_id FROM workflow_states WHERE tenant_id = ? AND status = ?;
--
-- Delete Workflow State (delete_state):
--   DELETE FROM workflow_states WHERE tenant_id = ? AND workflow_id = ?;
--
-- Tenant Isolation:
--   - No RLS in SQLite (PostgreSQL feature)
--   - Application enforces tenant_id filtering in WHERE clauses
--   - SqliteBackend sets tenant context, SqliteWorkflowStateStorage uses it
--
-- Type Mappings (PostgreSQL -> SQLite):
--   - UUID -> TEXT (stored as string, not using randomblob for primary key)
--   - TIMESTAMPTZ -> INTEGER (Unix epoch seconds via strftime('%s', 'now'))
--   - JSONB -> TEXT (JSON stored as text, parsed in application layer)
--   - now() -> strftime('%s', 'now')
--
-- Lifecycle State Machine:
--   pending ─> running ─┬─> completed
--                       ├─> failed
--                       └─> cancelled
--
-- Performance Characteristics:
--   - UPSERT pattern uses ON CONFLICT for atomic workflow updates
--   - Trigger automatically manages lifecycle timestamps (started_at, completed_at)
--   - Composite (tenant_id, workflow_id) primary key for efficient lookups
--   - Partial index on completed_at optimizes completed workflow queries
--   - Target: <10ms write, <5ms read, <50ms list by status (1000 workflows)
