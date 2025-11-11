-- Migration: Create workflow state storage table (Phase 13b.8.1)
-- Purpose: Store persistent workflow execution state with lifecycle tracking
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)
--
-- Workflow state tracks full execution lifecycle including:
-- - Workflow configuration and parameters
-- - Current execution step and status
-- - Execution history and step results
-- - Performance statistics and metrics
-- - Checkpoints for resumption capability
-- - Lifecycle timestamps (started_at, completed_at)

-- Create workflow_states table for persistent workflow tracking
CREATE TABLE IF NOT EXISTS llmspell.workflow_states (
    -- Tenant isolation (part of composite PK)
    tenant_id VARCHAR(255) NOT NULL,

    -- Primary key (composite with tenant_id)
    workflow_id VARCHAR(255) NOT NULL,

    -- Workflow identification
    workflow_name VARCHAR(500),

    -- Full workflow state (PersistentWorkflowState serialized as JSONB)
    state_data JSONB NOT NULL,

    -- Extracted fields for efficient queries
    current_step INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',

    -- Lifecycle timestamps
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite primary key: One workflow per (tenant, workflow_id)
    PRIMARY KEY (tenant_id, workflow_id),

    -- Status constraint: Must be valid workflow status
    CONSTRAINT valid_workflow_status CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    ),

    -- Step constraint: Step index must be non-negative
    CONSTRAINT positive_step_index CHECK (current_step >= 0)
);

-- Index for tenant-based queries (RLS performance)
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant
    ON llmspell.workflow_states(tenant_id);

-- Index for workflow lookup by tenant and ID
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant_workflow
    ON llmspell.workflow_states(tenant_id, workflow_id);

-- Index for status-based queries (find all running workflows)
CREATE INDEX IF NOT EXISTS idx_workflow_states_status
    ON llmspell.workflow_states(status);

-- Index for timestamp-based queries (find long-running, recent workflows)
CREATE INDEX IF NOT EXISTS idx_workflow_states_started
    ON llmspell.workflow_states(started_at DESC);

-- Index for completion queries
CREATE INDEX IF NOT EXISTS idx_workflow_states_completed
    ON llmspell.workflow_states(completed_at DESC)
    WHERE completed_at IS NOT NULL;

-- GIN index for JSONB queries on state_data
-- Enables fast queries on nested JSON fields like config, metadata, execution_stats
CREATE INDEX IF NOT EXISTS idx_workflow_states_data_gin
    ON llmspell.workflow_states USING GIN(state_data);

-- Specific JSONB path indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_workflow_states_execution_stats
    ON llmspell.workflow_states((state_data->'execution_stats'));

-- Composite index for tenant + status queries (find tenant's running workflows)
CREATE INDEX IF NOT EXISTS idx_workflow_states_tenant_status
    ON llmspell.workflow_states(tenant_id, status);

-- Enable Row-Level Security on workflow_states table
ALTER TABLE llmspell.workflow_states ENABLE ROW LEVEL SECURITY;

-- RLS POLICIES: Complete tenant isolation with four policies
-- Pattern: DROP IF EXISTS before CREATE for idempotency

-- SELECT policy: Only see workflows for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.workflow_states;
CREATE POLICY tenant_isolation_select ON llmspell.workflow_states
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert workflows for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.workflow_states;
CREATE POLICY tenant_isolation_insert ON llmspell.workflow_states
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update workflows for current tenant
-- Both USING and WITH CHECK ensure tenant_id cannot be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.workflow_states;
CREATE POLICY tenant_isolation_update ON llmspell.workflow_states
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete workflows for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.workflow_states;
CREATE POLICY tenant_isolation_delete ON llmspell.workflow_states
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Function to update last_updated timestamp automatically
CREATE OR REPLACE FUNCTION llmspell.update_workflow_states_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update last_updated on workflow state updates
DROP TRIGGER IF EXISTS trigger_workflow_states_updated_at ON llmspell.workflow_states;
CREATE TRIGGER trigger_workflow_states_updated_at
    BEFORE UPDATE ON llmspell.workflow_states
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_workflow_states_updated_at();

-- Function to auto-update completed_at when status changes to terminal state
CREATE OR REPLACE FUNCTION llmspell.update_workflow_completed_at()
RETURNS TRIGGER AS $$
BEGIN
    -- Set completed_at when transitioning to terminal status
    IF NEW.status IN ('completed', 'failed', 'cancelled') AND OLD.status NOT IN ('completed', 'failed', 'cancelled') THEN
        NEW.completed_at = now();
    END IF;

    -- Set started_at when transitioning to running (if not already set)
    IF NEW.status = 'running' AND OLD.status = 'pending' AND NEW.started_at IS NULL THEN
        NEW.started_at = now();
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update lifecycle timestamps
DROP TRIGGER IF EXISTS trigger_workflow_lifecycle ON llmspell.workflow_states;
CREATE TRIGGER trigger_workflow_lifecycle
    BEFORE UPDATE ON llmspell.workflow_states
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_workflow_completed_at();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'workflow_states';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'workflow_states';
-- SELECT indexname FROM pg_indexes WHERE tablename = 'workflow_states';
