// ABOUTME: Rate limiter module providing token bucket algorithm for API quota management
// ABOUTME: Implements thread-safe rate limiting with configurable burst capacity and refill rates

pub mod token_bucket;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use token_bucket::TokenBucket;
use tracing::debug;

/// Token bucket configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBucketConfig {
    /// Maximum number of tokens in the bucket
    pub capacity: usize,
    /// Rate at which tokens are refilled
    pub refill_rate: f64,
    /// Interval at which tokens are refilled
    pub refill_interval: Duration,
    /// Additional burst capacity beyond normal capacity
    pub burst_capacity: usize,
}

impl Default for TokenBucketConfig {
    fn default() -> Self {
        Self {
            capacity: 100,
            refill_rate: 10.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 50,
        }
    }
}

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Default token bucket configuration
    pub default_bucket_config: TokenBucketConfig,
    /// Whether to allow creating new buckets on demand
    pub allow_dynamic_buckets: bool,
    /// Maximum number of buckets to maintain
    pub max_buckets: usize,
    /// TTL for unused buckets
    pub bucket_ttl: Duration,
    /// Whether to track per-bucket statistics
    pub track_statistics: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            default_bucket_config: TokenBucketConfig::default(),
            allow_dynamic_buckets: true,
            max_buckets: 10000,
            bucket_ttl: Duration::from_secs(3600), // 1 hour
            track_statistics: true,
        }
    }
}

/// Bucket state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketState {
    /// Current number of tokens
    pub tokens: f64,
    /// Maximum capacity
    pub capacity: usize,
    /// Burst capacity
    pub burst_capacity: usize,
    /// Last access time
    pub last_access: DateTime<Utc>,
    /// Next refill time
    pub next_refill: DateTime<Utc>,
    /// Total tokens consumed
    pub total_consumed: u64,
    /// Total tokens refilled
    pub total_refilled: u64,
    /// Last refill amount
    pub last_refill_amount: f64,
}

/// Thread-safe rate limiter with multiple buckets
#[derive(Debug)]
pub struct RateLimiter {
    /// Token buckets keyed by identifier
    buckets: Arc<RwLock<HashMap<String, Arc<RwLock<TokenBucket>>>>>,
    /// Rate limiter configuration
    config: RateLimiterConfig,
    /// Bucket access times for TTL cleanup
    access_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
}

impl RateLimiter {
    /// Create a new rate limiter with default configuration
    pub fn new(bucket_config: TokenBucketConfig) -> Self {
        let config = RateLimiterConfig {
            default_bucket_config: bucket_config,
            ..Default::default()
        };
        Self::with_config(config)
    }

    /// Create a new rate limiter with custom configuration
    pub fn with_config(config: RateLimiterConfig) -> Self {
        Self {
            buckets: Arc::new(RwLock::new(HashMap::new())),
            config,
            access_times: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Try to acquire tokens from a bucket
    pub fn try_acquire(&self, key: &str, tokens: f64) -> (bool, f64) {
        // Get or create bucket
        let bucket = self.get_or_create_bucket(key);

        // Update access time
        {
            let mut access_times = self.access_times.write().unwrap();
            access_times.insert(key.to_string(), Utc::now());
        }

        // Try to acquire tokens
        let mut bucket_guard = bucket.write().unwrap();
        let (allowed, remaining) = bucket_guard.try_acquire(tokens);

        // Cleanup old buckets if needed
        if self.buckets.read().unwrap().len() > self.config.max_buckets {
            self.cleanup_old_buckets();
        }

        (allowed, remaining)
    }

    /// Force acquire tokens (may go negative)
    pub fn force_acquire(&self, key: &str, tokens: f64) -> f64 {
        let bucket = self.get_or_create_bucket(key);
        let mut bucket_guard = bucket.write().unwrap();
        bucket_guard.force_acquire(tokens)
    }

    /// Get the current state of a bucket
    pub fn get_bucket_state(&self, key: &str) -> BucketState {
        let bucket = self.get_or_create_bucket(key);
        let bucket_guard = bucket.read().unwrap();
        bucket_guard.get_state()
    }

    /// Reset a specific bucket
    pub fn reset_bucket(&self, key: &str) {
        let buckets = self.buckets.read().unwrap();
        if let Some(bucket) = buckets.get(key) {
            let mut bucket_guard = bucket.write().unwrap();
            bucket_guard.reset();
        }
    }

    /// Reset all buckets
    pub fn reset_all(&self) {
        let buckets = self.buckets.read().unwrap();
        for bucket in buckets.values() {
            let mut bucket_guard = bucket.write().unwrap();
            bucket_guard.reset();
        }
    }

    /// Get statistics for all buckets
    pub fn get_statistics(&self) -> HashMap<String, BucketState> {
        let buckets = self.buckets.read().unwrap();
        let mut stats = HashMap::new();

        for (key, bucket) in buckets.iter() {
            let bucket_guard = bucket.read().unwrap();
            stats.insert(key.clone(), bucket_guard.get_state());
        }

        stats
    }

    /// Get or create a bucket for the given key
    fn get_or_create_bucket(&self, key: &str) -> Arc<RwLock<TokenBucket>> {
        // Try to get existing bucket
        {
            let buckets = self.buckets.read().unwrap();
            if let Some(bucket) = buckets.get(key) {
                return bucket.clone();
            }
        }

        // Create new bucket if allowed
        if self.config.allow_dynamic_buckets {
            let mut buckets = self.buckets.write().unwrap();
            // Double-check after acquiring write lock
            if let Some(bucket) = buckets.get(key) {
                return bucket.clone();
            }

            // Create new bucket
            let bucket = Arc::new(RwLock::new(TokenBucket::new(
                self.config.default_bucket_config.clone(),
            )));
            buckets.insert(key.to_string(), bucket.clone());
            bucket
        } else {
            // Return a default bucket if dynamic creation is disabled
            Arc::new(RwLock::new(TokenBucket::new(
                self.config.default_bucket_config.clone(),
            )))
        }
    }

    /// Clean up old buckets that haven't been accessed recently
    fn cleanup_old_buckets(&self) {
        let now = Utc::now();
        let mut keys_to_remove = Vec::new();

        // Find buckets to remove
        {
            let access_times = self.access_times.read().unwrap();
            for (key, last_access) in access_times.iter() {
                let elapsed = now
                    .signed_duration_since(*last_access)
                    .to_std()
                    .unwrap_or(Duration::ZERO);
                if elapsed > self.config.bucket_ttl {
                    keys_to_remove.push(key.clone());
                }
            }
        }

        // Remove old buckets
        if !keys_to_remove.is_empty() {
            let mut buckets = self.buckets.write().unwrap();
            let mut access_times = self.access_times.write().unwrap();

            for key in keys_to_remove {
                buckets.remove(&key);
                access_times.remove(&key);
                debug!("Cleaned up old rate limiter bucket: {}", key);
            }
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(TokenBucketConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;
    #[test]
    fn test_rate_limiter_basic() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 5.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 5,
        };

        let limiter = RateLimiter::new(config);

        // Should allow initial requests up to capacity
        let (allowed, remaining) = limiter.try_acquire("test", 5.0);
        assert!(allowed);
        assert_eq!(remaining, 10.0); // 15 total (10 + 5 burst) - 5 used

        // Should allow more requests
        let (allowed, remaining) = limiter.try_acquire("test", 5.0);
        assert!(allowed);
        assert_eq!(remaining, 5.0);

        // Should allow using burst capacity
        let (allowed, remaining) = limiter.try_acquire("test", 5.0);
        assert!(allowed);
        assert_eq!(remaining, 0.0);

        // Should deny when exhausted
        let (allowed, _) = limiter.try_acquire("test", 1.0);
        assert!(!allowed);
    }
    #[test]
    fn test_rate_limiter_refill() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 10.0,
            refill_interval: Duration::from_millis(100),
            burst_capacity: 0,
        };

        let limiter = RateLimiter::new(config);

        // Use all tokens
        let (allowed, _) = limiter.try_acquire("test", 10.0);
        assert!(allowed);

        // Should be denied immediately
        let (allowed, _) = limiter.try_acquire("test", 1.0);
        assert!(!allowed);

        // Wait for refill
        thread::sleep(StdDuration::from_millis(150));

        // Should be allowed after refill
        let (allowed, remaining) = limiter.try_acquire("test", 1.0);
        assert!(allowed);
        assert!(remaining >= 9.0);
    }
    #[test]
    fn test_rate_limiter_multiple_buckets() {
        let limiter = RateLimiter::new(TokenBucketConfig {
            capacity: 5,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 0,
        });

        // Different keys should have separate buckets
        let (allowed1, _) = limiter.try_acquire("bucket1", 5.0);
        let (allowed2, _) = limiter.try_acquire("bucket2", 5.0);

        assert!(allowed1);
        assert!(allowed2);

        // Both should be exhausted
        let (allowed1, _) = limiter.try_acquire("bucket1", 1.0);
        let (allowed2, _) = limiter.try_acquire("bucket2", 1.0);

        assert!(!allowed1);
        assert!(!allowed2);
    }
    #[test]
    fn test_bucket_state() {
        let limiter = RateLimiter::new(TokenBucketConfig {
            capacity: 100,
            refill_rate: 10.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 50,
        });

        // Acquire some tokens
        limiter.try_acquire("test", 25.0);

        let state = limiter.get_bucket_state("test");
        assert_eq!(state.capacity, 100);
        assert_eq!(state.burst_capacity, 50);
        assert_eq!(state.tokens, 125.0); // 150 - 25
        assert_eq!(state.total_consumed, 25);
    }
    #[test]
    fn test_force_acquire() {
        let limiter = RateLimiter::new(TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 0,
        });

        // Use all tokens
        limiter.try_acquire("test", 10.0);

        // Force acquire should work even when exhausted
        let remaining = limiter.force_acquire("test", 5.0);
        assert_eq!(remaining, -5.0);

        // Bucket should be in debt
        let state = limiter.get_bucket_state("test");
        assert_eq!(state.tokens, -5.0);
    }
    #[test]
    fn test_reset_bucket() {
        let limiter = RateLimiter::new(TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 5,
        });

        // Use some tokens
        limiter.try_acquire("test", 8.0);

        // Reset the bucket
        limiter.reset_bucket("test");

        // Should have full capacity again
        let state = limiter.get_bucket_state("test");
        assert_eq!(state.tokens, 15.0); // 10 + 5 burst
    }
    #[test]
    fn test_statistics() {
        let limiter = RateLimiter::new(TokenBucketConfig::default());

        // Create multiple buckets
        limiter.try_acquire("bucket1", 5.0);
        limiter.try_acquire("bucket2", 10.0);
        limiter.try_acquire("bucket3", 15.0);

        let stats = limiter.get_statistics();
        assert_eq!(stats.len(), 3);
        assert!(stats.contains_key("bucket1"));
        assert!(stats.contains_key("bucket2"));
        assert!(stats.contains_key("bucket3"));
    }
}
