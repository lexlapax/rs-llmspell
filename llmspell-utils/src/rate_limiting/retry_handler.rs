//! ABOUTME: Retry logic with exponential and linear backoff strategies
//! ABOUTME: Handles automatic retries for rate-limited requests

use crate::rate_limiting::ProviderRateLimiter;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, warn};

/// Backoff strategy for retries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BackoffStrategy {
    /// No backoff - immediate retry
    None,
    /// Linear backoff - fixed increment
    Linear {
        /// Milliseconds to add for each retry
        increment_ms: u64,
    },
    /// Exponential backoff - doubles each time
    Exponential {
        /// Base delay in milliseconds
        base_ms: u64,
    },
    /// Custom backoff with jitter
    Custom {
        /// Base delay in milliseconds
        base_ms: u64,
        /// Maximum jitter in milliseconds
        max_jitter_ms: u64,
    },
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential { base_ms: 1000 }
    }
}

impl BackoffStrategy {
    /// Calculate delay for a given retry attempt (0-indexed)
    #[must_use]
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        match self {
            Self::None => Duration::ZERO,
            Self::Linear { increment_ms } => {
                Duration::from_millis(*increment_ms * (u64::from(attempt) + 1))
            }
            Self::Exponential { base_ms } => {
                let delay = *base_ms * 2u64.pow(attempt);
                // Cap at 5 minutes to prevent excessive delays
                Duration::from_millis(delay.min(300_000))
            }
            Self::Custom {
                base_ms,
                max_jitter_ms,
            } => {
                let base_delay = *base_ms * 2u64.pow(attempt);
                let jitter = rand::random::<u64>() % (*max_jitter_ms + 1);
                Duration::from_millis(base_delay + jitter)
            }
        }
    }
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Backoff strategy to use
    pub backoff_strategy: BackoffStrategy,
    /// Whether to retry on rate limit errors
    pub retry_on_rate_limit: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 3,
            backoff_strategy: BackoffStrategy::default(),
            retry_on_rate_limit: true,
        }
    }
}

/// Handles retry logic for rate-limited operations
pub struct RetryHandler;

impl Default for RetryHandler {
    fn default() -> Self {
        Self
    }
}

impl RetryHandler {
    /// Execute an operation with retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The operation fails after all retry attempts are exhausted
    /// - Rate limit is exceeded and `retry_on_rate_limit` is false
    /// - The provided operation returns an error
    ///
    /// # Panics
    ///
    /// This function will panic if no error was recorded after retry attempts are exhausted,
    /// which should never happen in practice as at least one operation attempt is always made.
    pub async fn execute_with_retry<F, T>(
        &self,
        provider: &str,
        operation: F,
        policy: RetryPolicy,
        rate_limiter: Option<ProviderRateLimiter>,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<T, Box<dyn std::error::Error + Send + Sync>>,
                        > + Send,
                >,
            > + Send,
    {
        let mut last_error = None;

        for attempt in 0..=policy.max_retries {
            // Check rate limit before attempting
            if let Some(ref limiter) = rate_limiter {
                match limiter.check_rate_limit(provider).await {
                    Ok(()) => {
                        debug!("Rate limit check passed for provider: {}", provider);
                    }
                    Err(e) => {
                        if !policy.retry_on_rate_limit {
                            error!("Rate limit exceeded for provider {}: {:?}", provider, e);
                            return Err(last_error.unwrap_or_else(|| {
                                // Create a generic error if we don't have one
                                Box::new(std::io::Error::new(
                                    std::io::ErrorKind::Other,
                                    format!("Rate limit exceeded for provider: {provider}"),
                                ))
                                    as Box<dyn std::error::Error + Send + Sync>
                            }));
                        }

                        warn!(
                            "Rate limit exceeded for provider {}, attempt {}/{}: {:?}",
                            provider,
                            attempt + 1,
                            policy.max_retries + 1,
                            e
                        );

                        // Calculate backoff delay
                        let delay = policy.backoff_strategy.calculate_delay(attempt);
                        if attempt < policy.max_retries {
                            debug!("Waiting {:?} before retry", delay);
                            sleep(delay).await;
                            continue;
                        }
                    }
                }
            }

            // Execute the operation
            match operation().await {
                Ok(result) => {
                    if attempt > 0 {
                        debug!(
                            "Operation succeeded for provider {} after {} retries",
                            provider, attempt
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error_msg = format!("{e:?}");

                    // Check if this is a rate limit error
                    let is_rate_limit = error_msg.to_lowercase().contains("rate limit")
                        || error_msg.contains("429")
                        || error_msg.contains("too many requests");

                    if is_rate_limit && !policy.retry_on_rate_limit {
                        error!("Rate limit error for provider {}: {}", provider, error_msg);
                        return Err(e);
                    }

                    warn!(
                        "Operation failed for provider {}, attempt {}/{}: {}",
                        provider,
                        attempt + 1,
                        policy.max_retries + 1,
                        error_msg
                    );

                    last_error = Some(e);

                    if attempt < policy.max_retries {
                        let delay = policy.backoff_strategy.calculate_delay(attempt);
                        debug!("Waiting {:?} before retry", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        error!(
            "All retry attempts exhausted for provider {} after {} attempts",
            provider,
            policy.max_retries + 1
        );

        Err(last_error.expect("Should have at least one error after retries"))
    }

    /// Execute with simple retry (no rate limiting)
    ///
    /// # Errors
    ///
    /// Returns an error if the operation fails after all retry attempts are exhausted.
    pub async fn simple_retry<F, T>(
        &self,
        operation: F,
        max_retries: u32,
        backoff: BackoffStrategy,
    ) -> Result<T, Box<dyn std::error::Error + Send + Sync>>
    where
        F: Fn() -> std::pin::Pin<
                Box<
                    dyn std::future::Future<
                            Output = Result<T, Box<dyn std::error::Error + Send + Sync>>,
                        > + Send,
                >,
            > + Send,
    {
        let policy = RetryPolicy {
            max_retries,
            backoff_strategy: backoff,
            retry_on_rate_limit: true,
        };

        self.execute_with_retry("generic", operation, policy, None)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    #[test]
    fn test_backoff_strategies() {
        let linear = BackoffStrategy::Linear { increment_ms: 100 };
        assert_eq!(linear.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(linear.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(linear.calculate_delay(2), Duration::from_millis(300));

        let exponential = BackoffStrategy::Exponential { base_ms: 100 };
        assert_eq!(exponential.calculate_delay(0), Duration::from_millis(100));
        assert_eq!(exponential.calculate_delay(1), Duration::from_millis(200));
        assert_eq!(exponential.calculate_delay(2), Duration::from_millis(400));

        // Test cap at 5 minutes
        assert_eq!(
            exponential.calculate_delay(20),
            Duration::from_millis(300_000)
        );
    }

    #[tokio::test]
    async fn test_retry_handler_success() {
        let handler = RetryHandler::default();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result = handler
            .simple_retry(
                move || {
                    let attempts = Arc::clone(&attempts_clone);
                    Box::pin(async move {
                        let count = attempts.fetch_add(1, Ordering::SeqCst);
                        if count < 2 {
                            Err(Box::new(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "Simulated failure",
                            ))
                                as Box<dyn std::error::Error + Send + Sync>)
                        } else {
                            Ok("Success")
                        }
                    })
                },
                3,
                BackoffStrategy::None,
            )
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(attempts.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_handler_exhausted() {
        let handler = RetryHandler::default();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = Arc::clone(&attempts);

        let result: Result<String, Box<dyn std::error::Error + Send + Sync>> = handler
            .simple_retry(
                move || {
                    let attempts = Arc::clone(&attempts_clone);
                    Box::pin(async move {
                        attempts.fetch_add(1, Ordering::SeqCst);
                        Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Always fails",
                        ))
                            as Box<dyn std::error::Error + Send + Sync>)
                    })
                },
                2,
                BackoffStrategy::None,
            )
            .await;

        assert!(result.is_err());
        assert_eq!(attempts.load(Ordering::SeqCst), 3); // initial + 2 retries
    }
}
