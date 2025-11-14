//! SQLite configuration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// SQLite backend configuration
///
/// Configures connection pooling, encryption, and performance tuning for SQLite.
/// Follows similar patterns to PostgresConfig for consistency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// Database file path (":memory:" for in-memory database)
    pub database_path: PathBuf,

    /// Maximum number of connections in pool
    ///
    /// SQLite with WAL mode supports multiple concurrent readers.
    /// Default: 20 connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum idle connections in pool
    #[serde(default = "default_min_idle_connections")]
    pub min_idle_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout: u64,

    /// Idle connection timeout in seconds (300s = 5 minutes)
    #[serde(default = "default_idle_timeout")]
    pub idle_timeout: u64,

    /// Enable encryption at rest (requires libsql encryption feature)
    #[serde(default)]
    pub enable_encryption: bool,

    /// Encryption key (hex-encoded AES-256 key if encryption enabled)
    pub encryption_key: Option<String>,

    /// WAL mode synchronous setting (NORMAL, FULL, OFF)
    ///
    /// NORMAL: Good balance of safety and performance (recommended)
    /// FULL: Maximum durability, slower writes
    /// OFF: Fastest, but risk of corruption on power loss
    #[serde(default = "default_synchronous")]
    pub synchronous: String,

    /// Cache size in KB (negative for KB, positive for pages)
    ///
    /// Default: -64000 (64MB cache)
    #[serde(default = "default_cache_size")]
    pub cache_size: i32,

    /// Memory-mapped I/O size in bytes (0 to disable)
    ///
    /// Default: 100MB mmap for faster reads on small databases
    #[serde(default = "default_mmap_size")]
    pub mmap_size: i64,

    /// Enable busy timeout in milliseconds (for write contention)
    ///
    /// Default: 5000ms (5 seconds)
    #[serde(default = "default_busy_timeout")]
    pub busy_timeout: u32,
}

fn default_max_connections() -> u32 {
    20
}

fn default_min_idle_connections() -> u32 {
    2
}

fn default_connection_timeout() -> u64 {
    5
}

fn default_idle_timeout() -> u64 {
    300
}

fn default_synchronous() -> String {
    "NORMAL".to_string()
}

fn default_cache_size() -> i32 {
    -64000 // 64MB
}

fn default_mmap_size() -> i64 {
    104_857_600 // 100MB
}

fn default_busy_timeout() -> u32 {
    5000
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            database_path: PathBuf::from(":memory:"),
            max_connections: default_max_connections(),
            min_idle_connections: default_min_idle_connections(),
            connection_timeout: default_connection_timeout(),
            idle_timeout: default_idle_timeout(),
            enable_encryption: false,
            encryption_key: None,
            synchronous: default_synchronous(),
            cache_size: default_cache_size(),
            mmap_size: default_mmap_size(),
            busy_timeout: default_busy_timeout(),
        }
    }
}

impl SqliteConfig {
    /// Create configuration for file-based database
    ///
    /// # Arguments
    ///
    /// * `database_path` - Path to SQLite database file
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use llmspell_storage::backends::sqlite::SqliteConfig;
    ///
    /// let config = SqliteConfig::new("./data/llmspell.db");
    /// ```
    pub fn new(database_path: impl Into<PathBuf>) -> Self {
        Self {
            database_path: database_path.into(),
            ..Default::default()
        }
    }

    /// Create in-memory database configuration (for testing)
    ///
    /// In-memory databases are destroyed when last connection closes.
    ///
    /// # Examples
    ///
    /// ```
    /// use llmspell_storage::backends::sqlite::SqliteConfig;
    ///
    /// let config = SqliteConfig::in_memory();
    /// ```
    pub fn in_memory() -> Self {
        // libsql's :memory: creates isolated databases per connection
        // Use unique temp file for each test to ensure connections share same database
        let unique_id = uuid::Uuid::new_v4();
        let temp_path = std::env::temp_dir().join(format!("llmspell_test_{}.db", unique_id));

        Self {
            database_path: temp_path,
            ..Default::default()
        }
    }

    /// Enable encryption with provided key
    ///
    /// # Arguments
    ///
    /// * `key` - Hex-encoded AES-256 encryption key (64 hex characters)
    pub fn with_encryption(mut self, key: String) -> Self {
        self.enable_encryption = true;
        self.encryption_key = Some(key);
        self
    }

    /// Set connection pool size
    pub fn with_max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set minimum idle connections
    pub fn with_min_idle(mut self, min: u32) -> Self {
        self.min_idle_connections = min;
        self
    }

    /// Set WAL synchronous mode
    ///
    /// # Arguments
    ///
    /// * `mode` - "NORMAL", "FULL", or "OFF"
    pub fn with_synchronous(mut self, mode: impl Into<String>) -> Self {
        self.synchronous = mode.into();
        self
    }

    /// Set cache size in KB
    pub fn with_cache_size_kb(mut self, kb: i32) -> Self {
        self.cache_size = -kb.abs(); // Negative for KB
        self
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns error if configuration is invalid
    pub fn validate(&self) -> Result<(), String> {
        if self.max_connections == 0 {
            return Err("max_connections must be > 0".to_string());
        }

        if self.min_idle_connections > self.max_connections {
            return Err("min_idle_connections cannot exceed max_connections".to_string());
        }

        if self.enable_encryption && self.encryption_key.is_none() {
            return Err("encryption_key required when enable_encryption=true".to_string());
        }

        if let Some(ref key) = self.encryption_key {
            if key.len() != 64 {
                return Err("encryption_key must be 64 hex characters (AES-256)".to_string());
            }
        }

        let valid_sync = ["NORMAL", "FULL", "OFF"];
        if !valid_sync.contains(&self.synchronous.as_str()) {
            return Err(format!(
                "synchronous must be one of: {}",
                valid_sync.join(", ")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SqliteConfig::default();
        assert_eq!(config.database_path, PathBuf::from(":memory:"));
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.synchronous, "NORMAL");
    }

    #[test]
    fn test_in_memory_config() {
        let config = SqliteConfig::in_memory();
        // in_memory() now uses unique temp files instead of :memory:
        // to ensure connections share the same database (libsql isolation fix)
        assert!(config
            .database_path
            .to_str()
            .unwrap()
            .contains("llmspell_test_"));
        assert!(config.database_path.to_str().unwrap().ends_with(".db"));
    }

    #[test]
    fn test_file_config() {
        let config = SqliteConfig::new("/tmp/test.db");
        assert_eq!(config.database_path, PathBuf::from("/tmp/test.db"));
    }

    #[test]
    fn test_with_encryption() {
        let key = "a".repeat(64);
        let config = SqliteConfig::in_memory().with_encryption(key.clone());
        assert!(config.enable_encryption);
        assert_eq!(config.encryption_key, Some(key));
    }

    #[test]
    fn test_validation() {
        let config = SqliteConfig::default();
        assert!(config.validate().is_ok());

        let bad_config = SqliteConfig {
            max_connections: 0,
            ..Default::default()
        };
        assert!(bad_config.validate().is_err());

        let bad_sync = SqliteConfig {
            synchronous: "INVALID".to_string(),
            ..Default::default()
        };
        assert!(bad_sync.validate().is_err());
    }
}
