//! Export format definitions for storage data migration
//!
//! Defines versioned JSON schema for exporting data from PostgreSQL or SQLite
//! backends. The format is designed for lossless roundtrip migration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Versioned export format for storage data
///
/// Top-level structure containing metadata and exported data from all tables.
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
    /// Exported data organized by table/migration
    pub data: ExportData,
}

impl ExportFormat {
    /// Create a new export format with version 1.0
    pub fn new(source_backend: String, migrations: Vec<String>) -> Self {
        Self {
            version: "1.0".to_string(),
            exported_at: Utc::now(),
            source_backend,
            migrations,
            data: ExportData::default(),
        }
    }
}

/// Container for all exported data organized by domain
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExportData {
    /// Vector embeddings by dimension (384, 768, 1536, 3072)
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub vector_embeddings: HashMap<usize, Vec<VectorEmbeddingExport>>,

    /// Knowledge graph (entities and relationships)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub knowledge_graph: Option<KnowledgeGraphExport>,

    /// Procedural memory patterns
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub procedural_memory: Vec<PatternExport>,

    /// Agent state entries
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub agent_state: Vec<AgentStateExport>,

    /// Key-value store entries
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub kv_store: Vec<KVEntryExport>,

    /// Workflow states
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workflow_states: Vec<WorkflowStateExport>,

    /// Sessions
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sessions: Vec<SessionExport>,

    /// Artifacts and content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<ArtifactsExport>,

    /// Event log entries
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub event_log: Vec<EventExport>,

    /// Hook history entries
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hook_history: Vec<HookExport>,
}

// ============================================================================
// Vector Embeddings (V3)
// ============================================================================

/// Exported vector embedding with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEmbeddingExport {
    /// Vector ID (UUID as string)
    pub id: String,
    /// Tenant ID
    pub tenant_id: String,
    /// Scope (session:xxx, user:xxx, global)
    pub scope: String,
    /// Vector dimension
    pub dimension: usize,
    /// Embedding data (base64 encoded MessagePack or JSON array)
    pub embedding: String,
    /// Metadata (JSON)
    pub metadata: serde_json::Value,
    /// Created timestamp (Unix microseconds)
    pub created_at: i64,
    /// Updated timestamp (Unix microseconds)
    pub updated_at: i64,
}

// ============================================================================
// Knowledge Graph (V4)
// ============================================================================

/// Exported knowledge graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeGraphExport {
    /// Entity nodes
    pub entities: Vec<EntityExport>,
    /// Relationship edges
    pub relationships: Vec<RelationshipExport>,
}

/// Exported entity node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExport {
    pub entity_id: String,
    pub tenant_id: String,
    pub entity_type: String,
    pub name: String,
    pub properties: serde_json::Value,
    pub valid_time_start: i64,
    pub valid_time_end: i64,
    pub transaction_time_start: i64,
    pub transaction_time_end: i64,
    pub created_at: i64,
}

/// Exported relationship edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipExport {
    pub relationship_id: String,
    pub tenant_id: String,
    pub source_entity_id: String,
    pub target_entity_id: String,
    pub relationship_type: String,
    pub properties: serde_json::Value,
    pub valid_time_start: i64,
    pub valid_time_end: i64,
    pub transaction_time_start: i64,
    pub transaction_time_end: i64,
    pub created_at: i64,
}

// ============================================================================
// Procedural Memory (V5)
// ============================================================================

/// Exported procedural memory pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExport {
    pub pattern_id: String,
    pub tenant_id: String,
    pub scope: String,
    pub key: String,
    pub value: String,
    pub frequency: i32,
    pub first_seen: i64,
    pub last_seen: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

// ============================================================================
// Agent State (V6)
// ============================================================================

/// Exported agent state entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStateExport {
    pub state_id: String,
    pub tenant_id: String,
    pub agent_id: String,
    pub agent_type: String,
    pub state_data: serde_json::Value,
    pub schema_version: i32,
    pub data_version: i32,
    pub checksum: String,
    pub created_at: i64,
    pub updated_at: i64,
}

// ============================================================================
// KV Store (V7)
// ============================================================================

/// Exported key-value store entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVEntryExport {
    pub kv_id: String,
    pub tenant_id: String,
    pub key: String,
    pub value: String,            // Base64 encoded BLOB
    pub metadata: Option<String>, // Optional JSON metadata
    pub created_at: i64,
    pub updated_at: i64,
}

// ============================================================================
// Workflow States (V8)
// ============================================================================

/// Exported workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStateExport {
    pub tenant_id: String,
    pub workflow_id: String,
    pub workflow_name: Option<String>,
    pub state_data: serde_json::Value,
    pub current_step: i32,
    pub status: String,
    pub started_at: Option<i64>,
    pub completed_at: Option<i64>,
    pub last_updated: i64,
    pub created_at: i64,
}

// ============================================================================
// Sessions (V9)
// ============================================================================

/// Exported session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionExport {
    pub id: String,
    pub tenant_id: String,
    pub session_id: String,
    pub session_data: serde_json::Value,
    pub status: String,
    pub created_at: i64,
    pub last_accessed_at: i64,
    pub expires_at: Option<i64>,
    pub artifact_count: i32,
    pub updated_at: i64,
}

// ============================================================================
// Artifacts (V10)
// ============================================================================

/// Exported artifacts data (content + metadata)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactsExport {
    /// Content-addressed storage
    pub content: Vec<ArtifactContentExport>,
    /// Artifact metadata and references
    pub artifacts: Vec<ArtifactMetadataExport>,
}

/// Exported artifact content (content-addressed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactContentExport {
    pub tenant_id: String,
    pub content_hash: String,
    pub storage_type: String,
    /// Base64 encoded data
    pub data: Option<String>,
    pub size_bytes: i64,
    pub is_compressed: bool,
    pub original_size_bytes: Option<i64>,
    pub reference_count: i32,
    pub created_at: i64,
    pub last_accessed_at: i64,
}

/// Exported artifact metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadataExport {
    pub tenant_id: String,
    pub artifact_id: String,
    pub session_id: String,
    pub sequence: i64,
    pub content_hash: String,
    pub metadata: serde_json::Value,
    pub name: String,
    pub artifact_type: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: i64,
    pub created_by: Option<String>,
    pub version: i32,
    pub parent_artifact_id: Option<String>,
    pub tags: Vec<String>,
    pub stored_at: i64,
    pub updated_at: i64,
}

// ============================================================================
// Event Log (V11)
// ============================================================================

/// Exported event log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventExport {
    pub id: String,
    pub tenant_id: String,
    pub event_id: String,
    pub event_type: String,
    pub correlation_id: String,
    pub timestamp: i64,
    pub sequence: i64,
    pub language: String,
    pub payload: serde_json::Value,
}

// ============================================================================
// Hook History (V13)
// ============================================================================

/// Exported hook history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookExport {
    pub id: String,
    pub execution_id: String,
    pub tenant_id: String,
    pub hook_id: String,
    pub hook_type: String,
    pub correlation_id: String,
    pub hook_context: String, // Base64 encoded compressed BLOB
    pub result_data: serde_json::Value,
    pub timestamp: i64,
    pub duration_ms: i64,
}
