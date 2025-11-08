# PostgreSQL Setup for rs-llmspell

**Phase 13b Storage Backend**: Comprehensive setup guide for PostgreSQL 18 with VectorChord 0.5.3

## Table of Contents

- [Overview](#overview)
- [Quick Start (Docker)](#quick-start-docker)
- [Manual Installation](#manual-installation)
  - [macOS Installation](#macos-installation)
  - [Linux Installation](#linux-installation)
- [Database Configuration](#database-configuration)
- [Multi-Tenancy Setup](#multi-tenancy-setup)
- [Connection Pooling](#connection-pooling)
- [Health Checks](#health-checks)
- [Backend Configuration](#backend-configuration)
- [Troubleshooting](#troubleshooting)

---

## Overview

rs-llmspell uses PostgreSQL 18 with the VectorChord extension for production storage backends. This setup provides:

- **Vector Similarity Search**: VectorChord HNSW indexes for episodic memory (8.47x speedup vs linear scan)
- **Bi-Temporal Graph**: GiST-indexed temporal knowledge graph for semantic memory
- **Multi-Tenant Isolation**: Row-Level Security (RLS) with <5% overhead
- **JSONB Storage**: GIN-indexed flexible schemas for procedural patterns, agent state, workflows
- **Event Sourcing**: Partitioned event log with time-range queries

**10 Storage Backends Supported:**
1. **Vector Embeddings**: 4 tables (384, 768, 1536, 3072 dimensions)
2. **Temporal Graph**: Entities + relationships with bi-temporal tracking
3. **Procedural Memory**: Patterns and consolidation rules
4. **Agent State**: Persistent agent configurations and state
5. **Workflow States**: Execution context and progress tracking
6. **Sessions**: Session management with artifact references
7. **Artifacts**: Large object storage with metadata
8. **Event Log**: Immutable audit trail with partitioning
9. **Hook History**: Hook execution history with replay support
10. **API Keys**: Secure key storage with tenant isolation

**System Requirements:**
- PostgreSQL 18+ (required for VectorChord 0.5.3 compatibility)
- 2 GB RAM minimum (4 GB recommended for production)
- 10 GB disk space (scales with data volume)
- VectorChord 0.5.3 extension (installs pgvector 0.8.1 as dependency)

---

## Quick Start (Docker)

**Fastest path to production-ready PostgreSQL with VectorChord.**

### 1. Start PostgreSQL Container

```bash
cd /path/to/rs-llmspell

# Start VectorChord-enabled PostgreSQL 18
docker compose -f docker/postgres/docker-compose.yml up -d

# Verify container health
docker ps | grep llmspell_postgres_dev

# Check logs for successful startup
docker logs llmspell_postgres_dev
```

**Expected output:**
```
PostgreSQL init process complete; ready for start up.
database system is ready to accept connections
```

### 2. Verify Extensions

```bash
# Connect to database
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev

# Verify VectorChord + pgvector installed
SELECT extname, extversion FROM pg_extension WHERE extname IN ('vchord', 'vector', 'uuid-ossp', 'pgcrypto');

# Expected output:
#   extname   | extversion
# ------------+------------
#   vchord    | 0.5.3
#   vector    | 0.8.1
#   uuid-ossp | 1.1
#   pgcrypto  | 1.3
```

### 3. Run Migrations

```bash
# From project root
cargo run -- database migrate

# Verify 15 migrations applied
docker exec -it llmspell_postgres_dev psql -U llmspell -d llmspell_dev \
  -c "SELECT version, name FROM refinery_schema_history ORDER BY version;"

# Expected: V1 through V15 listed
```

### 4. Test Connection

```bash
# Configure environment
export DATABASE_URL="postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev"

# Test with llmspell CLI
cargo run -- config set storage.backend postgres
cargo run -- config set storage.postgres.url "$DATABASE_URL"

# Verify connection
cargo run -- database health-check

# Expected output:
# ✓ PostgreSQL connection healthy
# ✓ VectorChord extension available
# ✓ All 15 migrations applied
# ✓ RLS policies active (12 tables)
```

**Docker Configuration Details:**

The `docker/postgres/docker-compose.yml` file provides:

```yaml
services:
  postgres:
    image: ghcr.io/tensorchord/vchord-postgres:pg18-v0.5.3
    container_name: llmspell_postgres_dev
    environment:
      POSTGRES_DB: llmspell_dev
      POSTGRES_USER: llmspell
      POSTGRES_PASSWORD: llmspell_dev_pass
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./init-scripts:/docker-entrypoint-initdb.d
    command: >
      postgres
      -c shared_preload_libraries='vchord'
      -c max_connections=100
      -c shared_buffers=512MB
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U llmspell"]
      interval: 10s
      timeout: 5s
      retries: 5
```

**Key settings:**
- `shared_preload_libraries='vchord'`: Loads VectorChord on startup (required)
- `max_connections=100`: Connection limit (tune based on workload)
- `shared_buffers=512MB`: Shared memory for caching (25% of RAM typical)
- `init-scripts`: Runs `01-extensions.sql` to create schema and roles

**Init script (`docker/postgres/init-scripts/01-extensions.sql`):**
- Installs `vchord`, `pgcrypto`, `uuid-ossp` extensions
- Creates `llmspell` schema
- Creates `llmspell_app` role (non-superuser for RLS)
- Grants permissions for application access

---

## Manual Installation

**For production deployments without Docker.**

### macOS Installation

#### Prerequisites

```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Update Homebrew
brew update
```

#### Install PostgreSQL 18

```bash
# Add PostgreSQL repository (for PostgreSQL 18)
brew tap homebrew/services

# Install PostgreSQL 18
brew install postgresql@18

# Start PostgreSQL service
brew services start postgresql@18

# Verify PostgreSQL running
psql --version
# Expected: psql (PostgreSQL) 18.x
```

#### Install VectorChord Extension

**VectorChord is not available via Homebrew. Use Docker for macOS, or build from source:**

```bash
# Option 1: Use Docker (RECOMMENDED for macOS)
# See Quick Start (Docker) section above

# Option 2: Build from source (advanced users)
# https://github.com/tensorchord/VectorChord#building-from-source

# WARNING: Building VectorChord requires:
# - Rust 1.70+
# - PGRX framework
# - PostgreSQL development headers
# - 30-60 minutes compile time
```

**Recommendation for macOS:** Use Docker setup (Quick Start section) for VectorChord. Manual installation requires complex source builds.

#### Create Database (PostgreSQL without VectorChord)

If using standard PostgreSQL without VectorChord:

```bash
# Create database and user
createuser -s llmspell
createdb -O llmspell llmspell_dev

# Create application role
psql -U llmspell -d llmspell_dev <<EOF
CREATE ROLE llmspell_app WITH LOGIN PASSWORD 'llmspell_dev_pass';
GRANT CONNECT ON DATABASE llmspell_dev TO llmspell_app;
EOF

# Install pgvector (vector search without VectorChord HNSW)
cd /tmp
git clone --branch v0.8.1 https://github.com/pgvector/pgvector.git
cd pgvector
make
make install

# Enable extensions
psql -U llmspell -d llmspell_dev <<EOF
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE SCHEMA IF NOT EXISTS llmspell;
GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
EOF
```

**Performance note:** Without VectorChord, vector search uses `ivfflat` indexes (slower than HNSW). For production, use Docker with VectorChord.

### Linux Installation

#### Ubuntu/Debian

```bash
# Add PostgreSQL APT repository
sudo apt update
sudo apt install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh

# Install PostgreSQL 18
sudo apt install -y postgresql-18 postgresql-server-dev-18

# Start PostgreSQL service
sudo systemctl start postgresql@18-main
sudo systemctl enable postgresql@18-main

# Verify PostgreSQL running
psql --version
# Expected: psql (PostgreSQL) 18.x
```

#### Install VectorChord on Linux

```bash
# Install build dependencies
sudo apt install -y build-essential git curl pkg-config libssl-dev

# Install Rust (required for VectorChord)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install PGRX (PostgreSQL Rust extension framework)
cargo install --locked cargo-pgrx
cargo pgrx init --pg18 $(which pg_config)

# Clone and build VectorChord
cd /tmp
git clone --branch v0.5.3 https://github.com/tensorchord/VectorChord.git
cd VectorChord
cargo pgrx install --pg-config $(which pg_config)

# Verify installation
sudo -u postgres psql -c "CREATE EXTENSION IF NOT EXISTS vchord CASCADE;"
sudo -u postgres psql -c "SELECT extname, extversion FROM pg_extension WHERE extname IN ('vchord', 'vector');"
```

#### Create Database

```bash
# Switch to postgres user
sudo -u postgres psql

# Create database and user
CREATE USER llmspell WITH PASSWORD 'llmspell_prod_pass' SUPERUSER;
CREATE DATABASE llmspell_prod OWNER llmspell;

# Create application role (non-superuser for RLS)
CREATE ROLE llmspell_app WITH LOGIN PASSWORD 'app_secure_password';
GRANT CONNECT ON DATABASE llmspell_prod TO llmspell_app;

# Connect to database and enable extensions
\c llmspell_prod
CREATE EXTENSION IF NOT EXISTS vchord CASCADE;
CREATE EXTENSION IF NOT EXISTS pgcrypto;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

# Create schema and grant permissions
CREATE SCHEMA IF NOT EXISTS llmspell;
GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
GRANT CREATE ON SCHEMA llmspell TO llmspell; -- for migrations

# Exit psql
\q
```

#### Configure PostgreSQL

Edit `/etc/postgresql/18/main/postgresql.conf`:

```conf
# Connection settings
listen_addresses = '*'                  # or specific IP for security
max_connections = 100

# Memory settings (adjust based on available RAM)
shared_buffers = 2GB                    # 25% of RAM
effective_cache_size = 6GB              # 75% of RAM
maintenance_work_mem = 512MB
work_mem = 64MB

# VectorChord settings
shared_preload_libraries = 'vchord'     # REQUIRED for VectorChord

# WAL settings (for durability)
wal_level = replica
max_wal_size = 4GB
min_wal_size = 1GB

# Performance settings
random_page_cost = 1.1                  # SSD storage
effective_io_concurrency = 200          # SSD parallelism
```

Edit `/etc/postgresql/18/main/pg_hba.conf`:

```conf
# TYPE  DATABASE        USER            ADDRESS                 METHOD

# Local connections
local   all             postgres                                peer
local   all             llmspell                                peer
local   all             llmspell_app                            scram-sha-256

# IPv4 local connections
host    llmspell_prod   llmspell        127.0.0.1/32            scram-sha-256
host    llmspell_prod   llmspell_app    127.0.0.1/32            scram-sha-256

# IPv4 network connections (adjust subnet for security)
host    llmspell_prod   llmspell        10.0.0.0/8              scram-sha-256
host    llmspell_prod   llmspell_app    10.0.0.0/8              scram-sha-256
```

Restart PostgreSQL:

```bash
sudo systemctl restart postgresql@18-main
```

---

## Database Configuration

### Connection String Format

```
postgresql://[user]:[password]@[host]:[port]/[database]?[params]
```

**Examples:**

```bash
# Development (Docker)
DATABASE_URL="postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev"

# Production (Linux)
DATABASE_URL="postgresql://llmspell_app:secure_pass@db.example.com:5432/llmspell_prod?sslmode=require"

# Connection pooling (via PgBouncer)
DATABASE_URL="postgresql://llmspell_app:secure_pass@localhost:6432/llmspell_prod?application_name=llmspell&connect_timeout=10"
```

**SSL/TLS parameters:**
- `sslmode=disable`: No SSL (development only)
- `sslmode=prefer`: Try SSL, fall back to plain (default)
- `sslmode=require`: Require SSL (production recommended)
- `sslmode=verify-ca`: Require SSL with CA verification
- `sslmode=verify-full`: Require SSL with full certificate validation

### rs-llmspell Configuration

Edit `~/.config/llmspell/config.toml`:

```toml
[storage]
backend = "postgres"  # "memory", "sled", or "postgres"

[storage.postgres]
# Connection URL
url = "postgresql://llmspell_app:secure_pass@localhost:5432/llmspell_prod"

# Connection pool settings
pool_size = 20              # Max connections (default: 16)
pool_timeout_secs = 30      # Connection acquisition timeout
idle_timeout_secs = 600     # Close idle connections after 10 minutes
max_lifetime_secs = 1800    # Recycle connections after 30 minutes

# Tenant isolation (RLS configuration)
default_tenant_id = "default"  # Fallback tenant for non-multi-tenant usage
enforce_tenant_isolation = true  # Enable RLS enforcement (production: true)

# Migration settings
auto_migrate = false        # Run migrations on startup (development: true, production: false)
migration_timeout_secs = 300  # Migration execution timeout
```

**Environment variable override:**

```bash
# Override DATABASE_URL
export LLMSPELL_STORAGE_POSTGRES_URL="postgresql://..."

# Override pool size
export LLMSPELL_STORAGE_POSTGRES_POOL_SIZE=25
```

---

## Multi-Tenancy Setup

rs-llmspell uses PostgreSQL Row-Level Security (RLS) for multi-tenant isolation.

### Architecture

```
┌──────────────────────────────────────────┐
│  Application (llmspell_app role)         │
│  Sets: app.current_tenant_id = 'tenant1' │
└────────────────┬─────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│  PostgreSQL RLS Policies                 │
│  WHERE tenant_id = current_setting(...)  │
└────────────────┬─────────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────────┐
│  12 Tables with RLS                      │
│  - vector_embeddings_{384,768,1536,3072} │
│  - entities, relationships               │
│  - procedural_memory                     │
│  - agent_state, workflow_states          │
│  - sessions, artifacts                   │
│  - event_log, hook_history, api_keys     │
└──────────────────────────────────────────┘
```

### Tenant Isolation Guarantee

**Every query is automatically filtered by `tenant_id`:**

```sql
-- Application executes:
SELECT * FROM llmspell.sessions;

-- PostgreSQL RLS transforms to:
SELECT * FROM llmspell.sessions
WHERE tenant_id = current_setting('app.current_tenant_id', true);
```

**Cross-tenant access is impossible** (even with SQL injection):
```sql
-- Malicious query attempt
SELECT * FROM llmspell.sessions WHERE tenant_id = 'other_tenant';

-- RLS enforces AND clause:
SELECT * FROM llmspell.sessions
WHERE tenant_id = 'other_tenant'
  AND tenant_id = current_setting('app.current_tenant_id', true);
-- Result: 0 rows (cannot access other tenant's data)
```

### Setting Tenant Context

**Rust API:**

```rust
use llmspell_storage::postgres::set_tenant_context;

async fn example(pool: &deadpool_postgres::Pool) -> Result<(), Error> {
    let conn = pool.get().await?;

    // Set tenant context for this connection
    set_tenant_context(&conn, "tenant-123").await?;

    // All queries on this connection are tenant-scoped
    let sessions = fetch_sessions(&conn).await?;
    // Returns only sessions where tenant_id = 'tenant-123'

    Ok(())
}
```

**SQL:**

```sql
-- Set tenant context (must be called before queries)
SET LOCAL app.current_tenant_id = 'tenant-123';

-- Query tenant-scoped data
SELECT * FROM llmspell.sessions;
-- RLS automatically filters to tenant-123
```

### RLS Performance

**Overhead:** <5% (validated in Phase 13b.3)

**Benchmark results:**
```
Without RLS:  1000 queries in 245ms (4,082 qps)
With RLS:     1000 queries in 257ms (3,892 qps)
Overhead:     4.9% (well below 5% target)
```

**Index strategy to minimize overhead:**
```sql
-- Composite indexes with tenant_id as first column
CREATE INDEX idx_sessions_tenant_created
  ON llmspell.sessions(tenant_id, created_at);

-- PostgreSQL can use index scan instead of sequential scan
```

### Multi-Tenant Configuration

```toml
[storage.postgres]
# Enable strict tenant isolation
enforce_tenant_isolation = true

# Default tenant for non-multi-tenant usage
default_tenant_id = "default"

# Optional: Tenant ID validation regex
tenant_id_pattern = "^[a-z0-9-]{3,255}$"
```

---

## Connection Pooling

rs-llmspell uses `deadpool-postgres` for connection pooling.

### Pool Configuration

```toml
[storage.postgres]
pool_size = 20              # Max connections
pool_timeout_secs = 30      # Wait 30s for available connection
idle_timeout_secs = 600     # Close idle connections after 10 min
max_lifetime_secs = 1800    # Recycle connections after 30 min
```

**Tuning Guidelines:**

| Workload | pool_size | Rationale |
|----------|-----------|-----------|
| **Development** | 5-10 | Low concurrency, fast iteration |
| **Web API** | 50-100 | High concurrency, short-lived requests |
| **Background Jobs** | 10-20 | Medium concurrency, long-running tasks |
| **Embedded** | 1-5 | Single-user, local database |

**Formula:** `pool_size = (number of CPU cores × 2) + effective_spindle_count`
- For SSD: `effective_spindle_count = 1`
- For HDD: `effective_spindle_count = number of disks`

**Example:** 8-core server with SSD:
```
pool_size = (8 × 2) + 1 = 17
Round up to 20 for headroom
```

### PostgreSQL Connection Limits

PostgreSQL `max_connections` must be ≥ sum of all connection pools:

```
max_connections = (llmspell_pool × instances) + admin_reserve + margin

Example:
- 3 llmspell instances × 20 connections = 60
- Admin reserve (psql, monitoring) = 10
- Safety margin = 30
- Total: max_connections = 100
```

### Connection Pooler (PgBouncer)

**For high-concurrency deployments (100+ clients):**

```bash
# Install PgBouncer
sudo apt install pgbouncer

# Configure /etc/pgbouncer/pgbouncer.ini
[databases]
llmspell_prod = host=localhost port=5432 dbname=llmspell_prod

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 6432
auth_type = scram-sha-256
auth_file = /etc/pgbouncer/userlist.txt
pool_mode = transaction       # transaction pooling (recommended)
max_client_conn = 1000        # Application connections
default_pool_size = 25        # PostgreSQL connections per database
reserve_pool_size = 5         # Reserve for admin queries
```

**Update rs-llmspell config:**

```toml
[storage.postgres]
# Connect to PgBouncer instead of PostgreSQL directly
url = "postgresql://llmspell_app:secure_pass@localhost:6432/llmspell_prod"
pool_size = 50  # Can be larger since PgBouncer multiplexes connections
```

---

## Health Checks

### CLI Health Check

```bash
cargo run -- database health-check
```

**Expected output:**
```
✓ PostgreSQL connection healthy
✓ VectorChord extension available (v0.5.3)
✓ pgvector extension available (v0.8.1)
✓ All 15 migrations applied
✓ RLS policies active (12 tables)
✓ Connection pool healthy (5/20 connections active)
```

### SQL Health Check

```sql
-- Extension versions
SELECT extname, extversion FROM pg_extension WHERE extname IN ('vchord', 'vector', 'uuid-ossp', 'pgcrypto');

-- Migration status
SELECT version, name, applied_on FROM refinery_schema_history ORDER BY version DESC LIMIT 5;

-- RLS status (should be 12 tables with RLS enabled)
SELECT schemaname, tablename, rowsecurity
FROM pg_tables
WHERE schemaname = 'llmspell' AND rowsecurity = true;

-- Connection pool usage
SELECT count(*) AS active_connections,
       max_connections
FROM pg_stat_activity, (SELECT setting::int AS max_connections FROM pg_settings WHERE name = 'max_connections') s
WHERE datname = 'llmspell_prod';
```

### Monitoring Queries

```sql
-- Long-running queries (>5 seconds)
SELECT pid, now() - query_start AS duration, state, query
FROM pg_stat_activity
WHERE state != 'idle' AND now() - query_start > interval '5 seconds'
ORDER BY duration DESC;

-- Database size
SELECT pg_size_pretty(pg_database_size('llmspell_prod')) AS db_size;

-- Table sizes
SELECT schemaname || '.' || tablename AS table,
       pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS size
FROM pg_tables
WHERE schemaname = 'llmspell'
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC;

-- Index usage (unused indexes waste space)
SELECT schemaname, tablename, indexname,
       idx_scan AS index_scans,
       pg_size_pretty(pg_relation_size(indexrelid)) AS index_size
FROM pg_stat_user_indexes
WHERE schemaname = 'llmspell' AND idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## Backend Configuration

rs-llmspell supports 3 storage backends: **memory**, **sled**, **postgres**.

### Backend Selection

```toml
[storage]
# Backend: "memory" (in-memory, testing only), "sled" (embedded DB), "postgres" (production)
backend = "postgres"
```

### Per-Component Backend Configuration

**All 10 components use the global `storage.backend` setting by default:**

```toml
[storage]
backend = "postgres"  # Applies to all 10 components

# Components:
# 1. Vector embeddings (vector_embeddings_{384,768,1536,3072})
# 2. Temporal graph (entities, relationships)
# 3. Procedural memory (procedural_memory)
# 4. Agent state (agent_state)
# 5. Workflow states (workflow_states)
# 6. Sessions (sessions)
# 7. Artifacts (artifacts)
# 8. Event log (event_log)
# 9. Hook history (hook_history)
# 10. API keys (api_keys)
```

### Hybrid Backend Configuration (Advanced)

**Override per-component backend:**

```toml
[storage]
backend = "postgres"  # Default for all components

[storage.memory]
backend = "memory"    # Override: use in-memory for episodic memory (testing)

[storage.state]
backend = "sled"      # Override: use sled for agent state (embedded)

# Result:
# - Vector embeddings, sessions, events → postgres
# - Episodic memory → memory (in-memory)
# - Agent state → sled (embedded DB)
```

**Rationale for hybrid backends:**
- **Testing:** Use `memory` backend for fast test execution
- **Embedded deployments:** Use `sled` for agent/workflow state (no PostgreSQL dependency)
- **High-throughput logging:** Use `postgres` for event log (durability + partitioning)

### Component-Specific Configuration

```toml
[storage.postgres.vector_embeddings]
# HNSW index parameters (per-dimension table)
hnsw_m = 16                # HNSW graph connectivity (higher = better recall, more memory)
hnsw_ef_construction = 128 # Build-time search depth (higher = slower build, better index)

[storage.postgres.event_log]
# Event log partitioning
partition_strategy = "monthly"  # "daily", "weekly", "monthly"
retention_days = 365            # Purge partitions older than 1 year

[storage.postgres.artifacts]
# Large object storage (TOAST compression)
compression_threshold_bytes = 1048576  # Compress artifacts >1 MB (default: 2 KB)
```

---

## Troubleshooting

### Connection Refused

**Error:**
```
Error: Connection refused (os error 61)
```

**Solutions:**

1. **Verify PostgreSQL running:**
   ```bash
   # Docker
   docker ps | grep llmspell_postgres_dev

   # Linux systemd
   sudo systemctl status postgresql@18-main

   # macOS Homebrew
   brew services list | grep postgresql
   ```

2. **Check port availability:**
   ```bash
   nc -zv localhost 5432
   # Expected: Connection to localhost port 5432 [tcp/postgresql] succeeded!
   ```

3. **Verify `listen_addresses` in `postgresql.conf`:**
   ```conf
   listen_addresses = '*'  # or 'localhost' for local-only
   ```

4. **Check firewall rules:**
   ```bash
   # Linux (allow PostgreSQL port)
   sudo ufw allow 5432/tcp
   ```

### Authentication Failed

**Error:**
```
Error: FATAL: password authentication failed for user "llmspell_app"
```

**Solutions:**

1. **Verify credentials:**
   ```bash
   psql -U llmspell_app -d llmspell_prod -W
   # Enter password and verify connection
   ```

2. **Check `pg_hba.conf` authentication method:**
   ```conf
   # Use scram-sha-256 for password authentication
   host    llmspell_prod   llmspell_app    127.0.0.1/32    scram-sha-256
   ```

3. **Reload PostgreSQL after config changes:**
   ```bash
   # Docker
   docker exec llmspell_postgres_dev pg_ctl reload

   # Linux
   sudo systemctl reload postgresql@18-main
   ```

### Extension Not Found

**Error:**
```
Error: extension "vchord" is not available
```

**Solutions:**

1. **Verify VectorChord image (Docker):**
   ```bash
   docker inspect llmspell_postgres_dev | grep Image
   # Expected: ghcr.io/tensorchord/vchord-postgres:pg18-v0.5.3
   ```

2. **Verify extension installed (manual install):**
   ```bash
   ls /usr/share/postgresql/18/extension/ | grep vchord
   # Expected: vchord--0.5.3.sql, vchord.control
   ```

3. **Check `shared_preload_libraries`:**
   ```sql
   SHOW shared_preload_libraries;
   -- Expected: vchord (or vchord,other_extensions)
   ```

### Migration Failures

**Error:**
```
Error: Migration V3__vector_embeddings.sql failed: relation "llmspell.vector_embeddings_384" already exists
```

**Solutions:**

1. **Check migration status:**
   ```bash
   cargo run -- database migration-status
   ```

2. **Rollback to previous migration (manual):**
   ```sql
   -- Drop problematic migration objects
   DROP TABLE IF EXISTS llmspell.vector_embeddings_384 CASCADE;

   -- Update migration history
   DELETE FROM refinery_schema_history WHERE version = 3;
   ```

3. **Re-run migrations:**
   ```bash
   cargo run -- database migrate
   ```

### RLS Policy Violations

**Error:**
```
Error: new row violates row-level security policy for table "sessions"
```

**Solutions:**

1. **Verify tenant context is set:**
   ```rust
   set_tenant_context(&conn, "tenant-123").await?;
   ```

2. **Check RLS policies:**
   ```sql
   SELECT schemaname, tablename, policyname, cmd, qual
   FROM pg_policies
   WHERE schemaname = 'llmspell' AND tablename = 'sessions';
   ```

3. **Disable RLS for debugging (development only):**
   ```sql
   ALTER TABLE llmspell.sessions DISABLE ROW LEVEL SECURITY;
   ```

### Performance Issues

**Symptom:** Queries taking >100ms

**Diagnostics:**

1. **Check query execution plans:**
   ```sql
   EXPLAIN ANALYZE
   SELECT * FROM llmspell.sessions WHERE tenant_id = 'tenant-123';
   ```

2. **Verify indexes are being used:**
   ```sql
   SELECT schemaname, tablename, indexname, idx_scan
   FROM pg_stat_user_indexes
   WHERE schemaname = 'llmspell'
   ORDER BY idx_scan ASC;
   ```

3. **Run VACUUM ANALYZE:**
   ```sql
   VACUUM ANALYZE llmspell.sessions;
   ```

4. **Check connection pool exhaustion:**
   ```bash
   # Monitor active connections
   watch -n 1 'psql -U llmspell -d llmspell_prod -c "SELECT count(*) FROM pg_stat_activity WHERE datname = '\''llmspell_prod'\'';"'
   ```

---

## Next Steps

- **Schema Reference**: See [schema-reference.md](./schema-reference.md) for detailed table schemas
- **Performance Tuning**: See [performance-tuning.md](./performance-tuning.md) for HNSW optimization
- **Backup/Restore**: See [backup-restore.md](./backup-restore.md) for disaster recovery procedures
- **Migration Guide**: See [migration-guide.md](./migration-guide.md) for upgrading from memory/sled backends

**Configuration Examples:**

```bash
# View all storage backends
cargo run -- config get storage

# Switch to PostgreSQL backend
cargo run -- config set storage.backend postgres
cargo run -- config set storage.postgres.url "postgresql://..."

# Test configuration
cargo run -- database health-check
```
