-- Migration V7: Generic Key-Value Store (Phase 13c.2.6)
--
-- Creates kv_store table for generic binary-safe storage.
-- KV store provides fallback storage for unrouted keys and arbitrary data persistence.
--
-- SQLite Adaptations from PostgreSQL V7:
--   - UUID -> TEXT (36-char UUID strings via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix timestamps via strftime('%s', 'now'))
--   - BYTEA -> BLOB (binary-safe storage in SQLite)
--   - JSONB -> TEXT (optional metadata stored as JSON text)
--   - RLS policies -> Application-level filtering (SQLite doesn't have RLS)
--
-- Dependencies:
--   - V1: Initial setup (PRAGMA, _migrations table)

-- ============================================================================
-- Table: kv_store (generic key-value storage)
-- ============================================================================

CREATE TABLE IF NOT EXISTS kv_store (
    kv_id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    tenant_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value BLOB NOT NULL, -- Binary-safe storage (BYTEA equivalent)
    metadata TEXT, -- Optional JSON metadata
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),

    -- Unique constraint: One value per (tenant, key)
    UNIQUE(tenant_id, key)
);

-- ============================================================================
-- Indexes for kv_store
-- ============================================================================

-- Index for tenant-based queries (application-level tenant isolation)
CREATE INDEX IF NOT EXISTS idx_kv_store_tenant ON kv_store(tenant_id);

-- Index for key prefix scanning (supports list_keys with prefix)
-- SQLite uses B-tree, so prefix queries are efficient with (tenant_id, key) composite index
CREATE INDEX IF NOT EXISTS idx_kv_store_key_prefix ON kv_store(tenant_id, key);

-- Index for recently updated keys
CREATE INDEX IF NOT EXISTS idx_kv_store_updated ON kv_store(updated_at DESC);

-- Insert V7 migration record
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (7, 'kv_store', 'v7-generic-kv-storage');

-- ============================================================================
-- Migration Notes
-- ============================================================================
--
-- KV Store UPSERT Pattern (set operation):
--   INSERT INTO kv_store (tenant_id, key, value, metadata)
--   VALUES (?1, ?2, ?3, ?4)
--   ON CONFLICT(tenant_id, key) DO UPDATE SET
--     value = excluded.value,
--     metadata = excluded.metadata,
--     updated_at = strftime('%s', 'now');
--
-- KV Store Retrieval (get operation):
--   SELECT value, metadata FROM kv_store
--   WHERE tenant_id = ? AND key = ?;
--
-- Key Prefix Scanning (list_keys operation):
--   SELECT key FROM kv_store
--   WHERE tenant_id = ? AND key LIKE ?||'%'
--   ORDER BY key;
--
-- Key Existence Check:
--   SELECT EXISTS(SELECT 1 FROM kv_store WHERE tenant_id = ? AND key = ?);
--
-- Batch Operations:
--   -- get_batch: SELECT key, value FROM kv_store WHERE tenant_id = ? AND key IN (?, ?, ...);
--   -- set_batch: Use transaction with multiple UPSERT statements
--   -- delete_batch: DELETE FROM kv_store WHERE tenant_id = ? AND key IN (?, ?, ...);
--
-- Tenant Isolation:
--   - No RLS in SQLite (PostgreSQL feature)
--   - Application enforces tenant_id filtering in WHERE clauses
--   - SqliteBackend sets tenant context, SqliteKVStorage uses it
--
-- Type Mappings (PostgreSQL -> SQLite):
--   - UUID -> TEXT (stored as 32-char lowercase hex via hex(randomblob(16)))
--   - TIMESTAMPTZ -> INTEGER (Unix epoch seconds via strftime('%s', 'now'))
--   - BYTEA -> BLOB (binary-safe, no escaping needed)
--   - JSONB -> TEXT (optional metadata as JSON text)
--   - now() -> strftime('%s', 'now')
--
-- Performance Characteristics:
--   - UPSERT pattern uses ON CONFLICT for atomic set operations
--   - Composite (tenant_id, key) index enables efficient prefix scans
--   - BLOB storage is binary-safe and space-efficient
--   - Target: <10ms write, <5ms read, <20ms prefix scan (100 keys)
