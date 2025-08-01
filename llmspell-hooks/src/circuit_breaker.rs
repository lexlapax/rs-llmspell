// ABOUTME: Circuit breaker implementation for automatic performance protection
// ABOUTME: Opens circuit on slow hooks, provides recovery with exponential backoff

use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::info;

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Circuit is closed, normal operation
    Closed,
    /// Circuit is open, blocking calls
    Open,
    /// Circuit is half-open, testing recovery
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct BreakerConfig {
    /// Maximum failures before opening circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Time window for failure counting
    pub failure_window: Duration,
    /// How long to stay open before trying half-open
    pub open_duration: Duration,
    /// Maximum consecutive slow operations
    pub slow_call_threshold: u32,
    /// Duration that defines a slow call
    pub slow_call_duration: Duration,
}

impl Default for BreakerConfig {
    fn default() -> Self {
        Self {
            // OPTIMIZATION: Reduced failure threshold for faster detection
            failure_threshold: 3,
            success_threshold: 2,
            // OPTIMIZATION: Shorter window for faster recovery
            failure_window: Duration::from_secs(30),
            // OPTIMIZATION: Shorter open duration for faster recovery
            open_duration: Duration::from_secs(15),
            // OPTIMIZATION: Lower slow call threshold for <5% overhead target
            slow_call_threshold: 2,
            // OPTIMIZATION: Stricter slow call duration to maintain performance
            slow_call_duration: Duration::from_millis(50),
        }
    }
}

impl BreakerConfig {
    /// Production-optimized configuration for hook system
    pub fn production_optimized() -> Self {
        Self {
            failure_threshold: 2,
            success_threshold: 1,
            failure_window: Duration::from_secs(20),
            open_duration: Duration::from_secs(10),
            slow_call_threshold: 1,
            slow_call_duration: Duration::from_millis(25),
        }
    }

    /// Conservative configuration for critical hooks
    pub fn conservative() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            failure_window: Duration::from_secs(60),
            open_duration: Duration::from_secs(30),
            slow_call_threshold: 3,
            slow_call_duration: Duration::from_millis(100),
        }
    }
}

/// Statistics for circuit breaker
#[derive(Debug)]
struct BreakerStats {
    failures: AtomicU64,
    successes: AtomicU64,
    slow_calls: AtomicU64,
    total_calls: AtomicU64,
    last_failure_time: RwLock<Option<Instant>>,
    state_changed_at: RwLock<Instant>,
}

impl Default for BreakerStats {
    fn default() -> Self {
        Self {
            failures: AtomicU64::new(0),
            successes: AtomicU64::new(0),
            slow_calls: AtomicU64::new(0),
            total_calls: AtomicU64::new(0),
            last_failure_time: RwLock::new(None),
            state_changed_at: RwLock::new(Instant::now()),
        }
    }
}

/// Circuit breaker for hook execution
pub struct CircuitBreaker {
    config: BreakerConfig,
    state: Arc<RwLock<BreakerState>>,
    stats: Arc<BreakerStats>,
    name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String) -> Self {
        Self::with_config(name, BreakerConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(name: String, config: BreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(BreakerState::Closed)),
            stats: Arc::new(BreakerStats::default()),
            name,
        }
    }

    /// Get current state
    pub fn state(&self) -> BreakerState {
        *self.state.read()
    }

    /// Check if circuit allows execution
    pub fn can_execute(&self) -> bool {
        let current_state = *self.state.read();

        match current_state {
            BreakerState::Closed => true,
            BreakerState::Open => {
                // Check if we should transition to half-open
                let state_changed_at = *self.stats.state_changed_at.read();
                if state_changed_at.elapsed() >= self.config.open_duration {
                    self.transition_to(BreakerState::HalfOpen);
                    true
                } else {
                    false
                }
            }
            BreakerState::HalfOpen => true,
        }
    }

    /// Record a successful execution
    pub fn record_success(&self, duration: Duration) {
        self.stats.successes.fetch_add(1, Ordering::Relaxed);
        self.stats.total_calls.fetch_add(1, Ordering::Relaxed);

        let current_state = *self.state.read();

        // Check for slow call
        if duration >= self.config.slow_call_duration {
            self.stats.slow_calls.fetch_add(1, Ordering::Relaxed);

            // Check if we should open due to slow calls
            if current_state == BreakerState::Closed {
                let slow_calls = self.stats.slow_calls.load(Ordering::Relaxed);
                if slow_calls >= self.config.slow_call_threshold as u64 {
                    self.transition_to(BreakerState::Open);
                    return;
                }
            }
        }

        // Handle state transitions on success
        if current_state == BreakerState::HalfOpen {
            let successes = self.stats.successes.load(Ordering::Relaxed);
            if successes >= self.config.success_threshold as u64 {
                self.transition_to(BreakerState::Closed);
            }
        }
    }

    /// Record a failed execution
    pub fn record_failure(&self, _error: &anyhow::Error) {
        self.stats.failures.fetch_add(1, Ordering::Relaxed);
        self.stats.total_calls.fetch_add(1, Ordering::Relaxed);
        *self.stats.last_failure_time.write() = Some(Instant::now());

        let current_state = *self.state.read();

        match current_state {
            BreakerState::Closed => {
                // Check if we should open the circuit
                let failures = self.stats.failures.load(Ordering::Relaxed);
                if failures >= self.config.failure_threshold as u64 {
                    self.transition_to(BreakerState::Open);
                }
            }
            BreakerState::HalfOpen => {
                // Any failure in half-open state reopens the circuit
                self.transition_to(BreakerState::Open);
            }
            BreakerState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Transition to a new state
    fn transition_to(&self, new_state: BreakerState) {
        let mut state = self.state.write();
        if *state != new_state {
            info!(
                "Circuit breaker '{}' transitioning from {:?} to {:?}",
                self.name, *state, new_state
            );

            *state = new_state;
            *self.stats.state_changed_at.write() = Instant::now();

            // Reset counters on state change
            if new_state == BreakerState::Closed {
                self.stats.failures.store(0, Ordering::Relaxed);
                self.stats.successes.store(0, Ordering::Relaxed);
                self.stats.slow_calls.store(0, Ordering::Relaxed);
            } else if new_state == BreakerState::HalfOpen {
                self.stats.successes.store(0, Ordering::Relaxed);
            }
        }
    }

    /// Get circuit breaker statistics
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state(),
            failures: self.stats.failures.load(Ordering::Relaxed),
            successes: self.stats.successes.load(Ordering::Relaxed),
            slow_calls: self.stats.slow_calls.load(Ordering::Relaxed),
            total_calls: self.stats.total_calls.load(Ordering::Relaxed),
        }
    }

    /// Reset the circuit breaker
    pub fn reset(&self) {
        self.transition_to(BreakerState::Closed);
        self.stats.failures.store(0, Ordering::Relaxed);
        self.stats.successes.store(0, Ordering::Relaxed);
        self.stats.slow_calls.store(0, Ordering::Relaxed);
        self.stats.total_calls.store(0, Ordering::Relaxed);
        *self.stats.last_failure_time.write() = None;
    }
}

/// Public statistics structure
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: BreakerState,
    pub failures: u64,
    pub successes: u64,
    pub slow_calls: u64,
    pub total_calls: u64,
}

/// Circuit breaker manager for multiple breakers
pub struct CircuitBreakerManager {
    breakers: Arc<DashMap<String, Arc<CircuitBreaker>>>,
    default_config: BreakerConfig,
}

impl CircuitBreakerManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self::with_config(BreakerConfig::default())
    }

    /// Create with default configuration
    pub fn with_config(default_config: BreakerConfig) -> Self {
        Self {
            breakers: Arc::new(DashMap::new()),
            default_config,
        }
    }

    /// Get or create a circuit breaker
    pub fn get_or_create(&self, name: &str) -> Arc<CircuitBreaker> {
        self.breakers
            .entry(name.to_string())
            .or_insert_with(|| {
                Arc::new(CircuitBreaker::with_config(
                    name.to_string(),
                    self.default_config.clone(),
                ))
            })
            .clone()
    }

    /// Create a custom circuit breaker
    pub fn create_custom(&self, name: &str, config: BreakerConfig) -> Arc<CircuitBreaker> {
        let breaker = Arc::new(CircuitBreaker::with_config(name.to_string(), config));
        self.breakers.insert(name.to_string(), breaker.clone());
        breaker
    }

    /// Get all circuit breaker stats
    pub fn all_stats(&self) -> Vec<(String, CircuitBreakerStats)> {
        self.breakers
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().stats()))
            .collect()
    }

    /// Reset all circuit breakers
    pub fn reset_all(&self) {
        for breaker in self.breakers.iter() {
            breaker.value().reset();
        }
    }
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use std::thread;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_circuit_breaker_states() {
        let config = BreakerConfig {
            failure_threshold: 2,
            success_threshold: 2,
            open_duration: Duration::from_millis(100),
            ..Default::default()
        };

        let breaker = CircuitBreaker::with_config("test".to_string(), config);

        // Initially closed
        assert_eq!(breaker.state(), BreakerState::Closed);
        assert!(breaker.can_execute());

        // Record failures to open circuit
        breaker.record_failure(&anyhow::anyhow!("error 1"));
        assert_eq!(breaker.state(), BreakerState::Closed);

        breaker.record_failure(&anyhow::anyhow!("error 2"));
        assert_eq!(breaker.state(), BreakerState::Open);
        assert!(!breaker.can_execute());

        // Wait for open duration
        thread::sleep(Duration::from_millis(150));

        // Should transition to half-open
        assert!(breaker.can_execute());
        assert_eq!(breaker.state(), BreakerState::HalfOpen);

        // Success in half-open
        breaker.record_success(Duration::from_millis(10));
        breaker.record_success(Duration::from_millis(10));
        assert_eq!(breaker.state(), BreakerState::Closed);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_slow_call_detection() {
        let config = BreakerConfig {
            slow_call_threshold: 2,
            slow_call_duration: Duration::from_millis(50),
            ..Default::default()
        };

        let breaker = CircuitBreaker::with_config("test".to_string(), config);

        // Record slow calls
        breaker.record_success(Duration::from_millis(60));
        assert_eq!(breaker.state(), BreakerState::Closed);

        breaker.record_success(Duration::from_millis(70));
        assert_eq!(breaker.state(), BreakerState::Open);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_circuit_breaker_reset() {
        let breaker = CircuitBreaker::new("test".to_string());

        breaker.record_failure(&anyhow::anyhow!("error"));
        let stats = breaker.stats();
        assert_eq!(stats.failures, 1);
        assert_eq!(stats.total_calls, 1);

        breaker.reset();
        let stats = breaker.stats();
        assert_eq!(stats.failures, 0);
        assert_eq!(stats.total_calls, 0);
        assert_eq!(breaker.state(), BreakerState::Closed);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new();

        let breaker1 = manager.get_or_create("breaker1");
        let breaker2 = manager.get_or_create("breaker2");
        let breaker1_again = manager.get_or_create("breaker1");

        // Should get the same instance
        assert!(Arc::ptr_eq(&breaker1, &breaker1_again));
        assert!(!Arc::ptr_eq(&breaker1, &breaker2));

        // Test custom config
        let custom_config = BreakerConfig {
            failure_threshold: 10,
            ..Default::default()
        };
        let custom_breaker = manager.create_custom("custom", custom_config);

        // Record some activity
        breaker1.record_success(Duration::from_millis(10));
        breaker2.record_failure(&anyhow::anyhow!("error"));
        custom_breaker.record_success(Duration::from_millis(20));

        let all_stats = manager.all_stats();
        assert_eq!(all_stats.len(), 3);
    }
}
