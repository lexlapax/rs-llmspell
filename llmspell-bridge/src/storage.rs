//! ABOUTME: Storage backend discovery for state persistence
//! ABOUTME: Provides discovery of available storage backends (Memory, Sled, RocksDB)

use crate::discovery::BridgeDiscovery;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a storage backend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    /// Backend name
    pub name: String,
    /// Description of the backend
    pub description: String,
    /// Backend type identifier
    pub backend_type: String,
    /// Whether backend is persistent across restarts
    pub persistent: bool,
    /// Whether backend supports compression
    pub supports_compression: bool,
    /// Whether backend supports encryption
    pub supports_encryption: bool,
    /// Performance characteristics
    pub performance: StoragePerformance,
    /// Required configuration parameters
    pub required_params: Vec<String>,
    /// Optional configuration parameters
    pub optional_params: Vec<String>,
}

/// Storage performance characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePerformance {
    /// Read latency category (low, medium, high)
    pub read_latency: String,
    /// Write latency category (low, medium, high)
    pub write_latency: String,
    /// Throughput category (low, medium, high)
    pub throughput: String,
    /// Suitable for large datasets
    pub large_dataset_suitable: bool,
}

/// Storage backend discovery service
pub struct StorageDiscovery {
    /// Available storage backends
    backends: HashMap<String, StorageInfo>,
}

impl StorageDiscovery {
    /// Create a new storage discovery service
    pub fn new() -> Self {
        let mut backends = HashMap::new();

        // Memory backend
        backends.insert(
            "memory".to_string(),
            StorageInfo {
                name: "memory".to_string(),
                description: "In-memory storage backend with fastest performance".to_string(),
                backend_type: "Memory".to_string(),
                persistent: false,
                supports_compression: true,
                supports_encryption: true,
                performance: StoragePerformance {
                    read_latency: "low".to_string(),
                    write_latency: "low".to_string(),
                    throughput: "high".to_string(),
                    large_dataset_suitable: false,
                },
                required_params: vec![],
                optional_params: vec!["max_size".to_string(), "eviction_policy".to_string()],
            },
        );

        // Sled backend
        backends.insert(
            "sled".to_string(),
            StorageInfo {
                name: "sled".to_string(),
                description: "Embedded database with crash-resistant storage".to_string(),
                backend_type: "Sled".to_string(),
                persistent: true,
                supports_compression: true,
                supports_encryption: true,
                performance: StoragePerformance {
                    read_latency: "medium".to_string(),
                    write_latency: "medium".to_string(),
                    throughput: "medium".to_string(),
                    large_dataset_suitable: true,
                },
                required_params: vec!["path".to_string()],
                optional_params: vec![
                    "cache_capacity".to_string(),
                    "use_compression".to_string(),
                    "mode".to_string(),
                ],
            },
        );

        // RocksDB backend
        backends.insert(
            "rocksdb".to_string(),
            StorageInfo {
                name: "rocksdb".to_string(),
                description: "High-performance embedded database for large datasets".to_string(),
                backend_type: "RocksDB".to_string(),
                persistent: true,
                supports_compression: true,
                supports_encryption: true,
                performance: StoragePerformance {
                    read_latency: "low".to_string(),
                    write_latency: "medium".to_string(),
                    throughput: "high".to_string(),
                    large_dataset_suitable: true,
                },
                required_params: vec!["path".to_string()],
                optional_params: vec![
                    "create_if_missing".to_string(),
                    "optimize_for_point_lookup".to_string(),
                    "block_cache_size".to_string(),
                    "write_buffer_size".to_string(),
                ],
            },
        );

        Self { backends }
    }

    /// Get information about a specific storage backend
    pub fn get_backend_info(&self, backend_name: &str) -> Option<StorageInfo> {
        self.backends.get(backend_name).cloned()
    }

    /// List all available backend names
    pub fn list_backend_names(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    /// Get backends by persistence characteristic
    pub fn get_persistent_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.persistent)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get backends suitable for large datasets
    pub fn get_large_dataset_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.performance.large_dataset_suitable)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}

impl Default for StorageDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of unified BridgeDiscovery trait for StorageDiscovery
#[async_trait::async_trait]
impl BridgeDiscovery<StorageInfo> for StorageDiscovery {
    async fn discover_types(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<StorageInfo> {
        self.get_backend_info(type_name)
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.backends.contains_key(type_name)
    }

    async fn list_types(&self) -> Vec<String> {
        self.list_backend_names()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, StorageInfo)>
    where
        F: Fn(&str, &StorageInfo) -> bool + Send,
    {
        self.backends
            .iter()
            .filter(|(name, info)| predicate(name, info))
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_discovery() {
        let discovery = StorageDiscovery::new();

        // Test listing backends
        let backends = discovery.list_backend_names();
        assert_eq!(backends.len(), 3);
        assert!(backends.contains(&"memory".to_string()));
        assert!(backends.contains(&"sled".to_string()));
        assert!(backends.contains(&"rocksdb".to_string()));

        // Test getting backend info
        let memory_info = discovery.get_backend_info("memory").unwrap();
        assert_eq!(memory_info.name, "memory");
        assert!(!memory_info.persistent);
        assert_eq!(memory_info.performance.read_latency, "low");

        // Test persistent backends
        let persistent = discovery.get_persistent_backends();
        assert_eq!(persistent.len(), 2);

        // Test large dataset backends
        let large_dataset = discovery.get_large_dataset_backends();
        assert_eq!(large_dataset.len(), 2);
    }

    #[tokio::test]
    async fn test_storage_bridge_discovery() {
        let discovery = StorageDiscovery::new();

        // Test discover_types
        let types = discovery.discover_types().await;
        assert_eq!(types.len(), 3);

        // Test get_type_info
        let sled_info = discovery.get_type_info("sled").await.unwrap();
        assert_eq!(sled_info.backend_type, "Sled");
        assert!(sled_info.persistent);

        // Test has_type
        assert!(discovery.has_type("memory").await);
        assert!(discovery.has_type("sled").await);
        assert!(!discovery.has_type("redis").await);

        // Test filter_types
        let high_perf = discovery
            .filter_types(|_, info| info.performance.throughput == "high")
            .await;
        assert_eq!(high_perf.len(), 2); // memory and rocksdb
    }
}
