//! ABOUTME: Configuration types for SessionManager providing operational parameters and limits
//! ABOUTME: Defines all configurable aspects of session management behavior

use chrono::Duration;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the `SessionManager`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct SessionManagerConfig {
    /// Maximum number of active sessions
    pub max_active_sessions: usize,
    /// Default session timeout
    pub default_session_timeout: Duration,
    /// Path for session storage
    pub storage_path: PathBuf,
    /// Enable automatic session persistence
    pub auto_persist: bool,
    /// Auto-persist interval in seconds
    pub persist_interval_secs: u64,
    /// Enable session activity tracking
    pub track_activity: bool,
    /// Maximum session storage size in bytes
    pub max_storage_size_bytes: u64,
    /// Enable compression for session data
    pub enable_compression: bool,
    /// Compression level (1-9, higher = better compression but slower)
    pub compression_level: u32,
    /// Enable content deduplication
    pub enable_deduplication: bool,
    /// Session cleanup configuration
    pub cleanup_config: CleanupConfig,
    /// Hook execution configuration
    pub hook_config: HookExecutionConfig,
    /// Event publishing configuration
    pub event_config: EventConfig,
}

impl SessionManagerConfig {
    /// Create a new builder for `SessionManagerConfig`
    pub fn builder() -> SessionManagerConfigBuilder {
        SessionManagerConfigBuilder::new()
    }

    /// Create configuration from `LLMSpellConfig`
    pub fn from_llm_config(config: &llmspell_config::LLMSpellConfig) -> Self {
        let mut builder = Self::builder();

        // Apply storage paths
        if let Some(sessions_dir) = &config.storage.sessions_dir {
            builder = builder.storage_path(sessions_dir.clone());
        }

        // Apply runtime settings relevant to sessions if any
        // Example: logic for max sessions could be mapped here if exposed in LLMSpellConfig

        builder.build()
    }
}

impl Default for SessionManagerConfig {
    fn default() -> Self {
        Self {
            max_active_sessions: 1000,
            default_session_timeout: Duration::hours(24),
            storage_path: PathBuf::from("./sessions"),
            auto_persist: true,
            persist_interval_secs: 300, // 5 minutes
            track_activity: true,
            max_storage_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
            enable_compression: true,
            compression_level: 3, // Fast compression
            enable_deduplication: true,
            cleanup_config: CleanupConfig::default(),
            hook_config: HookExecutionConfig::default(),
            event_config: EventConfig::default(),
        }
    }
}

/// Configuration for session cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct CleanupConfig {
    /// Enable automatic cleanup of old sessions
    pub enable_auto_cleanup: bool,
    /// Cleanup interval in seconds
    pub cleanup_interval_secs: u64,
    /// Delete sessions older than this duration
    pub delete_after: Duration,
    /// Archive sessions before deletion
    pub archive_before_delete: bool,
    /// Archive path
    pub archive_path: Option<PathBuf>,
}

impl Default for CleanupConfig {
    fn default() -> Self {
        Self {
            enable_auto_cleanup: true,
            cleanup_interval_secs: 3600, // 1 hour
            delete_after: Duration::days(30),
            archive_before_delete: true,
            archive_path: Some(PathBuf::from("./sessions/archive")),
        }
    }
}

/// Configuration for hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct HookExecutionConfig {
    /// Enable session lifecycle hooks
    pub enable_lifecycle_hooks: bool,
    /// Enable artifact hooks
    pub enable_artifact_hooks: bool,
    /// Enable automatic artifact collection
    pub enable_artifact_collection: bool,
    /// Hook timeout in milliseconds
    pub hook_timeout_ms: u64,
    /// Maximum concurrent hook executions
    pub max_concurrent_hooks: usize,
    /// Retry failed hooks
    pub retry_failed_hooks: bool,
    /// Maximum retry attempts
    pub max_retry_attempts: u32,
}

impl Default for HookExecutionConfig {
    fn default() -> Self {
        Self {
            enable_lifecycle_hooks: true,
            enable_artifact_hooks: true,
            enable_artifact_collection: true,
            hook_timeout_ms: 5000,
            max_concurrent_hooks: 10,
            retry_failed_hooks: true,
            max_retry_attempts: 3,
        }
    }
}

/// Configuration for event publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventConfig {
    /// Enable session events
    pub enable_session_events: bool,
    /// Enable artifact events
    pub enable_artifact_events: bool,
    /// Event buffer size
    pub event_buffer_size: usize,
    /// Event publish timeout in milliseconds
    pub publish_timeout_ms: u64,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            enable_session_events: true,
            enable_artifact_events: true,
            event_buffer_size: 1000,
            publish_timeout_ms: 1000,
        }
    }
}

/// Builder for `SessionManagerConfig`
#[must_use]
pub struct SessionManagerConfigBuilder {
    config: SessionManagerConfig,
}

impl SessionManagerConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: SessionManagerConfig::default(),
        }
    }

    /// Set maximum active sessions
    pub fn max_active_sessions(mut self, max: usize) -> Self {
        self.config.max_active_sessions = max;
        self
    }

    /// Set storage path
    pub fn storage_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.storage_path = path.into();
        self
    }

    /// Enable or disable auto-persistence
    pub fn auto_persist(mut self, enabled: bool) -> Self {
        self.config.auto_persist = enabled;
        self
    }

    /// Set compression level (1-9)
    pub fn compression_level(mut self, level: u32) -> Self {
        self.config.compression_level = level.clamp(1, 9);
        self
    }

    /// Set persist interval in seconds
    pub fn persist_interval_secs(mut self, secs: u64) -> Self {
        self.config.persist_interval_secs = secs;
        self
    }

    /// Set default session timeout
    pub fn default_session_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_session_timeout = timeout;
        self
    }

    /// Enable or disable activity tracking
    pub fn track_activity(mut self, enabled: bool) -> Self {
        self.config.track_activity = enabled;
        self
    }

    /// Set maximum storage size in bytes
    pub fn max_storage_size_bytes(mut self, bytes: u64) -> Self {
        self.config.max_storage_size_bytes = bytes;
        self
    }

    /// Enable or disable compression
    pub fn enable_compression(mut self, enabled: bool) -> Self {
        self.config.enable_compression = enabled;
        self
    }

    /// Enable or disable deduplication
    pub fn enable_deduplication(mut self, enabled: bool) -> Self {
        self.config.enable_deduplication = enabled;
        self
    }

    /// Set cleanup configuration
    pub fn cleanup_config(mut self, config: CleanupConfig) -> Self {
        self.config.cleanup_config = config;
        self
    }

    /// Set hook execution configuration
    pub fn hook_config(mut self, config: HookExecutionConfig) -> Self {
        self.config.hook_config = config;
        self
    }

    /// Set event configuration
    pub fn event_config(mut self, config: EventConfig) -> Self {
        self.config.event_config = config;
        self
    }

    /// Build the configuration
    pub fn build(self) -> SessionManagerConfig {
        self.config
    }
}

impl Default for SessionManagerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_default_config() {
        let config = SessionManagerConfig::default();
        assert_eq!(config.max_active_sessions, 1000);
        assert!(config.auto_persist);
        assert!(config.enable_compression);
        assert_eq!(config.compression_level, 3);
    }
    #[test]
    fn test_config_builder() {
        let config = SessionManagerConfigBuilder::new()
            .max_active_sessions(500)
            .storage_path("/tmp/sessions")
            .auto_persist(false)
            .compression_level(9)
            .build();

        assert_eq!(config.max_active_sessions, 500);
        assert_eq!(config.storage_path, PathBuf::from("/tmp/sessions"));
        assert!(!config.auto_persist);
        assert_eq!(config.compression_level, 9);
    }
    #[test]
    fn test_compression_level_clamping() {
        let config = SessionManagerConfigBuilder::new()
            .compression_level(100)
            .build();
        assert_eq!(config.compression_level, 9);

        let config = SessionManagerConfigBuilder::new()
            .compression_level(0)
            .build();
        assert_eq!(config.compression_level, 1);
    }
}
