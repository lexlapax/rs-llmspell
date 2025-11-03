-- PostgreSQL initialization script for llmspell Phase 13b.2
-- Enables extensions and creates schema for experimental storage backends

-- Enable VectorChord extension (CRITICAL: requires CASCADE for pgvector 0.8.1 dependency)
-- Validated in Task 13b.2.0.1: VectorChord 0.5.3 depends on pgvector 0.8.1
CREATE EXTENSION IF NOT EXISTS vchord CASCADE;

-- Enable cryptographic functions (for hashing, encryption)
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- Enable UUID generation functions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create llmspell schema for all application tables
CREATE SCHEMA IF NOT EXISTS llmspell;

-- Set search path to prioritize llmspell schema
ALTER DATABASE llmspell_dev SET search_path TO llmspell, public;

-- Grant all privileges to llmspell user
GRANT ALL PRIVILEGES ON SCHEMA llmspell TO llmspell;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA llmspell TO llmspell;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell;
GRANT ALL PRIVILEGES ON ALL FUNCTIONS IN SCHEMA llmspell TO llmspell;

-- Grant future object privileges (for objects created after this script)
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT ALL ON TABLES TO llmspell;
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT ALL ON SEQUENCES TO llmspell;
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT ALL ON FUNCTIONS TO llmspell;

-- Grant CREATE privilege on public schema for Refinery migration tracking table
GRANT CREATE ON SCHEMA public TO llmspell;
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO llmspell;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO llmspell;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON TABLES TO llmspell;
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT ALL ON SEQUENCES TO llmspell;

-- Create application user (Phase 13b.3: non-superuser for RLS testing/production)
-- CRITICAL: Must NOT be superuser (superusers bypass RLS)
CREATE ROLE llmspell_app WITH LOGIN PASSWORD 'llmspell_dev_pass';

-- Grant application permissions (all CRUD operations, no DDL)
GRANT CONNECT ON DATABASE llmspell_dev TO llmspell_app;
GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA llmspell TO llmspell_app;
GRANT USAGE ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell_app;

-- Future object privileges for llmspell_app (tables created by migrations)
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO llmspell_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell GRANT USAGE ON SEQUENCES TO llmspell_app;
