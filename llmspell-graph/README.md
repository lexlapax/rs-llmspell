# llmspell-graph

Bi-temporal knowledge graph with swappable storage backends for the LLMSpell memory system.

## Features

- **Bi-Temporal Tracking**: Track both event time (when it happened) and ingestion time (when we learned about it)
- **Swappable Backends**: Abstract storage via `GraphBackend` trait (SurrealDB, Neo4j, in-memory)
- **Time-Travel Queries**: Query historical knowledge states
- **Temporal Corrections**: Update past knowledge without losing history
- **Full Auditing**: Track complete knowledge evolution

## Architecture

```text
KnowledgeGraph Trait
├── Entity (nodes with properties)
├── Relationship (edges between entities)
└── TemporalQuery (bi-temporal filtering)

GraphBackend Implementations
├── SurrealDBBackend (embedded mode, no external server)
├── Neo4jBackend (future)
└── InMemoryBackend (future, for testing)
```

## Usage

```rust
use llmspell_graph::prelude::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    // Create graph with SurrealDB backend
    let graph = SurrealDBBackend::new("./data".into());

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
        "memory_safety".into(),
        "has_feature".into(),
        json!({}),
    );
    graph.add_relationship(rel).await?;

    // Query related entities
    let features = graph.get_related(&entity_id, "has_feature").await?;
    println!("Found {} features", features.len());

    Ok(())
}
```

## Temporal Queries

```rust
use chrono::{Utc, Duration};

// Query entities as they were 7 days ago
let past = Utc::now() - Duration::days(7);
let entity = graph.get_entity_at("entity-123", past).await?;

// Query by time range
let query = TemporalQuery::new()
    .with_entity_type("person".into())
    .with_ingestion_time_range(past, Utc::now())
    .with_limit(100);
let results = graph.query_temporal(query).await?;
```

## Implementation Status

Phase 13.2 (Temporal Knowledge Graph):
- ✅ Task 13.2.1: Crate structure and trait definitions
- ⏳ Task 13.2.2: Bi-temporal trait implementation (in progress)
- ⏳ Task 13.2.3: SurrealDB backend (embedded mode)
- ⏳ Task 13.2.5: Unit tests and benchmarks

## Dependencies

- `surrealdb = "2.0"` - Embedded graph database
- `tokio` - Async runtime
- `async-trait` - Async trait support
- `serde`, `serde_json` - Serialization
- `chrono` - Temporal types
- `uuid` - ID generation
- `parking_lot`, `dashmap` - Concurrency

## License

Apache-2.0
