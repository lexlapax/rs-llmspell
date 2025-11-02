-- Phase 13b.2: Initial Migration Framework Setup
-- This migration verifies the Refinery migration framework is working correctly
-- Actual table schemas will be added in Phase 13b.4+ (VectorChord Integration)

-- The refinery_schema_history table is created automatically by Refinery
-- to track migration versions. No additional setup needed for Phase 13b.2.

-- Verify llmspell schema exists (created by init scripts)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_namespace WHERE nspname = 'llmspell') THEN
        RAISE EXCEPTION 'llmspell schema does not exist - init scripts may not have run';
    END IF;
END $$;

-- Migration framework successfully initialized
