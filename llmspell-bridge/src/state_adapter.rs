// ABOUTME: Adapter to bridge StateManager to StateAccess trait for workflow integration
// ABOUTME: Provides state persistence for workflows through the StateAccess interface

use async_trait::async_trait;
use llmspell_core::{traits::state::StateAccess, LLMSpellError, Result};
use llmspell_kernel::state::{
    config::StorageBackendType, manager::StateManager, PersistenceConfig, StateScope,
};
use serde_json::Value;
use std::fmt;
use std::sync::Arc;
use tracing::{debug, warn};

/// Adapter that bridges `StateManager` to the `StateAccess` trait
///
/// This adapter allows workflows to use the full-featured `StateManager`
/// through the simplified `StateAccess` interface, providing state persistence
/// capabilities to all workflow executions.
#[derive(Clone)]
pub struct StateManagerAdapter {
    /// The underlying state manager instance
    state_manager: Arc<StateManager>,
    /// Default scope for state operations
    default_scope: StateScope,
}

impl fmt::Debug for StateManagerAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateManagerAdapter")
            .field("state_manager", &"Arc<StateManager>")
            .field("default_scope", &self.default_scope)
            .finish()
    }
}

impl StateManagerAdapter {
    /// Create a new adapter with the given state manager and scope
    pub const fn new(state_manager: Arc<StateManager>, default_scope: StateScope) -> Self {
        Self {
            state_manager,
            default_scope,
        }
    }

    /// Create an adapter from configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the state manager fails to initialize
    pub async fn from_config(
        config: &llmspell_config::StatePersistenceConfig,
    ) -> anyhow::Result<Self> {
        // Convert string backend type to enum
        // Phase 13c.2.8: Sled backend removed, RocksDB not yet implemented
        let backend_type = match config.backend_type.as_str() {
            "memory" => StorageBackendType::Memory,
            "rocksdb" => {
                // Create rocksdb config from settings
                StorageBackendType::RocksDB(llmspell_kernel::state::config::RocksDBConfig {
                    path: std::path::PathBuf::from("./data/rocksdb"),
                    create_if_missing: true,
                    optimize_for_point_lookup: false,
                })
            }
            #[cfg(feature = "postgres")]
            "postgres" => {
                warn!("PostgreSQL backend not yet supported via Lua bridge, defaulting to memory");
                StorageBackendType::Memory
            }
            _ => {
                warn!(
                    "Backend type '{}' not supported (use 'memory', 'rocksdb'), defaulting to memory",
                    config.backend_type
                );
                StorageBackendType::Memory
            }
        };

        // Create persistence config
        let persistence_config = PersistenceConfig {
            enabled: config.enabled,
            backend_type,
            flush_interval: std::time::Duration::from_secs(5),
            compression: config.backup.as_ref().is_none_or(|b| b.compression_enabled),
            encryption: None, // TODO: Add encryption config if needed
            backup_retention: std::time::Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            backup: config.backup.as_ref().map(|b| {
                llmspell_kernel::state::config::BackupConfig {
                    backup_dir: std::path::PathBuf::from(
                        b.backup_dir
                            .clone()
                            .unwrap_or_else(|| "./backups".to_string()),
                    ),
                    compression_enabled: b.compression_enabled,
                    compression_type: llmspell_kernel::state::config::CompressionType::Zstd,
                    compression_level: b.compression_level,
                    encryption_enabled: false,
                    max_backups: b.max_backups,
                    max_backup_age: b.max_backup_age.map(std::time::Duration::from_secs),
                    incremental_enabled: b.incremental_enabled,
                    full_backup_interval: std::time::Duration::from_secs(86400), // 1 day
                }
            }),
            performance: llmspell_kernel::state::config::PerformanceConfig::default(),
        };

        // Create state manager
        let state_manager = StateManager::with_backend(
            persistence_config.backend_type.clone(),
            persistence_config,
            None,
        )
        .await?;

        Ok(Self {
            state_manager: Arc::new(state_manager),
            default_scope: StateScope::Global,
        })
    }

    /// Create an in-memory adapter (useful for testing)
    ///
    /// # Errors
    ///
    /// Returns an error if the state manager fails to initialize
    pub async fn in_memory() -> anyhow::Result<Self> {
        let state_manager = StateManager::new(None).await?;
        Ok(Self {
            state_manager: Arc::new(state_manager),
            default_scope: StateScope::Global,
        })
    }

    /// Set the default scope for operations
    #[must_use]
    pub fn with_scope(mut self, scope: StateScope) -> Self {
        self.default_scope = scope;
        self
    }

    /// Get the underlying state manager (for advanced operations)
    #[must_use]
    pub const fn state_manager(&self) -> &Arc<StateManager> {
        &self.state_manager
    }
}

#[async_trait]
impl StateAccess for StateManagerAdapter {
    async fn read(&self, key: &str) -> Result<Option<Value>> {
        debug!(
            "StateManagerAdapter: Reading key '{}' from scope {:?}",
            key, self.default_scope
        );
        self.state_manager
            .get(self.default_scope.clone(), key)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to read state: {e}"),
                operation: Some("read".to_string()),
                source: None,
            })
    }

    async fn write(&self, key: &str, value: Value) -> Result<()> {
        debug!(
            "StateManagerAdapter: Writing key '{}' to scope {:?}",
            key, self.default_scope
        );
        self.state_manager
            .set(self.default_scope.clone(), key, value)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to write state: {e}"),
                operation: Some("write".to_string()),
                source: None,
            })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        debug!(
            "StateManagerAdapter: Deleting key '{}' from scope {:?}",
            key, self.default_scope
        );
        self.state_manager
            .delete(self.default_scope.clone(), key)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to delete state: {e}"),
                operation: Some("delete".to_string()),
                source: None,
            })
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        debug!(
            "StateManagerAdapter: Listing keys with prefix '{}' in scope {:?}",
            prefix, self.default_scope
        );
        let all_keys = self
            .state_manager
            .list_keys(self.default_scope.clone())
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to list keys: {e}"),
                operation: Some("list_keys".to_string()),
                source: None,
            })?;

        // Filter by prefix
        Ok(all_keys
            .into_iter()
            .filter(|k| k.starts_with(prefix))
            .collect())
    }
}

/// Builder for creating `StateManagerAdapter` with custom configuration
pub struct StateManagerAdapterBuilder {
    backend_type: StorageBackendType,
    scope: StateScope,
    persistence_config: Option<PersistenceConfig>,
}

impl StateManagerAdapterBuilder {
    /// Create a new builder with memory backend
    #[must_use]
    pub const fn new() -> Self {
        Self {
            backend_type: StorageBackendType::Memory,
            scope: StateScope::Global,
            persistence_config: None,
        }
    }

    /// Set the storage backend type
    #[must_use]
    pub fn backend(mut self, backend: StorageBackendType) -> Self {
        self.backend_type = backend;
        self
    }

    /// Set the default scope
    #[must_use]
    pub fn scope(mut self, scope: StateScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set custom persistence configuration
    #[must_use]
    pub fn persistence_config(mut self, config: PersistenceConfig) -> Self {
        self.persistence_config = Some(config);
        self
    }

    /// Build the adapter
    ///
    /// # Errors
    ///
    /// Returns an error if the state manager fails to initialize
    pub async fn build(self) -> anyhow::Result<StateManagerAdapter> {
        let persistence_config = self
            .persistence_config
            .unwrap_or_else(|| PersistenceConfig {
                enabled: true,
                backend_type: self.backend_type.clone(),
                ..Default::default()
            });

        let state_manager =
            StateManager::with_backend(self.backend_type, persistence_config, None).await?;

        Ok(StateManagerAdapter {
            state_manager: Arc::new(state_manager),
            default_scope: self.scope,
        })
    }
}

impl Default for StateManagerAdapterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// A no-prefix adapter that passes keys through without any scope prefix
/// Used for workflow state where keys already contain the full path
#[derive(Clone)]
pub struct NoScopeStateAdapter {
    state_manager: Arc<StateManager>,
}

impl NoScopeStateAdapter {
    /// Create a new no-scope adapter
    pub const fn new(state_manager: Arc<StateManager>) -> Self {
        Self { state_manager }
    }
}

impl fmt::Debug for NoScopeStateAdapter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NoScopeStateAdapter")
            .field("state_manager", &"Arc<StateManager>")
            .finish()
    }
}

#[async_trait]
impl StateAccess for NoScopeStateAdapter {
    async fn read(&self, key: &str) -> Result<Option<Value>> {
        use tracing::info;
        info!(
            "NoScopeStateAdapter: Reading key '{}' with Custom(\"\") scope",
            key
        );
        info!(
            "NoScopeStateAdapter: Full storage key will be: custom::{}",
            key
        );

        // Use Custom scope with empty string - this will add "custom::" prefix
        self.state_manager
            .get(StateScope::Custom(String::new()), key)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to read state: {e}"),
                operation: Some("read".to_string()),
                source: None,
            })
    }

    async fn write(&self, key: &str, value: Value) -> Result<()> {
        use tracing::info;
        info!(
            "NoScopeStateAdapter: Writing key '{}' with Custom(\"\") scope",
            key
        );
        info!(
            "NoScopeStateAdapter: Full storage key will be: custom::{}",
            key
        );

        // Use Custom scope with empty string to minimize prefix
        self.state_manager
            .set(StateScope::Custom(String::new()), key, value)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to write state: {e}"),
                operation: Some("write".to_string()),
                source: None,
            })
    }

    async fn delete(&self, key: &str) -> Result<bool> {
        debug!(
            "NoScopeStateAdapter: Deleting key '{}' without any scope prefix",
            key
        );

        self.state_manager
            .delete(StateScope::Custom(String::new()), key)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to delete state: {e}"),
                operation: Some("delete".to_string()),
                source: None,
            })
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        debug!("NoScopeStateAdapter: Listing keys with prefix '{}'", prefix);

        // List all keys in the Custom("") scope
        let all_keys = self
            .state_manager
            .list_keys(StateScope::Custom(String::new()))
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to list keys: {e}"),
                operation: Some("list_keys".to_string()),
                source: None,
            })?;

        // Filter by prefix and remove the "custom::" prefix that StateManager adds
        let filtered_keys: Vec<String> = all_keys
            .into_iter()
            .filter_map(|key| {
                // Remove "custom::" prefix if present
                let clean_key = key.strip_prefix("custom::").unwrap_or(&key);

                // Check if it matches our prefix
                if clean_key.starts_with(prefix) {
                    Some(clean_key.to_string())
                } else {
                    None
                }
            })
            .collect();

        debug!(
            "NoScopeStateAdapter: Found {} keys matching prefix '{}'",
            filtered_keys.len(),
            prefix
        );

        Ok(filtered_keys)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_manager_adapter_basic_operations() {
        let adapter = StateManagerAdapter::in_memory().await.unwrap();

        // Test write
        adapter
            .write("test_key", serde_json::json!({"value": 42}))
            .await
            .unwrap();

        // Test read
        let value = adapter.read("test_key").await.unwrap();
        assert_eq!(value, Some(serde_json::json!({"value": 42})));

        // Test delete
        let deleted = adapter.delete("test_key").await.unwrap();
        assert!(deleted);

        // Verify deletion
        let value = adapter.read("test_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_state_manager_adapter_list_keys() {
        let adapter = StateManagerAdapter::in_memory().await.unwrap();

        // Write multiple keys
        adapter
            .write("workflow:123:step1", serde_json::json!("data1"))
            .await
            .unwrap();
        adapter
            .write("workflow:123:step2", serde_json::json!("data2"))
            .await
            .unwrap();
        adapter
            .write("other:key", serde_json::json!("other"))
            .await
            .unwrap();

        // List with prefix
        let keys = adapter.list_keys("workflow:123").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"workflow:123:step1".to_string()));
        assert!(keys.contains(&"workflow:123:step2".to_string()));
    }

    #[tokio::test]
    async fn test_state_manager_adapter_with_scope() {
        let adapter = StateManagerAdapter::in_memory()
            .await
            .unwrap()
            .with_scope(StateScope::Agent("agent-123".to_string()));

        // Operations should use agent scope
        adapter
            .write("config", serde_json::json!({"model": "gpt-4"}))
            .await
            .unwrap();

        let value = adapter.read("config").await.unwrap();
        assert_eq!(value, Some(serde_json::json!({"model": "gpt-4"})));
    }
}
