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
//! # Types
//!
//! Entity and Relationship types are re-exported from `llmspell-graph` to avoid
//! duplication and ensure consistency across the memory system.

use async_trait::async_trait;
use chrono::{DateTime, Utc};

use crate::error::Result;

// Re-export graph types as canonical types for semantic memory
pub use llmspell_graph::types::{Entity, Relationship};

/// Semantic memory stores bi-temporal knowledge graph
///
/// Implemented by `GraphSemanticMemory` which wraps `llmspell-graph::KnowledgeGraph`.
///
/// # Example
///
/// ```rust,no_run
/// use llmspell_memory::prelude::*;
/// use llmspell_memory::semantic::GraphSemanticMemory;
/// use llmspell_storage::backends::sqlite::SqliteBackend;
/// use serde_json::json;
/// use chrono::Utc;
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let sqlite_backend = Arc::new(SqliteBackend::new(llmspell_storage::backends::sqlite::SqliteConfig::in_memory()).await?);
///     let semantic = GraphSemanticMemory::new_with_sqlite(sqlite_backend);
///
///     // Add an entity
///     semantic.upsert_entity(Entity::new(
///         "Alice".into(),
///         "person".into(),
///         json!({"role": "engineer"}),
///     )).await?;
///
///     // Query at a specific time
///     let past_time = Utc::now();
///     let entity = semantic.get_entity_at("person-123", past_time).await?;
///     Ok(())
/// }
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
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Option<Entity>>;

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
