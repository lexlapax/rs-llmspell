//! Session Manager - Core lifecycle management
//!
//! Manages session creation, persistence, restoration, and lifecycle transitions.
//! Integrates with kernel state management and message routing.

use super::{
    KernelSessionIntegration, Session, SessionConfig, SessionEvent, SessionEventType, SessionId,
    SessionMetrics, SessionSecurity, SessionStatus,
};
use crate::state::KernelState;
use anyhow::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, instrument, warn};

/// Type alias for session event handlers
type SessionEventHandler = Box<dyn Fn(&SessionEvent) + Send + Sync>;

/// Session manager for kernel
pub struct SessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    /// Session configuration
    config: SessionConfig,
    /// Session security manager
    #[allow(dead_code)] // Will be used in future phases
    security: Arc<SessionSecurity>,
    /// Integration with kernel state
    kernel_state: Option<Arc<KernelState>>,
    /// Session event handlers
    event_handlers: Vec<SessionEventHandler>,
    /// Cleanup task handle
    cleanup_handle: Option<tokio::task::JoinHandle<()>>,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(config: SessionConfig) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            config,
            security: Arc::new(SessionSecurity::new()),
            kernel_state: None,
            event_handlers: Vec::new(),
            cleanup_handle: None,
        }
    }

    /// Set kernel state for integration
    pub fn set_kernel_state(&mut self, state: Arc<KernelState>) {
        self.kernel_state = Some(state);
    }

    /// Create a new session
    ///
    /// # Errors
    ///
    /// Returns an error if session creation or state initialization fails
    #[instrument(level = "info", skip(self))]
    pub fn create_session(&mut self, owner: Option<String>) -> Result<SessionId> {
        let session_id = SessionId::new();
        info!("Creating session: {}", session_id);

        let mut session = Session::new(session_id.clone(), self.config.clone());

        // Set owner for multi-tenant isolation
        if let Some(owner) = owner {
            session.metadata.set_owner(owner);
        }

        // Apply default policies
        self.apply_default_policies(&mut session);

        // Store session
        self.sessions.write().insert(session_id.clone(), session);

        // Update kernel state if connected
        if let Some(kernel_state) = &self.kernel_state {
            kernel_state.update_session(|state| {
                state.set_id(session_id.as_str());
                Ok(())
            })?;
        }

        // Emit creation event
        self.emit_event(&SessionEvent::new(
            session_id.clone(),
            SessionEventType::Created,
        ));

        Ok(session_id)
    }

    /// Get a session by ID
    pub fn get_session(&self, id: &SessionId) -> Option<Session> {
        self.sessions.read().get(id).cloned()
    }

    /// Update a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or the updater function fails
    pub fn update_session<F>(&self, id: &SessionId, updater: F) -> Result<()>
    where
        F: FnOnce(&mut Session) -> Result<()>,
    {
        let mut sessions = self.sessions.write();
        let session = sessions
            .get_mut(id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        updater(session)?;

        // Update activity time
        session.metadata.touch();

        Ok(())
    }

    /// Pause a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or kernel state update fails
    #[instrument(level = "debug", skip(self))]
    pub fn pause_session(&self, id: &SessionId) -> Result<()> {
        self.update_session(id, |session| {
            session.pause();
            Ok(())
        })?;

        // Update kernel state
        if let Some(kernel_state) = &self.kernel_state {
            kernel_state.update_session(|state| {
                state.pause();
                Ok(())
            })?;
        }

        // Emit pause event
        self.emit_event(&SessionEvent::new(id.clone(), SessionEventType::Paused));

        Ok(())
    }

    /// Resume a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found or kernel state update fails
    #[instrument(level = "debug", skip(self))]
    pub fn resume_session(&self, id: &SessionId) -> Result<()> {
        self.update_session(id, |session| {
            session.resume();
            Ok(())
        })?;

        // Update kernel state
        if let Some(kernel_state) = &self.kernel_state {
            kernel_state.update_session(|state| {
                state.resume();
                Ok(())
            })?;
        }

        // Emit resume event
        self.emit_event(&SessionEvent::new(id.clone(), SessionEventType::Resumed));

        Ok(())
    }

    /// Archive a session
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    #[instrument(level = "info", skip(self))]
    pub fn archive_session(&self, id: &SessionId) -> Result<()> {
        self.update_session(id, |session| {
            session.archive();
            Ok(())
        })?;

        // Emit archive event
        self.emit_event(&SessionEvent::new(id.clone(), SessionEventType::Archived));

        info!("Session archived: {}", id);
        Ok(())
    }

    /// Start TTL cleanup task
    pub fn start_ttl_cleanup(&mut self) {
        let sessions = self.sessions.clone();
        let check_interval = Duration::from_secs(60); // Check every minute

        let handle = tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                let mut expired = Vec::new();
                {
                    let sessions_read = sessions.read();
                    for (id, session) in sessions_read.iter() {
                        if session.metadata.is_expired() {
                            expired.push(id.clone());
                        }
                    }
                }

                if !expired.is_empty() {
                    let mut sessions_write = sessions.write();
                    for id in expired {
                        if let Some(mut session) = sessions_write.remove(&id) {
                            session.metadata.status = SessionStatus::Expired;
                            info!("Session expired and removed: {}", id);
                        }
                    }
                }
            }
        });

        self.cleanup_handle = Some(handle);
    }

    /// Stop TTL cleanup task
    pub fn stop_ttl_cleanup(&mut self) {
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }

    /// Apply default policies to a session
    fn apply_default_policies(&self, session: &mut Session) {
        // Add rate limiting policy if configured
        if let Some(rate_config) = &self.config.rate_limit {
            let rate_policy =
                super::policies::RateLimitPolicy::new(rate_config.max_requests, rate_config.window);
            session.policies.push(Arc::new(rate_policy));
        }

        // Add timeout policy if configured
        if let Some(ttl) = self.config.session_ttl {
            let timeout_policy = super::policies::TimeoutPolicy::new(ttl);
            session.policies.push(Arc::new(timeout_policy));
        }
    }

    /// Emit a session event
    fn emit_event(&self, event: &SessionEvent) {
        for handler in &self.event_handlers {
            handler(event);
        }
    }

    /// Register an event handler
    pub fn register_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(&SessionEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Get all active sessions
    pub fn active_sessions(&self) -> Vec<SessionId> {
        self.sessions
            .read()
            .iter()
            .filter(|(_, session)| session.is_active())
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get session metrics
    pub fn get_metrics(&self, id: &SessionId) -> Option<SessionMetrics> {
        self.sessions.read().get(id).map(|s| s.metrics.clone())
    }

    /// Validate access to session (multi-tenant isolation)
    ///
    /// # Errors
    ///
    /// Returns an error if the session is not found
    pub fn validate_access(&self, id: &SessionId, user: &str) -> Result<bool> {
        let sessions = self.sessions.read();
        let session = sessions
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        // Check owner
        if let Some(owner) = &session.metadata.owner {
            if owner != user {
                warn!("Access denied for user {} to session {}", user, id);
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl KernelSessionIntegration for SessionManager {
    fn handle_kernel_message(&mut self, msg: serde_json::Value) -> Result<()> {
        // Extract session ID from message
        let session_id = msg
            .get("header")
            .and_then(|h| h.get("session"))
            .and_then(|s| s.as_str())
            .map(|s| SessionId::from_string(s.to_string()));

        if let Some(id) = session_id {
            // Update session metrics
            self.update_session(&id, |session| {
                session.update_metrics(|metrics| {
                    metrics.messages_processed += 1;
                });
                session.metadata.touch();
                Ok(())
            })?;

            // Apply policies
            if !self.apply_policies(&msg)? {
                return Err(anyhow::anyhow!("Message rejected by policy"));
            }

            // Track message
            self.track_message(&msg);
        }

        Ok(())
    }

    fn apply_policies(&self, msg: &serde_json::Value) -> Result<bool> {
        // Extract session ID
        let session_id = msg
            .get("header")
            .and_then(|h| h.get("session"))
            .and_then(|s| s.as_str())
            .map(|s| SessionId::from_string(s.to_string()));

        if let Some(id) = session_id {
            let sessions = self.sessions.read();
            if let Some(session) = sessions.get(&id) {
                for policy in &session.policies {
                    if !policy.check(msg)? {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    fn track_message(&mut self, msg: &serde_json::Value) {
        // Extract message type and session ID for tracking
        let msg_type = msg
            .get("header")
            .and_then(|h| h.get("msg_type"))
            .and_then(|t| t.as_str())
            .unwrap_or("unknown");

        let session_id = msg
            .get("header")
            .and_then(|h| h.get("session"))
            .and_then(|s| s.as_str())
            .map(|s| SessionId::from_string(s.to_string()));

        if let Some(id) = session_id {
            debug!("Tracking message {} for session {}", msg_type, id);

            // Emit message event
            self.emit_event(
                &SessionEvent::new(id, SessionEventType::MessageReceived)
                    .with_metadata("msg_type".to_string(), msg_type.to_string()),
            );
        }
    }
}

impl Drop for SessionManager {
    fn drop(&mut self) {
        self.stop_ttl_cleanup();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new(SessionConfig::default());
        assert_eq!(manager.active_sessions().len(), 0);
    }

    #[test]
    fn test_session_lifecycle_management() {
        let mut manager = SessionManager::new(SessionConfig::default());

        // Create session
        let session_id = manager.create_session(Some("user1".to_string())).unwrap();
        assert_eq!(manager.active_sessions().len(), 1);

        // Pause session
        manager.pause_session(&session_id).unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.metadata.status, SessionStatus::Paused);

        // Resume session
        manager.resume_session(&session_id).unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.metadata.status, SessionStatus::Active);

        // Archive session
        manager.archive_session(&session_id).unwrap();
        let session = manager.get_session(&session_id).unwrap();
        assert_eq!(session.metadata.status, SessionStatus::Archived);
    }

    #[test]
    fn test_multi_tenant_isolation() {
        let mut manager = SessionManager::new(SessionConfig::default());

        // Create session for user1
        let session_id = manager.create_session(Some("user1".to_string())).unwrap();

        // user1 should have access
        assert!(manager.validate_access(&session_id, "user1").unwrap());

        // user2 should not have access
        assert!(!manager.validate_access(&session_id, "user2").unwrap());
    }
}
