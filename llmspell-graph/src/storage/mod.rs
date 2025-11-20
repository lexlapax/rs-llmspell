//! Storage backends for knowledge graph
//!
//! This module provides a swappable backend design via the `GraphBackend` trait,
//! allowing different storage implementations:
//! - `PostgreSQL` (production, multi-tenant RLS, Phase 13b.5)
//! - `SQLite` (local persistent storage, Phase 13c.2.4)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use anyhow::Result;
use llmspell_core::types::storage::{Entity, Relationship, TemporalQuery};

/// Swappable backend trait for knowledge graph storage
///
/// Enables hot-swapping between different graph databases without
/// changing application code
#[async_trait]
pub trait GraphBackend: Send + Sync {
    /// Add a new entity
    async fn add_entity(&self, entity: Entity) -> Result<String>;

    /// Update an existing entity
    async fn update_entity(
        &self,
        id: &str,
        changes: HashMap<String, serde_json::Value>,
    ) -> Result<()>;

    /// Get current version of an entity
    async fn get_entity(&self, id: &str) -> Result<Entity>;

    /// Get entity at specific event time (temporal query)
    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity>;

    /// Add a relationship
    async fn add_relationship(&self, relationship: Relationship) -> Result<String>;

    /// Get related entities
    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>>;

    /// Get all relationships for an entity
    ///
    /// Returns both outgoing (from this entity) and incoming (to this entity) relationships.
    ///
    /// # Arguments
    /// * `entity_id` - The entity ID to query
    ///
    /// # Returns
    /// All relationships involving this entity
    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    /// Execute temporal query
    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;

    /// Delete entities before timestamp
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;

    /// Multi-hop graph traversal with depth limit and cycle prevention
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
    /// # Example
    /// ```ignore
    /// // Find all entities within 2 hops connected by "knows" relationships
    /// let results = backend.traverse("entity-1", Some("knows"), 2, None).await?;
    /// for (entity, depth, path) in results {
    ///     println!("Found {} at depth {} via path {}", entity.name, depth, path);
    /// }
    /// ```
    async fn traverse(
        &self,
        start_entity: &str,
        relationship_type: Option<&str>,
        max_depth: usize,
        at_time: Option<DateTime<Utc>>,
    ) -> Result<Vec<(Entity, usize, String)>>;
}
