//! ABOUTME: Circuit breaker configuration and thresholds
//! ABOUTME: Defines failure thresholds, timeouts, and recovery settings

use std::fmt;
use std::time::Duration;

/// Alert handler function type
pub type AlertHandler = Box<dyn Fn(String) + Send + Sync>;

/// Circuit breaker configuration
pub struct CircuitBreakerConfig {
    /// Number of consecutive failures to open circuit
    pub failure_threshold_count: u32,

    /// Failure rate percentage to open circuit (0-100)
    pub failure_threshold_percentage: Option<f32>,

    /// Time window for failure rate calculation
    pub failure_window: Option<Duration>,

    /// Time to wait before attempting to close circuit
    pub reset_timeout: Duration,

    /// Number of successes needed in half-open to close circuit
    pub success_threshold_count: u32,

    /// Number of test requests allowed in half-open state
    pub test_request_count: u32,

    /// Optional alert handler for circuit state changes
    pub alert_handler: Option<AlertHandler>,
}

impl fmt::Debug for CircuitBreakerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CircuitBreakerConfig")
            .field("failure_threshold_count", &self.failure_threshold_count)
            .field(
                "failure_threshold_percentage",
                &self.failure_threshold_percentage,
            )
            .field("failure_window", &self.failure_window)
            .field("reset_timeout", &self.reset_timeout)
            .field("success_threshold_count", &self.success_threshold_count)
            .field("test_request_count", &self.test_request_count)
            .field("alert_handler", &self.alert_handler.is_some())
            .finish()
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold_count: 5,
            failure_threshold_percentage: None,
            failure_window: None,
            reset_timeout: Duration::from_secs(60),
            success_threshold_count: 3,
            test_request_count: 5,
            alert_handler: None,
        }
    }
}

impl CircuitBreakerConfig {
    /// Create a new configuration with default values
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder-style method to set failure threshold count
    #[must_use]
    pub fn with_failure_threshold(mut self, count: u32) -> Self {
        self.failure_threshold_count = count;
        self
    }

    /// Builder-style method to set failure threshold percentage
    #[must_use]
    pub fn with_failure_percentage(mut self, percentage: f32, window: Duration) -> Self {
        self.failure_threshold_percentage = Some(percentage);
        self.failure_window = Some(window);
        self
    }

    /// Builder-style method to set reset timeout
    #[must_use]
    pub fn with_reset_timeout(mut self, timeout: Duration) -> Self {
        self.reset_timeout = timeout;
        self
    }

    /// Builder-style method to set success threshold
    #[must_use]
    pub fn with_success_threshold(mut self, count: u32) -> Self {
        self.success_threshold_count = count;
        self
    }

    /// Builder-style method to set test request count
    #[must_use]
    pub fn with_test_request_count(mut self, count: u32) -> Self {
        self.test_request_count = count;
        self
    }

    /// Builder-style method to set alert handler
    #[must_use]
    pub fn with_alert_handler<F>(mut self, handler: F) -> Self
    where
        F: Fn(String) + Send + Sync + 'static,
    {
        self.alert_handler = Some(Box::new(handler));
        self
    }

    /// Validate configuration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failure threshold count is 0
    /// - Failure percentage is outside 0-100 range
    /// - Failure window is not set when using percentage threshold
    /// - Success threshold count is 0
    /// - Test request count is less than success threshold count
    pub fn validate(&self) -> Result<(), String> {
        if self.failure_threshold_count == 0 {
            return Err("Failure threshold count must be greater than 0".to_string());
        }

        if let Some(percentage) = self.failure_threshold_percentage {
            if !(0.0..=100.0).contains(&percentage) {
                return Err("Failure percentage must be between 0 and 100".to_string());
            }

            if self.failure_window.is_none() {
                return Err(
                    "Failure window must be set when using percentage threshold".to_string()
                );
            }
        }

        if self.success_threshold_count == 0 {
            return Err("Success threshold count must be greater than 0".to_string());
        }

        if self.test_request_count < self.success_threshold_count {
            return Err("Test request count must be >= success threshold count".to_string());
        }

        Ok(())
    }
}

/// Threshold configuration for different severity levels
#[derive(Debug)]
pub struct ThresholdConfig {
    /// Warning level configuration
    pub warning: CircuitBreakerConfig,
    /// Critical level configuration
    pub critical: CircuitBreakerConfig,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            warning: CircuitBreakerConfig::new()
                .with_failure_threshold(3)
                .with_reset_timeout(Duration::from_secs(30)),
            critical: CircuitBreakerConfig::new()
                .with_failure_threshold(5)
                .with_reset_timeout(Duration::from_secs(60)),
        }
    }
}

/// Pre-configured settings for common services
pub struct ServicePresets;

impl ServicePresets {
    /// Configuration for HTTP APIs
    #[must_use]
    pub fn http_api() -> CircuitBreakerConfig {
        CircuitBreakerConfig::new()
            .with_failure_threshold(5)
            .with_reset_timeout(Duration::from_secs(30))
            .with_success_threshold(2)
            .with_test_request_count(3)
    }

    /// Configuration for database connections
    #[must_use]
    pub fn database() -> CircuitBreakerConfig {
        CircuitBreakerConfig::new()
            .with_failure_threshold(3)
            .with_reset_timeout(Duration::from_secs(60))
            .with_success_threshold(1)
            .with_test_request_count(2)
    }

    /// Configuration for message queues
    #[must_use]
    pub fn message_queue() -> CircuitBreakerConfig {
        CircuitBreakerConfig::new()
            .with_failure_threshold(10)
            .with_reset_timeout(Duration::from_secs(120))
            .with_success_threshold(5)
            .with_test_request_count(10)
    }

    /// Configuration for critical services (more conservative)
    #[must_use]
    pub fn critical_service() -> CircuitBreakerConfig {
        CircuitBreakerConfig::new()
            .with_failure_threshold(2)
            .with_reset_timeout(Duration::from_secs(300))
            .with_success_threshold(5)
            .with_test_request_count(10)
    }

    /// Configuration for high-volume services (more lenient)
    #[must_use]
    pub fn high_volume() -> CircuitBreakerConfig {
        CircuitBreakerConfig::new()
            .with_failure_threshold(20)
            .with_failure_percentage(50.0, Duration::from_secs(60))
            .with_reset_timeout(Duration::from_secs(30))
            .with_success_threshold(10)
            .with_test_request_count(20)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_config_validation() {
        // Valid config
        let config = CircuitBreakerConfig::new();
        assert!(config.validate().is_ok());

        // Invalid: zero failure threshold
        let config = CircuitBreakerConfig {
            failure_threshold_count: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());

        // Invalid: percentage out of range
        let config =
            CircuitBreakerConfig::new().with_failure_percentage(150.0, Duration::from_secs(60));
        assert!(config.validate().is_err());

        // Invalid: test count < success threshold
        let config = CircuitBreakerConfig {
            success_threshold_count: 5,
            test_request_count: 3,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_builder_methods() {
        let config = CircuitBreakerConfig::new()
            .with_failure_threshold(10)
            .with_reset_timeout(Duration::from_secs(120))
            .with_success_threshold(5);

        assert_eq!(config.failure_threshold_count, 10);
        assert_eq!(config.reset_timeout, Duration::from_secs(120));
        assert_eq!(config.success_threshold_count, 5);
    }
}
