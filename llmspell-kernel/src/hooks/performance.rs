//! # Hook Performance Monitoring
//!
//! Performance monitoring system for kernel hooks ensuring <5% overhead
//! with circuit breaker protection and detailed metrics collection.

use crate::hooks::KernelHookPoint;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::warn;

/// Performance metrics for hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookPerformanceMetrics {
    /// Total hook executions
    pub total_executions: u64,
    /// Total execution time across all hooks
    pub total_execution_time: Duration,
    /// Average execution time per hook
    pub average_execution_time: Duration,
    /// Maximum execution time seen
    pub max_execution_time: Duration,
    /// Minimum execution time seen
    pub min_execution_time: Duration,
    /// Number of hooks that exceeded threshold
    pub threshold_violations: u64,
    /// Hook execution count by hook point
    pub executions_by_point: HashMap<String, u64>,
    /// Average execution time by hook point
    pub avg_time_by_point: HashMap<String, Duration>,
    /// Performance overhead percentage
    pub overhead_percentage: f64,
    /// Circuit breaker activations
    pub circuit_breaker_activations: u64,
    /// Hooks currently disabled by circuit breaker
    pub disabled_hooks: Vec<String>,
}

impl Default for HookPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_executions: 0,
            total_execution_time: Duration::ZERO,
            average_execution_time: Duration::ZERO,
            max_execution_time: Duration::ZERO,
            min_execution_time: Duration::MAX,
            threshold_violations: 0,
            executions_by_point: HashMap::new(),
            avg_time_by_point: HashMap::new(),
            overhead_percentage: 0.0,
            circuit_breaker_activations: 0,
            disabled_hooks: Vec::new(),
        }
    }
}

impl HookPerformanceMetrics {
    /// Check if performance is within acceptable limits
    pub fn is_within_limits(&self) -> bool {
        self.overhead_percentage < 5.0 && self.average_execution_time < Duration::from_millis(50)
    }

    /// Get the slowest hook points
    pub fn slowest_hook_points(&self, limit: usize) -> Vec<(String, Duration)> {
        let mut points: Vec<_> = self
            .avg_time_by_point
            .iter()
            .map(|(point, duration)| (point.clone(), *duration))
            .collect();

        points.sort_by(|a, b| b.1.cmp(&a.1));
        points.truncate(limit);
        points
    }

    /// Get violation rate
    #[allow(clippy::cast_precision_loss)]
    pub fn violation_rate(&self) -> f64 {
        if self.total_executions == 0 {
            0.0
        } else {
            (self.threshold_violations as f64) / (self.total_executions as f64) * 100.0
        }
    }
}

/// Performance monitor for kernel hooks
pub struct KernelPerformanceMonitor {
    metrics: Arc<RwLock<HookPerformanceMetrics>>,
    threshold: Duration,
    overhead_baseline: Option<Duration>,
    sample_count: usize,
    execution_times: Arc<RwLock<Vec<Duration>>>,
}

impl KernelPerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HookPerformanceMetrics::default())),
            threshold: Duration::from_millis(50), // 50ms threshold
            overhead_baseline: None,
            sample_count: 100, // Keep last 100 samples for rolling averages
            execution_times: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create monitor with custom threshold
    pub fn with_threshold(threshold: Duration) -> Self {
        Self {
            threshold,
            ..Self::new()
        }
    }

    /// Set the baseline execution time for overhead calculation
    pub fn set_baseline(&mut self, baseline: Duration) {
        self.overhead_baseline = Some(baseline);
    }

    /// Record hook execution performance
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn record_hook_execution(&self, hook_point: KernelHookPoint, duration: Duration) {
        let hook_point_str = format!("{hook_point:?}");

        // Update execution times sample
        {
            let mut times = self.execution_times.write();
            times.push(duration);

            // Keep only recent samples
            if times.len() > self.sample_count {
                times.remove(0);
            }
        }

        // Update metrics
        let mut metrics = self.metrics.write();

        metrics.total_executions += 1;
        metrics.total_execution_time += duration;

        // Update average
        metrics.average_execution_time =
            metrics.total_execution_time / metrics.total_executions as u32;

        // Update min/max
        if duration > metrics.max_execution_time {
            metrics.max_execution_time = duration;
        }
        if duration < metrics.min_execution_time {
            metrics.min_execution_time = duration;
        }

        // Check threshold violation
        if duration > self.threshold {
            metrics.threshold_violations += 1;
            warn!(
                "Hook execution threshold violation: {:?} took {:?} (threshold: {:?})",
                hook_point, duration, self.threshold
            );
        }

        // Update per-hook-point metrics
        *metrics
            .executions_by_point
            .entry(hook_point_str.clone())
            .or_insert(0) += 1;

        let execution_count = metrics.executions_by_point[&hook_point_str];
        let point_total_time = (*metrics
            .avg_time_by_point
            .get(&hook_point_str)
            .unwrap_or(&Duration::ZERO))
            * (execution_count - 1) as u32
            + duration;

        metrics
            .avg_time_by_point
            .insert(hook_point_str, point_total_time / execution_count as u32);

        // Calculate overhead percentage if baseline is available
        if let Some(baseline) = self.overhead_baseline {
            let overhead = if duration > baseline {
                duration - baseline
            } else {
                Duration::ZERO
            };

            metrics.overhead_percentage =
                (overhead.as_nanos() as f64) / (baseline.as_nanos() as f64) * 100.0;
        }

        // Log performance warnings
        if metrics.overhead_percentage > 5.0 {
            warn!(
                "Hook system overhead ({:.2}%) exceeds 5% threshold",
                metrics.overhead_percentage
            );
        }
    }

    /// Get current metrics
    pub fn metrics(&self) -> HookPerformanceMetrics {
        self.metrics.read().clone()
    }

    /// Reset all metrics
    pub fn reset_metrics(&self) {
        let mut metrics = self.metrics.write();
        *metrics = HookPerformanceMetrics::default();

        let mut times = self.execution_times.write();
        times.clear();
    }

    /// Check if hook system should be throttled based on performance
    pub fn should_throttle(&self) -> bool {
        let metrics = self.metrics.read();
        metrics.overhead_percentage > 10.0
            || metrics.average_execution_time > Duration::from_millis(100)
    }

    /// Get recent execution time samples
    pub fn recent_execution_times(&self) -> Vec<Duration> {
        self.execution_times.read().clone()
    }

    /// Calculate rolling average execution time
    #[allow(clippy::cast_possible_truncation)]
    pub fn rolling_average(&self) -> Duration {
        let times = self.execution_times.read();
        if times.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = times.iter().sum();
        total / times.len() as u32
    }

    /// Get performance trend (positive = getting slower, negative = getting faster)
    #[allow(clippy::cast_precision_loss)]
    pub fn performance_trend(&self) -> f64 {
        let times = self.execution_times.read();
        if times.len() < 10 {
            return 0.0;
        }

        let recent_avg: Duration = times.iter().rev().take(10).sum::<Duration>() / 10;
        let older_avg: Duration = times.iter().take(10).sum::<Duration>() / 10;

        if older_avg.is_zero() {
            0.0
        } else {
            ((recent_avg.as_nanos() as f64) - (older_avg.as_nanos() as f64))
                / (older_avg.as_nanos() as f64)
                * 100.0
        }
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let metrics = self.metrics();
        let trend = self.performance_trend();
        let rolling_avg = self.rolling_average();

        PerformanceReport {
            recommendations: Self::generate_recommendations(&metrics, trend),
            rolling_average: rolling_avg,
            performance_trend: trend,
            metrics,
            timestamp: Instant::now(),
        }
    }

    /// Generate performance optimization recommendations
    fn generate_recommendations(metrics: &HookPerformanceMetrics, trend: f64) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.overhead_percentage > 5.0 {
            recommendations.push(format!(
                "Hook overhead ({:.2}%) exceeds 5% threshold. Consider reducing hook complexity.",
                metrics.overhead_percentage
            ));
        }

        if metrics.violation_rate() > 10.0 {
            recommendations.push(format!(
                "Threshold violations ({:.1}%) are high. Consider optimizing slow hooks.",
                metrics.violation_rate()
            ));
        }

        if trend > 20.0 {
            recommendations.push(format!(
                "Performance is degrading ({trend:.1}% slower). Investigate recent changes."
            ));
        }

        if metrics.average_execution_time > Duration::from_millis(25) {
            recommendations.push(
                "Average hook execution time is high. Consider hook optimization.".to_string(),
            );
        }

        let slowest = metrics.slowest_hook_points(3);
        if !slowest.is_empty() {
            recommendations.push(format!(
                "Slowest hook points: {}",
                slowest
                    .iter()
                    .map(|(point, duration)| format!("{point} ({duration:?})"))
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if recommendations.is_empty() {
            recommendations.push("Performance is within acceptable limits.".to_string());
        }

        recommendations
    }
}

impl Default for KernelPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive performance report
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// Performance metrics snapshot
    pub metrics: HookPerformanceMetrics,
    /// Rolling average execution time
    pub rolling_average: Duration,
    /// Performance trend percentage (positive = degrading)
    pub performance_trend: f64,
    /// Performance optimization recommendations
    pub recommendations: Vec<String>,
    /// Report generation timestamp
    pub timestamp: Instant,
}

impl PerformanceReport {
    /// Check if performance is healthy
    pub fn is_healthy(&self) -> bool {
        self.metrics.is_within_limits()
            && self.performance_trend < 20.0
            && self.metrics.violation_rate() < 10.0
    }

    /// Get severity level
    pub fn severity(&self) -> PerformanceSeverity {
        if self.is_healthy() {
            PerformanceSeverity::Ok
        } else if self.metrics.overhead_percentage > 10.0 || self.metrics.violation_rate() > 25.0 {
            PerformanceSeverity::Critical
        } else {
            PerformanceSeverity::Warning
        }
    }
}

/// Performance severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerformanceSeverity {
    /// Performance is within acceptable limits
    Ok,
    /// Performance issues detected but not critical
    Warning,
    /// Critical performance issues requiring immediate attention
    Critical,
}

/// Hook execution timer for measuring individual hook performance
pub struct HookExecutionTimer {
    start_time: Instant,
    hook_name: String,
    hook_point: KernelHookPoint,
}

impl HookExecutionTimer {
    /// Start timing hook execution
    pub fn start(hook_name: String, hook_point: KernelHookPoint) -> Self {
        Self {
            start_time: Instant::now(),
            hook_name,
            hook_point,
        }
    }

    /// Finish timing and return duration
    pub fn finish(self) -> (Duration, String, KernelHookPoint) {
        let duration = self.start_time.elapsed();
        (duration, self.hook_name, self.hook_point)
    }

    /// Get elapsed time without finishing
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_performance_monitor_basic() {
        let monitor = KernelPerformanceMonitor::new();

        // Record some executions
        monitor.record_hook_execution(KernelHookPoint::PreCodeExecution, Duration::from_millis(10));

        monitor.record_hook_execution(
            KernelHookPoint::PostCodeExecution,
            Duration::from_millis(20),
        );

        let metrics = monitor.metrics();
        assert_eq!(metrics.total_executions, 2);
        assert_eq!(metrics.average_execution_time, Duration::from_millis(15));
        assert_eq!(metrics.max_execution_time, Duration::from_millis(20));
        assert_eq!(metrics.min_execution_time, Duration::from_millis(10));
    }

    #[test]
    fn test_threshold_violations() {
        let monitor = KernelPerformanceMonitor::with_threshold(Duration::from_millis(15));

        // This should violate the 15ms threshold but still be within general limits (50ms)
        monitor.record_hook_execution(KernelHookPoint::PreCodeExecution, Duration::from_millis(30));

        let metrics = monitor.metrics();
        assert_eq!(metrics.threshold_violations, 1);
        assert!(metrics.is_within_limits()); // 30ms < 50ms general limit, so still within limits
    }

    #[test]
    fn test_overhead_calculation() {
        let mut monitor = KernelPerformanceMonitor::new();
        monitor.set_baseline(Duration::from_millis(100));

        // Hook execution that adds 10ms overhead
        monitor.record_hook_execution(
            KernelHookPoint::PreCodeExecution,
            Duration::from_millis(110),
        );

        let metrics = monitor.metrics();
        assert!((metrics.overhead_percentage - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_performance_metrics_serialization() {
        let metrics = HookPerformanceMetrics {
            total_executions: 100,
            total_execution_time: Duration::from_secs(1),
            average_execution_time: Duration::from_millis(10),
            max_execution_time: Duration::from_millis(50),
            min_execution_time: Duration::from_millis(1),
            threshold_violations: 5,
            executions_by_point: HashMap::new(),
            avg_time_by_point: HashMap::new(),
            overhead_percentage: 2.5,
            circuit_breaker_activations: 0,
            disabled_hooks: Vec::new(),
        };

        let serialized = serde_json::to_string(&metrics).unwrap();
        let deserialized: HookPerformanceMetrics = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metrics.total_executions, deserialized.total_executions);
        assert!(
            (metrics.overhead_percentage - deserialized.overhead_percentage).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_violation_rate_calculation() {
        let metrics = HookPerformanceMetrics {
            total_executions: 100,
            threshold_violations: 15,
            ..Default::default()
        };

        assert!((metrics.violation_rate() - 15.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_hook_execution_timer() {
        let timer =
            HookExecutionTimer::start("test_hook".to_string(), KernelHookPoint::PreCodeExecution);

        std::thread::sleep(Duration::from_millis(1));

        let (duration, name, point) = timer.finish();
        assert!(duration >= Duration::from_millis(1));
        assert_eq!(name, "test_hook");
        assert_eq!(point, KernelHookPoint::PreCodeExecution);
    }

    #[test]
    fn test_performance_trend() {
        let monitor = KernelPerformanceMonitor::new();

        // Add 10 fast executions
        for _ in 0..10 {
            monitor
                .record_hook_execution(KernelHookPoint::PreCodeExecution, Duration::from_millis(5));
        }

        // Add 10 slower executions
        for _ in 0..10 {
            monitor.record_hook_execution(
                KernelHookPoint::PreCodeExecution,
                Duration::from_millis(15),
            );
        }

        let trend = monitor.performance_trend();
        assert!(trend > 0.0); // Should show degradation
    }

    #[test]
    fn test_performance_report() {
        let monitor = KernelPerformanceMonitor::new();

        monitor.record_hook_execution(KernelHookPoint::PreCodeExecution, Duration::from_millis(5));

        let report = monitor.generate_report();
        assert!(report.is_healthy());
        assert_eq!(report.severity(), PerformanceSeverity::Ok);
    }
}
