# Storage Backends

**Thematic guide to llmspell's storage architecture and implementations**

🔗 **Quick Links**: `cargo doc --open -p llmspell-storage` | [Crate Index](crate-index.md)

---

## Overview

llmspell provides a unified storage abstraction layer with multiple backend implementations for different use cases. The `StorageBackend` trait enables backend-agnostic persistence, while specialized vector storage supports RAG and similarity search operations.

**Core Storage Components**:
- **StorageBackend trait**: Unified key-value interface
- **Vector Storage**: HNSW-based similarity search via vectorlite-rs SQLite extension
- **Multi-Tenant Collections**: Isolated data per tenant
- **Persistence Options**: In-memory, SQLite (libsql with HNSW), PostgreSQL

---

## Storage Architecture

### Layer Structure

```
Application Layer
    ↓
StorageBackend Trait (abstraction)
    ↓
    ├─→ InMemoryBackend (testing, development)
    ├─→ SqliteBackend (embedded persistent KV)
    └─→ SqliteVectorStorage (RAG, similarity search)
```

### Backend Selection Patterns

**Development/Testing**:
```rust
use llmspell_storage::InMemoryBackend;

let storage = InMemoryBackend::new();
// Fast, no persistence, isolated tests
```

**Embedded Persistence**:
```rust
use llmspell_storage::SqliteBackend;

let storage = SqliteBackend::new("/path/to/db")?;
// Embedded DB, persistence, single-process
```

**Vector Search (RAG)**:
```rust
use llmspell_storage::vector::SqliteVectorStorage;

let storage = SqliteVectorStorage::new(config)?;
// Similarity search, embeddings, multi-dimensional
```

---

## StorageBackend Trait

**Purpose**: Unified interface for key-value storage

**Core Operations**:
```rust
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> StorageResult<()>;
    async fn delete(&self, key: &str) -> StorageResult<()>;
    async fn exists(&self, key: &str) -> StorageResult<bool>;
    async fn list_keys(&self, prefix: Option<&str>) -> StorageResult<Vec<String>>;
    async fn clear(&self) -> StorageResult<()>;
}
```

**Advanced Operations**:
- `stats()` - Storage statistics (size, keys, etc.)
- `batch()` - Batch operations for efficiency
- `transaction()` - Atomic multi-operation transactions

**Usage Pattern**:
```rust
// Store data
storage.set("user:123", user_data).await?;

// Retrieve data
if let Some(data) = storage.get("user:123").await? {
    let user: User = deserialize(&data)?;
}

// List with prefix
let user_keys = storage.list_keys(Some("user:")).await?;

// Batch operations
storage.batch(vec![
    BatchOperation::Set { key: "k1", value: v1 },
    BatchOperation::Set { key: "k2", value: v2 },
]).await?;
```

📚 **Full Details**: [llmspell-storage.md](llmspell-storage.md#storagebackend-trait)

---

## Vector Storage (HNSW via vectorlite-rs)

**Purpose**: Fast similarity search for embeddings (Phase 8 RAG foundation, Phase 13c SQLite integration)

### Architecture

**HNSW (Hierarchical Navigable Small World)**:
- Implemented via vectorlite-rs pure Rust SQLite extension
- Graph-based approximate nearest neighbor search
- O(log N) search complexity
- Multi-layer structure for fast traversal
- Configurable precision vs speed trade-offs
- Integrated with SQLite for persistence and SQL querying

**Dimension Routing**:
Automatic routing based on embedding dimensions:
- 384-dim → `text-embedding-ada-002` models
- 768-dim → BERT-family models
- 1536-dim → OpenAI `text-embedding-3-small`
- 3072-dim → OpenAI `text-embedding-3-large`

### Key Features

**1. Similarity Search**:
```rust
let results = vector_storage.search(
    "collection_name",
    query_embedding,  // Vec<f32>
    k: 10,            // Top 10 results
    filter: None      // Optional metadata filter
).await?;

for result in results {
    println!("ID: {}, Score: {}, Metadata: {:?}",
        result.id, result.score, result.metadata);
}
```

**2. Metadata Filtering**:
```rust
let filter = MetadataFilter::builder()
    .eq("category", "documentation")
    .gt("created_at", "2024-01-01")
    .build();

let results = vector_storage.search_with_filter(
    "docs",
    embedding,
    k: 5,
    Some(filter)
).await?;
```

**3. Collection Management**:
```rust
// Create collection
vector_storage.create_collection("knowledge_base", dimension: 1536).await?;

// Add vectors
vector_storage.add(
    "knowledge_base",
    vec![
        Vector {
            id: "doc1",
            embedding: vec![...],  // 1536 dimensions
            metadata: json!({
                "title": "Getting Started",
                "category": "docs"
            })
        }
    ]
).await?;

// Delete collection
vector_storage.delete_collection("knowledge_base").await?;
```

**4. Multi-Tenant Isolation**:
```rust
// Tenant-specific collections
let tenant_collection = format!("tenant_{}:vectors", tenant_id);

vector_storage.create_collection(&tenant_collection, 1536).await?;
vector_storage.add(&tenant_collection, vectors).await?;
```

**5. Bi-Temporal Support**:
```rust
// Track event time and ingestion time
let vector = Vector {
    id: "event_1",
    embedding: vec![...],
    metadata: json!({
        "event_time": "2024-01-01T12:00:00Z",
        "ingestion_time": Utc::now(),
    })
};

// Query by time ranges
let filter = MetadataFilter::builder()
    .range("event_time", start_time, end_time)
    .build();
```

**6. TTL (Time-To-Live)**:
```rust
// Automatic expiration
vector_storage.add_with_ttl(
    "temporary_cache",
    vectors,
    ttl: Duration::from_hours(24)
).await?;

// Vectors auto-deleted after 24 hours
```

### Performance Characteristics

| Operation | Complexity | Typical Time |
|-----------|-----------|--------------|
| Insert | O(log N) | <1ms per vector |
| Search (k=10) | O(log N) | 2-10ms |
| Delete | O(1) | <1ms |
| Collection Create | O(1) | <1ms |

**Tuning Parameters**:
- `ef_construction`: Build-time precision (default: 200)
- `ef_search`: Query-time precision (default: 50)
- `M`: Connections per layer (default: 16)

```rust
let config = HNSWConfig {
    ef_construction: 400,  // Higher = better precision, slower build
    ef_search: 100,        // Higher = better recall, slower search
    M: 32,                 // Higher = more connections, more memory
    ..Default::default()
};
```

📚 **Full Details**: [llmspell-storage.md](llmspell-storage.md#vector-storage-hnsw)

---

## Backend Implementations

### InMemoryBackend

**Use Case**: Testing, development, ephemeral data

**Characteristics**:
- `Arc<DashMap>` for thread-safe concurrent access
- Zero persistence (data lost on restart)
- Fastest performance (~microseconds)
- No external dependencies

**When to Use**:
- Unit tests
- Integration tests
- Temporary caching
- Prototyping

```rust
let storage = InMemoryBackend::new();
// Automatically cleaned up on drop
```

### SqliteBackend

**Use Case**: Embedded persistence, single-process applications

**Characteristics**:
- Embedded relational database with key-value abstractions
- ACID transactions
- Crash recovery
- Low memory footprint
- File-based persistence
- Integrated vector search via vectorlite-rs HNSW extension

**When to Use**:
- CLI applications needing persistence
- Desktop applications
- Single-server deployments
- Configuration storage

```rust
let storage = SqliteBackend::new("/var/lib/llmspell/data")?;
// Data persists across restarts
```

**Configuration**:
```rust
let config = SqliteConfig {
    cache_capacity: 1_000_000_000,  // 1GB cache
    flush_every_ms: Some(1000),     // Flush every 1s
    ..Default::default()
};
let storage = SqliteBackend::with_config("/path", config)?;
```

### SqliteVectorStorage

**Use Case**: RAG, similarity search, semantic search

**Characteristics**:
- Optimized for high-dimensional vectors
- Approximate nearest neighbor (ANN) search
- Metadata filtering support
- Multi-tenant collections
- Dimension-aware routing

**When to Use**:
- Retrieval-Augmented Generation (RAG)
- Semantic search applications
- Recommendation systems
- Document similarity
- Image similarity (with vision embeddings)

```rust
let config = HNSWConfig::default()
    .with_dimension(1536)
    .with_max_elements(1_000_000);

let storage = SqliteVectorStorage::new(config)?;
```

📚 **Full Details**: [llmspell-storage.md](llmspell-storage.md#backend-implementations)

---

## Multi-Tenant Patterns

### Collection Naming

**Pattern**: `tenant_{id}:collection_name`

```rust
fn tenant_collection(tenant_id: &str, name: &str) -> String {
    format!("tenant_{}:{}", tenant_id, name)
}

// Usage
let collection = tenant_collection("acme_corp", "documents");
vector_storage.create_collection(&collection, 1536).await?;
```

### Tenant Isolation

**Key Principles**:
1. Never share collections across tenants
2. Validate tenant ID on every operation
3. Use metadata filtering as secondary check
4. Implement resource quotas per tenant

```rust
async fn search_tenant_documents(
    storage: &SqliteVectorStorage,
    tenant_id: &str,
    query: Vec<f32>,
) -> Result<Vec<SearchResult>> {
    // Primary isolation: tenant-specific collection
    let collection = tenant_collection(tenant_id, "documents");

    // Secondary isolation: metadata filter
    let filter = MetadataFilter::builder()
        .eq("tenant_id", tenant_id)
        .build();

    storage.search_with_filter(&collection, query, 10, Some(filter)).await
}
```

### Resource Quotas

```rust
pub struct TenantQuota {
    max_collections: usize,
    max_vectors_per_collection: usize,
    max_total_vectors: usize,
}

async fn enforce_quota(
    storage: &SqliteVectorStorage,
    tenant_id: &str,
    quota: &TenantQuota,
) -> Result<()> {
    let collections = storage.list_collections(Some(&tenant_collection(tenant_id, ""))).await?;

    if collections.len() >= quota.max_collections {
        return Err(StorageError::QuotaExceeded("collections"));
    }

    Ok(())
}
```

---

## Performance Optimization

### Batch Operations

**Pattern**: Group operations to reduce overhead

```rust
// Instead of individual operations
for item in items {
    storage.set(&item.key, item.value).await?;  // N round trips
}

// Use batch operations
let ops = items.into_iter()
    .map(|item| BatchOperation::Set {
        key: item.key,
        value: item.value
    })
    .collect();
storage.batch(ops).await?;  // 1 round trip
```

### Caching Strategy

```rust
use std::sync::Arc;
use dashmap::DashMap;

pub struct CachedStorage {
    backend: Arc<dyn StorageBackend>,
    cache: Arc<DashMap<String, Vec<u8>>>,
}

impl CachedStorage {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        // Check cache first
        if let Some(value) = self.cache.get(key) {
            return Ok(Some(value.clone()));
        }

        // Fallback to backend
        if let Some(value) = self.backend.get(key).await? {
            self.cache.insert(key.to_string(), value.clone());
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}
```

### Vector Search Optimization

**1. Dimension Reduction**:
```rust
// Use PCA or other techniques to reduce dimensions
// Trade-off: Faster search vs slightly lower precision
let reduced_embedding = reduce_dimensions(original_embedding, target: 384);
```

**2. Pre-filtering**:
```rust
// Apply filters before similarity search to reduce search space
let filter = MetadataFilter::builder()
    .eq("status", "published")
    .gt("date", "2024-01-01")
    .build();

// Only searches published documents from 2024
let results = storage.search_with_filter(collection, embedding, 10, Some(filter)).await?;
```

**3. Index Warming**:
```rust
// Load frequently accessed collections into memory
async fn warm_index(storage: &SqliteVectorStorage, collections: Vec<String>) {
    for collection in collections {
        // Trigger index loading
        let _ = storage.get_collection_info(&collection).await;
    }
}
```

---

## Testing Patterns

### Test Storage Abstraction

```rust
#[cfg(test)]
mod tests {
    use llmspell_storage::{InMemoryBackend, StorageBackend};

    async fn test_storage_backend<S: StorageBackend>(storage: S) {
        // Test works with any backend implementation
        storage.set("key", b"value".to_vec()).await.unwrap();
        let value = storage.get("key").await.unwrap();
        assert_eq!(value, Some(b"value".to_vec()));
    }

    #[tokio::test]
    async fn test_in_memory() {
        test_storage_backend(InMemoryBackend::new()).await;
    }

    #[tokio::test]
    async fn test_sqlite() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        test_storage_backend(SqliteBackend::new(&db_path).await.unwrap()).await;
    }
}
```

### Mock Vector Storage

```rust
use llmspell_testing::mocks::MockVectorStorage;

let mock_storage = MockVectorStorage::new()
    .with_search_results(vec![
        SearchResult { id: "doc1", score: 0.95, metadata: json!({}) },
        SearchResult { id: "doc2", score: 0.92, metadata: json!({}) },
    ]);

// Use in tests
let results = mock_storage.search("collection", embedding, 10, None).await?;
assert_eq!(results.len(), 2);
```

---

## Integration with Other Components

### RAG Pipeline (llmspell-rag)

```rust
use llmspell_rag::RAGPipeline;
use llmspell_storage::vector::SqliteVectorStorage;

let storage = Arc::new(SqliteVectorStorage::new(config)?);
let rag_pipeline = RAGPipeline::builder()
    .with_vector_storage(storage)
    .with_embedding_model("text-embedding-3-small")
    .build()?;
```

### Memory System (llmspell-memory)

```rust
use llmspell_memory::EpisodicMemory;
use llmspell_storage::vector::SqliteVectorStorage;

let storage = Arc::new(SqliteVectorStorage::new(config)?);
let episodic_memory = EpisodicMemory::builder()
    .with_backend(storage)
    .with_max_memories(10000)
    .build()?;
```

### State Persistence (llmspell-state-persistence)

```rust
use llmspell_state_persistence::PersistentState;
use llmspell_storage::SqliteBackend;

let storage = Arc::new(SqliteBackend::new("/var/lib/llmspell/state")?);
let state = PersistentState::new(storage);
```

---

## Export/Import API (Phase 13c.3.2)

**Lossless bidirectional migration between PostgreSQL and SQLite**

llmspell provides a comprehensive export/import system for migrating data between storage backends. The system is designed for zero data loss, transaction safety, and format versioning.

### Architecture

**Export Pipeline**:
```
PostgreSQL/SQLite Backend
    ↓
Backend Exporter (PostgresExporter / SqliteExporter)
    ↓
ExportFormat (versioned JSON structure)
    ↓
JSON File (with base64-encoded binary data)
```

**Import Pipeline**:
```
JSON File
    ↓
Parse & Validate ExportFormat
    ↓
Backend Importer (PostgresImporter / SqliteImporter)
    ↓
Transaction-Safe Import
    ↓
ImportStats (verification)
```

---

### Export Format Structure

**ExportFormat** (llmspell-storage/src/export_import/format.rs):

```rust
use llmspell_storage::export_import::ExportFormat;

pub struct ExportFormat {
    /// Format version (semantic versioning)
    pub version: String,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Source backend type ("postgresql" | "sqlite")
    pub source_backend: String,
    /// List of applied migrations (["V3", "V4", "V5", ...])
    pub migrations: Vec<String>,
    /// Exported data organized by table/migration
    pub data: ExportData,
}

pub struct ExportData {
    /// Vector embeddings by dimension (384, 768, 1536, 3072)
    pub vector_embeddings: HashMap<usize, Vec<VectorEmbeddingExport>>,
    /// Knowledge graph (entities and relationships)
    pub knowledge_graph: Option<KnowledgeGraphExport>,
    /// Procedural memory patterns
    pub procedural_memory: Vec<PatternExport>,
    /// Agent state entries
    pub agent_state: Vec<AgentStateExport>,
    /// Key-value store entries
    pub kv_store: Vec<KVEntryExport>,
    /// Workflow states
    pub workflow_states: Vec<WorkflowStateExport>,
    /// Sessions
    pub sessions: Vec<SessionExport>,
    /// Artifacts and content
    pub artifacts: Option<ArtifactsExport>,
    /// Event log entries
    pub event_log: Vec<EventExport>,
    /// Hook history entries
    pub hook_history: Vec<HookExport>,
}
```

**Format Characteristics**:
- **Version**: 1.0 (semantic versioning for compatibility)
- **Encoding**: JSON with base64 for binary data (BLOB fields)
- **Serde Attributes**: `#[serde(default, skip_serializing_if)]` for optional fields
- **Metadata**: Export timestamp, source backend, migration list

---

### Export API

#### SQLite Export

```rust
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
use llmspell_storage::export_import::SqliteExporter;
use std::sync::Arc;

// Create backend
let config = SqliteConfig::new("/path/to/llmspell.db");
let backend = Arc::new(SqliteBackend::new(config).await?);

// Create exporter
let exporter = SqliteExporter::new(backend);

// Export all data
let export_data = exporter.export_all().await?;

// Serialize to JSON
let json = serde_json::to_string_pretty(&export_data)?;
std::fs::write("export.json", json)?;

println!("Exported {} records", count_records(&export_data));
```

#### PostgreSQL Export

```rust
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
use llmspell_storage::export_import::PostgresExporter;
use std::sync::Arc;

// Create backend (requires "postgres" feature)
let config = PostgresConfig::new(
    std::env::var("DATABASE_URL").expect("DATABASE_URL not set")
);
let backend = Arc::new(PostgresBackend::new(config).await?);

// Create exporter
let exporter = PostgresExporter::new(backend);

// Export all data
let export_data = exporter.export_all().await?;

// Serialize to JSON
let json = serde_json::to_string_pretty(&export_data)?;
std::fs::write("pg-export.json", json)?;
```

**Export Methods**:
- `export_all()` - Export all tables and data types
- Automatic base64 encoding for binary data (BLOB fields)
- Metadata preserved (timestamps, IDs, relationships)

---

### Import API

#### SQLite Import

```rust
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
use llmspell_storage::export_import::SqliteImporter;
use std::sync::Arc;

// Create backend
let config = SqliteConfig::new("/path/to/target.db");
let backend = Arc::new(SqliteBackend::new(config).await?);

// Create importer
let importer = SqliteImporter::new(backend);

// Import from JSON file
let stats = importer.import_from_file("export.json").await?;

// Verify import statistics
println!("✅ Imported {} total records", stats.total());
println!("  - Vectors: {}", stats.vectors);
println!("  - Entities: {}", stats.entities);
println!("  - Sessions: {}", stats.sessions);
```

#### PostgreSQL Import

```rust
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
use llmspell_storage::export_import::PostgresImporter;
use std::sync::Arc;

// Create backend (requires "postgres" feature)
let config = PostgresConfig::new(std::env::var("DATABASE_URL")?);
let backend = Arc::new(PostgresBackend::new(config).await?);

// Create importer
let importer = PostgresImporter::new(backend);

// Import from JSON file
let stats = importer.import_from_file("pg-export.json").await?;

// Verify results
assert_eq!(stats.vectors, 500);
assert_eq!(stats.sessions, 12);
```

**Import Characteristics**:
- **Transaction-safe**: All-or-nothing import with automatic rollback on errors
- **Validates JSON**: Parses and validates format before importing
- **Detailed stats**: Returns ImportStats with counts for all data types

---

### ImportStats Structure

```rust
pub struct ImportStats {
    /// Number of vector embeddings imported
    pub vectors: usize,
    /// Number of entities imported
    pub entities: usize,
    /// Number of relationships imported
    pub relationships: usize,
    /// Number of procedural patterns imported
    pub patterns: usize,
    /// Number of agent states imported
    pub agent_states: usize,
    /// Number of KV entries imported
    pub kv_entries: usize,
    /// Number of workflow states imported
    pub workflow_states: usize,
    /// Number of sessions imported
    pub sessions: usize,
    /// Number of artifact content entries imported
    pub artifact_content: usize,
    /// Number of artifacts imported
    pub artifacts: usize,
    /// Number of events imported
    pub events: usize,
    /// Number of hooks imported
    pub hooks: usize,
}

impl ImportStats {
    /// Total records imported across all types
    pub fn total(&self) -> usize {
        self.vectors + self.entities + self.relationships +
        self.patterns + self.agent_states + self.kv_entries +
        self.workflow_states + self.sessions +
        self.artifact_content + self.artifacts +
        self.events + self.hooks
    }
}
```

---

### Migration Patterns

#### Pattern 1: SQLite → PostgreSQL (Development to Production)

```rust
use llmspell_storage::backends::{sqlite::*, postgres::*};
use llmspell_storage::export_import::*;
use std::sync::Arc;

async fn migrate_sqlite_to_postgres(
    sqlite_path: &str,
    postgres_url: &str,
) -> Result<ImportStats> {
    // 1. Export from SQLite
    let sqlite_config = SqliteConfig::new(sqlite_path);
    let sqlite_backend = Arc::new(SqliteBackend::new(sqlite_config).await?);
    let exporter = SqliteExporter::new(sqlite_backend);

    let export_data = exporter.export_all().await?;

    // 2. Save to temporary file
    let temp_file = "/tmp/migration.json";
    let json = serde_json::to_string_pretty(&export_data)?;
    std::fs::write(temp_file, json)?;

    // 3. Import to PostgreSQL
    let postgres_config = PostgresConfig::new(postgres_url.to_string());
    let postgres_backend = Arc::new(PostgresBackend::new(postgres_config).await?);
    let importer = PostgresImporter::new(postgres_backend);

    let stats = importer.import_from_file(temp_file).await?;

    // 4. Verify
    println!("Migration complete: {} records", stats.total());

    Ok(stats)
}
```

#### Pattern 2: PostgreSQL → SQLite (Production to Development)

```rust
async fn migrate_postgres_to_sqlite(
    postgres_url: &str,
    sqlite_path: &str,
) -> Result<ImportStats> {
    // 1. Export from PostgreSQL
    let postgres_config = PostgresConfig::new(postgres_url.to_string());
    let postgres_backend = Arc::new(PostgresBackend::new(postgres_config).await?);
    let exporter = PostgresExporter::new(postgres_backend);

    let export_data = exporter.export_all().await?;

    // 2. Import to SQLite
    let sqlite_config = SqliteConfig::new(sqlite_path);
    let sqlite_backend = Arc::new(SqliteBackend::new(sqlite_config).await?);
    let importer = SqliteImporter::new(sqlite_backend);

    // Import directly from export_data (no intermediate file)
    let json = serde_json::to_string(&export_data)?;
    let temp_file = "/tmp/migration.json";
    std::fs::write(temp_file, json)?;

    let stats = importer.import_from_file(temp_file).await?;

    Ok(stats)
}
```

#### Pattern 3: Roundtrip Verification

```rust
async fn verify_roundtrip_migration(db_path: &str) -> Result<()> {
    // 1. Export from source
    let config1 = SqliteConfig::new(db_path);
    let backend1 = Arc::new(SqliteBackend::new(config1).await?);
    let exporter1 = SqliteExporter::new(Arc::clone(&backend1));
    let export1 = exporter1.export_all().await?;

    // 2. Import to temporary database
    let temp_db = "/tmp/roundtrip-test.db";
    let config2 = SqliteConfig::new(temp_db);
    let backend2 = Arc::new(SqliteBackend::new(config2).await?);
    let importer = SqliteImporter::new(Arc::clone(&backend2));

    let json1 = serde_json::to_string(&export1)?;
    std::fs::write("/tmp/export1.json", &json1)?;
    importer.import_from_file("/tmp/export1.json").await?;

    // 3. Export from temporary database
    let exporter2 = SqliteExporter::new(backend2);
    let export2 = exporter2.export_all().await?;

    // 4. Compare data (ignore timestamps)
    let data1 = serde_json::to_value(&export1.data)?;
    let data2 = serde_json::to_value(&export2.data)?;

    assert_eq!(data1, data2, "Roundtrip data mismatch");

    println!("✅ Roundtrip verification passed");
    Ok(())
}
```

#### Pattern 4: Incremental Migration (Large Datasets)

```rust
async fn incremental_migration(
    source: Arc<dyn StorageBackend>,
    target: Arc<dyn StorageBackend>,
    batch_size: usize,
) -> Result<()> {
    let keys = source.list_keys(None).await?;

    for chunk in keys.chunks(batch_size) {
        let mut batch_ops = Vec::new();

        for key in chunk {
            if let Some(value) = source.get(key).await? {
                batch_ops.push(BatchOperation::Set {
                    key: key.clone(),
                    value,
                });
            }
        }

        target.batch(batch_ops).await?;
        println!("Migrated batch of {} keys", chunk.len());
    }

    Ok(())
}
```

---

### Testing Export/Import

#### Roundtrip Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_storage::backends::sqlite::*;
    use llmspell_storage::export_import::*;

    #[tokio::test]
    async fn test_export_import_roundtrip() {
        // Create source database with test data
        let source_db = create_test_backend("source.db").await;
        populate_test_data(&source_db).await;

        // Export
        let exporter = SqliteExporter::new(Arc::clone(&source_db));
        let export_data = exporter.export_all().await.unwrap();

        // Serialize and deserialize
        let json = serde_json::to_string_pretty(&export_data).unwrap();
        let parsed: ExportFormat = serde_json::from_str(&json).unwrap();

        // Import to target
        let target_db = create_test_backend("target.db").await;
        let importer = SqliteImporter::new(target_db);

        let temp_file = "/tmp/test-export.json";
        std::fs::write(temp_file, json).unwrap();
        let stats = importer.import_from_file(temp_file).await.unwrap();

        // Verify counts
        assert_eq!(stats.vectors, 10);
        assert_eq!(stats.sessions, 2);
    }

    #[tokio::test]
    async fn test_empty_database_export() {
        let backend = create_test_backend("empty.db").await;
        let exporter = SqliteExporter::new(backend);

        let export_data = exporter.export_all().await.unwrap();

        // Should export successfully with zero records
        assert_eq!(export_data.data.vector_embeddings.len(), 0);
        assert!(export_data.data.knowledge_graph.is_none());
    }
}
```

---

### Best Practices

#### Pre-Migration Validation

```rust
async fn validate_before_migration(
    backend: Arc<dyn StorageBackend>
) -> Result<()> {
    // 1. Check backend health
    backend.health_check().await?;

    // 2. Verify migrations applied
    let applied_migrations = backend.list_migrations().await?;
    println!("Applied migrations: {:?}", applied_migrations);

    // 3. Estimate export size
    let stats = backend.stats().await?;
    println!("Estimated export size: {} MB", stats.total_size_bytes / 1_000_000);

    Ok(())
}
```

#### Post-Migration Verification

```rust
async fn verify_migration(
    export_file: &str,
    target: Arc<dyn StorageBackend>,
    expected_stats: &ImportStats,
) -> Result<()> {
    // 1. Re-export from target
    let exporter = SqliteExporter::new(target);
    let verify_export = exporter.export_all().await?;

    // 2. Compare with original export
    let original: ExportFormat = serde_json::from_str(
        &std::fs::read_to_string(export_file)?
    )?;

    // 3. Verify data equality (ignore timestamps)
    let original_data = serde_json::to_value(&original.data)?;
    let verify_data = serde_json::to_value(&verify_export.data)?;

    assert_eq!(original_data, verify_data, "Migration data mismatch");

    println!("✅ Migration verified successfully");
    Ok(())
}
```

#### Error Handling

```rust
async fn safe_import_with_rollback(
    importer: SqliteImporter,
    file_path: &str,
) -> Result<ImportStats> {
    match importer.import_from_file(file_path).await {
        Ok(stats) => {
            println!("✅ Import successful: {} records", stats.total());
            Ok(stats)
        }
        Err(e) => {
            eprintln!("❌ Import failed: {}", e);
            eprintln!("Transaction automatically rolled back");
            Err(e)
        }
    }
}
```

---

### Related Patterns

#### Backup Before Migration

```rust
async fn backup_before_migration(
    backend: Arc<dyn StorageBackend>,
    backup_path: &str,
) -> Result<()> {
    let exporter = SqliteExporter::new(backend);
    let export_data = exporter.export_all().await?;

    let json = serde_json::to_string_pretty(&export_data)?;
    std::fs::write(backup_path, json)?;

    println!("Backup created: {}", backup_path);
    Ok(())
}
```

#### Selective Data Export

```rust
async fn export_sessions_only(
    backend: Arc<SqliteBackend>,
) -> Result<ExportFormat> {
    let exporter = SqliteExporter::new(backend);
    let mut export_data = exporter.export_all().await?;

    // Clear all data except sessions and artifacts
    export_data.data.vector_embeddings.clear();
    export_data.data.knowledge_graph = None;
    export_data.data.procedural_memory.clear();
    export_data.data.agent_state.clear();
    export_data.data.event_log.clear();

    Ok(export_data)
}
```

---

## Related Documentation

- **Detailed API**: [llmspell-storage.md](llmspell-storage.md)
- **Other Guides**:
  - [rag-pipeline.md](rag-pipeline.md) - RAG integration
  - [memory-backends.md](memory-backends.md) - Memory system
  - [core-traits.md](core-traits.md) - Foundation traits
- **Technical Docs**:
  - [../../technical/postgresql-schema.md](../../technical/postgresql-schema.md) - PostgreSQL backend
  - [../../technical/postgresql-performance.md](../../technical/postgresql-performance.md) - Performance tuning

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
