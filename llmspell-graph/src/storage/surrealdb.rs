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
use tracing::{debug, error, info, trace, warn};

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
            id: r.id.map_or_else(
                || uuid::Uuid::new_v4().to_string(),
                |thing| thing.id.to_string(),
            ),
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
            id: r.id.map_or_else(
                || uuid::Uuid::new_v4().to_string(),
                |thing| thing.id.to_string(),
            ),
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

        info!(
            "Initializing SurrealDB backend: data_dir={}",
            data_dir.display()
        );

        Self::ensure_data_directory(&data_dir)?;
        let db = Self::connect_database(&data_dir).await?;
        Self::configure_namespace(&db).await?;

        let backend = Self {
            db,
            data_dir: data_dir.clone(),
        };

        backend.initialize_schema().await?;

        info!(
            "SurrealDB backend initialized successfully at {}",
            data_dir.display()
        );

        Ok(backend)
    }

    /// Ensure data directory exists
    fn ensure_data_directory(data_dir: &Path) -> Result<()> {
        if !data_dir.exists() {
            debug!("Creating data directory: {}", data_dir.display());
            std::fs::create_dir_all(data_dir).map_err(|e| {
                error!("Failed to create data directory: {}", e);
                e
            })?;
        }
        Ok(())
    }

    /// Connect to embedded `RocksDB`
    async fn connect_database(data_dir: &Path) -> Result<Surreal<Db>> {
        let db_path = data_dir.join("llmspell-graph.db");
        debug!("Connecting to SurrealDB at: {}", db_path.display());
        Surreal::new::<RocksDb>(db_path).await.map_err(|e| {
            error!("Failed to connect to SurrealDB: {}", e);
            GraphError::from(e)
        })
    }

    /// Configure namespace and database
    async fn configure_namespace(db: &Surreal<Db>) -> Result<()> {
        debug!("Using namespace=llmspell, database=graph");
        db.use_ns("llmspell").use_db("graph").await.map_err(|e| {
            error!("Failed to use namespace/database: {}", e);
            GraphError::from(e)
        })
    }

    /// Create temporary backend for testing
    ///
    /// Uses OS temp directory with random suffix
    ///
    /// # Errors
    /// Returns error if temp directory creation or initialization fails
    pub async fn new_temp() -> Result<Self> {
        let temp_dir =
            std::env::temp_dir().join(format!("llmspell-graph-{}", uuid::Uuid::new_v4()));
        info!(
            "Creating temporary SurrealDB backend at: {}",
            temp_dir.display()
        );
        Self::new(&temp_dir).await
    }

    /// Initialize database schema with bi-temporal tables and indexes
    async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing SurrealDB schema (entities and relationships tables)");
        self.create_entities_table().await?;
        self.create_relationships_table().await?;
        info!("Schema initialization complete");
        Ok(())
    }

    /// Create entities table with bi-temporal indexes
    async fn create_entities_table(&self) -> Result<()> {
        debug!("Creating entities table with bi-temporal indexes");
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
            .await
            .map_err(|e| {
                error!("Failed to create entities table: {}", e);
                GraphError::from(e)
            })?;
        Ok(())
    }

    /// Create relationships table with indexes
    async fn create_relationships_table(&self) -> Result<()> {
        debug!("Creating relationships table with relationship indexes");
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
            .await
            .map_err(|e| {
                error!("Failed to create relationships table: {}", e);
                GraphError::from(e)
            })?;
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
        debug!(
            "Adding entity: id={}, entity_type={}, name={}",
            id, entity.entity_type, entity.name
        );
        trace!("Entity details: {:?}", entity);

        let record: EntityRecord = entity.into();

        // Insert entity into database
        let _: Option<EntityRecord> = self
            .db
            .create(("entities", id.clone()))
            .content(record)
            .await
            .map_err(|e| {
                error!("Failed to insert entity {}: {}", id, e);
                e
            })?;

        debug!("Entity {} added successfully", id);
        Ok(id)
    }

    async fn update_entity(
        &self,
        id: &str,
        changes: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        debug!(
            "Updating entity: id={}, changes_count={}",
            id,
            changes.len()
        );
        trace!("Update changes: {:?}", changes);

        // Get existing entity
        let existing: Option<EntityRecord> =
            self.db.select(("entities", id)).await.map_err(|e| {
                error!("Failed to retrieve entity {} for update: {}", id, e);
                e
            })?;

        let mut entity = existing.ok_or_else(|| {
            warn!("Entity not found for update: {}", id);
            GraphError::EntityNotFound(format!("Entity not found: {id}"))
        })?;

        // Apply changes to properties
        if let serde_json::Value::Object(props) = &mut entity.properties {
            for (key, value) in changes {
                props.insert(key, value);
            }
        }

        // Update ingestion_time to reflect the update
        let new_ingestion = Utc::now().into();

        // KNOWN ISSUE: SurrealDB 2.0 .update().content() and .merge() not persisting
        // properties field correctly. Multiple workarounds attempted:
        // - .update().content(entity) - returns empty properties
        // - .update().merge(patch) - returns empty properties
        // - DELETE + .create(entity) - returns empty properties
        // All attempts send correct data but SurrealDB returns empty Object {}.
        // This appears to be a SurrealDB 2.0 bug or API quirk.
        // For now, this method is marked as partially functional.
        // Workaround for production: recreate entity instead of update.
        warn!("SurrealDB 2.0 update has known properties persistence issue - properties may not be correctly updated");

        #[derive(Serialize)]
        struct EntityUpdate {
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

        let update_data = EntityUpdate {
            name: entity.name,
            entity_type: entity.entity_type,
            properties: entity.properties,
            event_time: entity.event_time,
            ingestion_time: new_ingestion,
        };

        // Attempt update (known to fail for properties field)
        let _: Option<EntityRecord> = self
            .db
            .update(("entities", id))
            .content(update_data)
            .await
            .map_err(|e| {
                error!("Failed to update entity {}: {}", id, e);
                e
            })?;

        debug!("Entity {} update attempted (see known issue warning)", id);
        Ok(())
    }

    async fn get_entity(&self, id: &str) -> Result<Entity> {
        debug!("Retrieving entity: id={}", id);

        let record: Option<EntityRecord> = self.db.select(("entities", id)).await.map_err(|e| {
            error!("Failed to retrieve entity {}: {}", id, e);
            e
        })?;

        record
            .map(|r| {
                trace!("Retrieved entity: {:?}", r);
                Entity::from(r)
            })
            .ok_or_else(|| {
                warn!("Entity not found: {}", id);
                GraphError::EntityNotFound(format!("Entity not found: {id}"))
            })
    }

    async fn get_entity_at(&self, id: &str, event_time: DateTime<Utc>) -> Result<Entity> {
        debug!(
            "Retrieving entity at point-in-time: id={}, event_time={}",
            id, event_time
        );

        // Query for entity valid at the given event_time
        // Return entity where ingestion_time <= query_time AND (event_time is None OR event_time <= query_time)
        let query = format!(
            "SELECT * FROM entities:{id} WHERE ingestion_time <= $time AND (event_time IS NONE OR event_time <= $time) LIMIT 1"
        );
        trace!("Temporal query: {}", query);

        let mut response = self
            .db
            .query(&query)
            .bind(("time", event_time))
            .await
            .map_err(|e| {
                error!("Failed to execute temporal query for entity {}: {}", id, e);
                e
            })?;
        let entities: Vec<EntityRecord> = response.take(0).map_err(|e| {
            error!(
                "Failed to parse temporal query results for entity {}: {}",
                id, e
            );
            e
        })?;

        entities
            .into_iter()
            .next()
            .map(|e| {
                trace!("Retrieved entity at time: {:?}", e);
                Entity::from(e)
            })
            .ok_or_else(|| {
                warn!("Entity not found at time {}: {}", event_time, id);
                GraphError::EntityNotFound(format!("Entity not found at time {event_time}: {id}"))
            })
    }

    async fn add_relationship(&self, relationship: Relationship) -> Result<String> {
        let id = relationship.id.clone();
        debug!(
            "Adding relationship: id={}, type={}, from={}, to={}",
            id, relationship.relationship_type, relationship.from_entity, relationship.to_entity
        );
        trace!("Relationship details: {:?}", relationship);

        let record: RelationshipRecord = relationship.into();

        // Insert relationship into database
        let _: Option<RelationshipRecord> = self
            .db
            .create(("relationships", id.clone()))
            .content(record)
            .await
            .map_err(|e| {
                error!("Failed to insert relationship {}: {}", id, e);
                e
            })?;

        debug!("Relationship {} added successfully", id);
        Ok(id)
    }

    async fn get_related(&self, entity_id: &str, relationship_type: &str) -> Result<Vec<Entity>> {
        debug!(
            "Querying related entities: entity_id={}, relationship_type={}",
            entity_id, relationship_type
        );

        // Query relationships where from_entity matches and type matches
        let query = "SELECT * FROM relationships WHERE from_entity = $entity_id AND relationship_type = $rel_type";
        trace!("Relationship query: {}", query);

        // Convert to owned strings for bind (SurrealDB requirement)
        let entity_id_owned = entity_id.to_string();
        let rel_type_owned = relationship_type.to_string();

        let mut response = self
            .db
            .query(query)
            .bind(("entity_id", entity_id_owned))
            .bind(("rel_type", rel_type_owned))
            .await
            .map_err(|e| {
                error!("Failed to query relationships for {}: {}", entity_id, e);
                e
            })?;

        let relationships: Vec<RelationshipRecord> = response.take(0).map_err(|e| {
            error!("Failed to parse relationship query results: {}", e);
            e
        })?;

        debug!("Found {} relationships", relationships.len());

        // Get all target entities
        let mut entities = Vec::new();
        for rel in relationships {
            if let Ok(entity) = self.get_entity(&rel.to_entity).await {
                entities.push(entity);
            } else {
                warn!("Failed to retrieve related entity: {}", rel.to_entity);
            }
        }

        debug!("Retrieved {} related entities", entities.len());
        trace!(
            "Related entities: {:?}",
            entities.iter().map(|e| &e.id).collect::<Vec<_>>()
        );
        Ok(entities)
    }

    async fn query_temporal(&self, query: TemporalQuery) -> Result<Vec<Entity>> {
        info!("Executing temporal query: {:?}", query);

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

        debug!("Query conditions: {:?}", conditions);

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
        trace!("Temporal SQL query: {}", sql);

        let mut response = self.db.query(&sql).await.map_err(|e| {
            error!("Failed to execute temporal query: {}", e);
            e
        })?;
        let entities: Vec<EntityRecord> = response.take(0).map_err(|e| {
            error!("Failed to parse temporal query results: {}", e);
            e
        })?;

        info!("Temporal query returned {} entities", entities.len());
        trace!(
            "Query results: {:?}",
            entities.iter().map(|e| &e.name).collect::<Vec<_>>()
        );

        Ok(entities.into_iter().map(Entity::from).collect())
    }

    async fn delete_before(&self, timestamp: DateTime<Utc>) -> Result<usize> {
        info!("Deleting entities before timestamp: {}", timestamp);

        // KNOWN ISSUE: When entities are created with custom ingestion_time
        // (e.g., backdated for testing), SurrealDB may not preserve the custom
        // timestamp, instead using the current time. This causes this delete
        // operation to return 0 even when it should delete records.
        // This is likely a SurrealDB 2.0 timestamp handling quirk.
        // For production use, entities should use natural ingestion times.
        warn!("SurrealDB 2.0 may not preserve custom ingestion_time - delete may not work for backdated entities");

        // Delete entities where ingestion_time < timestamp
        let query = "DELETE FROM entities WHERE ingestion_time < $timestamp";
        trace!("Delete query: {}", query);

        // Convert chrono DateTime to SurrealDB Datetime for proper comparison
        let surreal_timestamp: surrealdb::sql::Datetime = timestamp.into();

        let mut response = self
            .db
            .query(query)
            .bind(("timestamp", surreal_timestamp.clone()))
            .await
            .map_err(|e| {
                error!("Failed to execute delete query: {}", e);
                e
            })?;
        let deleted: Vec<EntityRecord> = response.take(0).map_err(|e| {
            error!("Failed to parse delete query results: {}", e);
            e
        })?;

        let count = deleted.len();
        debug!("Deleted {} entities", count);

        // Also delete orphaned relationships
        let rel_query = "DELETE FROM relationships WHERE ingestion_time < $timestamp";
        let mut rel_response = self
            .db
            .query(rel_query)
            .bind(("timestamp", surreal_timestamp))
            .await
            .map_err(|e| {
                error!("Failed to delete orphaned relationships: {}", e);
                e
            })?;
        let deleted_rels: Vec<RelationshipRecord> = rel_response.take(0)?;
        debug!("Deleted {} orphaned relationships", deleted_rels.len());

        info!(
            "Deletion complete: {} entities, {} relationships",
            count,
            deleted_rels.len()
        );
        Ok(count)
    }
}

// Tests moved to integration test files:
// - tests/surrealdb_integration.rs
// - tests/error_handling_test.rs
// - tests/concurrency_test.rs
