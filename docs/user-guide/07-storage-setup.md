# Storage Setup

**Quick start guide for setting up storage backends**

🔗 **Navigation**: [← User Guide](README.md) | [Configuration](03-configuration.md) | [Troubleshooting](10-troubleshooting.md)

---

## Overview

llmspell supports multiple storage backends for different use cases. This guide covers quick setup for the most common scenarios.

**Storage Backends:**
- **InMemory**: Development and testing (fast, ephemeral)
- **SQLite**: Embedded persistence for single-process apps
- **PostgreSQL**: Production multi-tenant deployments with vector search

**What You'll Learn:**
- Docker Compose setup for PostgreSQL (fastest path)
- Basic connection configuration
- Simple backup procedures
- Common troubleshooting steps

**For Deep Technical Details:**
- Complete PostgreSQL reference: [Technical Docs - PostgreSQL Guide](../technical/postgresql-guide.md)

---

## Quick Start: PostgreSQL with Docker

**Fastest path to production-ready storage (5 minutes)**

### 1. Start PostgreSQL

```bash
cd /path/to/rs-llmspell

# Start PostgreSQL 18 with VectorChord extension
docker compose -f docker/postgres/docker-compose.yml up -d

# Verify container is running
docker ps | grep llmspell_postgres_dev
```

**What You Get:**
- PostgreSQL 18 with VectorChord 0.5.3 (vector similarity search)
- pgvector 0.8.1 (embedding storage)
- Pre-configured for llmspell (database, user, extensions)
- Data persists in Docker volume

### 2. Verify Setup

```bash
# Connect to database
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev

# Check extensions
SELECT extname, extversion FROM pg_extension
WHERE extname IN ('vchord', 'vector');

# Expected output:
#  extname | extversion
# ---------+------------
#  vchord  | 0.5.3
#  vector  | 0.8.1
```

### 3. Run Migrations

```bash
# Apply database schema (15 migrations)
cargo run -- database migrate

# Verify tables created
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev \
  -c "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public';"
```

**Expected tables:**
- `embeddings_*` (4 dimension tables: 384, 768, 1536, 3072)
- `graph_entities`, `graph_relationships`
- `procedural_patterns`
- `sessions`, `artifacts`
- `event_log`

### 4. Configure llmspell

Update `config.toml`:

```toml
[storage]
backend = "postgresql"

[storage.postgresql]
host = "localhost"
port = 5432
database = "llmspell_dev"
username = "llmspell"
password = "llmspell_password"
pool_size = 10

# Enable vector search
vector_search = true
dimension = 1536  # Default for OpenAI embeddings
```

### 5. Test Connection

```bash
# Test storage backend
cargo run -- test storage

# Expected output:
# ✅ PostgreSQL connection: OK
# ✅ Vector search: OK (8.47x HNSW speedup)
# ✅ Multi-tenancy: OK (<5% overhead)
```

**You're done!** PostgreSQL storage is ready.

---

## Alternative: Embedded Storage (SQLite)

**For single-process applications (no Docker required)**

### Setup

```toml
# config.toml
[storage]
backend = "sqlite"

[storage.sqlite]
path = "/var/lib/llmspell/data/llmspell.db"
```

**Characteristics:**
- ✅ No external dependencies
- ✅ Embedded relational database
- ✅ ACID transactions
- ✅ Automatic crash recovery
- ✅ Vector similarity search (vectorlite-rs HNSW)
- ✅ Bi-temporal graph storage
- ⚠️ Single-process only (file locking)

**When to Use:**
- CLI applications
- Desktop tools
- Single-server deployments
- Development and testing
- Quick prototypes

---

## Development: In-Memory Storage

**For testing and development (fastest)**

```toml
# config.toml
[storage]
backend = "memory"
```

**Characteristics:**
- ✅ Fastest performance (~microseconds)
- ✅ No setup required
- ✅ Isolated tests
- ⚠️ Data lost on restart
- ⚠️ Not for production

---

## Connection Configuration

### PostgreSQL Connection String

```bash
# Environment variable (highest priority)
export DATABASE_URL="postgresql://llmspell:password@localhost:5432/llmspell_dev"

# Or config.toml
[storage.postgresql]
connection_string = "postgresql://llmspell:password@localhost:5432/llmspell_dev"
```

### Connection Pooling

```toml
[storage.postgresql]
pool_size = 10              # Max connections (default: 10)
pool_timeout_seconds = 30   # Connection timeout (default: 30)
idle_timeout_seconds = 600  # Idle connection timeout (default: 600)
```

**Sizing Guidelines:**
- Development: 5-10 connections
- Production (single instance): 20-50 connections
- Production (multi-instance): Calculate as `(instances × 20) + buffer`

### Health Checks

```bash
# CLI health check
cargo run -- database health

# Expected output:
# ✅ PostgreSQL connection: OK
# ✅ Database version: 18.2
# ✅ Extensions: vchord 0.5.3, vector 0.8.1
# ✅ Tables: 15/15 present
# ✅ Indexes: 24/24 present
```

---

## Basic Backup Procedures

### Quick Backup (Docker)

```bash
# Backup to file
docker exec llmspell_postgres_dev pg_dump -U llmspell llmspell_dev | gzip > backup.sql.gz

# Restore from backup
gunzip -c backup.sql.gz | docker exec -i llmspell_postgres_dev psql -U llmspell -d llmspell_dev
```

### Automated Daily Backups

```bash
#!/bin/bash
# /usr/local/bin/llmspell-backup.sh

BACKUP_DIR="/var/backups/llmspell"
DATE=$(date +%Y%m%d_%H%M%S)

# Create backup
docker exec llmspell_postgres_dev pg_dump -U llmspell llmspell_dev | \
  gzip > "${BACKUP_DIR}/llmspell_${DATE}.sql.gz"

# Keep last 7 days
find $BACKUP_DIR -name "llmspell_*.sql.gz" -mtime +7 -delete

echo "Backup complete: llmspell_${DATE}.sql.gz"
```

**Cron schedule (2 AM daily):**
```bash
0 2 * * * /usr/local/bin/llmspell-backup.sh
```

**For Production Backup Strategies:**
See [Technical Docs - Backup Guide](storage/backup-restore.md) for:
- Point-in-Time Recovery (PITR)
- Continuous WAL archiving
- Disaster recovery procedures
- Multi-region replication

---

## Migration Between Backends

**Lossless bidirectional migration between PostgreSQL and SQLite**

llmspell provides built-in migration tools for moving data between storage backends. This enables seamless transitions between development (SQLite) and production (PostgreSQL) environments, or scaling decisions as your deployment evolves.

### When to Migrate

**Common Migration Scenarios:**

1. **Development → Production**: Migrate local SQLite data to PostgreSQL for deployment
2. **Production → Development**: Export production data for local testing/debugging
3. **Backend Switching**: Scale up (SQLite → PostgreSQL) or scale down (PostgreSQL → SQLite)
4. **Cross-Environment**: Sync data between staging and production
5. **Disaster Recovery**: Restore from JSON export when database is lost

### Quick Migration Example

**SQLite → PostgreSQL (Most Common)**

```bash
# 1. Export from SQLite
llmspell storage export --backend sqlite --output dev-data.json

# 2. Set PostgreSQL connection
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"

# 3. Import to PostgreSQL
llmspell storage import --backend postgres --input dev-data.json

# 4. Verify migration
llmspell storage export --backend postgres --output verify.json
diff <(jq -S .data dev-data.json) <(jq -S .data verify.json)
```

### Migration Prerequisites

**Before migrating, ensure:**

1. **Target backend is running and accessible**
   - PostgreSQL: `DATABASE_URL` environment variable set
   - SQLite: Database file path exists or can be created

2. **Migrations are applied**
   ```bash
   # PostgreSQL
   cargo run -- database migrate

   # SQLite (automatic on first connection)
   ```

3. **Sufficient disk space**
   ```bash
   # Check export size estimate
   du -sh ~/.local/share/llmspell/llmspell.db
   ```

4. **PostgreSQL feature (if using PostgreSQL)**
   ```bash
   cargo build --features postgres
   ```

### Verification Steps

**After migration, verify data integrity:**

1. **Check import statistics**
   ```bash
   llmspell storage import --backend postgres --input data.json
   # ✅ Imported 1,234 total records:
   #   - Vectors: 500
   #   - Entities: 100
   #   - Sessions: 12
   #   ...
   ```

2. **Export from target and compare**
   ```bash
   # Export from source
   llmspell storage export --backend sqlite --output source.json

   # Export from target
   llmspell storage export --backend postgres --output target.json

   # Compare data (ignore timestamps)
   diff <(jq -S .data source.json) <(jq -S .data target.json)
   # Should be empty (identical)
   ```

3. **Run application tests**
   ```bash
   # Test queries work against new backend
   cargo test --test integration_tests
   ```

### Migration Data Format

**Export produces versioned JSON:**
- **Version**: 1.0 (semantic versioning for compatibility)
- **Metadata**: Export timestamp, source backend, applied migrations
- **Data**: All 10 table types (vectors, graph, sessions, etc.)
- **Encoding**: Base64 for binary data (BLOBs)

**What's Included:**
- Vector embeddings (all dimensions: 384, 768, 1536, 3072)
- Knowledge graph (entities and relationships)
- Procedural memory patterns
- Agent state
- Key-value store entries
- Workflow states
- Sessions and artifacts
- Event log
- Hook history

### Rollback Procedure

**If migration fails or needs reversal:**

1. **Keep original backend running** until migration is verified
2. **Export before importing** to create rollback point
3. **Use transactions** (automatic in import process)
   - If import fails, changes are automatically rolled back
   - Database remains in consistent state

**Rollback example:**
```bash
# Before migration, create backup export
llmspell storage export --backend postgres --output pre-migration-backup.json

# If something goes wrong, restore from backup
llmspell storage import --backend postgres --input pre-migration-backup.json
```

### Advanced Migration Workflows

**For complex migration scenarios, see:**
- **[Data Migration Guide](11-data-migration.md)** - Complete workflows with examples
- **[PostgreSQL Guide](../technical/postgresql-guide.md#migration)** - PostgreSQL-specific migration details
- **[CLI Reference](05-cli-reference.md#storage)** - Full command documentation

---

## Multi-Tenancy Setup

### Enable Row-Level Security

```sql
-- Connect to database
\c llmspell_dev

-- Enable RLS on embeddings tables
ALTER TABLE embeddings_1536 ENABLE ROW LEVEL SECURITY;

-- Create policy for tenant isolation
CREATE POLICY tenant_isolation ON embeddings_1536
    USING (metadata->>'tenant_id' = current_setting('app.tenant_id'));

-- Set tenant context in application
SET app.tenant_id = 'tenant_123';
```

### Application Configuration

```toml
[storage.postgresql]
multi_tenant = true
tenant_id_column = "tenant_id"
enable_rls = true
```

**Tenant Isolation Validation:**
```bash
# Test cross-tenant access prevention
cargo test --test multi_tenant_isolation

# Expected: All tests pass, zero cross-tenant leaks
```

---

## Troubleshooting

### Connection Issues

**Problem:** `connection refused`

```bash
# Check PostgreSQL is running
docker ps | grep postgres

# Check logs
docker logs llmspell_postgres_dev --tail 50

# Verify port mapping
docker port llmspell_postgres_dev 5432
```

**Problem:** `authentication failed`

```bash
# Verify credentials
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev

# Reset password if needed
docker exec -it llmspell_postgres_dev psql -U postgres -c \
  "ALTER USER llmspell WITH PASSWORD 'new_password';"
```

### Migration Failures

**Problem:** `migration already applied`

```bash
# Check migration status
cargo run -- database migrations list

# Force re-apply specific migration (if safe)
cargo run -- database migrations apply --migration 001_initial.sql --force
```

**Problem:** `extension "vchord" does not exist`

```bash
# Verify extensions installed
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev -c \
  "SELECT * FROM pg_available_extensions WHERE name IN ('vchord', 'vector');"

# If missing, rebuild container
docker compose -f docker/postgres/docker-compose.yml down -v
docker compose -f docker/postgres/docker-compose.yml up -d
```

### Performance Issues

**Problem:** Slow queries

```bash
# Check slow queries
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev -c \
  "SELECT query, mean_exec_time FROM pg_stat_statements ORDER BY mean_exec_time DESC LIMIT 10;"

# Analyze table statistics
ANALYZE embeddings_1536;
```

**For Advanced Troubleshooting:**
- See [Troubleshooting Guide](10-troubleshooting.md) for common issues
- See [Technical Docs - PostgreSQL Guide](../technical/postgresql-guide.md#3-performance-tuning) for query optimization

---

## Next Steps

### Learn More

- **Configuration**: [Configuration Guide](03-configuration.md) - All storage options
- **CLI Commands**: [CLI Reference](05-cli-reference.md) - Database management commands
- **Deployment**: [Deployment Guide](08-deployment.md) - Production setup

### Technical Deep Dives

- **PostgreSQL Guide**: [PostgreSQL Guide](../technical/postgresql-guide.md) - Complete reference (schema, performance, security, migration)
- **Backup/Restore**: [Backup Guide](storage/backup-restore.md) - Disaster recovery, PITR, automation

### Developer Resources

- **Storage Backends**: [Developer Guide - Storage](../developer-guide/reference/storage-backends.md)
- **Extending Storage**: [Developer Guide - Extending llmspell](../developer-guide/extending-llmspell.md)

---

**Version**: 0.13.0 | **Phase**: 13b.18.2 | **Last Updated**: 2025-11-08
