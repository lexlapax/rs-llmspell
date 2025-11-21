//! Export format definitions for storage data migration

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Versioned export format for storage data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportFormat {
    /// Format version (semantic versioning)
    pub version: String,
    /// Export timestamp
    pub exported_at: DateTime<Utc>,
    /// Source backend type ("postgresql" | "sqlite")
    pub source_backend: String,
    /// List of applied migrations (["V3", "V4", "V5", ...])
    pub migrations: Vec<String>,
    /// Exported data organized by migration/table
    pub data: serde_json::Value,
}

impl ExportFormat {
    /// Create a new export format with version 1.0
    pub fn new(source_backend: String, migrations: Vec<String>) -> Self {
        Self {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            source_backend,
            migrations,
            data: serde_json::json!({}),
        }
    }
}
