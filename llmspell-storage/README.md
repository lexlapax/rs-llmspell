# llmspell-storage

Storage backends and persistence for rs-llmspell, including vector storage for RAG applications.

## Features

### Key-Value Storage
- **MemoryBackend**: In-memory storage for testing and temporary data
- **SledBackend**: Embedded database for persistent local storage
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
use llmspell_storage::{MemoryBackend, StorageBackend};
use serde_json::json;

let backend = MemoryBackend::new();
let value = json!({"name": "Alice"});
backend.set("user:123", serde_json::to_vec(&value)?).await?;
```

### Vector Storage (Production RAG)
```rust
use llmspell_storage::{
    VectorEntry, VectorQuery, HNSWVectorStorage, HNSWConfig, DistanceMetric
};
use llmspell_state_traits::StateScope;
use std::collections::HashMap;

// Create production-optimized HNSW storage
let config = HNSWConfig::production()  // Preset for production workloads
    .with_distance_metric(DistanceMetric::Cosine)
    .with_max_connections(16)
    .with_ef_construction(200);

let storage = HNSWVectorStorage::new(1536, config);  // OpenAI ada-002 dimensions

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

## Performance Characteristics

### MemoryBackend
- Read/Write: O(1) average, <1μs
- Memory: All data in RAM
- Persistence: None

### SledBackend  
- Read/Write: O(log n), <100μs/1ms
- Memory: Configurable cache
- Persistence: ACID compliant

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
- `sled` - Embedded key-value database for persistence
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
│   ├── sled_backend.rs     # Persistent key-value storage
│   └── vector/
│       ├── hnsw.rs         # HNSW implementation with multi-tenant support
│       ├── dimension_router.rs # Multi-dimensional embedding routing
│       └── metadata_index.rs   # Inverted index for metadata filtering
├── vector_storage.rs       # VectorStorage trait with async operations
└── traits.rs              # Storage backend abstractions
```