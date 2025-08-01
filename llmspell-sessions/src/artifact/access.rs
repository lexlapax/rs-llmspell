//! ABOUTME: Artifact access control system providing session-based isolation and permissions
//! ABOUTME: Implements read/write permissions, audit logging, and cross-session sharing controls

use super::types::{ArtifactId, ArtifactType};
use crate::{Result, SessionError, SessionId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Permission level for artifact access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Permission {
    /// Read access only
    Read,
    /// Read and write access
    Write,
    /// Full control including sharing and deletion
    Admin,
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Permission::Read => write!(f, "read"),
            Permission::Write => write!(f, "write"),
            Permission::Admin => write!(f, "admin"),
        }
    }
}

/// Access control entry for an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlEntry {
    /// Session that has access
    pub session_id: SessionId,
    /// Permission level granted
    pub permission: Permission,
    /// When this permission was granted
    pub granted_at: DateTime<Utc>,
    /// Optional expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Who granted this permission (for audit trail)
    pub granted_by: Option<SessionId>,
}

impl AccessControlEntry {
    /// Create a new access control entry
    pub fn new(
        session_id: SessionId,
        permission: Permission,
        granted_by: Option<SessionId>,
    ) -> Self {
        Self {
            session_id,
            permission,
            granted_at: Utc::now(),
            expires_at: None,
            granted_by,
        }
    }

    /// Check if this access is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Check if this access allows the given permission
    pub fn allows(&self, required_permission: Permission) -> bool {
        if self.is_expired() {
            return false;
        }

        matches!(
            (self.permission, required_permission),
            (Permission::Admin, _)
                | (Permission::Write, Permission::Read | Permission::Write)
                | (Permission::Read, Permission::Read)
        )
    }
}

/// Access control list for an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlList {
    /// Owner session (has implicit admin access)
    pub owner: SessionId,
    /// Additional access entries
    pub entries: Vec<AccessControlEntry>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
}

impl AccessControlList {
    /// Create a new ACL with the given owner
    pub fn new(owner: SessionId) -> Self {
        let now = Utc::now();
        Self {
            owner,
            entries: Vec::new(),
            created_at: now,
            modified_at: now,
        }
    }

    /// Check if a session has the required permission
    pub fn has_permission(&self, session_id: &SessionId, permission: Permission) -> bool {
        // Owner always has admin access
        if self.owner == *session_id {
            return true;
        }

        // Check explicit entries
        self.entries
            .iter()
            .any(|entry| entry.session_id == *session_id && entry.allows(permission))
    }

    /// Grant permission to a session
    pub fn grant_permission(
        &mut self,
        session_id: SessionId,
        permission: Permission,
        granted_by: SessionId,
    ) -> Result<()> {
        // Only owner or admin can grant permissions
        if !self.has_permission(&granted_by, Permission::Admin) {
            return Err(SessionError::AccessDenied {
                message: format!("Session {granted_by} cannot grant permissions"),
            });
        }

        // Remove any existing entry for this session
        self.entries.retain(|entry| entry.session_id != session_id);

        // Add new entry
        self.entries.push(AccessControlEntry::new(
            session_id,
            permission,
            Some(granted_by),
        ));

        self.modified_at = Utc::now();
        Ok(())
    }

    /// Revoke permission from a session
    pub fn revoke_permission(
        &mut self,
        session_id: &SessionId,
        revoked_by: SessionId,
    ) -> Result<()> {
        // Only owner or admin can revoke permissions
        if !self.has_permission(&revoked_by, Permission::Admin) {
            return Err(SessionError::AccessDenied {
                message: format!("Session {revoked_by} cannot revoke permissions"),
            });
        }

        // Cannot revoke owner's permissions
        if self.owner == *session_id {
            return Err(SessionError::AccessDenied {
                message: "Cannot revoke owner's permissions".to_string(),
            });
        }

        // Remove entry
        let before_len = self.entries.len();
        self.entries.retain(|entry| entry.session_id != *session_id);

        if self.entries.len() == before_len {
            return Err(SessionError::ArtifactNotFound {
                id: format!("permissions for session {session_id}"),
            });
        }

        self.modified_at = Utc::now();
        Ok(())
    }

    /// Get all sessions with access
    pub fn get_sessions_with_access(&self) -> Vec<SessionId> {
        let mut sessions = vec![self.owner];
        sessions.extend(
            self.entries
                .iter()
                .filter(|entry| !entry.is_expired())
                .map(|entry| entry.session_id),
        );
        sessions
    }

    /// Clean up expired entries
    pub fn cleanup_expired(&mut self) -> bool {
        let before_len = self.entries.len();
        self.entries.retain(|entry| !entry.is_expired());

        if self.entries.len() == before_len {
            false
        } else {
            self.modified_at = Utc::now();
            true
        }
    }
}

/// Audit log entry for access attempts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessAuditEntry {
    /// Artifact that was accessed
    pub artifact_id: ArtifactId,
    /// Session attempting access
    pub session_id: SessionId,
    /// Type of access attempted
    pub access_type: AccessType,
    /// Whether access was granted
    pub granted: bool,
    /// Reason for denial (if denied)
    pub denial_reason: Option<String>,
    /// Timestamp of access attempt
    pub timestamp: DateTime<Utc>,
    /// Source IP or identifier (if available)
    pub source: Option<String>,
}

/// Type of access being attempted
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AccessType {
    /// Reading artifact content
    Read,
    /// Modifying artifact content
    Write,
    /// Deleting artifact
    Delete,
    /// Listing/searching artifacts
    List,
    /// Sharing artifact with another session
    Share,
    /// Changing permissions
    ChangePermissions,
}

impl fmt::Display for AccessType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AccessType::Read => write!(f, "read"),
            AccessType::Write => write!(f, "write"),
            AccessType::Delete => write!(f, "delete"),
            AccessType::List => write!(f, "list"),
            AccessType::Share => write!(f, "share"),
            AccessType::ChangePermissions => write!(f, "change_permissions"),
        }
    }
}

/// Access control manager for artifacts
#[derive(Debug)]
pub struct AccessControlManager {
    /// ACLs for each artifact (`artifact_id` -> ACL)
    acls: Arc<RwLock<HashMap<ArtifactId, AccessControlList>>>,
    /// Audit log entries
    audit_log: Arc<RwLock<Vec<AccessAuditEntry>>>,
    /// Configuration
    config: AccessControlConfig,
}

/// Configuration for access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlConfig {
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Maximum audit log entries to keep in memory
    pub max_audit_entries: usize,
    /// Default permissions for different artifact types
    pub default_permissions: HashMap<ArtifactType, Permission>,
    /// Whether to allow cross-session sharing
    pub allow_cross_session_sharing: bool,
    /// Auto-cleanup interval for expired permissions
    pub cleanup_interval_minutes: u64,
}

impl Default for AccessControlConfig {
    fn default() -> Self {
        let mut default_permissions = HashMap::new();
        default_permissions.insert(ArtifactType::UserInput, Permission::Admin);
        default_permissions.insert(ArtifactType::AgentOutput, Permission::Read);
        default_permissions.insert(ArtifactType::ToolResult, Permission::Read);
        default_permissions.insert(ArtifactType::SystemGenerated, Permission::Read);

        Self {
            enable_audit_logging: true,
            max_audit_entries: 10000,
            default_permissions,
            allow_cross_session_sharing: true,
            cleanup_interval_minutes: 60,
        }
    }
}

impl AccessControlManager {
    /// Create a new access control manager
    pub fn new(config: AccessControlConfig) -> Self {
        Self {
            acls: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Initialize ACL for a new artifact
    pub async fn initialize_acl(&self, artifact_id: ArtifactId, owner: SessionId) -> Result<()> {
        let mut acls = self.acls.write().await;

        if let Some(existing_acl) = acls.get(&artifact_id) {
            // If ACL already exists, verify it has the same owner
            if existing_acl.owner != owner {
                return Err(SessionError::InvalidOperation {
                    reason: format!(
                        "ACL already exists for artifact {} with different owner",
                        artifact_id.storage_key()
                    ),
                });
            }
            // ACL exists with same owner, this is fine (content deduplication case)
            return Ok(());
        }

        acls.insert(artifact_id, AccessControlList::new(owner));
        Ok(())
    }

    /// Check if a session has permission to access an artifact
    pub async fn check_permission(
        &self,
        artifact_id: &ArtifactId,
        session_id: &SessionId,
        access_type: AccessType,
    ) -> Result<bool> {
        let required_permission = match access_type {
            AccessType::Read | AccessType::List => Permission::Read,
            AccessType::Write => Permission::Write,
            AccessType::Delete | AccessType::Share | AccessType::ChangePermissions => {
                Permission::Admin
            }
        };

        let acls = self.acls.read().await;
        let has_permission = if let Some(acl) = acls.get(artifact_id) {
            acl.has_permission(session_id, required_permission)
        } else {
            // No ACL exists - deny access
            false
        };

        // Log the access attempt
        if self.config.enable_audit_logging {
            self.log_access_attempt(
                artifact_id.clone(),
                *session_id,
                access_type,
                has_permission,
                if has_permission {
                    None
                } else {
                    Some("Access denied by ACL".to_string())
                },
            )
            .await;
        }

        Ok(has_permission)
    }

    /// Grant permission to a session for an artifact
    pub async fn grant_permission(
        &self,
        artifact_id: &ArtifactId,
        target_session: SessionId,
        permission: Permission,
        granted_by: SessionId,
    ) -> Result<()> {
        if !self.config.allow_cross_session_sharing && target_session != granted_by {
            return Err(SessionError::AccessDenied {
                message: "Cross-session sharing is disabled".to_string(),
            });
        }

        let mut acls = self.acls.write().await;
        if let Some(acl) = acls.get_mut(artifact_id) {
            acl.grant_permission(target_session, permission, granted_by)?;

            // Log the permission grant
            if self.config.enable_audit_logging {
                self.log_access_attempt(
                    artifact_id.clone(),
                    granted_by,
                    AccessType::ChangePermissions,
                    true,
                    Some(format!(
                        "Granted {permission} permission to {target_session}"
                    )),
                )
                .await;
            }

            Ok(())
        } else {
            Err(SessionError::ArtifactNotFound {
                id: format!("ACL for artifact {}", artifact_id.storage_key()),
            })
        }
    }

    /// Revoke permission from a session for an artifact
    pub async fn revoke_permission(
        &self,
        artifact_id: &ArtifactId,
        target_session: &SessionId,
        revoked_by: SessionId,
    ) -> Result<()> {
        let mut acls = self.acls.write().await;
        if let Some(acl) = acls.get_mut(artifact_id) {
            acl.revoke_permission(target_session, revoked_by)?;

            // Log the permission revocation
            if self.config.enable_audit_logging {
                self.log_access_attempt(
                    artifact_id.clone(),
                    revoked_by,
                    AccessType::ChangePermissions,
                    true,
                    Some(format!("Revoked permissions from {target_session}")),
                )
                .await;
            }

            Ok(())
        } else {
            Err(SessionError::ArtifactNotFound {
                id: format!("ACL for artifact {}", artifact_id.storage_key()),
            })
        }
    }

    /// Get access control list for an artifact
    pub async fn get_acl(&self, artifact_id: &ArtifactId) -> Result<AccessControlList> {
        let acls = self.acls.read().await;
        acls.get(artifact_id)
            .cloned()
            .ok_or_else(|| SessionError::ArtifactNotFound {
                id: format!("ACL for artifact {}", artifact_id.storage_key()),
            })
    }

    /// List artifacts accessible by a session
    pub async fn list_accessible_artifacts(&self, session_id: &SessionId) -> Vec<ArtifactId> {
        let acls = self.acls.read().await;
        acls.iter()
            .filter(|(_, acl)| acl.has_permission(session_id, Permission::Read))
            .map(|(artifact_id, _)| artifact_id.clone())
            .collect()
    }

    /// Get audit log entries for an artifact
    pub async fn get_audit_log(&self, artifact_id: &ArtifactId) -> Vec<AccessAuditEntry> {
        let audit_log = self.audit_log.read().await;
        audit_log
            .iter()
            .filter(|entry| entry.artifact_id == *artifact_id)
            .cloned()
            .collect()
    }

    /// Cleanup expired permissions and old audit entries
    pub async fn cleanup(&self) {
        // Cleanup expired ACL entries
        {
            let mut acls = self.acls.write().await;
            for acl in acls.values_mut() {
                acl.cleanup_expired();
            }
        }

        // Cleanup old audit entries
        if self.config.enable_audit_logging {
            let mut audit_log = self.audit_log.write().await;
            let current_len = audit_log.len();
            if current_len > self.config.max_audit_entries {
                let keep_count = self.config.max_audit_entries * 3 / 4; // Keep 75%
                audit_log.drain(0..current_len - keep_count);
            }
        }
    }

    /// Internal method to log access attempts
    async fn log_access_attempt(
        &self,
        artifact_id: ArtifactId,
        session_id: SessionId,
        access_type: AccessType,
        granted: bool,
        denial_reason: Option<String>,
    ) {
        let entry = AccessAuditEntry {
            artifact_id,
            session_id,
            access_type,
            granted,
            denial_reason,
            timestamp: Utc::now(),
            source: None, // Could be extended to include IP/source info
        };

        let mut audit_log = self.audit_log.write().await;
        audit_log.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn create_test_session_id() -> SessionId {
        SessionId::from_uuid(Uuid::new_v4())
    }

    fn create_test_artifact_id() -> ArtifactId {
        ArtifactId::new("test_hash".to_string(), create_test_session_id(), 1)
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_permission_allows() {
        let entry = AccessControlEntry::new(create_test_session_id(), Permission::Write, None);

        assert!(entry.allows(Permission::Read));
        assert!(entry.allows(Permission::Write));
        assert!(!entry.allows(Permission::Admin));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_acl_owner_permissions() {
        let owner = create_test_session_id();
        let other = create_test_session_id();
        let acl = AccessControlList::new(owner);

        assert!(acl.has_permission(&owner, Permission::Admin));
        assert!(acl.has_permission(&owner, Permission::Write));
        assert!(acl.has_permission(&owner, Permission::Read));

        assert!(!acl.has_permission(&other, Permission::Read));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_acl_grant_revoke() {
        let owner = create_test_session_id();
        let user = create_test_session_id();
        let mut acl = AccessControlList::new(owner);

        // Grant permission
        assert!(acl.grant_permission(user, Permission::Read, owner).is_ok());
        assert!(acl.has_permission(&user, Permission::Read));
        assert!(!acl.has_permission(&user, Permission::Write));

        // Revoke permission
        assert!(acl.revoke_permission(&user, owner).is_ok());
        assert!(!acl.has_permission(&user, Permission::Read));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_access_control_manager() {
        let config = AccessControlConfig::default();
        let manager = AccessControlManager::new(config);

        let artifact_id = create_test_artifact_id();
        let owner = create_test_session_id();
        let user = create_test_session_id();

        // Initialize ACL
        assert!(manager
            .initialize_acl(artifact_id.clone(), owner)
            .await
            .is_ok());

        // Owner should have access
        assert!(manager
            .check_permission(&artifact_id, &owner, AccessType::Read)
            .await
            .unwrap());
        assert!(manager
            .check_permission(&artifact_id, &owner, AccessType::Delete)
            .await
            .unwrap());

        // Other session should not have access
        assert!(!manager
            .check_permission(&artifact_id, &user, AccessType::Read)
            .await
            .unwrap());

        // Grant permission
        assert!(manager
            .grant_permission(&artifact_id, user, Permission::Read, owner)
            .await
            .is_ok());
        assert!(manager
            .check_permission(&artifact_id, &user, AccessType::Read)
            .await
            .unwrap());
        assert!(!manager
            .check_permission(&artifact_id, &user, AccessType::Delete)
            .await
            .unwrap());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_audit_logging() {
        let config = AccessControlConfig::default();
        let manager = AccessControlManager::new(config);

        let artifact_id = create_test_artifact_id();
        let owner = create_test_session_id();
        let user = create_test_session_id();

        // Initialize ACL
        assert!(manager
            .initialize_acl(artifact_id.clone(), owner)
            .await
            .is_ok());

        // Make access attempts
        let _ = manager
            .check_permission(&artifact_id, &owner, AccessType::Read)
            .await;
        let _ = manager
            .check_permission(&artifact_id, &user, AccessType::Read)
            .await;

        // Check audit log
        let audit_entries = manager.get_audit_log(&artifact_id).await;
        assert_eq!(audit_entries.len(), 2);
        assert!(audit_entries[0].granted); // Owner access should be granted
        assert!(!audit_entries[1].granted); // User access should be denied
    }
}
