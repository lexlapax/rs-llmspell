//! Core types for memory system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Entry in episodic memory (a recorded interaction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicEntry {
    /// Unique identifier
    #[serde(default = "generate_id")]
    pub id: String,

    /// Session identifier
    pub session_id: String,

    /// Role (user, assistant, system)
    pub role: String,

    /// Content of the interaction
    pub content: String,

    /// When the interaction occurred
    pub timestamp: DateTime<Utc>,

    /// When this entry was ingested (for bi-temporal tracking)
    #[serde(default = "Utc::now")]
    pub ingestion_time: DateTime<Utc>,

    /// Additional metadata
    #[serde(default)]
    pub metadata: Value,

    /// Whether this entry has been consolidated into semantic memory
    #[serde(default)]
    pub processed: bool,

    /// Vector embedding (computed lazily)
    #[serde(skip)]
    pub embedding: Option<Vec<f32>>,
}

impl EpisodicEntry {
    /// Create a new episodic entry
    #[must_use]
    pub fn new(session_id: String, role: String, content: String) -> Self {
        Self {
            id: generate_id(),
            session_id,
            role,
            content,
            timestamp: Utc::now(),
            ingestion_time: Utc::now(),
            metadata: Value::Null,
            processed: false,
            embedding: None,
        }
    }

    /// Mark this entry as processed
    pub const fn mark_processed(&mut self) {
        self.processed = true;
    }
}

/// Consolidation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsolidationMode {
    /// Consolidate immediately
    Immediate,
    /// Consolidate in background daemon
    Background,
    /// Manual consolidation (test mode)
    Manual,
}

/// Result of consolidation operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationResult {
    /// Number of entries processed
    pub entries_processed: usize,

    /// Number of entities added
    pub entities_added: usize,

    /// Number of entities updated
    pub entities_updated: usize,

    /// Number of entities deleted
    pub entities_deleted: usize,

    /// Number of entries skipped (NOOP)
    pub entries_skipped: usize,

    /// Duration in milliseconds
    pub duration_ms: u64,
}

impl ConsolidationResult {
    /// Create empty result
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            entries_processed: 0,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            duration_ms: 0,
        }
    }
}

/// Generate a unique ID
fn generate_id() -> String {
    Uuid::new_v4().to_string()
}
