//! Knowledge graph trait with bi-temporal support

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::error::Result;
use crate::types::{Entity, Relationship, TemporalQuery};

/// Bi-temporal knowledge graph trait
///
/// Supports two time dimensions:
/// - **Event Time**: When the real-world event occurred
/// - **Ingestion Time**: When we learned about it
///
/// This enables:
/// - Time-travel queries (what did we know at time T?)
/// - Corrections (update past knowledge without losing history)
/// - Auditing (track knowledge evolution)
#[async_trait]
pub trait KnowledgeGraph: Send + Sync {
    /// Add a new entity to the graph
    ///
    /// # Arguments
    /// * `entity` - The entity to add
    ///
    /// # Returns
    /// The ID of the created entity
    async fn add_entity(&self, entity: Entity) -> Result<String>;

    /// Update an existing entity with new properties
    ///
    /// Creates a new version with current `ingestion_time` while preserving history
    ///
    /// # Arguments
    /// * `id` - Entity ID to update
    /// * `changes` - Property changes to apply
    async fn update_entity(
        &self,
        id: &str,
        changes: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Get the current version of an entity
    ///
    /// Returns the entity with the most recent `ingestion_time`
    async fn get_entity(&self, id: &str) -> Result<Entity>;

    /// Get entity as it was known at a specific event time
    ///
    /// Temporal query: returns entity version valid at given `event_time`
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>;

    /// Add a relationship between two entities
    ///
    /// # Arguments
    /// * `relationship` - The relationship to add
    ///
    /// # Returns
    /// The ID of the created relationship
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;

    /// Get all entities related to a given entity
    ///
    /// # Arguments
    /// * `entity_id` - Source entity ID
    /// * `relationship_type` - Type of relationship to follow (e.g., `has_feature`)
    ///
    /// # Returns
    /// List of target entities connected via the specified relationship
    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>>;

    /// Execute a temporal query on the graph
    ///
    /// Supports filtering by:
    /// - Entity type
    /// - Event time range
    /// - Ingestion time range
    /// - Property filters
    ///
    /// # Arguments
    /// * `query` - Temporal query parameters
    ///
    /// # Returns
    /// Entities matching the query criteria
    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;

    /// Delete all entities and relationships with ingestion time before the given timestamp
    ///
    /// Used for data retention/cleanup
    ///
    /// # Arguments
    /// * `timestamp` - Cutoff ingestion time
    ///
    /// # Returns
    /// Number of entities deleted
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;
}
