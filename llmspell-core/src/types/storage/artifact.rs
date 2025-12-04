//! Artifact storage types
//!
//! Types for persistent artifact storage with content-addressed deduplication.
//! Artifacts represent outputs, results, and data generated during workflow execution.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Content hash type (SHA-256 hex string)
pub type ContentHash = String;

/// Unique identifier for an artifact
///
/// Artifacts are content-addressed, meaning the content hash uniquely identifies
/// the artifact content. Session ID and metadata distinguish different artifacts
/// with potentially identical content.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::ArtifactId;
///
/// let id = ArtifactId::new(
///     "abc123".to_string(),
///     "session-456".to_string(),
/// );
/// assert_eq!(id.content_hash, "abc123");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId {
    /// Content hash (SHA-256) of the artifact
    pub content_hash: ContentHash,

    /// Session this artifact belongs to
    pub session_id: String,
}

impl ArtifactId {
    /// Create a new artifact ID
    ///
    /// # Arguments
    ///
    /// * `content_hash` - SHA-256 hash of artifact content
    /// * `session_id` - Owning session identifier
    #[must_use]
    pub fn new(content_hash: ContentHash, session_id: String) -> Self {
        Self {
            content_hash,
            session_id,
        }
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hash_preview = if self.content_hash.len() > 8 {
            &self.content_hash[..8]
        } else {
            &self.content_hash
        };
        write!(f, "{}/{hash_preview}", self.session_id)
    }
}

/// Type of artifact stored
///
/// Categorizes artifacts by their origin and purpose within the system.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::ArtifactType;
///
/// let artifact_type = ArtifactType::Code;
/// assert_eq!(artifact_type.as_str(), "code");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Source code artifacts
    Code,
    /// Data files (JSON, CSV, etc.)
    Data,
    /// Image files (PNG, JPG, etc.)
    Image,
    /// Document files (PDF, MD, TXT)
    Document,
    /// Binary files
    Binary,
}

impl ArtifactType {
    /// Get string representation
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Data => "data",
            Self::Image => "image",
            Self::Document => "document",
            Self::Binary => "binary",
        }
    }
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Persistent artifact data
///
/// Stores complete artifact information including content, metadata, and provenance.
/// Content is stored as bytes to support any file type. The content hash provides
/// content-addressed storage enabling deduplication.
///
/// # Content Hashing
///
/// The `content_hash` field should be calculated using SHA-256 of the raw content bytes.
/// Implementations should verify hash integrity when retrieving artifacts.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::{Artifact, ArtifactType};
/// use chrono::Utc;
/// use serde_json::json;
///
/// let artifact = Artifact {
///     artifact_id: llmspell_core::types::storage::ArtifactId::new(
///         "abc123".to_string(),
///         "session-456".to_string(),
///     ),
///     artifact_type: ArtifactType::Code,
///     content: b"fn main() {}".to_vec(),
///     metadata: json!({"language": "rust"}),
///     size_bytes: 12,
///     created_at: Utc::now(),
/// };
///
/// assert_eq!(artifact.size_bytes, 12);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Unique artifact identifier
    pub artifact_id: ArtifactId,

    /// Artifact type classification
    pub artifact_type: ArtifactType,

    /// Raw artifact content (bytes)
    ///
    /// Stored as `Vec<u8>` to support any file type. For text content,
    /// use `String::from_utf8()` to convert.
    pub content: Vec<u8>,

    /// Additional metadata (stored as JSONB in database)
    ///
    /// Common fields: name, description, mime_type, tags, custom attributes
    pub metadata: serde_json::Value,

    /// Size in bytes
    pub size_bytes: usize,

    /// Timestamp when artifact was created
    pub created_at: DateTime<Utc>,
}

impl Artifact {
    /// Create a new artifact
    ///
    /// # Arguments
    ///
    /// * `content_hash` - SHA-256 hash of content
    /// * `session_id` - Owning session identifier
    /// * `artifact_type` - Type classification
    /// * `content` - Raw content bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::{Artifact, ArtifactType};
    ///
    /// let content = b"Hello, world!";
    /// let artifact = Artifact::new(
    ///     "hash123".to_string(),
    ///     "session-1".to_string(),
    ///     ArtifactType::Document,
    ///     content.to_vec(),
    /// );
    ///
    /// assert_eq!(artifact.content, content);
    /// ```
    #[must_use]
    pub fn new(
        content_hash: ContentHash,
        session_id: String,
        artifact_type: ArtifactType,
        content: Vec<u8>,
    ) -> Self {
        let size_bytes = content.len();
        Self {
            artifact_id: ArtifactId::new(content_hash, session_id),
            artifact_type,
            content,
            metadata: serde_json::Value::Object(serde_json::Map::new()),
            size_bytes,
            created_at: Utc::now(),
        }
    }

    /// Create artifact with metadata
    ///
    /// # Arguments
    ///
    /// * `content_hash` - SHA-256 hash of content
    /// * `session_id` - Owning session identifier
    /// * `artifact_type` - Type classification
    /// * `content` - Raw content bytes
    /// * `metadata` - Additional metadata
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::{Artifact, ArtifactType};
    /// use serde_json::json;
    ///
    /// let artifact = Artifact::with_metadata(
    ///     "hash123".to_string(),
    ///     "session-1".to_string(),
    ///     ArtifactType::Code,
    ///     b"fn test() {}".to_vec(),
    ///     json!({"language": "rust", "name": "test.rs"}),
    /// );
    ///
    /// assert_eq!(artifact.metadata["language"], "rust");
    /// ```
    #[must_use]
    pub fn with_metadata(
        content_hash: ContentHash,
        session_id: String,
        artifact_type: ArtifactType,
        content: Vec<u8>,
        metadata: serde_json::Value,
    ) -> Self {
        let size_bytes = content.len();
        Self {
            artifact_id: ArtifactId::new(content_hash, session_id),
            artifact_type,
            content,
            metadata,
            size_bytes,
            created_at: Utc::now(),
        }
    }

    /// Get content as UTF-8 string
    ///
    /// # Returns
    ///
    /// `Ok(String)` if content is valid UTF-8, `Err` otherwise
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_core::types::storage::{Artifact, ArtifactType};
    ///
    /// let artifact = Artifact::new(
    ///     "hash".to_string(),
    ///     "session".to_string(),
    ///     ArtifactType::Document,
    ///     b"Hello".to_vec(),
    /// );
    ///
    /// assert_eq!(artifact.content_as_string().unwrap(), "Hello");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns error if content is not valid UTF-8
    pub fn content_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }

    /// Set metadata field
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    /// * `value` - Metadata value (JSON)
    pub fn set_metadata(&mut self, key: impl Into<String>, value: serde_json::Value) {
        if let serde_json::Value::Object(ref mut map) = self.metadata {
            map.insert(key.into(), value);
        }
    }

    /// Get metadata field
    ///
    /// # Arguments
    ///
    /// * `key` - Metadata key
    ///
    /// # Returns
    ///
    /// Reference to value if key exists, None otherwise
    #[must_use]
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        if let serde_json::Value::Object(ref map) = self.metadata {
            map.get(key)
        } else {
            None
        }
    }
}

/// Storage statistics for a session
///
/// Tracks artifact count and total storage usage for quota management
/// and cleanup operations.
///
/// # Examples
///
/// ```
/// use llmspell_core::types::storage::SessionStorageStats;
/// use chrono::Utc;
///
/// let stats = SessionStorageStats {
///     total_size_bytes: 1024000,
///     artifact_count: 10,
///     last_updated: Utc::now(),
/// };
///
/// assert_eq!(stats.total_size_bytes, 1024000);
/// assert_eq!(stats.artifact_count, 10);
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStorageStats {
    /// Total storage size in bytes
    pub total_size_bytes: usize,

    /// Number of artifacts stored
    pub artifact_count: usize,

    /// Last time stats were updated
    pub last_updated: DateTime<Utc>,
}

impl SessionStorageStats {
    /// Create new stats
    #[must_use]
    pub fn new() -> Self {
        Self {
            total_size_bytes: 0,
            artifact_count: 0,
            last_updated: Utc::now(),
        }
    }

    /// Update stats after adding artifact
    pub fn add_artifact(&mut self, size_bytes: usize) {
        self.total_size_bytes = self.total_size_bytes.saturating_add(size_bytes);
        self.artifact_count = self.artifact_count.saturating_add(1);
        self.last_updated = Utc::now();
    }

    /// Update stats after removing artifact
    pub fn remove_artifact(&mut self, size_bytes: usize) {
        self.total_size_bytes = self.total_size_bytes.saturating_sub(size_bytes);
        self.artifact_count = self.artifact_count.saturating_sub(1);
        self.last_updated = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_id() {
        let id = ArtifactId::new("abc123".to_string(), "session-1".to_string());
        assert_eq!(id.content_hash, "abc123");
        assert_eq!(id.session_id, "session-1");

        let display = format!("{id}");
        assert!(display.contains("session-1"));
        assert!(display.contains("abc123"));
    }

    #[test]
    fn test_artifact_type() {
        assert_eq!(ArtifactType::Code.as_str(), "code");
        assert_eq!(ArtifactType::Data.as_str(), "data");
        assert_eq!(ArtifactType::Image.as_str(), "image");
        assert_eq!(ArtifactType::Document.as_str(), "document");
        assert_eq!(ArtifactType::Binary.as_str(), "binary");
    }

    #[test]
    fn test_artifact_new() {
        let content = b"test content";
        let artifact = Artifact::new(
            "hash123".to_string(),
            "session-1".to_string(),
            ArtifactType::Code,
            content.to_vec(),
        );

        assert_eq!(artifact.content, content);
        assert_eq!(artifact.size_bytes, content.len());
        assert_eq!(artifact.artifact_type, ArtifactType::Code);
    }

    #[test]
    fn test_artifact_with_metadata() {
        let metadata = serde_json::json!({"name": "test.rs", "language": "rust"});
        let artifact = Artifact::with_metadata(
            "hash123".to_string(),
            "session-1".to_string(),
            ArtifactType::Code,
            b"fn main() {}".to_vec(),
            metadata.clone(),
        );

        assert_eq!(artifact.metadata, metadata);
    }

    #[test]
    fn test_artifact_content_as_string() {
        let artifact = Artifact::new(
            "hash".to_string(),
            "session".to_string(),
            ArtifactType::Document,
            b"Hello, world!".to_vec(),
        );

        let content_str = artifact.content_as_string().unwrap();
        assert_eq!(content_str, "Hello, world!");

        // Test invalid UTF-8
        let invalid_artifact = Artifact::new(
            "hash".to_string(),
            "session".to_string(),
            ArtifactType::Binary,
            vec![0xFF, 0xFE, 0xFD],
        );
        assert!(invalid_artifact.content_as_string().is_err());
    }

    #[test]
    fn test_artifact_metadata_operations() {
        let mut artifact = Artifact::new(
            "hash".to_string(),
            "session".to_string(),
            ArtifactType::Code,
            b"code".to_vec(),
        );

        // Set metadata
        artifact.set_metadata("language", serde_json::json!("rust"));
        artifact.set_metadata("version", serde_json::json!(1));

        // Get metadata
        assert_eq!(
            artifact.get_metadata("language"),
            Some(&serde_json::json!("rust"))
        );
        assert_eq!(
            artifact.get_metadata("version"),
            Some(&serde_json::json!(1))
        );
        assert_eq!(artifact.get_metadata("missing"), None);
    }

    #[test]
    fn test_session_storage_stats() {
        let mut stats = SessionStorageStats::new();
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.artifact_count, 0);

        // Add artifacts
        stats.add_artifact(1024);
        assert_eq!(stats.total_size_bytes, 1024);
        assert_eq!(stats.artifact_count, 1);

        stats.add_artifact(2048);
        assert_eq!(stats.total_size_bytes, 3072);
        assert_eq!(stats.artifact_count, 2);

        // Remove artifact
        stats.remove_artifact(1024);
        assert_eq!(stats.total_size_bytes, 2048);
        assert_eq!(stats.artifact_count, 1);
    }

    #[test]
    fn test_stats_saturation() {
        let mut stats = SessionStorageStats::new();

        // Test saturating_sub - should not underflow
        stats.remove_artifact(100);
        assert_eq!(stats.total_size_bytes, 0);
        assert_eq!(stats.artifact_count, 0);

        // Test saturating_add
        stats.total_size_bytes = usize::MAX;
        stats.add_artifact(100); // Should saturate at MAX
        assert_eq!(stats.total_size_bytes, usize::MAX);
    }
}
