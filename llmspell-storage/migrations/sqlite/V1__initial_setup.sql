-- SQLite Initial Setup (Phase 13c.2.1)
--
-- Configures SQLite for optimal performance and creates migration tracking table.
-- WAL mode enables concurrent reads while writing, PRAGMA settings optimize for workload.

-- Enable foreign key constraints (must be set per connection in SQLite)
PRAGMA foreign_keys = ON;

-- Use Write-Ahead Logging for better concurrency
-- WAL allows multiple readers even during writes
PRAGMA journal_mode = WAL;

-- NORMAL synchronous mode balances safety and performance
-- Still durable but doesn't wait for OS cache to flush to disk on every transaction
PRAGMA synchronous = NORMAL;

-- Increase cache size to 64MB (default is 2MB)
-- Negative value means KB, -64000 = 64MB cache
PRAGMA cache_size = -64000;

-- Use memory-mapped I/O for reads (100MB)
-- Faster than system calls for small databases
PRAGMA mmap_size = 104857600;

-- Analyze table statistics on first connection
-- Helps query planner make better decisions
PRAGMA optimize;

-- Migration version tracking table
-- Stores applied migrations to prevent re-application and enable rollback
CREATE TABLE IF NOT EXISTS _migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    applied_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    checksum TEXT NOT NULL
);

-- Create index for migration lookup
CREATE INDEX IF NOT EXISTS idx_migrations_name ON _migrations(name);

-- Insert V1 migration record
-- Checksum is placeholder for migration content hash
INSERT OR IGNORE INTO _migrations (version, name, checksum)
VALUES (1, 'initial_setup', 'v1-initial');
