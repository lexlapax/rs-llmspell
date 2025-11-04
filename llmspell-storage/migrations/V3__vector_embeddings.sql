-- Phase 13b.4.1: Multi-Dimension Vector Embeddings Tables
-- Creates 4 separate tables for different vector dimensions: 384, 768, 1536, 3072
-- Each table has identical structure except VECTOR column dimension
-- RLS policies applied for multi-tenant isolation (Phase 13b.3 pattern)

-- ============================================================================
-- Table 1: 384-dimensional vectors (All-MiniLM, small models)
-- ============================================================================
CREATE TABLE IF NOT EXISTS llmspell.vector_embeddings_384 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(384) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Indexes for tenant isolation + scope filtering + temporal queries
CREATE INDEX IF NOT EXISTS idx_vector_384_tenant ON llmspell.vector_embeddings_384(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vector_384_scope ON llmspell.vector_embeddings_384(scope);
CREATE INDEX IF NOT EXISTS idx_vector_384_created ON llmspell.vector_embeddings_384(created_at);

-- HNSW index for similarity search (cosine distance)
CREATE INDEX IF NOT EXISTS idx_vector_384_hnsw ON llmspell.vector_embeddings_384
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);  -- Smaller ef for smaller dims

-- Enable RLS and apply policies (Phase 13b.3 pattern: DROP before CREATE)
ALTER TABLE llmspell.vector_embeddings_384 ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.vector_embeddings_384;
CREATE POLICY tenant_isolation_select ON llmspell.vector_embeddings_384
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.vector_embeddings_384;
CREATE POLICY tenant_isolation_insert ON llmspell.vector_embeddings_384
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.vector_embeddings_384;
CREATE POLICY tenant_isolation_update ON llmspell.vector_embeddings_384
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.vector_embeddings_384;
CREATE POLICY tenant_isolation_delete ON llmspell.vector_embeddings_384
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Table 2: 768-dimensional vectors (sentence-transformers, BGE)
-- ============================================================================
CREATE TABLE IF NOT EXISTS llmspell.vector_embeddings_768 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(768) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vector_768_tenant ON llmspell.vector_embeddings_768(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vector_768_scope ON llmspell.vector_embeddings_768(scope);
CREATE INDEX IF NOT EXISTS idx_vector_768_created ON llmspell.vector_embeddings_768(created_at);

CREATE INDEX IF NOT EXISTS idx_vector_768_hnsw ON llmspell.vector_embeddings_768
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 128);  -- Standard HNSW params

ALTER TABLE llmspell.vector_embeddings_768 ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.vector_embeddings_768;
CREATE POLICY tenant_isolation_select ON llmspell.vector_embeddings_768
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.vector_embeddings_768;
CREATE POLICY tenant_isolation_insert ON llmspell.vector_embeddings_768
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.vector_embeddings_768;
CREATE POLICY tenant_isolation_update ON llmspell.vector_embeddings_768
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.vector_embeddings_768;
CREATE POLICY tenant_isolation_delete ON llmspell.vector_embeddings_768
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Table 3: 1536-dimensional vectors (OpenAI text-embedding-3-small)
-- ============================================================================
CREATE TABLE IF NOT EXISTS llmspell.vector_embeddings_1536 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(1536) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vector_1536_tenant ON llmspell.vector_embeddings_1536(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vector_1536_scope ON llmspell.vector_embeddings_1536(scope);
CREATE INDEX IF NOT EXISTS idx_vector_1536_created ON llmspell.vector_embeddings_1536(created_at);

CREATE INDEX IF NOT EXISTS idx_vector_1536_hnsw ON llmspell.vector_embeddings_1536
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 24, ef_construction = 256);  -- Larger params for high-dim

ALTER TABLE llmspell.vector_embeddings_1536 ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.vector_embeddings_1536;
CREATE POLICY tenant_isolation_select ON llmspell.vector_embeddings_1536
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.vector_embeddings_1536;
CREATE POLICY tenant_isolation_insert ON llmspell.vector_embeddings_1536
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.vector_embeddings_1536;
CREATE POLICY tenant_isolation_update ON llmspell.vector_embeddings_1536
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.vector_embeddings_1536;
CREATE POLICY tenant_isolation_delete ON llmspell.vector_embeddings_1536
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Table 4: 3072-dimensional vectors (OpenAI text-embedding-3-large)
-- ============================================================================
CREATE TABLE IF NOT EXISTS llmspell.vector_embeddings_3072 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(3072) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_vector_3072_tenant ON llmspell.vector_embeddings_3072(tenant_id);
CREATE INDEX IF NOT EXISTS idx_vector_3072_scope ON llmspell.vector_embeddings_3072(scope);
CREATE INDEX IF NOT EXISTS idx_vector_3072_created ON llmspell.vector_embeddings_3072(created_at);

-- NOTE: No vector similarity index for 3072 dimensions
-- Both HNSW and IVFFlat in pgvector have a 2000-dimension maximum.
-- For 3072-dimensional vectors (e.g., OpenAI text-embedding-3-large):
--   - Use linear scan for small datasets
--   - Use external vector DB (Qdrant, Milvus) for large-scale similarity search
--   - Consider dimension reduction techniques if needed

ALTER TABLE llmspell.vector_embeddings_3072 ENABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.vector_embeddings_3072;
CREATE POLICY tenant_isolation_select ON llmspell.vector_embeddings_3072
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.vector_embeddings_3072;
CREATE POLICY tenant_isolation_insert ON llmspell.vector_embeddings_3072
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.vector_embeddings_3072;
CREATE POLICY tenant_isolation_update ON llmspell.vector_embeddings_3072
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.vector_embeddings_3072;
CREATE POLICY tenant_isolation_delete ON llmspell.vector_embeddings_3072
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Grant permissions to application role (conditional, for V3->V11 migrations)
-- ============================================================================
-- Phase 13b.3 learning: Schema recreation removes grants, must re-grant
-- Phase 13b.11.0: Made conditional - V12 creates llmspell_app role and sets default privileges
-- These grants apply when running V3 before V12 exists (historical migrations)

DO $$
BEGIN
    -- Only grant if llmspell_app role exists (created by V12)
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'llmspell_app') THEN
        GRANT USAGE ON SCHEMA llmspell TO llmspell_app;

        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE
            llmspell.vector_embeddings_384,
            llmspell.vector_embeddings_768,
            llmspell.vector_embeddings_1536,
            llmspell.vector_embeddings_3072
        TO llmspell_app;

        GRANT USAGE ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell_app;
    END IF;
END $$;

-- Verification queries (for manual testing):
-- SELECT tablename FROM pg_tables WHERE schemaname = 'llmspell' AND tablename LIKE 'vector_embeddings_%';
-- SELECT tablename, indexname FROM pg_indexes WHERE schemaname = 'llmspell' AND indexname LIKE '%hnsw%';
-- SELECT tablename, policyname FROM pg_policies WHERE schemaname = 'llmspell' AND tablename LIKE 'vector_embeddings_%';
