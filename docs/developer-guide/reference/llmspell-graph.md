# llmspell-graph

## Purpose

Bi-temporal knowledge graph providing entity and relationship storage with full temporal tracking. This Phase 13 crate enables semantic memory by tracking both event time (when facts occurred) and ingestion time (when we learned them), supporting time-travel queries, corrections, and auditing.

## Core Concepts

- **Bi-Temporal Semantics**: Track event time and ingestion time independently
- **Knowledge Graph**: Store entities (nodes) and relationships (edges) with properties
- **Temporal Queries**: Query graph state at any point in time
- **Swappable Backends**: SurrealDB (embedded), Neo4j (future), InMemory (future)
- **Embedded Operation**: No external server required with SurrealDB backend
- **Corrections and Auditing**: Update knowledge without losing historical state
- **Session Integration**: Works with `llmspell-memory` for semantic memory storage

## Primary Traits/Structs

### KnowledgeGraph Trait

**Purpose**: Abstract interface for bi-temporal knowledge graph operations.

```rust
use llmspell_graph::prelude::*;
use chrono::{DateTime, Utc};

#[async_trait]
pub trait KnowledgeGraph: Send + Sync {
    /// Add entity to graph
    async fn add_entity(&self, entity: Entity) -> Result<String>;

    /// Add relationship between entities
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;

    /// Get entity by ID
    async fn get_entity(&self, id: &str) -> Result<Option<Entity>>;

    /// Get entity at specific point in time (temporal query)
    async fn get_entity_at(
        &self,
        id: &str,
        as_of: DateTime<Utc>,
    ) -> Result<Option<Entity>>;

    /// Find entities by type
    async fn find_entities(&self, entity_type: &str) -> Result<Vec<Entity>>;

    /// Get all relationships from entity
    async fn get_related(
        &self,
        from_id: &str,
        rel_type: &str,
    ) -> Result<Vec<Entity>>;

    /// Execute temporal range query
    async fn temporal_query(&self, query: TemporalQuery) -> Result<Vec<Entity>>;

    /// Update entity (creates new version with new ingestion time)
    async fn update_entity(&self, id: &str, entity: Entity) -> Result<()>;

    /// Delete entity (soft delete, preserves history)
    async fn delete_entity(&self, id: &str) -> Result<()>;
}
```

### Entity

**Purpose**: Represents a knowledge graph node (concept, person, object, etc.).

```rust
use llmspell_graph::Entity;
use serde_json::json;
use chrono::Utc;

let entity = Entity::new(
    "Rust".into(),
    "programming_language".into(),
    json!({
        "paradigm": "multi-paradigm",
        "memory_safety": true,
        "first_release": "2010"
    }),
)
.with_event_time(Some(Utc::now()))  // Optional: when this fact was true
.with_source("documentation".into()); // Optional: where this came from

// Entity structure
pub struct Entity {
    pub id: String,                     // Unique entity ID (UUID)
    pub name: String,                   // Entity name/label
    pub entity_type: String,            // Type classification
    pub properties: serde_json::Value,  // Arbitrary JSON properties
    pub created_at: DateTime<Utc>,      // Ingestion time (when we learned)
    pub event_time: Option<DateTime<Utc>>, // Event time (when it occurred)
    pub source: Option<String>,         // Source of information
}
```

### Relationship

**Purpose**: Represents a knowledge graph edge connecting two entities.

```rust
use llmspell_graph::Relationship;
use serde_json::json;

let rel = Relationship::new(
    "rust-entity-id".into(),
    "memory-safety-entity-id".into(),
    "has_feature".into(),
    json!({"importance": "high"}),
)
.with_event_time(Some(Utc::now()));

// Relationship structure
pub struct Relationship {
    pub id: String,                     // Unique relationship ID
    pub from_entity: String,            // Source entity ID
    pub to_entity: String,              // Target entity ID
    pub rel_type: String,               // Relationship type
    pub properties: serde_json::Value,  // Arbitrary JSON properties
    pub created_at: DateTime<Utc>,      // Ingestion time
    pub event_time: Option<DateTime<Utc>>, // Event time
}
```

### TemporalQuery

**Purpose**: Builder for complex temporal queries with filtering and time ranges.

```rust
use llmspell_graph::TemporalQuery;
use chrono::{Utc, Duration};

let query = TemporalQuery::new()
    .with_entity_type("person".into())
    .with_event_time_range(
        Utc::now() - Duration::days(30),
        Utc::now(),
    )
    .with_limit(100);

let results = graph.temporal_query(query).await?;
```

## Backend Implementations

### SurrealDBBackend

**Purpose**: Production backend using embedded SurrealDB (no external server required).

```rust
use llmspell_graph::SurrealDBBackend;
use llmspell_graph::prelude::*;

// Create embedded SurrealDB backend
let graph = SurrealDBBackend::new("./data/graph".into()).await?;

// Add entity
let entity = Entity::new(
    "Rust".into(),
    "programming_language".into(),
    json!({"paradigm": "multi-paradigm"}),
);
let entity_id = graph.add_entity(entity).await?;

// Add relationship
let rel = Relationship::new(
    entity_id.clone(),
    "safety-entity-id".into(),
    "has_feature".into(),
    json!({}),
);
graph.add_relationship(rel).await?;

// Query relationships
let features = graph.get_related(&entity_id, "has_feature").await?;
```

**Storage Details**:
- Embedded mode (no daemon/server process)
- File-based persistence in data directory
- Bi-temporal indexes for efficient time-travel queries
- Graph traversal optimizations

### Future Backends

**Neo4jBackend** (planned): Production-grade graph database with Cypher query language.

**InMemoryBackend** (planned): Testing backend with HashMap storage.

## Usage Patterns

### Basic Entity and Relationship Management

```rust
use llmspell_graph::prelude::*;
use serde_json::json;

async fn basic_graph_operations() -> Result<()> {
    let graph = SurrealDBBackend::new("./data".into()).await?;

    // Create entities
    let rust_id = graph.add_entity(
        Entity::new(
            "Rust".into(),
            "programming_language".into(),
            json!({"year": 2010}),
        )
    ).await?;

    let safety_id = graph.add_entity(
        Entity::new(
            "Memory Safety".into(),
            "feature".into(),
            json!({"category": "safety"}),
        )
    ).await?;

    // Create relationship
    graph.add_relationship(
        Relationship::new(
            rust_id.clone(),
            safety_id.clone(),
            "has_feature".into(),
            json!({}),
        )
    ).await?;

    // Query related entities
    let features = graph.get_related(&rust_id, "has_feature").await?;
    println!("Rust has {} features", features.len());

    Ok(())
}
```

### Temporal Queries and Time-Travel

```rust
use llmspell_graph::prelude::*;
use chrono::{Utc, Duration};

async fn temporal_queries() -> Result<()> {
    let graph = SurrealDBBackend::new("./data".into()).await?;

    // Add entity with event time
    let entity = Entity::new(
        "Rust 1.0".into(),
        "release".into(),
        json!({"version": "1.0.0"}),
    ).with_event_time(Some(Utc::now() - Duration::days(3650))); // ~10 years ago

    let entity_id = graph.add_entity(entity).await?;

    // Update entity (creates new version)
    let updated = Entity::new(
        "Rust 1.74".into(),
        "release".into(),
        json!({"version": "1.74.0"}),
    );
    graph.update_entity(&entity_id, updated).await?;

    // Time-travel: Get entity as it was 7 days ago
    let past = Utc::now() - Duration::days(7);
    let historical_entity = graph.get_entity_at(&entity_id, past).await?;

    // Range query: All releases in last year
    let query = TemporalQuery::new()
        .with_entity_type("release".into())
        .with_event_time_range(
            Utc::now() - Duration::days(365),
            Utc::now(),
        );

    let recent_releases = graph.temporal_query(query).await?;
    println!("Found {} releases in last year", recent_releases.len());

    Ok(())
}
```

### Knowledge Extraction from Conversations

```rust
use llmspell_graph::prelude::*;
use llmspell_memory::{DefaultMemoryManager, EpisodicEntry};

async fn extract_knowledge_from_memory() -> Result<()> {
    let graph = SurrealDBBackend::new("./data".into()).await?;
    let memory = DefaultMemoryManager::new_in_memory().await?;

    // Simulate conversation about Rust
    memory.episodic().add(
        EpisodicEntry::new(
            "session-1".into(),
            "user".into(),
            "Tell me about Rust's memory safety features".into(),
        )
    ).await?;

    memory.episodic().add(
        EpisodicEntry::new(
            "session-1".into(),
            "assistant".into(),
            "Rust provides memory safety through ownership, borrowing, and lifetimes".into(),
        )
    ).await?;

    // Extract entities (manually for now, LLM-driven in future)
    let rust_entity = graph.add_entity(
        Entity::new(
            "Rust".into(),
            "programming_language".into(),
            json!({}),
        )
    ).await?;

    let ownership_entity = graph.add_entity(
        Entity::new(
            "Ownership".into(),
            "memory_safety_feature".into(),
            json!({}),
        )
    ).await?;

    // Create relationship
    graph.add_relationship(
        Relationship::new(
            rust_entity,
            ownership_entity,
            "has_feature".into(),
            json!({"mentioned_in": "session-1"}),
        )
    ).await?;

    Ok(())
}
```

### Integration with Semantic Memory

```rust
use llmspell_graph::SurrealDBBackend;
use llmspell_memory::{GraphSemanticMemory, DefaultMemoryManager};
use std::sync::Arc;

async fn semantic_memory_integration() -> Result<()> {
    // Create graph backend
    let graph_backend = Arc::new(
        SurrealDBBackend::new("./data/graph".into()).await?
    );

    // Wrap in semantic memory interface
    let semantic_memory = GraphSemanticMemory::new(graph_backend.clone());

    // Add entity via semantic memory
    let entity = Entity::new(
        "Python".into(),
        "programming_language".into(),
        json!({"typing": "dynamic"}),
    );
    semantic_memory.add_entity(entity).await?;

    // Query via semantic memory
    let languages = semantic_memory
        .find_entities("programming_language")
        .await?;

    println!("Found {} languages", languages.len());

    Ok(())
}
```

## Temporal Semantics Explained

### Event Time vs Ingestion Time

```rust
use chrono::{Utc, Duration};

// Event time: When the real-world fact occurred
// Ingestion time: When we learned about it

let entity = Entity::new(
    "Historical Fact".into(),
    "fact".into(),
    json!({"description": "Something that happened in the past"}),
)
.with_event_time(Some(Utc::now() - Duration::days(365 * 100))); // 100 years ago

// created_at (ingestion time) is set automatically to Utc::now()
// event_time is set to 100 years ago

// This enables:
// 1. Time-travel: "What did we know about X on date Y?"
// 2. Corrections: "We just learned that X actually happened on date Z, not date Y"
// 3. Auditing: "When did we learn about this fact?"
```

### Corrections Without Losing History

```rust
// Original incorrect knowledge
let entity_id = graph.add_entity(
    Entity::new(
        "Rust 1.0 Release".into(),
        "event".into(),
        json!({"date": "2014-05-15"}), // Incorrect
    )
).await?;

// Correction: Rust 1.0 was actually released on 2015-05-15
graph.update_entity(
    &entity_id,
    Entity::new(
        "Rust 1.0 Release".into(),
        "event".into(),
        json!({"date": "2015-05-15"}), // Correct
    )
).await?;

// Both versions are stored:
// - Original version with ingestion_time T1
// - Updated version with ingestion_time T2
// Can query either version using get_entity_at()
```

## Performance Characteristics

| Operation | SurrealDB Backend | Notes |
|-----------|-------------------|-------|
| Add entity | ~5ms | Embedded mode, single write |
| Add relationship | ~6ms | Embedded mode, graph update |
| Get entity | ~2ms | Index lookup |
| Get related (depth 1) | ~10ms | Single traversal |
| Temporal query (1K entities) | ~50ms | Time-range filter + scan |
| Time-travel query | ~3ms | Temporal index lookup |

**Note**: Performance varies with dataset size and query complexity.

## Error Handling

```rust
use llmspell_graph::{GraphError, Result};

async fn handle_graph_errors() -> Result<()> {
    let graph = SurrealDBBackend::new("./data".into()).await?;

    match graph.get_entity("nonexistent-id").await {
        Ok(Some(entity)) => println!("Found: {}", entity.name),
        Ok(None) => println!("Entity not found"),
        Err(GraphError::BackendError(e)) => eprintln!("Backend error: {e}"),
        Err(GraphError::InvalidInput(msg)) => eprintln!("Invalid input: {msg}"),
        Err(e) => eprintln!("Other error: {e}"),
    }

    Ok(())
}
```

## Testing

```rust
use llmspell_graph::prelude::*;

#[tokio::test]
async fn test_entity_lifecycle() {
    let graph = SurrealDBBackend::new("./test_data".into())
        .await
        .expect("Failed to create graph");

    // Add entity
    let entity = Entity::new(
        "Test".into(),
        "test_type".into(),
        json!({"key": "value"}),
    );

    let id = graph.add_entity(entity.clone()).await.unwrap();
    assert!(!id.is_empty());

    // Retrieve entity
    let retrieved = graph.get_entity(&id).await.unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, "Test");

    // Update entity
    let updated = Entity::new(
        "Test Updated".into(),
        "test_type".into(),
        json!({"key": "new_value"}),
    );
    graph.update_entity(&id, updated).await.unwrap();

    // Verify update
    let latest = graph.get_entity(&id).await.unwrap();
    assert_eq!(latest.unwrap().name, "Test Updated");
}
```

## Configuration

### SurrealDB Backend Configuration

```rust
use llmspell_graph::SurrealDBBackend;

// Default: Embedded mode with RocksDB storage
let graph = SurrealDBBackend::new("./data/graph".into()).await?;

// Custom namespace and database
let graph = SurrealDBBackend::new_with_config(
    "./data/graph".into(),
    "llmspell".into(),    // namespace
    "knowledge".into(),   // database
).await?;
```

## Integration with llmspell-memory

The graph backend is used by `llmspell-memory` for semantic memory:

```rust
use llmspell_memory::{DefaultMemoryManager, GraphSemanticMemory};
use llmspell_graph::SurrealDBBackend;
use std::sync::Arc;

// Create graph backend
let graph = Arc::new(
    SurrealDBBackend::new("./data/graph".into()).await?
);

// Use with semantic memory
let semantic = GraphSemanticMemory::new(graph);

// Integrated in DefaultMemoryManager
let memory = DefaultMemoryManager::new_in_memory().await?;
let entities = memory.semantic().find_entities("concept").await?;
```

## Related Documentation

- [llmspell-memory](llmspell-memory.md) - Adaptive memory system using this graph
- [llmspell-context](llmspell-context.md) - Context retrieval using semantic memory
- [Phase 13 Design](../../../../docs/in-progress/phase-13-design-doc.md) - Architecture details

---

**Phase 13 Integration** | Bi-temporal knowledge graph for semantic memory | [SurrealDB Documentation](https://surrealdb.com/docs)
