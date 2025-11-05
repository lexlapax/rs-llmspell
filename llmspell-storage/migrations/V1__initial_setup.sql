-- Phase 13b.2: Initial Migration Framework Setup
-- This migration verifies the Refinery migration framework is working correctly
-- Actual table schemas will be added in Phase 13b.4+ (VectorChord Integration)

-- The refinery_schema_history table is created automatically by Refinery
-- to track migration versions. No additional setup needed for Phase 13b.2.

-- ============================================================================
-- Create Required PostgreSQL Extensions
-- ============================================================================

-- uuid-ossp: UUID generation functions (used in test tables and application code)
-- REQUIRED for all installations
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- vchord: VectorChord extension for vector similarity search (Phase 13b.4)
-- CASCADE automatically installs pgvector 0.8.1 as dependency
-- REQUIRED when using VectorChord Docker image (ghcr.io/tensorchord/vchord-postgres:pg18-v0.5.3)
-- This matches the init scripts from Phase 13b.2.2-13b.2.3
CREATE EXTENSION IF NOT EXISTS vchord CASCADE;

-- ============================================================================
-- Create llmspell Schema
-- ============================================================================

-- Create schema if it doesn't exist (idempotent)
CREATE SCHEMA IF NOT EXISTS llmspell;

-- Grant usage to application role (will be created in V12__application_role_rls_enforcement.sql)
-- This is forward-compatible and won't error if role doesn't exist yet
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'llmspell_app') THEN
        GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
    END IF;
END $$;

-- Migration framework successfully initialized
