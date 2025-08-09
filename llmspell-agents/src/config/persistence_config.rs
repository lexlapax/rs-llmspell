//! ABOUTME: Configuration for automatic agent state persistence
//! ABOUTME: Provides builder pattern for configuring state persistence behavior

use crate::hooks::state_persistence_hook::PersistenceConfig;
use std::time::Duration;

/// Builder for persistence configuration
pub struct PersistenceConfigBuilder {
    config: PersistenceConfig,
}

impl PersistenceConfigBuilder {
    /// Create a new builder with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: PersistenceConfig::default(),
        }
    }

    /// Enable auto-save with specified interval
    #[must_use]
    pub const fn with_auto_save(mut self, interval: Duration) -> Self {
        self.config.auto_save_interval = Some(interval);
        self
    }

    /// Set maximum retry attempts
    #[must_use]
    pub const fn with_max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set backoff multiplier for retries
    #[must_use]
    pub const fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.config.backoff_multiplier = multiplier;
        self
    }

    /// Set failure threshold for circuit breaker
    #[must_use]
    pub const fn with_failure_threshold(mut self, threshold: u32) -> Self {
        self.config.failure_threshold = threshold;
        self
    }

    /// Configure whether to save on pause
    #[must_use]
    pub fn save_on_pause(mut self, enabled: bool) -> Self {
        self.config.event_settings.save_on_pause = enabled;
        self
    }

    /// Configure whether to save on stop
    #[must_use]
    pub fn save_on_stop(mut self, enabled: bool) -> Self {
        self.config.event_settings.save_on_stop = enabled;
        self
    }

    /// Configure whether to restore on resume
    #[must_use]
    pub fn restore_on_resume(mut self, enabled: bool) -> Self {
        self.config.event_settings.restore_on_resume = enabled;
        self
    }

    /// Configure whether saves should be non-blocking
    #[must_use]
    pub fn non_blocking(mut self, enabled: bool) -> Self {
        self.config.event_settings.non_blocking = enabled;
        self
    }

    /// Build the configuration
    #[must_use]
    pub const fn build(self) -> PersistenceConfig {
        self.config
    }
}

impl Default for PersistenceConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Preset configurations for common use cases
pub mod presets {
    use super::{Duration, PersistenceConfig, PersistenceConfigBuilder};

    /// Configuration for development environments
    #[must_use]
    pub fn development() -> PersistenceConfig {
        PersistenceConfigBuilder::new()
            .with_auto_save(Duration::from_secs(60)) // Save every minute
            .with_max_retries(1) // Fail fast in dev
            .save_on_pause(true)
            .save_on_stop(true)
            .restore_on_resume(true)
            .non_blocking(false) // Synchronous in dev for easier debugging
            .build()
    }

    /// Configuration for production environments
    #[must_use]
    pub fn production() -> PersistenceConfig {
        PersistenceConfigBuilder::new()
            .with_auto_save(Duration::from_secs(300)) // Save every 5 minutes
            .with_max_retries(3)
            .with_backoff_multiplier(2.0)
            .with_failure_threshold(5)
            .save_on_pause(true)
            .save_on_stop(true)
            .restore_on_resume(true)
            .non_blocking(true) // Non-blocking in production
            .build()
    }

    /// Configuration for testing
    #[must_use]
    pub fn testing() -> PersistenceConfig {
        PersistenceConfigBuilder::new()
            .with_auto_save(Duration::from_millis(100)) // Fast auto-save for tests
            .with_max_retries(0) // No retries in tests
            .save_on_pause(true)
            .save_on_stop(true)
            .restore_on_resume(true)
            .non_blocking(false) // Synchronous for predictable tests
            .build()
    }

    /// Minimal configuration (only save on stop)
    #[must_use]
    pub fn minimal() -> PersistenceConfig {
        PersistenceConfigBuilder::new()
            .save_on_pause(false)
            .save_on_stop(true)
            .restore_on_resume(false)
            .non_blocking(true)
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_builder_default() {
        let config = PersistenceConfigBuilder::new().build();
        assert!(config.auto_save_interval.is_none());
        assert_eq!(config.max_retries, 3);
        assert!(config.event_settings.save_on_pause);
    }
    #[test]
    fn test_builder_custom() {
        let config = PersistenceConfigBuilder::new()
            .with_auto_save(Duration::from_secs(120))
            .with_max_retries(5)
            .save_on_pause(false)
            .build();

        assert_eq!(config.auto_save_interval, Some(Duration::from_secs(120)));
        assert_eq!(config.max_retries, 5);
        assert!(!config.event_settings.save_on_pause);
    }
    #[test]
    fn test_presets() {
        let dev = presets::development();
        assert_eq!(dev.auto_save_interval, Some(Duration::from_secs(60)));
        assert!(!dev.event_settings.non_blocking);

        let prod = presets::production();
        assert_eq!(prod.auto_save_interval, Some(Duration::from_secs(300)));
        assert!(prod.event_settings.non_blocking);

        let test = presets::testing();
        assert_eq!(test.auto_save_interval, Some(Duration::from_millis(100)));
        assert_eq!(test.max_retries, 0);

        let minimal = presets::minimal();
        assert!(minimal.auto_save_interval.is_none());
        assert!(!minimal.event_settings.save_on_pause);
        assert!(minimal.event_settings.save_on_stop);
    }
}
