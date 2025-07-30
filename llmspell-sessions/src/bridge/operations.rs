//! ABOUTME: Extended session operations for script bridge
//! ABOUTME: Provides additional session management functionality beyond basic CRUD

use crate::{Result, SessionId, SessionManager};
use std::collections::HashMap;
use std::sync::Arc;

/// Extended operations for session management
pub struct SessionOperations {
    session_manager: Arc<SessionManager>,
}

impl SessionOperations {
    /// Create new session operations handler
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Update session metadata
    pub async fn update_metadata(
        &self,
        session_id: &SessionId,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Get the session
        let session = self.session_manager.get_session(session_id).await?;

        // Update metadata fields
        let mut metadata = session.metadata.read().await.clone();
        for (key, value) in updates {
            match key.as_str() {
                "name" => {
                    if let Some(name) = value.as_str() {
                        metadata.name = Some(name.to_string());
                    }
                }
                "description" => {
                    if let Some(desc) = value.as_str() {
                        metadata.description = Some(desc.to_string());
                    }
                }
                "tags" => {
                    if let Some(tags) = value.as_array() {
                        metadata.tags = tags
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
                _ => {
                    // Add to custom metadata
                    metadata.custom_metadata.insert(key, value);
                }
            }
        }

        // Save the updated session
        *session.metadata.write().await = metadata;
        self.session_manager.save_session(&session).await
    }

    /// Get session tags
    pub async fn get_tags(&self, session_id: &SessionId) -> Result<Vec<String>> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        Ok(metadata.tags)
    }

    /// Set session tags (replaces all tags)
    pub async fn set_tags(&self, session_id: &SessionId, tags: Vec<String>) -> Result<()> {
        let session = self.session_manager.get_session(session_id).await?;
        let mut metadata = session.metadata.read().await.clone();
        metadata.tags = tags;
        *session.metadata.write().await = metadata;
        self.session_manager.save_session(&session).await
    }

    /// Add tags to session (appends to existing)
    pub async fn add_tags(&self, session_id: &SessionId, new_tags: Vec<String>) -> Result<()> {
        let session = self.session_manager.get_session(session_id).await?;
        let mut metadata = session.metadata.read().await.clone();

        // Add new tags, avoiding duplicates
        for tag in new_tags {
            if !metadata.tags.contains(&tag) {
                metadata.tags.push(tag);
            }
        }

        *session.metadata.write().await = metadata;
        self.session_manager.save_session(&session).await
    }

    /// Remove tags from session
    pub async fn remove_tags(
        &self,
        session_id: &SessionId,
        tags_to_remove: &[String],
    ) -> Result<()> {
        let session = self.session_manager.get_session(session_id).await?;
        let mut metadata = session.metadata.read().await.clone();

        // Remove specified tags
        metadata.tags.retain(|tag| !tags_to_remove.contains(tag));

        *session.metadata.write().await = metadata;
        self.session_manager.save_session(&session).await
    }

    /// Check if session has a specific tag
    pub async fn has_tag(&self, session_id: &SessionId, tag: &str) -> Result<bool> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        Ok(metadata.tags.contains(&tag.to_string()))
    }

    /// Get session statistics
    pub async fn get_session_stats(&self, session_id: &SessionId) -> Result<SessionStats> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        let artifacts = self.session_manager.list_artifacts(session_id).await?;

        Ok(SessionStats {
            session_id: *session_id,
            created_at: metadata.created_at,
            updated_at: metadata.updated_at,
            status: format!("{:?}", session.status().await),
            artifact_count: artifacts.len(),
            total_artifact_size: artifacts.iter().map(|a| a.size as u64).sum(),
            tag_count: metadata.tags.len(),
            operation_count: metadata.operation_count,
        })
    }

    /// Export session data
    pub async fn export_session(
        &self,
        session_id: &SessionId,
        include_artifacts: bool,
    ) -> Result<SessionExport> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        let snapshot = session.snapshot().await;

        let artifacts = if include_artifacts {
            Some(self.session_manager.list_artifacts(session_id).await?)
        } else {
            None
        };

        Ok(SessionExport {
            session_id: *session_id,
            metadata,
            snapshot,
            artifacts,
            export_timestamp: chrono::Utc::now(),
        })
    }
}

/// Session statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStats {
    /// Session identifier
    pub session_id: SessionId,
    /// Session creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last update timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Current session status
    pub status: String,
    /// Number of artifacts in session
    pub artifact_count: usize,
    /// Total size of artifacts in bytes
    pub total_artifact_size: u64,
    /// Number of tags
    pub tag_count: usize,
    /// Number of operations performed
    pub operation_count: u64,
}

/// Session export data
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionExport {
    /// Session identifier
    pub session_id: SessionId,
    /// Session metadata
    pub metadata: crate::SessionMetadata,
    /// Session state snapshot
    pub snapshot: crate::session::SessionSnapshot,
    /// Artifacts if included in export
    pub artifacts: Option<Vec<crate::artifact::ArtifactMetadata>>,
    /// Timestamp when export was created
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
}
