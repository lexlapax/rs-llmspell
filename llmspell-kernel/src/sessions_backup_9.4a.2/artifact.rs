//! Artifact Storage System
//!
//! Version-controlled artifact storage with metadata and deduplication.
//! Supports various artifact types including code, data, and debug snapshots.

use super::SessionId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use tracing::{debug, info, instrument};

/// Artifact identifier
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ArtifactId(String);

impl ArtifactId {
    /// Create new artifact ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    /// Create from string
    pub fn from_string(id: String) -> Self {
        Self(id)
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for ArtifactId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Artifact type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactType {
    /// Code artifact (scripts, functions)
    Code,
    /// Data artifact (datasets, results)
    Data,
    /// Debug snapshot
    DebugSnapshot,
    /// Execution output
    Output,
    /// Error or exception
    Error,
    /// Log data
    Log,
    /// Custom type
    Custom(String),
}

/// Session artifact with versioning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    /// Unique artifact ID
    pub id: ArtifactId,
    /// Session this artifact belongs to
    pub session_id: SessionId,
    /// Artifact type
    pub artifact_type: ArtifactType,
    /// Artifact name
    pub name: String,
    /// Artifact content (could be large)
    pub content: Vec<u8>,
    /// Content hash for deduplication
    pub content_hash: String,
    /// Version number
    pub version: u32,
    /// Parent version (for version history)
    pub parent_version: Option<u32>,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Size in bytes
    pub size: usize,
    /// Tags for searching
    pub tags: Vec<String>,
}

impl SessionArtifact {
    /// Create a new artifact
    pub fn new(
        session_id: SessionId,
        artifact_type: ArtifactType,
        name: String,
        content: Vec<u8>,
    ) -> Self {
        let size = content.len();
        let content_hash = Self::compute_hash(&content);

        Self {
            id: ArtifactId::new(),
            session_id,
            artifact_type,
            name,
            content,
            content_hash,
            version: 1,
            parent_version: None,
            created_at: SystemTime::now(),
            metadata: HashMap::new(),
            size,
            tags: Vec::new(),
        }
    }

    /// Create a new version of an artifact
    #[must_use]
    pub fn new_version(&self, content: Vec<u8>) -> Self {
        let size = content.len();
        let content_hash = Self::compute_hash(&content);

        Self {
            id: ArtifactId::new(),
            session_id: self.session_id.clone(),
            artifact_type: self.artifact_type.clone(),
            name: self.name.clone(),
            content,
            content_hash,
            version: self.version + 1,
            parent_version: Some(self.version),
            created_at: SystemTime::now(),
            metadata: self.metadata.clone(),
            size,
            tags: self.tags.clone(),
        }
    }

    /// Compute content hash
    fn compute_hash(content: &[u8]) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }

    /// Add tag
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }
}

/// Artifact storage backend
pub struct ArtifactStorage {
    /// Storage path
    storage_path: PathBuf,
    /// Artifact index (ID -> metadata)
    index: Arc<parking_lot::RwLock<HashMap<ArtifactId, ArtifactMetadata>>>,
    /// Content deduplication map (hash -> artifact IDs)
    dedup_map: Arc<parking_lot::RwLock<HashMap<String, Vec<ArtifactId>>>>,
    /// Version history (artifact name -> versions)
    version_history: Arc<parking_lot::RwLock<HashMap<String, Vec<ArtifactId>>>>,
}

/// Artifact metadata (stored in index, content stored separately)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    /// Unique artifact identifier
    pub id: ArtifactId,
    /// Session this artifact belongs to
    pub session_id: SessionId,
    /// Type of artifact
    pub artifact_type: ArtifactType,
    /// Artifact name
    pub name: String,
    /// Content hash for deduplication
    pub content_hash: String,
    /// Version number
    pub version: u32,
    /// Parent version for history
    pub parent_version: Option<u32>,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Size in bytes
    pub size: usize,
    /// Search tags
    pub tags: Vec<String>,
}

impl From<&SessionArtifact> for ArtifactMetadata {
    fn from(artifact: &SessionArtifact) -> Self {
        Self {
            id: artifact.id.clone(),
            session_id: artifact.session_id.clone(),
            artifact_type: artifact.artifact_type.clone(),
            name: artifact.name.clone(),
            content_hash: artifact.content_hash.clone(),
            version: artifact.version,
            parent_version: artifact.parent_version,
            created_at: artifact.created_at,
            metadata: artifact.metadata.clone(),
            size: artifact.size,
            tags: artifact.tags.clone(),
        }
    }
}

impl ArtifactStorage {
    /// Create new artifact storage
    ///
    /// # Errors
    ///
    /// Returns an error if the storage directory cannot be created
    pub fn new(storage_path: PathBuf) -> Result<Self> {
        // Create storage directory if it doesn't exist
        std::fs::create_dir_all(&storage_path)?;

        Ok(Self {
            storage_path,
            index: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            dedup_map: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            version_history: Arc::new(parking_lot::RwLock::new(HashMap::new())),
        })
    }

    /// Store an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact cannot be stored to disk
    #[instrument(level = "debug", skip(self, artifact))]
    pub fn store(&self, artifact: &SessionArtifact) -> Result<()> {
        // Store metadata in index
        let metadata = ArtifactMetadata::from(artifact);
        self.index.write().insert(artifact.id.clone(), metadata);

        // Update deduplication map
        self.dedup_map
            .write()
            .entry(artifact.content_hash.clone())
            .or_default()
            .push(artifact.id.clone());

        // Update version history
        self.version_history
            .write()
            .entry(artifact.name.clone())
            .or_default()
            .push(artifact.id.clone());

        // Store content to file (deduplicated by hash)
        let content_path = self.content_path(&artifact.content_hash);
        if content_path.exists() {
            debug!(
                "Content already exists (deduped): {}",
                artifact.content_hash
            );
        } else {
            // Ensure content directory exists
            if let Some(parent) = content_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&content_path, &artifact.content)?;
            debug!("Stored artifact content: {}", artifact.content_hash);
        }

        info!("Stored artifact: {} (v{})", artifact.name, artifact.version);
        Ok(())
    }

    /// Retrieve an artifact by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact content cannot be read from disk
    pub fn get(&self, id: &ArtifactId) -> Result<Option<SessionArtifact>> {
        let index = self.index.read();
        if let Some(metadata) = index.get(id) {
            // Load content from file
            let content_path = self.content_path(&metadata.content_hash);
            if content_path.exists() {
                let content = std::fs::read(&content_path)?;

                let artifact = SessionArtifact {
                    id: metadata.id.clone(),
                    session_id: metadata.session_id.clone(),
                    artifact_type: metadata.artifact_type.clone(),
                    name: metadata.name.clone(),
                    content,
                    content_hash: metadata.content_hash.clone(),
                    version: metadata.version,
                    parent_version: metadata.parent_version,
                    created_at: metadata.created_at,
                    metadata: metadata.metadata.clone(),
                    size: metadata.size,
                    tags: metadata.tags.clone(),
                };

                return Ok(Some(artifact));
            }
        }

        Ok(None)
    }

    /// List artifacts for a session
    pub fn list_by_session(&self, session_id: &SessionId) -> Vec<ArtifactMetadata> {
        self.index
            .read()
            .values()
            .filter(|m| &m.session_id == session_id)
            .cloned()
            .collect()
    }

    /// Get artifact versions
    pub fn get_versions(&self, name: &str) -> Vec<ArtifactId> {
        self.version_history
            .read()
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    /// Search artifacts by tags
    pub fn search_by_tags(&self, tags: &[String]) -> Vec<ArtifactMetadata> {
        self.index
            .read()
            .values()
            .filter(|m| tags.iter().any(|tag| m.tags.contains(tag)))
            .cloned()
            .collect()
    }

    /// Get content path
    fn content_path(&self, hash: &str) -> PathBuf {
        self.storage_path.join("content").join(hash)
    }

    /// Clean up orphaned content (not referenced by any artifact)
    ///
    /// # Errors
    ///
    /// Returns an error if content files cannot be deleted
    pub fn cleanup_orphaned(&self) -> Result<usize> {
        let mut cleaned = 0;

        // Get all referenced hashes
        let referenced_hashes: std::collections::HashSet<String> = self
            .index
            .read()
            .values()
            .map(|m| m.content_hash.clone())
            .collect();

        // Check content directory
        let content_dir = self.storage_path.join("content");
        if content_dir.exists() {
            for entry in std::fs::read_dir(&content_dir)? {
                let entry = entry?;
                if let Some(filename) = entry.file_name().to_str() {
                    if !referenced_hashes.contains(filename) {
                        std::fs::remove_file(entry.path())?;
                        cleaned += 1;
                    }
                }
            }
        }

        info!("Cleaned up {} orphaned content files", cleaned);
        Ok(cleaned)
    }
}

/// Artifact storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStorageConfig {
    /// Maximum artifact size
    pub max_size: usize,
    /// Enable deduplication
    pub enable_dedup: bool,
    /// Maximum versions to keep
    pub max_versions: usize,
}

impl Default for ArtifactStorageConfig {
    fn default() -> Self {
        Self {
            max_size: 100 * 1024 * 1024, // 100MB
            enable_dedup: true,
            max_versions: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_artifact_creation() {
        let artifact = SessionArtifact::new(
            SessionId::new(),
            ArtifactType::Code,
            "test.py".to_string(),
            b"print('hello')".to_vec(),
        );

        assert_eq!(artifact.version, 1);
        assert_eq!(artifact.name, "test.py");
        assert_eq!(artifact.artifact_type, ArtifactType::Code);
    }

    #[test]
    fn test_artifact_versioning() {
        let artifact1 = SessionArtifact::new(
            SessionId::new(),
            ArtifactType::Code,
            "test.py".to_string(),
            b"print('hello')".to_vec(),
        );

        let artifact2 = artifact1.new_version(b"print('world')".to_vec());

        assert_eq!(artifact2.version, 2);
        assert_eq!(artifact2.parent_version, Some(1));
        assert_eq!(artifact2.name, artifact1.name);
    }

    #[test]
    fn test_artifact_storage() {
        let dir = tempdir().unwrap();
        let storage = ArtifactStorage::new(dir.path().to_path_buf()).unwrap();

        let artifact = SessionArtifact::new(
            SessionId::new(),
            ArtifactType::Data,
            "data.json".to_string(),
            b"{\"key\": \"value\"}".to_vec(),
        );

        // Store artifact
        storage.store(&artifact).unwrap();

        // Retrieve artifact
        let retrieved = storage.get(&artifact.id).unwrap().unwrap();
        assert_eq!(retrieved.content, artifact.content);
        assert_eq!(retrieved.version, artifact.version);
    }

    #[test]
    fn test_content_deduplication() {
        let dir = tempdir().unwrap();
        let storage = ArtifactStorage::new(dir.path().to_path_buf()).unwrap();

        let session_id = SessionId::new();
        let content = b"duplicate content".to_vec();

        // Store two artifacts with same content
        let artifact1 = SessionArtifact::new(
            session_id.clone(),
            ArtifactType::Data,
            "file1.txt".to_string(),
            content.clone(),
        );

        let artifact2 = SessionArtifact::new(
            session_id,
            ArtifactType::Data,
            "file2.txt".to_string(),
            content,
        );

        storage.store(&artifact1).unwrap();
        storage.store(&artifact2).unwrap();

        // Both artifacts should reference the same content hash
        assert_eq!(artifact1.content_hash, artifact2.content_hash);

        // Content file should exist only once
        let content_path = storage.content_path(&artifact1.content_hash);
        assert!(content_path.exists());
    }
}
