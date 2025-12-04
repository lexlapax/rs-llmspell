-- Migration: Create procedural memory patterns table (Phase 13b.6.1)
-- Purpose: Store learned patterns from state transitions for procedural memory
-- Dependencies: V1__initial_setup.sql (llmspell schema exists)
--
-- Procedural memory tracks state transition patterns that occur repeatedly.
-- When a transition `scope:key → value` occurs ≥3 times, it becomes a learned pattern.
-- This enables the system to recognize and predict common state changes.

-- Create procedural_patterns table for pattern storage
CREATE TABLE IF NOT EXISTS llmspell.procedural_patterns (
    pattern_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,

    -- Pattern identity: (scope, key, value) uniquely identifies a transition
    scope VARCHAR(500) NOT NULL,
    key VARCHAR(500) NOT NULL,
    value TEXT NOT NULL,

    -- Pattern tracking metrics
    frequency INTEGER NOT NULL DEFAULT 1,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Unique constraint: One pattern per (tenant, scope, key, value) combination
    CONSTRAINT unique_procedural_pattern UNIQUE (tenant_id, scope, key, value),

    -- Frequency must be positive
    CONSTRAINT positive_frequency CHECK (frequency > 0)
);

-- Index for tenant-based queries (RLS performance)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_tenant
    ON llmspell.procedural_patterns(tenant_id);

-- Index for pattern queries by scope and key
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_scope_key
    ON llmspell.procedural_patterns(scope, key);

-- Index for frequency-based queries (top patterns)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_frequency
    ON llmspell.procedural_patterns(frequency DESC);

-- Index for time-based queries (pattern aging, cleanup)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_last_seen
    ON llmspell.procedural_patterns(last_seen DESC);

-- Composite index for learned pattern queries (scope, key, value with frequency filter)
CREATE INDEX IF NOT EXISTS idx_procedural_patterns_lookup
    ON llmspell.procedural_patterns(tenant_id, scope, key, value)
    WHERE frequency >= 3;

-- Enable Row-Level Security on procedural_patterns table
ALTER TABLE llmspell.procedural_patterns ENABLE ROW LEVEL SECURITY;

-- RLS POLICIES: Complete tenant isolation with four policies
-- Pattern: DROP IF EXISTS before CREATE for idempotency

-- SELECT policy: Only see patterns for current tenant
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.procedural_patterns;
CREATE POLICY tenant_isolation_select ON llmspell.procedural_patterns
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert patterns for current tenant
DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.procedural_patterns;
CREATE POLICY tenant_isolation_insert ON llmspell.procedural_patterns
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update patterns for current tenant
-- Both USING and WITH CHECK ensure tenant_id cannot be changed
DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.procedural_patterns;
CREATE POLICY tenant_isolation_update ON llmspell.procedural_patterns
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete patterns for current tenant
DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.procedural_patterns;
CREATE POLICY tenant_isolation_delete ON llmspell.procedural_patterns
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Function to update updated_at timestamp automatically
CREATE OR REPLACE FUNCTION llmspell.update_procedural_patterns_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at on pattern updates
DROP TRIGGER IF EXISTS trigger_procedural_patterns_updated_at ON llmspell.procedural_patterns;
CREATE TRIGGER trigger_procedural_patterns_updated_at
    BEFORE UPDATE ON llmspell.procedural_patterns
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_procedural_patterns_updated_at();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname = 'procedural_patterns';
-- SELECT policyname, cmd FROM pg_policies WHERE tablename = 'procedural_patterns';
-- SELECT indexname FROM pg_indexes WHERE tablename = 'procedural_patterns';
