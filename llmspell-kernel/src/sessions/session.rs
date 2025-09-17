//! ABOUTME: Core session implementation providing lifecycle management and state persistence
//! ABOUTME: Integrates with Phase 5 state persistence for automatic session context saving

use crate::sessions::types::CreateSessionOptions;
use crate::sessions::{
    Result, SessionConfig, SessionError, SessionId, SessionMetadata, SessionStatus,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, instrument, warn, Span};
use tracing::field::Empty;

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
    #[instrument(level = "info", skip(options), fields(
        session_id = Empty,
        session_name = ?options.name,
        created_by = ?options.created_by
    ))]
    pub fn new(options: CreateSessionOptions) -> Self {
        let id = SessionId::new();
        Span::current().record("session_id", &id.to_string());
        info!("Creating new session with id={}", id);

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
    #[instrument(level = "trace", skip(self))]
    pub async fn id(&self) -> SessionId {
        self.metadata.read().await.id
    }

    /// Get the current session status
    #[instrument(level = "trace", skip(self))]
    pub async fn status(&self) -> SessionStatus {
        self.metadata.read().await.status
    }

    /// Suspend the session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is not in Active state
    #[instrument(level = "info", skip(self), fields(session_id = Empty))]
    pub async fn suspend(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        Span::current().record("session_id", &metadata.id.to_string());

        match metadata.status {
            SessionStatus::Active => {
                info!("Suspending session {}", metadata.id);
                metadata.update_status(SessionStatus::Suspended);
                Ok(())
            }
            status => {
                warn!("Cannot suspend session in state {:?}", status);
                Err(SessionError::InvalidStateTransition {
                    from: status,
                    to: SessionStatus::Suspended,
                })
            }
        }
    }

    /// Resume a suspended session
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidStateTransition` if the session is not in Suspended state
    #[instrument(level = "info", skip(self), fields(session_id = Empty))]
    pub async fn resume(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        Span::current().record("session_id", &metadata.id.to_string());

        match metadata.status {
            SessionStatus::Suspended => {
                info!("Resuming session {}", metadata.id);
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
    #[instrument(level = "info", skip(self), fields(session_id = Empty))]
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
    #[instrument(level = "warn", skip(self), fields(session_id = Empty))]
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
    #[instrument(level = "debug", skip(self), fields(session_id = Empty, artifact_id = %artifact_id))]
    pub async fn add_artifact(&self, artifact_id: String) -> Result<()> {
        let mut artifacts = self.artifact_ids.write().await;
        let mut metadata = self.metadata.write().await;

        Span::current().record("session_id", &metadata.id.to_string());

        if !artifacts.contains(&artifact_id) {
            debug!("Adding artifact {} to session {}", artifact_id, metadata.id);
            artifacts.push(artifact_id);
            metadata.artifact_count = artifacts.len();
            metadata.operation_count += 1;
        }

        Ok(())
    }

    /// Get all artifact IDs for this session
    #[instrument(level = "trace", skip(self), fields(
        session_id = Empty,
        artifact_count = Empty
    ))]
    pub async fn artifact_ids(&self) -> Vec<String> {
        let artifacts = self.artifact_ids.read().await.clone();
        let metadata = self.metadata.read().await;
        Span::current().record("session_id", &metadata.id.to_string());
        Span::current().record("artifact_count", artifacts.len());
        artifacts
    }

    /// Set a session state value
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    #[instrument(level = "debug", skip(self, value), fields(
        session_id = Empty,
        state_key = %key
    ))]
    pub async fn set_state(&self, key: String, value: serde_json::Value) -> Result<()> {
        let mut state = self.state.write().await;
        let mut metadata = self.metadata.write().await;

        Span::current().record("session_id", &metadata.id.to_string());
        state.insert(key, value);
        metadata.operation_count += 1;

        Ok(())
    }

    /// Get a session state value
    #[instrument(level = "trace", skip(self), fields(
        session_id = Empty,
        state_key = %key
    ))]
    pub async fn get_state(&self, key: &str) -> Option<serde_json::Value> {
        let metadata = self.metadata.read().await;
        Span::current().record("session_id", &metadata.id.to_string());
        self.state.read().await.get(key).cloned()
    }

    /// Get all session state
    #[instrument(level = "trace", skip(self), fields(
        session_id = Empty,
        state_size = Empty
    ))]
    pub async fn get_all_state(&self) -> HashMap<String, serde_json::Value> {
        let state = self.state.read().await.clone();
        let metadata = self.metadata.read().await;
        Span::current().record("session_id", &metadata.id.to_string());
        Span::current().record("state_size", state.len());
        state
    }

    /// Clear session state
    ///
    /// # Errors
    ///
    /// Currently always succeeds, but returns Result for future error cases
    #[instrument(level = "info", skip(self), fields(session_id = Empty))]
    pub async fn clear_state(&self) -> Result<()> {
        let mut state = self.state.write().await;
        let mut metadata = self.metadata.write().await;

        Span::current().record("session_id", &metadata.id.to_string());
        info!("Clearing state for session {}", metadata.id);
        state.clear();
        metadata.operation_count += 1;

        Ok(())
    }

    /// Increment operation count and return the new sequence number
    ///
    /// # Errors
    ///
    /// Returns an error if the operation count cannot be incremented.
    #[instrument(level = "trace", skip(self), fields(
        session_id = Empty,
        new_count = Empty
    ))]
    pub async fn increment_operation_count(&self) -> Result<u64> {
        let mut metadata = self.metadata.write().await;
        metadata.operation_count += 1;
        metadata.updated_at = Utc::now();
        Span::current().record("session_id", &metadata.id.to_string());
        Span::current().record("new_count", metadata.operation_count);
        Ok(metadata.operation_count)
    }

    /// Increment artifact count
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact count cannot be incremented.
    #[instrument(level = "trace", skip(self), fields(session_id = Empty))]
    pub async fn increment_artifact_count(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        Span::current().record("session_id", &metadata.id.to_string());
        metadata.artifact_count += 1;
        metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Decrement artifact count
    ///
    /// # Errors
    ///
    /// Returns an error if the artifact count cannot be decremented.
    #[instrument(level = "trace", skip(self), fields(session_id = Empty))]
    pub async fn decrement_artifact_count(&self) -> Result<()> {
        let mut metadata = self.metadata.write().await;
        Span::current().record("session_id", &metadata.id.to_string());
        if metadata.artifact_count > 0 {
            metadata.artifact_count -= 1;
            metadata.updated_at = Utc::now();
        }
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
    #[instrument(level = "debug", skip(self), fields(session_id = Empty))]
    pub async fn snapshot(&self) -> SessionSnapshot {
        let metadata = self.metadata.read().await.clone();
        Span::current().record("session_id", &metadata.id.to_string());
        debug!("Creating snapshot for session {}", metadata.id);
        SessionSnapshot {
            metadata,
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
