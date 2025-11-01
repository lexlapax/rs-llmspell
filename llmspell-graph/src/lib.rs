//! # Bi-Temporal Knowledge Graph for `LLMSpell`
//!
//! This crate provides a production-ready bi-temporal knowledge graph for storing
//! entities and relationships with full temporal tracking.
//!
//! ## Bi-Temporal Semantics
//!
//! The graph tracks two time dimensions:
//! - **Event Time**: When the real-world event occurred (can be None if unknown)
//! - **Ingestion Time**: When we learned about it (always present)
//!
//! This enables:
//! - **Time-travel queries**: What did we know at time T?
//! - **Corrections**: Update past knowledge without losing history
//! - **Auditing**: Track knowledge evolution over time
//!
//! ## Architecture
//!
//! ```text
//! KnowledgeGraph Trait
//! ├── SurrealDBBackend (embedded mode, Phase 13.2)
//! ├── Neo4jBackend (future)
//! └── InMemoryBackend (future, for testing)
//! ```
//!
//! ## Swappable Storage Backends
//!
//! All storage operations are abstracted via the `GraphBackend` trait,
//! enabling hot-swapping between different graph databases:
//! - `SurrealDB` (embedded, no external server)
//! - `Neo4j` (future)
//! - `InMemory` (future, for testing)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llmspell_graph::prelude::*;
//! use serde_json::json;
//!
//! # async fn example() -> Result<()> {
//! // Create knowledge graph with SurrealDB backend
//! let graph = SurrealDBBackend::new("./data".into());
//!
//! // Add entity
//! let entity = Entity::new(
//!     "Rust".into(),
//!     "programming_language".into(),
//!     json!({"paradigm": "multi-paradigm"}),
//! );
//! let entity_id = graph.add_entity(entity).await?;
//!
//! // Add relationship
//! let rel = Relationship::new(
//!     entity_id.clone(),
//!     "memory_safety".into(),
//!     "has_feature".into(),
//!     json!({}),
//! );
//! graph.add_relationship(rel).await?;
//!
//! // Query: Get all related entities
//! let features = graph.get_related(&entity_id, "has_feature").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Temporal Queries
//!
//! ```rust,ignore
//! use llmspell_graph::prelude::*;
//! use chrono::{Utc, Duration};
//!
//! # async fn example(graph: &impl KnowledgeGraph) -> Result<()> {
//! // Query entities as they were known 7 days ago
//! let past = Utc::now() - Duration::days(7);
//! let entity = graph.get_entity_at("entity-123", past).await?;
//!
//! // Query all entities of a type within a time range
//! let query = TemporalQuery::new()
//!     .with_entity_type("person".into())
//!     .with_ingestion_time_range(past, Utc::now());
//! let results = graph.query_temporal(query).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Implementation Status
//!
//! - ✅ Phase 13.2.1: Crate structure and trait definitions
//! - ⏳ Phase 13.2.2: Bi-temporal trait implementation (in progress)
//! - ⏳ Phase 13.2.3: `SurrealDB` backend implementation (pending)
//! - ⏳ Phase 13.2.5: Unit tests and benchmarks (pending)

pub mod error;
pub mod extraction;
pub mod prelude;
pub mod storage;
pub mod traits;
pub mod types;

// Re-exports for convenience
pub use error::{GraphError, Result};
pub use storage::surrealdb::SurrealDBBackend;
pub use storage::GraphBackend;
pub use traits::KnowledgeGraph;
pub use types::{Entity, Relationship, TemporalQuery};
