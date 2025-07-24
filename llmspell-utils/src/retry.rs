// ABOUTME: Retry utility with exponential backoff and configurable retry strategies
// ABOUTME: Provides a common retry mechanism for operations that may fail temporarily

use std::fmt::Display;
use std::future::Future;
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration with exponential backoff
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    /// Initial delay between retries
    pub initial_delay: Duration,
    /// Maximum delay between retries
    pub max_delay: Duration,
    /// Exponential backoff factor (e.g., 2.0 for doubling)
    pub backoff_factor: f64,
    /// Optional jitter to add randomness to delays
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(1000),
            max_delay: Duration::from_secs(30),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    #[must_use]
    pub fn new(max_attempts: u32) -> Self {
        Self {
            max_attempts,
            ..Default::default()
        }
    }

    /// Set the initial delay
    #[must_use]
    pub fn with_initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Set the maximum delay
    #[must_use]
    pub fn with_max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Set the backoff factor
    #[must_use]
    pub fn with_backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor;
        self
    }

    /// Enable or disable jitter
    #[must_use]
    pub fn with_jitter(mut self, jitter: bool) -> Self {
        self.jitter = jitter;
        self
    }

    /// Calculate delay for a given attempt number
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_lossless
    )]
    fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::ZERO;
        }

        let exponential_delay =
            self.initial_delay.as_millis() as f64 * self.backoff_factor.powf((attempt - 1) as f64);

        let mut delay_ms = exponential_delay.min(self.max_delay.as_millis() as f64) as u64;

        // Add jitter if enabled (up to 20% variation)
        if self.jitter {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let jitter_factor = rng.gen_range(0.8..1.2);
            delay_ms = (delay_ms as f64 * jitter_factor) as u64;
        }

        Duration::from_millis(delay_ms)
    }
}

/// Error type for retry operations
#[derive(Debug, Error)]
pub enum RetryError<E> {
    #[error("Operation failed after {attempts} attempts: {error}")]
    /// All retry attempts have been exhausted
    ExhaustedRetries {
        /// Number of attempts made
        attempts: u32,
        /// The final error
        error: E,
    },

    #[error("Operation was cancelled")]
    /// The retry operation was cancelled
    Cancelled,
}

/// Retry policy determines which errors should trigger a retry
pub trait RetryPolicy<E> {
    /// Determine if an error should trigger a retry
    fn should_retry(&self, error: &E) -> bool;
}

/// Default retry policy that retries on all errors
pub struct AlwaysRetry;

impl<E> RetryPolicy<E> for AlwaysRetry {
    fn should_retry(&self, _error: &E) -> bool {
        true
    }
}

/// HTTP status code based retry policy
pub struct HttpStatusRetryPolicy {
    /// HTTP status codes that should trigger a retry
    pub retry_on_status: Vec<u16>,
}

impl HttpStatusRetryPolicy {
    #[must_use]
    /// Create a new HTTP retry strategy
    pub fn new(status_codes: Vec<u16>) -> Self {
        Self {
            retry_on_status: status_codes,
        }
    }

    /// Default HTTP status codes that should trigger retry
    #[must_use]
    pub fn default_retry_codes() -> Vec<u16> {
        vec![429, 500, 502, 503, 504]
    }
}

/// Retry an async operation with the given configuration
///
/// # Errors
///
/// Returns `RetryError::ExhaustedRetries` if all retry attempts fail or if the error is not retryable.
/// Returns `RetryError::Cancelled` if the operation is cancelled (not currently implemented).
pub async fn retry<F, Fut, T, E, P>(
    config: RetryConfig,
    policy: P,
    operation: F,
) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: RetryPolicy<E>,
    E: Display,
{
    let mut attempt = 0;

    loop {
        attempt += 1;
        debug!("Attempt {}/{}", attempt, config.max_attempts);

        match operation().await {
            Ok(result) => {
                if attempt > 1 {
                    debug!("Operation succeeded after {} attempts", attempt);
                }
                return Ok(result);
            }
            Err(error) => {
                if attempt >= config.max_attempts {
                    warn!("Operation failed after {} attempts: {}", attempt, error);
                    return Err(RetryError::ExhaustedRetries {
                        attempts: attempt,
                        error,
                    });
                }

                if !policy.should_retry(&error) {
                    debug!("Error is not retryable: {}", error);
                    return Err(RetryError::ExhaustedRetries {
                        attempts: attempt,
                        error,
                    });
                }

                let delay = config.calculate_delay(attempt);
                warn!(
                    "Attempt {} failed: {}. Retrying in {:?}",
                    attempt, error, delay
                );

                sleep(delay).await;
            }
        }
    }
}

/// Convenience function to retry with default configuration
///
/// # Errors
///
/// Returns `RetryError::ExhaustedRetries` if all retry attempts fail.
pub async fn retry_default<F, Fut, T, E>(operation: F) -> Result<T, RetryError<E>>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: Display,
{
    retry(RetryConfig::default(), AlwaysRetry, operation).await
}

/// Builder pattern for retry operations
pub struct RetryBuilder<P> {
    config: RetryConfig,
    policy: P,
}

impl Default for RetryBuilder<AlwaysRetry> {
    fn default() -> Self {
        Self {
            config: RetryConfig::default(),
            policy: AlwaysRetry,
        }
    }
}

impl<P> RetryBuilder<P> {
    /// Create a new retry builder with a policy
    #[must_use]
    pub fn with_policy(policy: P) -> Self {
        Self {
            config: RetryConfig::default(),
            policy,
        }
    }

    /// Set maximum attempts
    #[must_use]
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.config.max_attempts = attempts;
        self
    }

    /// Set initial delay
    #[must_use]
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.config.initial_delay = delay;
        self
    }

    /// Set maximum delay
    #[must_use]
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.config.max_delay = delay;
        self
    }

    /// Set backoff factor
    #[must_use]
    pub fn backoff_factor(mut self, factor: f64) -> Self {
        self.config.backoff_factor = factor;
        self
    }

    /// Enable or disable jitter
    #[must_use]
    pub fn jitter(mut self, jitter: bool) -> Self {
        self.config.jitter = jitter;
        self
    }

    /// Execute the retry operation
    ///
    /// # Errors
    ///
    /// Returns `RetryError::ExhaustedRetries` if all retry attempts fail or if the error is not retryable.
    pub async fn execute<F, Fut, T, E>(self, operation: F) -> Result<T, RetryError<E>>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        P: RetryPolicy<E>,
        E: Display,
    {
        retry(self.config, self.policy, operation).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_successful_operation() {
        let result = retry_default(|| async { Ok::<_, &str>(42) }).await;
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_retry_on_failure() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = counter.clone();

        let result = retry_default(|| {
            let count = counter_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                if count < 2 {
                    Err("temporary failure")
                } else {
                    Ok(42)
                }
            }
        })
        .await;

        assert_eq!(result.unwrap(), 42);
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_exhausted_retries() {
        let result = retry_default(|| async { Err::<i32, _>("permanent failure") }).await;

        match result {
            Err(RetryError::ExhaustedRetries { attempts, error }) => {
                assert_eq!(attempts, 3);
                assert_eq!(error, "permanent failure");
            }
            _ => panic!("Expected ExhaustedRetries error"),
        }
    }

    #[tokio::test]
    async fn test_custom_retry_policy() {
        struct CustomPolicy;

        impl RetryPolicy<&str> for CustomPolicy {
            fn should_retry(&self, error: &&str) -> bool {
                *error == "retryable"
            }
        }

        let result = RetryBuilder::with_policy(CustomPolicy)
            .max_attempts(2)
            .execute(|| async { Err::<i32, _>("not retryable") })
            .await;

        match result {
            Err(RetryError::ExhaustedRetries { attempts, .. }) => {
                assert_eq!(attempts, 1); // Should not retry
            }
            _ => panic!("Expected ExhaustedRetries error"),
        }
    }

    #[test]
    fn test_delay_calculation() {
        let config = RetryConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: false,
            ..Default::default()
        };

        assert_eq!(config.calculate_delay(0), Duration::ZERO);
        assert_eq!(config.calculate_delay(1), Duration::from_millis(100));
        assert_eq!(config.calculate_delay(2), Duration::from_millis(200));
        assert_eq!(config.calculate_delay(3), Duration::from_millis(400));
        assert_eq!(config.calculate_delay(10), Duration::from_secs(10)); // Capped at max
    }
}
