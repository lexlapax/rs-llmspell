//! Session persistence integration for Jupyter kernel
//!
//! Maps Jupyter kernel sessions to llmspell-sessions and provides
//! state persistence using llmspell-state-persistence.

use anyhow::Result;
use llmspell_sessions::SessionId;
use llmspell_state_persistence::{manager::StateManager, StateScope};
use serde_json::Value;
use std::collections::HashMap;
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
    /// Create a new session mapper
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
