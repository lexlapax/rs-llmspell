// ABOUTME: Key validation and namespace management for state isolation
// ABOUTME: Ensures key security and prevents traversal attacks

use crate::error::{StateError, StateResult};
use crate::scope::StateScope;
use unicode_normalization::UnicodeNormalization;

pub struct KeyManager;

impl KeyManager {
    /// Validate a state key
    pub fn validate_key(key: &str) -> StateResult<()> {
        // Check empty key
        if key.is_empty() {
            return Err(StateError::InvalidKey("Key cannot be empty".to_string()));
        }
        
        // Check key length
        if key.len() > 256 {
            return Err(StateError::InvalidKey(
                "Key cannot be longer than 256 characters".to_string(),
            ));
        }
        
        // Prevent path traversal
        if key.contains("..") || key.contains("\\") || key.contains("//") {
            return Err(StateError::InvalidKey(
                "Key contains invalid path traversal characters".to_string(),
            ));
        }
        
        // Check for invalid characters
        if key.contains('\0') || key.contains('\n') || key.contains('\r') {
            return Err(StateError::InvalidKey(
                "Key contains invalid control characters".to_string(),
            ));
        }
        
        // Check for reserved prefixes
        if key.starts_with("__") || key.starts_with("$$") {
            return Err(StateError::InvalidKey(
                "Key cannot start with reserved prefixes __ or $$".to_string(),
            ));
        }
        
        Ok(())
    }

    /// Create a scoped key with namespace prefix
    pub fn create_scoped_key(scope: &StateScope, key: &str) -> StateResult<String> {
        Self::validate_key(key)?;
        
        let normalized_key = key.nfc().collect::<String>();
        let scoped_key = format!("{}{}", scope.prefix(), normalized_key);
        
        // Ensure final key is still within limits
        if scoped_key.len() > 512 {
            return Err(StateError::InvalidKey(
                "Scoped key exceeds maximum length".to_string(),
            ));
        }
        
        Ok(scoped_key)
    }

    /// Extract the original key from a scoped key
    pub fn extract_key(scoped_key: &str, scope: &StateScope) -> Option<String> {
        let prefix = scope.prefix();
        if scoped_key.starts_with(&prefix) {
            Some(scoped_key[prefix.len()..].to_string())
        } else {
            None
        }
    }

    /// Check if a scoped key belongs to a specific scope
    pub fn belongs_to_scope(scoped_key: &str, scope: &StateScope) -> bool {
        let prefix = scope.prefix();
        if prefix.is_empty() {
            // Global scope - only keys without any prefix belong to it
            !scoped_key.contains(':')
        } else {
            scoped_key.starts_with(&prefix)
        }
    }

    /// Generate a collision-resistant key for internal use
    pub fn generate_internal_key(prefix: &str, id: &str) -> String {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        
        format!("__{}:{}:{}", prefix, id, timestamp)
    }

    /// Sanitize a key for safe storage
    pub fn sanitize_key(key: &str) -> String {
        key.chars()
            .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.' || *c == ':')
            .take(256)
            .collect()
    }
}

/// Permission system for state access control
#[derive(Debug, Clone, PartialEq)]
pub enum StatePermission {
    Read,
    Write,
    Delete,
    List,
}

/// Access control for state operations
pub struct StateAccessControl {
    permissions: std::collections::HashMap<(StateScope, String), Vec<StatePermission>>,
}

impl StateAccessControl {
    pub fn new() -> Self {
        Self {
            permissions: std::collections::HashMap::new(),
        }
    }

    /// Grant permission to an agent for a scope
    pub fn grant_permission(
        &mut self,
        agent_id: &str,
        scope: StateScope,
        permission: StatePermission,
    ) {
        let key = (scope, agent_id.to_string());
        let perms = self.permissions.entry(key).or_insert_with(Vec::new);
        if !perms.contains(&permission) {
            perms.push(permission);
        }
    }

    /// Check if an agent has permission for an operation
    pub fn has_permission(
        &self,
        agent_id: &str,
        scope: &StateScope,
        permission: &StatePermission,
    ) -> bool {
        // Check direct permission
        if let Some(perms) = self.permissions.get(&(scope.clone(), agent_id.to_string())) {
            if perms.contains(permission) {
                return true;
            }
        }

        // Check parent scope permissions
        if let Some(parent) = scope.parent() {
            self.has_permission(agent_id, &parent, permission)
        } else {
            false
        }
    }

    /// Revoke all permissions for an agent in a scope
    pub fn revoke_permissions(&mut self, agent_id: &str, scope: StateScope) {
        self.permissions.remove(&(scope, agent_id.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_validation() {
        // Valid keys
        assert!(KeyManager::validate_key("valid_key").is_ok());
        assert!(KeyManager::validate_key("key-with-dashes").is_ok());
        assert!(KeyManager::validate_key("key.with.dots").is_ok());
        assert!(KeyManager::validate_key("key:with:colons").is_ok());

        // Invalid keys
        assert!(KeyManager::validate_key("").is_err());
        assert!(KeyManager::validate_key("../etc/passwd").is_err());
        assert!(KeyManager::validate_key("key\\with\\backslash").is_err());
        assert!(KeyManager::validate_key("key//double//slash").is_err());
        assert!(KeyManager::validate_key("key\nwith\nnewline").is_err());
        assert!(KeyManager::validate_key("__reserved_prefix").is_err());
        assert!(KeyManager::validate_key(&"x".repeat(257)).is_err());
    }

    #[test]
    fn test_scoped_key_creation() {
        let scope = StateScope::Agent("agent123".to_string());
        let key = KeyManager::create_scoped_key(&scope, "my_key").unwrap();
        assert_eq!(key, "agent:agent123:my_key");

        // Test extraction
        let extracted = KeyManager::extract_key(&key, &scope);
        assert_eq!(extracted, Some("my_key".to_string()));

        // Test scope check
        assert!(KeyManager::belongs_to_scope(&key, &scope));
        assert!(!KeyManager::belongs_to_scope(&key, &StateScope::Global));
    }

    #[test]
    fn test_access_control() {
        let mut acl = StateAccessControl::new();
        let agent_id = "agent123";
        let scope = StateScope::Workflow("workflow456".to_string());

        // Grant permissions
        acl.grant_permission(agent_id, scope.clone(), StatePermission::Read);
        acl.grant_permission(agent_id, scope.clone(), StatePermission::Write);

        // Check permissions
        assert!(acl.has_permission(agent_id, &scope, &StatePermission::Read));
        assert!(acl.has_permission(agent_id, &scope, &StatePermission::Write));
        assert!(!acl.has_permission(agent_id, &scope, &StatePermission::Delete));

        // Check parent scope inheritance
        let child_scope = StateScope::Step {
            workflow_id: "workflow456".to_string(),
            step_name: "step1".to_string(),
        };
        assert!(acl.has_permission(agent_id, &child_scope, &StatePermission::Read));
    }

    #[test]
    fn test_key_sanitization() {
        let dirty_key = "key with spaces!@#$%^&*()";
        let clean_key = KeyManager::sanitize_key(dirty_key);
        assert_eq!(clean_key, "keywithspaces");

        let long_key = "x".repeat(300);
        let truncated = KeyManager::sanitize_key(&long_key);
        assert_eq!(truncated.len(), 256);
    }
}