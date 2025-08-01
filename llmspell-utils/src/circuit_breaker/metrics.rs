//! ABOUTME: Circuit breaker metrics collection and monitoring
//! ABOUTME: Tracks success/failure rates, state changes, and performance metrics

use super::CircuitState;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Maximum number of response times to keep
const MAX_RESPONSE_TIMES: usize = 100;

/// Circuit breaker metrics
#[derive(Debug, Clone, Default)]
pub struct CircuitMetrics {
    /// Total requests allowed
    pub total_allowed: u64,
    /// Total requests rejected
    pub total_rejected: u64,
    /// Total successful operations
    pub total_successes: u64,
    /// Total failed operations
    pub total_failures: u64,
    /// Current state
    pub current_state: CircuitState,
    /// Time in current state
    pub time_in_state: Duration,
    /// State change count
    pub state_changes: u64,
    /// Last state change time
    pub last_state_change: Option<Instant>,
    /// Recent response times (for allowed requests)
    pub recent_response_times: VecDeque<Duration>,
}

impl CircuitMetrics {
    /// Calculate success rate percentage
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            100.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = (self.total_successes as f64 / total as f64) * 100.0;
            rate
        }
    }

    /// Calculate rejection rate percentage
    #[must_use]
    pub fn rejection_rate(&self) -> f64 {
        let total = self.total_allowed + self.total_rejected;
        if total == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = (self.total_rejected as f64 / total as f64) * 100.0;
            rate
        }
    }

    /// Get average response time
    #[must_use]
    pub fn average_response_time(&self) -> Option<Duration> {
        if self.recent_response_times.is_empty() {
            None
        } else {
            let sum: Duration = self.recent_response_times.iter().sum();
            Some(sum / u32::try_from(self.recent_response_times.len()).unwrap_or(1))
        }
    }

    /// Get p95 response time
    #[must_use]
    pub fn p95_response_time(&self) -> Option<Duration> {
        if self.recent_response_times.is_empty() {
            None
        } else {
            let mut times: Vec<Duration> = self.recent_response_times.iter().copied().collect();
            times.sort();
            #[allow(
                clippy::cast_precision_loss,
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss
            )]
            let index = ((times.len() as f64 * 0.95).ceil() as usize).saturating_sub(1);
            times.get(index.min(times.len() - 1)).copied()
        }
    }

    /// Check if circuit is healthy
    #[must_use]
    pub fn is_healthy(&self) -> bool {
        self.current_state == CircuitState::Closed && self.success_rate() > 95.0
    }

    /// Check if circuit is degraded
    #[must_use]
    pub fn is_degraded(&self) -> bool {
        self.current_state == CircuitState::HalfOpen
            || (self.current_state == CircuitState::Closed && self.success_rate() < 90.0)
    }

    /// Check if circuit is critical
    #[must_use]
    pub fn is_critical(&self) -> bool {
        self.current_state == CircuitState::Open
    }
}

/// Metrics collector for circuit breakers
#[derive(Debug)]
pub struct MetricsCollector {
    /// Current metrics
    metrics: RwLock<CircuitMetrics>,
    /// State entry time
    state_entry_time: RwLock<Instant>,
    /// Response time tracking
    response_timer: RwLock<Option<Instant>>,
}

impl MetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: RwLock::new(CircuitMetrics {
                current_state: CircuitState::Closed,
                ..Default::default()
            }),
            state_entry_time: RwLock::new(Instant::now()),
            response_timer: RwLock::new(None),
        }
    }

    /// Record an allowed request
    pub async fn record_allowed(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_allowed += 1;

        // Start response timer
        *self.response_timer.write().await = Some(Instant::now());
    }

    /// Record a rejected request
    pub async fn record_rejected(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_rejected += 1;
    }

    /// Record a successful operation
    pub async fn record_success(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_successes += 1;

        // Record response time if timer was started
        if let Some(start_time) = *self.response_timer.read().await {
            let response_time = start_time.elapsed();
            if metrics.recent_response_times.len() >= MAX_RESPONSE_TIMES {
                metrics.recent_response_times.pop_front();
            }
            metrics.recent_response_times.push_back(response_time);
        }

        // Clear timer
        *self.response_timer.write().await = None;
    }

    /// Record a failed operation
    pub async fn record_failure(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.total_failures += 1;

        // Clear timer
        *self.response_timer.write().await = None;
    }

    /// Record a state change
    pub async fn record_state_change(&self, from: CircuitState, to: CircuitState) {
        let mut metrics = self.metrics.write().await;
        let state_entry_time = *self.state_entry_time.read().await;

        metrics.time_in_state = state_entry_time.elapsed();
        metrics.current_state = to;
        metrics.state_changes += 1;
        metrics.last_state_change = Some(Instant::now());

        *self.state_entry_time.write().await = Instant::now();

        tracing::info!(
            "Circuit state changed from {} to {} after {:?}",
            from,
            to,
            metrics.time_in_state
        );
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> CircuitMetrics {
        let mut metrics = self.metrics.read().await.clone();
        let state_entry_time = *self.state_entry_time.read().await;
        metrics.time_in_state = state_entry_time.elapsed();
        metrics
    }

    /// Reset metrics (for testing)
    pub async fn reset(&self) {
        *self.metrics.write().await = CircuitMetrics {
            current_state: CircuitState::Closed,
            ..Default::default()
        };
        *self.state_entry_time.write().await = Instant::now();
        *self.response_timer.write().await = None;
    }
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Alert levels for circuit breaker monitoring
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlertLevel {
    /// Circuit is healthy
    Healthy,
    /// Circuit is showing signs of degradation
    Warning,
    /// Circuit is in critical state
    Critical,
}

impl From<&CircuitMetrics> for AlertLevel {
    fn from(metrics: &CircuitMetrics) -> Self {
        if metrics.is_critical() {
            AlertLevel::Critical
        } else if metrics.is_degraded() {
            AlertLevel::Warning
        } else {
            AlertLevel::Healthy
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "util")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = MetricsCollector::new();

        // Record some operations
        collector.record_allowed().await;
        collector.record_success().await;

        collector.record_allowed().await;
        collector.record_failure().await;

        collector.record_rejected().await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.total_allowed, 2);
        assert_eq!(metrics.total_rejected, 1);
        assert_eq!(metrics.total_successes, 1);
        assert_eq!(metrics.total_failures, 1);
        assert!((metrics.success_rate() - 50.0).abs() < f64::EPSILON);
        assert!((metrics.rejection_rate() - 33.333_333).abs() < 0.001);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_state_tracking() {
        let collector = MetricsCollector::new();

        // Initial state
        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.current_state, CircuitState::Closed);
        assert_eq!(metrics.state_changes, 0);

        // Record state change
        collector
            .record_state_change(CircuitState::Closed, CircuitState::Open)
            .await;

        let metrics = collector.get_metrics().await;
        assert_eq!(metrics.current_state, CircuitState::Open);
        assert_eq!(metrics.state_changes, 1);
        assert!(metrics.last_state_change.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_alert_levels() {
        // Healthy state
        let mut metrics = CircuitMetrics {
            current_state: CircuitState::Closed,
            total_successes: 95,
            total_failures: 5,
            ..Default::default()
        };
        assert_eq!(AlertLevel::from(&metrics), AlertLevel::Healthy);

        // Warning state
        metrics.current_state = CircuitState::HalfOpen;
        assert_eq!(AlertLevel::from(&metrics), AlertLevel::Warning);

        // Critical state
        metrics.current_state = CircuitState::Open;
        assert_eq!(AlertLevel::from(&metrics), AlertLevel::Critical);
    }
}
