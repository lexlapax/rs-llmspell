//! ABOUTME: Memory system configuration for adaptive memory with LLM consolidation
//! ABOUTME: Handles episodic memory, semantic consolidation, and daemon scheduling

use serde::{Deserialize, Serialize};

/// Comprehensive memory system configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct MemoryConfig {
    /// Enable memory functionality
    #[serde(default)]
    pub enabled: bool,
    /// LLM consolidation configuration
    pub consolidation: ConsolidationConfig,
    /// Background daemon configuration
    pub daemon: DaemonConfig,
}

impl MemoryConfig {
    /// Create a new builder for `MemoryConfig`
    #[must_use]
    pub fn builder() -> MemoryConfigBuilder {
        MemoryConfigBuilder::new()
    }
}

/// LLM-driven consolidation configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct ConsolidationConfig {
    /// Provider name for LLM consolidation (falls back to global default_provider)
    pub provider_name: Option<String>,

    /// Number of episodes to consolidate in one batch
    pub batch_size: usize,

    /// Maximum concurrent consolidation operations
    pub max_concurrent: usize,

    /// Active session threshold in seconds (sessions accessed within this time are considered active)
    pub active_session_threshold_secs: u64,
}

impl Default for ConsolidationConfig {
    fn default() -> Self {
        Self {
            provider_name: None, // Falls back to default_provider
            batch_size: 10,
            max_concurrent: 3,
            active_session_threshold_secs: 300, // 5 minutes
        }
    }
}

impl ConsolidationConfig {
    /// Create a new builder for `ConsolidationConfig`
    #[must_use]
    pub fn builder() -> ConsolidationConfigBuilder {
        ConsolidationConfigBuilder::new()
    }
}

/// Background daemon configuration for memory consolidation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct DaemonConfig {
    /// Enable background consolidation daemon
    #[serde(default)]
    pub enabled: bool,

    /// Fast consolidation interval in seconds (when queue > queue_threshold_fast)
    pub fast_interval_secs: u64,

    /// Normal consolidation interval in seconds
    pub normal_interval_secs: u64,

    /// Slow consolidation interval in seconds (when queue < queue_threshold_slow)
    pub slow_interval_secs: u64,

    /// Queue size threshold for fast interval
    pub queue_threshold_fast: usize,

    /// Queue size threshold for slow interval
    pub queue_threshold_slow: usize,

    /// Maximum wait time for graceful shutdown in seconds
    pub shutdown_max_wait_secs: u64,

    /// Health check interval in seconds
    pub health_check_interval_secs: u64,
}

impl Default for DaemonConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            fast_interval_secs: 30,       // 30 seconds when queue is high
            normal_interval_secs: 300,    // 5 minutes normal operation
            slow_interval_secs: 600,      // 10 minutes when queue is low
            queue_threshold_fast: 10,     // Fast mode when queue > 10
            queue_threshold_slow: 3,      // Slow mode when queue < 3
            shutdown_max_wait_secs: 30,   // Wait up to 30 seconds for graceful shutdown
            health_check_interval_secs: 60, // Health check every minute
        }
    }
}

impl DaemonConfig {
    /// Create a new builder for `DaemonConfig`
    #[must_use]
    pub fn builder() -> DaemonConfigBuilder {
        DaemonConfigBuilder::new()
    }
}

/// Builder for `MemoryConfig`
#[derive(Debug, Clone)]
pub struct MemoryConfigBuilder {
    config: MemoryConfig,
}

impl MemoryConfigBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: MemoryConfig::default(),
        }
    }

    /// Enable or disable memory functionality
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set consolidation configuration
    #[must_use]
    pub fn consolidation(mut self, consolidation: ConsolidationConfig) -> Self {
        self.config.consolidation = consolidation;
        self
    }

    /// Set daemon configuration
    #[must_use]
    pub fn daemon(mut self, daemon: DaemonConfig) -> Self {
        self.config.daemon = daemon;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> MemoryConfig {
        self.config
    }
}

impl Default for MemoryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `ConsolidationConfig`
#[derive(Debug, Clone)]
pub struct ConsolidationConfigBuilder {
    config: ConsolidationConfig,
}

impl ConsolidationConfigBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ConsolidationConfig::default(),
        }
    }

    /// Set provider name for LLM consolidation
    #[must_use]
    pub fn provider_name(mut self, name: impl Into<String>) -> Self {
        self.config.provider_name = Some(name.into());
        self
    }

    /// Set batch size for consolidation
    #[must_use]
    pub const fn batch_size(mut self, size: usize) -> Self {
        self.config.batch_size = size;
        self
    }

    /// Set maximum concurrent consolidation operations
    #[must_use]
    pub const fn max_concurrent(mut self, max: usize) -> Self {
        self.config.max_concurrent = max;
        self
    }

    /// Set active session threshold in seconds
    #[must_use]
    pub const fn active_session_threshold_secs(mut self, secs: u64) -> Self {
        self.config.active_session_threshold_secs = secs;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> ConsolidationConfig {
        self.config
    }
}

impl Default for ConsolidationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `DaemonConfig`
#[derive(Debug, Clone)]
pub struct DaemonConfigBuilder {
    config: DaemonConfig,
}

impl DaemonConfigBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: DaemonConfig::default(),
        }
    }

    /// Enable or disable daemon
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    /// Set fast interval in seconds
    #[must_use]
    pub const fn fast_interval_secs(mut self, secs: u64) -> Self {
        self.config.fast_interval_secs = secs;
        self
    }

    /// Set normal interval in seconds
    #[must_use]
    pub const fn normal_interval_secs(mut self, secs: u64) -> Self {
        self.config.normal_interval_secs = secs;
        self
    }

    /// Set slow interval in seconds
    #[must_use]
    pub const fn slow_interval_secs(mut self, secs: u64) -> Self {
        self.config.slow_interval_secs = secs;
        self
    }

    /// Set queue threshold for fast interval
    #[must_use]
    pub const fn queue_threshold_fast(mut self, threshold: usize) -> Self {
        self.config.queue_threshold_fast = threshold;
        self
    }

    /// Set queue threshold for slow interval
    #[must_use]
    pub const fn queue_threshold_slow(mut self, threshold: usize) -> Self {
        self.config.queue_threshold_slow = threshold;
        self
    }

    /// Set maximum shutdown wait time in seconds
    #[must_use]
    pub const fn shutdown_max_wait_secs(mut self, secs: u64) -> Self {
        self.config.shutdown_max_wait_secs = secs;
        self
    }

    /// Set health check interval in seconds
    #[must_use]
    pub const fn health_check_interval_secs(mut self, secs: u64) -> Self {
        self.config.health_check_interval_secs = secs;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> DaemonConfig {
        self.config
    }
}

impl Default for DaemonConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_config_default() {
        let config = MemoryConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.consolidation.batch_size, 10);
        assert!(config.daemon.enabled);
    }

    #[test]
    fn test_memory_config_builder() {
        let config = MemoryConfig::builder()
            .enabled(true)
            .consolidation(ConsolidationConfig::builder().batch_size(20).build())
            .daemon(DaemonConfig::builder().fast_interval_secs(15).build())
            .build();

        assert!(config.enabled);
        assert_eq!(config.consolidation.batch_size, 20);
        assert_eq!(config.daemon.fast_interval_secs, 15);
    }

    #[test]
    fn test_consolidation_config_default() {
        let config = ConsolidationConfig::default();
        assert_eq!(config.provider_name, None);
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.max_concurrent, 3);
        assert_eq!(config.active_session_threshold_secs, 300);
    }

    #[test]
    fn test_consolidation_config_builder() {
        let config = ConsolidationConfig::builder()
            .provider_name("consolidation-llm")
            .batch_size(15)
            .max_concurrent(5)
            .active_session_threshold_secs(600)
            .build();

        assert_eq!(config.provider_name, Some("consolidation-llm".to_string()));
        assert_eq!(config.batch_size, 15);
        assert_eq!(config.max_concurrent, 5);
        assert_eq!(config.active_session_threshold_secs, 600);
    }

    #[test]
    fn test_daemon_config_default() {
        let config = DaemonConfig::default();
        assert!(config.enabled);
        assert_eq!(config.fast_interval_secs, 30);
        assert_eq!(config.normal_interval_secs, 300);
        assert_eq!(config.slow_interval_secs, 600);
        assert_eq!(config.queue_threshold_fast, 10);
        assert_eq!(config.queue_threshold_slow, 3);
        assert_eq!(config.shutdown_max_wait_secs, 30);
        assert_eq!(config.health_check_interval_secs, 60);
    }

    #[test]
    fn test_daemon_config_builder() {
        let config = DaemonConfig::builder()
            .enabled(false)
            .fast_interval_secs(20)
            .normal_interval_secs(200)
            .slow_interval_secs(500)
            .queue_threshold_fast(15)
            .queue_threshold_slow(5)
            .shutdown_max_wait_secs(60)
            .health_check_interval_secs(120)
            .build();

        assert!(!config.enabled);
        assert_eq!(config.fast_interval_secs, 20);
        assert_eq!(config.normal_interval_secs, 200);
        assert_eq!(config.slow_interval_secs, 500);
        assert_eq!(config.queue_threshold_fast, 15);
        assert_eq!(config.queue_threshold_slow, 5);
        assert_eq!(config.shutdown_max_wait_secs, 60);
        assert_eq!(config.health_check_interval_secs, 120);
    }

    #[test]
    fn test_consolidation_config_serialization() {
        let config = ConsolidationConfig::builder()
            .provider_name("test-provider")
            .batch_size(25)
            .build();

        let serialized = serde_json::to_string(&config).expect("Serialization should work");
        let deserialized: ConsolidationConfig =
            serde_json::from_str(&serialized).expect("Deserialization should work");

        assert_eq!(deserialized.provider_name, Some("test-provider".to_string()));
        assert_eq!(deserialized.batch_size, 25);
    }

    #[test]
    fn test_memory_config_serialization() {
        let config = MemoryConfig::builder()
            .enabled(true)
            .consolidation(
                ConsolidationConfig::builder()
                    .provider_name("mem-llm")
                    .build(),
            )
            .build();

        let serialized = serde_json::to_string(&config).expect("Serialization should work");
        let deserialized: MemoryConfig =
            serde_json::from_str(&serialized).expect("Deserialization should work");

        assert!(deserialized.enabled);
        assert_eq!(
            deserialized.consolidation.provider_name,
            Some("mem-llm".to_string())
        );
    }
}
