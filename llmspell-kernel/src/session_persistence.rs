//! Session persistence integration for Jupyter kernel
//!
//! Maps Jupyter kernel sessions to llmspell-sessions and provides
//! state persistence using llmspell-state-persistence.

use anyhow::Result;
use llmspell_sessions::SessionId;
use llmspell_state_persistence::{
    config::{PersistenceConfig, SledConfig, StorageBackendType},
    manager::StateManager,
    StateScope,
};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Session state information
#[derive(Debug, Clone)]
pub struct SessionState {
    /// Session ID
    pub session_id: SessionId,
    /// Jupyter session ID
    pub jupyter_id: String,
    /// Kernel ID this session belongs to
    pub kernel_id: String,
    /// Session creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity time
    pub last_activity: chrono::DateTime<chrono::Utc>,
    /// Execution count
    pub execution_count: u32,
    /// Active/suspended state
    pub active: bool,
}

/// Maps Jupyter session IDs to persistent sessions
#[derive(Clone)]
pub struct SessionMapper {
    /// Mapping from Jupyter session ID to session state
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    /// State manager from llmspell-state-persistence
    state_manager: Arc<StateManager>,
}

impl SessionMapper {
    /// Create a new session mapper with in-memory persistence
    ///
    /// # Errors
    /// Returns error if state manager initialization fails
    pub async fn new() -> Result<Self> {
        // Use default in-memory state manager
        let state_manager = Arc::new(
            StateManager::new()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create state manager: {}", e))?,
        );

        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            state_manager,
        })
    }

    /// Create a new session mapper with file-based persistence
    ///
    /// # Errors
    /// Returns error if state manager initialization fails
    pub async fn new_with_persistence(state_dir: PathBuf) -> Result<Self> {
        // Create Sled backend configuration
        let backend_type = StorageBackendType::Sled(SledConfig {
            path: state_dir.join("kernel_state.db"),
            cache_capacity: 64 * 1024 * 1024, // 64MB cache
            use_compression: true,
        });

        // Create persistence config
        let config = PersistenceConfig::builder()
            .enabled(true)
            .compression(true)
            .build();

        // Create state manager with file persistence
        let state_manager = Arc::new(
            StateManager::with_backend(backend_type, config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create state manager: {}", e))?,
        );

        // Start with empty sessions - will be restored later via restore_all_sessions()
        Ok(Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            state_manager,
        })
    }

    /// Create or get a session for a Jupyter session ID
    ///
    /// # Errors
    /// Returns error if session creation or persistence fails
    pub async fn get_or_create_session(
        &self,
        jupyter_session_id: &str,
        kernel_id: &str,
    ) -> Result<SessionId> {
        // Check if session already exists
        {
            let sessions = self.sessions.read().await;
            if let Some(session) = sessions.get(jupyter_session_id) {
                return Ok(session.session_id);
            }
        }

        // Create new session
        let session_id = SessionId::new();
        let now = chrono::Utc::now();
        let session_state = SessionState {
            session_id,
            jupyter_id: jupyter_session_id.to_string(),
            kernel_id: kernel_id.to_string(),
            created_at: now,
            last_activity: now,
            execution_count: 0,
            active: true,
        };

        // Store session state
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(jupyter_session_id.to_string(), session_state);
        }

        // Persist initial session state
        let scope = StateScope::Session(session_id.to_string());
        self.state_manager
            .set(scope.clone(), "created_at", serde_json::to_value(now)?)
            .await?;
        self.state_manager
            .set(scope, "kernel_id", Value::String(kernel_id.to_string()))
            .await?;

        Ok(session_id)
    }

    /// Get the session state for a Jupyter session ID
    pub async fn get_session(&self, jupyter_session_id: &str) -> Option<SessionState> {
        self.sessions.read().await.get(jupyter_session_id).cloned()
    }

    /// Get the llmspell `SessionId` for a Jupyter session ID
    pub async fn get_session_id(&self, jupyter_session_id: &str) -> Option<SessionId> {
        self.sessions
            .read()
            .await
            .get(jupyter_session_id)
            .map(|s| s.session_id)
    }

    /// Store kernel state for a session
    ///
    /// # Errors
    /// Returns error if state storage fails
    pub async fn store_kernel_state(
        &self,
        session_id: &SessionId,
        key: &str,
        value: Value,
    ) -> Result<()> {
        // Use session-scoped state
        let scope = StateScope::Session(session_id.to_string());
        self.state_manager.set(scope, key, value).await?;
        Ok(())
    }

    /// Retrieve kernel state for a session
    ///
    /// # Errors
    /// Returns error if state retrieval fails
    pub async fn get_kernel_state(
        &self,
        session_id: &SessionId,
        key: &str,
    ) -> Result<Option<Value>> {
        let scope = StateScope::Session(session_id.to_string());
        self.state_manager
            .get(scope, key)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get kernel state: {}", e))
    }

    /// Store execution count for a session
    ///
    /// # Errors
    /// Returns error if execution count storage fails
    pub async fn store_execution_count(&self, session_id: &SessionId, count: u32) -> Result<()> {
        // Update in-memory state
        let mut sessions = self.sessions.write().await;
        for session in sessions.values_mut() {
            if session.session_id == *session_id {
                session.execution_count = count;
                session.last_activity = chrono::Utc::now();
                break;
            }
        }
        drop(sessions);

        // Persist to state manager
        self.store_kernel_state(session_id, "execution_count", Value::from(count))
            .await
    }

    /// Get execution count for a session
    ///
    /// # Errors
    /// Returns error if execution count retrieval fails
    pub async fn get_execution_count(&self, session_id: &SessionId) -> Result<u32> {
        // Try to get from in-memory first
        let sessions = self.sessions.read().await;
        for session in sessions.values() {
            if session.session_id == *session_id {
                return Ok(session.execution_count);
            }
        }
        drop(sessions);

        // Fall back to persisted state
        let value = self.get_kernel_state(session_id, "execution_count").await?;
        match value {
            Some(Value::Number(n)) => {
                let val = n.as_u64().unwrap_or(0).min(u64::from(u32::MAX));
                Ok(u32::try_from(val).unwrap_or(u32::MAX))
            }
            _ => Ok(0),
        }
    }

    /// Store kernel variables for a session
    ///
    /// # Errors
    /// Returns error if variable storage fails
    pub async fn store_variables(
        &self,
        session_id: &SessionId,
        variables: HashMap<String, Value>,
    ) -> Result<()> {
        // Convert HashMap to serde_json::Map
        let map = variables.into_iter().collect();
        self.store_kernel_state(session_id, "variables", Value::Object(map))
            .await
    }

    /// Get kernel variables for a session
    ///
    /// # Errors
    /// Returns error if variable retrieval fails
    pub async fn get_variables(&self, session_id: &SessionId) -> Result<HashMap<String, Value>> {
        let value = self.get_kernel_state(session_id, "variables").await?;
        match value {
            Some(Value::Object(map)) => Ok(map.into_iter().collect()),
            _ => Ok(HashMap::new()),
        }
    }

    /// Mark a session as active
    ///
    /// # Errors
    /// Always returns Ok currently
    pub async fn activate_session(&self, session_id: &SessionId) -> Result<()> {
        {
            let mut sessions = self.sessions.write().await;
            for session in sessions.values_mut() {
                if session.session_id == *session_id {
                    session.active = true;
                    session.last_activity = chrono::Utc::now();
                    break;
                }
            }
        }
        Ok(())
    }

    /// Suspend a session
    ///
    /// # Errors
    /// Always returns Ok currently
    pub async fn suspend_session(&self, session_id: &SessionId) -> Result<()> {
        {
            let mut sessions = self.sessions.write().await;
            for session in sessions.values_mut() {
                if session.session_id == *session_id {
                    session.active = false;
                    session.last_activity = chrono::Utc::now();
                    break;
                }
            }
        }
        Ok(())
    }

    /// Complete a session
    ///
    /// # Errors
    /// Returns error if state persistence fails
    pub async fn complete_session(&self, jupyter_session_id: &str) -> Result<()> {
        // Remove from in-memory state
        let session = {
            let mut sessions = self.sessions.write().await;
            sessions.remove(jupyter_session_id)
        };

        if let Some(session) = session {
            // Persist completion to state manager
            let scope = StateScope::Session(session.session_id.to_string());
            self.state_manager
                .set(
                    scope.clone(),
                    "completed_at",
                    serde_json::to_value(chrono::Utc::now())?,
                )
                .await?;
            self.state_manager
                .set(
                    scope,
                    "final_execution_count",
                    Value::from(session.execution_count),
                )
                .await?;
        }
        Ok(())
    }

    /// Clean up all inactive sessions
    ///
    /// # Errors
    /// Always returns Ok currently
    pub async fn cleanup_inactive_sessions(
        &self,
        max_inactive_duration: chrono::Duration,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let mut sessions = self.sessions.write().await;

        let mut to_remove = Vec::new();
        for (jupyter_id, session) in sessions.iter() {
            if !session.active {
                let inactive_duration = now - session.last_activity;
                if inactive_duration > max_inactive_duration {
                    to_remove.push(jupyter_id.clone());
                }
            }
        }

        for jupyter_id in to_remove {
            sessions.remove(&jupyter_id);
        }
        drop(sessions);

        Ok(())
    }

    /// Save all current sessions to persistent storage
    ///
    /// # Errors
    /// Returns error if persistence fails
    pub async fn save_all_sessions(&self) -> Result<()> {
        let sessions = self.sessions.read().await;

        // Save each session's metadata
        for (jupyter_id, session) in sessions.iter() {
            let scope = StateScope::Global; // Use global scope for kernel-wide state
            let session_key = format!("session:{jupyter_id}");

            // Create session metadata object
            let metadata = serde_json::json!({
                "session_id": session.session_id.to_string(),
                "jupyter_id": session.jupyter_id,
                "kernel_id": session.kernel_id,
                "created_at": session.created_at,
                "last_activity": session.last_activity,
                "execution_count": session.execution_count,
                "active": session.active,
            });

            self.state_manager
                .set(scope, &session_key, metadata)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save session {}: {}", jupyter_id, e))?;
        }

        // Save the list of session IDs for restoration
        let session_ids: Vec<String> = sessions.keys().cloned().collect();
        drop(sessions);
        self.state_manager
            .set(
                StateScope::Global,
                "kernel:session_list",
                serde_json::to_value(session_ids)?,
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to save session list: {}", e))?;

        Ok(())
    }

    /// Mark kernel shutdown as clean
    ///
    /// # Errors
    /// Returns error if marking fails
    pub async fn mark_clean_shutdown(&self) -> Result<()> {
        self.state_manager
            .set(
                StateScope::Global,
                "kernel:shutdown_marker",
                serde_json::json!({
                    "clean": true,
                    "timestamp": chrono::Utc::now(),
                }),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to mark clean shutdown: {}", e))
    }

    /// Check if last shutdown was clean or a crash
    ///
    /// # Errors
    /// Returns error if check fails
    pub async fn was_clean_shutdown(&self) -> Result<bool> {
        match self
            .state_manager
            .get(StateScope::Global, "kernel:shutdown_marker")
            .await
        {
            Ok(Some(marker)) => {
                let is_clean = marker
                    .get("clean")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);

                // Clear the marker after checking
                let _ = self
                    .state_manager
                    .delete(StateScope::Global, "kernel:shutdown_marker")
                    .await;

                Ok(is_clean)
            }
            Ok(None) => {
                // No marker means it was likely a crash or first run
                Ok(false)
            }
            Err(e) => {
                tracing::warn!("Failed to check shutdown marker: {}", e);
                Ok(false) // Assume crash on error
            }
        }
    }

    /// Restore all sessions from persistent storage
    ///
    /// Attempts to restore all sessions, continuing even if individual sessions fail.
    /// Corrupted sessions are logged and skipped.
    ///
    /// # Errors
    /// Returns error only if session list retrieval fails completely
    pub async fn restore_all_sessions(&self) -> Result<()> {
        // Get the list of session IDs
        let session_ids = match self
            .state_manager
            .get(StateScope::Global, "kernel:session_list")
            .await
        {
            Ok(Some(value)) => match serde_json::from_value::<Vec<String>>(value) {
                Ok(ids) => ids,
                Err(e) => {
                    tracing::error!("Failed to parse session list: {}", e);
                    return Ok(()); // Continue with empty session list
                }
            },
            Ok(None) => {
                tracing::info!("No sessions to restore");
                return Ok(());
            }
            Err(e) => {
                tracing::warn!("Failed to get session list: {}", e);
                return Ok(()); // Continue with empty session list
            }
        };

        tracing::info!("Attempting to restore {} sessions", session_ids.len());
        let mut restored_count = 0;
        let mut failed_count = 0;

        for jupyter_id in session_ids {
            let session_key = format!("session:{jupyter_id}");

            match self
                .state_manager
                .get(StateScope::Global, &session_key)
                .await
            {
                Ok(Some(metadata)) => {
                    // Try to restore this session, log and continue if it fails
                    match Self::parse_session_metadata(&jupyter_id, &metadata) {
                        Ok(session_state) => {
                            self.sessions
                                .write()
                                .await
                                .insert(jupyter_id.clone(), session_state);
                            restored_count += 1;
                            tracing::debug!("Restored session: {}", jupyter_id);
                        }
                        Err(e) => {
                            failed_count += 1;
                            tracing::error!(
                                "Failed to parse session {} metadata: {}. Skipping.",
                                jupyter_id,
                                e
                            );
                        }
                    }
                }
                Ok(None) => {
                    failed_count += 1;
                    tracing::warn!("Session {} metadata not found. Skipping.", jupyter_id);
                }
                Err(e) => {
                    failed_count += 1;
                    tracing::error!(
                        "Failed to get session {} metadata: {}. Skipping.",
                        jupyter_id,
                        e
                    );
                }
            }
        }

        tracing::info!(
            "Session restoration complete: {} restored, {} failed",
            restored_count,
            failed_count
        );

        Ok(())
    }

    /// Restore specific sessions from persistent storage
    ///
    /// # Errors
    /// Returns error only if session list retrieval fails
    pub async fn restore_sessions(&self, session_ids: Vec<String>) -> Result<()> {
        tracing::info!(
            "Attempting to restore {} specific sessions",
            session_ids.len()
        );
        let mut restored_count = 0;
        let mut failed_count = 0;

        for jupyter_id in session_ids {
            let session_key = format!("session:{jupyter_id}");

            match self
                .state_manager
                .get(StateScope::Global, &session_key)
                .await
            {
                Ok(Some(metadata)) => {
                    // Try to restore this session, log and continue if it fails
                    match Self::parse_session_metadata(&jupyter_id, &metadata) {
                        Ok(session_state) => {
                            self.sessions
                                .write()
                                .await
                                .insert(jupyter_id.clone(), session_state);
                            restored_count += 1;
                            tracing::debug!("Restored session: {}", jupyter_id);
                        }
                        Err(e) => {
                            failed_count += 1;
                            tracing::error!(
                                "Failed to parse session {} metadata: {}. Skipping.",
                                jupyter_id,
                                e
                            );
                        }
                    }
                }
                Ok(None) => {
                    failed_count += 1;
                    tracing::warn!("Session {} metadata not found. Skipping.", jupyter_id);
                }
                Err(e) => {
                    failed_count += 1;
                    tracing::error!(
                        "Failed to get session {} metadata: {}. Skipping.",
                        jupyter_id,
                        e
                    );
                }
            }
        }

        tracing::info!(
            "Partial session restoration complete: {} restored, {} failed",
            restored_count,
            failed_count
        );

        Ok(())
    }

    /// Parse session metadata from stored JSON value
    fn parse_session_metadata(jupyter_id: &str, metadata: &Value) -> Result<SessionState> {
        // Extract session_id
        let session_id_str = metadata
            .get("session_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing session_id"))?;

        let session_id = session_id_str
            .parse::<SessionId>()
            .map_err(|e| anyhow::anyhow!("Invalid session_id: {}", e))?;

        // Extract other fields with defaults for corrupted data
        let kernel_id = metadata
            .get("kernel_id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let created_at = metadata
            .get("created_at")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(chrono::Utc::now);

        let last_activity = metadata
            .get("last_activity")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(chrono::Utc::now);

        let execution_count = u32::try_from(
            metadata
                .get("execution_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
        )
        .unwrap_or(0);

        let active = metadata
            .get("active")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        Ok(SessionState {
            session_id,
            jupyter_id: jupyter_id.to_string(),
            kernel_id,
            created_at,
            last_activity,
            execution_count,
            active,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_session_id_mapping() {
        let mapper = SessionMapper::new().await.unwrap();
        let jupyter_id = "test-jupyter-session";
        let kernel_id = "test-kernel";

        // Create session
        let session_id1 = mapper
            .get_or_create_session(jupyter_id, kernel_id)
            .await
            .unwrap();

        // Should return same session ID for same Jupyter ID
        let session_id2 = mapper
            .get_or_create_session(jupyter_id, kernel_id)
            .await
            .unwrap();
        assert_eq!(session_id1, session_id2);

        // Check that session exists
        let retrieved = mapper.get_session_id(jupyter_id).await;
        assert_eq!(retrieved, Some(session_id1));

        // Check session state
        let session_state = mapper.get_session(jupyter_id).await;
        assert!(session_state.is_some());
        assert_eq!(session_state.unwrap().jupyter_id, jupyter_id);
    }

    #[tokio::test]
    async fn test_session_state_synchronization() {
        let mapper = SessionMapper::new().await.unwrap();
        let jupyter_id = "state-test-session";
        let kernel_id = "test-kernel";

        let session_id = mapper
            .get_or_create_session(jupyter_id, kernel_id)
            .await
            .unwrap();

        // Store execution count
        mapper.store_execution_count(&session_id, 42).await.unwrap();

        // Retrieve execution count
        let count = mapper.get_execution_count(&session_id).await.unwrap();
        assert_eq!(count, 42);

        // Store variables
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), Value::from(10));
        vars.insert("y".to_string(), Value::from("hello"));
        mapper.store_variables(&session_id, vars).await.unwrap();

        // Retrieve variables
        let retrieved_vars = mapper.get_variables(&session_id).await.unwrap();
        assert_eq!(retrieved_vars.get("x"), Some(&Value::from(10)));
        assert_eq!(retrieved_vars.get("y"), Some(&Value::from("hello")));
    }

    #[tokio::test]
    async fn test_session_cleanup_on_disconnect() {
        let mapper = SessionMapper::new().await.unwrap();
        let jupyter_id = "cleanup-test-session";
        let kernel_id = "test-kernel";

        let _session_id = mapper
            .get_or_create_session(jupyter_id, kernel_id)
            .await
            .unwrap();

        // Session should exist
        assert!(mapper.get_session_id(jupyter_id).await.is_some());

        // Complete session using jupyter_id
        mapper.complete_session(jupyter_id).await.unwrap();

        // Session should be removed
        assert!(mapper.get_session_id(jupyter_id).await.is_none());
        assert!(mapper.get_session(jupyter_id).await.is_none());
    }

    #[tokio::test]
    async fn test_multi_client_session_isolation() {
        let mapper = SessionMapper::new().await.unwrap();

        // Create sessions for different clients
        let session1 = mapper
            .get_or_create_session("client1-session", "kernel1")
            .await
            .unwrap();
        let session2 = mapper
            .get_or_create_session("client2-session", "kernel1")
            .await
            .unwrap();

        // Sessions should be different
        assert_ne!(session1, session2);

        // Store different state for each session
        mapper.store_execution_count(&session1, 10).await.unwrap();
        mapper.store_execution_count(&session2, 20).await.unwrap();

        // Each session should have its own state
        let count1 = mapper.get_execution_count(&session1).await.unwrap();
        let count2 = mapper.get_execution_count(&session2).await.unwrap();
        assert_eq!(count1, 10);
        assert_eq!(count2, 20);
    }

    #[tokio::test]
    async fn test_file_based_persistence() {
        // Create a temporary directory for state storage
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Create a session and save it
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            let jupyter_id = "persistent-session";
            let kernel_id = "test-kernel";

            // Create a session
            let session_id = mapper
                .get_or_create_session(jupyter_id, kernel_id)
                .await
                .unwrap();

            // Store some state
            mapper.store_execution_count(&session_id, 42).await.unwrap();

            let mut vars = HashMap::new();
            vars.insert("x".to_string(), Value::from(100));
            vars.insert("y".to_string(), Value::from("persistent"));
            mapper.store_variables(&session_id, vars).await.unwrap();

            // Save all sessions
            mapper.save_all_sessions().await.unwrap();
        }

        // Create a new mapper and restore state
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Restore sessions
            mapper.restore_all_sessions().await.unwrap();

            let jupyter_id = "persistent-session";

            // Check if session was restored
            let session = mapper.get_session(jupyter_id).await;
            assert!(session.is_some(), "Session should be restored");

            if let Some(session_state) = session {
                // Check restored execution count
                let count = mapper
                    .get_execution_count(&session_state.session_id)
                    .await
                    .unwrap();
                assert_eq!(count, 42, "Execution count should be restored");

                // Check restored variables
                let vars = mapper
                    .get_variables(&session_state.session_id)
                    .await
                    .unwrap();
                assert_eq!(vars.get("x"), Some(&Value::from(100)));
                assert_eq!(vars.get("y"), Some(&Value::from("persistent")));
            }
        }
    }

    #[tokio::test]
    async fn test_kernel_restart_preserves_state() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Simulate first kernel run
        let session_id = {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            let session_id = mapper
                .get_or_create_session("restart-test", "kernel-1")
                .await
                .unwrap();

            // Simulate some work
            mapper.store_execution_count(&session_id, 5).await.unwrap();
            mapper
                .store_kernel_state(&session_id, "last_result", Value::from("success"))
                .await
                .unwrap();

            // Save on "shutdown"
            mapper.save_all_sessions().await.unwrap();

            session_id
        };

        // Simulate kernel restart
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Restore state
            mapper.restore_all_sessions().await.unwrap();

            // Get the same session
            let restored_session_id = mapper
                .get_or_create_session("restart-test", "kernel-1")
                .await
                .unwrap();

            // Session ID should be the same
            assert_eq!(session_id, restored_session_id);

            // State should be preserved
            let count = mapper
                .get_execution_count(&restored_session_id)
                .await
                .unwrap();
            assert_eq!(count, 5);

            let last_result = mapper
                .get_kernel_state(&restored_session_id, "last_result")
                .await
                .unwrap();
            assert_eq!(last_result, Some(Value::from("success")));
        }
    }

    #[tokio::test]
    async fn test_state_corruption_recovery() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // First, create and save valid sessions
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Create multiple sessions
            for i in 0u32..3 {
                let jupyter_id = format!("session-{i}");
                let session_id = mapper
                    .get_or_create_session(&jupyter_id, "kernel-1")
                    .await
                    .unwrap();
                mapper.store_execution_count(&session_id, i).await.unwrap();
            }

            mapper.save_all_sessions().await.unwrap();
        }

        // Now corrupt one session's data
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Directly write corrupted data for session-1
            mapper
                .state_manager
                .set(
                    StateScope::Global,
                    "session:session-1",
                    serde_json::json!({
                        // Missing required session_id field
                        "jupyter_id": "session-1",
                        "kernel_id": "kernel-1",
                        "invalid_data": "this will cause parsing to fail"
                    }),
                )
                .await
                .unwrap();
        }

        // Try to restore all sessions - should succeed with partial restoration
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // This should not fail, but log errors for corrupted session
            mapper.restore_all_sessions().await.unwrap();

            // Check that valid sessions were restored
            let session0 = mapper.get_session("session-0").await;
            assert!(session0.is_some(), "Valid session-0 should be restored");

            let session1 = mapper.get_session("session-1").await;
            assert!(
                session1.is_none(),
                "Corrupted session-1 should not be restored"
            );

            let session2 = mapper.get_session("session-2").await;
            assert!(session2.is_some(), "Valid session-2 should be restored");

            // Verify restored data is correct
            if let Some(s0) = session0 {
                let count = mapper.get_execution_count(&s0.session_id).await.unwrap();
                assert_eq!(count, 0);
            }

            if let Some(s2) = session2 {
                let count = mapper.get_execution_count(&s2.session_id).await.unwrap();
                assert_eq!(count, 2);
            }
        }
    }

    #[tokio::test]
    async fn test_large_state_objects() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Create large state objects
        let large_string = "x".repeat(1024 * 1024); // 1MB string
        let mut large_map = HashMap::new();
        for i in 0..1000 {
            large_map.insert(format!("key_{i}"), Value::from(format!("value_{i}")));
        }

        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            let session_id = mapper
                .get_or_create_session("large-session", "kernel-1")
                .await
                .unwrap();

            // Store large objects
            mapper
                .store_kernel_state(
                    &session_id,
                    "large_string",
                    Value::from(large_string.clone()),
                )
                .await
                .unwrap();

            mapper
                .store_kernel_state(
                    &session_id,
                    "large_map",
                    serde_json::to_value(&large_map).unwrap(),
                )
                .await
                .unwrap();

            // Store many variables
            let mut vars = HashMap::new();
            for i in 0..100 {
                vars.insert(format!("var_{i}"), Value::from(i * 10));
            }
            mapper.store_variables(&session_id, vars).await.unwrap();

            mapper.save_all_sessions().await.unwrap();
        }

        // Restore and verify large objects
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            mapper.restore_all_sessions().await.unwrap();

            let session = mapper.get_session("large-session").await.unwrap();

            // Verify large string
            let restored_string = mapper
                .get_kernel_state(&session.session_id, "large_string")
                .await
                .unwrap();
            assert_eq!(restored_string, Some(Value::from(large_string)));

            // Verify large map
            let restored_map = mapper
                .get_kernel_state(&session.session_id, "large_map")
                .await
                .unwrap()
                .unwrap();

            for i in 0..1000 {
                let key = format!("key_{i}");
                let expected = format!("value_{i}");
                assert_eq!(
                    restored_map.get(&key).and_then(|v| v.as_str()),
                    Some(expected.as_str())
                );
            }

            // Verify variables
            let vars = mapper.get_variables(&session.session_id).await.unwrap();
            assert_eq!(vars.len(), 100);
            for i in 0..100 {
                let key = format!("var_{i}");
                assert_eq!(vars.get(&key), Some(&Value::from(i * 10)));
            }
        }
    }

    #[tokio::test]
    async fn test_crash_vs_clean_shutdown() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Test crash scenario (no clean shutdown marker)
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            mapper
                .get_or_create_session("test-session", "kernel-1")
                .await
                .unwrap();

            mapper.save_all_sessions().await.unwrap();
            // Simulate crash - no mark_clean_shutdown() call
        }

        // Check that crash is detected
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            let was_clean = mapper.was_clean_shutdown().await.unwrap();
            assert!(!was_clean, "Should detect crash (no clean shutdown marker)");
        }

        // Test clean shutdown scenario
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            mapper
                .get_or_create_session("test-session", "kernel-1")
                .await
                .unwrap();

            mapper.save_all_sessions().await.unwrap();
            mapper.mark_clean_shutdown().await.unwrap(); // Clean shutdown
        }

        // Check that clean shutdown is detected
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            let was_clean = mapper.was_clean_shutdown().await.unwrap();
            assert!(was_clean, "Should detect clean shutdown");

            // Check marker is cleared after checking
            let was_clean_again = mapper.was_clean_shutdown().await.unwrap();
            assert!(
                !was_clean_again,
                "Marker should be cleared after first check"
            );
        }
    }

    #[tokio::test]
    async fn test_partial_state_restoration() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Create multiple sessions
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            for i in 0u32..5 {
                let jupyter_id = format!("session-{i}");
                let session_id = mapper
                    .get_or_create_session(&jupyter_id, "kernel-1")
                    .await
                    .unwrap();
                mapper
                    .store_execution_count(&session_id, i * 10)
                    .await
                    .unwrap();
            }

            mapper.save_all_sessions().await.unwrap();
        }

        // Restore only specific sessions
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Restore only sessions 1, 3, and 4
            let sessions_to_restore = vec![
                "session-1".to_string(),
                "session-3".to_string(),
                "session-4".to_string(),
            ];

            mapper.restore_sessions(sessions_to_restore).await.unwrap();

            // Check which sessions were restored
            assert!(mapper.get_session("session-0").await.is_none());
            assert!(mapper.get_session("session-1").await.is_some());
            assert!(mapper.get_session("session-2").await.is_none());
            assert!(mapper.get_session("session-3").await.is_some());
            assert!(mapper.get_session("session-4").await.is_some());

            // Verify restored data
            if let Some(s1) = mapper.get_session("session-1").await {
                let count = mapper.get_execution_count(&s1.session_id).await.unwrap();
                assert_eq!(count, 10);
            }

            if let Some(s3) = mapper.get_session("session-3").await {
                let count = mapper.get_execution_count(&s3.session_id).await.unwrap();
                assert_eq!(count, 30);
            }
        }
    }

    #[tokio::test]
    async fn test_comprehensive_restart() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path().to_path_buf();

        // Initial kernel run with various operations
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Create multiple sessions with different states
            let session1 = mapper
                .get_or_create_session("notebook-1", "kernel-1")
                .await
                .unwrap();
            mapper.store_execution_count(&session1, 15).await.unwrap();
            let mut vars1 = HashMap::new();
            vars1.insert("result".to_string(), Value::from("completed"));
            vars1.insert(
                "data".to_string(),
                serde_json::to_value(vec![1, 2, 3]).unwrap(),
            );
            mapper.store_variables(&session1, vars1).await.unwrap();

            let session2 = mapper
                .get_or_create_session("notebook-2", "kernel-1")
                .await
                .unwrap();
            mapper.store_execution_count(&session2, 7).await.unwrap();
            mapper
                .store_kernel_state(&session2, "checkpoint", Value::from("step-3"))
                .await
                .unwrap();

            // Activate session1, suspend session2
            mapper.activate_session(&session1).await.unwrap();
            mapper.suspend_session(&session2).await.unwrap();

            // Save and mark clean shutdown
            mapper.save_all_sessions().await.unwrap();
            mapper.mark_clean_shutdown().await.unwrap();
        }

        // Simulate kernel restart and full restoration
        {
            let mapper = SessionMapper::new_with_persistence(state_dir.clone())
                .await
                .unwrap();

            // Check shutdown status
            let was_clean = mapper.was_clean_shutdown().await.unwrap();
            assert!(was_clean, "Should detect clean shutdown");

            // Restore all sessions
            mapper.restore_all_sessions().await.unwrap();

            // Verify session 1
            let session1 = mapper.get_session("notebook-1").await.unwrap();
            assert!(session1.active, "Session 1 should be active");
            assert_eq!(session1.execution_count, 15);

            let vars1 = mapper.get_variables(&session1.session_id).await.unwrap();
            assert_eq!(vars1.get("result"), Some(&Value::from("completed")));

            // Verify session 2
            let session2 = mapper.get_session("notebook-2").await.unwrap();
            assert!(!session2.active, "Session 2 should be suspended");
            assert_eq!(session2.execution_count, 7);

            let checkpoint = mapper
                .get_kernel_state(&session2.session_id, "checkpoint")
                .await
                .unwrap();
            assert_eq!(checkpoint, Some(Value::from("step-3")));

            // Continue work after restart
            mapper
                .store_execution_count(&session1.session_id, 16)
                .await
                .unwrap();
            mapper
                .store_execution_count(&session2.session_id, 8)
                .await
                .unwrap();

            // Verify continuity
            let count1 = mapper
                .get_execution_count(&session1.session_id)
                .await
                .unwrap();
            assert_eq!(count1, 16, "Should continue from restored state");

            let count2 = mapper
                .get_execution_count(&session2.session_id)
                .await
                .unwrap();
            assert_eq!(count2, 8, "Should continue from restored state");
        }
    }
}
