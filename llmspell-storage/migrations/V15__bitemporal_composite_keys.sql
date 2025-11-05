-- Migration V15: Fix Bi-Temporal Composite Keys (Phase 13b - Graph Storage)
--
-- Problem:
--   V4 created entities and relationships tables with single-column primary keys
--   (entity_id and relationship_id respectively). This prevents storing multiple
--   temporal versions of the same logical entity/relationship.
--
-- Solution:
--   Change primary keys to composite: (id, transaction_time_start)
--   This allows multiple temporal versions of the same logical entity/relationship
--   while uniquely identifying each physical version.
--
-- Trade-offs:
--   - Must drop foreign key constraints from relationships table
--   - Referential integrity becomes application-enforced (standard for bi-temporal)
--   - Enables full bi-temporal versioning as designed
--
-- Dependencies:
--   - V4: Bi-temporal graph storage (initial schema)

-- ============================================================================
-- Step 1: Drop foreign key constraints on relationships table
-- ============================================================================
-- These reference entities(entity_id) which will no longer be unique

ALTER TABLE llmspell.relationships
    DROP CONSTRAINT IF EXISTS fk_from_entity;

ALTER TABLE llmspell.relationships
    DROP CONSTRAINT IF EXISTS fk_to_entity;

-- ============================================================================
-- Step 2: Fix entities table primary key
-- ============================================================================

-- Drop old primary key constraint
ALTER TABLE llmspell.entities
    DROP CONSTRAINT IF EXISTS entities_pkey;

-- Add composite primary key (entity_id, transaction_time_start)
-- This uniquely identifies each temporal version of an entity
ALTER TABLE llmspell.entities
    ADD PRIMARY KEY (entity_id, transaction_time_start);

-- Add index on entity_id alone for efficient lookup of all versions
CREATE INDEX IF NOT EXISTS idx_entities_id_lookup
    ON llmspell.entities(entity_id);

-- ============================================================================
-- Step 3: Fix relationships table primary key
-- ============================================================================

-- Drop old primary key constraint
ALTER TABLE llmspell.relationships
    DROP CONSTRAINT IF EXISTS relationships_pkey;

-- Add composite primary key (relationship_id, transaction_time_start)
ALTER TABLE llmspell.relationships
    ADD PRIMARY KEY (relationship_id, transaction_time_start);

-- Add index on relationship_id alone for efficient lookup of all versions
CREATE INDEX IF NOT EXISTS idx_relationships_id_lookup
    ON llmspell.relationships(relationship_id);

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Referential Integrity:
--   Foreign key constraints were dropped because entity_id and relationship_id
--   are no longer unique (multiple temporal versions exist). This is standard
--   practice for bi-temporal databases.
--
--   Application code must enforce:
--   1. Relationships only reference valid entity_ids
--   2. Cleanup is handled through soft-delete (setting transaction_time_end)
--   3. CASCADE behavior implemented in delete_before() method
--
-- Query Pattern Updates:
--   No changes needed to existing queries - they already filter by:
--   - transaction_time_end = 'infinity' (current version)
--   - valid_time_end = 'infinity' (currently valid)
--
-- Performance:
--   - Composite primary key provides efficient lookups
--   - Additional indexes on id columns support version history queries
--   - GiST temporal indexes (from V4) remain optimal for time-range queries
--
-- Backward Compatibility:
--   Existing data is preserved. The schema change enables new functionality
--   (entity updates, relationship updates) that previously failed with
--   duplicate key violations.
