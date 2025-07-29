//! ABOUTME: `SessionArtifact` implementation that represents a single artifact within a session
//! ABOUTME: Provides serialization, compression, and metadata management for session artifacts

use super::types::{ArtifactId, ArtifactMetadata, ArtifactType, ContentHash};
use crate::{Result, SessionError, SessionId};
use blake3;
use chrono::{DateTime, Utc};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};

/// Threshold for automatic compression (10KB)
const COMPRESSION_THRESHOLD: usize = 10 * 1024;

/// A single artifact within a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    /// Unique identifier for this artifact
    pub id: ArtifactId,
    /// Artifact metadata
    pub metadata: ArtifactMetadata,
    /// The actual content (may be compressed)
    content: Vec<u8>,
    /// When this artifact was stored
    pub stored_at: DateTime<Utc>,
    /// Storage format version
    pub storage_version: u32,
}

impl SessionArtifact {
    /// Current storage format version
    pub const STORAGE_VERSION: u32 = 1;

    /// Create a new session artifact
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact metadata is invalid
    pub fn new(
        session_id: SessionId,
        sequence: u64,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
    ) -> Result<Self> {
        let mut artifact =
            Self::create_with_metadata(session_id, sequence, artifact_type, name, content, None)?;

        // Auto-compress if above threshold
        if artifact.metadata.size > COMPRESSION_THRESHOLD && !artifact.metadata.is_compressed {
            artifact.compress()?;
        }

        Ok(artifact)
    }

    /// Create artifact with custom metadata
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact metadata is invalid
    pub fn create_with_metadata(
        session_id: SessionId,
        sequence: u64,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
        created_by: Option<String>,
    ) -> Result<Self> {
        // Calculate content hash
        let content_hash = Self::calculate_hash(&content);

        // Create artifact ID
        let id = ArtifactId::new(content_hash, session_id, sequence);

        // Create metadata
        let mut metadata = ArtifactMetadata::new(name, artifact_type, content.len());
        metadata.created_by = created_by;

        // Validate metadata
        metadata
            .validate()
            .map_err(|e| SessionError::Validation(format!("Invalid artifact metadata: {e}")))?;

        Ok(Self {
            id,
            metadata,
            content,
            stored_at: Utc::now(),
            storage_version: Self::STORAGE_VERSION,
        })
    }

    /// Calculate blake3 hash of content
    pub fn calculate_hash(content: &[u8]) -> ContentHash {
        let hash = blake3::hash(content);
        hash.to_hex().to_string()
    }

    /// Create from parts (used during retrieval)
    ///
    /// # Errors
    ///
    /// Returns an error if metadata validation fails
    pub fn from_parts(
        id: ArtifactId,
        metadata: ArtifactMetadata,
        content: Vec<u8>,
        stored_at: DateTime<Utc>,
    ) -> Result<Self> {
        // Validate metadata
        metadata
            .validate()
            .map_err(|e| SessionError::Validation(format!("Invalid artifact metadata: {e}")))?;

        Ok(Self {
            id,
            metadata,
            content,
            stored_at,
            storage_version: Self::STORAGE_VERSION,
        })
    }

    /// Get the raw content (decompressing if necessary)
    ///
    /// # Errors
    ///
    /// Returns an error if decompression fails
    pub fn get_content(&self) -> Result<Vec<u8>> {
        if self.metadata.is_compressed {
            decompress_size_prepended(&self.content).map_err(|e| SessionError::General {
                message: format!("Decompression failed: {e}"),
                source: None,
            })
        } else {
            Ok(self.content.clone())
        }
    }

    /// Get content as string (assumes UTF-8)
    ///
    /// # Errors
    ///
    /// Returns an error if the content is not valid UTF-8 or decompression fails
    pub fn get_content_string(&self) -> Result<String> {
        let content = self.get_content()?;
        String::from_utf8(content)
            .map_err(|e| SessionError::Deserialization(format!("Invalid UTF-8: {e}")))
    }

    /// Set new content, updating hash and metadata
    ///
    /// # Errors
    ///
    /// Returns an error if compression fails
    pub fn set_content(&mut self, content: Vec<u8>) -> Result<()> {
        // Update version info
        self.metadata.version.previous_hash = Some(self.id.content_hash.clone());
        self.metadata.version.version += 1;
        self.metadata.version.created_at = Utc::now();

        // Calculate new hash
        let new_hash = Self::calculate_hash(&content);
        self.id.content_hash = new_hash;

        // Update metadata
        self.metadata.size = content.len();
        self.metadata.is_compressed = false;
        self.metadata.original_size = None;

        // Set content
        self.content = content;
        self.stored_at = Utc::now();

        // Auto-compress if needed
        if self.metadata.size > COMPRESSION_THRESHOLD {
            self.compress()?;
        }

        Ok(())
    }

    /// Compress the artifact content
    ///
    /// # Errors
    ///
    /// This function currently does not fail but returns Result for future compatibility
    pub fn compress(&mut self) -> Result<()> {
        if self.metadata.is_compressed {
            return Ok(());
        }

        let original_size = self.content.len();
        let compressed = compress_prepend_size(&self.content);

        // Only use compression if it actually reduces size
        if compressed.len() < original_size {
            self.content = compressed;
            self.metadata.is_compressed = true;
            self.metadata.original_size = Some(original_size);
            self.metadata.size = self.content.len();
        }

        Ok(())
    }

    /// Decompress the artifact content
    ///
    /// # Errors
    ///
    /// Returns an error if decompression fails
    pub fn decompress(&mut self) -> Result<()> {
        if !self.metadata.is_compressed {
            return Ok(());
        }

        let decompressed =
            decompress_size_prepended(&self.content).map_err(|e| SessionError::General {
                message: format!("Decompression failed: {e}"),
                source: None,
            })?;

        self.content = decompressed;
        self.metadata.is_compressed = false;
        self.metadata.size = self.content.len();
        self.metadata.original_size = None;

        Ok(())
    }

    /// Update MIME type based on content
    pub fn detect_mime_type(&mut self) {
        // Simple detection based on name extension
        if let Some(ext) = self.metadata.name.split('.').next_back() {
            self.metadata.mime_type = match ext.to_lowercase().as_str() {
                "json" => "application/json",
                "xml" => "application/xml",
                "txt" | "log" => "text/plain",
                "html" => "text/html",
                "css" => "text/css",
                "js" => "application/javascript",
                "png" => "image/png",
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "pdf" => "application/pdf",
                "zip" => "application/zip",
                _ => "application/octet-stream",
            }
            .to_string();
        }
    }

    /// Create a derived artifact from this one
    ///
    /// # Errors
    ///
    /// Returns an error if the new artifact creation fails
    pub fn derive(
        &self,
        new_name: String,
        new_content: Vec<u8>,
        new_type: ArtifactType,
    ) -> Result<Self> {
        let mut derived = Self::new(
            self.id.session_id,
            self.id.sequence + 1,
            new_type,
            new_name,
            new_content,
        )?;

        // Set parent reference
        derived.metadata.parent_artifact = Some(self.id.clone());

        // Copy relevant tags
        for tag in &self.metadata.tags {
            if tag != "original" {
                let _ = derived.metadata.add_tag(tag.clone());
            }
        }

        Ok(derived)
    }

    /// Check if content matches hash
    pub fn verify_integrity(&self) -> bool {
        let Ok(content) = self.get_content() else {
            return false;
        };

        let calculated_hash = Self::calculate_hash(&content);
        calculated_hash == self.id.content_hash
    }

    /// Get storage size (actual bytes stored)
    pub fn storage_size(&self) -> usize {
        self.content.len()
    }

    /// Get compression ratio if compressed
    pub fn compression_ratio(&self) -> Option<f64> {
        self.metadata.original_size.map(|original| {
            #[allow(clippy::cast_precision_loss)]
            let ratio = self.content.len() as f64 / original as f64;
            ratio
        })
    }
}

/// Additional storage utilities for `SessionArtifact`
impl SessionArtifact {
    /// Serialize to storage format with integrity check
    ///
    /// # Errors
    ///
    /// Returns an error if serialization fails
    pub fn to_storage_with_checksum(&self) -> Result<Vec<u8>> {
        #[derive(Serialize, Deserialize)]
        struct StoredArtifact {
            artifact: SessionArtifact,
            checksum: String,
        }

        let stored = StoredArtifact {
            artifact: self.clone(),
            checksum: self.id.content_hash.clone(),
        };

        bincode::serialize(&stored)
            .map_err(|e| SessionError::Serialization(format!("Failed to serialize artifact: {e}")))
    }

    /// Deserialize from storage format with integrity check
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization fails, checksum doesn't match, or version is incompatible
    pub fn from_storage_with_checksum(bytes: &[u8]) -> Result<Self> {
        #[derive(Serialize, Deserialize)]
        struct StoredArtifact {
            artifact: SessionArtifact,
            checksum: String,
        }

        let stored: StoredArtifact = bincode::deserialize(bytes).map_err(|e| {
            SessionError::Deserialization(format!("Failed to deserialize artifact: {e}"))
        })?;

        // Verify checksum matches
        if stored.checksum != stored.artifact.id.content_hash {
            return Err(SessionError::Validation(
                "Artifact checksum mismatch".to_string(),
            ));
        }

        // Verify storage version compatibility
        if stored.artifact.storage_version > Self::STORAGE_VERSION {
            return Err(SessionError::Validation(format!(
                "Artifact storage version {} is newer than supported version {}",
                stored.artifact.storage_version,
                Self::STORAGE_VERSION
            )));
        }

        Ok(stored.artifact)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_artifact_creation() {
        let session_id = SessionId::new();
        let content = b"Hello, World!".to_vec();

        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "greeting.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        assert_eq!(artifact.id.session_id, session_id);
        assert_eq!(artifact.id.sequence, 1);
        assert_eq!(artifact.metadata.name, "greeting.txt");
        assert_eq!(artifact.metadata.artifact_type, ArtifactType::UserInput);
        assert_eq!(artifact.get_content().unwrap(), content);
        assert!(!artifact.metadata.is_compressed); // Too small to compress
    }

    #[test]
    fn test_content_hashing() {
        let content1 = b"Test content".to_vec();
        let content2 = b"Different content".to_vec();

        let hash1 = SessionArtifact::calculate_hash(&content1);
        let hash2 = SessionArtifact::calculate_hash(&content2);
        let hash3 = SessionArtifact::calculate_hash(&content1);

        assert_ne!(hash1, hash2);
        assert_eq!(hash1, hash3); // Same content = same hash
    }

    #[test]
    fn test_compression() {
        let session_id = SessionId::new();
        let content = "x".repeat(20 * 1024); // 20KB of 'x'

        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::SystemGenerated,
            "large.txt".to_string(),
            content.as_bytes().to_vec(),
        )
        .unwrap();

        // Should be auto-compressed
        assert!(artifact.metadata.is_compressed);
        assert!(artifact.metadata.original_size.is_some());

        // Verify content is still accessible
        assert_eq!(artifact.get_content_string().unwrap(), content);

        // Check compression ratio
        let ratio = artifact.compression_ratio().unwrap();
        assert!(ratio < 0.5); // Should compress well for repeated content
    }

    #[test]
    fn test_mime_type_detection() {
        let session_id = SessionId::new();

        let test_cases = vec![
            ("test.json", "application/json"),
            ("image.png", "image/png"),
            ("script.js", "application/javascript"),
            ("unknown.xyz", "application/octet-stream"),
        ];

        for (name, expected_mime) in test_cases {
            let mut artifact = SessionArtifact::new(
                session_id,
                1,
                ArtifactType::UserInput,
                name.to_string(),
                vec![],
            )
            .unwrap();

            artifact.detect_mime_type();
            assert_eq!(artifact.metadata.mime_type, expected_mime);
        }
    }

    #[test]
    fn test_storage_serialize() {
        let session_id = SessionId::new();
        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::ToolResult,
            "result.json".to_string(),
            br#"{"status": "success"}"#.to_vec(),
        )
        .unwrap();

        // Serialize
        let bytes = artifact.to_storage_with_checksum().unwrap();

        // Deserialize
        let restored = SessionArtifact::from_storage_with_checksum(&bytes).unwrap();

        // Verify
        assert_eq!(restored.id, artifact.id);
        assert_eq!(restored.metadata.name, artifact.metadata.name);
        assert_eq!(
            restored.get_content().unwrap(),
            artifact.get_content().unwrap()
        );
    }

    #[test]
    fn test_integrity_verification() {
        let session_id = SessionId::new();
        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::AgentOutput,
            "output.txt".to_string(),
            b"Agent output".to_vec(),
        )
        .unwrap();

        assert!(artifact.verify_integrity());
    }

    #[test]
    fn test_derived_artifact() {
        let session_id = SessionId::new();
        let original = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "input.txt".to_string(),
            b"Original content".to_vec(),
        )
        .unwrap();

        let derived = original
            .derive(
                "processed.txt".to_string(),
                b"Processed content".to_vec(),
                ArtifactType::ToolResult,
            )
            .unwrap();

        assert_eq!(derived.id.session_id, original.id.session_id);
        assert_eq!(derived.id.sequence, original.id.sequence + 1);
        assert_eq!(derived.metadata.parent_artifact, Some(original.id.clone()));
    }
}
