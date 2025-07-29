//! ABOUTME: Artifact storage system that provides content-addressed storage with any backend
//! ABOUTME: Abstracts over `StorageBackend` to support local, S3, cloud storage, etc.

use super::session_artifact::SessionArtifact;
use super::types::{ArtifactId, ArtifactMetadata, ArtifactType, ContentHash};
use crate::{Result, SessionError, SessionId};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_storage::traits::StorageBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for artifact storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactStorageConfig {
    /// Maximum size for a single artifact (bytes)
    pub max_artifact_size: usize,
    /// Maximum total storage per session (bytes)
    pub max_session_storage: usize,
    /// Enable content deduplication
    pub enable_deduplication: bool,
    /// Enable compression for artifacts above this size
    pub compression_threshold: usize,
    /// Cache size for frequently accessed artifacts
    pub cache_size: usize,
    /// Chunk size for streaming large artifacts
    pub chunk_size: usize,
    /// Storage key prefix (useful for multi-tenant scenarios)
    pub key_prefix: String,
}

impl Default for ArtifactStorageConfig {
    fn default() -> Self {
        Self {
            max_artifact_size: 100 * 1024 * 1024,         // 100MB
            max_session_storage: 10 * 1024 * 1024 * 1024, // 10GB
            enable_deduplication: true,
            compression_threshold: 10 * 1024, // 10KB
            cache_size: 100,
            chunk_size: 1024 * 1024, // 1MB chunks
            key_prefix: "artifacts".to_string(),
        }
    }
}

/// Metadata index entry for fast queries
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MetadataIndex {
    /// The artifact ID this index entry refers to
    artifact_id: ArtifactId,
    /// Full artifact metadata
    metadata: ArtifactMetadata,
    /// Storage key for the artifact content
    storage_key: String,
    /// Size of the content in bytes
    content_size: usize,
    /// Whether the content is stored in chunks
    is_chunked: bool,
    /// Number of chunks if chunked
    chunk_count: u32,
}

/// Storage statistics for a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStorageStats {
    /// Total storage size in bytes
    pub total_size: usize,
    /// Number of artifacts stored
    pub artifact_count: usize,
    /// Number of deduplicated artifacts
    pub deduplicated_count: usize,
    /// Last time stats were updated
    pub last_updated: DateTime<Utc>,
}

/// Artifact storage system that works with any `StorageBackend`
#[allow(dead_code)]
pub struct ArtifactStorage {
    /// The underlying storage backend (local, S3, etc.)
    storage_backend: Arc<dyn StorageBackend>,
    /// Configuration
    config: ArtifactStorageConfig,
    /// Metadata cache for fast lookups
    metadata_cache: Arc<RwLock<lru::LruCache<String, MetadataIndex>>>,
    /// Session storage statistics
    session_stats: Arc<RwLock<HashMap<SessionId, SessionStorageStats>>>,
    /// Content deduplication index (`content_hash` -> `reference_count`)
    dedup_index: Arc<RwLock<HashMap<ContentHash, usize>>>,
}

impl ArtifactStorage {
    /// Create a new artifact storage instance with any storage backend
    ///
    /// # Panics
    ///
    /// This function will panic if `config.cache_size` cannot be converted to a `NonZeroUsize`
    pub fn new(storage_backend: Arc<dyn StorageBackend>, config: ArtifactStorageConfig) -> Self {
        let cache_size = std::num::NonZeroUsize::new(config.cache_size)
            .unwrap_or(std::num::NonZeroUsize::new(100).unwrap());

        Self {
            storage_backend,
            config,
            metadata_cache: Arc::new(RwLock::new(lru::LruCache::new(cache_size))),
            session_stats: Arc::new(RwLock::new(HashMap::new())),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn with_backend(storage_backend: Arc<dyn StorageBackend>) -> Self {
        Self::new(storage_backend, ArtifactStorageConfig::default())
    }

    /// Generate a storage key for metadata
    #[allow(dead_code)]
    fn metadata_key(&self, artifact_id: &ArtifactId) -> String {
        format!(
            "{}/metadata/{}",
            self.config.key_prefix,
            artifact_id.storage_key()
        )
    }

    /// Generate a storage key for content
    #[allow(dead_code)]
    fn content_key(&self, content_hash: &ContentHash) -> String {
        format!("{}/content/{}", self.config.key_prefix, content_hash)
    }

    /// Generate a storage key for a content chunk
    #[allow(dead_code)]
    fn chunk_key(&self, content_hash: &ContentHash, chunk_index: u32) -> String {
        format!(
            "{}/chunks/{}/{}",
            self.config.key_prefix, content_hash, chunk_index
        )
    }

    /// Generate a storage key for session stats
    #[allow(dead_code)]
    fn session_stats_key(&self, session_id: &SessionId) -> String {
        format!("{}/sessions/{}/stats", self.config.key_prefix, session_id)
    }

    /// Generate a storage key for session artifact list
    #[allow(dead_code)]
    fn session_artifacts_key(&self, session_id: &SessionId) -> String {
        format!(
            "{}/sessions/{}/artifacts",
            self.config.key_prefix, session_id
        )
    }

    /// Store artifact metadata
    #[allow(dead_code)]
    async fn store_metadata(&self, artifact_id: &ArtifactId, index: &MetadataIndex) -> Result<()> {
        let key = self.metadata_key(artifact_id);
        let data = bincode::serialize(index).map_err(|e| {
            SessionError::Serialization(format!("Failed to serialize metadata: {e}"))
        })?;

        self.storage_backend
            .set(&key, data)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to store metadata: {e}")))?;

        // Update cache
        let mut cache = self.metadata_cache.write().await;
        cache.put(key.clone(), index.clone());

        Ok(())
    }

    /// Load artifact metadata
    #[allow(dead_code)]
    async fn load_metadata(&self, artifact_id: &ArtifactId) -> Result<Option<MetadataIndex>> {
        let key = self.metadata_key(artifact_id);

        // Check cache first
        {
            let mut cache = self.metadata_cache.write().await;
            if let Some(metadata) = cache.get(&key) {
                return Ok(Some(metadata.clone()));
            }
        }

        // Load from storage
        match self
            .storage_backend
            .get(&key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to load metadata: {e}")))?
        {
            Some(data) => {
                let index: MetadataIndex = bincode::deserialize(&data).map_err(|e| {
                    SessionError::Deserialization(format!("Failed to deserialize metadata: {e}"))
                })?;

                // Update cache
                let mut cache = self.metadata_cache.write().await;
                cache.put(key, index.clone());

                Ok(Some(index))
            }
            None => Ok(None),
        }
    }

    /// Check if content already exists (for deduplication)
    #[allow(dead_code)]
    async fn content_exists(&self, content_hash: &ContentHash) -> Result<bool> {
        if !self.config.enable_deduplication {
            return Ok(false);
        }

        let dedup_index = self.dedup_index.read().await;
        if dedup_index.contains_key(content_hash) {
            return Ok(true);
        }

        // Check storage
        let key = self.content_key(content_hash);
        self.storage_backend
            .exists(&key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to check content existence: {e}")))
    }

    /// Update deduplication index
    #[allow(dead_code)]
    async fn update_dedup_index(&self, content_hash: &ContentHash, increment: bool) -> Result<()> {
        if !self.config.enable_deduplication {
            return Ok(());
        }

        let mut dedup_index = self.dedup_index.write().await;

        if increment {
            *dedup_index.entry(content_hash.clone()).or_insert(0) += 1;
        } else if let Some(count) = dedup_index.get_mut(content_hash) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                dedup_index.remove(content_hash);
            }
        }

        Ok(())
    }

    /// Update session storage statistics
    #[allow(dead_code)]
    async fn update_session_stats(
        &self,
        session_id: &SessionId,
        size_delta: i64,
        artifact_delta: i64,
        deduplicated: bool,
    ) -> Result<()> {
        let mut stats_map = self.session_stats.write().await;
        let stats = stats_map.entry(*session_id).or_default();

        match size_delta {
            d if d > 0 => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delta = d as usize;
                stats.total_size = stats.total_size.saturating_add(delta);
            }
            d if d < 0 => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delta = (-d) as usize;
                stats.total_size = stats.total_size.saturating_sub(delta);
            }
            _ => {}
        }

        match artifact_delta {
            d if d > 0 => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delta = d as usize;
                stats.artifact_count = stats.artifact_count.saturating_add(delta);
            }
            d if d < 0 => {
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let delta = (-d) as usize;
                stats.artifact_count = stats.artifact_count.saturating_sub(delta);
            }
            _ => {}
        }

        if deduplicated {
            stats.deduplicated_count += 1;
        }

        stats.last_updated = Utc::now();

        // Persist stats
        let key = self.session_stats_key(session_id);
        let data = bincode::serialize(&stats)
            .map_err(|e| SessionError::Serialization(format!("Failed to serialize stats: {e}")))?;

        self.storage_backend
            .set(&key, data)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to store stats: {e}")))?;

        Ok(())
    }

    /// Check if adding an artifact would exceed storage limits
    #[allow(dead_code)]
    async fn check_storage_limits(
        &self,
        session_id: &SessionId,
        artifact_size: usize,
    ) -> Result<()> {
        // Check artifact size limit
        if artifact_size > self.config.max_artifact_size {
            return Err(SessionError::ResourceLimitExceeded {
                resource: "artifact_size".to_string(),
                message: format!(
                    "Artifact size {} exceeds maximum of {} bytes",
                    artifact_size, self.config.max_artifact_size
                ),
            });
        }

        // Check session storage limit
        let stats_map = self.session_stats.read().await;
        if let Some(stats) = stats_map.get(session_id) {
            let new_total = stats.total_size + artifact_size;
            if new_total > self.config.max_session_storage {
                return Err(SessionError::ResourceLimitExceeded {
                    resource: "session_storage".to_string(),
                    message: format!(
                        "Would exceed session storage limit of {} bytes",
                        self.config.max_session_storage
                    ),
                });
            }
        }

        Ok(())
    }
}

/// Trait for artifact storage operations
#[async_trait]
pub trait ArtifactStorageOps: Send + Sync {
    /// Store an artifact
    async fn store_artifact(&self, artifact: &SessionArtifact) -> Result<ArtifactId>;

    /// Retrieve an artifact
    async fn get_artifact(&self, artifact_id: &ArtifactId) -> Result<Option<SessionArtifact>>;

    /// Delete an artifact
    async fn delete_artifact(&self, artifact_id: &ArtifactId) -> Result<bool>;

    /// List artifacts for a session
    async fn list_session_artifacts(&self, session_id: &SessionId)
        -> Result<Vec<ArtifactMetadata>>;

    /// Query artifacts by criteria
    async fn query_artifacts(&self, query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>>;

    /// Get storage statistics
    async fn get_storage_stats(&self, session_id: &SessionId) -> Result<SessionStorageStats>;
}

/// Query criteria for artifacts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactQuery {
    /// Filter by specific session ID
    pub session_id: Option<SessionId>,
    /// Filter by artifact type
    pub artifact_type: Option<ArtifactType>,
    /// Filter by name pattern (substring match)
    pub name_pattern: Option<String>,
    /// Filter by tags (artifacts must have all specified tags)
    pub tags: Vec<String>,
    /// Filter by creation date (after)
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by creation date (before)
    pub created_before: Option<DateTime<Utc>>,
    /// Minimum artifact size in bytes
    pub min_size: Option<usize>,
    /// Maximum artifact size in bytes
    pub max_size: Option<usize>,
    /// Maximum number of results to return
    pub limit: Option<usize>,
}

// We'll implement the actual storage operations in the next task (6.2.4)
#[async_trait]
impl ArtifactStorageOps for ArtifactStorage {
    async fn store_artifact(&self, _artifact: &SessionArtifact) -> Result<ArtifactId> {
        // Will be implemented in TASK-6.2.4
        todo!("Implement in TASK-6.2.4")
    }

    async fn get_artifact(&self, _artifact_id: &ArtifactId) -> Result<Option<SessionArtifact>> {
        // Will be implemented in TASK-6.2.5
        todo!("Implement in TASK-6.2.5")
    }

    async fn delete_artifact(&self, _artifact_id: &ArtifactId) -> Result<bool> {
        // Will be implemented in TASK-6.2.5
        todo!("Implement in TASK-6.2.5")
    }

    async fn list_session_artifacts(
        &self,
        _session_id: &SessionId,
    ) -> Result<Vec<ArtifactMetadata>> {
        // Will be implemented in TASK-6.2.6
        todo!("Implement in TASK-6.2.6")
    }

    async fn query_artifacts(&self, _query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>> {
        // Will be implemented in TASK-6.2.6
        todo!("Implement in TASK-6.2.6")
    }

    async fn get_storage_stats(&self, session_id: &SessionId) -> Result<SessionStorageStats> {
        let stats_map = self.session_stats.read().await;
        Ok(stats_map.get(session_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_storage::MemoryBackend;

    #[tokio::test]
    async fn test_artifact_storage_creation() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = ArtifactStorage::with_backend(backend);

        assert_eq!(storage.config.max_artifact_size, 100 * 1024 * 1024);
        assert!(storage.config.enable_deduplication);
    }

    #[tokio::test]
    async fn test_key_generation() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = ArtifactStorage::with_backend(backend);

        let session_id = SessionId::new();
        let artifact_id = ArtifactId::new("hash123".to_string(), session_id, 1);

        let metadata_key = storage.metadata_key(&artifact_id);
        assert!(metadata_key.starts_with("artifacts/metadata/"));

        let content_key = storage.content_key(&"hash123".to_string());
        assert_eq!(content_key, "artifacts/content/hash123");

        let chunk_key = storage.chunk_key(&"hash123".to_string(), 0);
        assert_eq!(chunk_key, "artifacts/chunks/hash123/0");
    }

    #[tokio::test]
    async fn test_storage_limits() {
        let backend = Arc::new(MemoryBackend::new());
        let mut config = ArtifactStorageConfig::default();
        config.max_artifact_size = 1024; // 1KB limit for testing

        let storage = ArtifactStorage::new(backend, config);
        let session_id = SessionId::new();

        // Should fail - exceeds limit
        let result = storage.check_storage_limits(&session_id, 2048).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            SessionError::ResourceLimitExceeded { .. }
        ));

        // Should succeed - within limit
        let result = storage.check_storage_limits(&session_id, 512).await;
        assert!(result.is_ok());
    }
}
