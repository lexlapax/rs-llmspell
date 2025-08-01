// ABOUTME: Token bucket algorithm implementation for rate limiting with burst support
// ABOUTME: Provides precise rate limiting with configurable refill rates and burst capacity

use super::{BucketState, TokenBucketConfig};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Token bucket implementation
#[derive(Debug)]
pub struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Configuration
    config: TokenBucketConfig,
    /// Last refill time
    last_refill: DateTime<Utc>,
    /// Statistics
    total_consumed: u64,
    total_refilled: u64,
    last_refill_amount: f64,
}

impl TokenBucket {
    /// Create a new token bucket
    pub fn new(config: TokenBucketConfig) -> Self {
        let total_capacity = config.capacity + config.burst_capacity;
        Self {
            tokens: total_capacity as f64,
            config,
            last_refill: Utc::now(),
            total_consumed: 0,
            total_refilled: 0,
            last_refill_amount: 0.0,
        }
    }

    /// Try to acquire tokens
    pub fn try_acquire(&mut self, requested: f64) -> (bool, f64) {
        // Refill tokens based on elapsed time
        self.refill();

        if self.tokens >= requested {
            self.tokens -= requested;
            self.total_consumed += requested as u64;
            (true, self.tokens)
        } else {
            (false, self.tokens)
        }
    }

    /// Force acquire tokens (may go negative)
    pub fn force_acquire(&mut self, requested: f64) -> f64 {
        self.refill();
        self.tokens -= requested;
        self.total_consumed += requested as u64;
        self.tokens
    }

    /// Get current token count
    pub fn get_tokens(&mut self) -> f64 {
        self.refill();
        self.tokens
    }

    /// Get bucket state
    pub fn get_state(&self) -> BucketState {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_refill);
        let elapsed_seconds = elapsed.num_milliseconds() as f64 / 1000.0;

        // Calculate when next refill will happen
        let refill_interval_seconds = self.config.refill_interval.as_secs_f64();
        let time_until_next_refill =
            refill_interval_seconds - (elapsed_seconds % refill_interval_seconds);
        let next_refill =
            now + chrono::Duration::milliseconds((time_until_next_refill * 1000.0) as i64);

        BucketState {
            tokens: self.tokens,
            capacity: self.config.capacity,
            burst_capacity: self.config.burst_capacity,
            last_access: now,
            next_refill,
            total_consumed: self.total_consumed,
            total_refilled: self.total_refilled,
            last_refill_amount: self.last_refill_amount,
        }
    }

    /// Reset the bucket to full capacity
    pub fn reset(&mut self) {
        let total_capacity = self.config.capacity + self.config.burst_capacity;
        self.tokens = total_capacity as f64;
        self.last_refill = Utc::now();
        self.total_consumed = 0;
        self.total_refilled = 0;
        self.last_refill_amount = 0.0;
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Utc::now();
        let elapsed = now.signed_duration_since(self.last_refill);
        let elapsed_duration = elapsed.to_std().unwrap_or(Duration::ZERO);

        if elapsed_duration >= self.config.refill_interval {
            // Calculate how many refill intervals have passed
            let intervals =
                elapsed_duration.as_secs_f64() / self.config.refill_interval.as_secs_f64();
            let refill_amount = intervals * self.config.refill_rate;

            // Add tokens, capping at total capacity
            let total_capacity = (self.config.capacity + self.config.burst_capacity) as f64;
            let old_tokens = self.tokens;
            self.tokens = (self.tokens + refill_amount).min(total_capacity);

            // Update statistics
            let actual_refilled = self.tokens - old_tokens;
            if actual_refilled > 0.0 {
                self.total_refilled += actual_refilled as u64;
                self.last_refill_amount = actual_refilled;
            }

            // Update last refill time
            self.last_refill = now;
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_token_bucket_creation() {
        let config = TokenBucketConfig {
            capacity: 100,
            refill_rate: 10.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 50,
        };

        let bucket = TokenBucket::new(config);
        assert_eq!(bucket.tokens, 150.0); // capacity + burst
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_token_acquisition() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 5,
        };

        let mut bucket = TokenBucket::new(config);

        // Should allow acquiring tokens
        let (allowed, remaining) = bucket.try_acquire(5.0);
        assert!(allowed);
        assert_eq!(remaining, 10.0); // 15 - 5

        // Should allow acquiring more
        let (allowed, remaining) = bucket.try_acquire(10.0);
        assert!(allowed);
        assert_eq!(remaining, 0.0);

        // Should deny when exhausted
        let (allowed, remaining) = bucket.try_acquire(1.0);
        assert!(!allowed);
        assert_eq!(remaining, 0.0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_token_refill() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 10.0,
            refill_interval: Duration::from_millis(100),
            burst_capacity: 0,
        };

        let mut bucket = TokenBucket::new(config);

        // Use all tokens
        bucket.try_acquire(10.0);
        assert_eq!(bucket.tokens, 0.0);

        // Wait for refill
        thread::sleep(StdDuration::from_millis(150));

        // Should have refilled
        let tokens = bucket.get_tokens();
        assert!(tokens >= 10.0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_burst_capacity() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 20,
        };

        let mut bucket = TokenBucket::new(config);

        // Should be able to use burst capacity
        let (allowed, remaining) = bucket.try_acquire(25.0);
        assert!(allowed);
        assert_eq!(remaining, 5.0); // 30 - 25
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_force_acquire() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 0,
        };

        let mut bucket = TokenBucket::new(config);

        // Use all tokens
        bucket.try_acquire(10.0);

        // Force acquire should work
        let remaining = bucket.force_acquire(5.0);
        assert_eq!(remaining, -5.0);

        // Bucket should be in debt
        assert_eq!(bucket.tokens, -5.0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_bucket_state() {
        let config = TokenBucketConfig {
            capacity: 100,
            refill_rate: 10.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 50,
        };

        let mut bucket = TokenBucket::new(config);
        bucket.try_acquire(25.0);

        let state = bucket.get_state();
        assert_eq!(state.capacity, 100);
        assert_eq!(state.burst_capacity, 50);
        assert_eq!(state.tokens, 125.0);
        assert_eq!(state.total_consumed, 25);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_reset() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 1.0,
            refill_interval: Duration::from_secs(1),
            burst_capacity: 5,
        };

        let mut bucket = TokenBucket::new(config);

        // Use some tokens
        bucket.try_acquire(12.0);
        assert_eq!(bucket.tokens, 3.0);

        // Reset
        bucket.reset();
        assert_eq!(bucket.tokens, 15.0);
        assert_eq!(bucket.total_consumed, 0);
        assert_eq!(bucket.total_refilled, 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_refill_cap() {
        let config = TokenBucketConfig {
            capacity: 10,
            refill_rate: 100.0, // Very high refill rate
            refill_interval: Duration::from_millis(100),
            burst_capacity: 5,
        };

        let mut bucket = TokenBucket::new(config);

        // Use some tokens
        bucket.try_acquire(5.0);

        // Wait for multiple refill intervals
        thread::sleep(StdDuration::from_millis(500));

        // Should be capped at total capacity
        let tokens = bucket.get_tokens();
        assert_eq!(tokens, 15.0); // Should not exceed capacity + burst
    }
}
