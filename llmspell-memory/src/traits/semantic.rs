//! Semantic memory trait (to be implemented in Task 13.1.2)

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::error::Result;

/// Entity in the knowledge graph
#[derive(Debug, Clone)]
pub struct Entity {
    pub id: String,
    pub entity_type: String,
    pub name: String,
    pub properties: Value,
    pub event_time: DateTime<Utc>,
    pub ingestion_time: DateTime<Utc>,
}

/// Relationship between entities
#[derive(Debug, Clone)]
pub struct Relationship {
    pub id: String,
    pub from_entity: String,
    pub to_entity: String,
    pub relationship_type: String,
    pub properties: Value,
    pub event_time: DateTime<Utc>,
    pub ingestion_time: DateTime<Utc>,
}

/// Semantic memory stores bi-temporal knowledge graph
#[async_trait]
pub trait SemanticMemory: Send + Sync {
    /// Add or update an entity
    async fn upsert_entity(&self, entity: Entity) -> Result<()>;

    /// Get entity by ID (at current time)
    async fn get_entity(&self, id: &str) -> Result<Option<Entity>>;

    /// Get entity at a specific point in time
    async fn get_entity_at(
        &self,
        id: &str,
        event_time: DateTime<Utc>,
    ) -> Result<Option<Entity>>;

    /// Add a relationship between entities
    async fn add_relationship(&self, relationship: Relationship) -> Result<()>;

    /// Get relationships for an entity
    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>>;

    /// Query entities by type
    async fn query_by_type(&self, entity_type: &str) -> Result<Vec<Entity>>;

    /// Delete entity (soft delete with ingestion time)
    async fn delete_entity(&self, id: &str) -> Result<()>;
}
