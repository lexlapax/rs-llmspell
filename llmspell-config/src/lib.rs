//! ABOUTME: Central configuration management system for llmspell
//! ABOUTME: Handles TOML parsing, validation, and environment variable overrides

use anyhow::{Context, Result};
use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::debug;

// Re-export engine configurations from bridge
pub use crate::engines::{EngineConfigs, JSConfig, LuaConfig};
pub use crate::providers::{ProviderConfig, ProviderManagerConfig, ProviderManagerConfigBuilder};
pub use crate::tools::{FileOperationsConfig, ToolsConfig};

pub mod engines;
pub mod providers;
pub mod tools;
pub mod validation;

/// Configuration file discovery order
const CONFIG_SEARCH_PATHS: &[&str] = &[
    "llmspell.toml",
    ".llmspell.toml",
    "config/llmspell.toml",
    ".config/llmspell.toml",
];

/// Environment variable prefix
const ENV_PREFIX: &str = "LLMSPELL_";

/// Central LLMSpell configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LLMSpellConfig {
    /// Default script engine to use
    pub default_engine: String,
    /// Engine-specific configurations  
    pub engines: EngineConfigs,
    /// Provider configurations
    pub providers: ProviderManagerConfig,
    /// Global runtime settings
    pub runtime: GlobalRuntimeConfig,
    /// Tool-specific configurations
    pub tools: ToolsConfig,
}

impl Default for LLMSpellConfig {
    fn default() -> Self {
        Self {
            default_engine: "lua".to_string(),
            engines: EngineConfigs::default(),
            providers: ProviderManagerConfig::default(),
            runtime: GlobalRuntimeConfig::default(),
            tools: ToolsConfig::default(),
        }
    }
}

impl LLMSpellConfig {
    /// Create a new builder for `LLMSpellConfig`
    #[must_use]
    pub fn builder() -> LLMSpellConfigBuilder {
        LLMSpellConfigBuilder::new()
    }

    /// Load configuration from TOML file with validation
    pub async fn load_from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = fs::read_to_string(path)
            .await
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        Self::from_toml(&content)
    }

    /// Parse TOML content with environment variable overrides and validation
    pub fn from_toml(content: &str) -> Result<Self, ConfigError> {
        let mut config: LLMSpellConfig =
            toml::from_str(content).with_context(|| "Failed to parse TOML configuration")?;

        config.apply_env_overrides()?;
        config.validate()?;

        Ok(config)
    }

    /// Apply environment variable overrides
    pub fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Override default engine if specified
        if let Ok(engine) = env::var(format!("{}DEFAULT_ENGINE", ENV_PREFIX)) {
            debug!("Overriding default engine from env: {}", engine);
            self.default_engine = engine;
        }

        // Override runtime settings
        if let Ok(max_scripts) = env::var(format!("{}MAX_CONCURRENT_SCRIPTS", ENV_PREFIX)) {
            if let Ok(val) = max_scripts.parse::<usize>() {
                debug!("Overriding max_concurrent_scripts from env: {}", val);
                self.runtime.max_concurrent_scripts = val;
            }
        }

        if let Ok(timeout) = env::var(format!("{}SCRIPT_TIMEOUT_SECONDS", ENV_PREFIX)) {
            if let Ok(val) = timeout.parse::<u64>() {
                debug!("Overriding script_timeout_seconds from env: {}", val);
                self.runtime.script_timeout_seconds = val;
            }
        }

        // Override security settings
        if let Ok(allow_file) = env::var(format!("{}ALLOW_FILE_ACCESS", ENV_PREFIX)) {
            if let Ok(val) = allow_file.parse::<bool>() {
                debug!("Overriding allow_file_access from env: {}", val);
                self.runtime.security.allow_file_access = val;
            }
        }

        if let Ok(allow_network) = env::var(format!("{}ALLOW_NETWORK_ACCESS", ENV_PREFIX)) {
            if let Ok(val) = allow_network.parse::<bool>() {
                debug!("Overriding allow_network_access from env: {}", val);
                self.runtime.security.allow_network_access = val;
            }
        }

        Ok(())
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_config(self)
    }

    /// Get engine-specific configuration
    pub fn get_engine_config(&self, engine_name: &str) -> Result<serde_json::Value, ConfigError> {
        match engine_name {
            "lua" => Ok(serde_json::to_value(&self.engines.lua)?),
            "javascript" | "js" => Ok(serde_json::to_value(&self.engines.javascript)?),
            custom => {
                self.engines
                    .custom
                    .get(custom)
                    .cloned()
                    .ok_or_else(|| ConfigError::Validation {
                        field: Some("engine".to_string()),
                        message: format!("Engine configuration not found for '{custom}'"),
                    })
            }
        }
    }

    /// Check if an engine is configured
    #[must_use]
    pub fn supports_engine(&self, engine_name: &str) -> bool {
        match engine_name {
            "lua" | "javascript" | "js" => true,
            custom => self.engines.custom.contains_key(custom),
        }
    }

    /// Discover configuration file in standard locations
    pub async fn discover_config_file() -> Result<Option<PathBuf>, ConfigError> {
        // Check current directory first
        for path in CONFIG_SEARCH_PATHS {
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(Some(path));
            }
        }

        // Check home directory
        if let Ok(home_dir) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
            let home_path = PathBuf::from(home_dir);

            for filename in &[".llmspell.toml", ".config/llmspell.toml"] {
                let path = home_path.join(filename);
                if path.exists() {
                    return Ok(Some(path));
                }
            }
        }

        // Check system config directories (Linux/macOS style)
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            let path = PathBuf::from(xdg_config)
                .join("llmspell")
                .join("config.toml");
            if path.exists() {
                return Ok(Some(path));
            }
        }

        Ok(None)
    }

    /// Load configuration with automatic discovery
    pub async fn load_with_discovery(explicit_path: Option<&Path>) -> Result<Self, ConfigError> {
        // If explicit path provided, use it
        if let Some(path) = explicit_path {
            if path.exists() {
                return Self::load_from_file(path).await;
            } else {
                return Err(ConfigError::NotFound {
                    path: path.to_string_lossy().to_string(),
                    message: "Explicitly specified config file not found".to_string(),
                });
            }
        }

        // Discover config file
        if let Some(discovered_path) = Self::discover_config_file().await? {
            return Self::load_from_file(&discovered_path).await;
        }

        // No config file found, use defaults with environment overrides
        let mut config = Self::default();
        config.apply_env_overrides()?;
        config.validate()?;

        Ok(config)
    }
}

/// Builder for `LLMSpellConfig`
#[derive(Debug, Clone)]
pub struct LLMSpellConfigBuilder {
    config: LLMSpellConfig,
}

impl LLMSpellConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: LLMSpellConfig::default(),
        }
    }

    /// Set the default script engine
    #[must_use]
    pub fn default_engine(mut self, engine: impl Into<String>) -> Self {
        self.config.default_engine = engine.into();
        self
    }

    /// Set the engine configurations
    #[must_use]
    pub fn engines(mut self, engines: EngineConfigs) -> Self {
        self.config.engines = engines;
        self
    }

    /// Set the provider configuration
    #[must_use]
    pub fn providers(mut self, providers: ProviderManagerConfig) -> Self {
        self.config.providers = providers;
        self
    }

    /// Set the global runtime configuration
    #[must_use]
    pub fn runtime(mut self, runtime: GlobalRuntimeConfig) -> Self {
        self.config.runtime = runtime;
        self
    }

    /// Set the tools configuration
    #[must_use]
    pub fn tools(mut self, tools: ToolsConfig) -> Self {
        self.config.tools = tools;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> LLMSpellConfig {
        self.config
    }
}

impl Default for LLMSpellConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global runtime configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct GlobalRuntimeConfig {
    /// Maximum concurrent scripts
    pub max_concurrent_scripts: usize,
    /// Script execution timeout in seconds
    pub script_timeout_seconds: u64,
    /// Enable streaming by default
    pub enable_streaming: bool,
    /// Security settings
    pub security: SecurityConfig,
    /// State persistence settings
    pub state_persistence: StatePersistenceConfig,
    /// Session management settings
    pub sessions: SessionConfig,
}

impl Default for GlobalRuntimeConfig {
    fn default() -> Self {
        Self {
            max_concurrent_scripts: 10,
            script_timeout_seconds: 300,
            enable_streaming: true,
            security: SecurityConfig::default(),
            state_persistence: StatePersistenceConfig::default(),
            sessions: SessionConfig::default(),
        }
    }
}

impl GlobalRuntimeConfig {
    /// Create a new builder for `GlobalRuntimeConfig`
    #[must_use]
    pub fn builder() -> GlobalRuntimeConfigBuilder {
        GlobalRuntimeConfigBuilder::new()
    }
}

/// Builder for `GlobalRuntimeConfig`
#[derive(Debug, Clone)]
pub struct GlobalRuntimeConfigBuilder {
    config: GlobalRuntimeConfig,
}

impl GlobalRuntimeConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: GlobalRuntimeConfig::default(),
        }
    }

    /// Set the maximum concurrent scripts
    #[must_use]
    pub const fn max_concurrent_scripts(mut self, max: usize) -> Self {
        self.config.max_concurrent_scripts = max;
        self
    }

    /// Set the script execution timeout in seconds
    #[must_use]
    pub const fn script_timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.script_timeout_seconds = timeout;
        self
    }

    /// Enable or disable streaming
    #[must_use]
    pub const fn enable_streaming(mut self, enable: bool) -> Self {
        self.config.enable_streaming = enable;
        self
    }

    /// Set the security configuration
    #[must_use]
    pub const fn security(mut self, security: SecurityConfig) -> Self {
        self.config.security = security;
        self
    }

    /// Set the state persistence configuration
    #[must_use]
    pub fn state_persistence(mut self, persistence: StatePersistenceConfig) -> Self {
        self.config.state_persistence = persistence;
        self
    }

    /// Set the session configuration
    #[must_use]
    pub fn sessions(mut self, sessions: SessionConfig) -> Self {
        self.config.sessions = sessions;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> GlobalRuntimeConfig {
        self.config
    }
}

impl Default for GlobalRuntimeConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Security configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SecurityConfig {
    /// Allow file system access
    pub allow_file_access: bool,
    /// Allow network access
    pub allow_network_access: bool,
    /// Allow process spawning
    pub allow_process_spawn: bool,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: Option<usize>,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allow_file_access: false,
            allow_network_access: true,
            allow_process_spawn: false,
            max_memory_bytes: Some(50_000_000),   // 50MB
            max_execution_time_ms: Some(300_000), // 5 minutes
        }
    }
}

/// Core state persistence flags
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct CoreStateFlags {
    /// Enable state persistence
    pub enabled: bool,
    /// Enable migration functionality
    pub migration_enabled: bool,
}

impl Default for CoreStateFlags {
    fn default() -> Self {
        Self {
            enabled: true, // Changed from false - in-memory state by default
            migration_enabled: false,
        }
    }
}

/// Backup-related flags
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct BackupFlags {
    /// Automatic backup on migration
    pub backup_on_migration: bool,
    /// Enable backup functionality
    pub backup_enabled: bool,
}

impl Default for BackupFlags {
    fn default() -> Self {
        Self {
            backup_on_migration: true,
            backup_enabled: false,
        }
    }
}

/// State persistence feature flags
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct StatePersistenceFlags {
    /// Core state persistence features
    #[serde(flatten)]
    pub core: CoreStateFlags,
    /// Backup-related features
    #[serde(flatten)]
    pub backup: BackupFlags,
}

/// State persistence configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct StatePersistenceConfig {
    /// Feature flags for state persistence
    #[serde(flatten)]
    pub flags: StatePersistenceFlags,
    /// Backend type for storage (memory, file, redis, etc.)
    pub backend_type: String,
    /// Directory for schema definitions
    pub schema_directory: Option<String>,
    /// Maximum state size per key in bytes
    pub max_state_size_bytes: Option<usize>,
    /// Backup configuration
    pub backup: Option<BackupConfig>,
}

/// Backup configuration for state persistence
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackupConfig {
    /// Directory for backup storage
    pub backup_dir: Option<String>,
    /// Enable compression for backups
    pub compression_enabled: bool,
    /// Compression type to use
    pub compression_type: String,
    /// Compression level (1-9)
    pub compression_level: u8,
    /// Enable incremental backups
    pub incremental_enabled: bool,
    /// Maximum number of backups to keep
    pub max_backups: Option<usize>,
    /// Maximum age of backups in seconds
    pub max_backup_age: Option<u64>,
}

impl Default for StatePersistenceConfig {
    fn default() -> Self {
        Self {
            flags: StatePersistenceFlags::default(),
            backend_type: "memory".to_string(),
            schema_directory: None,
            max_state_size_bytes: Some(10_000_000), // 10MB per key
            backup: None,
        }
    }
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            backup_dir: Some("./backups".to_string()),
            compression_enabled: true,
            compression_type: "zstd".to_string(),
            compression_level: 3,
            incremental_enabled: true,
            max_backups: Some(10),
            max_backup_age: Some(2_592_000), // 30 days
        }
    }
}

/// Session management configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SessionConfig {
    /// Enable session management
    pub enabled: bool,
    /// Maximum number of concurrent sessions
    pub max_sessions: usize,
    /// Maximum artifacts per session
    pub max_artifacts_per_session: usize,
    /// Artifact compression threshold in bytes
    pub artifact_compression_threshold: usize,
    /// Session timeout in seconds
    pub session_timeout_seconds: u64,
    /// Storage backend type (memory, sled)
    pub storage_backend: String,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_sessions: 100,
            max_artifacts_per_session: 1000,
            artifact_compression_threshold: 10240, // 10KB
            session_timeout_seconds: 3600,         // 1 hour
            storage_backend: "memory".to_string(),
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration file not found: {path} - {message}")]
    NotFound { path: String, message: String },

    #[error("Configuration validation failed in field '{field:?}': {message}")]
    Validation {
        field: Option<String>,
        message: String,
    },

    #[error("Environment variable parsing error: {message}")]
    Environment { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("General error: {0}")]
    Other(#[from] anyhow::Error),
}

impl From<ConfigError> for LLMSpellError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::Validation { field, message } => {
                LLMSpellError::Validation { field, message }
            }
            ConfigError::NotFound { path, message } => LLMSpellError::Configuration {
                message: format!("Configuration file not found: {} - {}", path, message),
                source: None,
            },
            _ => LLMSpellError::Configuration {
                message: err.to_string(),
                source: Some(Box::new(err)),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llmspell_config_default() {
        let config = LLMSpellConfig::default();
        assert_eq!(config.default_engine, "lua");
        assert!(config.supports_engine("lua"));
        assert!(config.supports_engine("javascript"));
        assert!(!config.supports_engine("python"));
    }

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(!config.allow_file_access);
        assert!(config.allow_network_access);
        assert!(!config.allow_process_spawn);
        assert_eq!(config.max_memory_bytes, Some(50_000_000));
        assert_eq!(config.max_execution_time_ms, Some(300_000));
    }

    #[test]
    fn test_config_builder() {
        let config = LLMSpellConfig::builder()
            .default_engine("javascript")
            .build();

        assert_eq!(config.default_engine, "javascript");
    }
}
