//! ABOUTME: Artifact storage system that provides content-addressed storage with any backend
//! ABOUTME: Abstracts over `StorageBackend` to support local, S3, cloud storage, etc.

use super::access::{AccessControlConfig, AccessControlManager};
use super::session_artifact::SessionArtifact;
use super::types::{ArtifactId, ArtifactMetadata, ArtifactType, ContentHash};
use super::versioning::VersionManager;
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
    /// Access control configuration
    pub access_control: AccessControlConfig,
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
            access_control: AccessControlConfig::default(),
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
    /// Version manager for tracking artifact versions
    version_manager: Arc<VersionManager>,
    /// Access control manager
    access_control_manager: Arc<AccessControlManager>,
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

        let version_manager = Arc::new(VersionManager::new(
            storage_backend.clone(),
            config.key_prefix.clone(),
        ));

        let access_control_manager =
            Arc::new(AccessControlManager::new(config.access_control.clone()));

        Self {
            storage_backend,
            config,
            metadata_cache: Arc::new(RwLock::new(lru::LruCache::new(cache_size))),
            session_stats: Arc::new(RwLock::new(HashMap::new())),
            dedup_index: Arc::new(RwLock::new(HashMap::new())),
            version_manager,
            access_control_manager,
        }
    }

    /// Create with default configuration
    pub fn with_backend(storage_backend: Arc<dyn StorageBackend>) -> Self {
        Self::new(storage_backend, ArtifactStorageConfig::default())
    }

    /// Get access to the access control manager
    pub fn access_control_manager(&self) -> &Arc<AccessControlManager> {
        &self.access_control_manager
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

    /// Store content in chunks for large artifacts
    #[allow(dead_code)]
    async fn store_chunked_content(
        &self,
        content_hash: &ContentHash,
        content: &[u8],
    ) -> Result<()> {
        let chunk_size = self.config.chunk_size;
        let total_chunks = content.len().div_ceil(chunk_size);

        // Store each chunk
        for (i, chunk) in content.chunks(chunk_size).enumerate() {
            #[allow(clippy::cast_possible_truncation)]
            let chunk_key = self.chunk_key(content_hash, i as u32);
            self.storage_backend
                .set(&chunk_key, chunk.to_vec())
                .await
                .map_err(|e| SessionError::Storage(format!("Failed to store chunk {i}: {e}")))?;
        }

        // Store chunk metadata
        let metadata_key = format!(
            "{}/chunks/{}/metadata",
            self.config.key_prefix, content_hash
        );
        let chunk_metadata = serde_json::json!({
            "total_chunks": total_chunks,
            "chunk_size": chunk_size,
            "total_size": content.len(),
        });

        self.storage_backend
            .set(&metadata_key, serde_json::to_vec(&chunk_metadata).unwrap())
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to store chunk metadata: {e}")))?;

        Ok(())
    }

    /// Add artifact to session's artifact list
    #[allow(dead_code)]
    async fn add_to_session_artifacts(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<()> {
        let key = self.session_artifacts_key(session_id);

        // Get existing list
        let mut artifact_ids: Vec<ArtifactId> = match self.storage_backend.get(&key).await {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                SessionError::Deserialization(format!("Failed to deserialize artifact list: {e}"))
            })?,
            Ok(None) => Vec::new(),
            Err(e) => {
                return Err(SessionError::Storage(format!(
                    "Failed to get artifact list: {e}"
                )))
            }
        };

        // Add new artifact ID
        artifact_ids.push(artifact_id.clone());

        // Store updated list
        let data = bincode::serialize(&artifact_ids).map_err(|e| {
            SessionError::Serialization(format!("Failed to serialize artifact list: {e}"))
        })?;

        self.storage_backend
            .set(&key, data)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to store artifact list: {e}")))?;

        Ok(())
    }

    /// Load content from storage (handling chunked content)
    #[allow(dead_code, clippy::cast_possible_truncation)]
    async fn load_content(&self, content_hash: &ContentHash) -> Result<Vec<u8>> {
        // Check if content is chunked
        let metadata_key = format!(
            "{}/chunks/{}/metadata",
            self.config.key_prefix, content_hash
        );

        if let Some(chunk_metadata_data) =
            self.storage_backend.get(&metadata_key).await.map_err(|e| {
                SessionError::Storage(format!("Failed to check chunk metadata: {e}"))
            })?
        {
            // Load chunked content
            let chunk_metadata: serde_json::Value = serde_json::from_slice(&chunk_metadata_data)
                .map_err(|e| {
                    SessionError::Deserialization(format!("Failed to parse chunk metadata: {e}"))
                })?;

            let total_chunks = chunk_metadata["total_chunks"]
                .as_u64()
                .ok_or_else(|| SessionError::Deserialization("Missing total_chunks".to_string()))?
                as u32;

            let mut content = Vec::new();
            for i in 0..total_chunks {
                let chunk_key = self.chunk_key(content_hash, i);
                let chunk = self
                    .storage_backend
                    .get(&chunk_key)
                    .await
                    .map_err(|e| SessionError::Storage(format!("Failed to load chunk {i}: {e}")))?
                    .ok_or_else(|| SessionError::Storage(format!("Chunk {i} not found")))?;
                content.extend_from_slice(&chunk);
            }

            Ok(content)
        } else {
            // Load single blob
            let content_key = self.content_key(content_hash);
            self.storage_backend
                .get(&content_key)
                .await
                .map_err(|e| SessionError::Storage(format!("Failed to load content: {e}")))?
                .ok_or_else(|| SessionError::Storage("Content not found".to_string()))
        }
    }

    /// Delete chunked content
    #[allow(dead_code, clippy::cast_possible_truncation)]
    async fn delete_chunked_content(&self, content_hash: &ContentHash) -> Result<()> {
        // Get chunk metadata
        let metadata_key = format!(
            "{}/chunks/{}/metadata",
            self.config.key_prefix, content_hash
        );

        let chunk_metadata_data = self
            .storage_backend
            .get(&metadata_key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to get chunk metadata: {e}")))?
            .ok_or_else(|| SessionError::Storage("Chunk metadata not found".to_string()))?;

        let chunk_metadata: serde_json::Value = serde_json::from_slice(&chunk_metadata_data)
            .map_err(|e| {
                SessionError::Deserialization(format!("Failed to parse chunk metadata: {e}"))
            })?;

        let total_chunks = chunk_metadata["total_chunks"]
            .as_u64()
            .ok_or_else(|| SessionError::Deserialization("Missing total_chunks".to_string()))?
            as u32;

        // Delete all chunks
        for i in 0..total_chunks {
            let chunk_key = self.chunk_key(content_hash, i);
            self.storage_backend
                .delete(&chunk_key)
                .await
                .map_err(|e| SessionError::Storage(format!("Failed to delete chunk {i}: {e}")))?;
        }

        // Delete chunk metadata
        self.storage_backend
            .delete(&metadata_key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to delete chunk metadata: {e}")))?;

        Ok(())
    }

    /// Remove artifact from session's artifact list
    #[allow(dead_code)]
    async fn remove_from_session_artifacts(
        &self,
        session_id: &SessionId,
        artifact_id: &ArtifactId,
    ) -> Result<()> {
        let key = self.session_artifacts_key(session_id);

        // Get existing list
        let mut artifact_ids: Vec<ArtifactId> = match self.storage_backend.get(&key).await {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                SessionError::Deserialization(format!("Failed to deserialize artifact list: {e}"))
            })?,
            Ok(None) => return Ok(()), // No list, nothing to remove
            Err(e) => {
                return Err(SessionError::Storage(format!(
                    "Failed to get artifact list: {e}"
                )))
            }
        };

        // Remove the artifact ID
        artifact_ids.retain(|id| id != artifact_id);

        // Store updated list
        let data = bincode::serialize(&artifact_ids).map_err(|e| {
            SessionError::Serialization(format!("Failed to serialize artifact list: {e}"))
        })?;

        self.storage_backend
            .set(&key, data)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to update artifact list: {e}")))?;

        Ok(())
    }

    /// Get the latest version of an artifact by name
    ///
    /// # Errors
    ///
    /// Returns an error if artifact is not found or retrieval fails
    pub async fn get_latest_version(
        &self,
        session_id: &SessionId,
        name: &str,
    ) -> Result<SessionArtifact> {
        let history = self
            .version_manager
            .get_version_history(session_id, name)
            .await?;

        if history.current_version == 0 {
            return Err(SessionError::ArtifactNotFound {
                id: name.to_string(),
            });
        }

        let latest_id = history
            .versions
            .get(&history.current_version)
            .ok_or_else(|| SessionError::ArtifactNotFound {
                id: format!("{name}:v{}", history.current_version),
            })?;

        match self.get_artifact(latest_id).await? {
            Some(artifact) => Ok(artifact),
            None => Err(SessionError::ArtifactNotFound {
                id: format!("{name}:v{}", history.current_version),
            }),
        }
    }

    /// Get a specific version of an artifact by name and version number
    ///
    /// # Errors
    ///
    /// Returns an error if artifact version is not found or retrieval fails
    pub async fn get_specific_version(
        &self,
        session_id: &SessionId,
        name: &str,
        version: u32,
    ) -> Result<SessionArtifact> {
        let history = self
            .version_manager
            .get_version_history(session_id, name)
            .await?;

        let artifact_id =
            history
                .versions
                .get(&version)
                .ok_or_else(|| SessionError::ArtifactNotFound {
                    id: format!("{name}:v{version}"),
                })?;

        match self.get_artifact(artifact_id).await? {
            Some(artifact) => Ok(artifact),
            None => Err(SessionError::ArtifactNotFound {
                id: format!("{name}:v{version}"),
            }),
        }
    }

    /// Get all versions of an artifact by name
    ///
    /// # Errors
    ///
    /// Returns an error if version history cannot be loaded or artifacts cannot be retrieved
    pub async fn get_all_versions(
        &self,
        session_id: &SessionId,
        name: &str,
    ) -> Result<Vec<SessionArtifact>> {
        let history = self
            .version_manager
            .get_version_history(session_id, name)
            .await?;

        let mut artifacts = Vec::new();

        // Get artifacts in version order
        for version in 1..=history.current_version {
            if let Some(artifact_id) = history.versions.get(&version) {
                match self.get_artifact(artifact_id).await {
                    Ok(Some(artifact)) => artifacts.push(artifact),
                    Ok(None) => {
                        eprintln!("Warning: Artifact version {version} of {name} not found");
                    }
                    Err(e) => {
                        // Log error but continue with other versions
                        eprintln!("Warning: Could not retrieve version {version} of {name}: {e}");
                    }
                }
            }
        }

        Ok(artifacts)
    }

    /// Batch retrieve multiple artifacts by their IDs
    ///
    /// # Errors
    ///
    /// Returns an error if storage operations fail. Individual artifact failures are returned as None in the result vector.
    pub async fn get_artifacts_batch(
        &self,
        artifact_ids: &[ArtifactId],
    ) -> Result<Vec<Option<SessionArtifact>>> {
        let mut results = Vec::with_capacity(artifact_ids.len());

        for artifact_id in artifact_ids {
            match self.get_artifact(artifact_id).await {
                Ok(Some(artifact)) => results.push(Some(artifact)),
                Ok(None) | Err(_) => results.push(None), // Not found or failed to retrieve
            }
        }

        Ok(results)
    }

    /// Get artifacts from a session with pagination
    ///
    /// # Errors
    ///
    /// Returns an error if session artifacts cannot be loaded or deserialized
    pub async fn get_session_artifacts_paginated(
        &self,
        session_id: &SessionId,
        offset: usize,
        limit: usize,
    ) -> Result<Vec<SessionArtifact>> {
        let key = self.session_artifacts_key(session_id);

        let artifact_ids: Vec<ArtifactId> = match self.storage_backend.get(&key).await {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                SessionError::Deserialization(format!("Failed to deserialize artifact list: {e}"))
            })?,
            Ok(None) => return Ok(vec![]), // No artifacts
            Err(e) => {
                return Err(SessionError::Storage(format!(
                    "Failed to get artifact list: {e}"
                )))
            }
        };

        // Apply pagination
        let paginated_ids: Vec<&ArtifactId> =
            artifact_ids.iter().skip(offset).take(limit).collect();

        // Batch retrieve the paginated artifacts
        let batch_results = self
            .get_artifacts_batch(&paginated_ids.into_iter().cloned().collect::<Vec<_>>())
            .await?;

        // Filter out failed retrievals
        Ok(batch_results.into_iter().flatten().collect())
    }

    /// Stream artifact content in chunks (useful for large artifacts)
    ///
    /// # Errors
    ///
    /// Returns an error if artifact is not found or chunks cannot be streamed
    pub async fn stream_artifact_content(
        &self,
        artifact_id: &ArtifactId,
        chunk_size: usize,
    ) -> Result<Vec<Vec<u8>>> {
        // Get the artifact metadata
        let metadata_key = self.metadata_key(artifact_id);
        let metadata_data = self
            .storage_backend
            .get(&metadata_key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to get metadata: {e}")))?;

        let metadata_data = metadata_data.ok_or_else(|| SessionError::ArtifactNotFound {
            id: artifact_id.to_string(),
        })?;

        let metadata_index: MetadataIndex = bincode::deserialize(&metadata_data)
            .map_err(|e| SessionError::Deserialization(format!("Failed to parse metadata: {e}")))?;

        // Get content
        let content = self.load_content(&artifact_id.content_hash).await?;

        // Decompress if needed
        let final_content = if metadata_index.metadata.is_compressed {
            lz4_flex::decompress_size_prepended(&content).map_err(|e| SessionError::General {
                message: format!("Decompression failed: {e}"),
                source: None,
            })?
        } else {
            content
        };

        // Split into chunks
        let mut chunks = Vec::new();
        for chunk in final_content.chunks(chunk_size) {
            chunks.push(chunk.to_vec());
        }

        Ok(chunks)
    }

    /// Get artifact content as a stream of bytes
    ///
    /// # Errors
    ///
    /// Returns an error if artifact cannot be found or content cannot be loaded
    #[allow(clippy::manual_let_else)]
    pub async fn get_artifact_content_stream(&self, artifact_id: &ArtifactId) -> Result<Vec<u8>> {
        // Get the artifact metadata
        let metadata = match self.load_metadata(artifact_id).await? {
            Some(metadata) => metadata,
            None => {
                return Err(SessionError::ArtifactNotFound {
                    id: artifact_id.to_string(),
                })
            }
        };

        // Get content
        let content = self.load_content(&artifact_id.content_hash).await?;

        // Decompress if needed
        let final_content = if metadata.metadata.is_compressed {
            lz4_flex::decompress_size_prepended(&content).map_err(|e| SessionError::General {
                message: format!("Decompression failed: {e}"),
                source: None,
            })?
        } else {
            content
        };

        Ok(final_content)
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

    /// Search for artifacts by content hash
    pub async fn find_by_content_hash(
        &self,
        content_hash: &ContentHash,
    ) -> Result<Vec<ArtifactMetadata>> {
        // Build a metadata index from all sessions that might have this content
        let mut found_metadata = Vec::new();

        // We need to search through all stored metadata entries
        // In a production system, you'd maintain a reverse index from content_hash to artifact_ids
        // For now, we'll scan through the cache to find matches
        let cache = self.metadata_cache.read().await;
        for (_, index_entry) in cache.iter() {
            if index_entry.artifact_id.content_hash == *content_hash {
                found_metadata.push(index_entry.metadata.clone());
            }
        }

        Ok(found_metadata)
    }

    /// Count artifacts by type across all sessions
    pub async fn count_artifacts_by_type(&self) -> Result<HashMap<ArtifactType, usize>> {
        let mut counts = HashMap::new();

        // Aggregate from session stats
        let _stats_map = self.session_stats.read().await;

        // Note: In a production system, you'd maintain separate type counts
        // For now, return empty as we don't track type-specific counts in session stats
        counts.insert(ArtifactType::UserInput, 0);
        counts.insert(ArtifactType::AgentOutput, 0);
        counts.insert(ArtifactType::ToolResult, 0);
        counts.insert(ArtifactType::SystemGenerated, 0);

        Ok(counts)
    }

    /// Get total artifact count across all sessions
    pub async fn get_total_artifact_count(&self) -> Result<usize> {
        let stats_map = self.session_stats.read().await;
        let total = stats_map.values().map(|stats| stats.artifact_count).sum();
        Ok(total)
    }

    /// Get total storage size across all sessions
    pub async fn get_total_storage_size(&self) -> Result<usize> {
        let stats_map = self.session_stats.read().await;
        let total = stats_map.values().map(|stats| stats.total_size).sum();
        Ok(total)
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
    async fn store_artifact(&self, artifact: &SessionArtifact) -> Result<ArtifactId> {
        // Check storage limits first (use original size if compressed)
        let actual_size = artifact
            .metadata
            .original_size
            .unwrap_or(artifact.metadata.size);
        self.check_storage_limits(&artifact.id.session_id, actual_size)
            .await?;

        let content_hash = artifact.id.content_hash.clone();
        let session_id = artifact.id.session_id;

        // Get the next version number for this artifact name
        let version_info = self
            .version_manager
            .next_version(&session_id, &artifact.metadata.name)
            .await?;

        // Create a new artifact with updated version info
        let mut updated_metadata = artifact.metadata.clone();
        updated_metadata.version = version_info;

        // Create new artifact ID with the version sequence
        let versioned_id = ArtifactId::new(
            content_hash.clone(),
            session_id,
            u64::from(updated_metadata.version.version),
        );

        // Initialize access control for the versioned artifact (owner is the session that created it)
        self.access_control_manager
            .initialize_acl(versioned_id.clone(), session_id)
            .await?;

        // Check if content already exists (deduplication)
        let content_exists = self.content_exists(&content_hash).await?;

        // Store content if it doesn't exist
        if !content_exists {
            // Get the content from the artifact
            let content = artifact.get_content()?;

            // Store content based on size
            if content.len() > self.config.chunk_size {
                // Store in chunks for large artifacts
                self.store_chunked_content(&content_hash, &content).await?;
            } else {
                // Store as single blob
                let key = self.content_key(&content_hash);
                self.storage_backend
                    .set(&key, content.clone())
                    .await
                    .map_err(|e| SessionError::Storage(format!("Failed to store content: {e}")))?;
            }
        }

        // Update deduplication index
        self.update_dedup_index(&content_hash, true).await?;

        // Record the version in version history
        self.version_manager
            .record_version(
                &session_id,
                &updated_metadata.name,
                updated_metadata.version.version,
                &versioned_id,
                &content_hash,
            )
            .await?;

        // Create metadata index entry
        let index_entry = MetadataIndex {
            artifact_id: versioned_id.clone(),
            metadata: updated_metadata.clone(),
            storage_key: self.content_key(&content_hash),
            content_size: actual_size,
            is_chunked: actual_size > self.config.chunk_size,
            chunk_count: if actual_size > self.config.chunk_size {
                #[allow(clippy::cast_possible_truncation)]
                let count = actual_size.div_ceil(self.config.chunk_size) as u32;
                count
            } else {
                0
            },
        };

        // Store metadata
        self.store_metadata(&versioned_id, &index_entry).await?;

        // Update session artifacts list
        self.add_to_session_artifacts(&session_id, &versioned_id)
            .await?;

        // Update session statistics (use original size if compressed)
        let actual_size = updated_metadata
            .original_size
            .unwrap_or(updated_metadata.size);
        let deduplicated = content_exists;
        #[allow(clippy::cast_possible_wrap)]
        self.update_session_stats(&session_id, actual_size as i64, 1, deduplicated)
            .await?;

        Ok(versioned_id)
    }

    #[allow(clippy::manual_let_else)]
    async fn get_artifact(&self, artifact_id: &ArtifactId) -> Result<Option<SessionArtifact>> {
        // Load metadata first
        let metadata = match self.load_metadata(artifact_id).await? {
            Some(metadata) => metadata,
            None => return Ok(None),
        };

        // Get the content
        let content = self
            .load_content(&metadata.artifact_id.content_hash)
            .await?;

        // Verify content integrity
        let calculated_hash = blake3::hash(&content).to_string();
        if calculated_hash != metadata.artifact_id.content_hash {
            return Err(SessionError::IntegrityError {
                message: format!(
                    "Content hash mismatch: expected {}, got {}",
                    metadata.artifact_id.content_hash, calculated_hash
                ),
            });
        }

        // Reconstruct the SessionArtifact
        let artifact = SessionArtifact::from_parts(
            metadata.artifact_id.clone(),
            metadata.metadata.clone(),
            content,
            metadata.metadata.created_at,
        )?;

        Ok(Some(artifact))
    }

    #[allow(clippy::manual_let_else)]
    async fn delete_artifact(&self, artifact_id: &ArtifactId) -> Result<bool> {
        // Load metadata to get content hash and session info
        let metadata = match self.load_metadata(artifact_id).await? {
            Some(metadata) => metadata,
            None => return Ok(false), // Not found
        };

        let content_hash = &metadata.artifact_id.content_hash;
        let session_id = &artifact_id.session_id;

        // Delete metadata
        let metadata_key = self.metadata_key(artifact_id);
        self.storage_backend
            .delete(&metadata_key)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to delete metadata: {e}")))?;

        // Update deduplication index
        self.update_dedup_index(content_hash, false).await?;

        // Check if content should be deleted (no more references)
        let dedup_index = self.dedup_index.read().await;
        let should_delete_content = !dedup_index.contains_key(content_hash);
        drop(dedup_index);

        if should_delete_content {
            // Delete content
            if metadata.is_chunked {
                self.delete_chunked_content(content_hash).await?;
            } else {
                let content_key = self.content_key(content_hash);
                self.storage_backend
                    .delete(&content_key)
                    .await
                    .map_err(|e| SessionError::Storage(format!("Failed to delete content: {e}")))?;
            }
        }

        // Remove from session artifacts list
        self.remove_from_session_artifacts(session_id, artifact_id)
            .await?;

        // Update session statistics
        let actual_size = metadata
            .metadata
            .original_size
            .unwrap_or(metadata.metadata.size);
        #[allow(clippy::cast_possible_wrap)]
        self.update_session_stats(session_id, -(actual_size as i64), -1, false)
            .await?;

        Ok(true)
    }

    async fn list_session_artifacts(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<ArtifactMetadata>> {
        // Get the list of artifact IDs for this session
        let key = self.session_artifacts_key(session_id);
        let artifact_ids: Vec<ArtifactId> = match self.storage_backend.get(&key).await {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                SessionError::Deserialization(format!("Failed to deserialize artifact list: {e}"))
            })?,
            Ok(None) => return Ok(Vec::new()),
            Err(e) => {
                return Err(SessionError::Storage(format!(
                    "Failed to get artifact list: {e}"
                )))
            }
        };

        // Load metadata for each artifact
        let mut metadata_list = Vec::with_capacity(artifact_ids.len());
        for artifact_id in artifact_ids {
            if let Some(index) = self.load_metadata(&artifact_id).await? {
                metadata_list.push(index.metadata);
            }
            // Skip artifacts whose metadata might have been deleted
        }

        Ok(metadata_list)
    }

    async fn query_artifacts(&self, query: ArtifactQuery) -> Result<Vec<ArtifactMetadata>> {
        use super::metadata::MetadataIndex;
        use super::search::{ArtifactSearch, ArtifactSearchQuery, SortOrder};

        // Build a comprehensive metadata index from all sessions
        let mut global_index = MetadataIndex::new();

        // If session_id is specified, only search that session
        let sessions_to_search = if let Some(ref session_id) = query.session_id {
            vec![*session_id]
        } else {
            // Otherwise, we need to list all sessions
            // For now, we'll return empty if no session specified
            // In production, you'd iterate through all sessions in storage
            return Ok(Vec::new());
        };

        // Collect metadata from specified sessions
        for session_id in sessions_to_search {
            let key = self.session_artifacts_key(&session_id);
            let artifact_ids: Vec<ArtifactId> = match self.storage_backend.get(&key).await {
                Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                    SessionError::Deserialization(format!(
                        "Failed to deserialize artifact list: {e}"
                    ))
                })?,
                Ok(None) => continue,
                Err(e) => {
                    return Err(SessionError::Storage(format!(
                        "Failed to get artifact list: {e}"
                    )))
                }
            };

            // Add each artifact's metadata to the global index
            for artifact_id in artifact_ids {
                if let Some(index_entry) = self.load_metadata(&artifact_id).await? {
                    global_index.add_metadata(artifact_id, index_entry.metadata);
                }
            }
        }

        // Convert ArtifactQuery to ArtifactSearchQuery
        let search_query = ArtifactSearchQuery {
            session_id: query.session_id,
            artifact_type: query.artifact_type,
            name_pattern: query.name_pattern,
            tags: query.tags,
            created_after: query.created_after,
            created_before: query.created_before,
            min_size: query.min_size,
            max_size: query.max_size,
            sort_order: SortOrder::default(),
            offset: None,
            limit: query.limit,
        };

        // Create search engine and perform search
        let search_engine = ArtifactSearch::new(global_index);
        let search_result = search_engine.search(&search_query);

        Ok(search_result.artifacts)
    }

    async fn get_storage_stats(&self, session_id: &SessionId) -> Result<SessionStorageStats> {
        let stats_map = self.session_stats.read().await;
        Ok(stats_map.get(session_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifact::SessionArtifact;
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
    #[tokio::test]
    async fn test_store_artifact_basic() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        let content = b"test content".to_vec();
        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "test.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        // Store the artifact
        let artifact_id = storage.store_artifact(&artifact).await.unwrap();

        // Verify the artifact ID has version 1
        assert_eq!(artifact_id.sequence, 1);

        // Check that stats were updated
        let stats = storage.get_storage_stats(&session_id).await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.total_size, content.len());
        assert_eq!(stats.deduplicated_count, 0);
    }
    #[tokio::test]
    async fn test_store_artifact_versioning() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store first version
        let content1 = b"version 1 content".to_vec();
        let artifact1 = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "document.txt".to_string(),
            content1.clone(),
        )
        .unwrap();

        let id1 = storage.store_artifact(&artifact1).await.unwrap();
        assert_eq!(id1.sequence, 1);

        // Store second version with same name
        let content2 = b"version 2 content - updated".to_vec();
        let artifact2 = SessionArtifact::new(
            session_id,
            2,
            ArtifactType::UserInput,
            "document.txt".to_string(),
            content2.clone(),
        )
        .unwrap();

        let id2 = storage.store_artifact(&artifact2).await.unwrap();
        assert_eq!(id2.sequence, 2);

        // Different content should have different hashes
        assert_ne!(id1.content_hash, id2.content_hash);

        // Check stats - two artifacts stored
        let stats = storage.get_storage_stats(&session_id).await.unwrap();
        assert_eq!(stats.artifact_count, 2);
        assert_eq!(stats.total_size, content1.len() + content2.len());
    }
    #[tokio::test]
    async fn test_store_artifact_deduplication() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        let content = b"duplicate content".to_vec();

        // Store first artifact
        let artifact1 = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "file1.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        let id1 = storage.store_artifact(&artifact1).await.unwrap();

        // Store second artifact with same content but different name
        let artifact2 = SessionArtifact::new(
            session_id,
            2,
            ArtifactType::UserInput,
            "file2.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        let id2 = storage.store_artifact(&artifact2).await.unwrap();

        // Same content should have same hash
        assert_eq!(id1.content_hash, id2.content_hash);

        // Check that deduplication worked
        let stats = storage.get_storage_stats(&session_id).await.unwrap();
        assert_eq!(stats.artifact_count, 2);
        assert_eq!(stats.deduplicated_count, 1);
        // Total size counts both artifacts even if deduplicated
        assert_eq!(stats.total_size, content.len() * 2);
    }
    #[tokio::test]
    async fn test_store_large_artifact() {
        let backend = Arc::new(MemoryBackend::new());
        let mut config = ArtifactStorageConfig::default();
        config.chunk_size = 1024; // 1KB chunks for testing

        let storage = Arc::new(ArtifactStorage::new(backend, config));

        let session_id = SessionId::new();
        // Create content larger than chunk size
        let content = vec![b'x'; 3000]; // 3KB

        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::SystemGenerated,
            "large_file.bin".to_string(),
            content.clone(),
        )
        .unwrap();

        let artifact_id = storage.store_artifact(&artifact).await.unwrap();

        // Verify storage
        assert_eq!(artifact_id.sequence, 1);

        let stats = storage.get_storage_stats(&session_id).await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        assert_eq!(stats.total_size, content.len());
    }
    #[tokio::test]
    async fn test_store_artifact_with_compression() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        // Create compressible content (repetitive pattern)
        let content = "x".repeat(20 * 1024).into_bytes(); // 20KB of 'x'

        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::AgentOutput,
            "compressed.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        // Store artifact (should auto-compress)
        let artifact_id = storage.store_artifact(&artifact).await.unwrap();

        assert_eq!(artifact_id.sequence, 1);

        let stats = storage.get_storage_stats(&session_id).await.unwrap();
        assert_eq!(stats.artifact_count, 1);
        // Size reported is original size, not compressed
        assert_eq!(stats.total_size, content.len());
    }
    #[tokio::test]
    async fn test_version_retrieval() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        let artifact_name = "test_file.txt";

        // Store version 1
        let content_v1 = b"Version 1 content".to_vec();
        let artifact_v1 = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            artifact_name.to_string(),
            content_v1.clone(),
        )
        .unwrap();
        storage.store_artifact(&artifact_v1).await.unwrap();

        // Store version 2
        let content_v2 = b"Version 2 content".to_vec();
        let artifact_v2 = SessionArtifact::new(
            session_id,
            2,
            ArtifactType::UserInput,
            artifact_name.to_string(),
            content_v2.clone(),
        )
        .unwrap();
        storage.store_artifact(&artifact_v2).await.unwrap();

        // Test get_latest_version
        let latest = storage
            .get_latest_version(&session_id, artifact_name)
            .await
            .unwrap();
        assert_eq!(latest.get_content().unwrap(), content_v2);

        // Test get_specific_version
        let v1 = storage
            .get_specific_version(&session_id, artifact_name, 1)
            .await
            .unwrap();
        assert_eq!(v1.get_content().unwrap(), content_v1);

        // Test get_all_versions
        let all_versions = storage
            .get_all_versions(&session_id, artifact_name)
            .await
            .unwrap();
        assert_eq!(all_versions.len(), 2);
    }
    #[tokio::test]
    async fn test_batch_retrieval() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store multiple artifacts
        let mut artifact_ids = Vec::new();
        for i in 1..=3 {
            let content = format!("Content {}", i).into_bytes();
            let artifact = SessionArtifact::new(
                session_id,
                i,
                ArtifactType::SystemGenerated,
                format!("file_{}.txt", i),
                content,
            )
            .unwrap();
            let id = storage.store_artifact(&artifact).await.unwrap();
            artifact_ids.push(id);
        }

        // Test batch retrieval
        let results = storage.get_artifacts_batch(&artifact_ids).await.unwrap();
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.is_some()));

        // Test batch retrieval with non-existent ID
        let mut mixed_ids = artifact_ids.clone();
        let fake_id = ArtifactId::new("fake_hash".to_string(), session_id, 999);
        mixed_ids.push(fake_id);

        let mixed_results = storage.get_artifacts_batch(&mixed_ids).await.unwrap();
        assert_eq!(mixed_results.len(), 4);
        assert!(mixed_results[0].is_some());
        assert!(mixed_results[1].is_some());
        assert!(mixed_results[2].is_some());
        assert!(mixed_results[3].is_none()); // Fake ID should be None
    }
    #[tokio::test]
    async fn test_paginated_retrieval() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store 5 artifacts
        for i in 1..=5 {
            let content = format!("Content {}", i).into_bytes();
            let artifact = SessionArtifact::new(
                session_id,
                i,
                ArtifactType::ToolResult,
                format!("result_{}.json", i),
                content,
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        // Test pagination - first page
        let page1 = storage
            .get_session_artifacts_paginated(&session_id, 0, 2)
            .await
            .unwrap();
        assert_eq!(page1.len(), 2);

        // Test pagination - second page
        let page2 = storage
            .get_session_artifacts_paginated(&session_id, 2, 2)
            .await
            .unwrap();
        assert_eq!(page2.len(), 2);

        // Test pagination - last page
        let page3 = storage
            .get_session_artifacts_paginated(&session_id, 4, 2)
            .await
            .unwrap();
        assert_eq!(page3.len(), 1);

        // Test pagination - beyond available items
        let empty_page = storage
            .get_session_artifacts_paginated(&session_id, 10, 2)
            .await
            .unwrap();
        assert_eq!(empty_page.len(), 0);
    }
    #[tokio::test]
    async fn test_streaming_retrieval() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Create a large artifact (2KB)
        let content = "x".repeat(2048).into_bytes();
        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::AgentOutput,
            "large_file.txt".to_string(),
            content.clone(),
        )
        .unwrap();

        let artifact_id = storage.store_artifact(&artifact).await.unwrap();

        // Test streaming with 512-byte chunks
        let chunks = storage
            .stream_artifact_content(&artifact_id, 512)
            .await
            .unwrap();
        assert_eq!(chunks.len(), 4); // 2048 / 512 = 4 chunks
        assert_eq!(chunks[0].len(), 512);

        // Reconstruct content from streams
        let reconstructed: Vec<u8> = chunks.into_iter().flatten().collect();
        assert_eq!(reconstructed, content);

        // Test content stream
        let streamed_content = storage
            .get_artifact_content_stream(&artifact_id)
            .await
            .unwrap();
        assert_eq!(streamed_content, content);
    }
    #[tokio::test]
    async fn test_retrieval_error_cases() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        let fake_id = ArtifactId::new("fake_hash".to_string(), session_id, 1);

        // Test get_artifact with non-existent ID
        let result = storage.get_artifact(&fake_id).await.unwrap();
        assert!(result.is_none());

        // Test get_latest_version with non-existent name
        let result = storage
            .get_latest_version(&session_id, "non_existent.txt")
            .await;
        assert!(result.is_err());
        match result.unwrap_err() {
            SessionError::ArtifactNotFound { id, .. } => {
                assert_eq!(id, "non_existent.txt");
            }
            _ => panic!("Expected ArtifactNotFound error"),
        }

        // Test get_specific_version with non-existent version
        let result = storage
            .get_specific_version(&session_id, "test.txt", 99)
            .await;
        assert!(result.is_err());

        // Test streaming with non-existent artifact
        let result = storage.stream_artifact_content(&fake_id, 1024).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_list_session_artifacts() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Initially empty
        let artifacts = storage.list_session_artifacts(&session_id).await.unwrap();
        assert_eq!(artifacts.len(), 0);

        // Store some artifacts
        let artifact1 = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "input.txt".to_string(),
            b"input data".to_vec(),
        )
        .unwrap();
        let _id1 = storage.store_artifact(&artifact1).await.unwrap();

        let artifact2 = SessionArtifact::new(
            session_id,
            2,
            ArtifactType::ToolResult,
            "result.json".to_string(),
            b"result data".to_vec(),
        )
        .unwrap();
        let _id2 = storage.store_artifact(&artifact2).await.unwrap();

        // List should now return 2 artifacts
        let artifacts = storage.list_session_artifacts(&session_id).await.unwrap();
        assert_eq!(artifacts.len(), 2);

        // Verify metadata content
        let names: Vec<String> = artifacts.iter().map(|a| a.name.clone()).collect();
        assert!(names.contains(&"input.txt".to_string()));
        assert!(names.contains(&"result.json".to_string()));
    }
    #[tokio::test]
    async fn test_query_artifacts_by_type() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store artifacts of different types
        let types = vec![
            (ArtifactType::UserInput, "input1.txt"),
            (ArtifactType::UserInput, "input2.txt"),
            (ArtifactType::ToolResult, "result.json"),
            (ArtifactType::AgentOutput, "output.log"),
        ];

        for (i, (artifact_type, name)) in types.iter().enumerate() {
            let artifact = SessionArtifact::new(
                session_id,
                i as u64 + 1,
                artifact_type.clone(),
                name.to_string(),
                format!("content {}", i).into_bytes(),
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        // Query for UserInput artifacts
        let query = ArtifactQuery {
            session_id: Some(session_id),
            artifact_type: Some(ArtifactType::UserInput),
            ..Default::default()
        };

        let results = storage.query_artifacts(query).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|a| a.artifact_type == ArtifactType::UserInput));
    }
    #[tokio::test]
    async fn test_query_artifacts_by_name_pattern() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store artifacts with different names
        let names = vec![
            "test_file.txt",
            "config.json",
            "test_output.log",
            "readme.md",
        ];

        for (i, name) in names.iter().enumerate() {
            let artifact = SessionArtifact::new(
                session_id,
                i as u64 + 1,
                ArtifactType::UserInput,
                name.to_string(),
                format!("content {}", i).into_bytes(),
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        // Query for artifacts with "test" in the name
        let query = ArtifactQuery {
            session_id: Some(session_id),
            name_pattern: Some("test".to_string()),
            ..Default::default()
        };

        let results = storage.query_artifacts(query).await.unwrap();
        assert_eq!(results.len(), 2);

        let result_names: Vec<String> = results.iter().map(|a| a.name.clone()).collect();
        assert!(result_names.contains(&"test_file.txt".to_string()));
        assert!(result_names.contains(&"test_output.log".to_string()));
    }
    #[tokio::test]
    async fn test_query_artifacts_with_tags() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Create artifacts with tags
        let mut artifact1 = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "doc1.txt".to_string(),
            b"content 1".to_vec(),
        )
        .unwrap();
        artifact1.metadata.add_tag("important".to_string()).unwrap();
        artifact1.metadata.add_tag("reviewed".to_string()).unwrap();
        let _id1 = storage.store_artifact(&artifact1).await.unwrap();

        let mut artifact2 = SessionArtifact::new(
            session_id,
            2,
            ArtifactType::UserInput,
            "doc2.txt".to_string(),
            b"content 2".to_vec(),
        )
        .unwrap();
        artifact2.metadata.add_tag("draft".to_string()).unwrap();
        let _id2 = storage.store_artifact(&artifact2).await.unwrap();

        let mut artifact3 = SessionArtifact::new(
            session_id,
            3,
            ArtifactType::UserInput,
            "doc3.txt".to_string(),
            b"content 3".to_vec(),
        )
        .unwrap();
        artifact3.metadata.add_tag("important".to_string()).unwrap();
        artifact3.metadata.add_tag("final".to_string()).unwrap();
        let _id3 = storage.store_artifact(&artifact3).await.unwrap();

        // Query for artifacts with "important" tag
        let query = ArtifactQuery {
            session_id: Some(session_id),
            tags: vec!["important".to_string()],
            ..Default::default()
        };

        let results = storage.query_artifacts(query).await.unwrap();
        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|a| a.tags.contains(&"important".to_string())));
    }
    #[tokio::test]
    async fn test_query_artifacts_with_size_filters() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();

        // Store artifacts of different sizes
        let sizes = vec![
            ("small.txt", 100),
            ("medium.txt", 1000),
            ("large.txt", 5000),
            ("huge.txt", 10000),
        ];

        for (i, (name, size)) in sizes.iter().enumerate() {
            let content = "x".repeat(*size);
            let artifact = SessionArtifact::new(
                session_id,
                i as u64 + 1,
                ArtifactType::UserInput,
                name.to_string(),
                content.into_bytes(),
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        // Query for artifacts between 500 and 6000 bytes
        let query = ArtifactQuery {
            session_id: Some(session_id),
            min_size: Some(500),
            max_size: Some(6000),
            ..Default::default()
        };

        let results = storage.query_artifacts(query).await.unwrap();
        assert_eq!(results.len(), 2);

        let result_names: Vec<String> = results.iter().map(|a| a.name.clone()).collect();
        assert!(result_names.contains(&"medium.txt".to_string()));
        assert!(result_names.contains(&"large.txt".to_string()));
    }
    #[tokio::test]
    async fn test_find_by_content_hash() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id = SessionId::new();
        let content = b"unique content".to_vec();

        // Store an artifact
        let artifact = SessionArtifact::new(
            session_id,
            1,
            ArtifactType::UserInput,
            "file.txt".to_string(),
            content.clone(),
        )
        .unwrap();
        let artifact_id = storage.store_artifact(&artifact).await.unwrap();

        // Find by content hash
        let results = storage
            .find_by_content_hash(&artifact_id.content_hash)
            .await
            .unwrap();

        // Should find at least one artifact with this hash
        assert!(!results.is_empty());

        // Verify it's the correct artifact
        let found = &results[0];
        assert_eq!(found.name, "file.txt");
    }
    #[tokio::test]
    async fn test_count_statistics() {
        let backend = Arc::new(MemoryBackend::new());
        let storage = Arc::new(ArtifactStorage::with_backend(backend));

        let session_id1 = SessionId::new();
        let session_id2 = SessionId::new();

        // Store artifacts in different sessions
        for i in 0..3 {
            let artifact = SessionArtifact::new(
                session_id1,
                i + 1,
                ArtifactType::UserInput,
                format!("file{}.txt", i),
                format!("content {}", i).into_bytes(),
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        for i in 0..2 {
            let artifact = SessionArtifact::new(
                session_id2,
                i + 1,
                ArtifactType::ToolResult,
                format!("result{}.json", i),
                format!("result {}", i).into_bytes(),
            )
            .unwrap();
            storage.store_artifact(&artifact).await.unwrap();
        }

        // Test total count
        let total_count = storage.get_total_artifact_count().await.unwrap();
        assert_eq!(total_count, 5);

        // Test total size
        let total_size = storage.get_total_storage_size().await.unwrap();
        assert!(total_size > 0);
    }
}
