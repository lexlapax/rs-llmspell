//! ABOUTME: Extended session operations for script bridge
//! ABOUTME: Provides additional session management functionality beyond basic CRUD

use crate::sessions::{Result, SessionId, SessionManager};
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
    ///
    /// # Errors
    /// Returns error if session is not found or metadata update fails
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
    ///
    /// # Errors
    /// Returns error if session is not found
    pub async fn get_tags(&self, session_id: &SessionId) -> Result<Vec<String>> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        Ok(metadata.tags)
    }

    /// Set session tags (replaces all tags)
    ///
    /// # Errors
    /// Returns error if session is not found or session save fails
    pub async fn set_tags(&self, session_id: &SessionId, tags: Vec<String>) -> Result<()> {
        let session = self.session_manager.get_session(session_id).await?;
        let mut metadata = session.metadata.read().await.clone();
        metadata.tags = tags;
        *session.metadata.write().await = metadata;
        self.session_manager.save_session(&session).await
    }

    /// Add tags to session (appends to existing)
    ///
    /// # Errors
    /// Returns error if session is not found or session save fails
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
    ///
    /// # Errors
    /// Returns error if session is not found or session save fails
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
    ///
    /// # Errors
    /// Returns error if session is not found
    pub async fn has_tag(&self, session_id: &SessionId, tag: &str) -> Result<bool> {
        let session = self.session_manager.get_session(session_id).await?;
        let metadata = session.metadata.read().await.clone();
        Ok(metadata.tags.contains(&tag.to_string()))
    }

    /// Get session statistics
    ///
    /// # Errors
    /// Returns error if session is not found or artifacts cannot be retrieved
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
    ///
    /// # Errors
    /// Returns error if session is not found, artifacts cannot be retrieved, or serialization fails
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
    pub snapshot: crate::sessions::session::SessionSnapshot,
    /// Artifacts if included in export
    pub artifacts: Option<Vec<crate::sessions::artifact::ArtifactMetadata>>,
    /// Timestamp when export was created
    pub export_timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::{types::CreateSessionOptions, SessionManagerConfigBuilder};
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use crate::state::StateManager;
    use llmspell_storage::MemoryBackend;
    use std::sync::Arc;

    async fn setup_test_env() -> (Arc<SessionManager>, SessionId) {
        // Create dependencies
        let storage = Arc::new(MemoryBackend::new());
        let state_manager = Arc::new(
            StateManager::new()
                .await
                .expect("Failed to create state manager"),
        );
        let event_bus = Arc::new(EventBus::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        // Create session manager config
        let config = SessionManagerConfigBuilder::new()
            .storage_path("/tmp/test-sessions")
            .auto_persist(false)
            .build();

        let session_manager = Arc::new(
            SessionManager::new(
                state_manager,
                storage,
                hook_registry,
                hook_executor,
                &event_bus,
                config,
            )
            .expect("Failed to create session manager"),
        );

        // Create a test session
        let session_options = CreateSessionOptions {
            name: Some("test-session".to_string()),
            description: Some("Test session for unit tests".to_string()),
            tags: vec!["test".to_string(), "unit".to_string()],
            created_by: Option::default(),
            parent_session_id: Option::default(),
            config: None,
            metadata: HashMap::default(),
        };

        let session_id = session_manager
            .create_session(session_options)
            .await
            .expect("Failed to create session");

        (session_manager, session_id)
    }
    #[tokio::test]
    async fn test_update_metadata() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager.clone());

        // Test updating various metadata fields
        let mut updates = HashMap::new();
        updates.insert("name".to_string(), serde_json::json!("Updated Name"));
        updates.insert(
            "description".to_string(),
            serde_json::json!("Updated Description"),
        );
        updates.insert(
            "tags".to_string(),
            serde_json::json!(["new-tag1", "new-tag2"]),
        );
        updates.insert(
            "custom_field".to_string(),
            serde_json::json!("custom value"),
        );

        ops.update_metadata(&session_id, updates)
            .await
            .expect("Failed to update metadata");

        // Verify updates
        let session = session_manager
            .get_session(&session_id)
            .await
            .expect("Failed to get session");
        let metadata = session.metadata.read().await;

        assert_eq!(metadata.name, Some("Updated Name".to_string()));
        assert_eq!(
            metadata.description,
            Some("Updated Description".to_string())
        );
        assert_eq!(metadata.tags, vec!["new-tag1", "new-tag2"]);
        assert_eq!(
            metadata.custom_metadata.get("custom_field"),
            Some(&serde_json::json!("custom value"))
        );
    }
    #[tokio::test]
    async fn test_tag_operations() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager);

        // Test get_tags
        let initial_tags = ops.get_tags(&session_id).await.expect("Failed to get tags");
        assert_eq!(initial_tags, vec!["test", "unit"]);

        // Test set_tags (replaces all)
        let new_tags = vec!["tag1".to_string(), "tag2".to_string()];
        ops.set_tags(&session_id, new_tags.clone())
            .await
            .expect("Failed to set tags");

        let tags = ops.get_tags(&session_id).await.expect("Failed to get tags");
        assert_eq!(tags, new_tags);

        // Test add_tags (appends)
        let additional_tags = vec!["tag3".to_string(), "tag1".to_string()]; // tag1 is duplicate
        ops.add_tags(&session_id, additional_tags)
            .await
            .expect("Failed to add tags");

        let tags = ops.get_tags(&session_id).await.expect("Failed to get tags");
        assert_eq!(tags, vec!["tag1", "tag2", "tag3"]); // No duplicates

        // Test remove_tags
        let tags_to_remove = vec!["tag2".to_string()];
        ops.remove_tags(&session_id, &tags_to_remove)
            .await
            .expect("Failed to remove tags");

        let tags = ops.get_tags(&session_id).await.expect("Failed to get tags");
        assert_eq!(tags, vec!["tag1", "tag3"]);

        // Test has_tag
        assert!(ops
            .has_tag(&session_id, "tag1")
            .await
            .expect("Failed to check tag"));
        assert!(!ops
            .has_tag(&session_id, "nonexistent")
            .await
            .expect("Failed to check tag"));
    }
    #[tokio::test]
    async fn test_get_session_stats() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager.clone());

        // Add some artifacts
        for i in 0..3 {
            session_manager
                .store_artifact(
                    &session_id,
                    crate::sessions::artifact::ArtifactType::UserInput,
                    format!("test_artifact_{i}.txt"),
                    format!("Test content {i}").as_bytes().to_vec(),
                    None,
                )
                .await
                .expect("Failed to store artifact");
        }

        // Get stats
        let stats = ops
            .get_session_stats(&session_id)
            .await
            .expect("Failed to get stats");

        assert_eq!(stats.session_id, session_id);
        assert_eq!(stats.artifact_count, 3);
        assert!(stats.total_artifact_size > 0);
        assert_eq!(stats.tag_count, 2); // "test", "unit"
        assert_eq!(stats.status, "Active");
    }
    #[tokio::test]
    async fn test_export_session() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager.clone());

        // Add an artifact
        session_manager
            .store_artifact(
                &session_id,
                crate::sessions::artifact::ArtifactType::ToolResult,
                "export_test.json".to_string(),
                r#"{"test": "data"}"#.as_bytes().to_vec(),
                None,
            )
            .await
            .expect("Failed to store artifact");

        // Test export without artifacts
        let export = ops
            .export_session(&session_id, false)
            .await
            .expect("Failed to export session");

        assert_eq!(export.session_id, session_id);
        assert!(export.artifacts.is_none());
        assert_eq!(export.metadata.name, Some("test-session".to_string()));

        // Test export with artifacts
        let export = ops
            .export_session(&session_id, true)
            .await
            .expect("Failed to export session");

        assert!(export.artifacts.is_some());
        assert_eq!(export.artifacts.as_ref().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn test_update_metadata_edge_cases() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager);

        // Test with empty updates
        let empty_updates = HashMap::new();
        ops.update_metadata(&session_id, empty_updates)
            .await
            .expect("Failed to update with empty map");

        // Test with invalid tag types (should be filtered out)
        let mut updates = HashMap::new();
        updates.insert(
            "tags".to_string(),
            serde_json::json!([123, "valid-tag", null, true]),
        );

        ops.update_metadata(&session_id, updates)
            .await
            .expect("Failed to update metadata");

        let tags = ops.get_tags(&session_id).await.expect("Failed to get tags");
        assert_eq!(tags, vec!["valid-tag"]); // Only string values kept
    }
    #[tokio::test]
    async fn test_concurrent_tag_operations() {
        let (session_manager, session_id) = setup_test_env().await;
        let ops = Arc::new(SessionOperations::new(session_manager));

        // Spawn multiple concurrent tag operations
        let mut handles = vec![];

        for i in 0..5 {
            let ops_clone = ops.clone();
            let session_id_clone = session_id;
            let handle = tokio::spawn(async move {
                ops_clone
                    .add_tags(&session_id_clone, vec![format!("concurrent-{}", i)])
                    .await
            });
            handles.push(handle);
        }

        // Wait for all operations to complete
        for handle in handles {
            handle.await.unwrap().expect("Failed to add tags");
        }

        // Verify all tags were added
        let tags = ops.get_tags(&session_id).await.expect("Failed to get tags");

        // Should have initial tags plus all concurrent ones
        assert!(tags.len() >= 7); // 2 initial + 5 concurrent
        for i in 0..5 {
            assert!(tags.contains(&format!("concurrent-{i}")));
        }
    }
    #[tokio::test]
    async fn test_nonexistent_session() {
        let (session_manager, _) = setup_test_env().await;
        let ops = SessionOperations::new(session_manager);

        let nonexistent_id = SessionId::new();

        // All operations should fail with session not found
        assert!(ops.get_tags(&nonexistent_id).await.is_err());
        assert!(ops
            .set_tags(&nonexistent_id, vec!["tag".to_string()])
            .await
            .is_err());
        assert!(ops.has_tag(&nonexistent_id, "tag").await.is_err());
        assert!(ops.get_session_stats(&nonexistent_id).await.is_err());
        assert!(ops.export_session(&nonexistent_id, false).await.is_err());
    }
}
