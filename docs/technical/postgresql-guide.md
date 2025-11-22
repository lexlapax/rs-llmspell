# PostgreSQL Production Guide

**Version**: 0.13.0 (Phase 13b Complete)
**Status**: Production Ready
**Last Updated**: January 2025

> **🎯 Purpose**: Comprehensive PostgreSQL reference for rs-llmspell storage backend - from setup to production operations

**🔗 Navigation**: [← Technical Docs](README.md) | [Current Architecture](current-architecture.md) | [Storage Architecture](storage-architecture.md)

---

## Table of Contents

1. [Overview & Architecture](#overview--architecture)
2. [Setup & Configuration](#setup--configuration)
3. [Schema Reference](#schema-reference)
4. [Security & Multi-Tenancy](#security--multi-tenancy)
5. [Performance Optimization](#performance-optimization)
6. [Operations & Migration](#operations--migration)

---

## Overview & Architecture

### 3-Tier Storage Architecture

LLMSpell implements a hot-swappable storage backend system with PostgreSQL as the production option:

```text
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: Component-Specific Storage APIs (10 components)       │
│  ├─ VectorStorage (4 dimensions: 384, 768, 1536, 3072)         │
│  ├─ EpisodicMemoryStorage (conversation history)               │
│  ├─ SemanticMemoryStorage (bi-temporal graph)                  │
│  ├─ ProceduralMemoryStorage (patterns, skills)                 │
│  ├─ AgentStateStorage (agent snapshots)                        │
│  ├─ WorkflowStateStorage (execution tracking)                  │
│  ├─ SessionStorage (session lifecycle)                         │
│  ├─ ArtifactStorage (content-addressed files)                  │
│  ├─ EventLogStorage (partitioned events)                       │
│  └─ HookHistoryStorage (hook execution audit)                  │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 2: Unified StorageBackend Trait (12 methods)             │
│  ├─ get/set/delete (CRUD operations)                           │
│  ├─ exists/list_keys (querying)                                │
│  ├─ get_batch/set_batch/delete_batch (bulk ops)                │
│  └─ backend_type(), characteristics() (introspection)          │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: PostgresBackend Implementation                        │
│  ├─ PostgreSQL 18+ with VectorChord 0.5.3                      │
│  ├─ 15 tables across 15 migrations (2,434 lines SQL)           │
│  ├─ Multi-tenant RLS isolation (<5% overhead)                  │
│  └─ Connection pooling: (CPU × 2) + 1                          │
└─────────────────────────────────────────────────────────────────┘
```

### PostgreSQL Configuration

**Requirements**:
- PostgreSQL 18+
- Extensions: `vchord 0.5.3`, `vector 0.8.1`, `uuid-ossp`, `pgcrypto`
- Schema: `llmspell`
- Roles: `llmspell` (admin), `llmspell_app` (non-superuser for RLS)

**Schema Summary**:
- **15 migrations** (V1-V15): 2,434 lines of SQL DDL
- **15+ tables**: 10 storage backends across 12+ tables (vector embeddings split into 4 dimension tables)
- **Bi-temporal tracking**: Valid time + transaction time for entities and relationships
- **Content addressing**: Deduplication via blake3 hashing for artifacts
- **Partitioning**: Monthly range partitioning for event log
- **Row-Level Security**: Multi-tenant isolation on all tables via RLS
- **Index optimization**: HNSW (vectors), GiST (temporal ranges), GIN (JSONB)

### Backend Comparison

| Feature | Memory | SQLite | **PostgreSQL** |
|---------|--------|--------|---------------|
| **Persistent** | ❌ No | ✅ Yes | ✅ Yes |
| **Transactional** | ✅ Yes (RwLock) | ✅ Yes | ✅ Yes (ACID) |
| **Multi-tenant** | ❌ No | ❌ No | ✅ Yes (RLS) |
| **Multi-writer** | ❌ No | ❌ No | ✅ Yes |
| **Read Latency** | ~1 µs | ~50 µs | ~500 µs (local) |
| **Write Latency** | ~2 µs | ~200 µs | ~1000 µs (local) |
| **Scalability** | RAM-limited | Single-machine | Horizontal (replicas) |
| **Use Case** | Testing | Development | **Production** |

**Why PostgreSQL for Production**:
- Multi-writer concurrency
- Horizontal scalability (read replicas)
- ACID transactions with WAL durability
- Row-Level Security for multi-tenancy
- Vector similarity search (VectorChord HNSW)
- Bi-temporal graph support
- Point-in-Time Recovery (PITR)
- Rich querying (SQL, CTEs, window functions)

---

## Setup & Configuration

### Installation (Docker - Recommended)

**Quick Start** (5 minutes):

```bash
# 1. Pull Docker image with VectorChord
docker pull tensorchord/vchord-postgres:pg18-v0.5.3

# 2. Run PostgreSQL container
docker run -d \
  --name llmspell-postgres \
  -e POSTGRES_USER=llmspell \
  -e POSTGRES_PASSWORD=llmspell_dev_pass \
  -e POSTGRES_DB=llmspell_dev \
  -p 5432:5432 \
  -v llmspell_data:/var/lib/postgresql/data \
  tensorchord/vchord-postgres:pg18-v0.5.3

# 3. Wait for health check
sleep 10

# 4. Verify installation
docker exec llmspell-postgres psql -U llmspell -d llmspell_dev -c '\dx'
# Expected output: vchord 0.5.3, vector 0.8.1, uuid-ossp, pgcrypto
```

**Docker Compose** (`docker-compose.yml`):

```yaml
version: '3.8'
services:
  postgres:
    image: tensorchord/vchord-postgres:pg18-v0.5.3
    container_name: llmspell-postgres
    environment:
      POSTGRES_USER: llmspell
      POSTGRES_PASSWORD: llmspell_dev_pass
      POSTGRES_DB: llmspell_dev
      POSTGRES_INITDB_ARGS: "--auth-host=scram-sha-256"
    ports:
      - "5432:5432"
    volumes:
      - llmspell_data:/var/lib/postgresql/data
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U llmspell"]
      interval: 10s
      timeout: 5s
      retries: 5

volumes:
  llmspell_data:
```

Start: `docker-compose up -d`

### Native Installation (macOS/Linux)

**macOS** (Homebrew):

```bash
# 1. Install PostgreSQL 18
brew install postgresql@18

# 2. Install VectorChord extension
git clone https://github.com/tensorchord/vchord-postgres.git
cd vchord-postgres
USE_PGXS=1 make
USE_PGXS=1 make install

# 3. Start PostgreSQL
brew services start postgresql@18

# 4. Create database
createdb -U $USER llmspell_dev
```

**Ubuntu/Debian**:

```bash
# 1. Add PostgreSQL APT repository
sudo apt-get install -y postgresql-common
sudo /usr/share/postgresql-common/pgdg/apt.postgresql.org.sh

# 2. Install PostgreSQL 18
sudo apt-get update
sudo apt-get install -y postgresql-18 postgresql-server-dev-18

# 3. Install VectorChord extension
git clone https://github.com/tensorchord/vchord-postgres.git
cd vchord-postgres
make
sudo make install

# 4. Create database
sudo -u postgres createdb llmspell_dev
```

### Database Initialization

**Create Application Role** (non-superuser for RLS):

```sql
-- Connect as superuser
psql postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev

-- Create non-superuser application role
CREATE ROLE llmspell_app WITH LOGIN PASSWORD 'llmspell_app_pass';

-- Create schema
CREATE SCHEMA IF NOT EXISTS llmspell;

-- Grant permissions
GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA llmspell TO llmspell_app;
ALTER DEFAULT PRIVILEGES IN SCHEMA llmspell
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO llmspell_app;

-- Enable extensions
CREATE EXTENSION IF NOT EXISTS vchord;
CREATE EXTENSION IF NOT EXISTS vector;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS pgcrypto;
```

**Run Migrations**:

Migrations run automatically on `PostgresBackend::new()`. To run manually:

```bash
# Using llmspell CLI (auto-migration on first run)
llmspell storage validate --backend postgres

# Or manually via psql (if needed)
psql postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_dev \
  -f llmspell-storage/src/backends/postgres/migrations/V001__initial_setup.sql
# ... (repeat for V002-V015)
```

### Connection Pool Configuration

**Formula**: `pool_size = (CPU × 2) + 1`

**Examples**:
- **8-core server**: `(8 × 2) + 1 = 17` → round to **20**
- **16-core server**: `(16 × 2) + 1 = 33` → round to **35**

**rs-llmspell Configuration** (`~/.config/llmspell/config.toml`):

```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_prod"

# Connection pool settings
pool_size = 20              # Max connections (CPU × 2 + 1)
pool_timeout_secs = 30      # Timeout acquiring connection
idle_timeout_secs = 600     # Close idle connections (10 min)
max_lifetime_secs = 1800    # Recycle connections (30 min)

# RLS enforcement
enforce_tenant_isolation = true  # Enable RLS

# Migrations
auto_migrate = false  # Run migrations separately in production
```

**PostgreSQL Settings** (`postgresql.conf`):

```conf
max_connections = 100
# Formula: pool_size × instances + admin_reserve + margin
# Example: 20 × 3 + 10 + 30 = 100

shared_buffers = 4GB           # 25% of RAM (for 16 GB server)
work_mem = 64MB                # Per-query sort/hash memory
maintenance_work_mem = 512MB   # For VACUUM, CREATE INDEX

wal_buffers = 16MB
min_wal_size = 1GB
max_wal_size = 4GB

effective_cache_size = 12GB    # 75% of RAM
random_page_cost = 1.1         # SSD value (HDD: 4.0)
effective_io_concurrency = 200 # SSD parallel I/O
```

### Rust Integration

**Basic Usage**:

```rust
use llmspell_storage::{PostgresBackend, PostgresConfig};
use llmspell_core::traits::storage::StorageBackend;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create configuration
    let config = PostgresConfig::new(
        "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_prod"
    )
    .with_pool_size(20)
    .with_rls_enabled(true);

    // 2. Initialize backend (runs migrations if auto_migrate = true)
    let backend = PostgresBackend::new(config).await?;

    // 3. Set tenant context (required for RLS)
    backend.set_tenant_context("tenant-123").await?;

    // 4. Use storage
    backend.set("user:42", b"Alice".to_vec()).await?;
    let value = backend.get("user:42").await?;

    println!("User: {}", String::from_utf8(value.unwrap())?);

    Ok(())
}
```

**With TenantScoped Trait**:

```rust
use llmspell_core::{TenantScoped, StateScope};

async fn process_tenant_data(backend: &dyn TenantScoped) -> Result<()> {
    // Set tenant context
    backend.set_tenant_context("tenant-xyz".to_string(), StateScope::Global).await?;

    // Get current tenant
    let tenant = backend.tenant_id().await;
    println!("Processing data for: {:?}", tenant);

    Ok(())
}
```

---

## Schema Reference

### 15 Tables Summary

```text
llmspell (schema)
├── Extensions
│   ├── vchord 0.5.3 (Vector similarity search)
│   ├── vector 0.8.1 (pgvector dependency)
│   ├── uuid-ossp (UUID generation)
│   └── pgcrypto (Encryption functions)
│
├── Vector Storage (4 tables)
│   ├── vector_embeddings_384   (384-dim: All-MiniLM, small models)
│   ├── vector_embeddings_768   (768-dim: sentence-transformers, BGE)
│   ├── vector_embeddings_1536  (1536-dim: OpenAI text-embedding-3-small)
│   └── vector_embeddings_3072  (3072-dim: OpenAI text-embedding-3-large)
│
├── Temporal Graph (2 tables)
│   ├── entities (Bi-temporal nodes with versioning)
│   └── relationships (Bi-temporal edges with versioning)
│
├── State Storage (3 tables)
│   ├── procedural_patterns (Learned state transition patterns)
│   ├── agent_states (Persistent agent state with versioning)
│   └── workflow_states (Workflow execution lifecycle)
│
├── Session Management (3 tables)
│   ├── sessions (Session snapshots with expiration)
│   ├── artifact_content (Content-addressed storage with deduplication)
│   └── artifacts (Artifact metadata and references)
│
├── Event Sourcing (2 tables)
│   ├── event_log (Partitioned event log, monthly partitions)
│   └── hook_history (Hook execution history with compression)
│
├── Security (1 table)
│   └── api_keys (Encrypted API keys with pgcrypto)
│
└── Testing (1 table)
    └── test_table (RLS testing infrastructure)
```

### Migration History

| Version | Name | Lines | Purpose |
|---------|------|-------|---------|
| **V1** | `initial_setup` | 38 | Extensions (vchord, pgcrypto, uuid-ossp), llmspell schema |
| **V2** | `test_table_rls` | 51 | RLS testing infrastructure with test_table |
| **V3** | `vector_embeddings` | 215 | Vector storage (4 dimension tables: 384, 768, 1536, 3072) |
| **V4** | `temporal_graph` | 225 | Bi-temporal entities + relationships |
| **V5** | `procedural_memory` | 106 | Procedural patterns table |
| **V6** | `agent_state` | 138 | Agent state with versioning + integrity checks |
| **V7** | `kv_store` | 105 | Key-value store (deprecated, kept for compatibility) |
| **V8** | `workflow_states` | 159 | Workflow execution lifecycle tracking |
| **V9** | `sessions` | 157 | Session snapshots with expiration |
| **V10** | `artifacts` | 322 | Content-addressed artifacts with deduplication |
| **V11** | `event_log` | 256 | Partitioned event log (monthly range partitions) |
| **V12** | `application_role_rls_enforcement` | 151 | llmspell_app role + RLS enforcement |
| **V13** | `hook_history` | 200 | Hook execution history with compression |
| **V14** | `api_keys` | 219 | Encrypted API keys with pgcrypto |
| **V15** | `bitemporal_composite_keys` | 92 | Fix bi-temporal primary keys (composite keys) |
| **TOTAL** | | **2,434** | |

### Vector Embeddings Tables

**Tables**: `vector_embeddings_{384,768,1536,3072}`

**Schema** (384-dimensional example):

```sql
CREATE TABLE llmspell.vector_embeddings_384 (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(255) NOT NULL,
    embedding VECTOR(384) NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- HNSW vector similarity index
CREATE INDEX idx_vector_384_hnsw ON vector_embeddings_384
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);

-- B-tree indexes for filtering
CREATE INDEX idx_vector_384_tenant ON vector_embeddings_384(tenant_id);
CREATE INDEX idx_vector_384_scope ON vector_embeddings_384(scope);
CREATE INDEX idx_vector_384_created ON vector_embeddings_384(created_at);
```

**Dimension Tables**:

| Table | Dimensions | Use Case | HNSW Parameters |
|-------|------------|----------|-----------------|
| `vector_embeddings_384` | 384 | All-MiniLM, small models | `m=16, ef_construction=64` |
| `vector_embeddings_768` | 768 | sentence-transformers, BGE | `m=16, ef_construction=128` |
| `vector_embeddings_1536` | 1536 | OpenAI text-embedding-3-small | `m=24, ef_construction=256` |
| `vector_embeddings_3072` | 3072 | OpenAI text-embedding-3-large | **No HNSW** (pgvector 2000-dim limit) |

**Note on 3072 dimensions**: VectorChord/pgvector have a 2000-dimension limit for HNSW/IVFFlat indexes. For 3072-dimensional vectors:
- Linear scan for small datasets (<10K vectors)
- External vector DB (Qdrant, Milvus) for large-scale search
- Dimension reduction (PCA, UMAP) if needed

### Temporal Graph Tables

**Tables**: `entities`, `relationships`

**Entities Schema**:

```sql
CREATE TABLE llmspell.entities (
    entity_id UUID NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(255) NOT NULL,
    name VARCHAR(500) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',

    -- Bi-temporal timestamps
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (entity_id, transaction_time_start),

    CONSTRAINT valid_time_range CHECK (valid_time_start < valid_time_end),
    CONSTRAINT tx_time_range CHECK (transaction_time_start < transaction_time_end)
);

-- Indexes
CREATE INDEX idx_entities_tenant ON entities(tenant_id);
CREATE INDEX idx_entities_type ON entities(entity_type);
CREATE INDEX idx_entities_name ON entities(name);
CREATE INDEX idx_entities_id_lookup ON entities(entity_id);

-- GiST indexes for temporal range queries
CREATE INDEX idx_entities_valid_time ON entities
    USING GIST (tstzrange(valid_time_start, valid_time_end));

CREATE INDEX idx_entities_tx_time ON entities
    USING GIST (tstzrange(transaction_time_start, transaction_time_end));

-- GIN index for JSONB property queries
CREATE INDEX idx_entities_properties ON entities USING GIN(properties);
```

**Bi-Temporal Semantics**:
- **Valid Time**: When data was true in the real world
  - `valid_time_start`: Entity became valid
  - `valid_time_end`: Entity became invalid (or 'infinity' if still valid)
- **Transaction Time**: When data was recorded in the database
  - `transaction_time_start`: Record created
  - `transaction_time_end`: Record superseded (or 'infinity' if current version)

**Primary Key**: `(entity_id, transaction_time_start)` allows multiple temporal versions

**Query Patterns**:

```sql
-- Current version of all entities (transaction_time_end = 'infinity')
SELECT * FROM entities WHERE transaction_time_end = 'infinity';

-- Currently valid entities (valid_time_end = 'infinity')
SELECT * FROM entities WHERE valid_time_end = 'infinity';

-- Point-in-time query (what was known at 2025-01-01?)
SELECT * FROM entities
WHERE valid_time_start <= '2025-01-01'
  AND valid_time_end > '2025-01-01'
  AND transaction_time_start <= '2025-01-01'
  AND transaction_time_end > '2025-01-01';
```

### State Storage Tables

**Procedural Patterns**:

```sql
CREATE TABLE llmspell.procedural_patterns (
    pattern_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    scope VARCHAR(500) NOT NULL,
    key VARCHAR(500) NOT NULL,
    value TEXT NOT NULL,
    frequency INTEGER NOT NULL DEFAULT 1,
    first_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT unique_procedural_pattern UNIQUE (tenant_id, scope, key, value),
    CONSTRAINT positive_frequency CHECK (frequency > 0)
);

-- Partial index for learned patterns (frequency >= 3)
CREATE INDEX idx_procedural_patterns_lookup ON procedural_patterns(tenant_id, scope, key, value)
    WHERE frequency >= 3;
```

**Agent State**:

```sql
CREATE TABLE llmspell.agent_states (
    state_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL,
    agent_id VARCHAR(255) NOT NULL,
    agent_type VARCHAR(100) NOT NULL,
    state_data JSONB NOT NULL,
    schema_version INTEGER NOT NULL DEFAULT 1,
    data_version INTEGER NOT NULL DEFAULT 1,
    checksum VARCHAR(64) NOT NULL, -- SHA-256 of state_data
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT unique_agent_state UNIQUE (tenant_id, agent_id)
);
```

**Workflow State**:

```sql
CREATE TABLE llmspell.workflow_states (
    tenant_id VARCHAR(255) NOT NULL,
    workflow_id VARCHAR(255) NOT NULL,
    workflow_name VARCHAR(500),
    state_data JSONB NOT NULL,
    current_step INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, workflow_id),

    CONSTRAINT valid_workflow_status CHECK (
        status IN ('pending', 'running', 'completed', 'failed', 'cancelled')
    )
);
```

### Session & Artifacts Tables

**Sessions**:

```sql
CREATE TABLE llmspell.sessions (
    tenant_id VARCHAR(255) NOT NULL,
    session_id UUID NOT NULL,
    session_data JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ,
    artifact_count INTEGER NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, session_id),

    CONSTRAINT valid_session_status CHECK (
        status IN ('active', 'archived', 'expired')
    )
);
```

**Artifacts** (Content-Addressed Storage):

```sql
CREATE TABLE llmspell.artifact_content (
    tenant_id VARCHAR(255) NOT NULL,
    content_hash VARCHAR(64) NOT NULL, -- blake3 hash
    storage_type VARCHAR(20) NOT NULL,
    data BYTEA, -- <1MB
    large_object_oid OID, -- >=1MB
    size_bytes BIGINT NOT NULL,
    is_compressed BOOLEAN NOT NULL DEFAULT false,
    original_size_bytes BIGINT,
    reference_count INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (tenant_id, content_hash),

    CONSTRAINT valid_storage_type CHECK (storage_type IN ('bytea', 'large_object'))
);

CREATE TABLE llmspell.artifacts (
    tenant_id VARCHAR(255) NOT NULL,
    artifact_id VARCHAR(512) NOT NULL,
    session_id UUID NOT NULL,
    sequence BIGINT NOT NULL,
    content_hash VARCHAR(64) NOT NULL,
    metadata JSONB NOT NULL,
    name VARCHAR(255) NOT NULL,
    artifact_type VARCHAR(50) NOT NULL,
    mime_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    version INTEGER NOT NULL DEFAULT 1,
    tags TEXT[],

    PRIMARY KEY (tenant_id, artifact_id),

    FOREIGN KEY (tenant_id, content_hash)
        REFERENCES artifact_content(tenant_id, content_hash)
        ON DELETE RESTRICT
);
```

### Event Log (Partitioned)

```sql
CREATE TABLE llmspell.event_log (
    tenant_id VARCHAR(255) NOT NULL,
    event_id UUID NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    correlation_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    sequence BIGINT NOT NULL,
    language VARCHAR(50) NOT NULL,
    payload JSONB NOT NULL,

    PRIMARY KEY (tenant_id, timestamp, event_id)
) PARTITION BY RANGE (timestamp);

-- Monthly partitions (example for January 2025)
CREATE TABLE llmspell.event_log_2025_01
    PARTITION OF llmspell.event_log
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

-- Partition management function
SELECT llmspell.ensure_future_event_log_partitions();
```

### Entity-Relationship Diagram

```text
┌───────────────────────────────────────────────────────────────────────────┐
│                       VECTOR STORAGE (4 tables)                            │
├──────────────────────┬──────────────────────┬──────────────────────┬──────┤
│vector_embeddings_384 │vector_embeddings_768 │vector_embeddings_1536│ 3072 │
│  id (UUID)           │  id (UUID)           │  id (UUID)           │ ...  │
│  tenant_id           │  tenant_id           │  tenant_id           │      │
│  scope               │  scope               │  scope               │      │
│  embedding VECTOR    │  embedding VECTOR    │  embedding VECTOR    │      │
│  metadata JSONB      │  metadata JSONB      │  metadata JSONB      │      │
└──────────────────────┴──────────────────────┴──────────────────────┴──────┘
  Indexes: HNSW (cosine similarity), B-tree (tenant, scope, created_at)

┌───────────────────────────────────────────────────────────────────────────┐
│                       TEMPORAL GRAPH (2 tables)                            │
├──────────────────────────────────┬────────────────────────────────────────┤
│ entities                         │ relationships                          │
│  entity_id UUID                  │  relationship_id UUID                  │
│  tenant_id                       │  tenant_id                             │
│  entity_type                     │  from_entity UUID                      │
│  name                            │  to_entity UUID                        │
│  properties JSONB                │  relationship_type                     │
│  valid_time_start                │  properties JSONB                      │
│  valid_time_end                  │  valid_time_start/end                  │
│  transaction_time_start          │  transaction_time_start/end            │
│  transaction_time_end            │  PK: (rel_id, tx_time_start)           │
│  PK: (entity_id, tx_time_start)  │                                        │
└──────────────────────────────────┴────────────────────────────────────────┘
  Indexes: GiST (temporal ranges), GIN (properties), B-tree (tenant, type)

┌───────────────────────────────────────────────────────────────────────────┐
│                   SESSION MANAGEMENT (3 tables)                            │
├──────────────────┬────────────────────────┬─────────────────────────────┤
│ sessions         │ artifact_content       │ artifacts                   │
│  session_id UUID │  content_hash (blake3) │  artifact_id                │
│  tenant_id       │  tenant_id             │  session_id UUID            │
│  session_data    │  storage_type (enum)   │  tenant_id                  │
│  status (enum)   │  data BYTEA (<1MB)     │  sequence                   │
│  created_at      │  large_object_oid      │  content_hash               │
│  last_accessed   │  size_bytes            │  metadata JSONB             │
│  expires_at      │  reference_count       │  name, type, mime           │
│  artifact_count  │  created_at            │  version                    │
│  PK: (tenant,    │  last_accessed_at      │  tags TEXT[]                │
│    session_id)   │  PK: (tenant, hash)    │  FK: (tenant, hash)         │
└──────────────────┴────────────────────────┴─────────────────────────────┘
  Indexes: B-tree (tenant, session, status), GIN (session_data, metadata)
  Foreign Keys: artifacts -> sessions (CASCADE), artifacts -> content (RESTRICT)

┌───────────────────────────────────────────────────────────────────────────┐
│                     EVENT SOURCING (2 tables)                              │
├──────────────────────────────────┬────────────────────────────────────────┤
│ event_log (PARTITIONED)          │ hook_history                           │
│  event_id UUID                   │  execution_id UUID                     │
│  tenant_id                       │  tenant_id                             │
│  event_type                      │  hook_id                               │
│  correlation_id UUID             │  hook_type                             │
│  timestamp                       │  correlation_id UUID                   │
│  sequence                        │  hook_context BYTEA (compressed)       │
│  language                        │  result_data JSONB                     │
│  payload JSONB                   │  timestamp                             │
│  PK: (tenant, timestamp, id)     │  duration_ms                           │
│  PARTITION BY RANGE (timestamp)  │  triggering_component                  │
│  Partitions: event_log_YYYY_MM   │  tags TEXT[], retention_priority       │
└──────────────────────────────────┴────────────────────────────────────────┘
  Indexes: B-tree (tenant, time, correlation, type), GIN (payload, metadata)
  Partitioning: Monthly ranges, auto-managed via ensure_future_partitions()
```

---

## Security & Multi-Tenancy

### Row-Level Security (RLS) Architecture

**All 15 tables use identical RLS pattern for tenant isolation:**

```sql
-- Enable RLS on table
ALTER TABLE llmspell.{table} ENABLE ROW LEVEL SECURITY;
ALTER TABLE llmspell.{table} FORCE ROW LEVEL SECURITY; -- Enforces RLS even for table owner

-- SELECT policy: Only see rows for current tenant
CREATE POLICY tenant_isolation_select ON llmspell.{table}
    FOR SELECT
    USING (tenant_id = current_setting('app.current_tenant_id', true));

-- INSERT policy: Can only insert rows for current tenant
CREATE POLICY tenant_isolation_insert ON llmspell.{table}
    FOR INSERT
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- UPDATE policy: Can only update rows for current tenant
CREATE POLICY tenant_isolation_update ON llmspell.{table}
    FOR UPDATE
    USING (tenant_id = current_setting('app.current_tenant_id', true))
    WITH CHECK (tenant_id = current_setting('app.current_tenant_id', true));

-- DELETE policy: Can only delete rows for current tenant
CREATE POLICY tenant_isolation_delete ON llmspell.{table}
    FOR DELETE
    USING (tenant_id = current_setting('app.current_tenant_id', true));
```

### RLS Performance

**Measured Overhead**: 4.9% (Phase 13b.3 validation)

**Benchmark**:
```
Without RLS:  1000 queries in 245ms (4,082 qps)
With RLS:     1000 queries in 257ms (3,892 qps)
Overhead:     12ms (4.9%)
```

**Validation**: ✅ <5% target achieved

### Setting Tenant Context

**Rust**:

```rust
use llmspell_storage::PostgresBackend;

let backend = PostgresBackend::new(config).await?;

// Set tenant context (required before queries)
backend.set_tenant_context("tenant-123").await?;

// Get pooled client with tenant context applied
let client = backend.get_client().await?;

// Query automatically filtered by RLS
let rows = client.query(
    "SELECT * FROM llmspell.vector_embeddings_768 WHERE dimension = $1",
    &[&768]
).await?;
// Only returns rows where tenant_id = 'tenant-123'
```

**SQL**:

```sql
-- Set tenant context for session
SET LOCAL app.current_tenant_id = 'tenant-123';

-- All subsequent queries filtered by RLS
SELECT * FROM llmspell.vector_embeddings_768;
-- Only returns rows where tenant_id = 'tenant-123'
```

### Security Best Practices

**1. Never Use Superuser Connections**

RLS policies do not apply to superuser roles. Always use non-superuser application role:

```sql
-- Create non-superuser application role
CREATE ROLE llmspell_app WITH LOGIN PASSWORD 'secure_password';

-- Grant minimal privileges
GRANT USAGE ON SCHEMA llmspell TO llmspell_app;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA llmspell TO llmspell_app;
```

**2. Always Set Tenant Context Before Queries**

Unset context blocks all RLS access (returns 0 rows):

```rust
// ✅ CORRECT: Set tenant context first
backend.set_tenant_context("tenant-123").await?;
let client = backend.get_client().await?;
let rows = client.query("SELECT * FROM sessions", &[]).await?;

// ❌ WRONG: Unset context returns 0 rows
let client = backend.get_client().await?;
let rows = client.query("SELECT * FROM sessions", &[]).await?; // Returns 0 rows
```

**3. Validate Tenant IDs**

Use parameterized queries to prevent SQL injection:

```rust
// ✅ SAFE: Parameterized query
client.execute(
    "SELECT set_config('app.current_tenant_id', $1, false)",
    &[&tenant_id]
).await?;

// ❌ UNSAFE: String concatenation (SQL injection risk)
client.execute(
    &format!("SELECT set_config('app.current_tenant_id', '{}', false)", tenant_id),
    &[]
).await?;
```

**4. Cross-Tenant Access Prevention**

RLS prevents cross-tenant access even with SQL injection:

```sql
-- Malicious query attempt
SELECT * FROM sessions WHERE tenant_id = 'other_tenant';

-- RLS enforces AND clause:
SELECT * FROM sessions
WHERE tenant_id = 'other_tenant'
  AND tenant_id = current_setting('app.current_tenant_id', true);
-- Result: 0 rows (cannot see other tenant's data)
```

**Security guarantee**: RLS is enforced at PostgreSQL kernel level, not application layer.

### Troubleshooting RLS

**Issue: No Rows Returned Despite Data Existing**

**Diagnosis**:

```rust
// Check current tenant context
let tenant = backend.get_tenant_context().await;
println!("Current tenant: {:?}", tenant);

// Verify session variable in database
let client = backend.get_client().await?;
let row = client.query_one(
    "SELECT current_setting('app.current_tenant_id', true) AS tenant",
    &[]
).await?;
let db_tenant: String = row.get("tenant");
println!("Database tenant: {}", db_tenant);
```

**Fix**: Call `set_tenant_context()` before querying.

**Issue: RLS Policies Not Applied (All Tenants Visible)**

**Diagnosis**:

```sql
-- Check current role
SELECT current_user, usesuper FROM pg_user WHERE usename = current_user;
-- usesuper = true means RLS bypassed
```

**Fix**: Use non-superuser application role (`llmspell_app`).

---

## Performance Optimization

### Performance Targets

| Operation | p50 | p95 | p99 | Validated |
|-----------|-----|-----|-----|-----------|
| **Vector insert** | 0.5ms | 2ms | 5ms | ✅ Phase 13b.4 |
| **Vector search (k=10)** | 1ms | 5ms | 15ms | ✅ Phase 13b.4 |
| **Entity/relationship insert** | 0.8ms | 3ms | 8ms | ✅ Phase 13b.5 |
| **Temporal point-in-time query** | 2ms | 10ms | 25ms | ✅ Phase 13b.5 |
| **Session CRUD** | 0.5ms | 2ms | 5ms | ✅ Phase 13b.9 |
| **Artifact storage** | 1ms | 5ms | 15ms | ✅ Phase 13b.10 |
| **Event log append** | 0.1ms | 1ms | 3ms | ✅ Phase 13b.11 |
| **RLS policy overhead** | N/A | <5% | <5% | ✅ 4.9% measured |

### VectorChord HNSW Tuning

**3 Critical Parameters**:

```sql
CREATE INDEX idx_vector_768_hnsw ON vector_embeddings_768
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 128);

-- Query-time parameter (set per connection)
SET hnsw.ef_search = 40;
```

**Parameter 1: `m` (Graph Connectivity)**

**Trade-offs**:
- **Higher m**: Better recall, slower inserts, more memory
- **Lower m**: Faster inserts, less memory, lower recall

**Recommended Values**:

| Dimension | m | Rationale |
|-----------|---|-----------|
| **384** | 8-12 | Lower dimensions need less connectivity |
| **768** | 16 | Standard value (Phase 13b validated) |
| **1536** | 24 | Higher dimensions need more connectivity |
| **3072** | N/A | No HNSW support (pgvector 2000-dim limit) |

**Memory Formula**:
```
Memory per vector ≈ (m × 2) × 4 bytes + dimension × 4 bytes
```

**Examples**:
- 768-dim, m=16: `(16 × 2) × 4 + 768 × 4 = 3,200 bytes/vector`
- 10K vectors: `10,000 × 3,200 = 32 MB`
- 1M vectors: `1,000,000 × 3,200 = 3.2 GB`

**Phase 13b Results**:
- **m=12**: 92% recall, 15 MB index (10K vectors)
- **m=16**: 96% recall, 25 MB index (**selected**)
- **m=24**: 98% recall, 45 MB index (diminishing returns)

**Parameter 2: `ef_construction` (Build-Time Search Depth)**

**Trade-offs**:
- **Higher ef_construction**: Better index quality, slower builds
- **Lower ef_construction**: Faster builds, lower quality index

**Recommended Values**:

| Use Case | ef_construction | Build Time (10K vectors) |
|----------|-----------------|--------------------------|
| **Development/testing** | 64 | ~5 seconds |
| **Production (384/768-dim)** | 128 | ~15 seconds |
| **Production (1536-dim)** | 256 | ~45 seconds |
| **High accuracy critical** | 400 | ~2 minutes |

**Rule of Thumb**: `ef_construction = 2 × m` (minimum), `4 × m` (recommended)

**Parameter 3: `ef_search` (Query-Time Search Depth)**

**Trade-offs**:
- **Higher ef_search**: Better recall, slower searches
- **Lower ef_search**: Faster searches, lower recall

**Recommended Values**:

| Recall Target | ef_search | Latency (p95) |
|---------------|-----------|---------------|
| **90%** | 20 | 3ms |
| **95%** | 40 | 5ms (**default**) |
| **98%** | 100 | 12ms |
| **99%+** | 200 | 30ms |

**Setting ef_search**:

```rust
// Rust: Set per connection
conn.execute("SET hnsw.ef_search = 40", &[]).await?;

// SQL: Set globally
ALTER DATABASE llmspell_prod SET hnsw.ef_search = 40;
```

### RLS Performance Optimization

**Strategy 1: Composite Indexes**

Always put `tenant_id` as first column in multi-column indexes:

```sql
-- Good (composite index with tenant_id first)
CREATE INDEX idx_sessions_tenant_status ON sessions(tenant_id, status);

-- Query plan:
-- Single index scan filters both tenant_id (RLS) and status (app query)
```

**Strategy 2: Partial Indexes**

```sql
-- Full index (large)
CREATE INDEX idx_sessions_expires ON sessions(expires_at);
-- Size: 5 MB (100K sessions)

-- Partial index (small, faster)
CREATE INDEX idx_sessions_expires ON sessions(expires_at)
    WHERE expires_at IS NOT NULL;
-- Size: 500 KB (only 10K sessions with expiration)
-- Performance: 10x faster for expiration queries
```

### Event Log Partitioning

**Monthly Partitioning Strategy**:

```sql
-- Parent table (partitioned)
CREATE TABLE event_log (
    tenant_id VARCHAR(255),
    event_id UUID,
    timestamp TIMESTAMPTZ,
    ...
    PRIMARY KEY (tenant_id, timestamp, event_id)
) PARTITION BY RANGE (timestamp);

-- Child partitions
CREATE TABLE event_log_2025_01 PARTITION OF event_log
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

**Partition Pruning** (Query Optimization):

```sql
-- Without partition pruning (slow)
SELECT * FROM event_log
WHERE event_type = 'agent.state_changed';
-- Execution time: 500ms (scans 12 monthly partitions)

-- With partition pruning (fast)
SELECT * FROM event_log
WHERE timestamp >= '2025-01-01' AND timestamp < '2025-02-01'
  AND event_type = 'agent.state_changed';
-- Execution time: 40ms (scans 1 partition)
```

**Rule**: Always include `timestamp` range in event log queries

**Performance Benchmarks**:
```
Ingestion throughput:  10,000 events/sec (with partitioning)
Query performance:     12.5x speedup (time range queries)
```

### VACUUM Strategy

**Autovacuum Tuning**:

```sql
-- Aggressive settings for high-write tables
ALTER TABLE event_log SET (
    autovacuum_vacuum_scale_factor = 0.05,   -- Vacuum when 5% dead
    autovacuum_analyze_scale_factor = 0.02,  -- Analyze when 2% changed
    autovacuum_vacuum_cost_delay = 10,       -- Faster vacuuming
    autovacuum_vacuum_cost_limit = 1000      -- Higher I/O budget
);
```

**Manual VACUUM Schedule**:

```bash
#!/bin/bash
# /etc/cron.daily/llmspell-vacuum
psql $DATABASE_URL <<EOF
VACUUM (ANALYZE, VERBOSE) llmspell.event_log;
VACUUM (ANALYZE, VERBOSE) llmspell.sessions;
VACUUM (ANALYZE, VERBOSE) llmspell.artifacts;
EOF
```

### Query Optimization

**EXPLAIN ANALYZE Workflow**:

```sql
-- Step 1: EXPLAIN (no execution)
EXPLAIN SELECT * FROM sessions WHERE tenant_id = 'tenant-123' AND status = 'active';

-- Step 2: EXPLAIN ANALYZE (with execution + timing)
EXPLAIN (ANALYZE, BUFFERS) SELECT * FROM sessions WHERE tenant_id = 'tenant-123' AND status = 'active';

-- Look for:
-- - Sequential Scans (should be Index Scans)
-- - High execution time on specific nodes
-- - Buffers (shared hit = cache, read = disk I/O)
```

**Common Anti-Patterns**:

```sql
-- ❌ Bad: OFFSET for Pagination (slow for large offsets)
SELECT * FROM event_log ORDER BY timestamp DESC LIMIT 100 OFFSET 10000;
-- Must scan 10,100 rows, discard 10,000
-- Execution time: 500ms (offset 10K), 5s (offset 100K)

-- ✅ Good: Keyset Pagination
-- First page
SELECT * FROM event_log ORDER BY timestamp DESC LIMIT 100;

-- Next page (use last timestamp from previous page)
SELECT * FROM event_log
WHERE timestamp < $last_timestamp
ORDER BY timestamp DESC LIMIT 100;
-- Execution time: 5ms (constant, independent of page number)
```

### Monitoring Queries

```sql
-- Query performance (requires pg_stat_statements extension)
SELECT query, calls, mean_exec_time, max_exec_time
FROM pg_stat_statements
WHERE query LIKE '%llmspell%'
ORDER BY mean_exec_time * calls DESC
LIMIT 10;

-- Cache hit ratio (should be >95%)
SELECT
    sum(heap_blks_read) as heap_read,
    sum(heap_blks_hit)  as heap_hit,
    round(100.0 * sum(heap_blks_hit) / NULLIF(sum(heap_blks_hit) + sum(heap_blks_read), 0), 2) as cache_hit_ratio
FROM pg_statio_user_tables;

-- Index usage (unused indexes waste space)
SELECT schemaname, tablename, indexname, idx_scan
FROM pg_stat_user_indexes
WHERE schemaname = 'llmspell' AND idx_scan = 0
ORDER BY pg_relation_size(indexrelid) DESC;
```

---

## Operations & Migration

### Backup Strategy

**Automatic Backups** (recommended for production):

```bash
#!/bin/bash
# /etc/cron.daily/llmspell-backup

# Full database backup
pg_dump postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_prod \
  -F c -Z 9 \
  -f /backups/llmspell_prod_$(date +%Y%m%d_%H%M%S).dump

# Keep last 30 days
find /backups -name "llmspell_prod_*.dump" -mtime +30 -delete
```

**Schema-Only Backup**:

```bash
pg_dump -s postgresql://llmspell:llmspell_dev_pass@localhost:5432/llmspell_prod \
  > schema_backup_$(date +%Y%m%d).sql
```

**Partition Archival**:

```sql
-- Detach partition (makes it a standalone table)
ALTER TABLE event_log DETACH PARTITION event_log_2024_01;

-- Dump to file
pg_dump -t llmspell.event_log_2024_01 \
    -F c -f event_log_2024_01.dump \
    llmspell_prod

-- Drop from database
DROP TABLE llmspell.event_log_2024_01;
```

### Restore Procedures

**Full Restore**:

```bash
# Restore full database
pg_restore -d llmspell_prod \
  -c -C \
  /backups/llmspell_prod_20250115_020000.dump
```

**Restore Archived Partition**:

```sql
-- Restore table
pg_restore -t llmspell.event_log_2024_01 \
    -d llmspell_prod \
    event_log_2024_01.dump

-- Reattach as partition
ALTER TABLE event_log ATTACH PARTITION event_log_2024_01
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

### Data Migration (PostgreSQL ↔ SQLite) - Phase 13c.3.2

**Bidirectional Lossless Migration**

rs-llmspell provides built-in export/import tools for PostgreSQL ↔ SQLite migration with zero data loss.

**Key Features**:
- ✅ **Lossless roundtrip migration** (verified via integration tests)
- ✅ **All 10 data types** (V3-V11, V13: vectors, graph, patterns, agent/workflow/KV state, sessions, artifacts, events, hooks)
- ✅ **Transaction-safe import** with automatic rollback on errors
- ✅ **Versioned JSON format** for compatibility tracking
- ✅ **Base64 encoding** for binary data (BLOB/BYTEA) preservation

**Migration Commands**:

```bash
# Export from PostgreSQL
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"
llmspell storage export --backend postgres --output pg-export.json

# Export from SQLite
llmspell storage export --backend sqlite --output sqlite-export.json

# Import to PostgreSQL
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"
llmspell storage import --backend postgres --input sqlite-export.json

# Import to SQLite
llmspell storage import --backend sqlite --input pg-export.json
```

**Export Format**:

```json
{
  "version": "1.0",
  "exported_at": "2025-01-15T10:30:00Z",
  "source_backend": "postgresql",
  "migrations": ["V3", "V4", "V5", "V6", "V7", "V8", "V9", "V10", "V11", "V13"],
  "data": {
    "vector_embeddings": {
      "384": [...],
      "768": [...],
      "1536": [...],
      "3072": [...]
    },
    "knowledge_graph": {
      "entities": [...],
      "relationships": [...]
    },
    "procedural_memory": [...],
    "agent_state": [...],
    "kv_store": [...],
    "workflow_states": [...],
    "sessions": [...],
    "artifacts": [...],
    "event_log": [...],
    "hook_history": [...]
  }
}
```

**Migration Workflow 1: SQLite → PostgreSQL (Development to Production)**

```bash
# Step 1: Export from SQLite (development)
cd /path/to/dev/environment
llmspell storage export --backend sqlite --output dev-data.json

# Step 2: Validate export
jq '.version, .source_backend, (.data | keys)' dev-data.json
# Output:
# "1.0"
# "sqlite"
# ["agent_state", "artifacts", "event_log", "hook_history", "kv_store", ...]

# Step 3: Transfer to production server
scp dev-data.json prod-server:/tmp/llmspell-import.json

# Step 4: Import to PostgreSQL (production)
ssh prod-server
export DATABASE_URL="postgresql://llmspell:pass@localhost:5432/llmspell_prod"
llmspell storage import --backend postgres --input /tmp/llmspell-import.json

# Expected output:
# ✅ Imported 1,234 total records:
#   - Vectors (384-dim): 500
#   - Vectors (768-dim): 300
#   - Entities: 100
#   - Relationships: 150
#   - Procedural patterns: 20
#   - Agent states: 5
#   - Workflow states: 8
#   - Sessions: 10
#   - Artifacts: 25
#   - Events: 100
#   - Hooks: 16

# Step 5: Verify migration (roundtrip validation)
llmspell storage export --backend postgres --output prod-verify.json
diff <(jq -S .data dev-data.json) <(jq -S .data prod-verify.json)
# No output = perfect match (zero data loss)
```

**Migration Workflow 2: PostgreSQL → SQLite (Production to Development)**

```bash
# Step 1: Export from PostgreSQL (production)
export DATABASE_URL="postgresql://llmspell:pass@localhost:5432/llmspell_prod"
llmspell storage export --backend postgres --output prod-data.json

# Step 2: Transfer to development machine
scp prod-server:/tmp/prod-data.json /path/to/dev/llmspell/

# Step 3: Import to SQLite (development)
cd /path/to/dev/environment
llmspell storage import --backend sqlite --input prod-data.json

# Step 4: Verify migration
llmspell storage export --backend sqlite --output dev-verify.json
diff <(jq -S .data prod-data.json) <(jq -S .data dev-verify.json)
```

**Performance Benchmarks** (Phase 13c.3.2):

| Operation | Dataset Size | PostgreSQL Time | SQLite Time |
|-----------|--------------|-----------------|-------------|
| **Export** | 10K vectors + 1K entities | ~8s | ~5s |
| **Import** | 10K vectors + 1K entities | ~10s | ~6s |
| **Roundtrip** | Full dataset | ~18s | ~11s |
| **Throughput** | Export | 1.4K records/sec | 2.2K records/sec |
| **Throughput** | Import | 1.1K records/sec | 1.8K records/sec |

**Common Migration Scenarios**:

**Scenario 1: Pre-Production Testing**
```bash
# Export production data
ssh prod-server "llmspell storage export --backend postgres --output /tmp/prod.json"
scp prod-server:/tmp/prod.json ./

# Import to staging PostgreSQL
export DATABASE_URL="postgresql://user:pass@staging-db/llmspell_staging"
llmspell storage import --backend postgres --input prod.json
```

**Scenario 2: Disaster Recovery**
```bash
# If PostgreSQL database is lost but JSON export exists
export DATABASE_URL="postgresql://user:pass@new-db-server/llmspell_prod"

# Initialize new PostgreSQL backend (runs migrations)
llmspell storage validate --backend postgres

# Restore from JSON export
llmspell storage import --backend postgres --input last-backup.json
```

**Scenario 3: Backend Switching (Scale Down)**
```bash
# Migrate from PostgreSQL to SQLite (e.g., single-tenant deployment)
export DATABASE_URL="postgresql://user:pass@localhost/llmspell_prod"
llmspell storage export --backend postgres --output pre-switch.json
llmspell storage import --backend sqlite --input pre-switch.json
```

**Troubleshooting Migration Issues**:

**Issue 1: Import Fails with "Dimension mismatch"**

**Symptom**: `Error: Vector dimension 768 does not match table dimension 384`

**Cause**: Exporting vector from one dimension table, importing to another

**Solution**: Import preserves dimension mapping automatically. Check export data:
```bash
jq '.data.vector_embeddings | keys' export.json
# Should show: ["384", "768", "1536", "3072"]
```

**Issue 2: Import Fails with "Transaction rolled back"**

**Symptom**: `Error: Import transaction rolled back due to constraint violation`

**Cause**: Data integrity constraint violation (e.g., duplicate primary keys)

**Diagnosis**:
```bash
# Check export for duplicate IDs
jq '.data.agent_state | group_by(.state_id) | map(select(length > 1))' export.json
```

**Solution**: Import is transaction-safe and rolls back automatically. Fix data in source backend and re-export.

**Issue 3: Export File Too Large (>1GB)**

**Symptom**: `Error: Ran out of disk space writing export file`

**Solution**:
```bash
# Use compression
llmspell storage export --backend postgres --output /tmp/export.json
gzip /tmp/export.json
# Transfer compressed file
scp /tmp/export.json.gz prod-server:/tmp/
ssh prod-server "gunzip /tmp/export.json.gz && llmspell storage import --backend postgres --input /tmp/export.json"
```

**Issue 4: Binary Data Corruption**

**Symptom**: Artifacts or hook context data differs after import

**Diagnosis**: Check base64 encoding in export
```bash
jq '.data.artifacts[0].content_data' export.json
# Should be base64 string, not raw binary
```

**Fix**: Export uses base64 encoding automatically. If corruption occurs, file was modified during transfer. Use `scp -p` to preserve file integrity.

**Rollback Procedure**:

Import is transaction-safe and automatically rolls back on errors. To manually rollback after successful import:

```bash
# PostgreSQL: Restore from backup
pg_restore -d llmspell_prod -c /backups/pre-migration.dump

# SQLite: Restore from backup
cp /backups/llmspell_pre_migration.db ~/.local/share/llmspell/llmspell.db
```

**Best Practices**:

1. **Always backup before migration**:
   ```bash
   # PostgreSQL
   pg_dump -F c llmspell_prod > pre-migration.dump

   # SQLite
   cp ~/.local/share/llmspell/llmspell.db llmspell_pre_migration.db
   ```

2. **Verify roundtrip integrity**:
   ```bash
   # Export → Import → Export → Compare
   diff <(jq -S .data export1.json) <(jq -S .data export2.json)
   ```

3. **Use compression for large datasets**:
   ```bash
   gzip export.json  # 10:1 compression ratio typical
   ```

4. **Monitor import progress**: Import outputs per-table statistics
   ```bash
   llmspell storage import --backend postgres --input large.json
   # Importing vectors (384-dim): 10000/10000 ✅
   # Importing vectors (768-dim): 5000/5000 ✅
   # ...
   ```

5. **Test on non-production first**: Always validate migration on staging before production

**See Also**:
- [Storage Migration Internals](storage-migration-internals.md) - Technical deep dive
- [User Guide: Data Migration](../user-guide/11-data-migration.md) - Complete user guide
- [Developer Guide: Storage Backends](../developer-guide/reference/storage-backends.md) - Export/Import API

### Maintenance Tasks

**Weekly Tasks**:

```sql
-- Reindex all tables (during low-traffic window)
REINDEX DATABASE llmspell_prod CONCURRENTLY;

-- Update statistics
ANALYZE;

-- Check for bloat
SELECT schemaname, tablename,
       pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS total_size
FROM pg_tables
WHERE schemaname = 'llmspell'
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC;
```

**Monthly Tasks**:

```sql
-- Ensure future partitions exist
SELECT llmspell.ensure_future_event_log_partitions();

-- Archive old partitions (>90 days)
-- (see Partition Archival section above)

-- Review slow queries
SELECT query, mean_exec_time, calls
FROM pg_stat_statements
WHERE mean_exec_time > 100  -- >100ms average
ORDER BY mean_exec_time DESC
LIMIT 20;
```

### Troubleshooting

**Issue: Connection Pool Exhaustion**

**Symptom**: `Timeout waiting for connection from pool`

**Diagnosis**:

```sql
-- Current connection count
SELECT count(*) AS active_connections,
       max_connections
FROM pg_stat_activity,
     (SELECT setting::int AS max_connections FROM pg_settings WHERE name = 'max_connections') s
WHERE datname = 'llmspell_prod';

-- Long-running queries (may be blocking pool)
SELECT pid, now() - query_start AS duration, query
FROM pg_stat_activity
WHERE state != 'idle'
  AND now() - query_start > INTERVAL '5 seconds'
ORDER BY duration DESC;
```

**Solution**:
- Increase pool_size: Formula = `(CPU cores × 2) + 1`
- Kill long-running queries: `SELECT pg_terminate_backend(pid);`
- Enable connection metrics: `RUST_LOG=sqlx=debug`

**Issue: Slow Queries**

**Symptom**: Queries taking >1 second

**Diagnosis**:

```sql
-- Check for missing indexes
EXPLAIN (ANALYZE, BUFFERS) <your_slow_query>;
-- Look for "Seq Scan" (should be "Index Scan")
```

**Solution**:

```sql
-- Create missing indexes
CREATE INDEX CONCURRENTLY idx_table_column ON llmspell.table(column);

-- Rebuild existing indexes
REINDEX INDEX CONCURRENTLY idx_table_column;

-- Update statistics
ANALYZE llmspell.table;
```

**Issue: Disk Space Exhaustion**

**Diagnosis**:

```sql
-- Check database size
SELECT pg_size_pretty(pg_database_size('llmspell_prod'));

-- Check table sizes
SELECT schemaname, tablename,
       pg_size_pretty(pg_total_relation_size(schemaname || '.' || tablename)) AS size
FROM pg_tables
WHERE schemaname = 'llmspell'
ORDER BY pg_total_relation_size(schemaname || '.' || tablename) DESC
LIMIT 10;
```

**Solution**:
- Archive old partitions (event_log older than 90 days)
- VACUUM FULL bloated tables (during maintenance window)
- Delete orphaned artifact_content (reference_count = 0)

---

## References

### Documentation Links

- **Phase 13b Design**: `/docs/in-progress/phase-13-design-doc.md` - Memory system architecture
- **Current Architecture**: `/docs/technical/current-architecture.md` - System overview
- **Storage Architecture**: `/docs/technical/storage-architecture.md` - 3-tier storage design
- **Migration Guide**: `/docs/technical/migration-internals.md` - SQLite → PostgreSQL migration

### PostgreSQL Documentation

- [Row Security Policies](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
- [CREATE POLICY](https://www.postgresql.org/docs/current/sql-createpolicy.html)
- [Table Partitioning](https://www.postgresql.org/docs/current/ddl-partitioning.html)
- [VectorChord Extension](https://github.com/tensorchord/vchord-postgres)

### Code References

| Component | Location | Purpose |
|-----------|----------|---------|
| PostgresBackend | `llmspell-storage/src/backends/postgres/backend.rs` | Main backend implementation |
| RLS Helpers | `llmspell-storage/src/backends/postgres/rls.rs` | RLS policy generation |
| Migrations | `llmspell-storage/src/backends/postgres/migrations/` | 15 SQL migrations |
| Migration Engine | `llmspell-storage/src/migration/engine.rs` | Data migration orchestration |

---

**🔗 See Also**:
- [Current Architecture](current-architecture.md) - System design overview
- [Storage Architecture](storage-architecture.md) - 3-tier backend abstraction
- [Developer Guide: Tracing & Debugging](../developer-guide/06-tracing-debugging.md) - Debugging PostgreSQL queries

**Document Version**: 1.0
**Last Updated**: January 2025
**Phase**: 13b.20.1 - PostgreSQL Documentation Consolidation
