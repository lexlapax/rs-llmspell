//! ABOUTME: Artifact storage system with content-addressed deduplication using BLAKE3 hashing
//! ABOUTME: Provides efficient storage and retrieval of session artifacts with compression

use crate::SessionId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for an artifact (content hash)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Create artifact ID from hash string
    pub fn from_hash(hash: String) -> Self {
        Self(hash)
    }

    /// Get the hash string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type of artifact content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Plain text content
    Text,
    /// JSON structured data
    Json,
    /// Binary data
    Binary,
    /// Image data
    Image,
    /// Audio data
    Audio,
    /// Video data
    Video,
    /// Code/script content
    Code,
    /// Document (PDF, DOCX, etc.)
    Document,
    /// Archive (ZIP, TAR, etc.)
    Archive,
    /// Custom type with MIME type
    Custom,
}

impl ArtifactType {
    /// Get MIME type for the artifact type
    pub fn mime_type(&self) -> &str {
        match self {
            Self::Text => "text/plain",
            Self::Json => "application/json",
            Self::Binary | Self::Custom => "application/octet-stream",
            Self::Image => "image/*",
            Self::Audio => "audio/*",
            Self::Video => "video/*",
            Self::Code => "text/x-code",
            Self::Document => "application/*",
            Self::Archive => "application/x-archive",
        }
    }
}

/// Session artifact with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    /// Unique artifact ID (content hash)
    pub id: ArtifactId,
    /// Session that created this artifact
    pub session_id: SessionId,
    /// Type of artifact
    pub artifact_type: ArtifactType,
    /// Optional name for the artifact
    pub name: Option<String>,
    /// Optional description
    pub description: Option<String>,
    /// Size in bytes (uncompressed)
    pub size_bytes: u64,
    /// Compressed size in bytes
    pub compressed_size_bytes: u64,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Custom metadata
    pub metadata: serde_json::Value,
    /// MIME type if custom
    pub mime_type: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Artifact storage interface stub - to be implemented in Phase 6.2
#[derive(Debug, Clone)]
pub struct ArtifactStorage {
    // Implementation to be added in Phase 6.2
    _marker: std::marker::PhantomData<()>,
}

impl ArtifactStorage {
    /// Create new artifact storage
    pub fn new() -> Self {
        Self {
            _marker: std::marker::PhantomData,
        }
    }
}

impl Default for ArtifactStorage {
    fn default() -> Self {
        Self::new()
    }
}

// Full implementation will be added in Phase 6.2
