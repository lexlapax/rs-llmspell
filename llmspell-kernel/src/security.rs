//! Security and authentication for kernel connections
//!
//! Provides authentication, authorization, and security features for kernel-client communication.

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Authentication token for client connections
#[derive(Debug, Clone)]
pub struct AuthToken {
    /// Token value
    pub token: String,
    /// Client ID associated with the token
    pub client_id: String,
    /// Token creation time
    pub created_at: DateTime<Utc>,
    /// Token expiration time
    pub expires_at: DateTime<Utc>,
    /// Token permissions
    pub permissions: Permissions,
}

impl AuthToken {
    /// Create a new authentication token
    #[must_use]
    pub fn new(client_id: String, duration: Duration) -> Self {
        let now = Utc::now();
        Self {
            token: Uuid::new_v4().to_string(),
            client_id,
            created_at: now,
            expires_at: now + duration,
            permissions: Permissions::default(),
        }
    }

    /// Check if token is expired
    #[must_use]
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if token is valid
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
}

/// Permission types for authenticated clients
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Permission {
    /// Can execute code
    Execute,
    /// Can debug
    Debug,
    /// Can interrupt execution
    Interrupt,
    /// Can shutdown kernel
    Shutdown,
    /// Can access files
    AccessFiles,
    /// Can modify kernel state
    ModifyState,
}

/// Permission set for authenticated clients
#[derive(Debug, Clone, Default)]
pub struct Permissions {
    /// Set of granted permissions
    permissions: HashSet<Permission>,
}

impl Permissions {
    /// Create permissions with all rights
    #[must_use]
    pub fn all() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Execute);
        permissions.insert(Permission::Debug);
        permissions.insert(Permission::Interrupt);
        permissions.insert(Permission::Shutdown);
        permissions.insert(Permission::AccessFiles);
        permissions.insert(Permission::ModifyState);
        Self { permissions }
    }

    /// Create read-only permissions
    #[must_use]
    pub fn read_only() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Debug);
        Self { permissions }
    }

    /// Create execution-only permissions
    #[must_use]
    pub fn execute_only() -> Self {
        let mut permissions = HashSet::new();
        permissions.insert(Permission::Execute);
        permissions.insert(Permission::Interrupt);
        Self { permissions }
    }

    /// Check if a permission is granted
    #[must_use]
    pub fn has(&self, permission: Permission) -> bool {
        self.permissions.contains(&permission)
    }

    /// Grant a permission
    pub fn grant(&mut self, permission: Permission) {
        self.permissions.insert(permission);
    }

    /// Revoke a permission
    pub fn revoke(&mut self, permission: Permission) {
        self.permissions.remove(&permission);
    }
}

/// Security manager for kernel authentication and authorization
pub struct SecurityManager {
    /// Active authentication tokens
    tokens: Arc<RwLock<HashMap<String, AuthToken>>>,
    /// Kernel secret key for HMAC
    #[allow(dead_code)]
    kernel_key: String,
    /// Whether authentication is required
    auth_required: bool,
    /// Default token duration
    default_token_duration: Duration,
}

impl SecurityManager {
    /// Create a new security manager
    #[must_use]
    pub fn new(kernel_key: String, auth_required: bool) -> Self {
        Self {
            tokens: Arc::new(RwLock::new(HashMap::new())),
            kernel_key,
            auth_required,
            default_token_duration: Duration::hours(24),
        }
    }

    /// Create a new authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if token creation fails
    pub async fn create_token(&self, client_id: String) -> Result<AuthToken> {
        let token = AuthToken::new(client_id, self.default_token_duration);

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.token.clone(), token.clone());
        drop(tokens);

        tracing::info!("Created auth token for client {}", token.client_id);
        Ok(token)
    }

    /// Validate an authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if token is invalid or expired
    pub async fn validate_token(&self, token_str: &str) -> Result<AuthToken> {
        let tokens = self.tokens.read().await;

        let token = tokens
            .get(token_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid token"))?;

        if !token.is_valid() {
            anyhow::bail!("Token expired");
        }

        let cloned = token.clone();
        drop(tokens);
        Ok(cloned)
    }

    /// Revoke an authentication token
    ///
    /// # Errors
    ///
    /// Returns an error if revocation fails
    pub async fn revoke_token(&self, token_str: &str) -> Result<()> {
        let token = self.tokens.write().await.remove(token_str);
        if let Some(token) = token {
            tracing::info!("Revoked token for client {}", token.client_id);
        }

        Ok(())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> usize {
        let mut tokens = self.tokens.write().await;
        let before = tokens.len();

        tokens.retain(|_, token| token.is_valid());

        let removed = before - tokens.len();
        drop(tokens);
        if removed > 0 {
            tracing::info!("Cleaned up {} expired tokens", removed);
        }

        removed
    }

    /// Check if a client has permission for an action
    ///
    /// # Errors
    ///
    /// Returns an error if permission check fails
    pub async fn check_permission(&self, token_str: &str, action: &str) -> Result<bool> {
        if !self.auth_required {
            return Ok(true);
        }

        let token = self.validate_token(token_str).await?;

        let permission = match action {
            "execute" => Some(Permission::Execute),
            "debug" => Some(Permission::Debug),
            "interrupt" => Some(Permission::Interrupt),
            "shutdown" => Some(Permission::Shutdown),
            "access_files" => Some(Permission::AccessFiles),
            "modify_state" => Some(Permission::ModifyState),
            _ => None,
        };

        let allowed = permission.is_some_and(|p| token.permissions.has(p));

        if !allowed {
            tracing::warn!(
                "Permission denied for client {} to {}",
                token.client_id,
                action
            );
        }

        Ok(allowed)
    }

    /// Generate HMAC signature for message authentication
    #[must_use]
    pub fn sign_message(&self, message: &[u8]) -> String {
        // In a real implementation, use proper HMAC-SHA256
        // For now, return a placeholder
        format!("hmac-{}", hex::encode(&message[..message.len().min(8)]))
    }

    /// Verify HMAC signature
    #[must_use]
    pub fn verify_signature(&self, message: &[u8], signature: &str) -> bool {
        // In a real implementation, verify proper HMAC-SHA256
        // For now, do basic check
        let expected = self.sign_message(message);
        expected == signature
    }
}

impl Default for SecurityManager {
    fn default() -> Self {
        Self::new(Uuid::new_v4().to_string(), false)
    }
}

/// Audit log entry
#[derive(Debug, Clone)]
pub struct AuditEntry {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Client ID
    pub client_id: String,
    /// Action performed
    pub action: String,
    /// Whether action was allowed
    pub allowed: bool,
    /// Additional details
    pub details: Option<String>,
}

/// Audit logger for security events
pub struct AuditLog {
    entries: Arc<RwLock<Vec<AuditEntry>>>,
    max_entries: usize,
}

impl AuditLog {
    /// Create a new audit log
    #[must_use]
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(Vec::new())),
            max_entries,
        }
    }

    /// Log an audit entry
    pub async fn log(
        &self,
        client_id: String,
        action: String,
        allowed: bool,
        details: Option<String>,
    ) {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            client_id,
            action,
            allowed,
            details,
        };

        let mut entries = self.entries.write().await;
        entries.push(entry);

        // Keep only the most recent entries
        if entries.len() > self.max_entries {
            let drain_count = entries.len() - self.max_entries;
            entries.drain(0..drain_count);
        }
    }

    /// Get recent audit entries
    pub async fn get_recent(&self, count: usize) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        let start = entries.len().saturating_sub(count);
        entries[start..].to_vec()
    }

    /// Get entries for a specific client
    pub async fn get_by_client(&self, client_id: &str) -> Vec<AuditEntry> {
        let entries = self.entries.read().await;
        entries
            .iter()
            .filter(|e| e.client_id == client_id)
            .cloned()
            .collect()
    }
}

// Add hex crate for HMAC placeholder
use hex;
