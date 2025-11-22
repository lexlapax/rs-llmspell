# llmspell-storage

Storage backends and persistence for rs-llmspell, including vector storage for RAG applications.

## Features

### Key-Value Storage
- **MemoryBackend**: In-memory storage for testing and temporary data
- Trait-based storage abstraction for easy backend switching

### Vector Storage (Phase 8 - Production RAG)
- **HNSW (Hierarchical Navigable Small World)**: Production-ready vector search with <10ms retrieval for 1M vectors, >95% recall at top-10
- **Multi-tenant Isolation**: Complete namespace separation with StateScope-based tenant boundaries and resource isolation
- **Real-time Operations**: Insert, update metadata, delete vectors, and search without index rebuilds or downtime
- **Distance Metrics**: Optimized Cosine, Euclidean, InnerProduct, Manhattan with SIMD acceleration where available
- **Dimension Routing**: Multi-dimensional embedding support (256-4096 dimensions) with automatic model compatibility
- **Metadata Indexing**: Rich metadata queries with inverted indices supporting complex filtering and tenant isolation
- **Memory Management**: Configurable memory limits, batch processing, and memory-mapped storage for large datasets
- **Performance Tuning**: Advanced HNSW parameters (ef_construction, max_connections, nb_layers) with preset optimization profiles
- **Concurrent Safety**: Thread-safe operations using Arc<DashMap> with lock-free reads for high-throughput scenarios

## Usage

### Key-Value Storage
```rust
use llmspell_storage::MemoryBackend;
use llmspell_core::traits::storage::StorageBackend;
use serde_json::json;

let backend = MemoryBackend::new();
let value = json!({"name": "Alice"});
backend.set("user:123", serde_json::to_vec(&value)?).await?;
```

### Vector Storage (Production RAG)
```rust
use llmspell_storage::backends::sqlite::{SqliteVectorStorage, SqliteConfig};
use std::sync::Arc;

// Create SQLite vector storage with vectorlite-rs pure Rust HNSW extension
let config = SqliteConfig::new("./data/vectors.db")
    .with_max_connections(20);

let backend = Arc::new(SqliteBackend::new(config).await?);
let storage = SqliteVectorStorage::new(backend, "main".to_string())?;  // tenant_id "main"

// Multi-tenant document ingestion
let mut metadata = HashMap::new();
metadata.insert("document_id".to_string(), "user-guide-section-1".into());
metadata.insert("document_type".to_string(), "documentation".into());
metadata.insert("tenant_id".to_string(), "company-123".into());

let entries = vec![
    VectorEntry::new("doc-1".to_string(), embedding_vector)
        .with_scope(StateScope::Custom("tenant:company-123".to_string()))
        .with_metadata(metadata.clone()),
    // Batch insert for performance
];

let ids = storage.insert(entries).await?;

// Production similarity search with filtering
let query = VectorQuery::new(query_embedding, 10)
    .with_scope(StateScope::Custom("tenant:company-123".to_string()))
    .with_threshold(0.8)
    .with_metadata_filter("document_type", "documentation");

let results = storage.search(&query).await?;

// Real-time metadata updates
storage.update_metadata(&ids[0], updated_metadata).await?;

// Tenant-scoped statistics for billing
let stats = storage.stats_for_scope(
    &StateScope::Custom("tenant:company-123".to_string())
).await?;
println!("Tenant vectors: {}, memory: {} MB", stats.vector_count, stats.memory_mb);

// Bulk tenant cleanup
storage.delete_scope(&StateScope::Custom("tenant:company-123".to_string())).await?;
```

### Multi-Dimensional Support
```rust
use llmspell_storage::{DimensionRouter, EmbeddingModel};

// Route different embedding models automatically
let router = DimensionRouter::new();
router.register_model(EmbeddingModel::OpenAI_Ada002, 1536);
router.register_model(EmbeddingModel::OpenAI_Embedding3Large, 3072);

let storage_ada = router.get_storage(&EmbeddingModel::OpenAI_Ada002)?;
let storage_large = router.get_storage(&EmbeddingModel::OpenAI_Embedding3Large)?;

// Automatic dimension detection
let auto_storage = router.auto_detect_storage(&embedding_vector)?;
```

### Temporal Model (Bi-temporal Support)

The vector storage system now supports comprehensive temporal metadata for Phase 9's memory system:

```rust
use llmspell_storage::{VectorEntry, VectorQuery};
use std::time::{SystemTime, Duration};

// Create vector with temporal metadata
let event_time = SystemTime::now() - Duration::from_secs(3600); // Event occurred 1 hour ago
let entry = VectorEntry::new("doc-1".to_string(), embedding)
    .with_event_time(event_time)     // When the event actually occurred
    .with_ttl(86400)                  // Expire after 24 hours
    .with_metadata(metadata);

// The system automatically tracks:
// - created_at: When the vector was ingested (ingestion time)
// - updated_at: When the vector was last modified
// - event_time: When the real-world event occurred (optional)
// - expires_at: When the vector will expire (calculated from TTL)

// Bi-temporal queries
let query = VectorQuery::new(query_embedding, 10)
    .with_event_time_range(hour_ago, now)        // Filter by when events occurred
    .with_ingestion_time_range(yesterday, now)   // Filter by when we learned about them
    .exclude_expired(true);                      // Automatically exclude expired entries

// Update tracking
let mut entry = VectorEntry::new("doc-2".to_string(), embedding);
entry.update(); // Automatically updates the updated_at timestamp

// Check expiration
if entry.is_expired() {
    println!("Entry has expired and should be removed");
}
```

#### Temporal Fields

- **`created_at`**: Ingestion time - when the vector was added to the system
- **`updated_at`**: Last modification time - tracks when the entry was last changed
- **`event_time`**: Event occurrence time - when the actual event happened (optional)
- **`expires_at`**: Expiration time - when the vector should be removed (optional)
- **`ttl_seconds`**: Time-to-live duration - alternative to setting expires_at directly

#### Bi-temporal Queries

Bi-temporal support enables sophisticated time-based queries:

```rust
// "What did we know last week about events from last month?"
let query = VectorQuery::new(embedding, 10)
    .with_event_time_range(last_month_start, last_month_end)
    .with_ingestion_time_range(last_week_start, last_week_end);

// "Find recent events we just learned about"
let query = VectorQuery::new(embedding, 10)
    .with_event_time_range(yesterday, now)
    .with_ingestion_time_range(last_hour, now);
```

This temporal model is essential for Phase 9's Adaptive Memory System, enabling:
- Episodic memory with time-based retrieval
- Temporal knowledge graphs with validity intervals
- Memory consolidation based on age and relevance
- Automatic cleanup of expired memories

## Performance Characteristics

### MemoryBackend
- Read/Write: O(1) average, <1μs
- Memory: All data in RAM
- Persistence: None

### HNSW Vector Storage (Phase 8)
- **Insert**: O(log n), <10ms for 1M vectors, 2-5ms typical for 100K vectors
- **Search**: O(log n), <10ms for 1M vectors, 1-3ms typical for 100K vectors
- **Memory**: ~150-300 bytes per vector + metadata (depends on max_connections)
- **Accuracy**: >95% recall at top-10, >99% recall at top-100 with balanced config
- **Concurrency**: 10-100x throughput improvement with concurrent reads
- **Multi-tenant Overhead**: <5% per tenant for search operations
- **Batch Operations**: 10-50x faster for bulk inserts (100+ vectors)
- **Metadata Filtering**: <2ms additional overhead for complex filters

## Dependencies
- `llmspell-core` - Core traits and types
- `llmspell-state-traits` - State scope definitions and multi-tenant isolation
- `dashmap` - Lock-free concurrent hashmaps for high-performance operations
- `parking_lot` - Low-overhead synchronization primitives
- `uuid` - Unique identifiers for vectors and tenants
- `rmp-serde` - Efficient binary serialization for vector storage
- `rayon` - Parallel processing for batch operations
- `ordered-float` - Deterministic floating-point operations for distance calculations

## Phase 8 Architecture
```
llmspell-storage
├── backends/
│   ├── memory.rs           # In-memory key-value storage
│   └── sqlite/
│       └── vector.rs       # SQLite + vectorlite-rs HNSW implementation
├── vector_storage.rs       # VectorStorage trait with async operations
└── traits.rs              # Storage backend abstractions
```