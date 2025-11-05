-- Phase 13b.13.1: API Key Storage with Encryption
-- Creates api_keys table for storing encrypted API keys using pgcrypto
-- RLS policies applied for multi-tenant isolation
-- Encryption using pgp_sym_encrypt with passphrase from app.encryption_key session variable

-- ============================================================================
-- Enable pgcrypto Extension for Encryption
-- ============================================================================

-- pgcrypto: Cryptographic functions (pgp_sym_encrypt, pgp_sym_decrypt)
-- REQUIRED for encrypted API key storage
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ============================================================================
-- API Keys Table
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.api_keys (
    -- Primary identifiers
    key_id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,

    -- Service/Provider identification
    service VARCHAR(255) NOT NULL,  -- Service this key is for (e.g., "openai", "anthropic", "google_search")

    -- Encrypted key storage
    encrypted_key BYTEA NOT NULL,  -- pgp_sym_encrypt result

    -- Key metadata (JSON for extensibility)
    key_metadata JSONB NOT NULL DEFAULT '{}'::JSONB,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT true,
    usage_count BIGINT NOT NULL DEFAULT 0,

    -- Audit fields
    rotated_from VARCHAR(255),      -- Previous key_id if this was created via rotation
    deactivated_at TIMESTAMPTZ,

    -- Constraints
    UNIQUE (tenant_id, service, is_active)  -- Only one active key per tenant/service
);

-- ============================================================================
-- Indexes for Query Performance
-- ============================================================================

-- Index for tenant + service lookup (most common query pattern)
CREATE INDEX IF NOT EXISTS idx_api_keys_tenant_service
    ON llmspell.api_keys(tenant_id, service);

-- Index for expiration cleanup queries
CREATE INDEX IF NOT EXISTS idx_api_keys_expiration
    ON llmspell.api_keys(expires_at) WHERE expires_at IS NOT NULL;

-- Index for active keys filtering
CREATE INDEX IF NOT EXISTS idx_api_keys_active
    ON llmspell.api_keys(is_active) WHERE is_active = true;

-- GIN index for metadata queries
CREATE INDEX IF NOT EXISTS idx_api_keys_metadata
    ON llmspell.api_keys USING GIN(key_metadata);

-- ============================================================================
-- Row-Level Security (RLS) Policies
-- ============================================================================

-- Enable RLS on api_keys table
ALTER TABLE llmspell.api_keys ENABLE ROW LEVEL SECURITY;

-- Force RLS even for table owner (Phase 13b.11.0 pattern)
ALTER TABLE llmspell.api_keys FORCE ROW LEVEL SECURITY;

-- Drop existing policies if they exist (idempotent)
DROP POLICY IF EXISTS api_keys_tenant_select ON llmspell.api_keys;
DROP POLICY IF EXISTS api_keys_tenant_insert ON llmspell.api_keys;
DROP POLICY IF EXISTS api_keys_tenant_update ON llmspell.api_keys;
DROP POLICY IF EXISTS api_keys_tenant_delete ON llmspell.api_keys;

-- Policy: SELECT - Users can only see their tenant's API keys
CREATE POLICY api_keys_tenant_select ON llmspell.api_keys
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: INSERT - Users can only insert API keys for their tenant
CREATE POLICY api_keys_tenant_insert ON llmspell.api_keys
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: UPDATE - Users can only update their tenant's API keys
CREATE POLICY api_keys_tenant_update ON llmspell.api_keys
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- Policy: DELETE - Users can only delete their tenant's API keys
CREATE POLICY api_keys_tenant_delete ON llmspell.api_keys
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Grant Permissions to Application Role
-- ============================================================================

-- Grant table permissions to llmspell_app role (created in V12)
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'llmspell_app') THEN
        GRANT SELECT, INSERT, UPDATE, DELETE ON llmspell.api_keys TO llmspell_app;

        -- Grant execute on pgcrypto functions (needed for encryption/decryption)
        GRANT EXECUTE ON FUNCTION pgp_sym_encrypt(text, text) TO llmspell_app;
        GRANT EXECUTE ON FUNCTION pgp_sym_encrypt(text, text, text) TO llmspell_app;
        GRANT EXECUTE ON FUNCTION pgp_sym_decrypt(bytea, text) TO llmspell_app;
        GRANT EXECUTE ON FUNCTION pgp_sym_decrypt(bytea, text, text) TO llmspell_app;
    END IF;
END $$;

-- ============================================================================
-- Helper Functions for Key Management
-- ============================================================================

-- Function to cleanup expired API keys
CREATE OR REPLACE FUNCTION llmspell.cleanup_expired_api_keys()
RETURNS TABLE (
    deleted_key_id VARCHAR(255),
    service VARCHAR(255),
    expired_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    DELETE FROM llmspell.api_keys
    WHERE expires_at IS NOT NULL
      AND expires_at < now()
      AND tenant_id = current_setting('app.current_tenant_id', true)
    RETURNING key_id, service, expires_at;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION llmspell.cleanup_expired_api_keys IS
    'Delete expired API keys for current tenant, returns deleted key information';

-- Function to get API key statistics for current tenant
CREATE OR REPLACE FUNCTION llmspell.get_api_key_stats()
RETURNS TABLE (
    total_keys BIGINT,
    active_keys BIGINT,
    expired_keys BIGINT,
    keys_by_service JSONB,
    total_usage BIGINT,
    avg_usage_per_key NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    WITH service_counts AS (
        SELECT
            service,
            COUNT(*) as count
        FROM llmspell.api_keys
        WHERE tenant_id = current_setting('app.current_tenant_id', true)
        GROUP BY service
    )
    SELECT
        COUNT(*)::BIGINT as total_keys,
        COUNT(*) FILTER (WHERE is_active = true)::BIGINT as active_keys,
        COUNT(*) FILTER (WHERE expires_at IS NOT NULL AND expires_at < now())::BIGINT as expired_keys,
        (SELECT jsonb_object_agg(service, count) FROM service_counts) as keys_by_service,
        SUM(usage_count)::BIGINT as total_usage,
        AVG(usage_count)::NUMERIC as avg_usage_per_key
    FROM llmspell.api_keys
    WHERE tenant_id = current_setting('app.current_tenant_id', true);
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION llmspell.get_api_key_stats IS
    'Get comprehensive statistics about API keys for current tenant';

-- Function to rotate API key (deactivate old, prepare for new)
CREATE OR REPLACE FUNCTION llmspell.rotate_api_key(
    old_key_id VARCHAR(255)
)
RETURNS VARCHAR(255) AS $$
DECLARE
    new_key_id VARCHAR(255);
    old_service VARCHAR(255);
BEGIN
    -- Get service from old key and verify ownership
    SELECT service INTO old_service
    FROM llmspell.api_keys
    WHERE key_id = old_key_id
      AND tenant_id = current_setting('app.current_tenant_id', true);

    IF old_service IS NULL THEN
        RAISE EXCEPTION 'API key % not found or not owned by current tenant', old_key_id;
    END IF;

    -- Generate new key_id
    new_key_id := old_key_id || '_rotated_' || extract(epoch from now())::bigint::text;

    -- Deactivate old key
    UPDATE llmspell.api_keys
    SET is_active = false,
        deactivated_at = now()
    WHERE key_id = old_key_id
      AND tenant_id = current_setting('app.current_tenant_id', true);

    RETURN new_key_id;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION llmspell.rotate_api_key IS
    'Deactivate existing API key and generate new key_id for rotation (application stores new encrypted key)';

-- API keys table ready for Phase 13b.13.2 backend implementation
