//! ABOUTME: Storage backend traits and types
//! ABOUTME: Defines the unified storage interface for all backends

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of storage backend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StorageBackendType {
    /// In-memory storage (for testing/development)
    Memory,

    /// Sled embedded database
    Sled,

    /// RocksDB embedded database
    RocksDB,
}

/// Storage backend characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageCharacteristics {
    /// Whether the backend persists data
    pub persistent: bool,

    /// Whether the backend supports transactions
    pub transactional: bool,

    /// Whether the backend supports key prefix scanning
    pub supports_prefix_scan: bool,

    /// Whether the backend supports atomic operations
    pub supports_atomic_ops: bool,

    /// Estimated read latency in microseconds
    pub avg_read_latency_us: u64,

    /// Estimated write latency in microseconds
    pub avg_write_latency_us: u64,
}

/// Unified storage backend trait
#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Get a value by key
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Set a key-value pair
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;

    /// Delete a key
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists
    async fn exists(&self, key: &str) -> Result<bool>;

    /// List all keys with a given prefix
    async fn list_keys(&self, prefix: &str) -> Result<Vec<String>>;

    /// Get multiple values by keys
    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;

    /// Set multiple key-value pairs
    async fn set_batch(&self, items: HashMap<String, Vec<u8>>) -> Result<()>;

    /// Delete multiple keys
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;

    /// Clear all data (use with caution)
    async fn clear(&self) -> Result<()>;

    /// Get the backend type
    fn backend_type(&self) -> StorageBackendType;

    /// Get backend characteristics
    fn characteristics(&self) -> StorageCharacteristics;
}

/// Helper trait for serialization/deserialization
pub trait StorageSerialize: Sized {
    fn to_storage_bytes(&self) -> Result<Vec<u8>>;
    fn from_storage_bytes(bytes: &[u8]) -> Result<Self>;
}

/// Default implementation for serde types
impl<T> StorageSerialize for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn to_storage_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }

    fn from_storage_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(serde_json::from_slice(bytes)?)
    }
}
