-- Migration: Create generic key-value store table (Phase 13b.7.1)
-- Purpose: Provide generic storage backend for arbitrary key-value pairs with tenant isolation
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)
--
-- This table serves as the fallback storage for PostgresBackend's StorageBackend trait implementation.
-- Keys that don't match specialized routing patterns (agent:*, workflow:*, etc.) are stored here.
--
-- Design Philosophy:
-- - Simple, fast key-value operations
-- - Tenant-isolated via RLS
-- - Optional metadata as JSONB for extensibility
-- - Binary-safe value storage (BYTEA)

-- Create kv_store table for generic key-value storage
CREATE TABLE IF NOT EXISTS llmspell.kv_store (
    -- Primary key
    kv_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Tenant isolation
    tenant_id VARCHAR(255) NOT NULL,

    -- Key-value pair
    key VARCHAR(500) NOT NULL,
    value BYTEA NOT NULL,

    -- Optional metadata (extensibility)
    metadata JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Unique constraint: One value per (tenant, key) combination
    CONSTRAINT unique_kv_key UNIQUE (tenant_id, key)
);

-- Index for tenant-based queries (RLS performance)
CREATE INDEX IF NOT EXISTS idx_kv_store_tenant
    ON llmspell.kv_store(tenant_id);

-- Index for key prefix scanning (list_keys operations)
CREATE INDEX IF NOT EXISTS idx_kv_store_key_prefix
    ON llmspell.kv_store(tenant_id, key text_pattern_ops);

-- Index for timestamp-based queries (recency, cleanup)
CREATE INDEX IF NOT EXISTS idx_kv_store_updated
    ON llmspell.kv_store(updated_at DESC);

-- GIN index for JSONB metadata queries (optional filtering)
CREATE INDEX IF NOT EXISTS idx_kv_store_metadata_gin
    ON llmspell.kv_store USING GIN(metadata)
    WHERE metadata IS NOT NULL;

-- Enable Row-Level Security on kv_store table
ALTER TABLE llmspell.kv_store ENABLE ROW LEVEL SECURITY;

-- RLS POLICIES: Complete tenant isolation with four policies
-- Pattern: DROP IF EXISTS before CREATE for idempotency

-- SELECT policy: Only see keys for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.kv_store;
CREATE POLICY tenant_isolation_select ON llmspell.kv_store
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert keys for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.kv_store;
CREATE POLICY tenant_isolation_insert ON llmspell.kv_store
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update keys for current tenant
-- Both USING and WITH CHECK ensure tenant_id cannot be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.kv_store;
CREATE POLICY tenant_isolation_update ON llmspell.kv_store
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete keys for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.kv_store;
CREATE POLICY tenant_isolation_delete ON llmspell.kv_store
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Function to update updated_at timestamp automatically
CREATE OR REPLACE FUNCTION llmspell.update_kv_store_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at on value updates
DROP TRIGGER IF EXISTS trigger_kv_store_updated_at ON llmspell.kv_store;
CREATE TRIGGER trigger_kv_store_updated_at
    BEFORE UPDATE ON llmspell.kv_store
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_kv_store_updated_at();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'kv_store';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'kv_store';
-- SELECT indexname FROM pg_indexes WHERE tablename = 'kv_store';
