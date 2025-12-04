-- Migration: Create sessions storage table (Phase 13b.9.1)
-- Purpose: Store session snapshots with lifecycle tracking and expiration
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)
--
-- Session storage tracks full lifecycle including:
-- - Session metadata (status, timestamps, counts)
-- - Session configuration (max_duration, auto_save, retention)
-- - Session state (HashMap of state items)
-- - Artifact references and statistics
-- - Expiration and cleanup management
-- - Lifecycle timestamps (created_at, last_accessed_at, expires_at)

-- Create sessions table for session snapshot persistence
CREATE TABLE IF NOT EXISTS llmspell.sessions (
    -- Tenant isolation (part of composite PK)
    tenant_id VARCHAR(255) NOT NULL,

    -- Primary key (composite with tenant_id)
    session_id UUID NOT NULL,

    -- Full session snapshot (SessionSnapshot serialized as JSONB)
    session_data JSONB NOT NULL,

    -- Extracted fields for efficient queries
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    artifact_count INTEGER NOT NULL DEFAULT 0,

    -- Metadata timestamps
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite primary key: One session per (tenant, session_id)
    PRIMARY KEY (tenant_id, session_id),

    -- Status constraint: Must be valid session status
    CONSTRAINT valid_session_status CHECK (
        status IN ('active', 'archived', 'expired')
    ),

    -- Artifact count constraint: Must be non-negative
    CONSTRAINT non_negative_artifact_count CHECK (artifact_count >= 0)
);

-- Index for tenant-based queries (RLS performance)
CREATE INDEX IF NOT EXISTS idx_sessions_tenant
    ON llmspell.sessions(tenant_id);

-- Index for session lookup by tenant and ID
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_session
    ON llmspell.sessions(tenant_id, session_id);

-- Index for status-based queries (find all active sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_status
    ON llmspell.sessions(status);

-- Index for expiration queries (cleanup expired sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_expires
    ON llmspell.sessions(expires_at)
    WHERE expires_at IS NOT NULL;

-- Index for recent sessions (created_at DESC)
CREATE INDEX IF NOT EXISTS idx_sessions_created
    ON llmspell.sessions(created_at DESC);

-- Index for activity tracking (last_accessed_at DESC)
CREATE INDEX IF NOT EXISTS idx_sessions_accessed
    ON llmspell.sessions(last_accessed_at DESC);

-- GIN index for JSONB queries on session_data
-- Enables fast queries on nested JSON fields like config, state, metadata
CREATE INDEX IF NOT EXISTS idx_sessions_data_gin
    ON llmspell.sessions USING GIN(session_data);

-- Composite index for tenant + status queries (find tenant's active sessions)
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_status
    ON llmspell.sessions(tenant_id, status);

-- Composite index for expiration cleanup by tenant
CREATE INDEX IF NOT EXISTS idx_sessions_tenant_expires
    ON llmspell.sessions(tenant_id, expires_at)
    WHERE expires_at IS NOT NULL;

-- Enable Row-Level Security on sessions table
ALTER TABLE llmspell.sessions ENABLE ROW LEVEL SECURITY;

-- RLS POLICIES: Complete tenant isolation with four policies
-- Pattern: DROP IF EXISTS before CREATE for idempotency

-- SELECT policy: Only see sessions for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.sessions;
CREATE POLICY tenant_isolation_select ON llmspell.sessions
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert sessions for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.sessions;
CREATE POLICY tenant_isolation_insert ON llmspell.sessions
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update sessions for current tenant
-- Both USING and WITH CHECK ensure tenant_id cannot be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.sessions;
CREATE POLICY tenant_isolation_update ON llmspell.sessions
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete sessions for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.sessions;
CREATE POLICY tenant_isolation_delete ON llmspell.sessions
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Function to update updated_at timestamp automatically
CREATE OR REPLACE FUNCTION llmspell.update_sessions_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at on session updates
DROP TRIGGER IF EXISTS trigger_sessions_updated_at ON llmspell.sessions;
CREATE TRIGGER trigger_sessions_updated_at
    BEFORE UPDATE ON llmspell.sessions
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_sessions_updated_at();

-- Function to auto-update last_accessed_at when session is accessed
CREATE OR REPLACE FUNCTION llmspell.update_sessions_accessed_at()
RETURNS TRIGGER AS $$
BEGIN
    -- Only update last_accessed_at if it's been more than 1 minute since last access
    -- (prevents excessive updates on rapid access)
    IF OLD.last_accessed_at IS NULL OR
       NEW.last_accessed_at < (now() - INTERVAL '1 minute') THEN
        NEW.last_accessed_at = now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update last_accessed_at on access
DROP TRIGGER IF EXISTS trigger_sessions_accessed_at ON llmspell.sessions;
CREATE TRIGGER trigger_sessions_accessed_at
    BEFORE UPDATE ON llmspell.sessions
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_sessions_accessed_at();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'sessions';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'sessions';
-- SELECT indexname FROM pg_indexes WHERE tablename = 'sessions';
