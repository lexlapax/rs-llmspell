//! ABOUTME: Version management for artifacts, tracking versions by name within sessions
//! ABOUTME: Provides automatic version numbering and version history tracking

use super::types::{ArtifactId, ArtifactVersion, ContentHash};
use crate::sessions::{Result, SessionError, SessionId};
use chrono::Utc;
use llmspell_storage::traits::StorageBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Version history for a named artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionHistory {
    /// Artifact name
    pub name: String,
    /// Current version number
    pub current_version: u32,
    /// Map of version number to artifact ID
    pub versions: HashMap<u32, ArtifactId>,
    /// Map of version number to content hash
    pub version_hashes: HashMap<u32, ContentHash>,
}

/// Version manager for tracking artifact versions
pub struct VersionManager {
    /// Storage backend for persisting version data
    storage_backend: Arc<dyn StorageBackend>,
    /// Key prefix for version data
    key_prefix: String,
    /// In-memory cache of version histories
    version_cache: Arc<RwLock<HashMap<(SessionId, String), VersionHistory>>>,
}

impl VersionManager {
    /// Create a new version manager
    pub fn new(storage_backend: Arc<dyn StorageBackend>, key_prefix: String) -> Self {
        Self {
            storage_backend,
            key_prefix,
            version_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the next version number for an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if version history cannot be loaded from storage
    pub async fn next_version(
        &self,
        session_id: &SessionId,
        name: &str,
    ) -> Result<ArtifactVersion> {
        let key = (*session_id, name.to_string());

        // Check cache first
        {
            let cache = self.version_cache.read().await;
            if let Some(history) = cache.get(&key) {
                return Ok(ArtifactVersion {
                    version: history.current_version + 1,
                    previous_hash: history
                        .version_hashes
                        .get(&history.current_version)
                        .cloned(),
                    created_at: Utc::now(),
                });
            }
        }

        // Load from storage
        let history = self.load_version_history(session_id, name).await?;

        let version = ArtifactVersion {
            version: history.current_version + 1,
            previous_hash: history
                .version_hashes
                .get(&history.current_version)
                .cloned(),
            created_at: Utc::now(),
        };

        // Update cache
        let mut cache = self.version_cache.write().await;
        cache.insert(key, history);

        Ok(version)
    }

    /// Record a new version
    ///
    /// # Errors
    ///
    /// Returns an error if version history cannot be saved to storage
    pub async fn record_version(
        &self,
        session_id: &SessionId,
        name: &str,
        version: u32,
        artifact_id: &ArtifactId,
        content_hash: &ContentHash,
    ) -> Result<()> {
        let key = (*session_id, name.to_string());

        // Get or create history
        let mut history = self.load_version_history(session_id, name).await?;

        // Update history
        history.current_version = version;
        history.versions.insert(version, artifact_id.clone());
        history.version_hashes.insert(version, content_hash.clone());

        // Save to storage
        self.save_version_history(session_id, name, &history)
            .await?;

        // Update cache
        let mut cache = self.version_cache.write().await;
        cache.insert(key, history);

        Ok(())
    }

    /// Get version history for an artifact
    ///
    /// # Errors
    ///
    /// Returns an error if version history cannot be loaded from storage
    pub async fn get_version_history(
        &self,
        session_id: &SessionId,
        name: &str,
    ) -> Result<VersionHistory> {
        let key = (*session_id, name.to_string());

        // Check cache
        {
            let cache = self.version_cache.read().await;
            if let Some(history) = cache.get(&key) {
                return Ok(history.clone());
            }
        }

        // Load from storage
        self.load_version_history(session_id, name).await
    }

    /// Load version history from storage
    async fn load_version_history(
        &self,
        session_id: &SessionId,
        name: &str,
    ) -> Result<VersionHistory> {
        let storage_key = self.version_history_key(session_id, name);

        match self.storage_backend.get(&storage_key).await {
            Ok(Some(data)) => bincode::deserialize(&data).map_err(|e| {
                SessionError::Deserialization(format!("Failed to deserialize version history: {e}"))
            }),
            Ok(None) => {
                // Create new history
                Ok(VersionHistory {
                    name: name.to_string(),
                    current_version: 0,
                    versions: HashMap::new(),
                    version_hashes: HashMap::new(),
                })
            }
            Err(e) => Err(SessionError::Storage(format!(
                "Failed to load version history: {e}"
            ))),
        }
    }

    /// Save version history to storage
    async fn save_version_history(
        &self,
        session_id: &SessionId,
        name: &str,
        history: &VersionHistory,
    ) -> Result<()> {
        let storage_key = self.version_history_key(session_id, name);

        let data = bincode::serialize(history).map_err(|e| {
            SessionError::Serialization(format!("Failed to serialize version history: {e}"))
        })?;

        self.storage_backend
            .set(&storage_key, data)
            .await
            .map_err(|e| SessionError::Storage(format!("Failed to save version history: {e}")))?;

        Ok(())
    }

    /// Generate storage key for version history
    fn version_history_key(&self, session_id: &SessionId, name: &str) -> String {
        format!("{}/versions/{}/{}", self.key_prefix, session_id, name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_storage::MemoryBackend;
    #[tokio::test]
    async fn test_version_numbering() {
        let backend = Arc::new(MemoryBackend::new());
        let manager = VersionManager::new(backend, "test".to_string());

        let session_id = SessionId::new();
        let name = "test-artifact";

        // First version should be 1
        let v1 = manager.next_version(&session_id, name).await.unwrap();
        assert_eq!(v1.version, 1);
        assert!(v1.previous_hash.is_none());

        // Record the version
        let artifact_id = ArtifactId::new("hash1".to_string(), session_id, 1);
        manager
            .record_version(&session_id, name, 1, &artifact_id, &"hash1".to_string())
            .await
            .unwrap();

        // Next version should be 2
        let v2 = manager.next_version(&session_id, name).await.unwrap();
        assert_eq!(v2.version, 2);
        assert_eq!(v2.previous_hash, Some("hash1".to_string()));
    }
    #[tokio::test]
    async fn test_version_history() {
        let backend = Arc::new(MemoryBackend::new());
        let manager = VersionManager::new(backend, "test".to_string());

        let session_id = SessionId::new();
        let name = "test-artifact";

        // Add multiple versions
        for i in 1..=3 {
            let artifact_id = ArtifactId::new(format!("hash{}", i), session_id, i);
            manager
                .record_version(
                    &session_id,
                    name,
                    u32::try_from(i).expect("version number should fit in u32"),
                    &artifact_id,
                    &format!("hash{}", i),
                )
                .await
                .unwrap();
        }

        // Get history
        let history = manager
            .get_version_history(&session_id, name)
            .await
            .unwrap();
        assert_eq!(history.current_version, 3);
        assert_eq!(history.versions.len(), 3);
        assert_eq!(history.version_hashes.len(), 3);
    }
}
