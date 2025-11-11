-- Phase 13b.12.1: Hook Execution History with Compression
-- Creates hook_history table for storing hook executions with replay capabilities
-- RLS policies applied for multi-tenant isolation
-- Compression support for large hook contexts (BYTEA storage)

-- ============================================================================
-- Hook History Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.hook_history (
    -- Primary identifiers
    execution_id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,

    -- Hook identification
    hook_id VARCHAR(255) NOT NULL,
    hook_type VARCHAR(255) NOT NULL,
    correlation_id UUID NOT NULL,

    -- Execution data (compressed)
    hook_context BYTEA NOT NULL,        -- Compressed serialized HookContext
    result_data JSONB NOT NULL,         -- Serialized HookResult

    -- Execution metrics
    timestamp TIMESTAMPTZ NOT NULL,
    duration_ms INTEGER NOT NULL,

    -- Component information (from HookMetadata)
    triggering_component VARCHAR(255) NOT NULL,
    component_id VARCHAR(255) NOT NULL,
    modified_operation BOOLEAN NOT NULL DEFAULT false,

    -- Categorization and retention
    tags TEXT[] DEFAULT ARRAY[]::TEXT[],
    retention_priority INTEGER NOT NULL DEFAULT 0,

    -- Storage metadata
    context_size INTEGER NOT NULL,
    contains_sensitive_data BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB NOT NULL DEFAULT '{}'::JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ============================================================================
-- Indexes for Query Performance
-- ============================================================================

-- Index for hook execution history queries (most common pattern)
CREATE INDEX IF NOT EXISTS idx_hook_history_hook_time
    ON llmspell.hook_history(hook_id, timestamp DESC);

-- Index for correlation ID lookups (replay scenarios)
CREATE INDEX IF NOT EXISTS idx_hook_history_correlation
    ON llmspell.hook_history(correlation_id);

-- Index for hook type filtering
CREATE INDEX IF NOT EXISTS idx_hook_history_type
    ON llmspell.hook_history(hook_type);

-- Index for tenant isolation + temporal queries
CREATE INDEX IF NOT EXISTS idx_hook_history_tenant_time
    ON llmspell.hook_history(tenant_id, timestamp DESC);

-- Index for retention policy queries
CREATE INDEX IF NOT EXISTS idx_hook_history_retention
    ON llmspell.hook_history(retention_priority, timestamp);

-- GIN index for JSONB metadata queries
CREATE INDEX IF NOT EXISTS idx_hook_history_metadata
    ON llmspell.hook_history USING GIN(metadata);

-- GIN index for tag array queries
CREATE INDEX IF NOT EXISTS idx_hook_history_tags
    ON llmspell.hook_history USING GIN(tags);

-- ============================================================================
-- Row-Level Security (RLS) Policies
-- ============================================================================

-- Enable RLS on hook_history table
ALTER TABLE llmspell.hook_history ENABLE ROW LEVEL SECURITY;

-- Force RLS even for table owner (Phase 13b.11.0 pattern)
ALTER TABLE llmspell.hook_history FORCE ROW LEVEL SECURITY;

-- Drop existing policies if they exist (idempotent)
DROP POLICY IF EXISTS hook_history_tenant_select ON llmspell.hook_history;
DROP POLICY IF EXISTS hook_history_tenant_insert ON llmspell.hook_history;
DROP POLICY IF EXISTS hook_history_tenant_update ON llmspell.hook_history;
DROP POLICY IF EXISTS hook_history_tenant_delete ON llmspell.hook_history;

-- Policy: SELECT - Users can only see their tenant's hook executions
CREATE POLICY hook_history_tenant_select ON llmspell.hook_history
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: INSERT - Users can only insert hook executions for their tenant
CREATE POLICY hook_history_tenant_insert ON llmspell.hook_history
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: UPDATE - Users can only update their tenant's hook executions
CREATE POLICY hook_history_tenant_update ON llmspell.hook_history
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: DELETE - Users can only delete their tenant's hook executions
CREATE POLICY hook_history_tenant_delete ON llmspell.hook_history
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Grant Permissions to Application Role
-- ============================================================================

-- Grant table permissions to llmspell_app role (created in V12)
-- This is forward-compatible and won't error if role doesn't exist yet
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'llmspell_app') THEN
        GRANT SELECT, INSERT, UPDATE, DELETE ON llmspell.hook_history TO llmspell_app;
    END IF;
END $$;

-- ============================================================================
-- Cleanup and Archiving Functions
-- ============================================================================

-- Function to delete old hook executions based on retention policy
-- Lower priority executions are deleted first, respecting retention_priority
CREATE OR REPLACE FUNCTION llmspell.cleanup_old_hook_executions(
    before_date TIMESTAMPTZ,
    min_retention_priority INTEGER DEFAULT 0
) RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Delete executions older than before_date with priority below threshold
    DELETE FROM llmspell.hook_history
    WHERE timestamp < before_date
      AND retention_priority <= min_retention_priority;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION llmspell.cleanup_old_hook_executions IS
    'Delete hook executions older than specified date with retention priority below threshold';

-- Function to get storage statistics for hook history
CREATE OR REPLACE FUNCTION llmspell.get_hook_history_stats()
RETURNS TABLE (
    total_executions BIGINT,
    total_size_bytes BIGINT,
    executions_by_hook JSONB,
    executions_by_type JSONB,
    oldest_execution TIMESTAMPTZ,
    newest_execution TIMESTAMPTZ,
    avg_duration_ms NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    WITH hook_counts AS (
        SELECT
            hook_id,
            COUNT(*) as count
        FROM llmspell.hook_history
        WHERE tenant_id = current_setting('app.current_tenant_id', true)
        GROUP BY hook_id
    ),
    type_counts AS (
        SELECT
            hook_type,
            COUNT(*) as count
        FROM llmspell.hook_history
        WHERE tenant_id = current_setting('app.current_tenant_id', true)
        GROUP BY hook_type
    )
    SELECT
        COUNT(*)::BIGINT as total_executions,
        (SUM(context_size) + SUM(pg_column_size(result_data)))::BIGINT as total_size_bytes,
        (SELECT jsonb_object_agg(hook_id, count) FROM hook_counts) as executions_by_hook,
        (SELECT jsonb_object_agg(hook_type, count) FROM type_counts) as executions_by_type,
        MIN(timestamp) as oldest_execution,
        MAX(timestamp) as newest_execution,
        AVG(duration_ms)::NUMERIC as avg_duration_ms
    FROM llmspell.hook_history
    WHERE tenant_id = current_setting('app.current_tenant_id', true);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION llmspell.get_hook_history_stats IS
    'Get comprehensive statistics about hook execution history for current tenant';

-- Hook history table ready for Phase 13b.12.2 backend implementation
