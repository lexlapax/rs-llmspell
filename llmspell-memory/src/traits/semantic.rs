//! Semantic memory trait
//!
//! Semantic memory stores bi-temporal knowledge graphs extracted from episodic memories.
//! Unlike traditional knowledge graphs, bi-temporal graphs track both:
//! - **`event_time`**: When the fact was true in the real world
//! - **`ingestion_time`**: When we learned about the fact
//!
//! This enables temporal queries ("what did we know about X at time T?") and
//! knowledge evolution tracking.
//!
//! # Bi-Temporal Design
//!
//! ```text
//! event_time:     |----[Fact A: true]---->
//! ingestion_time:      ^
//!                      |
//!                 (when we learned)
//! ```
//!
//! # Implementation Status
//!
//! This trait is a **placeholder** for Phase 13.2 (Graph Layer Foundation).
//! The full implementation will use `SurrealDB` or `Neo4j` for graph storage.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::Result;

/// Entity in the knowledge graph
///
/// Represents a real-world entity (person, place, concept, etc.)
/// with bi-temporal tracking.
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::traits::Entity;
/// use serde_json::json;
/// use chrono::Utc;
///
/// let entity = Entity {
///     id: "person-123".into(),
///     entity_type: "person".into(),
///     name: "Alice".into(),
///     properties: json!({
///         "role": "software engineer",
///         "company": "Acme Corp"
///     }),
///     event_time: Utc::now(),
///     ingestion_time: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Unique entity identifier
    pub id: String,

    /// Entity type (person, place, concept, etc.)
    pub entity_type: String,

    /// Primary name/label for the entity
    pub name: String,

    /// Additional properties as JSON
    pub properties: Value,

    /// When this fact was true in the real world
    pub event_time: DateTime<Utc>,

    /// When we learned about this fact
    pub ingestion_time: DateTime<Utc>,
}

/// Relationship between entities in the knowledge graph
///
/// Represents a directed edge between two entities with bi-temporal tracking.
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory::traits::Relationship;
/// use serde_json::json;
/// use chrono::Utc;
///
/// let relationship = Relationship {
///     id: "rel-456".into(),
///     from_entity: "person-123".into(),
///     to_entity: "company-789".into(),
///     relationship_type: "works_at".into(),
///     properties: json!({
///         "start_date": "2023-01-01",
///         "position": "engineer"
///     }),
///     event_time: Utc::now(),
///     ingestion_time: Utc::now(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// Unique relationship identifier
    pub id: String,

    /// Source entity ID
    pub from_entity: String,

    /// Target entity ID
    pub to_entity: String,

    /// Relationship type (`works_at`, knows, `located_in`, etc.)
    pub relationship_type: String,

    /// Additional properties as JSON
    pub properties: Value,

    /// When this relationship was true in the real world
    pub event_time: DateTime<Utc>,

    /// When we learned about this relationship
    pub ingestion_time: DateTime<Utc>,
}

/// Semantic memory stores bi-temporal knowledge graph
///
/// **Status**: Placeholder for Phase 13.2 implementation.
///
/// # Example (Future)
///
/// ```rust,ignore
/// use llmspell_memory::prelude::*;
///
/// let semantic = SurrealDBSemanticMemory::new(config).await?;
///
/// // Add an entity
/// semantic.upsert_entity(Entity {
///     id: "person-123".into(),
///     entity_type: "person".into(),
///     name: "Alice".into(),
///     properties: json!({"role": "engineer"}),
///     event_time: Utc::now(),
///     ingestion_time: Utc::now(),
/// }).await?;
///
/// // Query at a specific time
/// let entity = semantic.get_entity_at("person-123", past_time).await?;
/// ```
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    /// Add or update an entity in the knowledge graph
    ///
    /// If the entity already exists (same ID), this creates a new temporal
    /// version with the provided `ingestion_time`.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to upsert
    async fn upsert_entity(&self, entity: Entity) -> Result<()>;

    /// Get the current version of an entity
    ///
    /// Returns the entity as of the current time (latest `ingestion_time`).
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID to retrieve
    ///
    /// # Returns
    ///
    /// The entity if it exists and is not deleted
    async fn get_entity(&self, id: &str) -> Result<Option<Entity>>;

    /// Get an entity as it was known at a specific point in time
    ///
    /// Performs a temporal query to retrieve the entity state
    /// at the given `event_time`.
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID to retrieve
    /// * `event_time` - The point in time to query
    ///
    /// # Returns
    ///
    /// The entity state at that time, if it existed
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>)
        -> Result<Option<Entity>>;

    /// Add a relationship between entities
    ///
    /// Creates a directed edge in the knowledge graph.
    ///
    /// # Arguments
    ///
    /// * `relationship` - The relationship to add
    async fn add_relationship(&self, relationship: Relationship) -> Result<()>;

    /// Get all relationships for an entity
    ///
    /// Returns both outgoing (from this entity) and incoming (to this entity)
    /// relationships.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - The entity ID to query
    ///
    /// # Returns
    ///
    /// All relationships involving this entity
    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    /// Query entities by type
    ///
    /// Returns all entities of a specific type (e.g., all "person" entities).
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The entity type to filter by
    ///
    /// # Returns
    ///
    /// All entities of this type
    async fn query_by_type(&self, entity_type: &str) -> Result<Vec<Entity>>;

    /// Delete an entity (soft delete with ingestion time)
    ///
    /// Marks the entity as deleted by adding a tombstone record
    /// with the current `ingestion_time`.
    ///
    /// # Arguments
    ///
    /// * `id` - The entity ID to delete
    async fn delete_entity(&self, id: &str) -> Result<()>;
}
