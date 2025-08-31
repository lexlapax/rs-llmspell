//! Debug session management for multi-client debugging
//!
//! Following Phase 9.1 architecture using `ExecutionManager` and unified types

use crate::{
    Breakpoint, DebugCommand, DebugState, ExecutionManager, SharedExecutionContext, StackFrame,
    Variable,
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;

/// Debug session manager using Phase 9.1 architecture
pub struct DebugSessionManager {
    /// Active debug sessions
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    /// `ExecutionManager` from `execution_bridge.rs` (not old "Debugger")
    execution_manager: Arc<ExecutionManager>,
    /// Persistent sessions for reconnection (indexed by `client_id`)
    persistent_sessions: Arc<RwLock<HashMap<String, String>>>, // client_id -> session_id
    /// Script locks to prevent conflicting debug sessions
    script_locks: Arc<RwLock<HashMap<std::path::PathBuf, String>>>, // script_path -> session_id
}

/// Individual debug session using unified types
pub struct DebugSession {
    /// Unique session identifier
    pub session_id: String,
    /// Client identifier
    pub client_id: String,
    /// Currently debugging script path
    pub script_path: Option<std::path::PathBuf>,
    /// Current debug state using unified `DebugState` type
    pub debug_state: DebugState,
    /// Current stack frame index
    pub current_frame: usize,
    /// Session-specific breakpoints using unified `Breakpoint` type
    pub breakpoints: Vec<Breakpoint>,
    /// Shared execution context from Phase 9.1
    pub shared_context: SharedExecutionContext,
    /// Watch expressions
    pub watch_expressions: Vec<String>,
    /// Session creation time
    pub created_at: SystemTime,
}

impl DebugSessionManager {
    /// Create new session manager
    #[must_use]
    pub fn new(execution_manager: Arc<ExecutionManager>) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            execution_manager,
            persistent_sessions: Arc::new(RwLock::new(HashMap::new())),
            script_locks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new debug session
    ///
    /// # Errors
    ///
    /// Returns an error if session creation fails
    pub async fn create_session(&self, client_id: String) -> Result<String> {
        // Check if client has a persistent session to reconnect to
        if let Some(existing_session_id) = self.persistent_sessions.read().await.get(&client_id) {
            if self.sessions.read().await.contains_key(existing_session_id) {
                return Ok(existing_session_id.clone());
            }
        }

        let session = DebugSession {
            session_id: uuid::Uuid::new_v4().to_string(),
            client_id: client_id.clone(),
            script_path: None,
            debug_state: DebugState::Terminated, // Use unified DebugState
            current_frame: 0,
            breakpoints: Vec::new(),
            shared_context: SharedExecutionContext::new(), // Initialize shared context
            watch_expressions: Vec::new(),
            created_at: SystemTime::now(),
        };

        let session_id = session.session_id.clone();

        // Store session
        self.sessions
            .write()
            .await
            .insert(session_id.clone(), session);

        // Store persistent mapping for reconnection
        self.persistent_sessions
            .write()
            .await
            .insert(client_id, session_id.clone());

        Ok(session_id)
    }

    /// Handle debug command for a specific session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or command fails
    pub async fn handle_debug_command(
        &self,
        session_id: &str,
        command: DebugCommand,
    ) -> Result<()> {
        {
            let sessions = self.sessions.read().await;
            let _session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
        }

        // Commands now route through ExecutionManager
        match command {
            DebugCommand::StepInto => {
                self.execution_manager
                    .send_command(DebugCommand::StepInto)
                    .await;
                self.update_session_state(session_id).await?;
            }
            DebugCommand::StepOver => {
                self.execution_manager
                    .send_command(DebugCommand::StepOver)
                    .await;
                self.update_session_state(session_id).await?;
            }
            DebugCommand::StepOut => {
                self.execution_manager
                    .send_command(DebugCommand::StepOut)
                    .await;
                self.update_session_state(session_id).await?;
            }
            DebugCommand::Continue => {
                self.execution_manager
                    .send_command(DebugCommand::Continue)
                    .await;
                self.update_session_state(session_id).await?;
            }
            DebugCommand::Pause => {
                self.execution_manager
                    .send_command(DebugCommand::Pause)
                    .await;
                self.update_session_state(session_id).await?;
            }
            DebugCommand::Terminate => {
                self.execution_manager
                    .send_command(DebugCommand::Terminate)
                    .await;
                self.update_session_state(session_id).await?;
            }
        }

        Ok(())
    }

    /// Add a breakpoint to a session and `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if adding the breakpoint fails
    pub async fn add_session_breakpoint(
        &self,
        session_id: &str,
        breakpoint: Breakpoint,
    ) -> Result<String> {
        // Add to ExecutionManager
        let bp_id = self
            .execution_manager
            .add_breakpoint(breakpoint.clone())
            .await;

        // Add to session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.breakpoints.push(breakpoint);
            }
        }

        Ok(bp_id)
    }

    /// Remove a breakpoint from session and `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if removing the breakpoint fails
    pub async fn remove_session_breakpoint(
        &self,
        session_id: &str,
        breakpoint_id: &str,
    ) -> Result<bool> {
        // Remove from ExecutionManager
        let removed = self
            .execution_manager
            .remove_breakpoint(breakpoint_id)
            .await;

        // Remove from session
        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.breakpoints.retain(|bp| bp.id != breakpoint_id);
            }
        }

        Ok(removed)
    }

    /// Get session variables via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    pub async fn get_session_variables(
        &self,
        session_id: &str,
        frame_id: Option<&str>,
    ) -> Result<Vec<Variable>> {
        {
            let sessions = self.sessions.read().await;
            let _session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
        }

        // Get variables via ExecutionManager
        let variables = self.execution_manager.get_variables(frame_id).await;
        Ok(variables)
    }

    /// Get session stack trace via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    pub async fn get_session_stack_trace(&self, session_id: &str) -> Result<Vec<StackFrame>> {
        {
            let sessions = self.sessions.read().await;
            let _session = sessions
                .get(session_id)
                .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;
        }

        // Get stack trace via ExecutionManager
        let stack = self.execution_manager.get_stack_trace().await;
        Ok(stack)
    }

    /// Update session state from `ExecutionManager`
    async fn update_session_state(&self, session_id: &str) -> Result<()> {
        let current_state = self.execution_manager.get_state().await;

        {
            let mut sessions = self.sessions.write().await;
            if let Some(session) = sessions.get_mut(session_id) {
                session.debug_state = current_state;
            }
        }

        Ok(())
    }

    /// Get session by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    pub async fn get_session(&self, session_id: &str) -> Result<DebugSession> {
        let sessions = self.sessions.read().await;
        sessions
            .get(session_id)
            .cloned()
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))
    }

    /// Remove session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be removed
    pub async fn remove_session(&self, session_id: &str) -> Result<bool> {
        let mut sessions = self.sessions.write().await;

        // Clean up script locks
        if let Some(session) = sessions.get(session_id) {
            if let Some(script_path) = &session.script_path {
                self.script_locks.write().await.remove(script_path);
            }

            // Clean up persistent session mapping
            self.persistent_sessions
                .write()
                .await
                .retain(|_, sid| sid != session_id);
        }

        Ok(sessions.remove(session_id).is_some())
    }

    /// List active sessions
    pub async fn list_sessions(&self) -> Vec<String> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Set script path for a session with conflict resolution
    ///
    /// # Errors
    ///
    /// Returns an error if another session is already debugging this script
    pub async fn set_session_script(
        &self,
        session_id: &str,
        script_path: std::path::PathBuf,
    ) -> Result<()> {
        // Check for conflicts
        let script_locks = self.script_locks.read().await;
        if let Some(existing_session_id) = script_locks.get(&script_path) {
            if existing_session_id != session_id {
                return Err(anyhow!(
                    "Script {:?} is already being debugged by session {}",
                    script_path,
                    existing_session_id
                ));
            }
        }
        drop(script_locks);

        // Update session
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| anyhow!("Session not found: {}", session_id))?;

        // Clean up old script lock if changing scripts
        if let Some(old_path) = &session.script_path {
            if old_path != &script_path {
                self.script_locks.write().await.remove(old_path);
            }
        }

        session.script_path = Some(script_path.clone());
        drop(sessions);

        // Set new script lock
        self.script_locks
            .write()
            .await
            .insert(script_path, session_id.to_string());

        Ok(())
    }

    /// Reconnect to an existing session
    ///
    /// # Errors
    ///
    /// Returns an error if no session exists for the client
    pub async fn reconnect_session(&self, client_id: &str) -> Result<String> {
        let session_id = {
            let persistent_sessions = self.persistent_sessions.read().await;
            persistent_sessions
                .get(client_id)
                .ok_or_else(|| anyhow!("No persistent session found for client: {}", client_id))?
                .clone()
        };

        // Verify session still exists
        if !self.sessions.read().await.contains_key(&session_id) {
            return Err(anyhow!("Session {} no longer exists", session_id));
        }

        Ok(session_id)
    }

    /// Check if a script is being debugged
    pub async fn is_script_locked(&self, script_path: &std::path::Path) -> bool {
        self.script_locks.read().await.contains_key(script_path)
    }

    /// Get the session ID debugging a specific script
    pub async fn get_script_session(&self, script_path: &std::path::Path) -> Option<String> {
        self.script_locks.read().await.get(script_path).cloned()
    }

    /// Clean up expired sessions (older than 1 hour with no activity)
    ///
    /// # Errors
    ///
    /// Returns an error if time calculation fails
    pub async fn cleanup_expired_sessions(&self) -> Result<usize> {
        let cutoff = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            - 3600; // 1 hour ago

        let mut sessions = self.sessions.write().await;
        let initial_count = sessions.len();

        sessions.retain(|_id, session| {
            session
                .created_at
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|d| d.as_secs() > cutoff)
                .unwrap_or(false)
        });

        Ok(initial_count - sessions.len())
    }
}

impl Clone for DebugSession {
    fn clone(&self) -> Self {
        Self {
            session_id: self.session_id.clone(),
            client_id: self.client_id.clone(),
            script_path: self.script_path.clone(),
            debug_state: self.debug_state.clone(),
            current_frame: self.current_frame,
            breakpoints: self.breakpoints.clone(),
            shared_context: self.shared_context.clone(),
            watch_expressions: self.watch_expressions.clone(),
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_bridge::lua::debug_state_cache_impl::LuaDebugStateCache;

    #[tokio::test]
    async fn test_session_creation() {
        let execution_manager =
            Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
        let session_manager = DebugSessionManager::new(execution_manager);

        let session_id = session_manager
            .create_session("client1".to_string())
            .await
            .unwrap();
        assert!(!session_id.is_empty());

        let session = session_manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.client_id, "client1");
        assert_eq!(session.debug_state, DebugState::Terminated);
    }

    #[tokio::test]
    async fn test_breakpoint_management() {
        let execution_manager =
            Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
        let session_manager = DebugSessionManager::new(execution_manager);

        let session_id = session_manager
            .create_session("client1".to_string())
            .await
            .unwrap();

        let breakpoint = Breakpoint::new("test.lua".to_string(), 10);
        let bp_id = session_manager
            .add_session_breakpoint(&session_id, breakpoint)
            .await
            .unwrap();
        assert!(!bp_id.is_empty());

        let removed = session_manager
            .remove_session_breakpoint(&session_id, &bp_id)
            .await
            .unwrap();
        assert!(removed);
    }

    #[tokio::test]
    async fn test_debug_commands() {
        let execution_manager =
            Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
        let session_manager = DebugSessionManager::new(execution_manager);

        let session_id = session_manager
            .create_session("client1".to_string())
            .await
            .unwrap();

        // Test various debug commands
        session_manager
            .handle_debug_command(&session_id, DebugCommand::Continue)
            .await
            .unwrap();
        session_manager
            .handle_debug_command(&session_id, DebugCommand::StepInto)
            .await
            .unwrap();
        session_manager
            .handle_debug_command(&session_id, DebugCommand::Pause)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_session_cleanup() {
        let execution_manager =
            Arc::new(ExecutionManager::new(Arc::new(LuaDebugStateCache::new())));
        let session_manager = DebugSessionManager::new(execution_manager);

        let session_id = session_manager
            .create_session("client1".to_string())
            .await
            .unwrap();
        assert_eq!(session_manager.list_sessions().await.len(), 1);

        let removed = session_manager.remove_session(&session_id).await.unwrap();
        assert!(removed);
        assert_eq!(session_manager.list_sessions().await.len(), 0);
    }
}
