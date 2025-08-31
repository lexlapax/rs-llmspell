//! Telemetry and metrics collection

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Metrics collected during test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Numeric metrics (counters, gauges)
    pub values: HashMap<String, f64>,
    /// Duration metrics (stored as milliseconds for serialization)
    #[serde(with = "duration_map_serde")]
    pub durations: HashMap<String, Duration>,
    /// Timestamp when metrics were collected (as milliseconds since start)
    pub timestamp_ms: u64,
}

/// Serialization helper for Duration HashMap
mod duration_map_serde {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(map: &HashMap<String, Duration>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let millis_map: HashMap<String, u64> = map
            .iter()
            .map(|(k, v)| (k.clone(), v.as_millis() as u64))
            .collect();
        millis_map.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<HashMap<String, Duration>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let millis_map: HashMap<String, u64> = HashMap::deserialize(deserializer)?;
        Ok(millis_map
            .into_iter()
            .map(|(k, v)| (k, Duration::from_millis(v)))
            .collect())
    }
}

impl Metrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            durations: HashMap::new(),
            timestamp_ms: 0, // Will be set when metrics are collected
        }
    }

    /// Record a numeric metric
    pub fn record_value(&mut self, name: impl Into<String>, value: f64) {
        self.values.insert(name.into(), value);
    }

    /// Record a duration metric
    pub fn record_duration(&mut self, name: impl Into<String>, duration: Duration) {
        self.durations.insert(name.into(), duration);
    }

    /// Get a numeric metric
    pub fn get_value(&self, name: &str) -> Option<f64> {
        self.values.get(name).copied()
    }

    /// Get a duration metric
    pub fn get_duration(&self, name: &str) -> Option<Duration> {
        self.durations.get(name).copied()
    }

    /// Merge another metrics instance into this one
    pub fn merge(&mut self, other: &Metrics) {
        for (k, v) in &other.values {
            self.values.insert(k.clone(), *v);
        }
        for (k, v) in &other.durations {
            self.durations.insert(k.clone(), *v);
        }
    }
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Telemetry collector for test execution
#[derive(Clone)]
pub struct TelemetryCollector {
    metrics: Arc<Mutex<Metrics>>,
    start_time: Instant,
}

impl TelemetryCollector {
    /// Create a new telemetry collector
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(Metrics::new())),
            start_time: Instant::now(),
        }
    }

    /// Record a numeric metric
    pub fn record_metric(&self, name: impl Into<String>, value: impl Into<f64>) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_value(name, value.into());
        }
    }

    /// Record a duration metric
    pub fn record_duration(&self, name: impl Into<String>, duration: Duration) {
        if let Ok(mut metrics) = self.metrics.lock() {
            metrics.record_duration(name, duration);
        }
    }

    /// Increment a counter metric
    pub fn increment(&self, name: impl Into<String>) {
        self.add(name, 1.0);
    }

    /// Add to a counter metric
    pub fn add(&self, name: impl Into<String>, value: f64) {
        let name = name.into();
        if let Ok(mut metrics) = self.metrics.lock() {
            let current = metrics.values.get(&name).copied().unwrap_or(0.0);
            metrics.values.insert(name, current + value);
        }
    }

    /// Record the start of an operation
    pub fn start_operation(&self, name: impl Into<String>) -> OperationTimer {
        OperationTimer {
            collector: self.clone(),
            name: name.into(),
            start: Instant::now(),
        }
    }

    /// Get a snapshot of current metrics
    pub fn snapshot(&self) -> Metrics {
        self.metrics.lock().unwrap().clone()
    }

    /// Get elapsed time since collector creation
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        if let Ok(mut metrics) = self.metrics.lock() {
            *metrics = Metrics::new();
        }
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

/// Timer for measuring operation duration
pub struct OperationTimer {
    collector: TelemetryCollector,
    name: String,
    start: Instant,
}

impl OperationTimer {
    /// Complete the operation and record its duration
    pub fn complete(self) {
        let duration = self.start.elapsed();
        self.collector.record_duration(&self.name, duration);
        self.collector
            .record_metric(format!("{}_ms", self.name), duration.as_millis() as f64);
    }
}

impl Drop for OperationTimer {
    fn drop(&mut self) {
        // Auto-complete if not explicitly completed
        let duration = self.start.elapsed();
        self.collector.record_duration(&self.name, duration);
    }
}

/// Helper to measure async operations
pub async fn measure<F, T>(collector: &TelemetryCollector, name: impl Into<String>, f: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let timer = collector.start_operation(name);
    let result = f.await;
    timer.complete();
    result
}
