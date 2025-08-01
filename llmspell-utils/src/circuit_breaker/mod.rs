//! ABOUTME: Circuit breaker pattern implementation for protecting external service calls
//! ABOUTME: Prevents cascading failures by monitoring errors and opening/closing circuits based on thresholds

use std::collections::HashMap;
use std::error::Error as StdError;
use std::sync::Arc;
use std::time::Instant;
use thiserror::Error;
use tokio::sync::RwLock;

mod config;
mod metrics;
mod state;

pub use config::{CircuitBreakerConfig, ServicePresets, ThresholdConfig};
pub use metrics::{CircuitMetrics, MetricsCollector as CircuitMetricsCollector};
pub use state::{CircuitState, StateTransition};

/// Circuit breaker errors
#[derive(Debug, Error)]
pub enum CircuitBreakerError {
    /// Circuit is open and rejecting requests
    #[error("Circuit breaker is open: {reason}")]
    CircuitOpen {
        /// Reason why the circuit is open
        reason: String,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Metric collection error
    #[error("Metrics error: {0}")]
    MetricsError(String),
}

/// Result type for circuit breaker operations
pub type CircuitBreakerResult<T> = Result<T, CircuitBreakerError>;

/// Circuit breaker instance for a single service
#[derive(Debug)]
pub struct CircuitBreaker {
    /// Current state of the circuit
    state: Arc<RwLock<CircuitState>>,
    /// Configuration for this circuit
    config: CircuitBreakerConfig,
    /// Metrics collector
    metrics: Arc<CircuitMetricsCollector>,
    /// Last state transition time
    last_transition: Arc<RwLock<Instant>>,
    /// Consecutive failure count
    failure_count: Arc<RwLock<u32>>,
    /// Consecutive success count in half-open state
    success_count: Arc<RwLock<u32>>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with configuration
    #[must_use]
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(CircuitState::Closed)),
            config,
            metrics: Arc::new(CircuitMetricsCollector::new()),
            last_transition: Arc::new(RwLock::new(Instant::now())),
            failure_count: Arc::new(RwLock::new(0)),
            success_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Check if the circuit allows a request
    ///
    /// # Errors
    ///
    /// Returns `CircuitBreakerError::CircuitOpen` if the circuit is open and rejecting requests
    pub async fn allow_request(&self) -> CircuitBreakerResult<()> {
        let mut state = self.state.write().await;
        let current_state = *state;

        match current_state {
            CircuitState::Closed => {
                self.metrics.record_allowed().await;
                Ok(())
            }
            CircuitState::Open => {
                // Check if we should transition to half-open
                let last_transition = *self.last_transition.read().await;
                if last_transition.elapsed() >= self.config.reset_timeout {
                    *state = CircuitState::HalfOpen;
                    *self.last_transition.write().await = Instant::now();
                    *self.success_count.write().await = 0;

                    self.metrics
                        .record_state_change(current_state, CircuitState::HalfOpen)
                        .await;
                    self.metrics.record_allowed().await;
                    Ok(())
                } else {
                    self.metrics.record_rejected().await;
                    Err(CircuitBreakerError::CircuitOpen {
                        reason: format!(
                            "Circuit will reset in {:?}",
                            self.config.reset_timeout - last_transition.elapsed()
                        ),
                    })
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests in half-open state
                let success_count = *self.success_count.read().await;
                if success_count < self.config.test_request_count {
                    self.metrics.record_allowed().await;
                    Ok(())
                } else {
                    self.metrics.record_rejected().await;
                    Err(CircuitBreakerError::CircuitOpen {
                        reason: "Half-open state test limit reached".to_string(),
                    })
                }
            }
        }
    }

    /// Record a successful request
    pub async fn record_success(&self) {
        let mut state = self.state.write().await;
        let current_state = *state;

        self.metrics.record_success().await;

        match current_state {
            CircuitState::Closed => {
                // Reset failure count on success
                *self.failure_count.write().await = 0;
            }
            CircuitState::HalfOpen => {
                let mut success_count = self.success_count.write().await;
                *success_count += 1;

                // Check if we should close the circuit
                if *success_count >= self.config.success_threshold_count {
                    *state = CircuitState::Closed;
                    *self.failure_count.write().await = 0;
                    *self.last_transition.write().await = Instant::now();

                    self.metrics
                        .record_state_change(current_state, CircuitState::Closed)
                        .await;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
                tracing::warn!("Success recorded while circuit is open");
            }
        }
    }

    /// Record a failed request
    pub async fn record_failure(&self) {
        let mut state = self.state.write().await;
        let current_state = *state;

        self.metrics.record_failure().await;

        match current_state {
            CircuitState::Closed => {
                let mut failure_count = self.failure_count.write().await;
                *failure_count += 1;

                // Check if we should open the circuit
                if *failure_count >= self.config.failure_threshold_count {
                    *state = CircuitState::Open;
                    *self.last_transition.write().await = Instant::now();

                    self.metrics
                        .record_state_change(current_state, CircuitState::Open)
                        .await;

                    // Alert on circuit open
                    if let Some(ref alert_handler) = self.config.alert_handler {
                        alert_handler(format!("Circuit opened after {} failures", *failure_count));
                    }
                }
            }
            CircuitState::HalfOpen => {
                // Single failure in half-open state reopens the circuit
                *state = CircuitState::Open;
                *self.failure_count.write().await = 0;
                *self.last_transition.write().await = Instant::now();

                self.metrics
                    .record_state_change(current_state, CircuitState::Open)
                    .await;
            }
            CircuitState::Open => {
                // Already open, nothing to do
            }
        }
    }

    /// Execute a function with circuit breaker protection
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The circuit is open and rejecting requests
    /// - The underlying operation fails
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, Box<dyn StdError + Send + Sync>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Send,
        E: StdError + Send + Sync + 'static,
    {
        // Check if request is allowed
        self.allow_request()
            .await
            .map_err(|e| Box::new(e) as Box<dyn StdError + Send + Sync>)?;

        // Execute the operation
        match operation().await {
            Ok(result) => {
                self.record_success().await;
                Ok(result)
            }
            Err(error) => {
                self.record_failure().await;
                Err(Box::new(error) as Box<dyn StdError + Send + Sync>)
            }
        }
    }

    /// Get current circuit state
    pub async fn current_state(&self) -> CircuitState {
        *self.state.read().await
    }

    /// Get circuit metrics
    pub async fn metrics(&self) -> CircuitMetrics {
        self.metrics.get_metrics().await
    }

    /// Force the circuit to a specific state (for testing/manual intervention)
    pub async fn force_state(&self, new_state: CircuitState) {
        let mut state = self.state.write().await;
        let old_state = *state;
        *state = new_state;
        *self.last_transition.write().await = Instant::now();

        if old_state != new_state {
            self.metrics.record_state_change(old_state, new_state).await;
        }
    }
}

/// Per-service circuit breaker manager
pub struct CircuitBreakerManager {
    /// Circuit breakers per service
    breakers: Arc<RwLock<HashMap<String, Arc<CircuitBreaker>>>>,
    /// Default configuration builder for new circuits
    default_config_builder: fn() -> CircuitBreakerConfig,
}

impl Default for CircuitBreakerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CircuitBreakerManager {
    /// Create a new circuit breaker manager with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            default_config_builder: CircuitBreakerConfig::default,
        }
    }

    /// Create a new circuit breaker manager with custom default configuration
    pub fn with_default_config(config_builder: fn() -> CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            default_config_builder: config_builder,
        }
    }

    /// Get or create a circuit breaker for a service
    pub async fn get_or_create(&self, service: &str) -> Arc<CircuitBreaker> {
        let breakers = self.breakers.read().await;
        if let Some(breaker) = breakers.get(service) {
            return Arc::clone(breaker);
        }
        drop(breakers);

        // Create new circuit breaker
        let mut breakers = self.breakers.write().await;
        let breaker = Arc::new(CircuitBreaker::new((self.default_config_builder)()));
        breakers.insert(service.to_string(), Arc::clone(&breaker));
        breaker
    }

    /// Configure a specific service with custom settings
    pub async fn configure_service(&self, service: &str, config: CircuitBreakerConfig) {
        let mut breakers = self.breakers.write().await;
        let breaker = Arc::new(CircuitBreaker::new(config));
        breakers.insert(service.to_string(), breaker);
    }

    /// Execute a function with circuit breaker protection for a service
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The circuit is open and rejecting requests
    /// - The underlying operation fails
    pub async fn execute<F, T, E>(
        &self,
        service: &str,
        operation: F,
    ) -> Result<T, Box<dyn StdError + Send + Sync>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>
            + Send,
        E: StdError + Send + Sync + 'static,
    {
        let breaker = self.get_or_create(service).await;
        breaker.execute(operation).await
    }

    /// Get metrics for all services
    pub async fn all_metrics(&self) -> HashMap<String, CircuitMetrics> {
        let breakers = self.breakers.read().await;
        let mut metrics = HashMap::new();

        for (service, breaker) in breakers.iter() {
            metrics.insert(service.clone(), breaker.metrics().await);
        }

        metrics
    }

    /// Get services in open state
    pub async fn open_circuits(&self) -> Vec<String> {
        let breakers = self.breakers.read().await;
        let mut open_services = Vec::new();

        for (service, breaker) in breakers.iter() {
            if breaker.current_state().await == CircuitState::Open {
                open_services.push(service.clone());
            }
        }

        open_services
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;
    use std::time::Duration;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_circuit_breaker_state_transitions() {
        let config = CircuitBreakerConfig {
            failure_threshold_count: 2,
            reset_timeout: Duration::from_millis(100),
            success_threshold_count: 2,
            test_request_count: 3,
            ..Default::default()
        };

        let breaker = CircuitBreaker::new(config);

        // Initially closed
        assert_eq!(breaker.current_state().await, CircuitState::Closed);
        assert!(breaker.allow_request().await.is_ok());

        // Record failures to open circuit
        breaker.record_failure().await;
        breaker.record_failure().await;
        assert_eq!(breaker.current_state().await, CircuitState::Open);

        // Should reject requests when open
        assert!(breaker.allow_request().await.is_err());

        // Wait for reset timeout
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should transition to half-open
        assert!(breaker.allow_request().await.is_ok());
        assert_eq!(breaker.current_state().await, CircuitState::HalfOpen);

        // Success in half-open should eventually close
        breaker.record_success().await;
        breaker.record_success().await;
        assert_eq!(breaker.current_state().await, CircuitState::Closed);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_circuit_breaker_execute() {
        let breaker = CircuitBreaker::new(CircuitBreakerConfig::default());

        // Successful operation
        let result = breaker
            .execute(|| Box::pin(async { Ok::<_, std::io::Error>(42) }))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);

        // Failed operation
        let result = breaker
            .execute(|| {
                Box::pin(async {
                    Err::<i32, _>(std::io::Error::new(std::io::ErrorKind::Other, "test error"))
                })
            })
            .await;

        assert!(result.is_err());
    }
}
