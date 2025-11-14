//! Semantic memory implementations (bi-temporal knowledge graph)
//!
//! Semantic memory wraps the knowledge graph layer (`llmspell-graph`) to provide
//! the `SemanticMemory` trait interface. This allows memory management to work
//! with knowledge graphs through a consistent API.
//!
//! # Architecture
//!
//! ```text
//! SemanticMemory trait → GraphSemanticMemory wrapper → KnowledgeGraph trait → PostgreSQL/SQLite Backend
//! ```
//!
//! # Types
//!
//! Entity and Relationship types are re-exported from `llmspell-graph` to avoid duplication.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};

use crate::error::{MemoryError, Result};
use crate::traits::SemanticMemory;

// Re-export graph types as canonical types
pub use llmspell_graph::types::{Entity, Relationship};

/// Semantic memory implementation using knowledge graph backend
///
/// Wraps any `KnowledgeGraph` implementation to provide the `SemanticMemory` trait.
/// This allows seamless integration between memory management and graph storage.
pub struct GraphSemanticMemory {
    graph: Arc<dyn llmspell_graph::traits::KnowledgeGraph>,
}

impl GraphSemanticMemory {
    /// Create new semantic memory from knowledge graph backend
    pub fn new(graph: Arc<dyn llmspell_graph::traits::KnowledgeGraph>) -> Self {
        Self { graph }
    }

    /// Create semantic memory with `SQLite` backend
    ///
    /// Uses `SQLite` bi-temporal graph storage with transaction-level isolation.
    ///
    /// # Arguments
    ///
    /// * `sqlite_backend` - `SQLite` backend instance
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use llmspell_storage::backends::sqlite::SqliteBackend;
    /// # use llmspell_memory::semantic::GraphSemanticMemory;
    /// # async fn example() -> llmspell_memory::Result<()> {
    /// let sqlite_backend = Arc::new(SqliteBackend::new(llmspell_storage::backends::sqlite::SqliteConfig::in_memory()).await?);
    /// let semantic = GraphSemanticMemory::new_with_sqlite(sqlite_backend);
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn new_with_sqlite(
        sqlite_backend: Arc<llmspell_storage::backends::sqlite::SqliteBackend>,
    ) -> Self {
        use llmspell_storage::backends::sqlite::SqliteGraphStorage;
        let graph = SqliteGraphStorage::new(sqlite_backend);
        Self::new(Arc::new(graph))
    }

    /// Create semantic memory with `PostgreSQL` backend
    ///
    /// Uses `PostgreSQL` bi-temporal graph storage with RLS tenant isolation.
    ///
    /// # Arguments
    ///
    /// * `postgres_backend` - `PostgreSQL` backend instance with connection pool
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use std::sync::Arc;
    /// # use llmspell_storage::{PostgresBackend, PostgresConfig};
    /// # use llmspell_memory::semantic::GraphSemanticMemory;
    /// # async fn example() -> llmspell_memory::Result<()> {
    /// let pg_config = PostgresConfig::new("postgresql://localhost/llmspell");
    /// let pg_backend = Arc::new(PostgresBackend::new(pg_config).await?);
    /// let semantic = GraphSemanticMemory::new_with_postgres(pg_backend);
    /// # Ok(())
    /// # }
    /// ```
    #[cfg(feature = "postgres")]
    #[must_use]
    pub fn new_with_postgres(postgres_backend: Arc<llmspell_storage::PostgresBackend>) -> Self {
        use llmspell_storage::backends::postgres::PostgresGraphStorage;
        let graph = PostgresGraphStorage::new(postgres_backend);
        Self::new(Arc::new(graph))
    }
}

#[async_trait]
impl SemanticMemory for GraphSemanticMemory {
    async fn upsert_entity(&self, entity: Entity) -> Result<()> {
        info!(
            "Upserting entity: id={}, type={}, name={}",
            entity.id, entity.entity_type, entity.name
        );
        trace!("Entity properties: {:?}", entity.properties);

        // For upsert semantics, we use add_entity which creates new entity
        // In the future, this could check if entity exists and update
        self.graph.add_entity(entity).await.map_err(|e| {
            error!("Failed to upsert entity: {}", e);
            MemoryError::Storage(e.to_string())
        })?;

        debug!("Entity upserted successfully");
        Ok(())
    }

    async fn get_entity(&self, id: &str) -> Result<Option<Entity>> {
        debug!("Retrieving entity: id={}", id);

        match self.graph.get_entity(id).await {
            Ok(entity) => {
                trace!(
                    "Entity found: name={}, type={}",
                    entity.name,
                    entity.entity_type
                );
                Ok(Some(entity))
            }
            Err(llmspell_graph::error::GraphError::EntityNotFound(_)) => {
                debug!("Entity not found: {}", id);
                Ok(None)
            }
            Err(e) => {
                error!("Failed to retrieve entity {}: {}", id, e);
                Err(MemoryError::Storage(e.to_string()))
            }
        }
    }

    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Option<Entity>> {
        match self.graph.get_entity_at(id, event_time).await {
            Ok(entity) => Ok(Some(entity)),
            Err(llmspell_graph::error::GraphError::EntityNotFound(_)) => Ok(None),
            Err(e) => Err(MemoryError::Storage(e.to_string())),
        }
    }

    async fn add_relationship(&self, relationship: Relationship) -> Result<()> {
        info!(
            "Adding relationship: type={}, from={}, to={}",
            relationship.relationship_type, relationship.from_entity, relationship.to_entity
        );
        trace!("Relationship properties: {:?}", relationship.properties);

        self.graph
            .add_relationship(relationship)
            .await
            .map_err(|e| {
                error!("Failed to add relationship: {}", e);
                MemoryError::Storage(e.to_string())
            })?;

        debug!("Relationship added successfully");
        Ok(())
    }

    async fn get_relationships(&self, entity_id: &str) -> Result<Vec<Relationship>> {
        debug!("Getting relationships for entity: id={}", entity_id);
        warn!("get_relationships not fully implemented - KnowledgeGraph trait needs expansion");

        // Get outgoing relationships
        // Note: Current KnowledgeGraph trait only has get_related which returns entities
        // For now, we'll return empty vec as full relationship API needs expansion
        // TODO: Expand KnowledgeGraph trait with get_relationships method
        let _ = entity_id;
        Ok(Vec::new())
    }

    async fn query_by_type(&self, entity_type: &str) -> Result<Vec<Entity>> {
        debug!("Querying entities by type: entity_type={}", entity_type);

        let query =
            llmspell_graph::types::TemporalQuery::new().with_entity_type(entity_type.to_string());

        let entities = self.graph.query_temporal(query).await.map_err(|e| {
            error!("Failed to query entities by type {}: {}", entity_type, e);
            MemoryError::Storage(e.to_string())
        })?;

        info!("Query by type returned {} entities", entities.len());
        trace!(
            "Entity names: {:?}",
            entities.iter().map(|e| &e.name).collect::<Vec<_>>()
        );

        Ok(entities)
    }

    async fn delete_entity(&self, id: &str) -> Result<()> {
        // Soft delete by updating entity with tombstone marker
        // For now, use update_entity with empty changes
        // TODO: Add explicit delete/tombstone support to KnowledgeGraph trait
        self.graph
            .update_entity(id, HashMap::new())
            .await
            .map_err(|e| MemoryError::Storage(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_graph_semantic_memory_create() {
        let backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .unwrap(),
        );
        backend.run_migrations().await.unwrap();
        let memory = GraphSemanticMemory::new_with_sqlite(backend);
        // Just verify it was created without panicking
        assert!(Arc::strong_count(&memory.graph) > 0);
    }

    #[tokio::test]
    async fn test_upsert_and_get_entity() {
        let backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .unwrap(),
        );
        backend.run_migrations().await.unwrap();
        let memory = GraphSemanticMemory::new_with_sqlite(backend);

        let entity = Entity::new(
            "Rust".into(),
            "programming_language".into(),
            json!({"paradigm": "multi-paradigm"}),
        );
        let id = entity.id.clone();

        memory.upsert_entity(entity).await.unwrap();

        let retrieved = memory.get_entity(&id).await.unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.name, "Rust");
        assert_eq!(retrieved.entity_type, "programming_language");
    }

    #[tokio::test]
    async fn test_get_nonexistent_entity() {
        let backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .unwrap(),
        );
        backend.run_migrations().await.unwrap();
        let memory = GraphSemanticMemory::new_with_sqlite(backend);

        let result = memory.get_entity("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_query_by_type() {
        let backend = Arc::new(
            llmspell_storage::backends::sqlite::SqliteBackend::new(
                llmspell_storage::backends::sqlite::SqliteConfig::in_memory(),
            )
            .await
            .unwrap(),
        );
        backend.run_migrations().await.unwrap();
        let memory = GraphSemanticMemory::new_with_sqlite(backend);

        memory
            .upsert_entity(Entity::new("Rust".into(), "language".into(), json!({})))
            .await
            .unwrap();

        memory
            .upsert_entity(Entity::new("Python".into(), "language".into(), json!({})))
            .await
            .unwrap();

        let results = memory.query_by_type("language").await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
