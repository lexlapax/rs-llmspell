//! `SurrealDB` backend implementation for knowledge graph
//!
//! Provides embedded, file-based graph storage with full bi-temporal support.

use crate::error::{GraphError, Result};
use crate::traits::KnowledgeGraph;
use crate::types::{Entity, Relationship, TemporalQuery};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use surrealdb::engine::local::{Db, RocksDb};
use surrealdb::Surreal;

/// `SurrealDB` backend for knowledge graph (embedded mode)
///
/// # Architecture
/// - Uses `SurrealDB` embedded mode with `RocksDB` storage
/// - File-based persistence at `<data_dir>/llmspell-graph.db`
/// - Thread-safe with Arc wrapper (clone-safe)
/// - CI-friendly (uses temp dirs for tests)
///
/// # Bi-Temporal Schema
/// - `entities` table: id, name, `entity_type`, properties, `event_time`, `ingestion_time`
/// - `relationships` table: id, `from_entity`, `to_entity`, `relationship_type`, properties, `event_time`, `ingestion_time`
/// - Indexes on name, `entity_type`, timestamps for fast queries
#[derive(Debug, Clone)]
pub struct SurrealDBBackend {
    /// `SurrealDB` connection (embedded `RocksDB`)
    db: Surreal<Db>,
    /// Path to data directory
    data_dir: PathBuf,
}

/// Internal entity representation for `SurrealDB` storage
#[derive(Debug, Serialize, Deserialize)]
struct EntityRecord {
    #[serde(skip_serializing)]
    id: Option<surrealdb::sql::Thing>,
    name: String,
    entity_type: String,
    properties: serde_json::Value,
    #[serde(
        default,
        serialize_with = "optional_datetime::serialize",
        deserialize_with = "optional_datetime::deserialize"
    )]
    event_time: Option<surrealdb::sql::Datetime>,
    #[serde(
        serialize_with = "datetime_serde::serialize",
        deserialize_with = "datetime_serde::deserialize"
    )]
    ingestion_time: surrealdb::sql::Datetime,
}

/// Custom serde module for datetime fields
mod datetime_serde {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use surrealdb::sql::Datetime;

    pub fn serialize<S>(dt: &Datetime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        dt.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Datetime, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DatetimeOrString {
            Datetime(Datetime),
            String(String),
        }

        match DatetimeOrString::deserialize(deserializer)? {
            DatetimeOrString::Datetime(dt) => Ok(dt),
            DatetimeOrString::String(s) => {
                // Remove SurrealDB datetime prefix if present
                let clean = s.trim_start_matches("d'").trim_end_matches('\'');
                let chrono_dt: DateTime<Utc> = clean.parse().map_err(serde::de::Error::custom)?;
                Ok(chrono_dt.into())
            }
        }
    }
}

/// Custom serde module for optional datetime fields
mod optional_datetime {
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serializer};
    use surrealdb::sql::Datetime;

    // Note: serde's serialize_with requires &FieldType, which is &Option<T> here.
    // Using Option<&T> would break serde's API contract, so we allow ref_option.
    #[allow(clippy::ref_option)]
    pub fn serialize<S>(dt: &Option<Datetime>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match dt.as_ref() {
            Some(d) => serializer.serialize_some(d),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Datetime>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum DatetimeOrString {
            Datetime(Datetime),
            String(String),
        }

        let opt = Option::<DatetimeOrString>::deserialize(deserializer)?;
        match opt {
            Some(DatetimeOrString::Datetime(dt)) => Ok(Some(dt)),
            Some(DatetimeOrString::String(s)) => {
                let clean = s.trim_start_matches("d'").trim_end_matches('\'');
                let chrono_dt: DateTime<Utc> = clean.parse().map_err(serde::de::Error::custom)?;
                Ok(Some(chrono_dt.into()))
            }
            None => Ok(None),
        }
    }
}

/// Internal relationship representation for `SurrealDB` storage
#[derive(Debug, Serialize, Deserialize)]
struct RelationshipRecord {
    #[serde(skip_serializing)]
    id: Option<surrealdb::sql::Thing>,
    from_entity: String,
    to_entity: String,
    relationship_type: String,
    properties: serde_json::Value,
    #[serde(
        default,
        serialize_with = "optional_datetime::serialize",
        deserialize_with = "optional_datetime::deserialize"
    )]
    event_time: Option<surrealdb::sql::Datetime>,
    #[serde(
        serialize_with = "datetime_serde::serialize",
        deserialize_with = "datetime_serde::deserialize"
    )]
    ingestion_time: surrealdb::sql::Datetime,
}

impl From<Entity> for EntityRecord {
    fn from(e: Entity) -> Self {
        Self {
            id: None, // Will be set by SurrealDB
            name: e.name,
            entity_type: e.entity_type,
            properties: e.properties,
            event_time: e.event_time.map(std::convert::Into::into),
            ingestion_time: e.ingestion_time.into(),
        }
    }
}

impl From<EntityRecord> for Entity {
    fn from(r: EntityRecord) -> Self {
        Self {
            id: r
                .id.map_or_else(|| uuid::Uuid::new_v4().to_string(), |thing| thing.id.to_string()),
            name: r.name,
            entity_type: r.entity_type,
            properties: r.properties,
            event_time: r.event_time.map(std::convert::Into::into),
            ingestion_time: r.ingestion_time.into(),
        }
    }
}

impl From<Relationship> for RelationshipRecord {
    fn from(r: Relationship) -> Self {
        Self {
            id: None, // Will be set by SurrealDB
            from_entity: r.from_entity,
            to_entity: r.to_entity,
            relationship_type: r.relationship_type,
            properties: r.properties,
            event_time: r.event_time.map(std::convert::Into::into),
            ingestion_time: r.ingestion_time.into(),
        }
    }
}

impl From<RelationshipRecord> for Relationship {
    fn from(r: RelationshipRecord) -> Self {
        Self {
            id: r
                .id.map_or_else(|| uuid::Uuid::new_v4().to_string(), |thing| thing.id.to_string()),
            from_entity: r.from_entity,
            to_entity: r.to_entity,
            relationship_type: r.relationship_type,
            properties: r.properties,
            event_time: r.event_time.map(std::convert::Into::into),
            ingestion_time: r.ingestion_time.into(),
        }
    }
}

impl SurrealDBBackend {
    /// Create new `SurrealDB` backend with embedded mode
    ///
    /// # Arguments
    /// * `data_dir` - Directory for database files
    ///
    /// # Returns
    /// Configured backend instance with initialized schema
    ///
    /// # Errors
    /// Returns error if database initialization or schema creation fails
    pub async fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        let data_dir = data_dir.as_ref().to_path_buf();

        // Create data directory if it doesn't exist
        if !data_dir.exists() {
            std::fs::create_dir_all(&data_dir)?;
        }

        // Connect to embedded RocksDB
        let db_path = data_dir.join("llmspell-graph.db");
        let db = Surreal::new::<RocksDb>(db_path).await?;

        // Use namespace and database
        db.use_ns("llmspell").use_db("graph").await?;

        let backend = Self { db, data_dir };

        // Initialize schema
        backend.initialize_schema().await?;

        Ok(backend)
    }

    /// Create temporary backend for testing
    ///
    /// Uses OS temp directory with random suffix
    ///
    /// # Errors
    /// Returns error if temp directory creation or initialization fails
    pub async fn new_temp() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join(format!("llmspell-graph-{}", uuid::Uuid::new_v4()));
        Self::new(&temp_dir).await
    }

    /// Initialize database schema with bi-temporal tables and indexes
    async fn initialize_schema(&self) -> Result<()> {
        // Define entities table
        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS entities SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS name ON entities TYPE string;
                 DEFINE FIELD IF NOT EXISTS entity_type ON entities TYPE string;
                 DEFINE FIELD IF NOT EXISTS properties ON entities TYPE object;
                 DEFINE FIELD IF NOT EXISTS event_time ON entities TYPE option<datetime>;
                 DEFINE FIELD IF NOT EXISTS ingestion_time ON entities TYPE datetime;
                 DEFINE INDEX IF NOT EXISTS idx_entity_name ON entities FIELDS name;
                 DEFINE INDEX IF NOT EXISTS idx_entity_type ON entities FIELDS entity_type;
                 DEFINE INDEX IF NOT EXISTS idx_event_time ON entities FIELDS event_time;
                 DEFINE INDEX IF NOT EXISTS idx_ingestion_time ON entities FIELDS ingestion_time;",
            )
            .await?;

        // Define relationships table
        self.db
            .query(
                "DEFINE TABLE IF NOT EXISTS relationships SCHEMAFULL;
                 DEFINE FIELD IF NOT EXISTS from_entity ON relationships TYPE string;
                 DEFINE FIELD IF NOT EXISTS to_entity ON relationships TYPE string;
                 DEFINE FIELD IF NOT EXISTS relationship_type ON relationships TYPE string;
                 DEFINE FIELD IF NOT EXISTS properties ON relationships TYPE object;
                 DEFINE FIELD IF NOT EXISTS event_time ON relationships TYPE option<datetime>;
                 DEFINE FIELD IF NOT EXISTS ingestion_time ON relationships TYPE datetime;
                 DEFINE INDEX IF NOT EXISTS idx_from_entity ON relationships FIELDS from_entity;
                 DEFINE INDEX IF NOT EXISTS idx_to_entity ON relationships FIELDS to_entity;
                 DEFINE INDEX IF NOT EXISTS idx_rel_type ON relationships FIELDS relationship_type;",
            )
            .await?;

        Ok(())
    }

    /// Get data directory path
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }
}

#[async_trait]
impl KnowledgeGraph for SurrealDBBackend {
    async fn add_entity(&self, entity: Entity) -> Result<String> {
        let id = entity.id.clone();
        let record: EntityRecord = entity.into();

        // Insert entity into database
        let _: Option<EntityRecord> = self
            .db
            .create(("entities", id.clone()))
            .content(record)
            .await?;

        Ok(id)
    }

    async fn update_entity(
        &self,
        id: &str,
        changes: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Get existing entity
        let existing: Option<EntityRecord> = self.db.select(("entities", id)).await?;

        let mut entity = existing.ok_or_else(|| {
            GraphError::EntityNotFound(format!("Entity not found: {id}"))
        })?;

        // Apply changes to properties
        if let serde_json::Value::Object(props) = &mut entity.properties {
            for (key, value) in changes {
                props.insert(key, value);
            }
        }

        // Update ingestion_time to reflect the update
        let new_ingestion_time: surrealdb::sql::Datetime = Utc::now().into();

        // Build struct for merge (not JSON)
        #[derive(Serialize)]
        struct UpdatePatch {
            properties: serde_json::Value,
            ingestion_time: surrealdb::sql::Datetime,
        }

        let patch = UpdatePatch {
            properties: entity.properties,
            ingestion_time: new_ingestion_time,
        };

        let _: Option<EntityRecord> = self
            .db
            .update(("entities", id))
            .merge(patch)
            .await?;

        Ok(())
    }

    async fn get_entity(&self, id: &str) -> Result<Entity> {
        let record: Option<EntityRecord> = self.db.select(("entities", id)).await?;

        record
            .map(Entity::from)
            .ok_or_else(|| GraphError::EntityNotFound(format!("Entity not found: {id}")))
    }

    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity> {
        // Query for entity valid at the given event_time
        // Return entity where ingestion_time <= query_time AND (event_time is None OR event_time <= query_time)
        let query = format!(
            "SELECT * FROM entities:{id} WHERE ingestion_time <= $time AND (event_time IS NONE OR event_time <= $time) LIMIT 1"
        );

        let mut response = self.db.query(query).bind(("time", event_time)).await?;
        let entities: Vec<EntityRecord> = response.take(0)?;

        entities
            .into_iter()
            .next()
            .map(Entity::from)
            .ok_or_else(|| {
                GraphError::EntityNotFound(format!("Entity not found at time {event_time}: {id}"))
            })
    }

    async fn add_relationship(&self, relationship: Relationship) -> Result<String> {
        let id = relationship.id.clone();
        let record: RelationshipRecord = relationship.into();

        // Insert relationship into database
        let _: Option<RelationshipRecord> = self
            .db
            .create(("relationships", id.clone()))
            .content(record)
            .await?;

        Ok(id)
    }

    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>> {
        // Query relationships where from_entity matches and type matches
        let query = "SELECT * FROM relationships WHERE from_entity = $entity_id AND relationship_type = $rel_type";

        // Convert to owned strings for bind (SurrealDB requirement)
        let entity_id_owned = entity_id.to_string();
        let rel_type_owned = relationship_type.to_string();

        let mut response = self
            .db
            .query(query)
            .bind(("entity_id", entity_id_owned))
            .bind(("rel_type", rel_type_owned))
            .await?;

        let relationships: Vec<RelationshipRecord> = response.take(0)?;

        // Get all target entities
        let mut entities = Vec::new();
        for rel in relationships {
            if let Ok(entity) = self.get_entity(&rel.to_entity).await {
                entities.push(entity);
            }
        }

        Ok(entities)
    }

    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>> {
        let mut conditions = Vec::new();

        // Build query conditions
        if let Some(entity_type) = &query.entity_type {
            conditions.push(format!("entity_type = '{entity_type}'"));
        }

        if let Some(start) = query.event_time_start {
            conditions.push(format!("event_time >= {start:?}"));
        }

        if let Some(end) = query.event_time_end {
            conditions.push(format!("event_time <= {end:?}"));
        }

        if let Some(start) = query.ingestion_time_start {
            conditions.push(format!("ingestion_time >= {start:?}"));
        }

        if let Some(end) = query.ingestion_time_end {
            conditions.push(format!("ingestion_time <= {end:?}"));
        }

        // Build WHERE clause
        let where_clause = if conditions.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", conditions.join(" AND "))
        };

        // Build LIMIT clause
        let limit_clause = query.limit.map_or(String::new(), |l| format!(" LIMIT {l}"));

        // Execute query
        let sql = format!("SELECT * FROM entities{where_clause}{limit_clause}");

        let mut response = self.db.query(sql).await?;
        let entities: Vec<EntityRecord> = response.take(0)?;

        Ok(entities.into_iter().map(Entity::from).collect())
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize> {
        // Delete entities where ingestion_time < timestamp
        let query = "DELETE FROM entities WHERE ingestion_time < $timestamp";

        let mut response = self.db.query(query).bind(("timestamp", timestamp)).await?;
        let deleted: Vec<EntityRecord> = response.take(0)?;

        let count = deleted.len();

        // Also delete orphaned relationships
        let _query = "DELETE FROM relationships WHERE ingestion_time < $timestamp";
        let _response = self.db.query(_query).bind(("timestamp", timestamp)).await?;

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_new_temp_backend() {
        let backend = SurrealDBBackend::new_temp().await.expect("Failed to create temp backend");
        assert!(backend.data_dir().exists());
    }

    #[tokio::test]
    async fn test_add_and_get_entity() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        let entity = Entity::new(
            "Rust".into(),
            "programming_language".into(),
            json!({"paradigm": "multi-paradigm"}),
        );

        let id = backend.add_entity(entity.clone()).await.unwrap();
        let retrieved = backend.get_entity(&id).await.unwrap();

        assert_eq!(retrieved.name, "Rust");
        assert_eq!(retrieved.entity_type, "programming_language");
    }

    #[tokio::test]
    async fn test_update_entity() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        let entity = Entity::new("Python".into(), "programming_language".into(), json!({}));
        let id = backend.add_entity(entity).await.unwrap();

        let mut changes = HashMap::new();
        changes.insert("version".into(), json!("3.12"));
        backend.update_entity(&id, changes).await.unwrap();

        let updated = backend.get_entity(&id).await.unwrap();
        assert_eq!(updated.properties["version"], "3.12");
    }

    #[tokio::test]
    async fn test_add_and_get_relationship() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        let entity1 = Entity::new("Rust".into(), "language".into(), json!({}));
        let entity2 = Entity::new("Memory Safety".into(), "feature".into(), json!({}));

        let id1 = backend.add_entity(entity1).await.unwrap();
        let id2 = backend.add_entity(entity2).await.unwrap();

        let rel = Relationship::new(id1.clone(), id2, "has_feature".into(), json!({}));
        let rel_id = backend.add_relationship(rel).await.unwrap();

        assert!(!rel_id.is_empty());
    }

    #[tokio::test]
    async fn test_get_related() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        let lang = Entity::new("Rust".into(), "language".into(), json!({}));
        let feat1 = Entity::new("Safety".into(), "feature".into(), json!({}));
        let feat2 = Entity::new("Speed".into(), "feature".into(), json!({}));

        let lang_id = backend.add_entity(lang).await.unwrap();
        let feat1_id = backend.add_entity(feat1).await.unwrap();
        let feat2_id = backend.add_entity(feat2).await.unwrap();

        backend
            .add_relationship(Relationship::new(
                lang_id.clone(),
                feat1_id,
                "has_feature".into(),
                json!({}),
            ))
            .await
            .unwrap();

        backend
            .add_relationship(Relationship::new(lang_id.clone(), feat2_id, "has_feature".into(), json!({})))
            .await
            .unwrap();

        let related = backend.get_related(&lang_id, "has_feature").await.unwrap();
        assert_eq!(related.len(), 2);
    }

    #[tokio::test]
    async fn test_temporal_query() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        backend
            .add_entity(Entity::new("Rust".into(), "language".into(), json!({})))
            .await
            .unwrap();

        backend
            .add_entity(Entity::new("Python".into(), "language".into(), json!({})))
            .await
            .unwrap();

        let query = TemporalQuery::new()
            .with_entity_type("language".into())
            .with_limit(10);

        let results = backend.query_temporal(query).await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_delete_before() {
        let backend = SurrealDBBackend::new_temp().await.unwrap();

        let mut old_entity = Entity::new("Old".into(), "test".into(), json!({}));
        old_entity.ingestion_time = Utc::now() - chrono::Duration::days(30);
        backend.add_entity(old_entity).await.unwrap();

        backend
            .add_entity(Entity::new("New".into(), "test".into(), json!({})))
            .await
            .unwrap();

        let cutoff = Utc::now() - chrono::Duration::days(7);
        let deleted = backend.delete_before(cutoff).await.unwrap();

        assert_eq!(deleted, 1);
    }
}
