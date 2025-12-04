//! Knowledge graph types
//!
//! Domain types for bi-temporal knowledge graph operations including:
//! - `Entity`: Graph entity with bi-temporal tracking
//! - `Relationship`: Entity relationship with bi-temporal tracking
//! - `TemporalQuery`: Query builder for temporal graph queries
//!
//! # Bi-Temporal Semantics
//!
//! All types support two time dimensions:
//! - **Event Time**: When the real-world event occurred
//! - **Ingestion Time**: When we learned about it
//!
//! This enables:
//! - Time-travel queries (what did we know at time T?)
//! - Corrections (update past knowledge without losing history)
//! - Auditing (track knowledge evolution)
//!
//! Migrated from llmspell-graph/src/types.rs as part of Phase 13c.3.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

// ============================================================================
// Core Graph Types
// ============================================================================

/// An entity in the knowledge graph with bi-temporal tracking
///
/// Represents a node in the knowledge graph with full temporal versioning support.
/// Each entity has a unique ID, type classification, and arbitrary properties stored as JSON.
///
/// # Bi-Temporal Semantics
/// - `event_time`: When the real-world event occurred (can be None if unknown)
/// - `ingestion_time`: When we learned about it (always set)
///
/// # Builder Pattern
///
/// ```
/// # use llmspell_core::types::storage::graph::Entity;
/// # use serde_json::json;
/// # use chrono::Utc;
/// let entity = Entity::new(
///     "Rust".to_string(),
///     "programming_language".to_string(),
///     json!({"paradigm": "multi-paradigm"})
/// )
/// .with_event_time(Utc::now());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    /// Unique identifier for the entity
    pub id: String,

    /// Entity name/label (e.g., "Rust", "Python")
    pub name: String,

    /// Entity type/category (e.g., "programming_language", "person", "concept")
    pub entity_type: String,

    /// Additional properties as JSON
    pub properties: Value,

    /// When the event occurred in the real world (optional)
    pub event_time: Option<DateTime<Utc>>,

    /// When we ingested this knowledge (always present)
    pub ingestion_time: DateTime<Utc>,
}

impl Entity {
    /// Create a new entity with auto-generated ID and current ingestion time
    ///
    /// # Arguments
    ///
    /// * `name` - Entity name/label
    /// * `entity_type` - Entity type/category for classification
    /// * `properties` - Additional properties as JSON
    ///
    /// # Examples
    ///
    /// ```
    /// # use llmspell_core::types::storage::graph::Entity;
    /// # use serde_json::json;
    /// let entity = Entity::new(
    ///     "Claude".to_string(),
    ///     "ai_assistant".to_string(),
    ///     json!({"capabilities": ["coding", "analysis"]})
    /// );
    /// ```
    #[must_use]
    pub fn new(name: String, entity_type: String, properties: Value) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            entity_type,
            properties,
            event_time: None,
            ingestion_time: Utc::now(),
        }
    }

    /// Set explicit event time (when the real-world event occurred)
    ///
    /// # Arguments
    ///
    /// * `event_time` - Timestamp of the real-world event
    #[must_use]
    pub const fn with_event_time(mut self, event_time: DateTime<Utc>) -> Self {
        self.event_time = Some(event_time);
        self
    }

    /// Set explicit entity ID (instead of auto-generated UUID)
    ///
    /// # Arguments
    ///
    /// * `id` - Custom entity ID
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
}

/// A relationship between two entities with bi-temporal tracking
///
/// Represents a directed edge in the knowledge graph connecting two entities.
/// Supports arbitrary relationship types and properties with full temporal versioning.
///
/// # Bi-Temporal Semantics
/// - `event_time`: When the relationship was established in the real world
/// - `ingestion_time`: When we learned about the relationship
///
/// # Builder Pattern
///
/// ```
/// # use llmspell_core::types::storage::graph::Relationship;
/// # use serde_json::json;
/// let rel = Relationship::new(
///     "entity-1".to_string(),
///     "entity-2".to_string(),
///     "has_feature".to_string(),
///     json!({"strength": 0.9})
/// );
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Relationship {
    /// Unique identifier for the relationship
    pub id: String,

    /// Source entity ID
    pub from_entity: String,

    /// Target entity ID
    pub to_entity: String,

    /// Relationship type (e.g., "has_feature", "works_at", "caused_by")
    pub relationship_type: String,

    /// Additional properties as JSON
    pub properties: Value,

    /// When the relationship was established in the real world (optional)
    pub event_time: Option<DateTime<Utc>>,

    /// When we learned about this relationship (always present)
    pub ingestion_time: DateTime<Utc>,
}

impl Relationship {
    /// Create a new relationship with auto-generated ID and current ingestion time
    ///
    /// # Arguments
    ///
    /// * `from_entity` - Source entity ID
    /// * `to_entity` - Target entity ID
    /// * `relationship_type` - Type of relationship (e.g., "has_feature", "depends_on")
    /// * `properties` - Additional properties as JSON
    ///
    /// # Examples
    ///
    /// ```
    /// # use llmspell_core::types::storage::graph::Relationship;
    /// # use serde_json::json;
    /// let rel = Relationship::new(
    ///     "rust-lang".to_string(),
    ///     "memory-safety".to_string(),
    ///     "has_feature".to_string(),
    ///     json!({"priority": "high"})
    /// );
    /// ```
    #[must_use]
    pub fn new(
        from_entity: String,
        to_entity: String,
        relationship_type: String,
        properties: Value,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            from_entity,
            to_entity,
            relationship_type,
            properties,
            event_time: None,
            ingestion_time: Utc::now(),
        }
    }

    /// Set explicit event time (when the relationship was established)
    ///
    /// # Arguments
    ///
    /// * `event_time` - Timestamp when the relationship was established
    #[must_use]
    pub const fn with_event_time(mut self, event_time: DateTime<Utc>) -> Self {
        self.event_time = Some(event_time);
        self
    }

    /// Set explicit relationship ID (instead of auto-generated UUID)
    ///
    /// # Arguments
    ///
    /// * `id` - Custom relationship ID
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
}

/// Query parameters for temporal graph queries
///
/// Configures temporal queries on the knowledge graph supporting filters on:
/// - Entity type classification
/// - Event time range (when events occurred)
/// - Ingestion time range (when we learned about events)
/// - Property filters (arbitrary JSON property matching)
/// - Result limits
///
/// # Builder Pattern
///
/// ```
/// # use llmspell_core::types::storage::graph::TemporalQuery;
/// # use chrono::{Utc, Duration};
/// # use serde_json::json;
/// let query = TemporalQuery::new()
///     .with_entity_type("programming_language".to_string())
///     .with_event_time_range(
///         Utc::now() - Duration::days(30),
///         Utc::now()
///     )
///     .with_property("paradigm".to_string(), json!("functional"))
///     .with_limit(100);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalQuery {
    /// Entity type filter (optional)
    pub entity_type: Option<String>,

    /// Event time range start (optional)
    pub event_time_start: Option<DateTime<Utc>>,

    /// Event time range end (optional)
    pub event_time_end: Option<DateTime<Utc>>,

    /// Ingestion time range start (optional)
    pub ingestion_time_start: Option<DateTime<Utc>>,

    /// Ingestion time range end (optional)
    pub ingestion_time_end: Option<DateTime<Utc>>,

    /// Property filters (key-value pairs)
    pub property_filters: Vec<(String, Value)>,

    /// Maximum number of results
    pub limit: Option<usize>,
}

impl TemporalQuery {
    /// Create an empty temporal query (no filters applied)
    ///
    /// Use builder methods to add filters incrementally.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entity_type: None,
            event_time_start: None,
            event_time_end: None,
            ingestion_time_start: None,
            ingestion_time_end: None,
            property_filters: Vec::new(),
            limit: None,
        }
    }

    /// Filter by entity type
    ///
    /// # Arguments
    ///
    /// * `entity_type` - Entity type to filter by (e.g., "programming_language")
    #[must_use]
    pub fn with_entity_type(mut self, entity_type: String) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Filter by event time range
    ///
    /// Returns entities where the event occurred within the specified time range.
    ///
    /// # Arguments
    ///
    /// * `start` - Start of event time range (inclusive)
    /// * `end` - End of event time range (inclusive)
    #[must_use]
    pub const fn with_event_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.event_time_start = Some(start);
        self.event_time_end = Some(end);
        self
    }

    /// Filter by ingestion time range
    ///
    /// Returns entities that were ingested (learned about) within the specified time range.
    /// Useful for finding "what we knew at time T".
    ///
    /// # Arguments
    ///
    /// * `start` - Start of ingestion time range (inclusive)
    /// * `end` - End of ingestion time range (inclusive)
    #[must_use]
    pub const fn with_ingestion_time_range(
        mut self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Self {
        self.ingestion_time_start = Some(start);
        self.ingestion_time_end = Some(end);
        self
    }

    /// Add property filter
    ///
    /// Filters entities by matching a specific property value. Multiple property
    /// filters are combined with AND logic.
    ///
    /// # Arguments
    ///
    /// * `key` - Property key to filter on
    /// * `value` - Expected property value (JSON)
    #[must_use]
    pub fn with_property(mut self, key: String, value: Value) -> Self {
        self.property_filters.push((key, value));
        self
    }

    /// Set maximum number of results
    ///
    /// # Arguments
    ///
    /// * `limit` - Maximum number of entities to return
    #[must_use]
    pub const fn with_limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
}

impl Default for TemporalQuery {
    fn default() -> Self {
        Self::new()
    }
}
