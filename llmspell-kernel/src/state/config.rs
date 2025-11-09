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
    pub backup: Option<BackupConfig>,
    pub performance: PerformanceConfig,
}

impl Default for PersistenceConfig {
    fn default() -> Self {
        Self {
            enabled: true, // Enabled by default with in-memory backend
            backend_type: StorageBackendType::Memory,
            flush_interval: Duration::from_secs(5),
            compression: true,
            encryption: None,
            backup_retention: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            backup: None,
            performance: PerformanceConfig::default(),
        }
    }
}

impl PersistenceConfig {
    /// Create a new builder for `PersistenceConfig`
    pub fn builder() -> PersistenceConfigBuilder {
        PersistenceConfigBuilder::new()
    }
}

/// Builder for `PersistenceConfig`
#[derive(Debug, Clone)]
pub struct PersistenceConfigBuilder {
    config: PersistenceConfig,
}

impl PersistenceConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: PersistenceConfig::default(),
        }
    }

    /// Enable or disable persistence
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set the storage backend type
    #[must_use]
    pub fn backend_type(mut self, backend_type: StorageBackendType) -> Self {
        self.config.backend_type = backend_type;
        self
    }

    /// Set the flush interval
    #[must_use]
    pub fn flush_interval(mut self, interval: Duration) -> Self {
        self.config.flush_interval = interval;
        self
    }

    /// Enable or disable compression
    #[must_use]
    pub fn compression(mut self, compression: bool) -> Self {
        self.config.compression = compression;
        self
    }

    /// Set the encryption configuration
    #[must_use]
    pub fn encryption(mut self, encryption: Option<EncryptionConfig>) -> Self {
        self.config.encryption = encryption;
        self
    }

    /// Set the backup retention duration
    #[must_use]
    pub fn backup_retention(mut self, retention: Duration) -> Self {
        self.config.backup_retention = retention;
        self
    }

    /// Set the backup configuration
    #[must_use]
    pub fn backup(mut self, backup: Option<BackupConfig>) -> Self {
        self.config.backup = backup;
        self
    }

    /// Set the performance configuration
    #[must_use]
    pub fn performance(mut self, performance: PerformanceConfig) -> Self {
        self.config.performance = performance;
        self
    }

    /// Build the `PersistenceConfig`
    pub fn build(self) -> PersistenceConfig {
        self.config
    }
}

impl Default for PersistenceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageBackendType {
    Memory,
    Sled(SledConfig),
    RocksDB(RocksDBConfig),
    #[cfg(feature = "postgres")]
    Postgres(PostgresConfig),
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

/// `PostgreSQL` configuration for kernel state storage (Phase 13b.2)
#[cfg(feature = "postgres")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    /// Database connection string (postgresql://user:pass@host:port/database)
    pub connection_string: String,

    /// Maximum size of connection pool (default: 20)
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Connection timeout in milliseconds (default: 5000)
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Enable Row-Level Security for multi-tenancy (default: true)
    #[serde(default = "default_enable_rls")]
    pub enable_rls: bool,
}

#[cfg(feature = "postgres")]
fn default_pool_size() -> u32 {
    20
}

#[cfg(feature = "postgres")]
fn default_timeout_ms() -> u64 {
    5000
}

#[cfg(feature = "postgres")]
fn default_enable_rls() -> bool {
    true
}

#[cfg(feature = "postgres")]
impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            connection_string: String::new(),
            pool_size: 20,
            timeout_ms: 5000,
            enable_rls: true,
        }
    }
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

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Base directory for storing backups
    pub backup_dir: std::path::PathBuf,

    /// Enable compression for backups
    pub compression_enabled: bool,

    /// Compression type to use
    pub compression_type: CompressionType,

    /// Compression level (1-9, higher = better compression)
    pub compression_level: u8,

    /// Enable encryption for sensitive data
    pub encryption_enabled: bool,

    /// Maximum number of backups to retain
    pub max_backups: Option<usize>,

    /// Maximum age of backups to retain
    pub max_backup_age: Option<Duration>,

    /// Enable incremental backups
    pub incremental_enabled: bool,

    /// Interval for full backups when using incremental
    pub full_backup_interval: Duration,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: std::path::PathBuf::from("./backups"),
            compression_enabled: true,
            compression_type: CompressionType::Zstd,
            compression_level: 3,
            encryption_enabled: false,
            max_backups: Some(10),
            max_backup_age: Some(Duration::from_secs(30 * 24 * 3600)), // 30 days
            incremental_enabled: true,
            full_backup_interval: Duration::from_secs(7 * 24 * 3600), // 7 days
        }
    }
}

/// Compression type for backups
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Lz4,
    Zstd,
}

impl CompressionType {
    /// Get file extension for compression type
    pub fn extension(&self) -> &'static str {
        match self {
            CompressionType::None => "",
            CompressionType::Lz4 => ".lz4",
            CompressionType::Zstd => ".zst",
        }
    }
}

impl std::fmt::Display for CompressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompressionType::None => write!(f, "none"),
            CompressionType::Lz4 => write!(f, "lz4"),
            CompressionType::Zstd => write!(f, "zstd"),
        }
    }
}
