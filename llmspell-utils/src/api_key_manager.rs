//! API Key Management System
//!
//! Provides secure storage and management of API keys with support for:
//! - Environment variable loading
//! - Configuration file support
//! - Key rotation
//! - Audit logging
//! - Secure storage

#![allow(clippy::missing_errors_doc)]
#![allow(clippy::uninlined_format_args)]
#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::new_without_default)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::must_use_candidate)]

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// API key metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyMetadata {
    /// Key identifier
    pub key_id: String,
    /// Service this key is for (e.g., "google_search", "sendgrid")
    pub service: String,
    /// When the key was created
    pub created_at: DateTime<Utc>,
    /// When the key was last used
    pub last_used: Option<DateTime<Utc>>,
    /// When the key expires (if applicable)
    pub expires_at: Option<DateTime<Utc>>,
    /// Whether the key is currently active
    pub is_active: bool,
    /// Usage count
    pub usage_count: u64,
}

/// API key entry
#[derive(Debug, Clone)]
struct ApiKeyEntry {
    /// The actual API key (stored securely)
    key: String,
    /// Metadata about the key
    metadata: ApiKeyMetadata,
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAuditEntry {
    /// Timestamp of the event
    pub timestamp: DateTime<Utc>,
    /// Key that was accessed
    pub key_id: String,
    /// Service that accessed the key
    pub service: String,
    /// Type of access (read, rotate, deactivate)
    pub action: ApiKeyAction,
    /// Optional details about the access
    pub details: Option<String>,
}

/// Types of API key actions for audit logging
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ApiKeyAction {
    /// Key was read/accessed
    Read,
    /// New key was created
    Create,
    /// Key was updated
    Update,
    /// Key was rotated
    Rotate,
    /// Key was deactivated
    Deactivate,
    /// Key was deleted
    Delete,
}

/// API key storage backend
pub trait ApiKeyStorage: Send + Sync {
    /// Store a key
    fn store(&mut self, key_id: &str, key: &str, metadata: &ApiKeyMetadata) -> Result<(), String>;

    /// Get a key
    fn get(&self, key_id: &str) -> Result<Option<String>, String>;

    /// Get metadata
    fn get_metadata(&self, key_id: &str) -> Result<Option<ApiKeyMetadata>, String>;

    /// Update key metadata
    fn update_metadata(&mut self, key_id: &str, metadata: &ApiKeyMetadata) -> Result<(), String>;

    /// Delete a key
    fn delete(&mut self, key_id: &str) -> Result<(), String>;
    /// List all key IDs
    fn list_keys(&self) -> Result<Vec<String>, String>;
}

/// In-memory storage (for development/testing)
#[derive(Default)]
pub struct InMemoryStorage {
    keys: HashMap<String, ApiKeyEntry>,
}

impl ApiKeyStorage for InMemoryStorage {
    fn store(&mut self, key_id: &str, key: &str, metadata: &ApiKeyMetadata) -> Result<(), String> {
        self.keys.insert(
            key_id.to_string(),
            ApiKeyEntry {
                key: key.to_string(),
                metadata: metadata.clone(),
            },
        );
        Ok(())
    }

    fn get(&self, key_id: &str) -> Result<Option<String>, String> {
        Ok(self.keys.get(key_id).map(|entry| entry.key.clone()))
    }

    fn get_metadata(&self, key_id: &str) -> Result<Option<ApiKeyMetadata>, String> {
        Ok(self.keys.get(key_id).map(|entry| entry.metadata.clone()))
    }

    fn update_metadata(&mut self, key_id: &str, metadata: &ApiKeyMetadata) -> Result<(), String> {
        if let Some(entry) = self.keys.get_mut(key_id) {
            entry.metadata = metadata.clone();
            Ok(())
        } else {
            Err(format!("Key '{}' not found", key_id))
        }
    }

    fn delete(&mut self, key_id: &str) -> Result<(), String> {
        self.keys
            .remove(key_id)
            .map(|_| ())
            .ok_or_else(|| format!("Key '{}' not found", key_id))
    }

    fn list_keys(&self) -> Result<Vec<String>, String> {
        Ok(self.keys.keys().cloned().collect())
    }
}

/// API Key Manager
pub struct ApiKeyManager {
    /// Storage backend
    storage: Arc<RwLock<Box<dyn ApiKeyStorage>>>,
    /// Audit log
    audit_log: Arc<RwLock<Vec<ApiKeyAuditEntry>>>,
    /// Environment variable prefix for loading keys
    env_prefix: String,
}

impl ApiKeyManager {
    /// Create a new API key manager with in-memory storage
    pub fn new() -> Self {
        Self::with_storage(Box::new(InMemoryStorage::default()))
    }

    /// Create a new API key manager with custom storage
    pub fn with_storage(storage: Box<dyn ApiKeyStorage>) -> Self {
        Self {
            storage: Arc::new(RwLock::new(storage)),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            env_prefix: "LLMSPELL_API_KEY_".to_string(),
        }
    }

    /// Set the environment variable prefix
    pub fn set_env_prefix(&mut self, prefix: &str) {
        self.env_prefix = prefix.to_string();
    }

    /// Load API keys from environment variables
    pub fn load_from_env(&self) -> Result<usize, String> {
        let mut loaded = 0;

        for (key, value) in std::env::vars() {
            if key.starts_with(&self.env_prefix) {
                let service = key.trim_start_matches(&self.env_prefix).to_lowercase();
                let key_id = format!("env_{}", service);

                let metadata = ApiKeyMetadata {
                    key_id: key_id.clone(),
                    service: service.clone(),
                    created_at: Utc::now(),
                    last_used: None,
                    expires_at: None,
                    is_active: true,
                    usage_count: 0,
                };

                self.add_key(&key_id, &value, metadata)?;
                loaded += 1;

                debug!("Loaded API key for service '{}' from environment", service);
            }
        }

        Ok(loaded)
    }

    /// Load API keys from a configuration file
    pub fn load_from_config(&self, config: HashMap<String, String>) -> Result<usize, String> {
        let mut loaded = 0;

        for (service, key) in config {
            let key_id = format!("config_{}", service);

            let metadata = ApiKeyMetadata {
                key_id: key_id.clone(),
                service: service.clone(),
                created_at: Utc::now(),
                last_used: None,
                expires_at: None,
                is_active: true,
                usage_count: 0,
            };

            self.add_key(&key_id, &key, metadata)?;
            loaded += 1;

            debug!("Loaded API key for service '{}' from config", service);
        }

        Ok(loaded)
    }

    /// Add a new API key
    pub fn add_key(&self, key_id: &str, key: &str, metadata: ApiKeyMetadata) -> Result<(), String> {
        let mut storage = self.storage.write();
        storage.store(key_id, key, &metadata)?;

        self.log_action(ApiKeyAuditEntry {
            timestamp: Utc::now(),
            key_id: key_id.to_string(),
            service: metadata.service.clone(),
            action: ApiKeyAction::Create,
            details: None,
        });

        info!(
            "Added API key '{}' for service '{}'",
            key_id, metadata.service
        );
        Ok(())
    }

    /// Get an API key by service name
    pub fn get_key(&self, service: &str) -> Result<Option<String>, String> {
        // First, find the key_id for the service
        let key_id_to_use = {
            let storage = self.storage.read();
            let key_ids = storage.list_keys()?;

            let mut found_key_id = None;
            for key_id in key_ids {
                if let Some(_key) = storage.get(&key_id)? {
                    // Get metadata to check service directly from storage to avoid nested locks
                    if let Some(metadata) = storage.get_metadata(&key_id)? {
                        if metadata.service == service && metadata.is_active {
                            // Check expiration
                            if let Some(expires_at) = metadata.expires_at {
                                if expires_at < Utc::now() {
                                    warn!(
                                        "API key '{}' for service '{}' has expired",
                                        key_id, service
                                    );
                                    continue;
                                }
                            }
                            found_key_id = Some(key_id);
                            break;
                        }
                    }
                }
            }
            found_key_id
        };

        // Now update metadata and retrieve key
        if let Some(key_id) = key_id_to_use {
            let mut storage = self.storage.write();

            if let Some(key) = storage.get(&key_id)? {
                // Get metadata directly from storage to avoid nested locks
                if let Some(mut metadata) = storage.get_metadata(&key_id)? {
                    metadata.last_used = Some(Utc::now());
                    metadata.usage_count += 1;
                    storage.update_metadata(&key_id, &metadata)?;

                    self.log_action(ApiKeyAuditEntry {
                        timestamp: Utc::now(),
                        key_id: key_id.clone(),
                        service: service.to_string(),
                        action: ApiKeyAction::Read,
                        details: None,
                    });

                    return Ok(Some(key));
                }
            }
        }

        Ok(None)
    }

    /// Get metadata for a key
    pub fn get_metadata(&self, key_id: &str) -> Result<Option<ApiKeyMetadata>, String> {
        let storage = self.storage.read();
        storage.get_metadata(key_id)
    }

    /// Rotate an API key
    pub fn rotate_key(&self, service: &str, new_key: &str) -> Result<(), String> {
        // Find the current key for the service
        let storage = self.storage.read();
        let key_ids = storage.list_keys()?;
        drop(storage);

        for key_id in key_ids {
            if let Some(metadata) = self.get_metadata(&key_id)? {
                if metadata.service == service && metadata.is_active {
                    // Deactivate old key
                    self.deactivate_key(&key_id)?;

                    // Add new key
                    let new_key_id = format!("{}_rotated_{}", key_id, Utc::now().timestamp());
                    let new_metadata = ApiKeyMetadata {
                        key_id: new_key_id.clone(),
                        service: service.to_string(),
                        created_at: Utc::now(),
                        last_used: None,
                        expires_at: metadata.expires_at,
                        is_active: true,
                        usage_count: 0,
                    };

                    self.add_key(&new_key_id, new_key, new_metadata)?;

                    self.log_action(ApiKeyAuditEntry {
                        timestamp: Utc::now(),
                        key_id: key_id.clone(),
                        service: service.to_string(),
                        action: ApiKeyAction::Rotate,
                        details: Some(format!("Rotated to key '{}'", new_key_id)),
                    });

                    info!("Rotated API key for service '{}'", service);
                    return Ok(());
                }
            }
        }

        Err(format!("No active key found for service '{}'", service))
    }

    /// Deactivate a key
    pub fn deactivate_key(&self, key_id: &str) -> Result<(), String> {
        let mut storage = self.storage.write();

        // Get metadata directly from storage to avoid nested locks
        if let Some(mut metadata) = storage.get_metadata(key_id)? {
            metadata.is_active = false;
            storage.update_metadata(key_id, &metadata)?;

            self.log_action(ApiKeyAuditEntry {
                timestamp: Utc::now(),
                key_id: key_id.to_string(),
                service: metadata.service.clone(),
                action: ApiKeyAction::Deactivate,
                details: None,
            });

            info!("Deactivated API key '{}'", key_id);
            Ok(())
        } else {
            Err(format!("Key '{}' not found", key_id))
        }
    }

    /// Get audit log entries
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<ApiKeyAuditEntry> {
        let log = self.audit_log.read();
        match limit {
            Some(n) => log.iter().rev().take(n).cloned().collect(),
            None => log.clone(),
        }
    }

    /// Clear audit log
    pub fn clear_audit_log(&self) {
        let mut log = self.audit_log.write();
        log.clear();
    }

    /// Log an action to the audit log
    fn log_action(&self, entry: ApiKeyAuditEntry) {
        let mut log = self.audit_log.write();
        log.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_manager_basic() {
        let manager = ApiKeyManager::new();

        // Add a key
        let metadata = ApiKeyMetadata {
            key_id: "test_key".to_string(),
            service: "test_service".to_string(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            usage_count: 0,
        };

        manager
            .add_key("test_key", "secret_value", metadata)
            .unwrap();

        // Retrieve the key
        let key = manager.get_key("test_service").unwrap();
        assert_eq!(key, Some("secret_value".to_string()));

        // Check audit log
        let audit_log = manager.get_audit_log(None);
        assert_eq!(audit_log.len(), 2); // Create + Read
    }

    #[test]
    fn test_key_rotation() {
        let manager = ApiKeyManager::new();

        // Add initial key
        let metadata = ApiKeyMetadata {
            key_id: "test_key".to_string(),
            service: "test_service".to_string(),
            created_at: Utc::now(),
            last_used: None,
            expires_at: None,
            is_active: true,
            usage_count: 0,
        };

        manager.add_key("test_key", "old_secret", metadata).unwrap();

        // Rotate the key
        manager.rotate_key("test_service", "new_secret").unwrap();

        // Verify new key is active
        let key = manager.get_key("test_service").unwrap();
        assert_eq!(key, Some("new_secret".to_string()));

        // Check audit log
        let audit_log = manager.get_audit_log(None);
        assert!(audit_log
            .iter()
            .any(|entry| matches!(entry.action, ApiKeyAction::Rotate)));
    }

    #[test]
    fn test_load_from_env() {
        // Set test environment variable
        std::env::set_var("LLMSPELL_API_KEY_TEST", "env_secret");

        let manager = ApiKeyManager::new();
        let loaded = manager.load_from_env().unwrap();
        assert!(loaded > 0);

        // Verify key was loaded
        let key = manager.get_key("test").unwrap();
        assert_eq!(key, Some("env_secret".to_string()));

        // Clean up
        std::env::remove_var("LLMSPELL_API_KEY_TEST");
    }
}
