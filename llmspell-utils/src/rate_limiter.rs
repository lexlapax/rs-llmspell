// ABOUTME: Rate limiting utility with multiple algorithms (token bucket, sliding window)
// ABOUTME: Provides a common rate limiting mechanism for API calls and resource usage

use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Rate limiter error types
#[derive(Debug, Error)]
pub enum RateLimitError {
    #[error("Rate limit exceeded. Please wait {wait_time:?} before retrying")]
    /// Rate limit has been exceeded
    RateLimitExceeded {
        /// Time to wait before retrying
        wait_time: Duration,
    },

    #[error("Invalid rate limit configuration: {message}")]
    /// Invalid rate limiter configuration
    InvalidConfiguration {
        /// Error message describing the issue
        message: String,
    },
}

/// Rate limiting algorithm
#[derive(Debug, Clone)]
pub enum RateLimitAlgorithm {
    /// Token bucket algorithm - allows bursts
    TokenBucket,
    /// Sliding window algorithm - smooth rate limiting
    SlidingWindow,
    /// Fixed window algorithm - simple time-based windows
    FixedWindow,
}

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Maximum number of requests allowed
    pub max_requests: u32,
    /// Time window for the rate limit
    pub window: Duration,
    /// Algorithm to use
    pub algorithm: RateLimitAlgorithm,
    /// Whether to allow bursts (for token bucket)
    pub allow_burst: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 60,
            window: Duration::from_secs(60),
            algorithm: RateLimitAlgorithm::SlidingWindow,
            allow_burst: false,
        }
    }
}

impl RateLimiterConfig {
    /// Create a rate limiter for requests per second
    #[must_use]
    pub fn per_second(requests: u32) -> Self {
        Self {
            max_requests: requests,
            window: Duration::from_secs(1),
            ..Default::default()
        }
    }

    /// Create a rate limiter for requests per minute
    #[must_use]
    pub fn per_minute(requests: u32) -> Self {
        Self {
            max_requests: requests,
            window: Duration::from_secs(60),
            ..Default::default()
        }
    }

    /// Create a rate limiter for requests per hour
    #[must_use]
    pub fn per_hour(requests: u32) -> Self {
        Self {
            max_requests: requests,
            window: Duration::from_secs(3600),
            ..Default::default()
        }
    }

    /// Set the algorithm
    #[must_use]
    pub fn with_algorithm(mut self, algorithm: RateLimitAlgorithm) -> Self {
        self.algorithm = algorithm;
        self
    }

    /// Enable burst mode (for token bucket)
    #[must_use]
    pub fn with_burst(mut self) -> Self {
        self.allow_burst = true;
        self.algorithm = RateLimitAlgorithm::TokenBucket;
        self
    }
}

/// Base trait for rate limiting implementations
#[async_trait::async_trait]
trait RateLimitStrategy: Send + Sync {
    /// Try to acquire a permit
    async fn try_acquire(&self) -> Result<(), Duration>;

    /// Get the current availability
    async fn available_permits(&self) -> u32;
}

/// Sliding window rate limiter implementation
struct SlidingWindowLimiter {
    config: RateLimiterConfig,
    requests: Arc<Mutex<Vec<Instant>>>,
}

impl SlidingWindowLimiter {
    fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            requests: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl RateLimitStrategy for SlidingWindowLimiter {
    async fn try_acquire(&self) -> Result<(), Duration> {
        let now = Instant::now();
        let window_start = now.checked_sub(self.config.window).unwrap_or(now);

        let mut requests = self.requests.lock().await;

        // Remove old requests outside the window
        requests.retain(|&time| time > window_start);

        if requests.len() < self.config.max_requests as usize {
            requests.push(now);
            debug!(
                "Rate limit: {}/{} requests used",
                requests.len(),
                self.config.max_requests
            );
            Ok(())
        } else {
            // Calculate wait time until the oldest request expires
            let wait_time = (requests[0] + self.config.window).saturating_duration_since(now);
            warn!(
                "Rate limit exceeded: {}/{} requests",
                requests.len(),
                self.config.max_requests
            );
            Err(wait_time)
        }
    }

    async fn available_permits(&self) -> u32 {
        let now = Instant::now();
        let window_start = now.checked_sub(self.config.window).unwrap_or(now);

        let mut requests = self.requests.lock().await;
        requests.retain(|&time| time > window_start);

        self.config
            .max_requests
            .saturating_sub(requests.len().try_into().unwrap_or(u32::MAX))
    }
}

/// Token bucket rate limiter implementation
struct TokenBucketLimiter {
    config: RateLimiterConfig,
    tokens: Arc<Mutex<f64>>,
    last_refill: Arc<Mutex<Instant>>,
}

impl TokenBucketLimiter {
    fn new(config: RateLimiterConfig) -> Self {
        let initial_tokens = if config.allow_burst {
            f64::from(config.max_requests)
        } else {
            0.0
        };

        Self {
            config,
            tokens: Arc::new(Mutex::new(initial_tokens)),
            last_refill: Arc::new(Mutex::new(Instant::now())),
        }
    }

    async fn refill(&self) {
        let now = Instant::now();
        let mut tokens = self.tokens.lock().await;
        let mut last_refill = self.last_refill.lock().await;

        let elapsed = now.duration_since(*last_refill);
        let refill_rate = f64::from(self.config.max_requests) / self.config.window.as_secs_f64();
        let new_tokens = elapsed.as_secs_f64() * refill_rate;

        *tokens = (*tokens + new_tokens).min(f64::from(self.config.max_requests));
        *last_refill = now;
    }
}

#[async_trait::async_trait]
impl RateLimitStrategy for TokenBucketLimiter {
    async fn try_acquire(&self) -> Result<(), Duration> {
        self.refill().await;

        let mut tokens = self.tokens.lock().await;

        if *tokens >= 1.0 {
            *tokens -= 1.0;
            debug!("Token bucket: {:.1} tokens remaining", *tokens);
            Ok(())
        } else {
            // Calculate wait time for next token
            let refill_rate =
                f64::from(self.config.max_requests) / self.config.window.as_secs_f64();
            let wait_secs = (1.0 - *tokens) / refill_rate;
            let wait_time = Duration::from_secs_f64(wait_secs);

            warn!(
                "Token bucket empty: {:.1} tokens, wait {:?}",
                *tokens, wait_time
            );
            Err(wait_time)
        }
    }

    async fn available_permits(&self) -> u32 {
        self.refill().await;
        let tokens = self.tokens.lock().await;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            *tokens as u32
        }
    }
}

/// Fixed window rate limiter implementation
struct FixedWindowLimiter {
    config: RateLimiterConfig,
    window_start: Arc<Mutex<Instant>>,
    count: Arc<Mutex<u32>>,
}

impl FixedWindowLimiter {
    fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            window_start: Arc::new(Mutex::new(Instant::now())),
            count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait::async_trait]
impl RateLimitStrategy for FixedWindowLimiter {
    async fn try_acquire(&self) -> Result<(), Duration> {
        let now = Instant::now();
        let mut window_start = self.window_start.lock().await;
        let mut count = self.count.lock().await;

        // Check if we need to reset the window
        if now.duration_since(*window_start) >= self.config.window {
            *window_start = now;
            *count = 0;
        }

        if *count < self.config.max_requests {
            *count += 1;
            debug!(
                "Fixed window: {}/{} requests",
                *count, self.config.max_requests
            );
            Ok(())
        } else {
            let wait_time = (*window_start + self.config.window).saturating_duration_since(now);
            warn!(
                "Fixed window limit: {}/{} requests",
                *count, self.config.max_requests
            );
            Err(wait_time)
        }
    }

    async fn available_permits(&self) -> u32 {
        let now = Instant::now();
        let window_start = self.window_start.lock().await;
        let count = self.count.lock().await;

        if now.duration_since(*window_start) >= self.config.window {
            self.config.max_requests
        } else {
            self.config.max_requests.saturating_sub(*count)
        }
    }
}

/// Main rate limiter struct
pub struct RateLimiter {
    strategy: Box<dyn RateLimitStrategy>,
}

impl RateLimiter {
    /// Create a new rate limiter with the given configuration
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError::InvalidConfiguration` if `max_requests` is 0.
    pub fn new(config: RateLimiterConfig) -> Result<Self, RateLimitError> {
        if config.max_requests == 0 {
            return Err(RateLimitError::InvalidConfiguration {
                message: "max_requests must be greater than 0".to_string(),
            });
        }

        let strategy: Box<dyn RateLimitStrategy> = match config.algorithm {
            RateLimitAlgorithm::SlidingWindow => Box::new(SlidingWindowLimiter::new(config)),
            RateLimitAlgorithm::TokenBucket => Box::new(TokenBucketLimiter::new(config)),
            RateLimitAlgorithm::FixedWindow => Box::new(FixedWindowLimiter::new(config)),
        };

        Ok(Self { strategy })
    }

    /// Try to acquire a permit without waiting
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError::RateLimitExceeded` if no permits are available.
    pub async fn try_acquire(&self) -> Result<(), RateLimitError> {
        match self.strategy.try_acquire().await {
            Ok(()) => Ok(()),
            Err(wait_time) => Err(RateLimitError::RateLimitExceeded { wait_time }),
        }
    }

    /// Acquire a permit, waiting if necessary
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError::InvalidConfiguration` if the rate limiter is misconfigured.
    pub async fn acquire(&self) -> Result<(), RateLimitError> {
        loop {
            match self.try_acquire().await {
                Ok(()) => return Ok(()),
                Err(RateLimitError::RateLimitExceeded { wait_time }) => {
                    debug!("Rate limited, waiting {:?}", wait_time);
                    sleep(wait_time).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    /// Get the number of available permits
    pub async fn available_permits(&self) -> u32 {
        self.strategy.available_permits().await
    }

    /// Check if a permit is available without consuming it
    pub async fn is_available(&self) -> bool {
        self.available_permits().await > 0
    }
}

/// Builder pattern for rate limiters
#[derive(Default)]
pub struct RateLimiterBuilder {
    config: RateLimiterConfig,
}

impl RateLimiterBuilder {
    /// Set requests per second
    #[must_use]
    pub fn per_second(mut self, requests: u32) -> Self {
        self.config = RateLimiterConfig::per_second(requests);
        self
    }

    /// Set requests per minute
    #[must_use]
    pub fn per_minute(mut self, requests: u32) -> Self {
        self.config = RateLimiterConfig::per_minute(requests);
        self
    }

    /// Set requests per hour
    #[must_use]
    pub fn per_hour(mut self, requests: u32) -> Self {
        self.config = RateLimiterConfig::per_hour(requests);
        self
    }

    /// Set custom rate limit
    #[must_use]
    pub fn custom(mut self, requests: u32, window: Duration) -> Self {
        self.config.max_requests = requests;
        self.config.window = window;
        self
    }

    /// Use sliding window algorithm
    #[must_use]
    pub fn sliding_window(mut self) -> Self {
        self.config.algorithm = RateLimitAlgorithm::SlidingWindow;
        self
    }

    /// Use token bucket algorithm
    #[must_use]
    pub fn token_bucket(mut self) -> Self {
        self.config.algorithm = RateLimitAlgorithm::TokenBucket;
        self
    }

    /// Use fixed window algorithm
    #[must_use]
    pub fn fixed_window(mut self) -> Self {
        self.config.algorithm = RateLimitAlgorithm::FixedWindow;
        self
    }

    /// Allow bursts (token bucket only)
    #[must_use]
    pub fn allow_burst(mut self) -> Self {
        self.config.allow_burst = true;
        self.config.algorithm = RateLimitAlgorithm::TokenBucket;
        self
    }

    /// Build the rate limiter
    ///
    /// # Errors
    ///
    /// Returns `RateLimitError::InvalidConfiguration` if the configuration is invalid.
    pub fn build(self) -> Result<RateLimiter, RateLimitError> {
        RateLimiter::new(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_sliding_window_rate_limiter() {
        let limiter = RateLimiterBuilder::default()
            .per_second(2)
            .sliding_window()
            .build()
            .unwrap();

        // First two should succeed
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());

        // Third should fail
        assert!(limiter.try_acquire().await.is_err());

        // Wait for window to slide
        sleep(Duration::from_secs(1)).await;

        // Should succeed again
        assert!(limiter.acquire().await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_token_bucket_rate_limiter() {
        let limiter = RateLimiterBuilder::default()
            .per_second(2)
            .token_bucket()
            .allow_burst()
            .build()
            .unwrap();

        // Should allow burst
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());

        // Should be empty
        assert_eq!(limiter.available_permits().await, 0);

        // Wait for refill
        sleep(Duration::from_millis(500)).await;

        // Should have ~1 token
        assert!(limiter.acquire().await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_fixed_window_rate_limiter() {
        let limiter = RateLimiterBuilder::default()
            .per_second(2)
            .fixed_window()
            .build()
            .unwrap();

        // Use up the window
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.try_acquire().await.is_err());

        // Wait for new window
        sleep(Duration::from_secs(1)).await;

        // New window should allow requests
        assert!(limiter.acquire().await.is_ok());
        assert!(limiter.acquire().await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_available_permits() {
        let limiter = RateLimiterBuilder::default()
            .per_minute(10)
            .build()
            .unwrap();

        assert_eq!(limiter.available_permits().await, 10);

        limiter.acquire().await.unwrap();
        assert_eq!(limiter.available_permits().await, 9);

        limiter.acquire().await.unwrap();
        assert_eq!(limiter.available_permits().await, 8);
    }
}
