//! ABOUTME: Provider-specific rate limiting framework for external API integrations
//! ABOUTME: Manages rate limits per API provider with automatic retry and backoff

use crate::rate_limiter::{RateLimitError, RateLimiter, RateLimiterBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
#[cfg(feature = "rate-limiting-http")]
use tracing::warn;
use tracing::{debug, info};

mod metrics;
mod provider_limits;
mod retry_handler;

pub use metrics::{MetricsCollector, RateLimitMetrics};
pub use provider_limits::{ProviderLimits, RateLimitConfig as ProviderRateLimitConfig};
pub use retry_handler::{BackoffStrategy, RetryHandler, RetryPolicy};

/// Rate limit information from API response headers
#[derive(Debug, Clone)]
pub struct RateLimitInfo {
    /// Requests remaining in current window
    pub remaining: Option<u32>,
    /// Total limit for the window
    pub limit: Option<u32>,
    /// When the rate limit resets (Unix timestamp)
    pub reset_at: Option<u64>,
    /// Time to wait before retrying (from Retry-After header)
    pub retry_after: Option<Duration>,
}

impl RateLimitInfo {
    /// Parse rate limit information from HTTP headers
    #[cfg(feature = "rate-limiting-http")]
    #[must_use]
    pub fn from_headers(headers: &reqwest::header::HeaderMap) -> Self {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .or_else(|| headers.get("x-rate-limit-remaining"))
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());

        let limit = headers
            .get("x-ratelimit-limit")
            .or_else(|| headers.get("x-rate-limit-limit"))
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u32>().ok());

        let reset_at = headers
            .get("x-ratelimit-reset")
            .or_else(|| headers.get("x-rate-limit-reset"))
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok());

        let retry_after = headers
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| {
                // Try to parse as seconds first, then as HTTP date
                v.parse::<u64>().map(Duration::from_secs).ok()
            });

        Self {
            remaining,
            limit,
            reset_at,
            retry_after,
        }
    }

    /// Calculate wait time based on rate limit info
    ///
    /// # Panics
    ///
    /// This function will panic if the system time is before the Unix epoch,
    /// which should never happen on systems with correctly set clocks.
    #[must_use]
    pub fn wait_time(&self) -> Option<Duration> {
        if let Some(retry_after) = self.retry_after {
            return Some(retry_after);
        }

        if let Some(remaining) = self.remaining {
            if remaining == 0 {
                if let Some(reset_at) = self.reset_at {
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs();

                    if reset_at > now {
                        return Some(Duration::from_secs(reset_at - now));
                    }
                }
            }
        }

        None
    }
}

/// Provider-aware rate limiter
pub struct ProviderRateLimiter {
    /// Rate limiters per provider
    limiters: Arc<RwLock<HashMap<String, Arc<RateLimiter>>>>,
    /// Provider configurations
    provider_configs: Arc<RwLock<HashMap<String, ProviderRateLimitConfig>>>,
    /// Retry handler
    retry_handler: Arc<RetryHandler>,
    /// Metrics collector
    metrics: Arc<MetricsCollector>,
}

impl Default for ProviderRateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProviderRateLimiter {
    /// Create a new provider rate limiter
    #[must_use]
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            provider_configs: Arc::new(RwLock::new(HashMap::new())),
            retry_handler: Arc::new(RetryHandler),
            metrics: Arc::new(MetricsCollector::new()),
        }
    }

    /// Create with provider configurations
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError` if any of the rate limiter configurations are invalid.
    pub async fn with_configs(
        configs: HashMap<String, ProviderRateLimitConfig>,
    ) -> Result<Self, RateLimitError> {
        let mut limiter = Self::new();

        for (provider, config) in configs {
            limiter.add_provider(&provider, config).await?;
        }

        Ok(limiter)
    }

    /// Add a provider with rate limit configuration
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError` if the rate limiter configuration is invalid.
    pub async fn add_provider(
        &mut self,
        provider: &str,
        config: ProviderRateLimitConfig,
    ) -> Result<(), RateLimitError> {
        let mut builder = RateLimiterBuilder::default().per_minute(config.requests_per_minute);

        if config.allow_burst {
            builder = builder.allow_burst();
        }

        let rate_limiter = builder.build()?;

        let mut limiters = self.limiters.write().await;
        let mut configs = self.provider_configs.write().await;

        limiters.insert(provider.to_string(), Arc::new(rate_limiter));
        configs.insert(provider.to_string(), config);

        info!("Added rate limiter for provider: {}", provider);
        Ok(())
    }

    /// Check if a request is allowed for a provider
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError` if the rate limit has been exceeded for the provider.
    pub async fn check_rate_limit(&self, provider: &str) -> Result<(), RateLimitError> {
        let limiters = self.limiters.read().await;

        if let Some(limiter) = limiters.get(provider) {
            match limiter.try_acquire().await {
                Ok(()) => {
                    self.metrics.record_allowed(provider).await;
                    Ok(())
                }
                Err(e) => {
                    self.metrics.record_denied(provider).await;
                    Err(e)
                }
            }
        } else {
            // No rate limiter configured for this provider, allow by default
            debug!("No rate limiter configured for provider: {}", provider);
            Ok(())
        }
    }

    /// Execute a request with rate limiting and retry logic
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The operation fails after all retry attempts are exhausted
    /// - Rate limit is exceeded and retries are disabled
    /// - The provided operation returns an error
    pub async fn execute_with_retry<F, T>(
        &self,
        provider: &str,
        operation: F,
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
        let configs = self.provider_configs.read().await;
        let retry_policy = if let Some(config) = configs.get(provider) {
            RetryPolicy {
                max_retries: config.max_retries,
                backoff_strategy: config.backoff_strategy.clone(),
                retry_on_rate_limit: true,
            }
        } else {
            RetryPolicy::default()
        };

        self.retry_handler
            .execute_with_retry(provider, operation, retry_policy, Some(self.clone()))
            .await
    }

    /// Update rate limit from response headers
    #[cfg(feature = "rate-limiting-http")]
    pub async fn update_from_headers(&self, provider: &str, headers: &reqwest::header::HeaderMap) {
        let info = RateLimitInfo::from_headers(headers);

        if let Some(wait_time) = info.wait_time() {
            warn!(
                "Rate limit approaching for provider {}: {:?} remaining, wait {:?}",
                provider, info.remaining, wait_time
            );

            // Record metrics
            if let Some(remaining) = info.remaining {
                self.metrics.update_remaining(provider, remaining).await;
            }
        }
    }

    /// Get current metrics for a provider
    pub async fn get_metrics(&self, provider: &str) -> Option<RateLimitMetrics> {
        self.metrics.get_provider_metrics(provider).await
    }

    /// Get all provider metrics
    pub async fn get_all_metrics(&self) -> HashMap<String, RateLimitMetrics> {
        self.metrics.get_all_metrics().await
    }
}

impl Clone for ProviderRateLimiter {
    fn clone(&self) -> Self {
        Self {
            limiters: Arc::clone(&self.limiters),
            provider_configs: Arc::clone(&self.provider_configs),
            retry_handler: Arc::clone(&self.retry_handler),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "rate-limiting-http")]
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    #[cfg(feature = "rate-limiting-http")]
    fn test_rate_limit_info_from_headers() {
        let mut headers = HeaderMap::new();
        headers.insert("x-ratelimit-remaining", HeaderValue::from_static("42"));
        headers.insert("x-ratelimit-limit", HeaderValue::from_static("100"));
        headers.insert("x-ratelimit-reset", HeaderValue::from_static("1234567890"));

        let info = RateLimitInfo::from_headers(&headers);
        assert_eq!(info.remaining, Some(42));
        assert_eq!(info.limit, Some(100));
        assert_eq!(info.reset_at, Some(1234567890));
    }

    #[tokio::test]
    async fn test_provider_rate_limiter() {
        let mut limiter = ProviderRateLimiter::new();

        let config = ProviderRateLimitConfig {
            requests_per_minute: 60,
            requests_per_hour: Some(1000),
            daily_limit: None,
            allow_burst: true,
            max_retries: 3,
            backoff_strategy: BackoffStrategy::Exponential { base_ms: 100 },
        };

        limiter.add_provider("test_provider", config).await.unwrap();

        // First request should succeed
        assert!(limiter.check_rate_limit("test_provider").await.is_ok());

        // Check metrics
        let metrics = limiter.get_metrics("test_provider").await.unwrap();
        assert_eq!(metrics.requests_allowed, 1);
        assert_eq!(metrics.requests_denied, 0);
    }
}
