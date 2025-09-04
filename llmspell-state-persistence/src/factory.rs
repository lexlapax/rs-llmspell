//! Factory for creating shared StateManager instances from configuration
//!
//! This factory ensures a single StateManager instance is created and shared
//! between the kernel and ScriptRuntime, preventing file lock conflicts.

use crate::config::{PersistenceConfig, RocksDBConfig, SledConfig, StorageBackendType};
use crate::manager::StateManager;
use llmspell_config::LLMSpellConfig;
use llmspell_state_traits::StateError;
use std::path::PathBuf;
use std::sync::Arc;

/// Factory for creating shared StateManager instances
pub struct StateFactory;

impl StateFactory {
    /// Create a shared StateManager from LLMSpellConfig
    ///
    /// Returns None if state persistence is disabled in the config.
    /// Returns Some(Arc<StateManager>) if state persistence is enabled.
    pub async fn create_from_config(
        config: &LLMSpellConfig,
    ) -> Result<Option<Arc<StateManager>>, StateError> {
        // Check if state persistence is enabled
        if !config.runtime.state_persistence.enabled {
            return Ok(None);
        }

        // Convert backend type string to enum
        let backend_type = match config.runtime.state_persistence.backend_type.as_str() {
            "memory" => StorageBackendType::Memory,
            "sled" => {
                // Create default Sled config for file-based storage
                StorageBackendType::Sled(SledConfig {
                    path: PathBuf::from(".llmspell/state"),
                    cache_capacity: 1024 * 1024 * 1024, // 1GB
                    use_compression: true,
                })
            }
            "rocksdb" => {
                // Create default RocksDB config
                StorageBackendType::RocksDB(RocksDBConfig {
                    path: PathBuf::from(".llmspell/state"),
                    create_if_missing: true,
                    optimize_for_point_lookup: false,
                })
            }
            _ => {
                // Default to memory for unsupported types
                StorageBackendType::Memory
            }
        };

        // Create a simplified persistence configuration
        // Most fields from llmspell-config are not directly mappable
        let persistence_config = PersistenceConfig {
            enabled: config.runtime.state_persistence.enabled,
            backend_type: backend_type.clone(),
            ..Default::default()
        };

        // Create the StateManager with the configured backend
        let state_manager = StateManager::with_backend(backend_type, persistence_config).await?;

        Ok(Some(Arc::new(state_manager)))
    }

    /// Create a shared StateManager with specific backend type
    ///
    /// This method allows direct backend specification for testing purposes.
    pub async fn create_with_backend(
        backend_type: StorageBackendType,
        config: PersistenceConfig,
    ) -> Result<Arc<StateManager>, StateError> {
        let state_manager = StateManager::with_backend(backend_type, config).await?;
        Ok(Arc::new(state_manager))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_creates_memory_backend() {
        let config = LLMSpellConfig::builder()
            .runtime(
                llmspell_config::GlobalRuntimeConfig::builder()
                    .state_persistence(llmspell_config::StatePersistenceConfig {
                        enabled: true,
                        backend_type: "memory".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build();

        let result = StateFactory::create_from_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[tokio::test]
    async fn test_factory_returns_none_when_disabled() {
        let config = LLMSpellConfig::builder()
            .runtime(
                llmspell_config::GlobalRuntimeConfig::builder()
                    .state_persistence(llmspell_config::StatePersistenceConfig {
                        enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build();

        let result = StateFactory::create_from_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_factory_handles_unknown_backend() {
        let config = LLMSpellConfig::builder()
            .runtime(
                llmspell_config::GlobalRuntimeConfig::builder()
                    .state_persistence(llmspell_config::StatePersistenceConfig {
                        enabled: true,
                        backend_type: "unknown_backend".to_string(),
                        ..Default::default()
                    })
                    .build(),
            )
            .build();

        // Should default to memory backend instead of error
        let result = StateFactory::create_from_config(&config).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }
}
