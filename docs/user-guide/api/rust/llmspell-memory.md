# llmspell-memory

## Purpose

Adaptive memory system providing three types of memory: episodic (interaction history), semantic (knowledge graph), and procedural (learned patterns). This Phase 13 crate enables LLM applications to maintain contextual awareness across sessions with multi-backend support for production scalability.

## Core Concepts

- **Three Memory Types**: Episodic (conversations), Semantic (knowledge graph), Procedural (patterns)
- **Hot-Swappable Backends**: HNSW (production), InMemory (testing/development), ChromaDB, Qdrant
- **Session Isolation**: Multi-tenant memory with session-scoped data
- **Vector Search**: Semantic similarity search for episodic memory retrieval
- **Knowledge Graph Integration**: Entities and relationships stored in `llmspell-graph`
- **Consolidation Engine**: LLM-driven extraction of knowledge from conversations
- **Performance**: <2ms episodic add, 8.47x HNSW speedup at 10K entries vs linear
- **Temporal Support**: Track both event time and ingestion time for memory entries

## Primary Traits/Structs

### MemoryManager Trait

**Purpose**: Unified interface for all memory types (episodic, semantic, procedural).

```rust
use llmspell_memory::prelude::*;

#[async_trait]
pub trait MemoryManager: Send + Sync {
    /// Access episodic memory (conversation history)
    fn episodic(&self) -> &dyn EpisodicMemory;

    /// Access semantic memory (knowledge graph)
    fn semantic(&self) -> &dyn SemanticMemory;

    /// Access procedural memory (learned patterns)
    fn procedural(&self) -> &dyn ProceduralMemory;

    /// Consolidate episodic memories into semantic knowledge
    async fn consolidate(
        &self,
        session_id: &str,
        mode: ConsolidationMode,
        llm_provider: Option<Arc<dyn LLMProvider>>,
    ) -> Result<ConsolidationResult>;

    /// Check if consolidation engine is available
    fn has_consolidation(&self) -> bool;
}
```

### DefaultMemoryManager

**Purpose**: Production-ready implementation of `MemoryManager` with configurable backends.

```rust
use llmspell_memory::{
    DefaultMemoryManager, MemoryConfig,
    EpisodicBackendType, EpisodicEntry
};

// In-memory backend (testing/development)
let memory = DefaultMemoryManager::new_in_memory()
    .await
    .expect("Failed to create memory manager");

// HNSW backend (production)
let config = MemoryConfig {
    episodic_backend: EpisodicBackendType::HNSW,
    vector_dimensions: 1536,
    hnsw_m: 16,
    hnsw_ef_construction: 200,
    consolidation_batch_size: 10,
    ..Default::default()
};

let memory = DefaultMemoryManager::new(config)
    .await
    .expect("Failed to create memory manager");

// Add episodic memory
let entry = EpisodicEntry::new(
    "session-123".into(),
    "user".into(),
    "What is Rust?".into(),
);
memory.episodic().add(entry).await?;

// Search episodic memories
let results = memory.episodic()
    .search("Rust programming", 10)
    .await?;
```

### EpisodicMemory Trait

**Purpose**: Interface for episodic memory storage and retrieval (conversation history).

```rust
#[async_trait]
pub trait EpisodicMemory: Send + Sync {
    /// Add conversation entry
    async fn add(&self, entry: EpisodicEntry) -> Result<String>;

    /// Search by semantic similarity
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<EpisodicEntry>>;

    /// Get all entries for a session
    async fn get_session(&self, session_id: &str) -> Result<Vec<EpisodicEntry>>;

    /// List unprocessed entries for consolidation
    async fn list_unprocessed(&self, session_id: &str) -> Result<Vec<String>>;

    /// Mark entries as processed after consolidation
    async fn mark_processed(&self, ids: Vec<String>) -> Result<()>;

    /// Get specific entry by ID
    async fn get(&self, id: &str) -> Result<Option<EpisodicEntry>>;

    /// Delete entries by session
    async fn delete_session(&self, session_id: &str) -> Result<()>;
}
```

### EpisodicEntry

**Purpose**: Represents a single conversation exchange with metadata and temporal tracking.

```rust
use llmspell_memory::EpisodicEntry;
use std::collections::HashMap;
use chrono::Utc;

let mut metadata = HashMap::new();
metadata.insert("topic".to_string(), "programming".to_string());
metadata.insert("priority".to_string(), "high".to_string());

let entry = EpisodicEntry::new(
    "session-456".into(),
    "assistant".into(),
    "Rust is a systems programming language...".into(),
)
.with_metadata(metadata)
.with_event_time(Some(Utc::now()));

// Entry structure
pub struct EpisodicEntry {
    pub id: String,              // Unique entry ID (UUID)
    pub session_id: String,      // Session identifier
    pub role: String,            // "user", "assistant", "system"
    pub content: String,         // Message content
    pub metadata: HashMap<String, String>,  // Custom metadata
    pub created_at: DateTime<Utc>,         // Ingestion time
    pub event_time: Option<DateTime<Utc>>, // When event occurred
    pub processed: bool,         // Consolidation status
}
```

### SemanticMemory Trait

**Purpose**: Interface for knowledge graph storage (entities and relationships).

```rust
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    /// Add entity to knowledge graph
    async fn add_entity(&self, entity: Entity) -> Result<String>;

    /// Add relationship between entities
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;

    /// Find entities by type
    async fn find_entities(&self, entity_type: &str) -> Result<Vec<Entity>>;

    /// Get entity by ID
    async fn get_entity(&self, id: &str) -> Result<Option<Entity>>;

    /// Find relationships by type
    async fn find_relationships(
        &self,
        from_entity: &str,
        rel_type: &str,
    ) -> Result<Vec<Relationship>>;

    /// Search entities by name
    async fn search_entities(&self, query: &str, limit: usize) -> Result<Vec<Entity>>;
}
```

### Entity and Relationship

**Purpose**: Core types for semantic memory (knowledge graph nodes and edges).

```rust
use llmspell_memory::{Entity, Relationship};
use serde_json::json;

// Create entity
let entity = Entity::new(
    "Rust".into(),
    "programming_language".into(),
    json!({"paradigm": "multi-paradigm", "memory_safety": true}),
);

// Create relationship
let rel = Relationship::new(
    entity_id.clone(),
    "BLAS".into(),
    "uses_library".into(),
    json!({"version": "0.22"}),
);
```

### ConsolidationMode

**Purpose**: Control how episodic memories are consolidated into semantic knowledge.

```rust
use llmspell_memory::ConsolidationMode;

pub enum ConsolidationMode {
    /// Process immediately (real-time consolidation)
    Immediate,

    /// Process in background (async consolidation)
    Background,

    /// Only mark as processed, no extraction
    MarkOnly,
}
```

### MemoryConfig

**Purpose**: Configuration for memory backends and consolidation settings.

```rust
use llmspell_memory::{MemoryConfig, EpisodicBackendType};

let config = MemoryConfig {
    // Episodic backend selection
    episodic_backend: EpisodicBackendType::HNSW,

    // Vector dimensions (matches embedding model)
    vector_dimensions: 1536,

    // HNSW parameters
    hnsw_m: 16,                      // Links per node
    hnsw_ef_construction: 200,        // Build-time search width
    hnsw_ef_search: 100,              // Query-time search width

    // Consolidation settings
    consolidation_batch_size: 10,     // Entries per batch
    consolidation_enabled: true,      // Enable LLM consolidation

    // Storage paths (for persistent backends)
    data_path: "./data/memory".into(),

    ..Default::default()
};
```

## Backend Implementations

### HNSWEpisodicMemory

**Purpose**: Production backend using HNSW (Hierarchical Navigable Small World) for vector search.

```rust
use llmspell_memory::{HNSWEpisodicMemory, EpisodicEntry};

let backend = HNSWEpisodicMemory::new(
    1536,  // vector dimensions
    16,    // M (links per node)
    200,   // ef_construction
    100,   // ef_search
).await?;

// 8.47x faster than linear scan at 10K entries
let entry = EpisodicEntry::new("session-1".into(), "user".into(), "Hello".into());
backend.add(entry).await?;

let results = backend.search("greeting", 5).await?;
```

### InMemoryEpisodicMemory

**Purpose**: Testing/development backend with linear search (no external dependencies).

```rust
use llmspell_memory::InMemoryEpisodicMemory;

let backend = InMemoryEpisodicMemory::new().await?;

// Simple BTreeMap storage, linear search
// Perfect for tests, not recommended for production >1K entries
```

### GraphSemanticMemory

**Purpose**: Semantic memory backend using `llmspell-graph` for bi-temporal knowledge graph.

```rust
use llmspell_memory::GraphSemanticMemory;
use llmspell_graph::SurrealDBBackend;

let graph_backend = SurrealDBBackend::new("./data/graph".into()).await?;
let semantic_memory = GraphSemanticMemory::new(Arc::new(graph_backend));

// Stores entities and relationships with temporal tracking
```

## Usage Patterns

### Basic Memory Operations

```rust
use llmspell_memory::prelude::*;

// Create memory manager
let memory = DefaultMemoryManager::new_in_memory().await?;

// Add episodic memory
let entry = EpisodicEntry::new(
    "session-123".into(),
    "user".into(),
    "What is Rust ownership?".into(),
);
memory.episodic().add(entry).await?;

// Search episodic memories
let results = memory.episodic()
    .search("ownership", 5)
    .await?;

for entry in results {
    println!("[{}] {}: {}", entry.created_at, entry.role, entry.content);
}
```

### Consolidation Workflow

```rust
use llmspell_memory::{ConsolidationMode, DefaultMemoryManager};

let memory = DefaultMemoryManager::new_in_memory().await?;

// Add conversation history
for i in 0..10 {
    let entry = EpisodicEntry::new(
        "session-456".into(),
        "user".into(),
        format!("Message {i} about Rust programming"),
    );
    memory.episodic().add(entry).await?;
}

// Consolidate into semantic knowledge
let result = memory.consolidate(
    "session-456",
    ConsolidationMode::Immediate,
    None,  // Uses NoopConsolidationEngine by default
).await?;

println!("Processed {} entries", result.entries_processed);

// Query semantic memory for extracted entities
let entities = memory.semantic()
    .find_entities("programming_language")
    .await?;
```

### Multi-Session Isolation

```rust
// Session A
for i in 0..5 {
    memory.episodic().add(
        EpisodicEntry::new("session-a".into(), "user".into(), format!("A: {i}"))
    ).await?;
}

// Session B
for i in 0..5 {
    memory.episodic().add(
        EpisodicEntry::new("session-b".into(), "user".into(), format!("B: {i}"))
    ).await?;
}

// Query session A only
let session_a_entries = memory.episodic()
    .get_session("session-a")
    .await?;

assert_eq!(session_a_entries.len(), 5);
```

## Performance Characteristics

| Operation | HNSW Backend | InMemory Backend |
|-----------|--------------|------------------|
| Add entry | <2ms | <1ms |
| Search (100 entries) | <5ms | <10ms |
| Search (10K entries) | <10ms | ~85ms |
| Consolidation (10 entries) | ~500ms | ~500ms |

**HNSW Speedup**: 8.47x faster than linear scan at 10K entries

## Integration with Other Crates

### With llmspell-graph

Semantic memory uses `llmspell-graph` for bi-temporal knowledge graph storage:

```rust
use llmspell_memory::GraphSemanticMemory;
use llmspell_graph::SurrealDBBackend;

let graph = SurrealDBBackend::new("./data".into()).await?;
let semantic = GraphSemanticMemory::new(Arc::new(graph));
```

### With llmspell-context

Context assembly uses memory for retrieval:

```rust
use llmspell_context::ContextBridge;
use llmspell_memory::DefaultMemoryManager;

let memory = DefaultMemoryManager::new_in_memory().await?;
let context_bridge = ContextBridge::new(memory.clone());

let result = context_bridge.assemble(
    "Rust ownership",
    "episodic",
    2000,  // token budget
    Some("session-123"),
).await?;
```

### With llmspell-bridge (Lua API)

Memory exposed via `Memory` global in Lua scripts:

```lua
-- Add episodic memory
local id = Memory.episodic.add("session-1", "user", "What is Rust?")

-- Search memories
local results = Memory.episodic.search("session-1", "ownership", 10)

-- Consolidate
local result = Memory.consolidate("session-1", "immediate")
```

## Testing Utilities

```rust
use llmspell_memory::DefaultMemoryManager;

#[tokio::test]
async fn test_memory_operations() {
    let memory = DefaultMemoryManager::new_in_memory()
        .await
        .expect("Failed to create memory");

    let entry = EpisodicEntry::new(
        "test-session".into(),
        "user".into(),
        "Test message".into(),
    );

    let id = memory.episodic().add(entry).await.unwrap();
    assert!(!id.is_empty());

    let results = memory.episodic()
        .search("Test", 5)
        .await
        .unwrap();

    assert!(!results.is_empty());
}
```

## Configuration Best Practices

### Development

```rust
// Use InMemory backend for fast iteration
let config = MemoryConfig {
    episodic_backend: EpisodicBackendType::InMemory,
    ..Default::default()
};
```

### Production

```rust
// Use HNSW with optimized parameters
let config = MemoryConfig {
    episodic_backend: EpisodicBackendType::HNSW,
    vector_dimensions: 1536,
    hnsw_m: 16,                // 8-48 range (16 is balanced)
    hnsw_ef_construction: 200, // Higher = better recall, slower build
    hnsw_ef_search: 100,       // Higher = better recall, slower search
    consolidation_enabled: true,
    ..Default::default()
};
```

## Error Handling

```rust
use llmspell_memory::{MemoryError, Result};

async fn handle_memory_operations() -> Result<()> {
    let memory = DefaultMemoryManager::new_in_memory().await?;

    match memory.episodic().search("query", 10).await {
        Ok(results) => println!("Found {} results", results.len()),
        Err(MemoryError::BackendError(e)) => eprintln!("Backend error: {e}"),
        Err(MemoryError::InvalidInput(msg)) => eprintln!("Invalid input: {msg}"),
        Err(e) => eprintln!("Other error: {e}"),
    }

    Ok(())
}
```

## Related Documentation

- [llmspell-graph](llmspell-graph.md) - Bi-temporal knowledge graph backend
- [llmspell-context](llmspell-context.md) - Context engineering and retrieval
- [Memory Configuration Guide](../../memory-configuration.md) - User-facing configuration
- [Memory Lua API](../lua/README.md#memory) - Script-level memory access

---

**Phase 13 Integration** | See [Phase 13 Design Doc](../../../../docs/in-progress/phase-13-design-doc.md) for architecture details
