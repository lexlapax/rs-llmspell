// ABOUTME: Configuration types for persistent state management
// ABOUTME: Defines persistence configuration and state schema structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistenceConfig {
    pub enabled: bool,
    pub backend_type: StorageBackendType,
    pub flush_interval: Duration,
    pub compression: bool,
    pub encryption: Option<EncryptionConfig>,
    pub backup_retention: Duration,
    pub performance: PerformanceConfig,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Disabled by default for backward compatibility
            backend_type: StorageBackendType::Memory,
            flush_interval: Duration::from_secs(5),
            compression: true,
            encryption: None,
            backup_retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            performance: PerformanceConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    Memory,
    Sled(SledConfig),
    RocksDB(RocksDBConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SledConfig {
    pub path: std::path::PathBuf,
    pub cache_capacity: u64,
    pub use_compression: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RocksDBConfig {
    pub path: std::path::PathBuf,
    pub create_if_missing: bool,
    pub optimize_for_point_lookup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: EncryptionAlgorithm,
    pub key_derivation: KeyDerivationConfig,
    pub rotation_interval: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
    ChaCha20Poly1305,
    AES256GCM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationConfig {
    pub method: String,
    pub iterations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub batch_size: usize,
    pub flush_interval: Duration,
    pub immediate_flush_patterns: Vec<String>,
    pub cache_size_limit: usize,
    pub compression_threshold: usize,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval: Duration::from_secs(5),
            immediate_flush_patterns: vec!["critical:*".to_string()],
            cache_size_limit: 10_000,
            compression_threshold: 1024, // Compress values larger than 1KB
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSchema {
    pub version: u32,
    pub hash: String,
    pub fields: HashMap<String, FieldSchema>,
    pub compatibility: CompatibilityLevel,
    pub migration_path: Vec<MigrationStep>,
}

impl StateSchema {
    pub fn v1() -> Self {
        Self {
            version: 1,
            hash: "initial".to_string(),
            fields: HashMap::new(),
            compatibility: CompatibilityLevel::BackwardCompatible,
            migration_path: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FieldSchema {
    pub field_type: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompatibilityLevel {
    BackwardCompatible,
    ForwardCompatible,
    BreakingChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MigrationStep {
    pub from_version: u32,
    pub to_version: u32,
    pub migration_type: String,
    pub description: String,
}
