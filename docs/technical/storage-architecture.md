# Storage Architecture

**Version**: 0.13.0
**Phase**: 13b (Experimental Memory & Context Engineering)
**Last Updated**: January 2025

> **🎯 Purpose**: Document the unified storage abstraction, backend architecture, and 10 component-specific storage systems

**🔗 Navigation**: [← Technical Docs](README.md) | [Kernel Execution Paths →](kernel-execution-paths.md)

---

## Table of Contents

1. [Overview](#overview)
2. [Storage Backend Trait](#storage-backend-trait)
3. [Backend Implementations](#backend-implementations)
4. [10 Storage Components](#10-storage-components)
5. [Hot-Swap Architecture](#hot-swap-architecture)
6. [Performance Characteristics](#performance-characteristics)
7. [Migration Strategies](#migration-strategies)
8. [Code References](#code-references)
9. [Usage Examples](#usage-examples)
10. [Troubleshooting](#troubleshooting)

---

## Overview

LLMSpell implements a **3-tier storage architecture** with unified abstraction:

### Architecture Layers

```text
┌─────────────────────────────────────────────────────────────────┐
│ Layer 1: Component-Specific Storage APIs                       │
│  ├─ VectorStorage (4 dimensions: 384, 768, 1536, 3072)         │
│  ├─ EpisodicMemoryStorage (conversation history)               │
│  ├─ SemanticMemoryStorage (knowledge graph)                    │
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
│  ├─ clear (admin operations)                                   │
│  ├─ backend_type() (introspection)                             │
│  └─ characteristics() (performance profile)                    │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Layer 3: Backend Implementations                               │
│  ├─ MemoryBackend (HashMap, non-persistent, testing)           │
│  ├─ SledBackend (embedded KV, persistent, development)         │
│  └─ PostgresBackend (relational, production, scalable)         │
└─────────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Unified Interface**: All backends implement same `StorageBackend` trait
2. **Hot-Swappable**: Components can switch backends without code changes
3. **Per-Component Selection**: Each storage component can use different backend
4. **Performance Profiling**: Backends expose characteristics for optimization
5. **Test-to-Production Path**: Same code works from memory → sled → postgres

---

## Storage Backend Trait

### Core Definition

**Location**: `llmspell-storage/src/traits.rs:48-82`

```rust
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set a key-value pair
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Get multiple values by keys
    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;

    /// Set multiple key-value pairs
    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>;

    /// Delete multiple keys
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;

    /// Clear all data (use with caution)
    async fn clear(&self) -> Result<()>;

    /// Get the backend type
    fn backend_type(&self) -> StorageBackendType;

    /// Get backend characteristics
    fn characteristics(&self) -> StorageCharacteristics;
}
```

### 12 Core Methods

| Method | Purpose | Batch Equivalent | Performance Impact |
|--------|---------|------------------|-------------------|
| `get()` | Retrieve value by key | `get_batch()` | O(1) for HashMap/B-tree |
| `set()` | Store key-value pair | `set_batch()` | O(1) insert + fsync overhead |
| `delete()` | Remove key | `delete_batch()` | O(1) + compaction trigger |
| `exists()` | Check key presence | N/A | O(1) lookup only |
| `list_keys()` | Prefix scan | N/A | O(n) where n = matching keys |
| `get_batch()` | Bulk retrieval | Self | O(k) where k = key count |
| `set_batch()` | Bulk insert | Self | Single transaction/fsync |
| `delete_batch()` | Bulk delete | Self | Single transaction |
| `clear()` | Truncate all data | Self | TRUNCATE TABLE (fast) |
| `backend_type()` | Introspection | N/A | O(1) enum return |
| `characteristics()` | Performance profile | N/A | O(1) struct return |

### StorageCharacteristics

**Location**: `llmspell-storage/src/traits.rs:26-45`

```rust
pub struct StorageCharacteristics {
    /// Whether the backend persists data
    pub persistent: bool,

    /// Whether the backend supports transactions
    pub transactional: bool,

    /// Whether the backend supports key prefix scanning
    pub supports_prefix_scan: bool,

    /// Whether the backend supports atomic operations
    pub supports_atomic_ops: bool,

    /// Estimated read latency in microseconds
    pub avg_read_latency_us: u64,

    /// Estimated write latency in microseconds
    pub avg_write_latency_us: u64,
}
```

**Why This Matters**: Allows runtime performance optimization decisions based on backend capabilities.

---

## Backend Implementations

### 1. MemoryBackend

**Purpose**: Fast non-persistent storage for testing and development

**Location**: `llmspell-storage/src/backends/memory.rs`

**Implementation**:
```rust
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}
```

**Characteristics**:
- **Persistent**: ❌ No (data lost on restart)
- **Transactional**: ✅ Yes (RwLock provides atomic ops)
- **Prefix Scan**: ✅ Yes (linear HashMap scan)
- **Atomic Ops**: ✅ Yes (RwLock guarantees)
- **Read Latency**: ~1 µs (in-memory HashMap lookup)
- **Write Latency**: ~2 µs (HashMap insert + lock)

**Pros**:
- Fastest possible performance (no I/O)
- Zero setup required
- Deterministic behavior for tests

**Cons**:
- Non-persistent (data lost on exit)
- No multi-process sharing
- Memory limited (entire dataset in RAM)

**Use Cases**:
- Unit tests
- Integration tests
- CI/CD pipelines
- Benchmarking without I/O overhead

**Configuration**:
```toml
[storage]
backend = "memory"  # Fastest, non-persistent
```

### 2. SledBackend

**Purpose**: Embedded persistent storage for development and small deployments

**Location**: `llmspell-storage/src/backends/sled_backend.rs`

**Implementation**:
```rust
pub struct SledBackend {
    db: sled::Db,
}
```

**Characteristics**:
- **Persistent**: ✅ Yes (disk-backed with WAL)
- **Transactional**: ✅ Yes (Sled transactions)
- **Prefix Scan**: ✅ Yes (efficient B-tree range scans)
- **Atomic Ops**: ✅ Yes (compare-and-swap)
- **Read Latency**: ~50 µs (B-tree lookup + page cache)
- **Write Latency**: ~200 µs (WAL append + periodic fsync)

**Pros**:
- Zero external dependencies (embedded)
- ACID transactions
- Crash recovery via WAL
- Efficient B-tree structure
- Lock-free reads

**Cons**:
- Single-process only (no multi-writer)
- Limited scalability (single-machine)
- Compaction overhead (background threads)

**Use Cases**:
- Development environments
- Single-machine deployments
- Embedded applications
- CLI tools with persistence

**Configuration**:
```toml
[storage]
backend = "sled"
path = "./sled_data"  # Persistent storage path

[storage.sled]
cache_capacity_mb = 512  # Page cache size
mode = "fast"  # "fast", "balanced", or "safe"
```

**Performance Modes**:
- **fast**: Fsync every 1000ms (risk: 1s data loss)
- **balanced**: Fsync every 100ms (risk: 100ms data loss)
- **safe**: Fsync every write (slowest, zero data loss)

### 3. PostgresBackend

**Purpose**: Production-grade scalable storage with ACID guarantees

**Location**: `llmspell-storage/src/backends/postgres/backend.rs`

**Implementation**:
```rust
pub struct PostgresBackend {
    pool: Arc<PgPool>,
    tenant_id: String,
}
```

**Characteristics**:
- **Persistent**: ✅ Yes (PostgreSQL durability)
- **Transactional**: ✅ Yes (full ACID)
- **Prefix Scan**: ✅ Yes (B-tree index on keys)
- **Atomic Ops**: ✅ Yes (SQL transactions)
- **Read Latency**: ~500 µs (network + query execution)
- **Write Latency**: ~1000 µs (network + WAL + replication)

**Pros**:
- Multi-writer concurrency
- Horizontal scalability (replicas)
- ACID transactions
- Row-Level Security (RLS) for multi-tenancy
- Vector similarity search (VectorChord HNSW)
- Bi-temporal graph support
- Point-in-Time Recovery (PITR)
- Rich querying (SQL, CTEs, window functions)

**Cons**:
- External dependency (PostgreSQL 18+)
- Higher latency vs embedded backends
- More complex deployment
- Resource intensive (connection pools)

**Use Cases**:
- Production deployments
- Multi-tenant SaaS
- Distributed systems
- High availability requirements
- Advanced querying needs

**Configuration**:
```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "postgresql://user:pass@localhost:5432/llmspell_prod"
pool_size = 20  # Formula: (CPU × 2) + 1
pool_timeout_secs = 30
idle_timeout_secs = 600
max_lifetime_secs = 1800
enforce_tenant_isolation = true  # Enable RLS
auto_migrate = false  # Run migrations separately in production
```

---

## 10 Storage Components

### Component Storage Matrix

| Component | Tables (PostgreSQL) | Indexes | Partitioning | RLS | Storage Features |
|-----------|---------------------|---------|--------------|-----|------------------|
| Vector Embeddings | 4 dimension tables | HNSW | By dimension | ✅ | Similarity search |
| Episodic Memory | 1 table | B-tree, GIN | No | ✅ | Temporal ordering |
| Semantic Memory (Graph) | 2 tables (entities, relationships) | GiST, B-tree | No | ✅ | Bi-temporal |
| Procedural Memory | 1 table | GIN (JSONB) | No | ✅ | Pattern matching |
| Agent State | 1 table | B-tree | No | ✅ | Snapshot storage |
| Workflow States | 1 table | B-tree | No | ✅ | State machine |
| Sessions | 1 table | B-tree | No | ✅ | TTL expiration |
| Artifacts | 1 table | Hash (blake3) | No | ✅ | Content-addressed |
| Event Log | 1 table | B-tree | Monthly ranges | ✅ | Time-series |
| Hook History | 1 table | B-tree | No | ✅ | Audit trail |

### 1. Vector Embeddings Storage

**Tables**: `vector_embeddings_384`, `_768`, `_1536`, `_3072`

**Schema** (384-dimensional example):
```sql
CREATE TABLE llmspell.vector_embeddings_384 (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    embedding vector(384) NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT fk_tenant FOREIGN KEY (tenant_id)
        REFERENCES llmspell.tenants(tenant_id) ON DELETE CASCADE
);

-- HNSW vector similarity index
CREATE INDEX idx_vector_384_hnsw ON vector_embeddings_384
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 128);
```

**Features**:
- **4 dimension sizes**: 384 (fast), 768 (standard), 1536 (GPT), 3072 (large models)
- **HNSW indexes**: Hierarchical Navigable Small World for <5ms similarity search
- **Multi-tenancy**: RLS policies enforce tenant isolation
- **Metadata**: JSONB for flexible annotation

**Performance**:
- **8.47x speedup** vs linear scan (measured with 10K vectors)
- **<5ms p95 latency** for top-10 similarity search
- **95% recall** with `ef_search = 40`

### 2. Episodic Memory Storage

**Table**: Stores conversation history for memory retrieval

**Purpose**: Time-ordered conversation turns for context assembly

**Features**:
- Session-based partitioning
- Role-based filtering (user, assistant, system)
- Timestamp ordering for temporal queries
- GIN index on metadata JSONB

### 3. Semantic Memory (Graph) Storage

**Tables**: `entities`, `relationships`

**Schema**:
```sql
CREATE TABLE llmspell.entities (
    entity_id UUID NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    entity_type VARCHAR(255) NOT NULL,
    name VARCHAR(500) NOT NULL,
    properties JSONB NOT NULL DEFAULT '{}',
    valid_time_start TIMESTAMPTZ NOT NULL,
    valid_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    transaction_time_start TIMESTAMPTZ NOT NULL DEFAULT now(),
    transaction_time_end TIMESTAMPTZ NOT NULL DEFAULT 'infinity',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (entity_id, transaction_time_start)
);

CREATE INDEX idx_entities_temporal ON llmspell.entities
    USING gist (entity_id, tstzrange(valid_time_start, valid_time_end));
```

**Features**:
- **Bi-temporal modeling**: Valid time + transaction time
- **Graph queries**: Relationships with source/target entities
- **GiST indexes**: Efficient temporal range queries
- **JSONB properties**: Flexible schema evolution

### 4. Procedural Memory Storage

**Table**: Stores patterns, skills, and learned procedures

**Features**:
- Success/failure tracking
- Execution count metrics
- GIN index on pattern metadata
- Category-based organization

### 5. Agent State Storage

**Table**: Snapshot storage for agent state persistence

**Features**:
- SHA-256 checksums for integrity
- Agent lifecycle tracking
- Snapshot versioning

### 6. Workflow States Storage

**Table**: Workflow execution state machine

**Features**:
- Status tracking (pending, running, completed, failed)
- Execution duration metrics
- Error tracking

### 7. Sessions Storage

**Table**: Session lifecycle management

**Features**:
- TTL-based expiration
- Snapshot storage (JSONB)
- Artifact linking

### 8. Artifacts Storage

**Table**: Content-addressed file storage

**Features**:
- **blake3 hashing**: Content deduplication
- **Metadata**: MIME types, compression flags
- **Artifact types**: code, document, image, data, other

**Schema**:
```sql
CREATE TABLE llmspell.artifacts (
    artifact_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    content_hash VARCHAR(128) NOT NULL,  -- blake3
    content BYTEA NOT NULL,
    content_type VARCHAR(255),  -- MIME type
    is_compressed BOOLEAN NOT NULL DEFAULT false,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX idx_artifacts_dedup
    ON llmspell.artifacts (tenant_id, content_hash);
```

**Deduplication**:
- Same content uploaded twice → single storage
- Reference counting via unique index
- Space savings: 50-90% in typical workloads

### 9. Event Log Storage

**Table**: Partitioned time-series event log

**Schema**:
```sql
CREATE TABLE llmspell.event_log (
    event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(255) NOT NULL,
    event_data JSONB NOT NULL,
    correlation_id UUID,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now()
) PARTITION BY RANGE (timestamp);

-- Monthly partitions (example for January 2025)
CREATE TABLE llmspell.event_log_2025_01
    PARTITION OF llmspell.event_log
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

**Features**:
- **Monthly partitioning**: 12.5x query speedup (partition pruning)
- **Correlation IDs**: Trace distributed requests
- **JSONB event data**: Flexible schema
- **Retention policies**: Auto-drop old partitions

**Performance**:
- **10K events/sec ingestion** (measured with batch inserts)
- **<100ms query latency** for recent events (hot partition)

### 10. Hook History Storage

**Table**: Hook execution audit trail

**Features**:
- Execution timestamps
- Input/output capture
- Error tracking
- Performance metrics

---

## Hot-Swap Architecture

### Per-Component Backend Selection

Each storage component can independently choose its backend:

```toml
[storage]
backend = "postgres"  # Global default

# Override for specific components
[storage.vector_embeddings]
backend = "hnsw"  # Use HNSW for vectors

[storage.sessions]
backend = "memory"  # Use memory for fast session access

[storage.event_log]
backend = "postgres"  # Use PostgreSQL for durability
```

### Backend Selection Flow

```text
Component Request
      ↓
Check component-specific backend config
      ↓
If not set → Use global backend setting
      ↓
Initialize StorageBackend implementation
      ↓
Call trait methods (get/set/delete/etc.)
```

### Migration Pattern

**Scenario**: Migrate from Sled to PostgreSQL without downtime

```rust
// Phase 1: Dual-write to both backends
let sled_backend = Arc::new(SledBackend::new("./data")?);
let postgres_backend = Arc::new(PostgresBackend::new(pool)?);

// Write to both
sled_backend.set(key, value.clone()).await?;
postgres_backend.set(key, value).await?;

// Phase 2: Read from PostgreSQL, fallback to Sled
match postgres_backend.get(key).await? {
    Some(value) => Ok(value),
    None => sled_backend.get(key).await?,
}

// Phase 3: Switch reads to PostgreSQL only
// Phase 4: Stop writing to Sled
// Phase 5: Remove Sled backend
```

---

## Performance Characteristics

### Latency Comparison

| Operation | Memory | Sled | PostgreSQL (local) | PostgreSQL (remote) |
|-----------|--------|------|-------------------|-------------------|
| Single get() | 1 µs | 50 µs | 500 µs | 2 ms |
| Single set() | 2 µs | 200 µs | 1 ms | 3 ms |
| Batch get (100) | 100 µs | 2 ms | 10 ms | 20 ms |
| Batch set (100) | 200 µs | 5 ms | 15 ms | 30 ms |
| Prefix scan (1K keys) | 500 µs | 5 ms | 50 ms | 100 ms |

### Throughput Comparison

| Backend | Reads/sec | Writes/sec | Batch Inserts/sec |
|---------|-----------|------------|-------------------|
| Memory | 1M+ | 500K+ | 100K+ (100-item batches) |
| Sled | 50K | 5K | 10K |
| PostgreSQL | 10K | 2K | 20K (with batching) |

**Note**: Benchmarks measured on M1 MacBook Pro, single-threaded.

### Storage Overhead

| Backend | Metadata Overhead | Index Overhead | Compaction |
|---------|-------------------|----------------|-----------|
| Memory | None | None | N/A |
| Sled | ~10% (B-tree nodes) | Included | Automatic (background) |
| PostgreSQL | ~20% (row headers, TOAST) | 50-100% (HNSW indexes) | VACUUM (scheduled) |

---

## Migration Strategies

### Development → Production Path

**Stage 1: Local Development (Memory)**
```toml
[storage]
backend = "memory"
```
- Instant startup
- Zero config
- Fast iteration

**Stage 2: Persistent Development (Sled)**
```toml
[storage]
backend = "sled"
path = "./dev_data"
```
- Survives restarts
- ACID guarantees
- Single-machine testing

**Stage 3: Production (PostgreSQL)**
```toml
[storage]
backend = "postgres"

[storage.postgres]
url = "${DATABASE_URL}"
pool_size = 20
enforce_tenant_isolation = true
```
- Multi-tenant isolation
- Horizontal scalability
- PITR recovery

### Data Migration Tools

**Export from Sled**:
```rust
async fn export_sled_to_json(sled_backend: &SledBackend) -> Result<()> {
    let keys = sled_backend.list_keys("").await?;
    let mut data = HashMap::new();

    for key in keys {
        if let Some(value) = sled_backend.get(&key).await? {
            data.insert(key, value);
        }
    }

    let json = serde_json::to_string(&data)?;
    std::fs::write("export.json", json)?;
    Ok(())
}
```

**Import to PostgreSQL**:
```rust
async fn import_json_to_postgres(
    postgres_backend: &PostgresBackend,
    path: &str
) -> Result<()> {
    let json = std::fs::read_to_string(path)?;
    let data: HashMap<String, Vec<u8>> = serde_json::from_str(&json)?;

    // Batch import for performance
    postgres_backend.set_batch(data).await?;
    Ok(())
}
```

---

## Code References

### Core Trait Files

| File | Lines | Purpose |
|------|-------|---------|
| `llmspell-storage/src/traits.rs` | 103 | StorageBackend trait, characteristics |
| `llmspell-storage/src/backends/memory.rs` | 120 | MemoryBackend implementation |
| `llmspell-storage/src/backends/sled_backend.rs` | 250 | SledBackend implementation |
| `llmspell-storage/src/backends/postgres/backend.rs` | 400+ | PostgresBackend implementation |

### Component-Specific Storage

| Component | Location | Purpose |
|-----------|----------|---------|
| Vector Storage | `llmspell-storage/src/backends/vector/hnsw.rs` | HNSW vector similarity |
| Graph Storage | `llmspell-storage/src/backends/postgres/graph.rs` | Bi-temporal graph |
| Procedural Storage | `llmspell-storage/src/backends/postgres/procedural.rs` | Pattern storage |

### Key Trait Methods

```rust
// llmspell-storage/src/traits.rs:50-82

async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
async fn delete(&self, key: &str) -> Result<()>;
async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;
async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;
async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>;
fn backend_type(&self) -> StorageBackendType;
fn characteristics(&self) -> StorageCharacteristics;
```

---

## Usage Examples

### Basic CRUD Operations

```rust
use llmspell_storage::{MemoryBackend, StorageBackend};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let backend = MemoryBackend::new();

    // Set value
    backend.set("user:123", b"Alice".to_vec()).await?;

    // Get value
    if let Some(value) = backend.get("user:123").await? {
        println!("User: {}", String::from_utf8(value)?);
    }

    // Delete value
    backend.delete("user:123").await?;

    Ok(())
}
```

### Batch Operations

```rust
use std::collections::HashMap;

// Batch insert (single transaction)
let mut items = HashMap::new();
items.insert("user:1".to_string(), b"Alice".to_vec());
items.insert("user:2".to_string(), b"Bob".to_vec());
items.insert("user:3".to_string(), b"Charlie".to_vec());

backend.set_batch(items).await?;

// Batch retrieve
let keys = vec!["user:1".to_string(), "user:2".to_string()];
let results = backend.get_batch(&keys).await?;

for (key, value) in results {
    println!("{}: {}", key, String::from_utf8(value)?);
}
```

### Prefix Scanning

```rust
// List all user keys
let user_keys = backend.list_keys("user:").await?;
println!("Found {} users", user_keys.len());

// List all session keys
let session_keys = backend.list_keys("session:").await?;
```

### Backend Characteristics Inspection

```rust
let chars = backend.characteristics();

if chars.persistent {
    println!("Backend persists data to disk");
}

if chars.transactional {
    println!("Backend supports atomic transactions");
}

println!("Average read latency: {} µs", chars.avg_read_latency_us);
println!("Average write latency: {} µs", chars.avg_write_latency_us);
```

---

## Troubleshooting

### Issue: Slow Prefix Scans

**Symptom**: `list_keys("prefix:")` takes >1 second

**Diagnosis**:
- Check key count: `SELECT COUNT(*) FROM table WHERE key LIKE 'prefix:%'`
- Memory backend: Linear scan O(n) over all keys
- Sled backend: B-tree range scan (faster)
- PostgreSQL: B-tree index scan (fastest with proper index)

**Solution**:
```sql
-- Ensure index on key column (PostgreSQL)
CREATE INDEX idx_storage_keys ON storage_table (key text_pattern_ops);
```

### Issue: Connection Pool Exhaustion

**Symptom**: `Timeout waiting for connection from pool`

**Diagnosis**:
```toml
[storage.postgres]
pool_size = 20  # Too small?
pool_timeout_secs = 30
```

**Solution**:
- Increase pool_size: Formula = `(CPU cores × 2) + 1`
- Check for leaked connections (missing close calls)
- Enable connection metrics: `RUST_LOG=sqlx=debug`

### Issue: HNSW Index Build Slow

**Symptom**: First query takes >30 seconds

**Cause**: Index not pre-built or `ef_construction` too high

**Solution**:
```sql
-- Reduce ef_construction for faster builds (lower recall)
CREATE INDEX idx_vector_hnsw ON vector_embeddings_768
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 64);  -- Lower from 128

-- Pre-build index with maintenance_work_mem
SET maintenance_work_mem = '1GB';
REINDEX INDEX idx_vector_hnsw;
```

### Issue: Sled Compaction Spikes

**Symptom**: Periodic latency spikes (100ms+) during writes

**Cause**: Background compaction kicking in

**Solution**:
```toml
[storage.sled]
mode = "balanced"  # Reduce fsync frequency
cache_capacity_mb = 1024  # Larger cache = fewer flushes
```

**Alternative**: Switch to PostgreSQL for production workloads

---

**🔗 See Also**:
- [PostgreSQL Setup Guide](../user-guide/storage/postgresql-setup.md) - Complete setup instructions
- [Schema Reference](../user-guide/storage/schema-reference.md) - All 15 tables documented
- [Performance Tuning](../user-guide/storage/performance-tuning.md) - Optimization guide
- [Backup & Restore](../user-guide/storage/backup-restore.md) - Disaster recovery
