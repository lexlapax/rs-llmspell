-- Migration: Create application role and enforce RLS (Phase 13b.11.0)
-- Purpose: Implement least privilege security model for multi-tenant SaaS
-- Dependencies: All previous migrations (V1-V11)
--
-- Security Architecture:
-- - llmspell: Admin user (migrations, maintenance, SUPERUSER, BYPASSRLS)
-- - llmspell_app: Application runtime user (queries only, RLS enforced)
--
-- Defense-in-Depth Layers:
-- 1. Separate roles (admin vs app) - PRIMARY DEFENSE
--    - Application cannot modify schema, drop tables, or bypass RLS
--    - Compromised app limited to data plane operations
-- 2. FORCE ROW LEVEL SECURITY - SECONDARY DEFENSE
--    - RLS enforced even if user privilege escalation occurs
--    - Defense against privilege bugs or misconfigurations
-- 3. Application tenant context - TERTIARY DEFENSE
-- 4. Audit logging - DETECTION

-- ============================================================================
-- Create Application Role
-- ============================================================================

-- Create application user (idempotent - handles existing role)
DO $$
BEGIN
    CREATE ROLE llmspell_app WITH
        LOGIN
        PASSWORD 'llmspell_app_pass'
        NOSUPERUSER
        NOCREATEDB
        NOCREATEROLE
        NOREPLICATION
        CONNECTION LIMIT -1;
EXCEPTION WHEN duplicate_object THEN
    RAISE NOTICE 'Role llmspell_app already exists, skipping creation.';
END $$;

COMMENT ON ROLE llmspell_app IS 'Application runtime user with least privilege (Phase 13b.11.0)';

-- ============================================================================
-- Grant Schema Access
-- ============================================================================

GRANT USAGE ON SCHEMA llmspell TO llmspell_app;

-- ============================================================================
-- Grant Table Permissions
-- ============================================================================

-- Data plane operations only (SELECT, INSERT, UPDATE, DELETE)
-- No DDL operations (CREATE, ALTER, DROP)
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA llmspell TO llmspell_app;

-- ============================================================================
-- Grant Sequence Permissions
-- ============================================================================

-- Required for SERIAL/BIGSERIAL columns and nextval() operations
GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell_app;

-- ============================================================================
-- Grant Function Execution Permissions
-- ============================================================================

-- Required for partition management, triggers, and utility functions
GRANT EXECUTE ON ALL FUNCTIONS IN SCHEMA llmspell TO llmspell_app;

-- ============================================================================
-- Set Default Privileges for Future Objects
-- ============================================================================

-- Ensure future tables created by llmspell (admin) are accessible to llmspell_app
ALTER DEFAULT PRIVILEGES FOR ROLE llmspell IN SCHEMA llmspell
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO llmspell_app;

-- Ensure future sequences are accessible
ALTER DEFAULT PRIVILEGES FOR ROLE llmspell IN SCHEMA llmspell
    GRANT USAGE, SELECT ON SEQUENCES TO llmspell_app;

-- Ensure future functions are executable
ALTER DEFAULT PRIVILEGES FOR ROLE llmspell IN SCHEMA llmspell
    GRANT EXECUTE ON FUNCTIONS TO llmspell_app;

-- ============================================================================
-- FORCE ROW LEVEL SECURITY (Defense-in-Depth)
-- ============================================================================

-- FORCE RLS enforces policies even for table owners and users with BYPASSRLS
-- This is PostgreSQL best practice for multi-tenant SaaS applications
-- Reference: https://www.postgresql.org/docs/current/ddl-rowsecurity.html

-- Phase 13b.11: Event Log (V11)
ALTER TABLE llmspell.event_log FORCE ROW LEVEL SECURITY;

-- Phase 13b.10: Artifact Storage (V10)
ALTER TABLE llmspell.artifacts FORCE ROW LEVEL SECURITY;
ALTER TABLE llmspell.artifact_content FORCE ROW LEVEL SECURITY;

-- Phase 13b.9: Session Storage (V9)
ALTER TABLE llmspell.sessions FORCE ROW LEVEL SECURITY;

-- Phase 13b.8: Workflow State Storage (V8)
ALTER TABLE llmspell.workflow_states FORCE ROW LEVEL SECURITY;

-- Phase 13b.7: Agent State Storage (V7)
ALTER TABLE llmspell.agent_states FORCE ROW LEVEL SECURITY;

-- Phase 13b.6: Procedural Memory (V6)
ALTER TABLE llmspell.procedural_patterns FORCE ROW LEVEL SECURITY;

-- Phase 13b.5: Knowledge Graph (V5)
ALTER TABLE llmspell.entities FORCE ROW LEVEL SECURITY;
ALTER TABLE llmspell.relationships FORCE ROW LEVEL SECURITY;

-- Phase 13b.4: Vector Storage (V4)
ALTER TABLE llmspell.vector_embeddings_384 FORCE ROW LEVEL SECURITY;
ALTER TABLE llmspell.vector_embeddings_768 FORCE ROW LEVEL SECURITY;
ALTER TABLE llmspell.vector_embeddings_1536 FORCE ROW LEVEL SECURITY;
ALTER TABLE llmspell.vector_embeddings_3072 FORCE ROW LEVEL SECURITY;

-- Phase 13b.2: Key-Value Store (V2)
ALTER TABLE llmspell.kv_store FORCE ROW LEVEL SECURITY;

-- Test data table (if exists)
ALTER TABLE llmspell.test_data FORCE ROW LEVEL SECURITY;

-- ============================================================================
-- Verification Queries (Commented Out)
-- ============================================================================

-- Verify role created:
-- SELECT rolname, rolsuper, rolcreatedb, rolcreaterole, rolbypassrls, rolconnlimit
-- FROM pg_roles WHERE rolname = 'llmspell_app';

-- Verify table permissions:
-- SELECT grantee, table_schema, table_name, privilege_type
-- FROM information_schema.table_privileges
-- WHERE grantee = 'llmspell_app' AND table_schema = 'llmspell'
-- ORDER BY table_name;

-- Verify FORCE RLS applied:
-- SELECT schemaname, tablename, rowsecurity
-- FROM pg_tables
-- WHERE schemaname = 'llmspell' AND rowsecurity = true
-- ORDER BY tablename;

-- Test connection as llmspell_app:
-- \c llmspell llmspell_app
-- SELECT current_user, current_database();
-- SELECT set_config('app.current_tenant_id', 'test_tenant_1', false);
-- SELECT * FROM llmspell.sessions LIMIT 1;  -- Should see only tenant1 data
