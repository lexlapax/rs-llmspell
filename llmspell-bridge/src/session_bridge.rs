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

    // TODO: Add replay operations, timeline access, etc. in subsequent tasks
}

// TODO: Add thread-local session context management in Task 6.5.5
