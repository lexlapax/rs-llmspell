// ABOUTME: Storage backend adapter for state persistence
// ABOUTME: Integrates with Phase 3.3 llmspell-storage infrastructure

use crate::config::StorageBackendType;
use crate::performance::UnifiedSerializer;
use llmspell_state_traits::{StateError, StateResult};
use llmspell_storage::{StorageBackend, StorageSerialize};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Creates appropriate storage backend based on configuration
pub async fn create_storage_backend(
    backend_type: &StorageBackendType,
) -> StateResult<Arc<dyn StorageBackend>> {
    match backend_type {
        StorageBackendType::Memory => {
            let backend = llmspell_storage::MemoryBackend::new();
            Ok(Arc::new(backend) as Arc<dyn StorageBackend>)
        }
        StorageBackendType::Sled(config) => {
            let backend = llmspell_storage::SledBackend::new_with_path(&config.path)
                .map_err(|e| StateError::storage(e.to_string()))?;
            Ok(Arc::new(backend) as Arc<dyn StorageBackend>)
        }
        StorageBackendType::RocksDB(_config) => {
            // RocksDB backend to be implemented in future phase
            Err(StateError::storage(
                "RocksDB backend not yet implemented".to_string(),
            ))
        }
    }
}

/// Wrapper for state-specific storage operations
pub struct StateStorageAdapter {
    backend: Arc<dyn StorageBackend>,
    namespace: String,
    fast_serializer: UnifiedSerializer,
}

impl StateStorageAdapter {
    pub fn new(backend: Arc<dyn StorageBackend>, namespace: String) -> Self {
        Self {
            backend,
            namespace,
            fast_serializer: UnifiedSerializer::fast(),
        }
    }

    /// Store a value with state-specific key formatting
    pub async fn store<T: StorageSerialize>(&self, key: &str, value: &T) -> StateResult<()> {
        let namespaced_key = self.make_key(key);
        let bytes = value
            .to_storage_bytes()
            .map_err(|e| StateError::serialization(e.to_string()))?;

        self.backend
            .set(&namespaced_key, bytes)
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }

    /// Load a value with state-specific key formatting
    pub async fn load<T: StorageSerialize>(&self, key: &str) -> StateResult<Option<T>> {
        let namespaced_key = self.make_key(key);

        match self.backend.get(&namespaced_key).await {
            Ok(Some(bytes)) => {
                let value = T::from_storage_bytes(&bytes)
                    .map_err(|e| StateError::serialization(e.to_string()))?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StateError::storage(e.to_string())),
        }
    }

    /// Delete a value
    pub async fn delete(&self, key: &str) -> StateResult<()> {
        let namespaced_key = self.make_key(key);
        self.backend
            .delete(&namespaced_key)
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }

    /// Check if a key exists
    pub async fn exists(&self, key: &str) -> StateResult<bool> {
        let namespaced_key = self.make_key(key);
        self.backend
            .exists(&namespaced_key)
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }

    /// List all keys in the namespace
    pub async fn list_keys(&self, prefix: &str) -> StateResult<Vec<String>> {
        let namespaced_prefix = self.make_key(prefix);
        let keys = self
            .backend
            .list_keys(&namespaced_prefix)
            .await
            .map_err(|e| StateError::storage(e.to_string()))?;

        // Remove namespace prefix from keys
        let prefix_len = self.namespace.len() + 1;
        Ok(keys
            .into_iter()
            .filter_map(|k| {
                if k.len() > prefix_len {
                    Some(k[prefix_len..].to_string())
                } else {
                    None
                }
            })
            .collect())
    }

    /// Clear all data in the namespace
    pub async fn clear_namespace(&self) -> StateResult<()> {
        let keys = self.list_keys("").await?;
        let namespaced_keys: Vec<_> = keys.iter().map(|k| self.make_key(k)).collect();

        self.backend
            .delete_batch(&namespaced_keys)
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }

    fn make_key(&self, key: &str) -> String {
        format!("{}:{}", self.namespace, key)
    }

    /// Fast store method using UnifiedSerializer for benchmark data
    pub async fn store_fast<T: Serialize>(&self, key: &str, value: &T) -> StateResult<()> {
        let namespaced_key = self.make_key(key);
        let bytes = self.fast_serializer.serialize(value)?;

        self.backend
            .set(&namespaced_key, bytes)
            .await
            .map_err(|e| StateError::storage(e.to_string()))
    }

    /// Fast load method using UnifiedSerializer
    pub async fn load_fast<T: for<'de> Deserialize<'de>>(
        &self,
        key: &str,
    ) -> StateResult<Option<T>> {
        let namespaced_key = self.make_key(key);

        match self.backend.get(&namespaced_key).await {
            Ok(Some(bytes)) => {
                let value = self.fast_serializer.deserialize(&bytes)?;
                Ok(Some(value))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StateError::storage(e.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct TestData {
        value: String,
        count: u32,
    }
    #[tokio::test]
    async fn test_state_storage_adapter() {
        let backend = create_storage_backend(&StorageBackendType::Memory)
            .await
            .unwrap();
        let adapter = StateStorageAdapter::new(backend, "test".to_string());

        let test_data = TestData {
            value: "test".to_string(),
            count: 42,
        };

        // Store and retrieve
        adapter.store("key1", &test_data).await.unwrap();
        let loaded: Option<TestData> = adapter.load("key1").await.unwrap();
        assert_eq!(loaded, Some(test_data));

        // Delete
        adapter.delete("key1").await.unwrap();
        let deleted: Option<TestData> = adapter.load("key1").await.unwrap();
        assert_eq!(deleted, None);
    }
}
