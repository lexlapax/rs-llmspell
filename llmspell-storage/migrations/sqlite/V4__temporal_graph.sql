-- Migration V4: Bi-Temporal Graph Storage (Phase 13c.2.4)
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
-- SQLite Adaptations from PostgreSQL V4:
--   - UUID -> TEXT (36-char UUID strings)
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps)
--   - JSONB -> TEXT (JSON as text)
--   - 'infinity' -> 9999999999 (far future Unix timestamp: ~2286-11-20)
--   - GiST indexes -> B-tree INTEGER indexes (SQLite doesn't have GiST)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)
--   - V3: Vector embeddings (independent, can run in any order)

-- ============================================================================
-- Table 1: Entities (nodes in knowledge graph)
-- ============================================================================

CREATE TABLE IF NOT EXISTS entities (
    entity_id TEXT PRIMARY KEY,  -- UUID as TEXT
    tenant_id TEXT NOT NULL,
    entity_type TEXT NOT NULL,  -- e.g., "person", "concept", "event"
    name TEXT NOT NULL,
    properties TEXT NOT NULL DEFAULT '{}',  -- JSON as TEXT

    -- Bi-temporal timestamps (Unix epoch seconds)
    -- Valid time: When this entity version was true in the real world
    valid_time_start INTEGER NOT NULL,
    valid_time_end INTEGER NOT NULL DEFAULT 9999999999,

    -- Transaction time: When this entity version was recorded in the database
    transaction_time_start INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    transaction_time_end INTEGER NOT NULL DEFAULT 9999999999,

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Time-range constraints (start < end for both dimensions)
    CHECK (valid_time_start < valid_time_end),
    CHECK (transaction_time_start < transaction_time_end)
);

-- Standard indexes for tenant isolation and entity lookup
CREATE INDEX IF NOT EXISTS idx_entities_tenant ON entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);

-- B-tree indexes for temporal range queries
-- SQLite doesn't have GiST, but B-tree indexes still accelerate range queries
CREATE INDEX IF NOT EXISTS idx_entities_valid_time_start ON entities(valid_time_start);
CREATE INDEX IF NOT EXISTS idx_entities_valid_time_end ON entities(valid_time_end);
CREATE INDEX IF NOT EXISTS idx_entities_tx_time_start ON entities(transaction_time_start);
CREATE INDEX IF NOT EXISTS idx_entities_tx_time_end ON entities(transaction_time_end);

-- ============================================================================
-- Table 2: Relationships (edges in knowledge graph)
-- ============================================================================

CREATE TABLE IF NOT EXISTS relationships (
    relationship_id TEXT PRIMARY KEY,  -- UUID as TEXT
    tenant_id TEXT NOT NULL,
    from_entity TEXT NOT NULL,  -- Source entity
    to_entity TEXT NOT NULL,    -- Target entity
    relationship_type TEXT NOT NULL,  -- e.g., "knows", "part_of", "created_by"
    properties TEXT NOT NULL DEFAULT '{}',   -- JSON as TEXT

    -- Bi-temporal timestamps (Unix epoch seconds)
    valid_time_start INTEGER NOT NULL,
    valid_time_end INTEGER NOT NULL DEFAULT 9999999999,
    transaction_time_start INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    transaction_time_end INTEGER NOT NULL DEFAULT 9999999999,

    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Foreign keys to entities (ON DELETE CASCADE for cleanup)
    FOREIGN KEY (from_entity) REFERENCES entities(entity_id) ON DELETE CASCADE,
    FOREIGN KEY (to_entity) REFERENCES entities(entity_id) ON DELETE CASCADE,

    -- Time-range constraints
    CHECK (valid_time_start < valid_time_end),
    CHECK (transaction_time_start < transaction_time_end)
);

-- Indexes for relationship queries and graph traversal
CREATE INDEX IF NOT EXISTS idx_relationships_tenant ON relationships(tenant_id);
CREATE INDEX IF NOT EXISTS idx_relationships_from ON relationships(from_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_to ON relationships(to_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_type ON relationships(relationship_type);

-- Composite index for efficient graph traversal queries
CREATE INDEX IF NOT EXISTS idx_relationships_from_type ON relationships(from_entity, relationship_type);

-- B-tree indexes for temporal range queries
CREATE INDEX IF NOT EXISTS idx_relationships_valid_time_start ON relationships(valid_time_start);
CREATE INDEX IF NOT EXISTS idx_relationships_valid_time_end ON relationships(valid_time_end);
CREATE INDEX IF NOT EXISTS idx_relationships_tx_time_start ON relationships(transaction_time_start);
CREATE INDEX IF NOT EXISTS idx_relationships_tx_time_end ON relationships(transaction_time_end);

-- Insert V4 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (4, 'temporal_graph', 'v4-bitemporal-graph');

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- Bi-Temporal Query Patterns (SQLite with Unix timestamps):
--
-- 1. Current state (both dimensions at 'now'):
--    SELECT * FROM entities
--    WHERE valid_time_start <= strftime('%s', 'now') AND valid_time_end > strftime('%s', 'now')
--      AND transaction_time_start <= strftime('%s', 'now') AND transaction_time_end > strftime('%s', 'now')
--
-- 2. Historical state ("What was true at time T?"):
--    SELECT * FROM entities
--    WHERE valid_time_start <= ?T AND valid_time_end > ?T
--      AND transaction_time_start <= strftime('%s', 'now') AND transaction_time_end > strftime('%s', 'now')
--
-- 3. Database snapshot ("What did we know at time T?"):
--    SELECT * FROM entities
--    WHERE transaction_time_start <= ?T AND transaction_time_end > ?T
--
-- 4. Full bi-temporal point query:
--    SELECT * FROM entities
--    WHERE valid_time_start <= ?VT AND valid_time_end > ?VT
--      AND transaction_time_start <= ?TT AND transaction_time_end > ?TT
--
-- Graph Traversal with Recursive CTEs:
--   - SQLite supports recursive CTEs (WITH RECURSIVE)
--   - Use relationships table for edge traversal
--   - Filter by valid_time for temporal consistency
--   - Use json_array() for path tracking and cycle prevention
--   - Example: See SqliteGraphStorage::traverse() implementation
--
-- Performance Considerations:
--   - B-tree indexes on time range boundaries (start/end) accelerate range queries
--   - Valid time queries more common than transaction time
--   - Composite index (from_entity, relationship_type) for graph traversal
--   - JSON properties stored as TEXT, parsed with json_extract()
--
-- Tenant Isolation:
--   - No RLS in SQLite (PostgreSQL feature)
--   - Application enforces tenant_id filtering in WHERE clauses
--   - SqliteBackend sets tenant context, SqliteGraphStorage uses it
--
-- Type Mappings (PostgreSQL -> SQLite):
--   - UUID -> TEXT (stored as 36-char hyphenated UUID string)
--   - TIMESTAMPTZ -> INTEGER (Unix epoch seconds)
--   - JSONB -> TEXT (JSON serialized as text)
--   - 'infinity' -> 9999999999 (Unix timestamp ~2286-11-20)
