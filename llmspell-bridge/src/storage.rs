//! ABOUTME: Storage backend discovery for state persistence
//! ABOUTME: Provides discovery of available storage backends (`Memory`, `Postgres`)

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
    #[must_use]
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

        // Postgres backend
        backends.insert(
            "postgres".to_string(),
            StorageInfo {
                name: "postgres".to_string(),
                description:
                    "High-performance production database for multi-tenant and large datasets"
                        .to_string(),
                backend_type: "Postgres".to_string(),
                persistent: true,
                supports_compression: true,
                supports_encryption: true,
                performance: StoragePerformance {
                    read_latency: "low".to_string(),
                    write_latency: "low".to_string(),
                    throughput: "high".to_string(),
                    large_dataset_suitable: true,
                },
                required_params: vec!["connection_string".to_string()],
                optional_params: vec![
                    "pool_size".to_string(),
                    "timeout_ms".to_string(),
                    "max_connections".to_string(),
                    "idle_timeout".to_string(),
                ],
            },
        );

        Self { backends }
    }

    /// Get information about a specific storage backend
    #[must_use]
    pub fn get_backend_info(&self, backend_name: &str) -> Option<StorageInfo> {
        self.backends.get(backend_name).cloned()
    }

    /// List all available backend names
    #[must_use]
    pub fn list_backend_names(&self) -> Vec<String> {
        self.backends.keys().cloned().collect()
    }

    /// Get backends by persistence characteristic
    #[must_use]
    pub fn get_persistent_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.persistent)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get backends suitable for large datasets
    #[must_use]
    pub fn get_large_dataset_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.performance.large_dataset_suitable)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get backends that support compression
    #[must_use]
    pub fn get_compression_enabled_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.supports_compression)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get backends that support encryption
    #[must_use]
    pub fn get_encryption_enabled_backends(&self) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| info.supports_encryption)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get backends by performance characteristics
    #[must_use]
    pub fn get_backends_by_performance(
        &self,
        latency: &str,
        throughput: &str,
    ) -> Vec<(String, StorageInfo)> {
        self.backends
            .iter()
            .filter(|(_, info)| {
                info.performance.read_latency == latency
                    && info.performance.throughput == throughput
            })
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}

impl Default for StorageDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of unified `BridgeDiscovery` trait for `StorageDiscovery`
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

/// Configuration for storage backend selection and setup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Selected backend name (memory, postgres)
    pub backend: String,
    /// Backend-specific configuration parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Enable compression if backend supports it
    pub enable_compression: bool,
    /// Enable encryption if backend supports it
    pub enable_encryption: bool,
    /// Performance optimization preset (`fast`, `balanced`, `storage_optimized`)
    pub performance_preset: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: "memory".to_string(),
            parameters: HashMap::new(),
            enable_compression: false,
            enable_encryption: false,
            performance_preset: "balanced".to_string(),
        }
    }
}

impl StorageConfig {
    /// Create a new builder for `StorageConfig`
    #[must_use]
    pub fn builder() -> StorageConfigBuilder {
        StorageConfigBuilder::new()
    }
}

/// Builder for `StorageConfig`
#[derive(Debug, Clone, Default)]
pub struct StorageConfigBuilder {
    config: StorageConfig,
}

impl StorageConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the backend type
    #[must_use]
    pub fn backend(mut self, backend: impl Into<String>) -> Self {
        self.config.backend = backend.into();
        self
    }

    /// Add a configuration parameter
    #[must_use]
    pub fn parameter(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.parameters.insert(key.into(), value);
        self
    }

    /// Set multiple parameters at once
    #[must_use]
    pub fn parameters(mut self, parameters: HashMap<String, serde_json::Value>) -> Self {
        self.config.parameters = parameters;
        self
    }

    /// Enable compression
    #[must_use]
    pub const fn enable_compression(mut self, enable: bool) -> Self {
        self.config.enable_compression = enable;
        self
    }

    /// Enable encryption
    #[must_use]
    pub const fn enable_encryption(mut self, enable: bool) -> Self {
        self.config.enable_encryption = enable;
        self
    }

    /// Set performance preset
    #[must_use]
    pub fn performance_preset(mut self, preset: impl Into<String>) -> Self {
        self.config.performance_preset = preset.into();
        self
    }

    /// Convenience method to configure for memory backend
    #[must_use]
    pub fn memory_backend(mut self) -> Self {
        self.config.backend = "memory".to_string();
        self
    }

    /// Convenience method to configure for postgres backend
    #[must_use]
    pub fn postgres_backend(mut self, connection_string: impl Into<String>) -> Self {
        self.config.backend = "postgres".to_string();
        self.config.parameters.insert(
            "connection_string".to_string(),
            serde_json::Value::String(connection_string.into()),
        );
        self
    }

    /// Build the final `StorageConfig`
    #[must_use]
    pub fn build(self) -> StorageConfig {
        self.config
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
        assert_eq!(backends.len(), 2);
        assert!(backends.contains(&"memory".to_string()));
        assert!(backends.contains(&"postgres".to_string()));

        // Test getting backend info
        let memory_info = discovery.get_backend_info("memory").unwrap();
        assert_eq!(memory_info.name, "memory");
        assert!(!memory_info.persistent);
        assert_eq!(memory_info.performance.read_latency, "low");

        // Test persistent backends
        let persistent = discovery.get_persistent_backends();
        assert_eq!(persistent.len(), 1);

        // Test large dataset backends
        let large_dataset = discovery.get_large_dataset_backends();
        assert_eq!(large_dataset.len(), 1);

        // Test compression-enabled backends
        let compression_backends = discovery.get_compression_enabled_backends();
        assert_eq!(compression_backends.len(), 2); // All backends support compression

        // Test encryption-enabled backends
        let encryption_backends = discovery.get_encryption_enabled_backends();
        assert_eq!(encryption_backends.len(), 2); // All backends support encryption

        // Test performance-based filtering
        let high_perf = discovery.get_backends_by_performance("low", "high");
        assert_eq!(high_perf.len(), 2); // memory and postgres
    }

    #[tokio::test]
    async fn test_storage_bridge_discovery() {
        let discovery = StorageDiscovery::new();

        // Test discover_types
        let types = discovery.discover_types().await;
        assert_eq!(types.len(), 2);

        // Test get_type_info
        let postgres_info = discovery.get_type_info("postgres").await.unwrap();
        assert_eq!(postgres_info.backend_type, "Postgres");
        assert!(postgres_info.persistent);

        // Test has_type
        assert!(discovery.has_type("memory").await);
        assert!(discovery.has_type("postgres").await);
        assert!(!discovery.has_type("redis").await);

        // Test filter_types
        let high_perf = discovery
            .filter_types(|_, info| info.performance.throughput == "high")
            .await;
        assert_eq!(high_perf.len(), 2); // memory and postgres
    }

    #[test]
    fn test_storage_config_builder() {
        // Test default config
        let default_config = StorageConfig::default();
        assert_eq!(default_config.backend, "memory");
        assert!(!default_config.enable_compression);
        assert!(!default_config.enable_encryption);

        // Test builder pattern
        let config = StorageConfig::builder()
            .postgres_backend("postgresql://user:pass@localhost/test")
            .enable_compression(true)
            .enable_encryption(true)
            .performance_preset("fast")
            .parameter(
                "pool_size",
                serde_json::Value::Number(serde_json::Number::from(20)),
            )
            .build();

        assert_eq!(config.backend, "postgres");
        assert!(config.enable_compression);
        assert!(config.enable_encryption);
        assert_eq!(config.performance_preset, "fast");
        assert_eq!(
            config.parameters.get("connection_string"),
            Some(&serde_json::Value::String(
                "postgresql://user:pass@localhost/test".to_string()
            ))
        );
        assert_eq!(
            config.parameters.get("pool_size"),
            Some(&serde_json::Value::Number(serde_json::Number::from(20)))
        );

        // Test convenience methods
        let memory_config = StorageConfig::builder().memory_backend().build();
        assert_eq!(memory_config.backend, "memory");

        let postgres_config = StorageConfig::builder()
            .postgres_backend("postgresql://localhost/db")
            .build();
        assert_eq!(postgres_config.backend, "postgres");
        assert_eq!(
            postgres_config.parameters.get("connection_string"),
            Some(&serde_json::Value::String(
                "postgresql://localhost/db".to_string()
            ))
        );
    }
}
