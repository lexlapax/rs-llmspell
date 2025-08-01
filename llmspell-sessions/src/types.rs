//! ABOUTME: Core type definitions for session management including SessionId, SessionStatus, and configuration types
//! ABOUTME: Provides strongly-typed identifiers and metadata structures for session lifecycle management

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Unique identifier for a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    /// Create a new unique session ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a session ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }

    /// Get the inner UUID as a string
    pub fn as_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for SessionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Current status of a session
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionStatus {
    /// Session is actively being used
    Active,
    /// Session is temporarily suspended
    Suspended,
    /// Session completed successfully
    Completed,
    /// Session failed with error
    Failed,
    /// Session has been archived
    Archived,
}

impl SessionStatus {
    /// Check if the session is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Archived)
    }

    /// Check if the session can be resumed
    pub fn can_resume(&self) -> bool {
        matches!(self, Self::Suspended)
    }

    /// Check if the session is active
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Suspended => write!(f, "suspended"),
            Self::Completed => write!(f, "completed"),
            Self::Failed => write!(f, "failed"),
            Self::Archived => write!(f, "archived"),
        }
    }
}

/// Configuration for session behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Maximum session duration in seconds (None for unlimited)
    pub max_duration_secs: Option<u64>,
    /// Auto-save interval in seconds (None to disable)
    pub auto_save_interval_secs: Option<u64>,
    /// Maximum number of artifacts to retain
    pub max_artifacts: Option<usize>,
    /// Enable automatic artifact collection
    pub auto_collect_artifacts: bool,
    /// Enable session replay recording
    pub enable_replay: bool,
    /// Session retention days (None for unlimited)
    pub retention_days: Option<u32>,
    /// Custom metadata for the session
    pub metadata: HashMap<String, serde_json::Value>,
    /// Resource limits for the session
    pub resource_limits: ResourceLimits,
    /// Hook configuration
    pub hook_config: HookConfig,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            max_duration_secs: Some(24 * 60 * 60), // 24 hours
            auto_save_interval_secs: Some(300),    // 5 minutes
            max_artifacts: Some(1000),
            auto_collect_artifacts: true,
            enable_replay: true,
            retention_days: Some(30),
            metadata: HashMap::new(),
            resource_limits: ResourceLimits::default(),
            hook_config: HookConfig::default(),
        }
    }
}

/// Resource limits for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<u64>,
    /// Maximum CPU time in seconds
    pub max_cpu_seconds: Option<u64>,
    /// Maximum number of operations
    pub max_operations: Option<u64>,
    /// Maximum storage size in bytes
    pub max_storage_bytes: Option<u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_bytes: Some(1024 * 1024 * 1024), // 1GB
            max_cpu_seconds: Some(3600),                // 1 hour
            max_operations: Some(100_000),
            max_storage_bytes: Some(10 * 1024 * 1024 * 1024), // 10GB
        }
    }
}

/// Hook configuration for session events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)] // These are legitimate configuration flags
pub struct HookConfig {
    /// Enable session start hooks
    pub on_start: bool,
    /// Enable session end hooks
    pub on_end: bool,
    /// Enable session suspend hooks
    pub on_suspend: bool,
    /// Enable session resume hooks
    pub on_resume: bool,
    /// Enable artifact creation hooks
    pub on_artifact_create: bool,
    /// Hook timeout in milliseconds
    pub timeout_ms: u64,
}

impl Default for HookConfig {
    fn default() -> Self {
        Self {
            on_start: true,
            on_end: true,
            on_suspend: true,
            on_resume: true,
            on_artifact_create: true,
            timeout_ms: 5000,
        }
    }
}

/// Metadata about a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Session identifier
    pub id: SessionId,
    /// Current session status
    pub status: SessionStatus,
    /// Session creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
    /// Session start timestamp (when first activated)
    pub started_at: Option<DateTime<Utc>>,
    /// Session end timestamp (when completed/failed)
    pub ended_at: Option<DateTime<Utc>>,
    /// User who created the session
    pub created_by: Option<String>,
    /// Session name/title
    pub name: Option<String>,
    /// Session description
    pub description: Option<String>,
    /// Number of artifacts in session
    pub artifact_count: usize,
    /// Total size of artifacts in bytes
    pub total_artifact_size: u64,
    /// Number of operations performed
    pub operation_count: u64,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Parent session ID if this is a child session
    pub parent_session_id: Option<SessionId>,
    /// Custom metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

impl SessionMetadata {
    /// Create new metadata for a session
    pub fn new(id: SessionId, name: Option<String>, created_by: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id,
            status: SessionStatus::Active,
            created_at: now,
            updated_at: now,
            started_at: Some(now),
            ended_at: None,
            created_by,
            name,
            description: None,
            artifact_count: 0,
            total_artifact_size: 0,
            operation_count: 0,
            tags: Vec::new(),
            parent_session_id: None,
            custom_metadata: HashMap::new(),
        }
    }

    /// Update the session status
    pub fn update_status(&mut self, status: SessionStatus) {
        self.status = status;
        self.updated_at = Utc::now();

        if status.is_terminal() && self.ended_at.is_none() {
            self.ended_at = Some(Utc::now());
        }
    }

    /// Calculate session duration
    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.ended_at) {
            (Some(start), Some(end)) => Some(end - start),
            (Some(start), None) => Some(Utc::now() - start),
            _ => None,
        }
    }

    /// Add a tag to the session
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let tag = tag.into();
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a tag from the session
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        let initial_len = self.tags.len();
        self.tags.retain(|t| t != tag);
        if self.tags.len() < initial_len {
            self.updated_at = Utc::now();
            true
        } else {
            false
        }
    }
}

/// Session creation options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateSessionOptions {
    /// Session name
    pub name: Option<String>,
    /// Session description
    pub description: Option<String>,
    /// User creating the session
    pub created_by: Option<String>,
    /// Initial tags
    pub tags: Vec<String>,
    /// Parent session ID
    pub parent_session_id: Option<SessionId>,
    /// Custom configuration (overrides defaults)
    pub config: Option<SessionConfig>,
    /// Initial metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Session query filters
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionQuery {
    /// Filter by status
    pub status: Option<SessionStatus>,
    /// Filter by creator
    pub created_by: Option<String>,
    /// Filter by tags (sessions must have all specified tags)
    pub tags: Vec<String>,
    /// Filter by parent session
    pub parent_session_id: Option<SessionId>,
    /// Filter by creation date (from)
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by creation date (to)
    pub created_before: Option<DateTime<Utc>>,
    /// Search in name and description
    pub search_text: Option<String>,
    /// Maximum number of results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
    /// Sort order
    pub sort_by: SessionSortBy,
    /// Sort direction
    pub sort_desc: bool,
}

/// Sort options for session queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionSortBy {
    /// Sort by creation date
    CreatedAt,
    /// Sort by update date
    UpdatedAt,
    /// Sort by session name
    Name,
    /// Sort by artifact count
    ArtifactCount,
    /// Sort by operation count
    OperationCount,
}

impl Default for SessionSortBy {
    fn default() -> Self {
        Self::UpdatedAt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_session_id() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);

        let uuid = Uuid::new_v4();
        let id3 = SessionId::from_uuid(uuid);
        assert_eq!(id3.as_uuid(), &uuid);

        let id_str = id1.to_string();
        let parsed_id = SessionId::from_str(&id_str).unwrap();
        assert_eq!(id1, parsed_id);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_session_status() {
        assert!(SessionStatus::Active.is_active());
        assert!(!SessionStatus::Suspended.is_active());
        assert!(SessionStatus::Suspended.can_resume());
        assert!(!SessionStatus::Active.can_resume());
        assert!(SessionStatus::Completed.is_terminal());
        assert!(!SessionStatus::Active.is_terminal());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_session_metadata() {
        let id = SessionId::new();
        let mut metadata = SessionMetadata::new(
            id,
            Some("Test Session".to_string()),
            Some("user123".to_string()),
        );

        assert_eq!(metadata.status, SessionStatus::Active);
        assert!(metadata.started_at.is_some());
        assert!(metadata.ended_at.is_none());

        metadata.update_status(SessionStatus::Completed);
        assert_eq!(metadata.status, SessionStatus::Completed);
        assert!(metadata.ended_at.is_some());

        metadata.add_tag("test");
        assert!(metadata.tags.contains(&"test".to_string()));

        metadata.add_tag("test"); // Duplicate
        assert_eq!(metadata.tags.len(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_resource_limits_default() {
        let limits = ResourceLimits::default();
        assert!(limits.max_memory_bytes.is_some());
        assert!(limits.max_cpu_seconds.is_some());
        assert!(limits.max_operations.is_some());
        assert!(limits.max_storage_bytes.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert!(config.auto_collect_artifacts);
        assert!(config.enable_replay);
        assert_eq!(config.auto_save_interval_secs, Some(300));
    }
}
