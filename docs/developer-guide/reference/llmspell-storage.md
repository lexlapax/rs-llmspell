# llmspell-storage

## Purpose

Storage abstraction layer providing unified interface for data persistence with multiple backend implementations including memory, embedded database (Sled), and HNSW-based vector storage for RAG operations. This crate is the foundation for Phase 8's vector search capabilities, enabling efficient similarity search at scale with multi-tenant isolation.

## Core Concepts

- **Unified Storage Interface**: Single `StorageBackend` trait for all storage types
- **Vector Storage**: HNSW (Hierarchical Navigable Small World) graphs for fast similarity search
- **Dimension Routing**: Automatic routing based on embedding dimensions (384, 768, 1536, 3072)
- **Metadata Filtering**: Efficient filtering on vector metadata during search
- **Multi-Tenant Isolation**: Tenant-specific collections with data isolation
- **Persistence Options**: In-memory for testing, Sled for embedded, vector storage for RAG
- **Async-First Design**: All operations are async for non-blocking IO
- **Collection Management**: Organize vectors into named collections
- **Bi-temporal Support**: Track both event time and ingestion time for sophisticated temporal queries
- **TTL Mechanism**: Automatic expiration of vectors based on time-to-live settings

## Primary Traits/Structs

### StorageBackend Trait

**Purpose**: Unified interface for all storage implementations enabling backend-agnostic data persistence.

**When to implement**: Creating custom storage backends for specific databases or cloud storage.

**Required methods**:
- `get()` - Retrieve value by key
- `set()` - Store key-value pair
- `delete()` - Remove key
- `exists()` - Check key existence
- `list_keys()` - List all keys with optional prefix
- `clear()` - Remove all data

```rust
use async_trait::async_trait;
use llmspell_storage::{StorageResult, StorageError};

#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Get value by key
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>>;
    
    /// Set key-value pair
    async fn set(&self, key: &str, value: Vec<u8>) -> StorageResult<()>;
    
    /// Delete key
    async fn delete(&self, key: &str) -> StorageResult<()>;
    
    /// Check if key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;
    
    /// List keys with optional prefix filter
    async fn list_keys(&self, prefix: Option<&str>) -> StorageResult<Vec<String>>;
    
    /// Clear all data
    async fn clear(&self) -> StorageResult<()>;
    
    /// Get storage statistics
    async fn stats(&self) -> StorageResult<StorageStats> {
        Ok(StorageStats::default())
    }
    
    /// Perform batch operations
    async fn batch(&self, ops: Vec<BatchOperation>) -> StorageResult<Vec<BatchResult>> {
        // Default implementation executes serially
        let mut results = Vec::new();
        for op in ops {
            let result = match op {
                BatchOperation::Set { key, value } => {
                    self.set(&key, value).await.map(|_| BatchResult::Success)
                }
                BatchOperation::Delete { key } => {
                    self.delete(&key).await.map(|_| BatchResult::Success)
                }
            };
            results.push(result.unwrap_or(BatchResult::Failed));
        }
        Ok(results)
    }
}
```

### VectorStorage Trait

**Purpose**: Specialized storage for vector embeddings with similarity search capabilities.

**When to implement**: Creating custom vector databases or search algorithms.

```rust
use llmspell_storage::vector::{
    VectorStorage, VectorEntry, VectorSearchOptions, 
    VectorSearchResult, CollectionConfig
};

#[async_trait]
pub trait VectorStorage: Send + Sync {
    /// Store vector with metadata
    async fn store(
        &self,
        collection: &str,
        entry: VectorEntry,
    ) -> StorageResult<String>; // Returns vector ID
    
    /// Search for similar vectors
    async fn search(
        &self,
        collection: &str,
        query: &[f32],
        options: VectorSearchOptions,
    ) -> StorageResult<Vec<VectorSearchResult>>;
    
    /// Get vector by ID
    async fn get(
        &self,
        collection: &str,
        id: &str,
    ) -> StorageResult<Option<VectorEntry>>;
    
    /// Delete vector
    async fn delete(
        &self,
        collection: &str,
        id: &str,
    ) -> StorageResult<()>;
    
    /// Create collection with configuration
    async fn create_collection(
        &self,
        name: &str,
        config: CollectionConfig,
    ) -> StorageResult<()>;
    
    /// Delete entire collection
    async fn delete_collection(&self, name: &str) -> StorageResult<()>;
    
    /// List collections
    async fn list_collections(&self) -> StorageResult<Vec<String>>;
    
    /// Get collection statistics
    async fn collection_stats(&self, name: &str) -> StorageResult<CollectionStats>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEntry {
    /// Unique identifier
    pub id: String,
    /// Embedding vector
    pub embedding: Vec<f32>,
    /// Metadata for filtering and context
    pub metadata: HashMap<String, Value>,
    /// Tenant/user/session binding
    pub scope: StateScope,
    /// Creation timestamp (ingestion time)
    pub created_at: SystemTime,
    /// Last update timestamp
    pub updated_at: SystemTime,
    /// Event time - when the event actually occurred (optional)
    pub event_time: Option<SystemTime>,
    /// Expiration time (optional)
    pub expires_at: Option<SystemTime>,
    /// Time-to-live in seconds (optional)
    pub ttl_seconds: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct VectorQuery {
    /// Query vector
    pub vector: Vec<f32>,
    /// Number of results to return
    pub k: usize,
    /// Optional metadata filters
    pub filter: Option<HashMap<String, Value>>,
    /// Optional scope restriction
    pub scope: Option<StateScope>,
    /// Similarity threshold (0.0 to 1.0)
    pub threshold: Option<f32>,
    /// Include metadata in results
    pub include_metadata: bool,
    /// Filter by event time range (bi-temporal query)
    pub event_time_range: Option<(SystemTime, SystemTime)>,
    /// Filter by ingestion time range (bi-temporal query)
    pub ingestion_time_range: Option<(SystemTime, SystemTime)>,
    /// Exclude expired entries
    pub exclude_expired: bool,
}

#[derive(Debug, Clone)]
pub struct VectorSearchResult {
    pub id: String,
    pub score: f32,
    pub vector: Option<Vec<f32>>,
    pub metadata: HashMap<String, Value>,
    pub payload: Option<Vec<u8>>,
}
```

### HNSWVectorStorage

**Purpose**: High-performance vector storage using HNSW algorithm for approximate nearest neighbor search.

```rust
use llmspell_storage::vector::HNSWVectorStorage;

pub struct HNSWVectorStorage {
    indices: Arc<RwLock<HashMap<String, HNSWIndex>>>,
    dimension_router: DimensionRouter,
    metadata_index: MetadataIndex,
    config: HNSWConfig,
}

impl HNSWVectorStorage {
    pub fn new(config: HNSWConfig) -> Self {
        Self {
            indices: Arc::new(RwLock::new(HashMap::new())),
            dimension_router: DimensionRouter::new(),
            metadata_index: MetadataIndex::new(),
            config,
        }
    }
    
    /// Optimized search with early termination
    pub async fn search_with_optimization(
        &self,
        collection: &str,
        query: &[f32],
        options: VectorSearchOptions,
    ) -> StorageResult<Vec<VectorSearchResult>> {
        // Route to correct index based on dimension
        let dimension = query.len();
        let index = self.dimension_router.get_index(collection, dimension)?;
        
        // Apply metadata pre-filtering
        let candidate_ids = if let Some(filter) = &options.metadata_filter {
            self.metadata_index.filter(collection, filter).await?
        } else {
            None
        };
        
        // HNSW search with pruning
        let results = index.search(
            query,
            options.limit * 2, // Over-fetch for filtering
            candidate_ids.as_deref(),
        )?;
        
        // Post-process results
        let mut final_results = Vec::new();
        for result in results {
            if let Some(threshold) = options.threshold {
                if result.distance > threshold {
                    continue;
                }
            }
            
            final_results.push(VectorSearchResult {
                id: result.id,
                score: 1.0 - result.distance, // Convert distance to similarity
                vector: if options.include_vectors {
                    Some(result.vector)
                } else {
                    None
                },
                metadata: result.metadata,
                payload: result.payload,
            });
            
            if final_results.len() >= options.limit {
                break;
            }
        }
        
        Ok(final_results)
    }
}

#[derive(Debug, Clone)]
pub struct HNSWConfig {
    pub m: usize,              // Number of bi-directional links
    pub ef_construction: usize, // Size of dynamic candidate list
    pub ef_search: usize,       // Size of dynamic list for search
    pub max_elements: usize,    // Maximum number of elements
    pub seed: Option<u64>,      // Random seed for reproducibility
    pub distance_type: DistanceType,
}

impl Default for HNSWConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            max_elements: 1_000_000,
            seed: None,
            distance_type: DistanceType::Cosine,
        }
    }
}
```

### Backend Implementations

**Purpose**: Concrete storage implementations for different use cases.

```rust
use llmspell_storage::{MemoryBackend, SledBackend};

/// In-memory storage for testing
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryBackend {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

/// Embedded database storage
pub struct SledBackend {
    db: sled::Db,
    config: SledConfig,
}

impl SledBackend {
    pub fn new_with_path(path: impl AsRef<Path>) -> StorageResult<Self> {
        let config = sled::Config::new()
            .path(path)
            .cache_capacity(1024 * 1024 * 1024) // 1GB cache
            .flush_every_ms(Some(1000));
        
        let db = config.open()?;
        
        Ok(Self {
            db,
            config: SledConfig::default(),
        })
    }
    
    pub fn with_config(config: SledConfig) -> StorageResult<Self> {
        let db = config.to_sled_config().open()?;
        Ok(Self { db, config })
    }
}
```

## Usage Patterns

### Basic Key-Value Storage

**When to use**: Simple data persistence needs without complex queries.

**Benefits**: Simple API, works with any backend.

**Example**:
```rust
use llmspell_storage::{StorageBackend, MemoryBackend};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct UserData {
    name: String,
    email: String,
    preferences: HashMap<String, String>,
}

async fn store_user_data(
    backend: &dyn StorageBackend,
    user_id: &str,
    data: UserData,
) -> Result<(), Error> {
    let key = format!("user:{}", user_id);
    let value = serde_json::to_vec(&data)?;
    backend.set(&key, value).await?;
    Ok(())
}

async fn get_user_data(
    backend: &dyn StorageBackend,
    user_id: &str,
) -> Result<Option<UserData>, Error> {
    let key = format!("user:{}", user_id);
    
    match backend.get(&key).await? {
        Some(bytes) => {
            let data = serde_json::from_slice(&bytes)?;
            Ok(Some(data))
        }
        None => Ok(None),
    }
}
```

### Vector Storage for RAG

**When to use**: Implementing retrieval-augmented generation with similarity search.

**Benefits**: Fast nearest neighbor search, metadata filtering, multi-tenant support.

**Example**:
```rust
use llmspell_storage::vector::{
    HNSWVectorStorage, VectorEntry, VectorSearchOptions,
    CollectionConfig, MetadataFilter
};

async fn setup_rag_storage() -> Result<HNSWVectorStorage, Error> {
    let storage = HNSWVectorStorage::new(HNSWConfig {
        m: 24,
        ef_construction: 400,
        ef_search: 100,
        ..Default::default()
    });
    
    // Create collection for documents
    storage.create_collection("documents", CollectionConfig {
        dimension: 1536, // OpenAI embedding dimension
        distance_type: DistanceType::Cosine,
        tenant_isolation: true,
    }).await?;
    
    Ok(storage)
}

async fn ingest_document(
    storage: &HNSWVectorStorage,
    doc_id: &str,
    content: &str,
    embedding: Vec<f32>,
    tenant_id: &str,
    event_time: Option<SystemTime>,
    ttl_hours: Option<u64>,
) -> Result<String, Error> {
    // Create entry with temporal metadata
    let mut entry = VectorEntry::new(doc_id.to_string(), embedding)
        .with_scope(StateScope::Custom(format!("tenant:{}", tenant_id)))
        .with_metadata(hashmap! {
            "content" => json!(content),
            "tenant_id" => json!(tenant_id),
        });
    
    // Set event time if provided (when the document was created)
    if let Some(event_time) = event_time {
        entry = entry.with_event_time(event_time);
    }
    
    // Set TTL if provided (auto-expire old documents)
    if let Some(hours) = ttl_hours {
        entry = entry.with_ttl(hours * 3600); // Convert hours to seconds
    }
    
    storage.insert(vec![entry]).await?.into_iter().next().ok_or(Error::NotFound)
}

async fn search_documents(
    storage: &HNSWVectorStorage,
    query_embedding: Vec<f32>,
    tenant_id: &str,
    limit: usize,
    exclude_expired: bool,
) -> Result<Vec<SearchResult>, Error> {
    let query = VectorQuery::new(query_embedding, limit)
        .with_scope(StateScope::Custom(format!("tenant:{}", tenant_id)))
        .with_threshold(0.7) // Minimum similarity
        .exclude_expired(exclude_expired); // Filter out expired documents
    
    let results = storage.search(&query).await?;
    
    Ok(results.into_iter().map(|r| SearchResult {
        id: r.id,
        score: r.score,
        content: String::from_utf8_lossy(&r.payload.unwrap_or_default()).to_string(),
    }).collect())
}
```

### Multi-Tenant Data Isolation

**When to use**: SaaS applications requiring strict data isolation between tenants.

**Benefits**: Secure isolation, efficient filtering, tenant-specific operations.

**Example**:
```rust
use llmspell_storage::StorageBackend;

pub struct MultiTenantStorage {
    backend: Arc<dyn StorageBackend>,
}

impl MultiTenantStorage {
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }
    
    fn tenant_key(&self, tenant_id: &str, key: &str) -> String {
        format!("tenant:{}:{}", tenant_id, key)
    }
    
    pub async fn get(&self, tenant_id: &str, key: &str) -> StorageResult<Option<Vec<u8>>> {
        self.backend.get(&self.tenant_key(tenant_id, key)).await
    }
    
    pub async fn set(&self, tenant_id: &str, key: &str, value: Vec<u8>) -> StorageResult<()> {
        self.backend.set(&self.tenant_key(tenant_id, key), value).await
    }
    
    pub async fn list_tenant_keys(&self, tenant_id: &str) -> StorageResult<Vec<String>> {
        let prefix = format!("tenant:{}:", tenant_id);
        let keys = self.backend.list_keys(Some(&prefix)).await?;
        
        // Strip tenant prefix from keys
        Ok(keys.into_iter()
            .map(|k| k.strip_prefix(&prefix).unwrap_or(&k).to_string())
            .collect())
    }
    
    pub async fn delete_tenant(&self, tenant_id: &str) -> StorageResult<()> {
        let prefix = format!("tenant:{}:", tenant_id);
        let keys = self.backend.list_keys(Some(&prefix)).await?;
        
        for key in keys {
            self.backend.delete(&key).await?;
        }
        
        Ok(())
    }
}
```

### Temporal Support and Bi-temporal Queries

**When to use**: Time-based retrieval, memory systems, event sourcing, audit trails.

**Benefits**: Distinguish between when events occurred vs when discovered, automatic expiration, temporal analytics.

**Example**:
```rust
use llmspell_storage::{VectorEntry, VectorQuery};
use std::time::{SystemTime, Duration};

// Create entry with temporal metadata
async fn ingest_with_temporal_metadata(
    storage: &HNSWVectorStorage,
    content: &str,
    embedding: Vec<f32>,
    event_time: SystemTime,  // When the event actually happened
    ttl_hours: u64,          // How long to keep this data
) -> Result<String, Error> {
    let entry = VectorEntry::new(uuid::Uuid::new_v4().to_string(), embedding)
        .with_event_time(event_time)      // Set when event occurred
        .with_ttl(ttl_hours * 3600)       // Auto-expire after N hours
        .with_metadata(hashmap! {
            "content" => json!(content),
        });
    
    // created_at and updated_at are set automatically
    // expires_at is calculated from TTL
    
    storage.insert(vec![entry]).await?.into_iter().next().ok_or(Error::NotFound)
}

// Bi-temporal query example
async fn temporal_search(
    storage: &HNSWVectorStorage,
    query_embedding: Vec<f32>,
) -> Result<Vec<VectorResult>, Error> {
    let now = SystemTime::now();
    let yesterday = now - Duration::from_secs(86400);
    let last_week = now - Duration::from_secs(7 * 86400);
    let last_hour = now - Duration::from_secs(3600);
    
    // "What did we know last hour about events from yesterday?"
    let query = VectorQuery::new(query_embedding, 10)
        .with_event_time_range((yesterday, now))        // Events that happened yesterday
        .with_ingestion_time_range((last_hour, now))    // That we learned about in the last hour
        .exclude_expired(true);                         // Don't return expired entries
    
    storage.search(&query).await
}

// Check and update temporal fields
async fn manage_temporal_data(
    storage: &HNSWVectorStorage,
    entry_id: &str,
) -> Result<(), Error> {
    // Get entry
    if let Some(mut entry) = storage.get(entry_id).await? {
        // Check if expired
        if entry.is_expired() {
            println!("Entry {} has expired", entry_id);
            storage.delete(&[entry_id.to_string()]).await?;
        } else {
            // Update entry (automatically updates updated_at)
            entry.update();
            entry.metadata.insert("last_accessed".to_string(), json!(SystemTime::now()));
            storage.update_metadata(entry_id, entry.metadata).await?;
        }
    }
    Ok(())
}
```

**Temporal Fields Reference**:
- `created_at`: When the vector was ingested (set automatically)
- `updated_at`: When the vector was last modified (updated automatically)
- `event_time`: When the real-world event occurred (optional, set by user)
- `expires_at`: When the vector expires (calculated from TTL)
- `ttl_seconds`: Time-to-live duration in seconds

**Bi-temporal Query Use Cases**:
1. **Audit Trail**: "Show all changes made last week to documents from last month"
2. **Knowledge Evolution**: "What did we know at time X about topic Y?"
3. **Memory Consolidation**: Find old memories to compress or archive
4. **Compliance**: Track when sensitive data was learned vs when it occurred
5. **Debugging**: Understand system state at specific points in time

## Integration Examples

### With RAG System

```rust
use llmspell_storage::vector::HNSWVectorStorage;
use llmspell_rag::{RAGPipeline, ChunkingStrategy};

pub struct DocumentRAG {
    vector_storage: Arc<HNSWVectorStorage>,
    embedder: Arc<dyn Embedder>,
}

impl DocumentRAG {
    pub async fn ingest_document(
        &self,
        document: Document,
        tenant_id: &str,
    ) -> Result<(), Error> {
        // Chunk document
        let chunks = ChunkingStrategy::Sliding {
            size: 500,
            overlap: 50,
        }.chunk(&document.content);
        
        // Generate embeddings
        let embeddings = self.embedder.embed_batch(&chunks).await?;
        
        // Store in vector storage with temporal metadata
        for (chunk, embedding) in chunks.iter().zip(embeddings) {
            let entry = VectorEntry::new(Uuid::new_v4().to_string(), embedding)
                .with_scope(StateScope::Custom(format!("tenant:{}", tenant_id)))
                .with_event_time(document.created_at) // When document was created
                .with_ttl(30 * 24 * 3600) // Keep for 30 days
                .with_metadata(hashmap! {
                    "document_id" => json!(document.id),
                    "tenant_id" => json!(tenant_id),
                    "chunk_index" => json!(chunk.index),
                });
            
            self.vector_storage.insert(vec![entry]).await?;
        }
        
        Ok(())
    }
}
```

### With State Persistence

```rust
use llmspell_storage::StorageBackend;
use llmspell_state_persistence::StateManager;

pub struct PersistentStateManager {
    storage: Arc<dyn StorageBackend>,
}

impl StateManager for PersistentStateManager {
    async fn get(&self, key: &str) -> Result<Option<Value>, StateError> {
        match self.storage.get(key).await? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
    
    async fn set(&self, key: &str, value: Value) -> Result<(), StateError> {
        let bytes = serde_json::to_vec(&value)?;
        self.storage.set(key, bytes).await?;
        Ok(())
    }
}
```

## PostgreSQL Backend (Phase 13b.16)

**Purpose**: Production-grade storage backend using PostgreSQL 18 + VectorChord for HNSW vector similarity search, multi-tenancy, and bi-temporal graph storage.

**When to use**: Production deployments requiring persistence, ACID guarantees, complex queries, multi-tenant isolation, and high-performance vector search.

### PostgreSQLBackend

**Location**: `llmspell-storage/src/postgres/backend.rs`

**Purpose**: Implements `StorageBackend` trait for PostgreSQL with connection pooling, migrations, and RLS enforcement.

```rust
use llmspell_storage::postgres::PostgreSQLBackend;
use sqlx::PgPool;

pub struct PostgreSQLBackend {
    pool: PgPool,
    tenant_id: Option<String>,
    enforce_rls: bool,
}

impl PostgreSQLBackend {
    /// Create from database URL
    pub async fn new(database_url: &str) -> StorageResult<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self {
            pool,
            tenant_id: None,
            enforce_rls: false,
        })
    }

    /// Create with explicit pool
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            tenant_id: None,
            enforce_rls: false,
        }
    }

    /// Enable Row-Level Security with tenant_id
    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self.enforce_rls = true;
        self
    }

    /// Run database migrations (V1-V15)
    pub async fn migrate(&self) -> StorageResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
        Ok(())
    }

    /// Execute query with automatic RLS tenant injection
    async fn execute_with_tenant<T, F>(&self, f: F) -> StorageResult<T>
    where
        F: FnOnce(&PgPool) -> BoxFuture<StorageResult<T>>,
    {
        if self.enforce_rls {
            // Set session tenant_id for RLS
            let tenant_id = self.tenant_id.as_ref()
                .ok_or_else(|| StorageError::Config("Tenant ID required for RLS".into()))?;

            sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
                .bind(tenant_id)
                .execute(&self.pool)
                .await?;
        }

        f(&self.pool).await
    }
}
```

### Storage Component Implementations

**Purpose**: 10 specialized storage components backed by PostgreSQL tables with HNSW indexes, RLS, and bi-temporal support.

**Component Matrix**:

| Component | Table(s) | HNSW Index | RLS | Bi-Temporal | Purpose |
|-----------|----------|------------|-----|-------------|---------|
| **VectorEmbeddings** | `vector_embeddings_{384,768,1536,3072}` | ✅ (m=16) | ✅ | ✅ | RAG embeddings by dimension |
| **EpisodicMemory** | `episodic_memory` | ✅ (m=12) | ✅ | ✅ | Episodic agent memories |
| **SemanticMemory** | `entities`, `relationships`, `entity_embeddings` | ✅ (m=16) | ✅ | ✅ | Knowledge graph with embeddings |
| **ProceduralMemory** | `procedural_memory` | ✅ (m=12) | ✅ | ✅ | Skills/patterns with context |
| **AgentState** | `agent_states` | ❌ | ✅ | ❌ | Stateful agent data |
| **WorkflowState** | `workflow_states` | ❌ | ✅ | ❌ | Workflow execution state |
| **SessionStorage** | `sessions` | ❌ | ✅ | ❌ | User/agent session data |
| **ArtifactStorage** | `artifacts` | ❌ | ✅ | ❌ | Content-addressed artifacts (blake3) |
| **EventLog** | `event_log` (partitioned) | ❌ | ✅ | ✅ | Event stream with monthly partitions |
| **HookHistory** | `hook_history` | ❌ | ✅ | ❌ | Hook execution replay data |

### VectorEmbeddings Storage

**Purpose**: Dimension-routed vector storage with HNSW similarity search (8.47x speedup measured).

**Tables**: `llmspell.vector_embeddings_{384,768,1536,3072}`

**API Location**: `llmspell-storage/src/postgres/vector.rs`

```rust
use llmspell_storage::postgres::VectorEmbeddingsStorage;

impl VectorEmbeddingsStorage {
    /// Store embedding with automatic dimension routing
    pub async fn store(
        &self,
        tenant_id: &str,
        embedding_id: Uuid,
        embedding: Vec<f32>,
        metadata: JsonValue,
    ) -> StorageResult<()> {
        let dimension = embedding.len();
        let table = match dimension {
            384 => "vector_embeddings_384",
            768 => "vector_embeddings_768",
            1536 => "vector_embeddings_1536",
            3072 => "vector_embeddings_3072",
            _ => return Err(StorageError::InvalidDimension(dimension)),
        };

        let query = format!(
            "INSERT INTO llmspell.{}
             (tenant_id, embedding_id, embedding, metadata, created_at)
             VALUES ($1, $2, $3, $4, now())",
            table
        );

        sqlx::query(&query)
            .bind(tenant_id)
            .bind(embedding_id)
            .bind(embedding)
            .bind(metadata)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// HNSW similarity search with cosine distance
    pub async fn search(
        &self,
        tenant_id: &str,
        query_embedding: Vec<f32>,
        k: usize,
        threshold: Option<f32>,
    ) -> StorageResult<Vec<VectorSearchResult>> {
        let dimension = query_embedding.len();
        let table = self.route_dimension(dimension)?;

        // HNSW index scan with vector_cosine_ops
        let query = format!(
            "SELECT embedding_id, metadata,
                    1 - (embedding <=> $1) as similarity
             FROM llmspell.{}
             WHERE tenant_id = $2
             ORDER BY embedding <=> $1
             LIMIT $3",
            table
        );

        let mut results: Vec<VectorSearchResult> = sqlx::query_as(&query)
            .bind(&query_embedding)
            .bind(tenant_id)
            .bind(k as i32)
            .fetch_all(&self.pool)
            .await?;

        // Apply threshold filter
        if let Some(threshold) = threshold {
            results.retain(|r| r.similarity >= threshold);
        }

        Ok(results)
    }
}
```

**Index Configuration** (`migrations/V003__create_vector_tables.sql`):
```sql
CREATE INDEX idx_vector_768_hnsw ON llmspell.vector_embeddings_768
    USING hnsw (embedding vector_cosine_ops)
    WITH (m = 16, ef_construction = 128);

-- Query-time tuning
SET hnsw.ef_search = 40;  -- 95% recall target
```

### SemanticMemory Storage (Knowledge Graph)

**Purpose**: Bi-temporal knowledge graph with entity embeddings for semantic similarity.

**Tables**: `llmspell.entities`, `llmspell.relationships`, `llmspell.entity_embeddings`

**API Location**: `llmspell-storage/src/postgres/graph.rs`

```rust
use llmspell_storage::postgres::SemanticMemoryStorage;

impl SemanticMemoryStorage {
    /// Store entity with valid time and transaction time
    pub async fn store_entity(
        &self,
        tenant_id: &str,
        entity_id: Uuid,
        entity_type: &str,
        properties: JsonValue,
        valid_from: DateTime<Utc>,
        embedding: Option<Vec<f32>>,
    ) -> StorageResult<()> {
        let mut tx = self.pool.begin().await?;

        // Insert entity with bi-temporal tracking
        sqlx::query(
            "INSERT INTO llmspell.entities
             (entity_id, tenant_id, entity_type, properties,
              valid_time_start, valid_time_end, transaction_time_start)
             VALUES ($1, $2, $3, $4, $5, 'infinity', now())"
        )
        .bind(entity_id)
        .bind(tenant_id)
        .bind(entity_type)
        .bind(&properties)
        .bind(valid_from)
        .execute(&mut *tx)
        .await?;

        // Store embedding if provided
        if let Some(embedding) = embedding {
            sqlx::query(
                "INSERT INTO llmspell.entity_embeddings
                 (tenant_id, entity_id, embedding_384, created_at)
                 VALUES ($1, $2, $3, now())"
            )
            .bind(tenant_id)
            .bind(entity_id)
            .bind(embedding)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Query entities by valid time (as-of query)
    pub async fn query_as_of(
        &self,
        tenant_id: &str,
        as_of_time: DateTime<Utc>,
    ) -> StorageResult<Vec<Entity>> {
        let entities = sqlx::query_as(
            "SELECT entity_id, entity_type, properties
             FROM llmspell.entities
             WHERE tenant_id = $1
               AND valid_time_start <= $2
               AND valid_time_end > $2
               AND transaction_time_start <= now()
               AND transaction_time_end > now()"
        )
        .bind(tenant_id)
        .bind(as_of_time)
        .fetch_all(&self.pool)
        .await?;

        Ok(entities)
    }

    /// Semantic search over entity embeddings
    pub async fn semantic_search(
        &self,
        tenant_id: &str,
        query_embedding: Vec<f32>,
        k: usize,
    ) -> StorageResult<Vec<EntitySearchResult>> {
        let results = sqlx::query_as(
            "SELECT e.entity_id, e.entity_type, e.properties,
                    1 - (ee.embedding_384 <=> $1) as similarity
             FROM llmspell.entities e
             JOIN llmspell.entity_embeddings ee USING (entity_id, tenant_id)
             WHERE e.tenant_id = $2
               AND e.valid_time_end = 'infinity'
               AND e.transaction_time_end = 'infinity'
             ORDER BY ee.embedding_384 <=> $1
             LIMIT $3"
        )
        .bind(&query_embedding)
        .bind(tenant_id)
        .bind(k as i32)
        .fetch_all(&self.pool)
        .await?;

        Ok(results)
    }
}
```

### ArtifactStorage

**Purpose**: Content-addressed artifact storage with blake3 deduplication (50-90% space savings).

**Table**: `llmspell.artifacts`

**API Location**: `llmspell-storage/src/postgres/artifacts.rs`

```rust
use llmspell_storage::postgres::ArtifactStorage;
use blake3;

impl ArtifactStorage {
    /// Store artifact with automatic deduplication
    pub async fn store(
        &self,
        tenant_id: &str,
        artifact_id: Uuid,
        artifact_type: &str,
        content: &[u8],
        metadata: JsonValue,
    ) -> StorageResult<String> {
        // Content-addressed hash
        let content_hash = blake3::hash(content).to_hex().to_string();

        // Insert (ON CONFLICT DO NOTHING for deduplication)
        sqlx::query(
            "INSERT INTO llmspell.artifacts
             (tenant_id, artifact_id, artifact_type, content_hash,
              content_blob, metadata, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, now())
             ON CONFLICT (content_hash) DO NOTHING"
        )
        .bind(tenant_id)
        .bind(artifact_id)
        .bind(artifact_type)
        .bind(&content_hash)
        .bind(content)
        .bind(metadata)
        .execute(&self.pool)
        .await?;

        Ok(content_hash)
    }

    /// Retrieve artifact by content hash
    pub async fn get_by_hash(
        &self,
        tenant_id: &str,
        content_hash: &str,
    ) -> StorageResult<Option<Artifact>> {
        let artifact = sqlx::query_as(
            "SELECT artifact_id, artifact_type, content_blob, metadata
             FROM llmspell.artifacts
             WHERE tenant_id = $1 AND content_hash = $2"
        )
        .bind(tenant_id)
        .bind(content_hash)
        .fetch_optional(&self.pool)
        .await?;

        Ok(artifact)
    }
}
```

### EventLog Storage

**Purpose**: High-throughput event stream with monthly partitioning (12.5x query speedup measured).

**Table**: `llmspell.event_log` (parent) + monthly partitions

**API Location**: `llmspell-storage/src/postgres/events.rs`

```rust
use llmspell_storage::postgres::EventLogStorage;

impl EventLogStorage {
    /// Append event to log with automatic partition routing
    pub async fn append(
        &self,
        tenant_id: &str,
        event_id: Uuid,
        event_type: &str,
        payload: JsonValue,
        correlation_id: Option<Uuid>,
    ) -> StorageResult<()> {
        // PostgreSQL automatically routes to monthly partition
        sqlx::query(
            "INSERT INTO llmspell.event_log
             (tenant_id, event_id, event_type, payload,
              correlation_id, timestamp)
             VALUES ($1, $2, $3, $4, $5, now())"
        )
        .bind(tenant_id)
        .bind(event_id)
        .bind(event_type)
        .bind(payload)
        .bind(correlation_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Query events with partition pruning
    pub async fn query_range(
        &self,
        tenant_id: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<Vec<Event>> {
        // Partition pruning eliminates non-matching monthly partitions
        let events = sqlx::query_as(
            "SELECT event_id, event_type, payload, timestamp
             FROM llmspell.event_log
             WHERE tenant_id = $1
               AND timestamp >= $2
               AND timestamp < $3
             ORDER BY timestamp"
        )
        .bind(tenant_id)
        .bind(start)
        .bind(end)
        .fetch_all(&self.pool)
        .await?;

        Ok(events)
    }
}
```

**Partition Configuration** (`migrations/V010__create_event_log.sql`):
```sql
CREATE TABLE llmspell.event_log (
    tenant_id VARCHAR(255) NOT NULL,
    event_id UUID NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- ...
    PRIMARY KEY (tenant_id, timestamp, event_id)
) PARTITION BY RANGE (timestamp);

-- Monthly partitions created automatically by trigger
CREATE TABLE llmspell.event_log_2025_01 PARTITION OF llmspell.event_log
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');
```

### Connection Pool Configuration

**API Location**: `llmspell-storage/src/postgres/pool.rs`

```rust
use sqlx::postgres::{PgPoolOptions, PgConnectOptions};

pub struct PoolConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        let cpu_count = num_cpus::get() as u32;
        Self {
            max_connections: (cpu_count * 2) + 1,  // Formula: (cores × 2) + 1
            min_connections: cpu_count,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl PoolConfig {
    pub async fn create_pool(&self, database_url: &str) -> StorageResult<PgPool> {
        let pool = PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(self.acquire_timeout)
            .idle_timeout(Some(self.idle_timeout))
            .connect(database_url)
            .await?;

        Ok(pool)
    }
}
```

### Row-Level Security (RLS)

**Purpose**: Multi-tenant data isolation at database level (<5% overhead measured).

**Configuration**: Enabled per-component in config.toml

**Migration**: `migrations/V002__enable_rls.sql`

```sql
-- Enable RLS on all tenant-isolated tables
ALTER TABLE llmspell.vector_embeddings_768 ENABLE ROW LEVEL SECURITY;
ALTER TABLE llmspell.entities ENABLE ROW LEVEL SECURITY;
ALTER TABLE llmspell.event_log ENABLE ROW LEVEL SECURITY;
-- ... (all 10+ tables)

-- Policy: Users can only see their own tenant's data
CREATE POLICY tenant_isolation ON llmspell.vector_embeddings_768
    USING (tenant_id = current_setting('app.current_tenant_id'));
```

**API Usage**:
```rust
// Automatic RLS enforcement
let backend = PostgreSQLBackend::new(&database_url)
    .await?
    .with_tenant("tenant-123");

// All queries automatically filtered by tenant_id
let results = backend.search(query_embedding, 10).await?;
```

## Configuration

```toml
[storage]
# Backend selection: "memory", "sled", "postgres"
backend = "postgres"

# PostgreSQL configuration (Phase 13b.16)
[storage.postgres]
url = "postgresql://llmspell_app:secure_pass@localhost:5432/llmspell_prod"
pool_size = 20  # (CPU cores × 2) + 1
min_connections = 8
acquire_timeout_secs = 30
idle_timeout_secs = 600
enforce_tenant_isolation = true
run_migrations = true

# Component-specific backend overrides
[storage.components.vector_embeddings]
backend = "postgres"  # Use PostgreSQL for vectors

[storage.components.episodic_memory]
backend = "postgres"

[storage.components.semantic_memory]
backend = "postgres"

[storage.components.agent_state]
backend = "sled"  # Use Sled for fast local state

# Sled configuration (embedded database)
[storage.sled]
path = "./data/storage"
cache_capacity = 1073741824  # 1GB
compression = true
flush_every_ms = 1000

# Vector storage configuration (HNSW)
[storage.vector]
type = "hnsw"

[storage.vector.hnsw]
m = 16
ef_construction = 200
ef_search = 50
max_elements = 1000000
distance_type = "cosine"  # or "euclidean", "inner_product"

# Dimension-specific settings
[storage.vector.dimensions.384]
m = 12
ef_search = 40

[storage.vector.dimensions.768]
m = 16
ef_search = 50

[storage.vector.dimensions.1536]
m = 24
ef_search = 75

[storage.vector.dimensions.3072]
m = 32
ef_search = 100
```

## Performance Considerations

- **HNSW Parameters**: Higher `m` and `ef_construction` improve recall but increase memory and build time
- **Batch Operations**: Use batch methods for multiple operations to reduce overhead
- **Metadata Indexing**: Pre-filter with metadata to reduce vector comparisons
- **Dimension Routing**: Separate indices by dimension for optimal performance
- **Memory vs Disk**: Memory backend for <1GB data, Sled for larger persistent data
- **Vector Normalization**: Normalize vectors before storage for cosine similarity
- **Concurrent Access**: All backends are thread-safe but may have contention

## Security Considerations

- **Tenant Isolation**: Always filter by tenant_id in multi-tenant scenarios
- **Input Validation**: Validate vector dimensions before storage
- **Key Sanitization**: Sanitize keys to prevent injection attacks
- **Access Control**: Implement access control at the application layer
- **Encryption**: Enable encryption at rest for sensitive data
- **Backup**: Regular backups for Sled backend data
- **Resource Limits**: Set max_elements to prevent memory exhaustion

## Migration Guide

### From v0.7.x to v0.8.x (Phase 8)

New features:
- HNSW vector storage implementation
- Dimension routing for optimized performance
- Metadata filtering on vector search
- Multi-tenant collection support
- Batch operations for efficiency
- **Bi-temporal support** with event time and ingestion time tracking
- **TTL mechanism** for automatic vector expiration
- **Temporal queries** for sophisticated time-based retrieval

Breaking changes:
- `VectorStorage` trait methods now async
- `VectorEntry` structure completely redesigned:
  - Renamed `vector` field to `embedding`
  - Added `scope: StateScope` for tenant isolation
  - Added temporal fields: `created_at`, `updated_at`, `event_time`, `expires_at`, `ttl_seconds`
  - Now created with `VectorEntry::new()` builder pattern
- `VectorQuery` replaces `VectorSearchOptions`:
  - New temporal filter fields: `event_time_range`, `ingestion_time_range`, `exclude_expired`
  - Created with `VectorQuery::new()` builder pattern
- Metadata must be JSON-serializable Values

Migration steps:
1. Update vector storage initialization to use HNSWConfig
2. Replace `VectorEntry { ... }` with `VectorEntry::new(id, embedding).with_scope(...)`
3. Replace `VectorSearchOptions` with `VectorQuery::new(vector, k)`
4. Add temporal metadata where applicable (event_time, TTL)
5. Update search calls to handle temporal filters
6. Add tenant_id to scope for multi-tenant apps

Example migration:
```rust
// Old (v0.7.x)
let entry = VectorEntry {
    id: "doc-1".to_string(),
    vector: vec![1.0, 2.0, 3.0],
    metadata: metadata,
    payload: None,
};

// New (v0.8.x)
let entry = VectorEntry::new("doc-1".to_string(), vec![1.0, 2.0, 3.0])
    .with_scope(StateScope::Global)
    .with_event_time(SystemTime::now())
    .with_ttl(86400) // 24 hours
    .with_metadata(metadata);
```