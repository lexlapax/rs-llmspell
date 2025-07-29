//! ABOUTME: Core session implementation providing lifecycle management and state persistence
//! ABOUTME: Integrates with Phase 5 state persistence for automatic session context saving

use crate::types::CreateSessionOptions;
use crate::{Result, SessionConfig, SessionError, SessionId, SessionMetadata, SessionStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Core session structure managing lifecycle and state
#[derive(Debug, Clone)]
pub struct Session {
    /// Session metadata
    pub metadata: Arc<RwLock<SessionMetadata>>,
    /// Session configuration
    pub config: SessionConfig,
    /// Session-specific state storage
    state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Artifact IDs associated with this session
    artifact_ids: Arc<RwLock<Vec<String>>>,
}

impl Session {
    /// Create a new session with the given options
    pub fn new(options: CreateSessionOptions) -> Self {
        let id = SessionId::new();
        let metadata = SessionMetadata::new(id, options.name.clone(), options.created_by.clone());

        // Apply initial metadata from options
        let mut metadata = metadata;
        if let Some(desc) = options.description {
            metadata.description = Some(desc);
        }
        for tag in options.tags {
            metadata.add_tag(tag);
        }
        if let Some(parent_id) = options.parent_session_id {
            metadata.parent_session_id = Some(parent_id);
        }

        Self {
            metadata: Arc::new(RwLock::new(metadata)),
            config: options.config.unwrap_or_default(),
            state: Arc::new(RwLock::new(HashMap::new())),
            artifact_ids: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get the session ID
    pub async fn id(&self) -> SessionId {
        self.metadata.read().await.id
    }

    /// Get the current session status
    pub async fn status(&self) -> SessionStatus {
        self.metadata.read().await.status
    }

    /// Suspend the session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is not in Active state
    pub async fn suspend(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        match metadata.status {
            SessionStatus::Active => {
                metadata.update_status(SessionStatus::Suspended);
                Ok(())
            }
            status => Err(SessionError::InvalidStateTransition {
                from: status,
                to: SessionStatus::Suspended,
            }),
        }
    }

    /// Resume a suspended session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is not in Suspended state
    pub async fn resume(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        match metadata.status {
            SessionStatus::Suspended => {
                metadata.update_status(SessionStatus::Active);
                Ok(())
            }
            status => Err(SessionError::InvalidStateTransition {
                from: status,
                to: SessionStatus::Active,
            }),
        }
    }

    /// Complete the session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is not in Active or Suspended state
    pub async fn complete(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        match metadata.status {
            SessionStatus::Active | SessionStatus::Suspended => {
                metadata.update_status(SessionStatus::Completed);
                Ok(())
            }
            status => Err(SessionError::InvalidStateTransition {
                from: status,
                to: SessionStatus::Completed,
            }),
        }
    }

    /// Mark the session as failed
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is already in a terminal state
    pub async fn fail(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        if metadata.status.is_terminal() {
            Err(SessionError::InvalidStateTransition {
                from: metadata.status,
                to: SessionStatus::Failed,
            })
        } else {
            metadata.update_status(SessionStatus::Failed);
            Ok(())
        }
    }

    /// Add an artifact ID to this session
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    pub async fn add_artifact(&self, artifact_id: String) -> Result<()> {
        let mut artifacts = self.artifact_ids.write().await;
        let mut metadata = self.metadata.write().await;

        if !artifacts.contains(&artifact_id) {
            artifacts.push(artifact_id);
            metadata.artifact_count = artifacts.len();
            metadata.operation_count += 1;
        }

        Ok(())
    }

    /// Get all artifact IDs for this session
    pub async fn artifact_ids(&self) -> Vec<String> {
        self.artifact_ids.read().await.clone()
    }

    /// Set a session state value
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    pub async fn set_state(&self, key: String, value: serde_json::Value) -> Result<()> {
        let mut state = self.state.write().await;
        let mut metadata = self.metadata.write().await;

        state.insert(key, value);
        metadata.operation_count += 1;

        Ok(())
    }

    /// Get a session state value
    pub async fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        self.state.read().await.get(key).cloned()
    }

    /// Get all session state
    pub async fn get_all_state(&self) -> HashMap<String, serde_json::Value> {
        self.state.read().await.clone()
    }

    /// Clear session state
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    pub async fn clear_state(&self) -> Result<()> {
        let mut state = self.state.write().await;
        let mut metadata = self.metadata.write().await;

        state.clear();
        metadata.operation_count += 1;

        Ok(())
    }
}

/// Current snapshot format version
pub const SNAPSHOT_VERSION: u32 = 1;

/// Serializable representation of a session for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Session configuration
    pub config: SessionConfig,
    /// Session state
    pub state: HashMap<String, serde_json::Value>,
    /// Artifact IDs
    pub artifact_ids: Vec<String>,
    /// Snapshot timestamp
    pub snapshot_at: DateTime<Utc>,
    /// Snapshot format version
    #[serde(default = "default_version")]
    pub version: u32,
}

fn default_version() -> u32 {
    0 // For backward compatibility with snapshots that don't have a version
}

impl Session {
    /// Create a snapshot of the session for persistence
    pub async fn snapshot(&self) -> SessionSnapshot {
        SessionSnapshot {
            metadata: self.metadata.read().await.clone(),
            config: self.config.clone(),
            state: self.state.read().await.clone(),
            artifact_ids: self.artifact_ids.read().await.clone(),
            snapshot_at: Utc::now(),
            version: SNAPSHOT_VERSION,
        }
    }

    /// Restore a session from a snapshot
    pub fn from_snapshot(snapshot: SessionSnapshot) -> Self {
        Self {
            metadata: Arc::new(RwLock::new(snapshot.metadata)),
            config: snapshot.config,
            state: Arc::new(RwLock::new(snapshot.state)),
            artifact_ids: Arc::new(RwLock::new(snapshot.artifact_ids)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_lifecycle() {
        let session = Session::new(CreateSessionOptions::default());

        // Check initial state
        assert_eq!(session.status().await, SessionStatus::Active);

        // Test suspend
        session.suspend().await.unwrap();
        assert_eq!(session.status().await, SessionStatus::Suspended);

        // Test resume
        session.resume().await.unwrap();
        assert_eq!(session.status().await, SessionStatus::Active);

        // Test complete
        session.complete().await.unwrap();
        assert_eq!(session.status().await, SessionStatus::Completed);

        // Can't transition from terminal state
        assert!(session.suspend().await.is_err());
    }

    #[tokio::test]
    async fn test_session_state() {
        let session = Session::new(CreateSessionOptions::default());

        // Set state
        session
            .set_state("key1".to_string(), serde_json::json!({"value": 42}))
            .await
            .unwrap();

        // Get state
        let value = session.get_state("key1").await.unwrap();
        assert_eq!(value["value"], 42);

        // Clear state
        session.clear_state().await.unwrap();
        assert!(session.get_state("key1").await.is_none());
    }

    #[tokio::test]
    async fn test_session_artifacts() {
        let session = Session::new(CreateSessionOptions::default());

        // Add artifacts
        session.add_artifact("artifact1".to_string()).await.unwrap();
        session.add_artifact("artifact2".to_string()).await.unwrap();

        // Check artifacts
        let artifacts = session.artifact_ids().await;
        assert_eq!(artifacts.len(), 2);
        assert!(artifacts.contains(&"artifact1".to_string()));
        assert!(artifacts.contains(&"artifact2".to_string()));

        // Check metadata updated
        let metadata = session.metadata.read().await;
        assert_eq!(metadata.artifact_count, 2);
    }
}
