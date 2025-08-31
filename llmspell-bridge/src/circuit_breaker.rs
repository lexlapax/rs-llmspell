//! Circuit breaker trait abstraction for hook introspection and fault tolerance
//!
//! Provides trait-based abstraction for circuit breaker functionality with
//! workload-aware adaptive thresholds and recovery patterns, integrated with
//! the hook monitoring system for fault tolerance.

use crate::hook_profiler::WorkloadClassifier;
use std::time::{Duration, Instant};

/// Circuit breaker state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Circuit is closed - operations flow normally
    Closed,
    /// Circuit is open - operations are blocked
    Open,
    /// Circuit is half-open - testing if service recovered
    HalfOpen,
}

/// Operation context for circuit breaker decisions
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// Operation name/identifier
    pub operation_name: String,
    /// Workload classification for adaptive thresholds
    pub workload: WorkloadClassifier,
    /// Duration of the operation
    pub duration: Duration,
    /// Whether operation succeeded or failed
    pub success: bool,
}

/// Configuration for circuit breaker with workload-aware thresholds
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Error rate threshold for micro operations (high tolerance)
    pub micro_operation_threshold: f64,
    /// Error rate threshold for light operations
    pub light_operation_threshold: f64,
    /// Error rate threshold for medium operations
    pub medium_operation_threshold: f64,
    /// Error rate threshold for heavy operations (strict)
    pub heavy_operation_threshold: f64,
    /// Enable adaptive backoff based on recovery patterns
    pub adaptive_backoff: bool,
    /// Minimum backoff duration in milliseconds
    pub min_backoff_ms: u64,
    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,
    /// Number of recent operations to consider for error rate
    pub window_size: usize,
    /// Number of test requests in half-open state
    pub half_open_test_requests: usize,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            micro_operation_threshold: 0.8,  // 80% error rate for micro ops
            light_operation_threshold: 0.6,  // 60% error rate for light ops
            medium_operation_threshold: 0.4, // 40% error rate for medium ops
            heavy_operation_threshold: 0.2,  // 20% error rate for heavy ops
            adaptive_backoff: true,
            min_backoff_ms: 100,        // 100ms minimum
            max_backoff_ms: 60_000,     // 60s maximum
            window_size: 100,           // Consider last 100 operations
            half_open_test_requests: 5, // Test with 5 requests
        }
    }
}

impl CircuitBreakerConfig {
    /// Create config optimized for micro operations (high fault tolerance)
    #[must_use]
    pub fn micro() -> Self {
        Self {
            micro_operation_threshold: 0.9,
            light_operation_threshold: 0.8,
            medium_operation_threshold: 0.7,
            heavy_operation_threshold: 0.5,
            min_backoff_ms: 50,
            max_backoff_ms: 10_000,
            ..Default::default()
        }
    }

    /// Create config optimized for heavy operations (strict fault tolerance)
    #[must_use]
    pub fn heavy() -> Self {
        Self {
            micro_operation_threshold: 0.5,
            light_operation_threshold: 0.3,
            medium_operation_threshold: 0.2,
            heavy_operation_threshold: 0.1,
            min_backoff_ms: 1000,
            max_backoff_ms: 300_000, // 5 minutes
            ..Default::default()
        }
    }

    /// Get error rate threshold for specific workload
    #[must_use]
    pub const fn threshold_for_workload(&self, workload: WorkloadClassifier) -> f64 {
        match workload {
            WorkloadClassifier::Micro => self.micro_operation_threshold,
            WorkloadClassifier::Light => self.light_operation_threshold,
            WorkloadClassifier::Medium => self.medium_operation_threshold,
            WorkloadClassifier::Heavy => self.heavy_operation_threshold,
        }
    }
}

/// Circuit breaker report containing fault tolerance metrics
#[derive(Debug, Clone)]
pub struct CircuitBreakerReport {
    /// Current circuit state
    pub state: CircuitState,
    /// Error rate in the current window
    pub error_rate: f64,
    /// Number of operations in current window
    pub operations_count: usize,
    /// Number of failed operations
    pub failures_count: usize,
    /// Current backoff duration (if open)
    pub backoff_duration: Option<Duration>,
    /// Time until next state transition attempt
    pub next_attempt: Option<Instant>,
    /// Number of state transitions
    pub state_transitions: u64,
}

/// Trait for circuit breaker implementation with adaptive thresholds
pub trait CircuitBreaker: Send + Sync {
    /// Check if operation should be allowed based on current state
    ///
    /// # Returns
    /// `true` if operation should proceed, `false` if circuit is open
    fn allow_operation(&self, context: &OperationContext) -> bool;

    /// Record operation result for circuit breaker decision making
    fn record_operation(&mut self, context: OperationContext);

    /// Force circuit to open state (manual trip)
    fn trip(&mut self);

    /// Force circuit to closed state (manual reset)
    fn reset(&mut self);

    /// Get current circuit state
    fn state(&self) -> CircuitState;

    /// Check if circuit is open (blocking operations)
    fn is_open(&self) -> bool {
        self.state() == CircuitState::Open
    }

    /// Get current configuration
    fn config(&self) -> &CircuitBreakerConfig;

    /// Generate current circuit breaker report
    fn report(&self) -> CircuitBreakerReport;

    /// Adapt backoff duration based on observed recovery patterns
    fn adapt_backoff(&mut self, recovery_time: Duration);
}

/// Real circuit breaker implementation with exponential backoff
pub struct ExponentialBackoffBreaker {
    /// Current configuration
    config: CircuitBreakerConfig,
    /// Current circuit state
    state: CircuitState,
    /// Recent operation results (true = success, false = failure)
    operation_history: Vec<bool>,
    /// Current backoff duration
    current_backoff: Duration,
    /// Time when circuit was last opened
    last_failure_time: Option<Instant>,
    /// Number of test requests made in half-open state
    half_open_test_count: usize,
    /// Total number of state transitions
    state_transitions: u64,
    /// Recovery time observations for adaptive backoff
    recovery_observations: Vec<Duration>,
}

impl ExponentialBackoffBreaker {
    /// Create a new exponential backoff circuit breaker
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            current_backoff: Duration::from_millis(config.min_backoff_ms),
            state: CircuitState::Closed,
            operation_history: Vec::with_capacity(config.window_size),
            last_failure_time: None,
            half_open_test_count: 0,
            state_transitions: 0,
            recovery_observations: Vec::new(),
            config,
        }
    }

    /// Calculate current error rate from operation history
    #[allow(clippy::cast_precision_loss)] // Acceptable for ratio calculation
    fn calculate_error_rate(&self) -> f64 {
        if self.operation_history.is_empty() {
            return 0.0;
        }

        let failures = self
            .operation_history
            .iter()
            .filter(|&&success| !success)
            .count();
        failures as f64 / self.operation_history.len() as f64
    }

    /// Check if backoff period has elapsed
    fn is_backoff_elapsed(&self) -> bool {
        self.last_failure_time
            .is_none_or(|last_failure| last_failure.elapsed() >= self.current_backoff)
    }

    /// Transition to new state
    fn transition_to(&mut self, new_state: CircuitState) {
        if self.state != new_state {
            self.state = new_state;
            self.state_transitions += 1;

            match new_state {
                CircuitState::Open => {
                    self.last_failure_time = Some(Instant::now());
                    self.half_open_test_count = 0;
                }
                CircuitState::HalfOpen => {
                    self.half_open_test_count = 0;
                }
                CircuitState::Closed => {
                    self.last_failure_time = None;
                    self.half_open_test_count = 0;
                    // Reset backoff on successful recovery
                    self.current_backoff = Duration::from_millis(self.config.min_backoff_ms);
                }
            }
        }
    }

    /// Update backoff duration using exponential strategy
    fn update_backoff(&mut self) {
        let new_backoff = self.current_backoff.mul_f64(2.0);
        let max_backoff = Duration::from_millis(self.config.max_backoff_ms);
        self.current_backoff = new_backoff.min(max_backoff);
    }
}

impl Default for ExponentialBackoffBreaker {
    fn default() -> Self {
        Self::new(CircuitBreakerConfig::default())
    }
}

impl CircuitBreaker for ExponentialBackoffBreaker {
    fn allow_operation(&self, _context: &OperationContext) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if we should transition to half-open
                self.is_backoff_elapsed()
            }
            CircuitState::HalfOpen => {
                // Allow limited test operations
                self.half_open_test_count < self.config.half_open_test_requests
            }
        }
    }

    fn record_operation(&mut self, context: OperationContext) {
        // Add to operation history
        self.operation_history.push(context.success);

        // Maintain window size
        if self.operation_history.len() > self.config.window_size {
            self.operation_history.remove(0);
        }

        match self.state {
            CircuitState::Closed => {
                if !context.success {
                    let error_rate = self.calculate_error_rate();
                    let threshold = self.config.threshold_for_workload(context.workload);

                    if error_rate > threshold {
                        self.transition_to(CircuitState::Open);
                        self.update_backoff();
                    }
                }
            }
            CircuitState::Open => {
                if self.is_backoff_elapsed() {
                    self.transition_to(CircuitState::HalfOpen);
                }
            }
            CircuitState::HalfOpen => {
                self.half_open_test_count += 1;

                if context.success {
                    // Success in half-open - check if we can close
                    if self.half_open_test_count >= self.config.half_open_test_requests {
                        let recent_successes = self
                            .operation_history
                            .iter()
                            .rev()
                            .take(self.config.half_open_test_requests)
                            .all(|&success| success);

                        if recent_successes {
                            if let Some(last_failure) = self.last_failure_time {
                                let recovery_time = last_failure.elapsed();
                                self.recovery_observations.push(recovery_time);

                                // Keep only recent observations
                                if self.recovery_observations.len() > 10 {
                                    self.recovery_observations.remove(0);
                                }
                            }
                            self.transition_to(CircuitState::Closed);
                        }
                    }
                } else {
                    // Failure in half-open - back to open
                    self.transition_to(CircuitState::Open);
                    self.update_backoff();
                }
            }
        }
    }

    fn trip(&mut self) {
        self.transition_to(CircuitState::Open);
        self.update_backoff();
    }

    fn reset(&mut self) {
        self.transition_to(CircuitState::Closed);
    }

    fn state(&self) -> CircuitState {
        self.state
    }

    fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    fn report(&self) -> CircuitBreakerReport {
        let error_rate = self.calculate_error_rate();
        let failures_count = self
            .operation_history
            .iter()
            .filter(|&&success| !success)
            .count();

        let next_attempt = if self.state == CircuitState::Open {
            self.last_failure_time.map(|t| t + self.current_backoff)
        } else {
            None
        };

        let backoff_duration = if self.state == CircuitState::Open {
            Some(self.current_backoff)
        } else {
            None
        };

        CircuitBreakerReport {
            state: self.state,
            error_rate,
            operations_count: self.operation_history.len(),
            failures_count,
            backoff_duration,
            next_attempt,
            state_transitions: self.state_transitions,
        }
    }

    fn adapt_backoff(&mut self, recovery_time: Duration) {
        if !self.config.adaptive_backoff || self.recovery_observations.is_empty() {
            return;
        }

        // Calculate average recovery time
        let total: Duration = self.recovery_observations.iter().sum();
        #[allow(clippy::cast_possible_truncation)] // Vector length is bounded by implementation
        let avg_recovery = total / (self.recovery_observations.len() as u32).max(1);

        // Adjust backoff based on observed recovery patterns
        let new_backoff = if recovery_time > avg_recovery {
            // Slower recovery - increase backoff
            self.current_backoff.mul_f64(1.5)
        } else {
            // Faster recovery - slightly decrease backoff
            self.current_backoff.mul_f64(0.8)
        };

        // Keep within configured bounds
        let min_backoff = Duration::from_millis(self.config.min_backoff_ms);
        let max_backoff = Duration::from_millis(self.config.max_backoff_ms);
        self.current_backoff = new_backoff.clamp(min_backoff, max_backoff);
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Test constants are safe for exact comparison
mod tests {
    use super::*;

    #[test]
    fn test_circuit_breaker_config_defaults() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.micro_operation_threshold, 0.8);
        assert_eq!(config.heavy_operation_threshold, 0.2);
        assert!(config.adaptive_backoff);
    }

    #[test]
    fn test_circuit_breaker_config_presets() {
        let micro_config = CircuitBreakerConfig::micro();
        assert_eq!(micro_config.micro_operation_threshold, 0.9);
        assert_eq!(micro_config.min_backoff_ms, 50);

        let heavy_config = CircuitBreakerConfig::heavy();
        assert_eq!(heavy_config.heavy_operation_threshold, 0.1);
        assert_eq!(heavy_config.min_backoff_ms, 1000);
    }

    #[test]
    fn test_threshold_for_workload() {
        let config = CircuitBreakerConfig::default();

        assert_eq!(
            config.threshold_for_workload(WorkloadClassifier::Micro),
            0.8
        );
        assert_eq!(
            config.threshold_for_workload(WorkloadClassifier::Heavy),
            0.2
        );
    }

    #[test]
    fn test_exponential_backoff_breaker_lifecycle() {
        let config = CircuitBreakerConfig::default();
        let mut breaker = ExponentialBackoffBreaker::new(config);

        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(!breaker.is_open());

        let context = OperationContext {
            operation_name: "test_op".to_string(),
            workload: WorkloadClassifier::Light,
            duration: Duration::from_millis(10),
            success: true,
        };

        assert!(breaker.allow_operation(&context));

        // Record successful operation
        breaker.record_operation(context.clone());
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Record multiple failures to trip circuit
        let failure_context = OperationContext {
            success: false,
            ..context
        };

        // Need enough failures to exceed threshold (60% for light workload)
        for _ in 0..70 {
            breaker.record_operation(failure_context.clone());
        }

        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_report() {
        let mut breaker = ExponentialBackoffBreaker::default();

        let success_context = OperationContext {
            operation_name: "test".to_string(),
            workload: WorkloadClassifier::Medium,
            duration: Duration::from_millis(5),
            success: true,
        };

        let failure_context = OperationContext {
            success: false,
            ..success_context.clone()
        };

        // Record some operations
        breaker.record_operation(success_context);
        breaker.record_operation(failure_context);

        let report = breaker.report();
        assert_eq!(report.operations_count, 2);
        assert_eq!(report.failures_count, 1);
        assert_eq!(report.error_rate, 0.5);
        // With 50% error rate and Medium workload threshold of 40%, circuit should open
        assert_eq!(report.state, CircuitState::Open);
    }

    #[test]
    fn test_manual_trip_and_reset() {
        let mut breaker = ExponentialBackoffBreaker::default();

        assert_eq!(breaker.state(), CircuitState::Closed);

        breaker.trip();
        assert_eq!(breaker.state(), CircuitState::Open);

        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }
}
