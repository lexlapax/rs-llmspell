-- Migration: Create artifacts storage tables (Phase 13b.10.1)
-- Purpose: Content-addressed artifact storage with deduplication and dual storage strategy
-- Dependencies: V1__initial_setup.sql (llmspell schema), V9__sessions.sql (sessions table)
--
-- Artifact storage implements:
-- - Content-addressed storage with blake3 hashing
-- - Deduplication via reference counting
-- - Dual storage: BYTEA (<1MB) vs Large Objects (>=1MB)
-- - Artifact metadata with versioning
-- - Session integration via foreign key

-- ============================================================================
-- Content Storage Table (Content-Addressed with Deduplication)
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.artifact_content (
    -- Tenant isolation (part of composite PK)
    tenant_id VARCHAR(255) NOT NULL,

    -- Content-addressed primary key (blake3 hash)
    content_hash VARCHAR(64) NOT NULL,

    -- Storage strategy
    storage_type VARCHAR(20) NOT NULL,

    -- BYTEA storage for small artifacts (<1MB)
    data BYTEA,

    -- Large Object storage for large artifacts (>=1MB)
    large_object_oid OID,

    -- Content metadata
    size_bytes BIGINT NOT NULL,
    is_compressed BOOLEAN NOT NULL DEFAULT false,
    original_size_bytes BIGINT,

    -- Deduplication tracking
    reference_count INTEGER NOT NULL DEFAULT 1,

    -- Lifecycle timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite primary key: One content per (tenant, hash)
    PRIMARY KEY (tenant_id, content_hash),

    -- Storage type constraint
    CONSTRAINT valid_storage_type CHECK (
        storage_type IN ('bytea', 'large_object')
    ),

    -- Reference count constraint
    CONSTRAINT positive_reference_count CHECK (reference_count > 0),

    -- Storage consistency constraints
    CONSTRAINT bytea_storage_valid CHECK (
        (storage_type = 'bytea' AND data IS NOT NULL AND large_object_oid IS NULL)
        OR (storage_type = 'large_object' AND large_object_oid IS NOT NULL AND data IS NULL)
    ),

    -- Size constraint (max 100MB)
    CONSTRAINT max_artifact_size CHECK (size_bytes <= 104857600)
);

-- Index for content access patterns
CREATE INDEX IF NOT EXISTS idx_artifact_content_tenant
    ON llmspell.artifact_content(tenant_id);

-- Index for reference count queries (cleanup zero-ref content)
CREATE INDEX IF NOT EXISTS idx_artifact_content_refcount
    ON llmspell.artifact_content(reference_count)
    WHERE reference_count = 0;

-- Index for large object cleanup
CREATE INDEX IF NOT EXISTS idx_artifact_content_large_objects
    ON llmspell.artifact_content(large_object_oid)
    WHERE large_object_oid IS NOT NULL;

-- Index for access tracking
CREATE INDEX IF NOT EXISTS idx_artifact_content_accessed
    ON llmspell.artifact_content(last_accessed_at DESC);

-- ============================================================================
-- Artifacts Table (Metadata and References)
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.artifacts (
    -- Tenant isolation (part of composite PK)
    tenant_id VARCHAR(255) NOT NULL,

    -- Artifact identifier (format: "{session_id}:{sequence}:{content_hash}")
    artifact_id VARCHAR(512) NOT NULL,

    -- Session reference (extracted from artifact_id)
    session_id UUID NOT NULL,

    -- Sequence number within session
    sequence BIGINT NOT NULL,

    -- Content reference
    content_hash VARCHAR(64) NOT NULL,

    -- Artifact metadata (full ArtifactMetadata as JSONB)
    metadata JSONB NOT NULL,

    -- Extracted fields for efficient queries
    name VARCHAR(255) NOT NULL,
    artifact_type VARCHAR(50) NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by VARCHAR(255),

    -- Version tracking
    version INTEGER NOT NULL DEFAULT 1,
    parent_artifact_id VARCHAR(512),

    -- Tags for searching (extracted from metadata)
    tags TEXT[],

    -- Lifecycle timestamps
    stored_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite primary key
    PRIMARY KEY (tenant_id, artifact_id),

    -- Foreign key to content table
    FOREIGN KEY (tenant_id, content_hash)
        REFERENCES llmspell.artifact_content(tenant_id, content_hash)
        ON DELETE RESTRICT,  -- Cannot delete content while artifacts reference it

    -- Foreign key to sessions table
    FOREIGN KEY (tenant_id, session_id)
        REFERENCES llmspell.sessions(tenant_id, session_id)
        ON DELETE CASCADE,  -- Delete artifacts when session is deleted

    -- Unique constraint on (tenant_id, session_id, sequence)
    CONSTRAINT unique_session_sequence UNIQUE (tenant_id, session_id, sequence),

    -- Version constraint
    CONSTRAINT positive_version CHECK (version > 0),

    -- Sequence constraint
    CONSTRAINT non_negative_sequence CHECK (sequence >= 0)
);

-- Index for session-based queries (list all artifacts in session)
CREATE INDEX IF NOT EXISTS idx_artifacts_session
    ON llmspell.artifacts(tenant_id, session_id, created_at DESC);

-- Index for artifact type queries
CREATE INDEX IF NOT EXISTS idx_artifacts_type
    ON llmspell.artifacts(artifact_type, created_at DESC);

-- Index for content hash lookup (find all artifacts with same content)
CREATE INDEX IF NOT EXISTS idx_artifacts_content
    ON llmspell.artifacts(tenant_id, content_hash);

-- Index for name searches
CREATE INDEX IF NOT EXISTS idx_artifacts_name
    ON llmspell.artifacts(name);

-- Index for created_at range queries
CREATE INDEX IF NOT EXISTS idx_artifacts_created
    ON llmspell.artifacts(created_at DESC);

-- Index for size-based queries
CREATE INDEX IF NOT EXISTS idx_artifacts_size
    ON llmspell.artifacts(size_bytes DESC);

-- GIN index for tag searches
CREATE INDEX IF NOT EXISTS idx_artifacts_tags
    ON llmspell.artifacts USING GIN(tags);

-- GIN index for JSONB metadata queries
CREATE INDEX IF NOT EXISTS idx_artifacts_metadata
    ON llmspell.artifacts USING GIN(metadata);

-- Composite index for tenant + type queries
CREATE INDEX IF NOT EXISTS idx_artifacts_tenant_type
    ON llmspell.artifacts(tenant_id, artifact_type, created_at DESC);

-- ============================================================================
-- Row-Level Security (RLS)
-- ============================================================================

-- Enable RLS on both tables
ALTER TABLE llmspell.artifact_content ENABLE ROW LEVEL SECURITY;
ALTER TABLE llmspell.artifacts ENABLE ROW LEVEL SECURITY;

-- Content table policies (4 policies: SELECT, INSERT, UPDATE, DELETE)
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.artifact_content;
CREATE POLICY tenant_isolation_select ON llmspell.artifact_content
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.artifact_content;
CREATE POLICY tenant_isolation_insert ON llmspell.artifact_content
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.artifact_content;
CREATE POLICY tenant_isolation_update ON llmspell.artifact_content
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.artifact_content;
CREATE POLICY tenant_isolation_delete ON llmspell.artifact_content
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- Artifacts table policies (4 policies: SELECT, INSERT, UPDATE, DELETE)
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.artifacts;
CREATE POLICY tenant_isolation_select ON llmspell.artifacts
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.artifacts;
CREATE POLICY tenant_isolation_insert ON llmspell.artifacts
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.artifacts;
CREATE POLICY tenant_isolation_update ON llmspell.artifacts
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.artifacts;
CREATE POLICY tenant_isolation_delete ON llmspell.artifacts
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Triggers and Functions
-- ============================================================================

-- Function to auto-update updated_at on artifacts
CREATE OR REPLACE FUNCTION llmspell.update_artifacts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update updated_at
DROP TRIGGER IF EXISTS trigger_artifacts_updated_at ON llmspell.artifacts;
CREATE TRIGGER trigger_artifacts_updated_at
    BEFORE UPDATE ON llmspell.artifacts
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_artifacts_updated_at();

-- Function to increment reference count on content
CREATE OR REPLACE FUNCTION llmspell.increment_content_refcount()
RETURNS TRIGGER AS $$
BEGIN
    -- Increment reference count when new artifact references content
    UPDATE llmspell.artifact_content
    SET reference_count = reference_count + 1,
        last_accessed_at = now()
    WHERE tenant_id = NEW.tenant_id
      AND content_hash = NEW.content_hash;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to increment refcount when artifact is inserted
DROP TRIGGER IF EXISTS trigger_increment_refcount ON llmspell.artifacts;
CREATE TRIGGER trigger_increment_refcount
    AFTER INSERT ON llmspell.artifacts
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.increment_content_refcount();

-- Function to decrement reference count on content
CREATE OR REPLACE FUNCTION llmspell.decrement_content_refcount()
RETURNS TRIGGER AS $$
BEGIN
    -- Decrement reference count when artifact is deleted
    UPDATE llmspell.artifact_content
    SET reference_count = reference_count - 1
    WHERE tenant_id = OLD.tenant_id
      AND content_hash = OLD.content_hash;

    RETURN OLD;
END;
$$ LANGUAGE plpgsql;

-- Trigger to decrement refcount when artifact is deleted
DROP TRIGGER IF EXISTS trigger_decrement_refcount ON llmspell.artifacts;
CREATE TRIGGER trigger_decrement_refcount
    AFTER DELETE ON llmspell.artifacts
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.decrement_content_refcount();

-- Function to update last_accessed_at on content
CREATE OR REPLACE FUNCTION llmspell.update_content_accessed_at()
RETURNS TRIGGER AS $$
BEGIN
    -- Update last_accessed_at when content is accessed (throttled to 1 minute)
    IF OLD.last_accessed_at IS NULL OR
       NEW.last_accessed_at < (now() - INTERVAL '1 minute') THEN
        NEW.last_accessed_at = now();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-update last_accessed_at on content access
DROP TRIGGER IF EXISTS trigger_content_accessed_at ON llmspell.artifact_content;
CREATE TRIGGER trigger_content_accessed_at
    BEFORE UPDATE ON llmspell.artifact_content
    FOR EACH ROW
    EXECUTE FUNCTION llmspell.update_content_accessed_at();

-- Verification queries (manual check):
-- SELECT relname, relrowsecurity FROM pg_class WHERE relname IN ('artifact_content', 'artifacts');
-- SELECT policyname, cmd FROM pg_policies WHERE tablename IN ('artifact_content', 'artifacts');
-- SELECT indexname FROM pg_indexes WHERE tablename IN ('artifact_content', 'artifacts');
