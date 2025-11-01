//! Storage backends for knowledge graph
//!
//! This module provides a swappable backend design via the `GraphBackend` trait,
//! allowing different storage implementations:
//! - `SurrealDB` (embedded mode, Phase 13.2)
//! - `Neo4j` (future)
//! - `InMemory` (future, for testing)

pub mod surrealdb;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::error::Result;
use crate::types::{Entity, Relationship, TemporalQuery};

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

    /// Execute temporal query
    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>>;

    /// Delete entities before timestamp
    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize>;
}
