//! Knowledge graph trait with bi-temporal support
//!
//! Defines the core abstraction for bi-temporal knowledge graph operations.
//! Supports two time dimensions:
//! - **Event Time**: When the real-world event occurred
//! - **Ingestion Time**: When we learned about it
//!
//! This enables:
//! - Time-travel queries (what did we know at time T?)
//! - Corrections (update past knowledge without losing history)
//! - Auditing (track knowledge evolution)
//!
//! Migrated from llmspell-graph/src/traits/knowledge_graph.rs as part of Phase 13c.3.

use crate::types::storage::graph::{Entity, Relationship, TemporalQuery};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Bi-temporal knowledge graph trait
///
/// Provides a unified interface for storing and querying temporal knowledge graphs
/// across different backend implementations. All operations support bi-temporal semantics
/// with both event time and ingestion time tracking.
///
/// # Performance Characteristics
///
/// | Backend | Add Entity | Query | Traverse (4-hop) |
/// |---------|-----------|-------|------------------|
/// | SQLite | ~5ms | ~10ms | <50ms |
/// | PostgreSQL | ~15ms | ~20ms | <100ms |
///
/// # Bi-Temporal Queries
///
/// The graph supports both:
/// - **Event Time**: When events actually occurred in the real world
/// - **Ingestion Time**: When we learned about them
///
/// This allows:
/// ```ignore
/// // Get entity as we knew it on 2024-01-01
/// let historical = graph.get_entity_at("entity-1", date(2024, 1, 1)).await?;
///
/// // Query entities that occurred in January but we learned about in February
/// let query = TemporalQuery::new()
///     .with_event_time_range(jan_1, jan_31)
///     .with_ingestion_time_range(feb_1, feb_28);
/// ```
///
/// # Examples
///
/// ```no_run
/// # use llmspell_core::traits::storage::KnowledgeGraph;
/// # use llmspell_core::types::storage::graph::{Entity, Relationship};
/// # use serde_json::json;
/// # async fn example(graph: impl KnowledgeGraph) -> anyhow::Result<()> {
/// // Add an entity
/// let entity = Entity::new(
///     "Rust".to_string(),
///     "programming_language".to_string(),
///     json!({"paradigm": "multi-paradigm"})
/// );
/// let id = graph.add_entity(entity).await?;
///
/// // Add a relationship
/// let rel = Relationship::new(
///     id.clone(),
///     "memory-safety".to_string(),
///     "has_feature".to_string(),
///     json!({})
/// );
/// graph.add_relationship(rel).await?;
///
/// // Query related entities
/// let related = graph.get_related(&id, "has_feature").await?;
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait KnowledgeGraph: Send + Sync {
    /// Add a new entity to the graph
    ///
    /// Creates a new entity node with a unique ID and current ingestion time.
    /// The entity's event time (if set) tracks when the real-world event occurred.
    ///
    /// # Arguments
    /// * `entity` - The entity to add with name, type, and properties
    ///
    /// # Returns
    /// The unique ID assigned to the created entity
    ///
    /// # Errors
    /// Returns an error if the storage backend is unavailable or the entity is invalid
    ///
    /// # Examples
    /// ```ignore
    /// let entity = Entity::new(
    ///     "Python".to_string(),
    ///     "programming_language".to_string(),
    ///     json!({"version": "3.12"})
    /// );
    /// let id = graph.add_entity(entity).await?;
    /// ```
    async fn add_entity(&self, entity: Entity) -> Result<String>;

    /// Update an existing entity with new properties
    ///
    /// Creates a new version with current `ingestion_time` while preserving history.
    /// The entity's previous versions remain accessible via temporal queries.
    ///
    /// # Arguments
    /// * `id` - Entity ID to update
    /// * `changes` - Property changes to apply (merged with existing properties)
    ///
    /// # Errors
    /// Returns an error if the entity is not found or the update fails
    ///
    /// # Examples
    /// ```ignore
    /// let mut changes = HashMap::new();
    /// changes.insert("version".to_string(), json!("3.13"));
    /// graph.update_entity("entity-1", changes).await?;
    /// ```
    async fn update_entity(
        &self,
        id: &str,
        changes: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Get the current version of an entity
    ///
    /// Returns the entity with the most recent `ingestion_time`, representing
    /// the current known state.
    ///
    /// # Arguments
    /// * `id` - Unique entity identifier
    ///
    /// # Returns
    /// The current entity version
    ///
    /// # Errors
    /// Returns an error if the entity is not found
    async fn get_entity(&self, id: &str) -> Result<Entity>;

    /// Get entity as it was known at a specific event time
    ///
    /// Temporal query that returns the entity version valid at the given `event_time`.
    /// This enables "time-travel" queries to see historical states.
    ///
    /// # Arguments
    /// * `id` - Entity ID to query
    /// * `event_time` - Point in event time to query
    ///
    /// # Returns
    /// Entity version valid at the specified event time
    ///
    /// # Errors
    /// Returns an error if no version exists for that time period
    ///
    /// # Examples
    /// ```ignore
    /// // Get entity as it was on January 1, 2024
    /// use chrono::NaiveDate;
    /// let jan_1 = NaiveDate::from_ymd_opt(2024, 1, 1)
    ///     .unwrap()
    ///     .and_hms_opt(0, 0, 0)
    ///     .unwrap()
    ///     .and_utc();
    /// let historical = graph.get_entity_at("entity-1", jan_1).await?;
    /// ```
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>;

    /// Add a relationship between two entities
    ///
    /// Creates a directed edge from one entity to another with a typed relationship.
    /// Supports bi-temporal tracking like entities.
    ///
    /// # Arguments
    /// * `relationship` - The relationship to add with source, target, type, and properties
    ///
    /// # Returns
    /// The unique ID assigned to the created relationship
    ///
    /// # Errors
    /// Returns an error if either entity doesn't exist or the storage fails
    ///
    /// # Examples
    /// ```ignore
    /// let rel = Relationship::new(
    ///     "python".to_string(),
    ///     "duck-typing".to_string(),
    ///     "has_feature".to_string(),
    ///     json!({"priority": "high"})
    /// );
    /// let rel_id = graph.add_relationship(rel).await?;
    /// ```
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;

    /// Get all entities related to a given entity
    ///
    /// Follows outgoing relationships of the specified type to find connected entities.
    ///
    /// # Arguments
    /// * `entity_id` - Source entity ID
    /// * `relationship_type` - Type of relationship to follow (e.g., "has_feature", "depends_on")
    ///
    /// # Returns
    /// List of target entities connected via the specified relationship type
    ///
    /// # Examples
    /// ```ignore
    /// // Get all features of Python
    /// let features = graph.get_related("python", "has_feature").await?;
    /// ```
    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>>;

    /// Get all relationships for an entity
    ///
    /// Returns both outgoing (from this entity) and incoming (to this entity) relationships.
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID to query
    ///
    /// # Returns
    /// All relationships involving this entity (both directions)
    ///
    /// # Examples
    /// ```ignore
    /// let all_rels = graph.get_relationships("entity-1").await?;
    /// for rel in all_rels {
    ///     if rel.from_entity == "entity-1" {
    ///         println!("Outgoing: {} -> {}", rel.relationship_type, rel.to_entity);
    ///     } else {
    ///         println!("Incoming: {} <- {}", rel.relationship_type, rel.from_entity);
    ///     }
    /// }
    /// ```
    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    /// Execute a temporal query on the graph
    ///
    /// Supports filtering by:
    /// - Entity type
    /// - Event time range (when events occurred)
    /// - Ingestion time range (when we learned about them)
    /// - Property filters (arbitrary JSON matching)
    /// - Result limit
    ///
    /// # Arguments
    /// * `query` - Temporal query parameters built using builder pattern
    ///
    /// # Returns
    /// Entities matching all specified criteria
    ///
    /// # Examples
    /// ```ignore
    /// // Find all programming languages we learned about in February
    /// // that have an "async" feature
    /// let query = TemporalQuery::new()
    ///     .with_entity_type("programming_language".to_string())
    ///     .with_ingestion_time_range(feb_1, feb_28)
    ///     .with_property("features".to_string(), json!("async"))
    ///     .with_limit(100);
    ///
    /// let results = graph.query_temporal(query).await?;
    /// ```
    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;

    /// Delete all entities and relationships with ingestion time before the given timestamp
    ///
    /// Used for data retention and cleanup. Removes all knowledge that was ingested
    /// (learned about) before the specified time.
    ///
    /// # Arguments
    /// * `timestamp` - Cutoff ingestion time (exclusive)
    ///
    /// # Returns
    /// Number of entities deleted
    ///
    /// # Errors
    /// Returns an error if the deletion fails
    ///
    /// # Examples
    /// ```ignore
    /// // Delete all entities older than 90 days
    /// use chrono::Duration;
    /// let cutoff = Utc::now() - Duration::days(90);
    /// let deleted = graph.delete_before(cutoff).await?;
    /// println!("Deleted {} old entities", deleted);
    /// ```
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;

    /// Multi-hop graph traversal with depth limit and cycle prevention
    ///
    /// Performs breadth-first traversal starting from a given entity, following
    /// relationships up to a maximum depth. Prevents infinite loops via cycle detection.
    ///
    /// # Arguments
    /// * `start_entity` - Starting entity ID
    /// * `relationship_type` - Optional relationship type filter (None = traverse all types)
    /// * `max_depth` - Maximum traversal depth (1-4 hops recommended, 10 max)
    /// * `at_time` - Optional temporal point for bi-temporal queries (None = current time)
    ///
    /// # Returns
    /// Vector of (Entity, depth, path) tuples reachable from `start_entity` within `max_depth` hops.
    /// Path is JSON array string of entity IDs traversed to reach this entity.
    ///
    /// # Performance
    /// - 1-hop: O(k) where k = avg relationships per node
    /// - N-hop: O(k^N) worst case, O(k*N) with cycle prevention
    /// - Target: <50ms for 4-hop traversal on 100K node graph
    ///
    /// # Examples
    /// ```ignore
    /// // Find all entities within 2 hops connected by "knows" relationships
    /// let results = graph.traverse("person-1", Some("knows"), 2, None).await?;
    /// for (entity, depth, path) in results {
    ///     println!("Found {} at depth {} via path {}", entity.name, depth, path);
    /// }
    ///
    /// // Find all dependencies (any depth up to 4) as of January 1
    /// let jan_1 = /* date */;
    /// let deps = graph.traverse("package-1", Some("depends_on"), 4, Some(jan_1)).await?;
    /// ```
    async fn traverse(
        &self,
        start_entity: &str,
        relationship_type: Option<&str>,
        max_depth: usize,
        at_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(Entity, usize, String)>>;
}
