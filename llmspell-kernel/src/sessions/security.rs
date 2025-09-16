//! ABOUTME: Session security and isolation enforcement
//! ABOUTME: Provides session boundary validation and access control

use crate::sessions::{Result, SessionError, SessionId};
use std::collections::HashSet;
use tracing::warn;

/// Session security policy enforcer
#[derive(Debug, Clone)]
pub struct SessionSecurityManager {
    /// Currently active sessions that can be accessed
    active_sessions: HashSet<SessionId>,
    /// Whether to enforce strict isolation (default: true)
    strict_isolation: bool,
}

impl SessionSecurityManager {
    /// Create a new security manager
    pub fn new(strict_isolation: bool) -> Self {
        Self {
            active_sessions: HashSet::new(),
            strict_isolation,
        }
    }

    /// Register an active session
    pub fn register_session(&mut self, session_id: &SessionId) {
        self.active_sessions.insert(*session_id);
    }

    /// Unregister a session (when completed or deleted)
    pub fn unregister_session(&mut self, session_id: &SessionId) {
        self.active_sessions.remove(session_id);
    }

    /// Check if a session can access another session's resources
    pub fn can_access_session(
        &self,
        requesting_session: &SessionId,
        target_session: &SessionId,
    ) -> Result<bool> {
        // Same session can always access its own resources
        if requesting_session == target_session {
            return Ok(true);
        }

        // In strict isolation mode, cross-session access is denied
        if self.strict_isolation {
            warn!(
                "Session {} attempted to access session {} resources - denied by strict isolation",
                requesting_session, target_session
            );
            return Ok(false);
        }

        // In non-strict mode, allow access to active sessions only
        if !self.active_sessions.contains(target_session) {
            warn!(
                "Session {} attempted to access inactive session {} - denied",
                requesting_session, target_session
            );
            return Ok(false);
        }

        Ok(true)
    }

    /// Validate cross-session operation
    pub fn validate_cross_session_access(
        &self,
        requesting_session: &SessionId,
        target_session: &SessionId,
        operation: &str,
    ) -> Result<()> {
        if !self.can_access_session(requesting_session, target_session)? {
            return Err(SessionError::AccessDenied {
                message: format!(
                    "Session {} cannot perform '{}' on session {} resources due to isolation policy",
                    requesting_session, operation, target_session
                ),
            });
        }
        Ok(())
    }

    /// Validate state scope access
    pub fn validate_state_scope_access(
        &self,
        requesting_session: Option<&SessionId>,
        state_scope: &str,
    ) -> Result<()> {
        // Parse session ID from state scope (format: "session:session_id")
        if let Some(scope_session_id) = state_scope.strip_prefix("session:") {
            if let Ok(target_session) = scope_session_id.parse::<SessionId>() {
                if let Some(req_session) = requesting_session {
                    self.validate_cross_session_access(
                        req_session,
                        &target_session,
                        "state_access",
                    )?;
                } else {
                    // No session context - only allow if not strict
                    if self.strict_isolation {
                        return Err(SessionError::AccessDenied {
                            message: format!(
                                "Cannot access session state '{}' without session context",
                                state_scope
                            ),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Get list of active sessions
    pub fn active_sessions(&self) -> Vec<SessionId> {
        self.active_sessions.iter().copied().collect()
    }

    /// Check if session is active
    pub fn is_session_active(&self, session_id: &SessionId) -> bool {
        self.active_sessions.contains(session_id)
    }
}

impl Default for SessionSecurityManager {
    fn default() -> Self {
        Self::new(true) // Default to strict isolation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_same_session_access() {
        let manager = SessionSecurityManager::new(true);
        let session_id = SessionId::new();

        assert!(manager
            .can_access_session(&session_id, &session_id)
            .unwrap());
    }
    #[test]
    fn test_strict_isolation() {
        let mut manager = SessionSecurityManager::new(true);
        let session1 = SessionId::new();
        let session2 = SessionId::new();

        manager.register_session(&session1);
        manager.register_session(&session2);

        // Cross-session access should be denied in strict mode
        assert!(!manager.can_access_session(&session1, &session2).unwrap());
    }
    #[test]
    fn test_non_strict_isolation() {
        let mut manager = SessionSecurityManager::new(false);
        let session1 = SessionId::new();
        let session2 = SessionId::new();

        manager.register_session(&session1);
        manager.register_session(&session2);

        // Cross-session access should be allowed in non-strict mode for active sessions
        assert!(manager.can_access_session(&session1, &session2).unwrap());
    }
    #[test]
    fn test_inactive_session_access() {
        let mut manager = SessionSecurityManager::new(false);
        let session1 = SessionId::new();
        let session2 = SessionId::new();

        manager.register_session(&session1);
        // session2 is not registered (inactive)

        // Access to inactive session should be denied even in non-strict mode
        assert!(!manager.can_access_session(&session1, &session2).unwrap());
    }
    #[test]
    fn test_state_scope_validation() {
        let mut manager = SessionSecurityManager::new(true);
        let session1 = SessionId::new();
        let session2 = SessionId::new();

        manager.register_session(&session1);
        manager.register_session(&session2);

        let scope = format!("session:{}", session2);

        // Should fail in strict isolation
        assert!(manager
            .validate_state_scope_access(Some(&session1), &scope)
            .is_err());

        // Should succeed for same session
        let own_scope = format!("session:{}", session1);
        assert!(manager
            .validate_state_scope_access(Some(&session1), &own_scope)
            .is_ok());
    }
}
