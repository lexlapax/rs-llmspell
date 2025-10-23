//! `SurrealDB` backend implementation for knowledge graph
//!
//! This module will be fully implemented in Task 13.2.3.
//! For now, this is a stub to ensure crate structure compiles.

use crate::error::Result;
use crate::traits::KnowledgeGraph;
use crate::types::{Entity, Relationship, TemporalQuery};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::path::PathBuf;

/// `SurrealDB` backend for knowledge graph (embedded mode)
///
/// # Architecture
/// - Uses `SurrealDB` embedded mode (`file://path`)
/// - No external server required
/// - Thread-safe with Arc/RwLock
/// - CI-friendly (uses temp dirs for tests)
///
/// # Implementation Status
/// - ⏳ Stub created in Task 13.2.1
/// - ⏳ Will be fully implemented in Task 13.2.3
#[derive(Debug)]
pub struct SurrealDBBackend {
    /// Path to data directory (unused until Task 13.2.3)
    #[allow(dead_code)]
    data_dir: PathBuf,
}

impl SurrealDBBackend {
    /// Create new `SurrealDB` backend with embedded mode
    ///
    /// # Arguments
    /// * `data_dir` - Directory for database files
    ///
    /// # Returns
    /// Configured backend instance
    ///
    /// # Implementation Note
    /// This is a stub - will be implemented in Task 13.2.3
    #[must_use]
    pub const fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }
}

#[async_trait]
impl KnowledgeGraph for SurrealDBBackend {
    async fn add_entity(&self, _entity: Entity) -> Result<String> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn update_entity(
        &self,
        _id: &str,
        _changes: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn get_entity(&self, _id: &str) -> Result<Entity> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn get_entity_at(&self, _id: &str, _event_time: DateTime<Utc>) -> Result<Entity> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn add_relationship(&self, _relationship: Relationship) -> Result<String> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn get_related(&self, _entity_id: &str, _relationship_type: &str) -> Result<Vec<Entity>> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn query_temporal(&self, _query: TemporalQuery) -> Result<Vec<Entity>> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }

    async fn delete_before(&self, _timestamp: DateTime<Utc>) -> Result<usize> {
        unimplemented!("Task 13.2.3: Implement SurrealDB backend")
    }
}
