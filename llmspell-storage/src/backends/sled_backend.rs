//! ABOUTME: Sled storage backend implementation
//! ABOUTME: Provides persistent embedded database storage

use crate::traits::{StorageBackend, StorageBackendType, StorageCharacteristics};
use anyhow::Result;
use async_trait::async_trait;
use sled::Db;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Sled storage backend
pub struct SledBackend {
    db: Arc<Db>,
}

impl SledBackend {
    /// Create new sled backend with default path
    pub fn new() -> Result<Self> {
        let db = sled::open("rs-llmspell.db")?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create new sled backend with custom path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db = sled::open(path)?;
        Ok(Self { db: Arc::new(db) })
    }

    /// Create temporary sled backend (for testing)
    pub fn new_temporary() -> Result<Self> {
        let db = sled::Config::new().temporary(true).open()?;
        Ok(Self { db: Arc::new(db) })
    }
}

#[async_trait]
impl StorageBackend for SledBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(key)?.map(|v| v.to_vec()))
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()> {
        self.db.insert(key, value)?;
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.db.remove(key)?;
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        Ok(self.db.contains_key(key)?)
    }

    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for result in self.db.scan_prefix(prefix) {
            let (key, _) = result?;
            if let Ok(key_str) = String::from_utf8(key.to_vec()) {
                keys.push(key_str);
            }
        }
        Ok(keys)
    }

    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>> {
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = self.db.get(key)? {
                result.insert(key.clone(), value.to_vec());
            }
        }

        Ok(result)
    }

    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()> {
        let mut batch = sled::Batch::default();

        for (key, value) in items {
            batch.insert(key.as_bytes(), value);
        }

        self.db.apply_batch(batch)?;
        Ok(())
    }

    async fn delete_batch(&self, keys: &[String]) -> Result<()> {
        let mut batch = sled::Batch::default();

        for key in keys {
            batch.remove(key.as_bytes());
        }

        self.db.apply_batch(batch)?;
        Ok(())
    }

    async fn clear(&self) -> Result<()> {
        self.db.clear()?;
        Ok(())
    }

    fn backend_type(&self) -> StorageBackendType {
        StorageBackendType::Sled
    }

    fn characteristics(&self) -> StorageCharacteristics {
        StorageCharacteristics {
            persistent: true,
            transactional: true,
            supports_prefix_scan: true,
            supports_atomic_ops: true,
            avg_read_latency_us: 10,
            avg_write_latency_us: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_sled_backend_basic_operations() {
        let backend = SledBackend::new_temporary().unwrap();

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
    async fn test_sled_backend_persistence() {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        // Create and write data
        {
            let backend = SledBackend::new_with_path(&db_path).unwrap();
            backend
                .set("persistent_key", b"persistent_value".to_vec())
                .await
                .unwrap();
        }

        // Reopen and verify data persists
        {
            let backend = SledBackend::new_with_path(&db_path).unwrap();
            let value = backend.get("persistent_key").await.unwrap();
            assert_eq!(value, Some(b"persistent_value".to_vec()));
        }
    }
}
