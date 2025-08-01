// ABOUTME: Performance monitoring for hook execution with metrics collection
// ABOUTME: Tracks execution time, memory usage, and provides performance insights

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance metrics for a hook
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub execution_count: u64,
    pub total_duration: Duration,
    pub min_duration: Duration,
    pub max_duration: Duration,
    pub avg_duration: Duration,
    pub p50_duration: Duration,
    pub p95_duration: Duration,
    pub p99_duration: Duration,
    pub last_execution: Option<DateTime<Utc>>,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            execution_count: 0,
            total_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            max_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            p50_duration: Duration::ZERO,
            p95_duration: Duration::ZERO,
            p99_duration: Duration::ZERO,
            last_execution: None,
        }
    }
}

/// Performance sample for percentile calculations
#[derive(Debug, Clone)]
struct PerformanceSample {
    duration: Duration,
    timestamp: DateTime<Utc>,
}

/// Performance monitor for hook execution
pub struct PerformanceMonitor {
    samples: Arc<RwLock<HashMap<String, Vec<PerformanceSample>>>>,
    counters: Arc<RwLock<HashMap<String, AtomicU64>>>,
    config: PerformanceConfig,
}

/// Configuration for performance monitoring
#[derive(Debug, Clone)]
pub struct PerformanceConfig {
    /// Maximum samples to keep per hook
    pub max_samples: usize,
    /// Sample retention duration
    pub retention_period: Duration,
    /// Enable percentile calculations
    pub calculate_percentiles: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            max_samples: 1000,
            retention_period: Duration::from_secs(3600), // 1 hour
            calculate_percentiles: true,
        }
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new() -> Self {
        Self::with_config(PerformanceConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: PerformanceConfig) -> Self {
        Self {
            samples: Arc::new(RwLock::new(HashMap::new())),
            counters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Start tracking a hook execution
    pub fn start_execution(&self, hook_name: &str) -> ExecutionTimer {
        ExecutionTimer {
            hook_name: hook_name.to_string(),
            start_time: Instant::now(),
            monitor: self.clone(),
            completed: false,
        }
    }

    /// Record an execution duration
    pub fn record_execution(&self, hook_name: &str, duration: Duration) {
        let mut samples = self.samples.write();
        let hook_samples = samples.entry(hook_name.to_string()).or_default();

        // Add new sample
        hook_samples.push(PerformanceSample {
            duration,
            timestamp: Utc::now(),
        });

        // Clean up old samples
        let cutoff = Utc::now() - chrono::Duration::from_std(self.config.retention_period).unwrap();
        hook_samples.retain(|s| s.timestamp > cutoff);

        // Limit sample count
        if hook_samples.len() > self.config.max_samples {
            let to_remove = hook_samples.len() - self.config.max_samples;
            hook_samples.drain(0..to_remove);
        }

        // Update counter
        let counters = self.counters.read();
        if let Some(counter) = counters.get(hook_name) {
            counter.fetch_add(1, Ordering::Relaxed);
        } else {
            drop(counters);
            let mut counters = self.counters.write();
            counters
                .entry(hook_name.to_string())
                .or_insert_with(|| AtomicU64::new(0))
                .fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Get performance metrics for a hook
    pub fn get_metrics(&self, hook_name: &str) -> Option<PerformanceMetrics> {
        let samples = self.samples.read();
        let hook_samples = samples.get(hook_name)?;

        if hook_samples.is_empty() {
            return None;
        }

        let execution_count = self
            .counters
            .read()
            .get(hook_name)
            .map(|c| c.load(Ordering::Relaxed))
            .unwrap_or(0);

        let total_duration: Duration = hook_samples.iter().map(|s| s.duration).sum();

        let min_duration = hook_samples
            .iter()
            .map(|s| s.duration)
            .min()
            .unwrap_or(Duration::ZERO);

        let max_duration = hook_samples
            .iter()
            .map(|s| s.duration)
            .max()
            .unwrap_or(Duration::ZERO);

        let avg_duration = if !hook_samples.is_empty() {
            total_duration / hook_samples.len() as u32
        } else {
            Duration::ZERO
        };

        let (p50, p95, p99) = if self.config.calculate_percentiles {
            let mut durations: Vec<_> = hook_samples.iter().map(|s| s.duration).collect();
            durations.sort();

            let p50_idx = durations.len() / 2;
            let p95_idx = (durations.len() as f64 * 0.95) as usize;
            let p99_idx = (durations.len() as f64 * 0.99) as usize;

            (
                durations.get(p50_idx).copied().unwrap_or(Duration::ZERO),
                durations.get(p95_idx).copied().unwrap_or(Duration::ZERO),
                durations.get(p99_idx).copied().unwrap_or(Duration::ZERO),
            )
        } else {
            (Duration::ZERO, Duration::ZERO, Duration::ZERO)
        };

        let last_execution = hook_samples.last().map(|s| s.timestamp);

        Some(PerformanceMetrics {
            execution_count,
            total_duration,
            min_duration,
            max_duration,
            avg_duration,
            p50_duration: p50,
            p95_duration: p95,
            p99_duration: p99,
            last_execution,
        })
    }

    /// Get all hook metrics
    pub fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        let samples = self.samples.read();
        samples
            .keys()
            .filter_map(|name| {
                self.get_metrics(name)
                    .map(|metrics| (name.clone(), metrics))
            })
            .collect()
    }

    /// Clear metrics for a specific hook
    pub fn clear_metrics(&self, hook_name: &str) {
        self.samples.write().remove(hook_name);
        self.counters.write().remove(hook_name);
    }

    /// Clear all metrics
    pub fn clear_all_metrics(&self) {
        self.samples.write().clear();
        self.counters.write().clear();
    }

    /// Check if a hook is performing within threshold
    pub fn is_within_threshold(&self, hook_name: &str, threshold: Duration) -> bool {
        if let Some(metrics) = self.get_metrics(hook_name) {
            metrics.avg_duration <= threshold
        } else {
            true // No data means we assume it's fine
        }
    }

    /// Get hooks exceeding performance threshold
    pub fn get_slow_hooks(&self, threshold: Duration) -> Vec<(String, PerformanceMetrics)> {
        self.get_all_metrics()
            .into_iter()
            .filter(|(_, metrics)| metrics.avg_duration > threshold)
            .collect()
    }
}

impl Clone for PerformanceMonitor {
    fn clone(&self) -> Self {
        Self {
            samples: self.samples.clone(),
            counters: self.counters.clone(),
            config: self.config.clone(),
        }
    }
}

/// Timer for tracking execution duration
pub struct ExecutionTimer {
    hook_name: String,
    start_time: Instant,
    monitor: PerformanceMonitor,
    completed: bool,
}

impl ExecutionTimer {
    /// Complete the timing and record the duration
    pub fn complete(mut self) -> Duration {
        let duration = self.start_time.elapsed();
        self.monitor.record_execution(&self.hook_name, duration);
        self.completed = true;
        duration
    }
}

impl Drop for ExecutionTimer {
    fn drop(&mut self) {
        // Auto-complete if not explicitly completed
        if !self.completed {
            let duration = self.start_time.elapsed();
            self.monitor.record_execution(&self.hook_name, duration);
        }
    }
}

/// Performance report for diagnostics
#[derive(Debug)]
pub struct PerformanceReport {
    pub total_hooks: usize,
    pub total_executions: u64,
    pub avg_duration: Duration,
    pub slowest_hook: Option<(String, Duration)>,
    pub most_frequent_hook: Option<(String, u64)>,
    pub slow_hooks: Vec<(String, PerformanceMetrics)>,
}

impl PerformanceMonitor {
    /// Generate a performance report
    pub fn generate_report(&self, slow_threshold: Duration) -> PerformanceReport {
        let all_metrics = self.get_all_metrics();

        let total_hooks = all_metrics.len();
        let total_executions: u64 = all_metrics.values().map(|m| m.execution_count).sum();

        let total_duration: Duration = all_metrics.values().map(|m| m.total_duration).sum();

        let avg_duration = if total_executions > 0 {
            total_duration / total_executions as u32
        } else {
            Duration::ZERO
        };

        let slowest_hook = all_metrics
            .iter()
            .max_by_key(|(_, m)| m.max_duration)
            .map(|(name, m)| (name.clone(), m.max_duration));

        let most_frequent_hook = all_metrics
            .iter()
            .max_by_key(|(_, m)| m.execution_count)
            .map(|(name, m)| (name.clone(), m.execution_count));

        let slow_hooks = self.get_slow_hooks(slow_threshold);

        PerformanceReport {
            total_hooks,
            total_executions,
            avg_duration,
            slowest_hook,
            most_frequent_hook,
            slow_hooks,
        }
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use std::thread;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_performance_monitoring() {
        let monitor = PerformanceMonitor::new();

        // Record some executions
        monitor.record_execution("hook1", Duration::from_millis(10));
        monitor.record_execution("hook1", Duration::from_millis(20));
        monitor.record_execution("hook1", Duration::from_millis(30));

        monitor.record_execution("hook2", Duration::from_millis(5));
        monitor.record_execution("hook2", Duration::from_millis(15));

        // Get metrics
        let metrics1 = monitor.get_metrics("hook1").unwrap();
        assert_eq!(metrics1.execution_count, 3);
        assert_eq!(metrics1.min_duration, Duration::from_millis(10));
        assert_eq!(metrics1.max_duration, Duration::from_millis(30));
        assert_eq!(metrics1.avg_duration, Duration::from_millis(20));

        let metrics2 = monitor.get_metrics("hook2").unwrap();
        assert_eq!(metrics2.execution_count, 2);
        assert_eq!(metrics2.avg_duration, Duration::from_millis(10));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_execution_timer() {
        let monitor = PerformanceMonitor::new();

        {
            let timer = monitor.start_execution("timed_hook");
            thread::sleep(Duration::from_millis(50));
            let duration = timer.complete();
            assert!(duration >= Duration::from_millis(50));
        }

        let metrics = monitor.get_metrics("timed_hook").unwrap();
        assert_eq!(metrics.execution_count, 1);
        assert!(metrics.max_duration >= Duration::from_millis(50));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_slow_hook_detection() {
        let monitor = PerformanceMonitor::new();

        monitor.record_execution("fast_hook", Duration::from_millis(5));
        monitor.record_execution("slow_hook", Duration::from_millis(150));
        monitor.record_execution("medium_hook", Duration::from_millis(50));

        let slow_hooks = monitor.get_slow_hooks(Duration::from_millis(100));
        assert_eq!(slow_hooks.len(), 1);
        assert_eq!(slow_hooks[0].0, "slow_hook");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_performance_report() {
        let monitor = PerformanceMonitor::new();

        // Record varied executions
        for _ in 0..10 {
            monitor.record_execution("frequent_hook", Duration::from_millis(10));
        }

        monitor.record_execution("slow_hook", Duration::from_millis(200));
        monitor.record_execution("fast_hook", Duration::from_millis(1));

        let report = monitor.generate_report(Duration::from_millis(100));

        assert_eq!(report.total_hooks, 3);
        assert_eq!(report.total_executions, 12);
        assert_eq!(report.most_frequent_hook.unwrap().0, "frequent_hook");
        assert_eq!(report.slowest_hook.unwrap().0, "slow_hook");
        assert_eq!(report.slow_hooks.len(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_metrics_cleanup() {
        let config = PerformanceConfig {
            max_samples: 3,
            retention_period: Duration::from_secs(1),
            calculate_percentiles: true,
        };

        let monitor = PerformanceMonitor::with_config(config);

        // Add more samples than max
        for i in 0..5 {
            monitor.record_execution("test_hook", Duration::from_millis(i * 10));
        }

        let samples = monitor.samples.read();
        let hook_samples = samples.get("test_hook").unwrap();
        assert_eq!(hook_samples.len(), 3); // Should be limited to max_samples
    }
}
