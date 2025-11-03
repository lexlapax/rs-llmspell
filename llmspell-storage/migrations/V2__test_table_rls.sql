-- Migration: Create test table with RLS policies (Phase 13b.3.2)
-- Purpose: Validate RLS infrastructure before applying to production tables
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)

-- Create test table for RLS validation
CREATE TABLE IF NOT EXISTS llmspell.test_data (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create index for tenant queries (performance)
CREATE INDEX IF NOT EXISTS idx_test_data_tenant ON llmspell.test_data(tenant_id);

-- Enable Row-Level Security on test_data table
ALTER TABLE llmspell.test_data ENABLE ROW LEVEL SECURITY;

-- CREATE RLS POLICIES: Four policies for complete tenant isolation
-- Use DROP IF EXISTS before CREATE for idempotency (CREATE POLICY doesn't support IF NOT EXISTS)

-- SELECT policy: Only see rows for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.test_data;
CREATE POLICY tenant_isolation_select ON llmspell.test_data
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert rows for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.test_data;
CREATE POLICY tenant_isolation_insert ON llmspell.test_data
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update rows for current tenant
-- Both USING and WITH CHECK ensure tenant_id can't be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.test_data;
CREATE POLICY tenant_isolation_update ON llmspell.test_data
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete rows for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.test_data;
CREATE POLICY tenant_isolation_delete ON llmspell.test_data
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Verification: Table should have RLS enabled and 4 policies
-- Query to verify (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'test_data';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'test_data';
