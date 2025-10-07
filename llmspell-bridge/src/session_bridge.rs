//! ABOUTME: Core session bridge providing language-agnostic session operations
//! ABOUTME: Wraps `SessionManager` for script access with async operations

use llmspell_core::{error::LLMSpellError, Result};
use llmspell_kernel::sessions::{
    manager::SessionManager,
    session::Session,
    types::{CreateSessionOptions, SessionQuery},
    SessionId, SessionMetadata,
};
use std::sync::Arc;

/// Helper macro to convert `SessionError` to `LLMSpellError`
macro_rules! convert_err {
    ($expr:expr) => {
        $expr.map_err(|e| LLMSpellError::Component {
            message: format!("Session error: {}", e),
            source: None,
        })
    };
}

/// Core session bridge for language-agnostic session operations
///
/// This bridge wraps the `SessionManager` and provides async interfaces
/// for script languages, following the pattern established by `HookBridge`.
pub struct SessionBridge {
    /// Reference to the session manager
    session_manager: Arc<SessionManager>,
}

impl SessionBridge {
    /// Create a new session bridge
    #[must_use]
    pub const fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Create a new session
    ///
    /// # Errors
    ///
    /// Returns an error if session creation fails
    pub async fn create_session(&self, options: CreateSessionOptions) -> Result<SessionId> {
        convert_err!(self.session_manager.create_session(options).await)
    }

    /// Get an existing session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    pub async fn get_session(&self, session_id: &SessionId) -> Result<Session> {
        convert_err!(self.session_manager.get_session(session_id).await)
    }

    /// List sessions with optional filtering
    ///
    /// # Errors
    ///
    /// Returns an error if listing sessions fails
    pub async fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionMetadata>> {
        convert_err!(self.session_manager.list_sessions(query).await)
    }

    /// Suspend a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be suspended
    pub async fn suspend_session(&self, session_id: &SessionId) -> Result<()> {
        convert_err!(self.session_manager.suspend_session(session_id).await)
    }

    /// Resume a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be resumed
    pub async fn resume_session(&self, session_id: &SessionId) -> Result<()> {
        convert_err!(self.session_manager.resume_session(session_id).await)
    }

    /// Complete a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be completed
    pub async fn complete_session(&self, session_id: &SessionId) -> Result<()> {
        convert_err!(self.session_manager.complete_session(session_id).await)
    }

    /// Delete a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be deleted
    pub async fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        convert_err!(self.session_manager.delete_session(session_id).await)
    }

    /// Save a session to storage
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be saved
    pub async fn save_session(&self, session: &Session) -> Result<()> {
        convert_err!(self.session_manager.save_session(session).await)
    }

    /// Load a session from storage
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be loaded
    pub async fn load_session(&self, session_id: &SessionId) -> Result<Session> {
        convert_err!(self.session_manager.load_session(session_id).await)
    }

    /// Save all active sessions
    ///
    /// # Errors
    ///
    /// Returns an error if any session cannot be saved
    pub async fn save_all_sessions(&self) -> Result<()> {
        convert_err!(self.session_manager.save_all_active_sessions().await)
    }

    /// Restore recent sessions
    ///
    /// # Errors
    ///
    /// Returns an error if sessions cannot be restored
    pub async fn restore_recent_sessions(&self, count: usize) -> Result<Vec<SessionId>> {
        convert_err!(self.session_manager.restore_recent_sessions(count).await)
    }

    /// Check if a session can be replayed
    ///
    /// # Errors
    ///
    /// Returns an error if the session status cannot be checked
    pub async fn can_replay_session(&self, session_id: &SessionId) -> Result<bool> {
        convert_err!(self.session_manager.can_replay_session(session_id).await)
    }

    /// Replay a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be replayed
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        config: llmspell_kernel::sessions::replay::session_adapter::SessionReplayConfig,
    ) -> Result<serde_json::Value> {
        let result = convert_err!(
            self.session_manager
                .replay_session(session_id, config)
                .await
        )?;

        Ok(serde_json::json!({
            "session_id": result.session_id.to_string(),
            "correlation_id": result.correlation_id.to_string(),
            "start_time": chrono::DateTime::<chrono::Utc>::from(result.start_time).to_rfc3339(),
            "total_duration": result.total_duration.as_secs_f64(),
            "hooks_replayed": result.hooks_replayed,
            "successful_replays": result.successful_replays,
            "failed_replays": result.failed_replays,
            "metadata": result.metadata,
        }))
    }

    /// Get session timeline
    ///
    /// # Errors
    ///
    /// Returns an error if timeline retrieval fails
    pub async fn get_session_timeline(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<serde_json::Value>> {
        let timeline = convert_err!(self.session_manager.get_session_timeline(session_id).await)?;

        // Convert timeline events to JSON
        Ok(timeline
            .into_iter()
            .map(|event| {
                serde_json::json!({
                    "hook_id": event.hook_id,
                    "execution_id": event.execution_id,
                    "correlation_id": event.correlation_id,
                    "timestamp": chrono::DateTime::<chrono::Utc>::from(event.timestamp).to_rfc3339(),
                    "result": event.result,
                })
            })
            .collect())
    }

    /// Get session metadata
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session not found
    /// - Metadata access fails
    pub async fn get_session_metadata(&self, session_id: &SessionId) -> Result<serde_json::Value> {
        let session = convert_err!(self.session_manager.get_session(session_id).await)?;
        let metadata = session.metadata.read().await.clone();
        Ok(llmspell_kernel::sessions::bridge::conversions::session_metadata_to_json(&metadata))
    }

    /// Update session metadata
    ///
    /// # Errors
    ///
    /// Returns an error if the session metadata cannot be updated
    pub async fn update_session_metadata(
        &self,
        session_id: &SessionId,
        updates: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Create a SessionOperations instance for extended operations
        let ops = llmspell_kernel::sessions::bridge::operations::SessionOperations::new(
            self.session_manager.clone(),
        );
        convert_err!(ops.update_metadata(session_id, updates).await)
    }

    /// Get session tags
    ///
    /// # Errors
    ///
    /// Returns an error if the session tags cannot be retrieved
    pub async fn get_session_tags(&self, session_id: &SessionId) -> Result<Vec<String>> {
        let ops = llmspell_kernel::sessions::bridge::operations::SessionOperations::new(
            self.session_manager.clone(),
        );
        convert_err!(ops.get_tags(session_id).await)
    }

    /// Set session tags
    ///
    /// # Errors
    ///
    /// Returns an error if the session tags cannot be set
    pub async fn set_session_tags(&self, session_id: &SessionId, tags: Vec<String>) -> Result<()> {
        let ops = llmspell_kernel::sessions::bridge::operations::SessionOperations::new(
            self.session_manager.clone(),
        );
        convert_err!(ops.set_tags(session_id, tags).await)
    }

    /// Get replay metadata for a session
    ///
    /// # Errors
    ///
    /// Returns an error if the replay metadata cannot be retrieved
    pub async fn get_session_replay_metadata(
        &self,
        session_id: &SessionId,
    ) -> Result<serde_json::Value> {
        let metadata = convert_err!(
            self.session_manager
                .get_session_replay_metadata(session_id)
                .await
        )?;

        Ok(serde_json::json!({
            "session_id": metadata.session_id.to_string(),
            "correlation_id": metadata.correlation_id.to_string(),
            "total_hooks": metadata.total_hooks,
            "first_hook_timestamp": metadata.first_hook_timestamp.map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
            "last_hook_timestamp": metadata.last_hook_timestamp.map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
            "total_duration": metadata.total_duration.map(|d| d.as_secs_f64()),
            "can_replay": metadata.can_replay,
        }))
    }

    /// List replayable sessions
    ///
    /// # Errors
    ///
    /// Returns an error if the replayable sessions cannot be listed
    pub async fn list_replayable_sessions(&self) -> Result<Vec<SessionId>> {
        convert_err!(self.session_manager.list_replayable_sessions().await)
    }
}

// Thread-local storage for current session context
thread_local! {
    static CURRENT_SESSION: std::cell::RefCell<Option<SessionId>> = const { std::cell::RefCell::new(None) };
}

/// Session context management
impl SessionBridge {
    /// Get the current session ID
    #[must_use]
    pub fn get_current_session() -> Option<SessionId> {
        CURRENT_SESSION.with(|current| *current.borrow())
    }

    /// Set the current session ID
    pub fn set_current_session(session_id: Option<SessionId>) {
        CURRENT_SESSION.with(|current| {
            *current.borrow_mut() = session_id;
        });
    }

    /// Clear the current session context
    pub fn clear_current_session() {
        Self::set_current_session(None);
    }

    /// Execute a closure with a specific session context
    pub fn with_session_context<F, R>(session_id: SessionId, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let previous = Self::get_current_session();
        Self::set_current_session(Some(session_id));
        let result = f();
        Self::set_current_session(previous);
        result
    }
}
