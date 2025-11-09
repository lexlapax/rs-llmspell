-- Migration V4: Bi-Temporal Graph Storage (Phase 13b.5.1)
--
-- Creates entities and relationships tables with bi-temporal semantics for semantic memory.
-- Bi-temporal design tracks both:
--   - Valid time: When data was true in the real world
--   - Transaction time: When data was recorded in the database
--
-- This enables:
--   - Point-in-time queries ("What was known at time T?")
--   - Historical reconstruction ("What did we know about X on date Y?")
--   - Audit trails (immutable transaction history)
--   - Temporal joins (relationships valid at specific times)
--
-- Dependencies:
--   - V1: Initial setup (schema, uuid extension)
--   - V2: RLS test infrastructure
--   - V3: Vector embeddings (independent, can run in any order)
--
-- RLS Strategy: Row-level security enforced on both tables for multi-tenant isolation

-- ============================================================================
-- Table 1: Entities (nodes in knowledge graph)
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.entities (
    entity_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(255) NOT NULL,  -- e.g., "person", "concept", "event"
    name VARCHAR(500) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',  -- Flexible schema for entity attributes

    -- Bi-temporal timestamps
    -- Valid time: When this entity version was true in the real world
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    -- Transaction time: When this entity version was recorded in the database
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Time-range constraints (start < end for both dimensions)
    CONSTRAINT valid_time_range CHECK (valid_time_start < valid_time_end),
    CONSTRAINT tx_time_range CHECK (transaction_time_start < transaction_time_end)
);

-- Standard indexes for tenant isolation and entity lookup
CREATE INDEX IF NOT EXISTS idx_entities_tenant ON llmspell.entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_entities_type ON llmspell.entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON llmspell.entities(name);

-- GiST indexes for temporal range queries (critical for bi-temporal performance)
-- These enable O(log n) queries like "find entities valid at time T"
CREATE INDEX IF NOT EXISTS idx_entities_valid_time ON llmspell.entities
    USING GIST (tstzrange(valid_time_start, valid_time_end));

CREATE INDEX IF NOT EXISTS idx_entities_tx_time ON llmspell.entities
    USING GIST (tstzrange(transaction_time_start, transaction_time_end));

-- GIN index for JSONB property queries
CREATE INDEX IF NOT EXISTS idx_entities_properties ON llmspell.entities
    USING GIN (properties);

-- Enable Row-Level Security
ALTER TABLE llmspell.entities ENABLE ROW LEVEL SECURITY;

-- RLS Policies (Phase 13b.3 pattern: DROP before CREATE for idempotency)
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.entities;
CREATE POLICY tenant_isolation_select ON llmspell.entities
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.entities;
CREATE POLICY tenant_isolation_insert ON llmspell.entities
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.entities;
CREATE POLICY tenant_isolation_update ON llmspell.entities
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.entities;
CREATE POLICY tenant_isolation_delete ON llmspell.entities
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Table 2: Relationships (edges in knowledge graph)
-- ============================================================================

CREATE TABLE IF NOT EXISTS llmspell.relationships (
    relationship_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    from_entity UUID NOT NULL,  -- Source entity
    to_entity UUID NOT NULL,    -- Target entity
    relationship_type VARCHAR(255) NOT NULL,  -- e.g., "knows", "part_of", "created_by"
    properties JSONB NOT NULL DEFAULT '{}',   -- Relationship attributes

    -- Bi-temporal timestamps (same semantics as entities)
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Foreign keys to entities (ON DELETE CASCADE for cleanup)
    CONSTRAINT fk_from_entity FOREIGN KEY (from_entity)
        REFERENCES llmspell.entities(entity_id) ON DELETE CASCADE,
    CONSTRAINT fk_to_entity FOREIGN KEY (to_entity)
        REFERENCES llmspell.entities(entity_id) ON DELETE CASCADE,

    -- Time-range constraints
    CONSTRAINT valid_time_range CHECK (valid_time_start < valid_time_end),
    CONSTRAINT tx_time_range CHECK (transaction_time_start < transaction_time_end)
);

-- Indexes for relationship queries and graph traversal
CREATE INDEX IF NOT EXISTS idx_relationships_tenant ON llmspell.relationships(tenant_id);
CREATE INDEX IF NOT EXISTS idx_relationships_from ON llmspell.relationships(from_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_to ON llmspell.relationships(to_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_type ON llmspell.relationships(relationship_type);

-- Composite index for efficient graph traversal queries
CREATE INDEX IF NOT EXISTS idx_relationships_from_type ON llmspell.relationships(from_entity, relationship_type);

-- GiST indexes for temporal range queries
CREATE INDEX IF NOT EXISTS idx_relationships_valid_time ON llmspell.relationships
    USING GIST (tstzrange(valid_time_start, valid_time_end));

CREATE INDEX IF NOT EXISTS idx_relationships_tx_time ON llmspell.relationships
    USING GIST (tstzrange(transaction_time_start, transaction_time_end));

-- GIN index for JSONB property queries
CREATE INDEX IF NOT EXISTS idx_relationships_properties ON llmspell.relationships
    USING GIN (properties);

-- Enable Row-Level Security
ALTER TABLE llmspell.relationships ENABLE ROW LEVEL SECURITY;

-- RLS Policies (same pattern as entities)
DROP POLICY IF EXISTS tenant_isolation_select ON llmspell.relationships;
CREATE POLICY tenant_isolation_select ON llmspell.relationships
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_insert ON llmspell.relationships;
CREATE POLICY tenant_isolation_insert ON llmspell.relationships
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_update ON llmspell.relationships;
CREATE POLICY tenant_isolation_update ON llmspell.relationships
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

DROP POLICY IF EXISTS tenant_isolation_delete ON llmspell.relationships;
CREATE POLICY tenant_isolation_delete ON llmspell.relationships
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- ============================================================================
-- Privilege Grants (conditional, for V4->V11 migrations)
-- ============================================================================
-- Phase 13b.11.0: Made conditional - V12 creates llmspell_app role and sets default privileges
-- These grants apply when running V4 before V12 exists (historical migrations)

DO $$
BEGIN
    -- Only grant if llmspell_app role exists (created by V12)
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'llmspell_app') THEN
        -- Grant schema usage (required for non-superuser access)
        GRANT USAGE ON SCHEMA llmspell TO llmspell_app;

        -- Grant table operations to application user
        GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE
            llmspell.entities,
            llmspell.relationships
        TO llmspell_app;

        -- Grant sequence usage for default values
        GRANT USAGE ON ALL SEQUENCES IN SCHEMA llmspell TO llmspell_app;
    END IF;
END $$;

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Bi-Temporal Query Patterns:
--
-- 1. Current state (both dimensions at 'now'):
--    SELECT * FROM entities
--    WHERE valid_time_start <= now() AND valid_time_end > now()
--      AND transaction_time_start <= now() AND transaction_time_end > now()
--
-- 2. Historical state ("What was true at time T?"):
--    SELECT * FROM entities
--    WHERE valid_time_start <= $T AND valid_time_end > $T
--      AND transaction_time_start <= now() AND transaction_time_end > now()
--
-- 3. Database snapshot ("What did we know at time T?"):
--    SELECT * FROM entities
--    WHERE transaction_time_start <= $T AND transaction_time_end > $T
--
-- 4. Full bi-temporal point query:
--    SELECT * FROM entities
--    WHERE valid_time_start <= $VT AND valid_time_end > $VT
--      AND transaction_time_start <= $TT AND transaction_time_end > $TT
--
-- Graph Traversal:
--   - Use recursive CTEs with relationships table
--   - Filter by valid_time for temporal consistency
--   - Use path arrays for cycle prevention
--   - See Task 13b.5.3 for implementation details
--
-- Performance Considerations:
--   - GiST indexes critical for time-range queries (O(log n) vs O(n))
--   - Valid time queries more common than transaction time
--   - Composite index (from_entity, relationship_type) for graph traversal
--   - JSONB properties indexed with GIN for flexible queries
