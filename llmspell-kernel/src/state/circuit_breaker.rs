//! Circuit breaker pattern for resource protection

use anyhow::{anyhow, Result};
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed, normal operation
    Closed,
    /// Circuit is open, failing fast
    Open,
    /// Circuit is half-open, testing recovery
    HalfOpen,
}

/// Circuit breaker for protecting resources
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: Arc<RwLock<CircuitState>>,
    /// Failure count
    failure_count: AtomicU32,
    /// Success count (for half-open state)
    success_count: AtomicU32,
    /// Total call count
    total_calls: AtomicU64,
    /// Failure threshold before opening
    failure_threshold: u32,
    /// Success threshold for closing from half-open
    success_threshold: u32,
    /// Last failure time
    last_failure_time: Arc<RwLock<Option<Instant>>>,
    /// Reset timeout for open state
    reset_timeout: Duration,
    /// Half-open test delay
    _half_open_test_delay: Duration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    ///
    /// # Parameters
    /// - `failure_threshold`: Number of failures before opening
    /// - `reset_timeout`: How long to wait before trying again
    /// - `half_open_test_delay`: Delay before testing in half-open state
    pub fn new(
        failure_threshold: u32,
        reset_timeout: Duration,
        half_open_test_delay: Duration,
    ) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            failure_count: AtomicU32::new(0),
            success_count: AtomicU32::new(0),
            total_calls: AtomicU64::new(0),
            failure_threshold,
            success_threshold: failure_threshold / 2, // Half of threshold for recovery
            last_failure_time: Arc::new(RwLock::new(None)),
            reset_timeout,
            _half_open_test_delay: half_open_test_delay,
        }
    }

    /// Execute a function with circuit breaker protection
    ///
    /// # Errors
    ///
    /// Returns an error if the circuit is open or the function fails
    pub fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        self.total_calls.fetch_add(1, Ordering::Relaxed);

        // Check current state
        let current_state = self.check_state();

        match current_state {
            CircuitState::Open => Err(anyhow!("Circuit breaker is open")),
            CircuitState::Closed | CircuitState::HalfOpen => match f() {
                Ok(result) => {
                    self.on_success();
                    Ok(result)
                }
                Err(e) => {
                    self.on_failure();
                    Err(e)
                }
            },
        }
    }

    /// Check and update circuit state
    fn check_state(&self) -> CircuitState {
        let mut state = self.state.write();

        match *state {
            CircuitState::Open => {
                // Check if we should transition to half-open
                if let Some(last_failure) = *self.last_failure_time.read() {
                    if last_failure.elapsed() >= self.reset_timeout {
                        *state = CircuitState::HalfOpen;
                        self.failure_count.store(0, Ordering::Relaxed);
                        self.success_count.store(0, Ordering::Relaxed);
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Stay in half-open until we get enough data
                // State transitions happen in on_success/on_failure
            }
            CircuitState::Closed => {
                // Check if we should open
                if self.failure_count.load(Ordering::Relaxed) >= self.failure_threshold {
                    *state = CircuitState::Open;
                    *self.last_failure_time.write() = Some(Instant::now());
                }
            }
        }

        *state
    }

    /// Handle successful call
    fn on_success(&self) {
        let state = *self.state.read();

        match state {
            CircuitState::HalfOpen => {
                let success_count = self.success_count.fetch_add(1, Ordering::Relaxed) + 1;

                // Close circuit if we have enough successes
                if success_count >= self.success_threshold {
                    let mut state_write = self.state.write();
                    *state_write = CircuitState::Closed;
                    self.failure_count.store(0, Ordering::Relaxed);
                    self.success_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success in closed state
                if self.failure_count.load(Ordering::Relaxed) > 0 {
                    self.failure_count.store(0, Ordering::Relaxed);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but no action needed
            }
        }
    }

    /// Handle failed call
    fn on_failure(&self) {
        *self.last_failure_time.write() = Some(Instant::now());

        let state = *self.state.read();

        match state {
            CircuitState::HalfOpen => {
                // Immediately reopen on failure in half-open state
                let mut state_write = self.state.write();
                *state_write = CircuitState::Open;
                self.failure_count.store(0, Ordering::Relaxed);
                self.success_count.store(0, Ordering::Relaxed);
            }
            CircuitState::Closed => {
                let failure_count = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

                // Transition to open if threshold reached
                if failure_count >= self.failure_threshold {
                    let mut state_write = self.state.write();
                    *state_write = CircuitState::Open;
                }
            }
            CircuitState::Open => {
                // Already open, no action needed
            }
        }
    }

    /// Check if circuit is open
    pub fn is_open(&self) -> bool {
        self.check_state() == CircuitState::Open
    }

    /// Check if circuit is closed
    pub fn is_closed(&self) -> bool {
        self.check_state() == CircuitState::Closed
    }

    /// Check if circuit is half-open
    pub fn is_half_open(&self) -> bool {
        self.check_state() == CircuitState::HalfOpen
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.check_state()
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        *self.state.write() = CircuitState::Closed;
        self.failure_count.store(0, Ordering::Relaxed);
        self.success_count.store(0, Ordering::Relaxed);
        *self.last_failure_time.write() = None;
    }

    /// Get statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.check_state(),
            failure_count: self.failure_count.load(Ordering::Relaxed),
            success_count: self.success_count.load(Ordering::Relaxed),
            total_calls: self.total_calls.load(Ordering::Relaxed),
            last_failure_time: *self.last_failure_time.read(),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state
    pub state: CircuitState,
    /// Current failure count
    pub failure_count: u32,
    /// Current success count (half-open state)
    pub success_count: u32,
    /// Total calls made
    pub total_calls: u64,
    /// Last failure time
    pub last_failure_time: Option<Instant>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_opens_on_threshold() {
        let cb = CircuitBreaker::new(3, Duration::from_secs(1), Duration::from_millis(100));

        assert!(cb.is_closed());

        // Fail 3 times
        for _ in 0..3 {
            let _: Result<(), _> = cb.call(|| Err(anyhow!("test failure")));
        }

        assert!(cb.is_open());

        // Should fail fast when open
        let result = cb.call(|| Ok(42));
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_breaker_transitions_to_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100), Duration::from_millis(10));

        // Open the circuit
        for _ in 0..2 {
            let _: Result<(), _> = cb.call(|| Err(anyhow!("test failure")));
        }

        assert!(cb.is_open());

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should be half-open now
        assert!(cb.is_half_open());
    }

    #[test]
    fn test_circuit_breaker_closes_from_half_open() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100), Duration::from_millis(10));

        // Open the circuit
        for _ in 0..2 {
            let _: Result<(), _> = cb.call(|| Err(anyhow!("test failure")));
        }

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(150));

        // Succeed enough times to close
        for _ in 0..2 {
            let _ = cb.call(|| Ok(42));
        }

        assert!(cb.is_closed());
    }

    #[test]
    fn test_circuit_breaker_reopens_from_half_open_on_failure() {
        let cb = CircuitBreaker::new(2, Duration::from_millis(100), Duration::from_millis(10));

        // Open the circuit
        for _ in 0..2 {
            let _: Result<(), _> = cb.call(|| Err(anyhow!("test failure")));
        }

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(150));

        assert!(cb.is_half_open());

        // Fail once in half-open state
        let _: Result<(), _> = cb.call(|| Err(anyhow!("test failure")));

        assert!(cb.is_open());
    }
}
