//! ABOUTME: Core session bridge providing language-agnostic session operations
//! ABOUTME: Wraps SessionManager for script access with async-to-sync conversion

use llmspell_core::{error::LLMSpellError, Result};
use llmspell_sessions::{
    manager::SessionManager,
    session::Session,
    types::{CreateSessionOptions, SessionQuery},
    SessionId, SessionMetadata,
};
use std::sync::Arc;

/// Convert mlua error to LLMSpellError
fn lua_to_llmspell_error(e: mlua::Error) -> LLMSpellError {
    LLMSpellError::Script {
        message: e.to_string(),
        language: Some("lua".to_string()),
        line: None,
        source: None,
    }
}

/// Core session bridge for language-agnostic session operations
///
/// This bridge wraps the SessionManager and provides synchronous interfaces
/// for script languages, following the pattern established by HookBridge.
pub struct SessionBridge {
    /// Reference to the session manager
    session_manager: Arc<SessionManager>,
}

impl SessionBridge {
    /// Create a new session bridge
    pub fn new(session_manager: Arc<SessionManager>) -> Self {
        Self { session_manager }
    }

    /// Create a new session
    pub fn create_session(&self, options: CreateSessionOptions) -> Result<SessionId> {
        // Use block_on pattern from HookBridge
        crate::lua::sync_utils::block_on_async(
            "session_create",
            async move { self.session_manager.create_session(options).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get an existing session
    pub fn get_session(&self, session_id: &SessionId) -> Result<Session> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_get",
            async move { manager.get_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// List sessions with optional filtering
    pub fn list_sessions(&self, query: SessionQuery) -> Result<Vec<SessionMetadata>> {
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_list",
            async move { manager.list_sessions(query).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Suspend a session
    pub fn suspend_session(&self, session_id: &SessionId) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_suspend",
            async move { manager.suspend_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Resume a session
    pub fn resume_session(&self, session_id: &SessionId) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_resume",
            async move { manager.resume_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Complete a session
    pub fn complete_session(&self, session_id: &SessionId) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_complete",
            async move { manager.complete_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &SessionId) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_delete",
            async move { manager.delete_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Save a session to storage
    pub fn save_session(&self, session: &Session) -> Result<()> {
        let session = session.clone();
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_save",
            async move { manager.save_session(&session).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Load a session from storage
    pub fn load_session(&self, session_id: &SessionId) -> Result<Session> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_load",
            async move { manager.load_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Save all active sessions
    pub fn save_all_sessions(&self) -> Result<()> {
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_save_all",
            async move { manager.save_all_active_sessions().await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Restore recent sessions
    pub fn restore_recent_sessions(&self, count: usize) -> Result<Vec<SessionId>> {
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_restore_recent",
            async move { manager.restore_recent_sessions(count).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Check if a session can be replayed
    pub fn can_replay_session(&self, session_id: &SessionId) -> Result<bool> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_can_replay",
            async move { manager.can_replay_session(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Replay a session
    pub fn replay_session(
        &self,
        session_id: &SessionId,
        _options: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async::<_, serde_json::Value, LLMSpellError>(
            "session_replay",
            async move {
                // For now, use default config - full implementation in Task 6.5.6
                let config = llmspell_sessions::replay::session_adapter::SessionReplayConfig::default();
                let result = manager.replay_session(&session_id, config).await
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Session error: {}", e),
                        source: None,
                    })?;
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
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get session timeline
    pub fn get_session_timeline(&self, session_id: &SessionId) -> Result<Vec<serde_json::Value>> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async::<_, Vec<serde_json::Value>, LLMSpellError>(
            "session_get_timeline",
            async move {
                let timeline = manager.get_session_timeline(&session_id).await
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Session error: {}", e),
                        source: None,
                    })?;
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
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get session metadata
    pub fn get_session_metadata(&self, session_id: &SessionId) -> Result<serde_json::Value> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async::<_, serde_json::Value, LLMSpellError>(
            "session_get_metadata",
            async move {
                let session = manager.get_session(&session_id).await.map_err(|e| {
                    LLMSpellError::Component {
                        message: format!("Session error: {}", e),
                        source: None,
                    }
                })?;
                let metadata = session.metadata.read().await.clone();
                Ok(llmspell_sessions::bridge::conversions::session_metadata_to_json(&metadata))
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Update session metadata
    pub fn update_session_metadata(
        &self,
        session_id: &SessionId,
        updates: std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();

        // Create a SessionOperations instance for extended operations
        let ops = llmspell_sessions::bridge::operations::SessionOperations::new(manager.clone());

        crate::lua::sync_utils::block_on_async(
            "session_update_metadata",
            async move { ops.update_metadata(&session_id, updates).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get session tags
    pub fn get_session_tags(&self, session_id: &SessionId) -> Result<Vec<String>> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        let ops = llmspell_sessions::bridge::operations::SessionOperations::new(manager);

        crate::lua::sync_utils::block_on_async(
            "session_get_tags",
            async move { ops.get_tags(&session_id).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Set session tags
    pub fn set_session_tags(&self, session_id: &SessionId, tags: Vec<String>) -> Result<()> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        let ops = llmspell_sessions::bridge::operations::SessionOperations::new(manager);

        crate::lua::sync_utils::block_on_async(
            "session_set_tags",
            async move { ops.set_tags(&session_id, tags).await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// Get replay metadata for a session
    pub fn get_session_replay_metadata(&self, session_id: &SessionId) -> Result<serde_json::Value> {
        let session_id = *session_id;
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async::<_, serde_json::Value, LLMSpellError>(
            "session_get_replay_metadata",
            async move {
                let metadata = manager.get_session_replay_metadata(&session_id).await
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Session error: {}", e),
                        source: None,
                    })?;
                Ok(serde_json::json!({
                    "session_id": metadata.session_id.to_string(),
                    "correlation_id": metadata.correlation_id.to_string(),
                    "total_hooks": metadata.total_hooks,
                    "first_hook_timestamp": metadata.first_hook_timestamp.map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
                    "last_hook_timestamp": metadata.last_hook_timestamp.map(|t| chrono::DateTime::<chrono::Utc>::from(t).to_rfc3339()),
                    "total_duration": metadata.total_duration.map(|d| d.as_secs_f64()),
                    "can_replay": metadata.can_replay,
                }))
            },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }

    /// List replayable sessions
    pub fn list_replayable_sessions(&self) -> Result<Vec<SessionId>> {
        let manager = self.session_manager.clone();
        crate::lua::sync_utils::block_on_async(
            "session_list_replayable",
            async move { manager.list_replayable_sessions().await },
            None,
        )
        .map_err(lua_to_llmspell_error)
    }
}

// Thread-local storage for current session context
thread_local! {
    static CURRENT_SESSION: std::cell::RefCell<Option<SessionId>> = const { std::cell::RefCell::new(None) };
}

/// Session context management
impl SessionBridge {
    /// Get the current session ID
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
