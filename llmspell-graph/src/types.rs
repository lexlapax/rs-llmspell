//! Core types for knowledge graph

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// An entity in the knowledge graph with bi-temporal tracking
///
/// # Bi-Temporal Semantics
/// - `event_time`: When the real-world event occurred (can be None if unknown)
/// - `ingestion_time`: When we learned about it (always set)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entity {
    /// Unique identifier for the entity
    pub id: String,

    /// Entity name/label (e.g., "Rust", "Python")
    pub name: String,

    /// Entity type/category (e.g., `programming_language`, "person", "concept")
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

    /// Create entity with explicit event time
    #[must_use]
    pub const fn with_event_time(mut self, event_time: DateTime<Utc>) -> Self {
        self.event_time = Some(event_time);
        self
    }

    /// Create entity with explicit ID
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
}

/// A relationship between two entities with bi-temporal tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Relationship {
    /// Unique identifier for the relationship
    pub id: String,

    /// Source entity ID
    pub from_entity: String,

    /// Target entity ID
    pub to_entity: String,

    /// Relationship type (e.g., `has_feature`, `works_at`, `caused_by`)
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

    /// Create relationship with explicit event time
    #[must_use]
    pub const fn with_event_time(mut self, event_time: DateTime<Utc>) -> Self {
        self.event_time = Some(event_time);
        self
    }

    /// Create relationship with explicit ID
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.id = id;
        self
    }
}

/// Query parameters for temporal graph queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalQuery {
    /// Entity type filter (optional)
    pub entity_type: Option<String>,

    /// Event time range (optional)
    pub event_time_start: Option<DateTime<Utc>>,
    pub event_time_end: Option<DateTime<Utc>>,

    /// Ingestion time range (optional)
    pub ingestion_time_start: Option<DateTime<Utc>>,
    pub ingestion_time_end: Option<DateTime<Utc>>,

    /// Property filters (key-value pairs)
    pub property_filters: Vec<(String, Value)>,

    /// Maximum number of results
    pub limit: Option<usize>,
}

impl TemporalQuery {
    /// Create an empty temporal query
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
    #[must_use]
    pub fn with_entity_type(mut self, entity_type: String) -> Self {
        self.entity_type = Some(entity_type);
        self
    }

    /// Filter by event time range
    #[must_use]
    pub const fn with_event_time_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.event_time_start = Some(start);
        self.event_time_end = Some(end);
        self
    }

    /// Filter by ingestion time range
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
    #[must_use]
    pub fn with_property(mut self, key: String, value: Value) -> Self {
        self.property_filters.push((key, value));
        self
    }

    /// Set result limit
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
