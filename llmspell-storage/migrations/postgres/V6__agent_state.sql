-- Migration: Create agent state storage table (Phase 13b.7.1)
-- Purpose: Store persistent agent state with versioning and integrity checks
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)
--
-- Agent state tracks the full operational state of agent instances including:
-- - Conversation history and context
-- - Tool usage statistics
-- - Execution state and workflow position
-- - Metadata and capabilities
-- - Hook registrations and correlation tracking

-- Create agent_states table for persistent agent state
CREATE TABLE IF NOT EXISTS llmspell.agent_states (
    -- Primary key
    state_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Tenant isolation
    tenant_id VARCHAR(255) NOT NULL,

    -- Agent identification
    agent_id VARCHAR(255) NOT NULL,
    agent_type VARCHAR(100) NOT NULL,

    -- State data (full PersistentAgentState serialized as JSONB)
    state_data JSONB NOT NULL,

    -- Versioning and integrity
    schema_version INTEGER NOT NULL DEFAULT 1,
    data_version INTEGER NOT NULL DEFAULT 1,
    checksum VARCHAR(64) NOT NULL, -- SHA-256 hash of state_data

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Unique constraint: One active state per (tenant, agent_id)
    CONSTRAINT unique_agent_state UNIQUE (tenant_id, agent_id),

    -- Schema version must be positive
    CONSTRAINT positive_schema_version CHECK (schema_version > 0),
    CONSTRAINT positive_data_version CHECK (data_version > 0)
);

-- Index for tenant-based queries (RLS performance)
CREATE INDEX IF NOT EXISTS idx_agent_states_tenant
    ON llmspell.agent_states(tenant_id);

-- Index for agent_type queries (filter by type)
CREATE INDEX IF NOT EXISTS idx_agent_states_type
    ON llmspell.agent_states(agent_type);

-- Index for timestamp-based queries (recency, cleanup)
CREATE INDEX IF NOT EXISTS idx_agent_states_updated
    ON llmspell.agent_states(updated_at DESC);

-- GIN index for JSONB queries on state_data
-- Enables fast queries on nested JSON fields like execution_state, metadata, etc.
CREATE INDEX IF NOT EXISTS idx_agent_states_data_gin
    ON llmspell.agent_states USING GIN(state_data);

-- Specific JSONB path indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_agent_states_execution_state
    ON llmspell.agent_states((state_data->'state'->>'execution_state'));

CREATE INDEX IF NOT EXISTS idx_agent_states_metadata_name
    ON llmspell.agent_states((state_data->'metadata'->>'name'));

-- Enable Row-Level Security on agent_states table
ALTER TABLE llmspell.agent_states ENABLE ROW LEVEL SECURITY;

-- RLS POLICIES: Complete tenant isolation with four policies
-- Pattern: DROP IF EXISTS before CREATE for idempotency

-- SELECT policy: Only see states for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.agent_states;
CREATE POLICY tenant_isolation_select ON llmspell.agent_states
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert states for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.agent_states;
CREATE POLICY tenant_isolation_insert ON llmspell.agent_states
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update states for current tenant
-- Both USING and WITH CHECK ensure tenant_id cannot be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.agent_states;
CREATE POLICY tenant_isolation_update ON llmspell.agent_states
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete states for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.agent_states;
CREATE POLICY tenant_isolation_delete ON llmspell.agent_states
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Function to update updated_at timestamp automatically
CREATE OR REPLACE FUNCTION llmspell.update_agent_states_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at on state updates
DROP TRIGGER IF EXISTS trigger_agent_states_updated_at ON llmspell.agent_states;
CREATE TRIGGER trigger_agent_states_updated_at
    BEFORE UPDATE ON llmspell.agent_states
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_agent_states_updated_at();

-- Function to auto-increment data_version on updates
CREATE OR REPLACE FUNCTION llmspell.increment_agent_state_version()
RETURNS TRIGGER AS $$
BEGIN
    -- Only increment if state_data actually changed
    IF NEW.state_data IS DISTINCT FROM OLD.state_data THEN
        NEW.data_version = OLD.data_version + 1;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-increment data_version when state changes
DROP TRIGGER IF EXISTS trigger_agent_state_version ON llmspell.agent_states;
CREATE TRIGGER trigger_agent_state_version
    BEFORE UPDATE ON llmspell.agent_states
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.increment_agent_state_version();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'agent_states';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'agent_states';
-- SELECT indexname FROM pg_indexes WHERE tablename = 'agent_states';
