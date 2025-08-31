//! Null circuit breaker implementation for testing
//!
//! Provides a no-op circuit breaker that implements the `CircuitBreaker` trait
//! without any actual fault tolerance functionality. Safe for use in tests
//! as it never blocks operations or causes side effects.

use crate::circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerReport, CircuitState, OperationContext,
};
use std::time::Duration;

/// Null circuit breaker that does nothing (for testing)
pub struct NullCircuitBreaker {
    /// Configuration (stored but not used)
    config: CircuitBreakerConfig,
    /// Operation counter for reports
    operation_count: usize,
}

impl NullCircuitBreaker {
    /// Create a new null circuit breaker
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: CircuitBreakerConfig::default(),
            operation_count: 0,
        }
    }

    /// Create with custom config (for testing different configurations)
    #[must_use]
    pub const fn with_config(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            operation_count: 0,
        }
    }
}

impl Default for NullCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreaker for NullCircuitBreaker {
    fn allow_operation(&self, _context: &OperationContext) -> bool {
        // Never block operations - always allow
        true
    }

    fn record_operation(&mut self, _context: OperationContext) {
        // Count operations for reporting but don't change behavior
        self.operation_count += 1;
    }

    fn trip(&mut self) {
        // No-op - don't actually change state to avoid blocking tests
    }

    fn reset(&mut self) {
        // No-op - state is always closed anyway
    }

    fn state(&self) -> CircuitState {
        // Always report as closed (never blocks)
        CircuitState::Closed
    }

    fn config(&self) -> &CircuitBreakerConfig {
        &self.config
    }

    fn report(&self) -> CircuitBreakerReport {
        // Return minimal valid report
        CircuitBreakerReport {
            state: CircuitState::Closed,
            error_rate: 0.0,
            operations_count: self.operation_count,
            failures_count: 0,
            backoff_duration: None,
            next_attempt: None,
            state_transitions: 0,
        }
    }

    fn adapt_backoff(&mut self, _recovery_time: Duration) {
        // No-op - no actual backoff to adapt
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Test constants are safe for exact comparison
mod tests {
    use super::*;
    use crate::hook_profiler::WorkloadClassifier;

    #[test]
    fn test_null_circuit_breaker_lifecycle() {
        let mut breaker = NullCircuitBreaker::new();

        // Always closed, never blocks
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(!breaker.is_open());

        let context = OperationContext {
            operation_name: "test_op".to_string(),
            workload: WorkloadClassifier::Heavy,
            duration: Duration::from_millis(100),
            success: false,
        };

        // Always allows operations regardless of context
        assert!(breaker.allow_operation(&context));

        // Manual trip does nothing
        breaker.trip();
        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_operation(&context));

        // Reset does nothing (but doesn't fail)
        breaker.reset();
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_null_circuit_breaker_operations() {
        let mut breaker = NullCircuitBreaker::new();

        let success_context = OperationContext {
            operation_name: "success_op".to_string(),
            workload: WorkloadClassifier::Micro,
            duration: Duration::from_millis(1),
            success: true,
        };

        let failure_context = OperationContext {
            operation_name: "failure_op".to_string(),
            workload: WorkloadClassifier::Heavy,
            duration: Duration::from_millis(1000),
            success: false,
        };

        // Record many failures - should never change behavior
        for _ in 0..1000 {
            breaker.record_operation(failure_context.clone());
            assert!(breaker.allow_operation(&failure_context));
            assert_eq!(breaker.state(), CircuitState::Closed);
        }

        // Mix of success and failure - still no change
        for _ in 0..100 {
            breaker.record_operation(success_context.clone());
            breaker.record_operation(failure_context.clone());
        }

        assert_eq!(breaker.state(), CircuitState::Closed);
        assert!(breaker.allow_operation(&failure_context));
    }

    #[test]
    fn test_null_circuit_breaker_report() {
        let mut breaker = NullCircuitBreaker::new();

        let context = OperationContext {
            operation_name: "test".to_string(),
            workload: WorkloadClassifier::Light,
            duration: Duration::from_millis(50),
            success: false,
        };

        // Record some operations
        for _ in 0..10 {
            breaker.record_operation(context.clone());
        }

        let report = breaker.report();
        assert_eq!(report.state, CircuitState::Closed);
        assert_eq!(report.operations_count, 10);
        assert_eq!(report.failures_count, 0); // Null implementation reports no failures
        assert_eq!(report.error_rate, 0.0);
        assert!(report.backoff_duration.is_none());
        assert!(report.next_attempt.is_none());
        assert_eq!(report.state_transitions, 0);
    }

    #[test]
    fn test_null_circuit_breaker_with_custom_config() {
        let config = CircuitBreakerConfig::heavy();
        let breaker = NullCircuitBreaker::with_config(config.clone());

        // Config is stored but doesn't affect behavior
        assert_eq!(
            breaker.config().heavy_operation_threshold,
            config.heavy_operation_threshold
        );
        assert_eq!(breaker.config().min_backoff_ms, config.min_backoff_ms);

        // Still always allows operations
        let context = OperationContext {
            operation_name: "heavy_op".to_string(),
            workload: WorkloadClassifier::Heavy,
            duration: Duration::from_millis(5000),
            success: false,
        };

        assert!(breaker.allow_operation(&context));
        assert_eq!(breaker.state(), CircuitState::Closed);
    }

    #[test]
    fn test_null_circuit_breaker_safe_for_tests() {
        // Verify it's safe to use in test scenarios without side effects
        let mut breaker = NullCircuitBreaker::new();

        let context = OperationContext {
            operation_name: "test_hook".to_string(),
            workload: WorkloadClassifier::Medium,
            duration: Duration::from_millis(25),
            success: false,
        };

        // These should all be safe no-ops that never block execution
        for i in 0..10000 {
            breaker.record_operation(OperationContext {
                operation_name: format!("op_{i}"),
                success: i % 2 == 0, // Mix success/failure
                ..context.clone()
            });

            // Should never block even with high failure rates
            assert!(breaker.allow_operation(&context));

            if i % 1000 == 0 {
                breaker.trip(); // Should be safe no-ops
                breaker.reset();
                breaker.adapt_backoff(Duration::from_secs(1));
            }
        }

        // State should remain unchanged throughout
        assert_eq!(breaker.state(), CircuitState::Closed);

        let report = breaker.report();
        assert_eq!(report.operations_count, 10000);
        assert_eq!(report.state, CircuitState::Closed);
        assert_eq!(report.error_rate, 0.0); // Null implementation shows no errors
    }
}
