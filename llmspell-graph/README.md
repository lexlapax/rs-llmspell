# llmspell-graph

Bi-temporal knowledge graph with swappable storage backends for the LLMSpell memory system.

## Features

- **Bi-Temporal Tracking**: Track both event time (when it happened) and ingestion time (when we learned about it)
- **Swappable Backends**: Abstract storage via `GraphBackend` trait (SQLite, PostgreSQL)
- **Time-Travel Queries**: Query historical knowledge states
- **Temporal Corrections**: Update past knowledge without losing history
- **Full Auditing**: Track complete knowledge evolution
- **Graph Traversal**: Recursive CTE-based traversal with cycle detection

## Architecture

```text
KnowledgeGraph Trait
├── Entity (nodes with properties)
├── Relationship (edges between entities)
├── TemporalQuery (bi-temporal filtering)
└── traverse() (graph traversal with cycle detection)

Storage Implementations (via llmspell-storage)
├── SqliteGraphStorage (embedded, vectorlite-rs HNSW)
├── PostgresGraphStorage (production, multi-tenant RLS)
└── GraphBackend trait (storage abstraction layer)
```

## Usage

```rust
use llmspell_graph::prelude::*;
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteGraphStorage, SqliteConfig};
use serde_json::json;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Create SQLite backend (or use PostgresGraphStorage for production)
    let config = SqliteConfig::new("./data/graph.db");
    let backend = Arc::new(SqliteBackend::new(config).await?);
    let graph = SqliteGraphStorage::new(backend);

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

    // Traverse graph (up to 3 hops with cycle detection)
    let traversal = graph.traverse(&entity_id, None, 3, None).await?;
    println!("Found {} connected entities", traversal.len());

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

Phase 13 (Temporal Knowledge Graph):
- ✅ Task 13.2.1: Crate structure and trait definitions
- ✅ Task 13.2.2: Bi-temporal trait implementation
- ✅ Task 13c.2.4: SQLite graph storage (via llmspell-storage)
- ✅ Task 13c.2.4: PostgreSQL graph storage (via llmspell-storage)
- ✅ Task 13c.2.8: Graph traversal with recursive CTEs
- ✅ Task 13c.2.8: Legacy backend removal (SurrealDB deleted)

**Note**: Graph storage implementations moved to `llmspell-storage` crate for unified storage architecture.

## Dependencies

- `chrono` - Temporal types
- `serde`, `serde_json` - Serialization
- `async-trait` - Async trait support

**Storage backends** (provided by `llmspell-storage`):
- SQLite with vectorlite-rs HNSW (embedded, <2ms queries)
- PostgreSQL with GiST indexes (production, multi-tenant RLS)

## License

Apache-2.0
