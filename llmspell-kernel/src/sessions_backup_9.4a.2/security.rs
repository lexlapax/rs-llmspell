//! Session Security
//!
//! Access control, authentication, and security policies for sessions.

use super::SessionId;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, warn};

/// Access control levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    /// Read-only access
    ReadOnly,
    /// Execute permissions
    Execute,
    /// Full control (read, write, execute, delete)
    FullControl,
    /// Administrative access
    Admin,
}

/// Security role
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Role {
    /// Session owner
    Owner,
    /// Collaborator with write access
    Collaborator,
    /// Viewer with read-only access
    Viewer,
    /// System administrator
    Admin,
    /// Custom role
    Custom(String),
}

/// User identity
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct UserId(String);

impl UserId {
    /// Create new user ID
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Get as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Access control entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    /// User ID
    pub user: UserId,
    /// User role
    pub role: Role,
    /// Access level
    pub access: AccessLevel,
    /// Granted permissions
    pub permissions: HashSet<Permission>,
}

/// Permission types
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum Permission {
    /// Read session data
    Read,
    /// Write/modify session
    Write,
    /// Execute code
    Execute,
    /// Delete session
    Delete,
    /// Manage permissions
    ManagePermissions,
    /// View artifacts
    ViewArtifacts,
    /// Create artifacts
    CreateArtifacts,
    /// Delete artifacts
    DeleteArtifacts,
    /// Custom permission
    Custom(String),
}

/// Session security manager
pub struct SessionSecurity {
    /// Access control list (session -> ACL)
    acl: Arc<parking_lot::RwLock<HashMap<SessionId, Vec<AccessControlEntry>>>>,
    /// Role-based permissions
    role_permissions: Arc<parking_lot::RwLock<HashMap<Role, HashSet<Permission>>>>,
    /// Audit log
    audit_log: Arc<parking_lot::RwLock<Vec<AuditEntry>>>,
    /// Security configuration
    config: SecurityConfig,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable access control
    pub enable_acl: bool,
    /// Enable audit logging
    pub enable_audit: bool,
    /// Require authentication
    pub require_auth: bool,
    /// Default access level for new sessions
    pub default_access: AccessLevel,
    /// Maximum audit log entries
    pub max_audit_entries: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_acl: true,
            enable_audit: true,
            require_auth: false,
            default_access: AccessLevel::Execute,
            max_audit_entries: 10000,
        }
    }
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Timestamp
    pub timestamp: std::time::SystemTime,
    /// User who performed the action
    pub user: UserId,
    /// Session affected
    pub session: SessionId,
    /// Action performed
    pub action: AuditAction,
    /// Result of the action
    pub result: AuditResult,
    /// Additional context
    pub context: HashMap<String, String>,
}

/// Audit actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    /// Session created
    SessionCreated,
    /// Session accessed
    SessionAccessed,
    /// Session modified
    SessionModified,
    /// Session deleted
    SessionDeleted,
    /// Permission granted
    PermissionGranted,
    /// Permission revoked
    PermissionRevoked,
    /// Code executed
    CodeExecuted,
    /// Artifact accessed
    ArtifactAccessed,
    /// Security violation attempt
    SecurityViolation,
}

/// Audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditResult {
    /// Action succeeded
    Success,
    /// Action failed
    Failure(String),
    /// Access denied
    Denied,
}

impl Default for SessionSecurity {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionSecurity {
    /// Create new security manager
    pub fn new() -> Self {
        Self::with_config(SecurityConfig::default())
    }

    /// Create with configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        let mut security = Self {
            acl: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            role_permissions: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            audit_log: Arc::new(parking_lot::RwLock::new(Vec::new())),
            config,
        };

        // Initialize default role permissions
        security.init_default_permissions();
        security
    }

    /// Initialize default role permissions
    fn init_default_permissions(&mut self) {
        let mut role_perms = self.role_permissions.write();

        // Owner has all permissions
        role_perms.insert(
            Role::Owner,
            vec![
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::Delete,
                Permission::ManagePermissions,
                Permission::ViewArtifacts,
                Permission::CreateArtifacts,
                Permission::DeleteArtifacts,
            ]
            .into_iter()
            .collect(),
        );

        // Collaborator can read, write, execute
        role_perms.insert(
            Role::Collaborator,
            vec![
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::ViewArtifacts,
                Permission::CreateArtifacts,
            ]
            .into_iter()
            .collect(),
        );

        // Viewer can only read
        role_perms.insert(
            Role::Viewer,
            vec![Permission::Read, Permission::ViewArtifacts]
                .into_iter()
                .collect(),
        );

        // Admin has all permissions
        role_perms.insert(
            Role::Admin,
            vec![
                Permission::Read,
                Permission::Write,
                Permission::Execute,
                Permission::Delete,
                Permission::ManagePermissions,
                Permission::ViewArtifacts,
                Permission::CreateArtifacts,
                Permission::DeleteArtifacts,
            ]
            .into_iter()
            .collect(),
        );
    }

    /// Grant access to a session
    ///
    /// # Errors
    ///
    /// Returns an error if access control is disabled
    pub fn grant_access(
        &self,
        session: &SessionId,
        user: &UserId,
        role: &Role,
        access: AccessLevel,
    ) -> Result<()> {
        if !self.config.enable_acl {
            return Ok(());
        }

        // Get permissions for role
        let permissions = self
            .role_permissions
            .read()
            .get(role)
            .cloned()
            .unwrap_or_default();

        let entry = AccessControlEntry {
            user: user.clone(),
            role: role.clone(),
            access,
            permissions,
        };

        // Add to ACL
        self.acl
            .write()
            .entry(session.clone())
            .or_default()
            .push(entry);

        // Audit log
        self.audit(
            user.clone(),
            session.clone(),
            AuditAction::PermissionGranted,
            AuditResult::Success,
            vec![("role".to_string(), format!("{role:?}"))]
                .into_iter()
                .collect(),
        );

        debug!(
            "Granted {:?} access to user {} for session {}",
            role, user, session
        );
        Ok(())
    }

    /// Revoke access from a session
    ///
    /// # Errors
    ///
    /// Returns an error if access control is disabled
    pub fn revoke_access(&self, session: &SessionId, user: &UserId) -> Result<()> {
        if !self.config.enable_acl {
            return Ok(());
        }

        let mut acl = self.acl.write();
        if let Some(entries) = acl.get_mut(session) {
            entries.retain(|e| &e.user != user);
        }

        // Audit log
        self.audit(
            user.clone(),
            session.clone(),
            AuditAction::PermissionRevoked,
            AuditResult::Success,
            HashMap::new(),
        );

        debug!("Revoked access for user {} from session {}", user, session);
        Ok(())
    }

    /// Check if user has permission
    pub fn has_permission(
        &self,
        session: &SessionId,
        user: &UserId,
        permission: &Permission,
    ) -> bool {
        if !self.config.enable_acl {
            return true; // ACL disabled, allow all
        }

        let acl = self.acl.read();
        if let Some(entries) = acl.get(session) {
            for entry in entries {
                if &entry.user == user && entry.permissions.contains(permission) {
                    return true;
                }
            }
        }

        false
    }

    /// Check access level
    pub fn check_access(&self, session: &SessionId, user: &UserId) -> Option<AccessLevel> {
        if !self.config.enable_acl {
            return Some(self.config.default_access);
        }

        let acl = self.acl.read();
        if let Some(entries) = acl.get(session) {
            for entry in entries {
                if &entry.user == user {
                    return Some(entry.access);
                }
            }
        }

        None
    }

    /// Validate operation
    ///
    /// # Errors
    ///
    /// Returns an error if operation validation fails
    pub fn validate_operation(
        &self,
        session: &SessionId,
        user: &UserId,
        operation: &str,
    ) -> Result<bool> {
        // Map operation to permission
        let permission = match operation {
            "read" => Permission::Read,
            "write" => Permission::Write,
            "execute" => Permission::Execute,
            "delete" => Permission::Delete,
            _ => Permission::Custom(operation.to_string()),
        };

        let allowed = self.has_permission(session, user, &permission);

        // Audit log
        let action = if operation == "execute" {
            AuditAction::CodeExecuted
        } else {
            AuditAction::SessionAccessed
        };

        let result = if allowed {
            AuditResult::Success
        } else {
            AuditResult::Denied
        };

        self.audit(
            user.clone(),
            session.clone(),
            action,
            result.clone(),
            vec![("operation".to_string(), operation.to_string())]
                .into_iter()
                .collect(),
        );

        if !allowed {
            warn!(
                "Access denied: user {} attempted {} on session {}",
                user, operation, session
            );
        }

        Ok(allowed)
    }

    /// Add audit log entry
    fn audit(
        &self,
        user: UserId,
        session: SessionId,
        action: AuditAction,
        result: AuditResult,
        context: HashMap<String, String>,
    ) {
        if !self.config.enable_audit {
            return;
        }

        let entry = AuditEntry {
            timestamp: std::time::SystemTime::now(),
            user,
            session,
            action,
            result,
            context,
        };

        let mut log = self.audit_log.write();
        log.push(entry);

        // Limit log size
        let len = log.len();
        if len > self.config.max_audit_entries {
            log.drain(0..len - self.config.max_audit_entries);
        }
    }

    /// Get audit log for a session
    pub fn get_audit_log(&self, session: &SessionId) -> Vec<AuditEntry> {
        self.audit_log
            .read()
            .iter()
            .filter(|e| &e.session == session)
            .cloned()
            .collect()
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) {
        self.audit_log.write().clear();
    }
}

/// Access control helper
pub struct AccessControl;

impl AccessControl {
    /// Check if access level permits operation
    pub fn can_perform(level: AccessLevel, operation: &str) -> bool {
        match operation {
            "read" => true, // All levels can read
            "execute" => matches!(
                level,
                AccessLevel::Execute | AccessLevel::FullControl | AccessLevel::Admin
            ),
            "write" | "delete" => matches!(level, AccessLevel::FullControl | AccessLevel::Admin),
            "admin" => matches!(level, AccessLevel::Admin),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_access_control() {
        let security = SessionSecurity::new();
        let session = SessionId::new();
        let user1 = UserId::new("user1".to_string());
        let user2 = UserId::new("user2".to_string());

        // Grant owner access to user1
        security
            .grant_access(&session, &user1, &Role::Owner, AccessLevel::FullControl)
            .unwrap();

        // Grant viewer access to user2
        security
            .grant_access(&session, &user2, &Role::Viewer, AccessLevel::ReadOnly)
            .unwrap();

        // Check permissions
        assert!(security.has_permission(&session, &user1, &Permission::Write));
        assert!(!security.has_permission(&session, &user2, &Permission::Write));
        assert!(security.has_permission(&session, &user2, &Permission::Read));
    }

    #[test]
    fn test_operation_validation() {
        let security = SessionSecurity::new();
        let session = SessionId::new();
        let user = UserId::new("user".to_string());

        // Grant execute access
        security
            .grant_access(&session, &user, &Role::Collaborator, AccessLevel::Execute)
            .unwrap();

        // Validate operations
        assert!(security
            .validate_operation(&session, &user, "read")
            .unwrap());
        assert!(security
            .validate_operation(&session, &user, "execute")
            .unwrap());
        assert!(!security
            .validate_operation(&session, &user, "delete")
            .unwrap());
    }

    #[test]
    fn test_audit_logging() {
        let config = SecurityConfig {
            enable_audit: true,
            ..Default::default()
        };

        let security = SessionSecurity::with_config(config);
        let session = SessionId::new();
        let user = UserId::new("user".to_string());

        // Perform operations that generate audit logs
        security
            .grant_access(&session, &user, &Role::Owner, AccessLevel::FullControl)
            .unwrap();
        security
            .validate_operation(&session, &user, "execute")
            .unwrap();

        // Check audit log
        let log = security.get_audit_log(&session);
        assert_eq!(log.len(), 2);
        assert!(matches!(log[0].action, AuditAction::PermissionGranted));
        assert!(matches!(log[1].action, AuditAction::CodeExecuted));
    }

    #[test]
    fn test_access_level_permissions() {
        assert!(AccessControl::can_perform(AccessLevel::ReadOnly, "read"));
        assert!(!AccessControl::can_perform(
            AccessLevel::ReadOnly,
            "execute"
        ));

        assert!(AccessControl::can_perform(AccessLevel::Execute, "read"));
        assert!(AccessControl::can_perform(AccessLevel::Execute, "execute"));
        assert!(!AccessControl::can_perform(AccessLevel::Execute, "delete"));

        assert!(AccessControl::can_perform(AccessLevel::FullControl, "read"));
        assert!(AccessControl::can_perform(
            AccessLevel::FullControl,
            "execute"
        ));
        assert!(AccessControl::can_perform(
            AccessLevel::FullControl,
            "delete"
        ));

        assert!(AccessControl::can_perform(AccessLevel::Admin, "admin"));
    }
}
