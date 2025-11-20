//! ABOUTME: In-memory storage backend implementation
//! ABOUTME: Provides fast non-persistent storage for testing and development

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::storage::StorageBackend;
use llmspell_core::types::storage::{StorageBackendType, StorageCharacteristics};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory storage backend
#[derive(Debug)]
pub struct MemoryBackend {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl MemoryBackend {
    /// Create new in-memory backend
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryBackend {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for MemoryBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.read().await;
        Ok(data.get(key).cloned())
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        let mut data = self.data.write().await;
        data.insert(key.to_string(), value);
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut data = self.data.write().await;
        data.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let data = self.data.read().await;
        Ok(data.contains_key(key))
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let data = self.data.read().await;
        let keys: Vec<String> = data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();
        Ok(keys)
    }

    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>> {
        let data = self.data.read().await;
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = data.get(key) {
                result.insert(key.clone(), value.clone());
            }
        }

        Ok(result)
    }

    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()> {
        let mut data = self.data.write().await;
        for (key, value) in items {
            data.insert(key, value);
        }
        Ok(())
    }

    async fn delete_batch(&self, keys: &[String]) -> Result<()> {
        let mut data = self.data.write().await;
        for key in keys {
            data.remove(key);
        }
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().await;
        data.clear();
        Ok(())
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Memory
    }

    fn characteristics(&self) -> StorageCharacteristics {
        StorageCharacteristics {
            persistent: false,
            transactional: false,
            supports_prefix_scan: true,
            supports_atomic_ops: false,
            avg_read_latency_us: 1,
            avg_write_latency_us: 1,
        }
    }

    async fn run_migrations(&self) -> Result<()> {
        // No migrations needed for in-memory backend
        Ok(())
    }

    async fn migration_version(&self) -> Result<usize> {
        // No migrations for in-memory backend
        Ok(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_memory_backend_basic_operations() {
        let backend = MemoryBackend::new();

        // Test set and get
        backend.set("key1", b"value1".to_vec()).await.unwrap();
        let value = backend.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // Test exists
        assert!(backend.exists("key1").await.unwrap());
        assert!(!backend.exists("key2").await.unwrap());

        // Test delete
        backend.delete("key1").await.unwrap();
        assert!(!backend.exists("key1").await.unwrap());
    }
    #[tokio::test]
    async fn test_memory_backend_batch_operations() {
        let backend = MemoryBackend::new();

        // Test batch set
        let mut items = HashMap::new();
        items.insert("key1".to_string(), b"value1".to_vec());
        items.insert("key2".to_string(), b"value2".to_vec());
        backend.set_batch(items).await.unwrap();

        // Test batch get
        let keys = vec!["key1".to_string(), "key2".to_string(), "key3".to_string()];
        let values = backend.get_batch(&keys).await.unwrap();
        assert_eq!(values.len(), 2);
        assert_eq!(values.get("key1"), Some(&b"value1".to_vec()));
        assert_eq!(values.get("key2"), Some(&b"value2".to_vec()));

        // Test batch delete
        backend.delete_batch(&keys[..2]).await.unwrap();
        assert!(!backend.exists("key1").await.unwrap());
        assert!(!backend.exists("key2").await.unwrap());
    }
    #[tokio::test]
    async fn test_memory_backend_prefix_scan() {
        let backend = MemoryBackend::new();

        // Set some keys with common prefix
        backend.set("user:1", b"Alice".to_vec()).await.unwrap();
        backend.set("user:2", b"Bob".to_vec()).await.unwrap();
        backend.set("post:1", b"Hello".to_vec()).await.unwrap();

        // Test prefix scan
        let user_keys = backend.list_keys("user:").await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user:1".to_string()));
        assert!(user_keys.contains(&"user:2".to_string()));

        let post_keys = backend.list_keys("post:").await.unwrap();
        assert_eq!(post_keys.len(), 1);
        assert!(post_keys.contains(&"post:1".to_string()));
    }
}
