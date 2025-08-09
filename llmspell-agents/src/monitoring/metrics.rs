//! ABOUTME: Agent metrics collection and reporting
//! ABOUTME: Provides counters, gauges, histograms for monitoring agent behavior

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Type of metric
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter - monotonically increasing value
    Counter,
    /// Gauge - value that can go up and down
    Gauge,
    /// Histogram - distribution of values
    Histogram,
    /// Summary - statistical summary of values
    Summary,
}

/// Metric value
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter value
    Counter(u64),
    /// Gauge value
    Gauge(f64),
    /// Histogram with buckets
    Histogram {
        sum: f64,
        count: u64,
        buckets: Vec<(f64, u64)>,
    },
    /// Summary with quantiles
    Summary {
        sum: f64,
        count: u64,
        quantiles: Vec<(f64, f64)>,
    },
}

/// Label for metrics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricLabel {
    /// Label name
    pub name: String,
    /// Label value
    pub value: String,
}

/// Counter metric
#[derive(Debug)]
pub struct Counter {
    value: Arc<AtomicU64>,
}

impl Counter {
    /// Create a new counter
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Increment the counter
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Increment by a specific amount
    pub fn inc_by(&self, amount: u64) {
        self.value.fetch_add(amount, Ordering::Relaxed);
    }

    /// Get the current value
    #[must_use]
    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// Reset the counter
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}

/// Gauge metric
#[derive(Debug)]
pub struct Gauge {
    value: Arc<RwLock<f64>>,
}

impl Gauge {
    /// Create a new gauge
    #[must_use]
    pub fn new() -> Self {
        Self {
            value: Arc::new(RwLock::new(0.0)),
        }
    }

    /// Set the gauge value
    pub fn set(&self, value: f64) {
        if let Ok(mut v) = self.value.write() {
            *v = value;
        }
    }

    /// Increment the gauge
    pub fn inc(&self) {
        if let Ok(mut v) = self.value.write() {
            *v += 1.0;
        }
    }

    /// Decrement the gauge
    pub fn dec(&self) {
        if let Ok(mut v) = self.value.write() {
            *v -= 1.0;
        }
    }

    /// Add to the gauge
    pub fn add(&self, delta: f64) {
        if let Ok(mut v) = self.value.write() {
            *v += delta;
        }
    }

    /// Subtract from the gauge
    pub fn sub(&self, delta: f64) {
        if let Ok(mut v) = self.value.write() {
            *v -= delta;
        }
    }

    /// Get the current value
    #[must_use]
    pub fn get(&self) -> f64 {
        self.value.read().map(|v| *v).unwrap_or(0.0)
    }
}

impl Default for Gauge {
    fn default() -> Self {
        Self::new()
    }
}

/// Histogram metric
#[derive(Debug)]
pub struct Histogram {
    buckets: Vec<f64>,
    bucket_counts: Vec<Arc<AtomicU64>>,
    sum: Arc<RwLock<f64>>,
    count: Arc<AtomicU64>,
}

impl Histogram {
    /// Create a new histogram with given buckets
    #[must_use]
    pub fn new(buckets: Vec<f64>) -> Self {
        let bucket_counts = buckets
            .iter()
            .map(|_| Arc::new(AtomicU64::new(0)))
            .collect();

        Self {
            buckets,
            bucket_counts,
            sum: Arc::new(RwLock::new(0.0)),
            count: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Create a histogram with default buckets
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(vec![
            0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
        ])
    }

    /// Observe a value
    pub fn observe(&self, value: f64) {
        // Update sum
        if let Ok(mut sum) = self.sum.write() {
            *sum += value;
        }

        // Update count
        self.count.fetch_add(1, Ordering::Relaxed);

        // Update buckets
        for (i, &bucket) in self.buckets.iter().enumerate() {
            if value <= bucket {
                self.bucket_counts[i].fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Get the current histogram data
    #[must_use]
    pub fn get(&self) -> MetricValue {
        let sum = self.sum.read().map(|v| *v).unwrap_or(0.0);
        let count = self.count.load(Ordering::Relaxed);

        let buckets: Vec<(f64, u64)> = self
            .buckets
            .iter()
            .zip(&self.bucket_counts)
            .map(|(&bucket, count)| (bucket, count.load(Ordering::Relaxed)))
            .collect();

        MetricValue::Histogram {
            sum,
            count,
            buckets,
        }
    }
}

/// Timer for measuring durations
pub struct Timer {
    start: Instant,
    histogram: Arc<Histogram>,
}

impl Timer {
    /// Create a new timer
    #[must_use]
    pub fn new(histogram: Arc<Histogram>) -> Self {
        Self {
            start: Instant::now(),
            histogram,
        }
    }

    /// Stop the timer and record the duration
    pub fn stop(self) {
        let duration = self.start.elapsed();
        self.histogram.observe(duration.as_secs_f64());
    }
}

/// Agent-specific metrics
#[derive(Debug)]
pub struct AgentMetrics {
    /// Agent ID
    pub agent_id: String,
    /// Number of requests processed
    pub requests_total: Counter,
    /// Number of failed requests
    pub requests_failed: Counter,
    /// Current active requests
    pub requests_active: Gauge,
    /// Request duration histogram
    pub request_duration: Histogram,
    /// Tool invocations
    pub tool_invocations: Counter,
    /// Memory usage
    pub memory_bytes: Gauge,
    /// CPU usage percentage
    pub cpu_percent: Gauge,
    /// Custom metrics
    pub custom: HashMap<String, Box<dyn MetricAccess>>,
}

impl AgentMetrics {
    /// Create new agent metrics
    #[must_use]
    pub fn new(agent_id: String) -> Self {
        Self {
            agent_id,
            requests_total: Counter::new(),
            requests_failed: Counter::new(),
            requests_active: Gauge::new(),
            request_duration: Histogram::with_defaults(),
            tool_invocations: Counter::new(),
            memory_bytes: Gauge::new(),
            cpu_percent: Gauge::new(),
            custom: HashMap::new(),
        }
    }

    /// Start timing a request
    #[must_use]
    pub fn start_request(&self) -> Timer {
        self.requests_active.inc();
        Timer::new(Arc::new(Histogram::with_defaults()))
    }

    /// Complete a request
    pub fn complete_request(&self, timer: Timer, success: bool) {
        timer.stop();
        self.requests_active.dec();
        self.requests_total.inc();

        if !success {
            self.requests_failed.inc();
        }
    }

    /// Record a tool invocation
    pub fn record_tool_invocation(&self) {
        self.tool_invocations.inc();
    }

    /// Update resource usage
    pub fn update_resources(&self, memory_bytes: f64, cpu_percent: f64) {
        self.memory_bytes.set(memory_bytes);
        self.cpu_percent.set(cpu_percent);
    }

    /// Add a custom metric
    pub fn add_custom_metric(&mut self, name: String, metric: Box<dyn MetricAccess>) {
        self.custom.insert(name, metric);
    }
}

/// Trait for accessing metric values
pub trait MetricAccess: Send + Sync + std::fmt::Debug {
    /// Get the metric type
    fn metric_type(&self) -> MetricType;

    /// Get the current value
    fn value(&self) -> MetricValue;

    /// Reset the metric
    fn reset(&self);
}

impl MetricAccess for Counter {
    fn metric_type(&self) -> MetricType {
        MetricType::Counter
    }

    fn value(&self) -> MetricValue {
        MetricValue::Counter(self.get())
    }

    fn reset(&self) {
        self.reset();
    }
}

impl MetricAccess for Gauge {
    fn metric_type(&self) -> MetricType {
        MetricType::Gauge
    }

    fn value(&self) -> MetricValue {
        MetricValue::Gauge(self.get())
    }

    fn reset(&self) {
        self.set(0.0);
    }
}

impl MetricAccess for Histogram {
    fn metric_type(&self) -> MetricType {
        MetricType::Histogram
    }

    fn value(&self) -> MetricValue {
        self.get()
    }

    fn reset(&self) {
        // Reset all buckets
        for bucket_count in &self.bucket_counts {
            bucket_count.store(0, Ordering::Relaxed);
        }
        self.count.store(0, Ordering::Relaxed);
        if let Ok(mut sum) = self.sum.write() {
            *sum = 0.0;
        }
    }
}

/// Metric registry for managing all metrics
#[derive(Debug)]
pub struct MetricRegistry {
    /// Registered metrics
    metrics: Arc<RwLock<HashMap<String, Arc<dyn MetricAccess>>>>,
    /// Agent metrics
    agent_metrics: Arc<RwLock<HashMap<String, Arc<AgentMetrics>>>>,
}

impl MetricRegistry {
    /// Create a new metric registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            agent_metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a metric
    ///
    /// # Errors
    ///
    /// Currently never returns an error, but the Result type is provided for future
    /// extensibility (e.g., validation of metric names or handling registration conflicts).
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    pub fn register(&self, name: String, metric: Arc<dyn MetricAccess>) -> Result<()> {
        let mut metrics = self.metrics.write().unwrap();
        metrics.insert(name, metric);
        Ok(())
    }

    /// Get a metric
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn get(&self, name: &str) -> Option<Arc<dyn MetricAccess>> {
        self.metrics.read().unwrap().get(name).cloned()
    }

    /// Get or create agent metrics
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn get_agent_metrics(&self, agent_id: &str) -> Arc<AgentMetrics> {
        let mut agent_metrics = self.agent_metrics.write().unwrap();
        agent_metrics
            .entry(agent_id.to_string())
            .or_insert_with(|| Arc::new(AgentMetrics::new(agent_id.to_string())))
            .clone()
    }

    /// Collect all metrics
    #[must_use]
    pub fn collect(&self) -> HashMap<String, MetricValue> {
        let mut result = HashMap::new();

        // Collect registered metrics
        if let Ok(metrics) = self.metrics.read() {
            for (name, metric) in metrics.iter() {
                result.insert(name.clone(), metric.value());
            }
        }

        // Collect agent metrics
        if let Ok(agents) = self.agent_metrics.read() {
            for (agent_id, metrics) in agents.iter() {
                let prefix = format!("agent.{agent_id}");
                result.insert(
                    format!("{prefix}.requests_total"),
                    MetricValue::Counter(metrics.requests_total.get()),
                );
                result.insert(
                    format!("{prefix}.requests_failed"),
                    MetricValue::Counter(metrics.requests_failed.get()),
                );
                result.insert(
                    format!("{prefix}.requests_active"),
                    MetricValue::Gauge(metrics.requests_active.get()),
                );
                result.insert(
                    format!("{prefix}.request_duration"),
                    metrics.request_duration.get(),
                );
                result.insert(
                    format!("{prefix}.tool_invocations"),
                    MetricValue::Counter(metrics.tool_invocations.get()),
                );
                result.insert(
                    format!("{prefix}.memory_bytes"),
                    MetricValue::Gauge(metrics.memory_bytes.get()),
                );
                result.insert(
                    format!("{prefix}.cpu_percent"),
                    MetricValue::Gauge(metrics.cpu_percent.get()),
                );

                // Add custom metrics
                for (custom_name, custom_metric) in &metrics.custom {
                    result.insert(format!("{prefix}.{custom_name}"), custom_metric.value());
                }
            }
        }

        result
    }

    /// Reset all metrics
    pub fn reset_all(&self) {
        if let Ok(metrics) = self.metrics.write() {
            for metric in metrics.values() {
                metric.reset();
            }
        }
    }
}

impl Default for MetricRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_counter() {
        let counter = Counter::new();
        assert_eq!(counter.get(), 0);

        counter.inc();
        assert_eq!(counter.get(), 1);

        counter.inc_by(5);
        assert_eq!(counter.get(), 6);

        counter.reset();
        assert_eq!(counter.get(), 0);
    }
    #[test]
    #[allow(clippy::float_cmp)] // Test assertions on float values
    fn test_gauge() {
        let gauge = Gauge::new();
        assert_eq!(gauge.get(), 0.0);

        gauge.set(42.5);
        assert_eq!(gauge.get(), 42.5);

        gauge.inc();
        assert_eq!(gauge.get(), 43.5);

        gauge.add(10.0);
        assert_eq!(gauge.get(), 53.5);

        gauge.sub(3.5);
        assert_eq!(gauge.get(), 50.0);
    }
    #[test]
    fn test_histogram() {
        let hist = Histogram::new(vec![0.1, 0.5, 1.0, 5.0]);

        hist.observe(0.05);
        hist.observe(0.3);
        hist.observe(0.7);
        hist.observe(2.0);

        match hist.get() {
            MetricValue::Histogram {
                sum,
                count,
                buckets,
            } => {
                assert_eq!(count, 4);
                assert!((sum - 3.05).abs() < 0.001);
                assert_eq!(buckets[0].1, 1); // 0.05 <= 0.1
                assert_eq!(buckets[1].1, 2); // 0.05, 0.3 <= 0.5
                assert_eq!(buckets[2].1, 3); // 0.05, 0.3, 0.7 <= 1.0
                assert_eq!(buckets[3].1, 4); // all <= 5.0
            }
            _ => panic!("Expected histogram value"),
        }
    }
    #[test]
    #[allow(clippy::float_cmp)] // Test assertions on float values
    fn test_agent_metrics() {
        let metrics = AgentMetrics::new("test-agent".to_string());

        // Test request tracking
        let timer = metrics.start_request();
        assert_eq!(metrics.requests_active.get(), 1.0);

        metrics.complete_request(timer, true);
        assert_eq!(metrics.requests_active.get(), 0.0);
        assert_eq!(metrics.requests_total.get(), 1);
        assert_eq!(metrics.requests_failed.get(), 0);

        // Test failed request
        let timer = metrics.start_request();
        metrics.complete_request(timer, false);
        assert_eq!(metrics.requests_total.get(), 2);
        assert_eq!(metrics.requests_failed.get(), 1);

        // Test tool invocations
        metrics.record_tool_invocation();
        metrics.record_tool_invocation();
        assert_eq!(metrics.tool_invocations.get(), 2);

        // Test resource updates
        metrics.update_resources(1024.0 * 1024.0, 25.5);
        assert_eq!(metrics.memory_bytes.get(), 1024.0 * 1024.0);
        assert_eq!(metrics.cpu_percent.get(), 25.5);
    }
    #[test]
    fn test_metric_registry() {
        let registry = MetricRegistry::new();

        // Register a counter
        let counter = Arc::new(Counter::new());
        counter.inc_by(10);
        registry
            .register("test.counter".to_string(), counter)
            .unwrap();

        // Get agent metrics
        let agent_metrics = registry.get_agent_metrics("agent-1");
        agent_metrics.requests_total.inc_by(5);
        agent_metrics.memory_bytes.set(2048.0);

        // Collect all metrics
        let collected = registry.collect();
        assert_eq!(
            collected.get("test.counter"),
            Some(&MetricValue::Counter(10))
        );
        assert_eq!(
            collected.get("agent.agent-1.requests_total"),
            Some(&MetricValue::Counter(5))
        );
        assert_eq!(
            collected.get("agent.agent-1.memory_bytes"),
            Some(&MetricValue::Gauge(2048.0))
        );
    }
}
