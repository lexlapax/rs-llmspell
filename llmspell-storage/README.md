# llmspell-storage

Storage backends and persistence for rs-llmspell, including vector storage for RAG applications.

## Features

### Key-Value Storage
- **MemoryBackend**: In-memory storage for testing and temporary data
- **SledBackend**: Embedded database for persistent local storage
- Trait-based storage abstraction for easy backend switching

### Vector Storage
- **HNSW (Hierarchical Navigable Small World)**: High-performance vector search with <10ms retrieval for 1M vectors
- **Dimension Router**: Multi-dimension support for different embedding models (256-4096 dimensions)
- **Metadata Index**: Efficient filtering with inverted indices
- **Multi-tenant isolation**: Namespace separation for different users/sessions
- **Matryoshka representation**: Dimension reduction for storage optimization

## Usage

### Key-Value Storage
```rust
use llmspell_storage::{MemoryBackend, StorageBackend};
use serde_json::json;

let backend = MemoryBackend::new();
let value = json!({"name": "Alice"});
backend.set("user:123", serde_json::to_vec(&value)?).await?;
```

### Vector Storage
```rust
use llmspell_storage::{VectorEntry, VectorQuery, HNSWVectorStorage, HNSWConfig};
use llmspell_state_traits::StateScope;

// Create HNSW storage with configuration
let config = HNSWConfig::balanced();
let storage = HNSWVectorStorage::new(1536, config);

// Insert vectors with metadata and scope
let entry = VectorEntry::new("doc-1".to_string(), vec![0.1, 0.2, 0.3])
    .with_scope(StateScope::Custom("tenant:123".to_string()))
    .with_metadata(metadata);

let ids = storage.insert(vec![entry]).await?;

// Search with tenant isolation
let query = VectorQuery::new(vec![0.1, 0.2, 0.3], 10)
    .with_scope(StateScope::Custom("tenant:123".to_string()))
    .with_threshold(0.8);

let results = storage.search(&query).await?;
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

### HNSW Vector Storage
- Insert: O(log n), <10ms for 1M vectors
- Search: O(log n), <10ms for 1M vectors  
- Memory: ~100-200 bytes per vector + metadata
- Accuracy: >95% recall at top-10

## Dependencies
- `llmspell-core` - Core traits and types
- `llmspell-state-traits` - State scope definitions
- `sled` - Embedded key-value database
- `dashmap` - Concurrent hashmaps
- `parking_lot` - Synchronization primitives
- `uuid` - Unique identifiers for vectors