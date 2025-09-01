//! Metrics collection for sidecar observability
//!
//! Applying adaptive performance patterns from Phase 9.3.3

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Metrics data for a single operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    /// Operation name
    pub operation: String,
    /// Protocol type
    pub protocol: String,
    /// Channel type
    pub channel: String,
    /// Duration of the operation
    pub duration: Duration,
    /// Success or failure
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Timestamp as milliseconds since epoch
    pub timestamp_ms: u64,
    /// Additional tags
    pub tags: HashMap<String, String>,
}

/// Aggregated metrics for reporting
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Total requests
    pub total_requests: u64,
    /// Successful requests
    pub successful_requests: u64,
    /// Failed requests
    pub failed_requests: u64,
    /// Average latency
    pub avg_latency_ms: f64,
    /// P50 latency
    pub p50_latency_ms: f64,
    /// P95 latency
    pub p95_latency_ms: f64,
    /// P99 latency
    pub p99_latency_ms: f64,
    /// Requests per protocol
    pub requests_by_protocol: HashMap<String, u64>,
    /// Requests per channel
    pub requests_by_channel: HashMap<String, u64>,
}

/// Sidecar-specific metrics
#[derive(Debug, Clone, Default)]
pub struct SidecarMetrics {
    /// Protocol negotiation time
    pub negotiation_time_ms: f64,
    /// Circuit breaker trips
    pub circuit_breaker_trips: u64,
    /// Message adaptation time
    pub adaptation_time_ms: f64,
    /// Active connections
    pub active_connections: usize,
    /// Service discovery queries
    pub discovery_queries: u64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Metrics collector trait - Layer 1: Trait abstraction
#[async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Record a metric for an operation
    async fn record(&self, metric: MetricData);

    /// Record protocol negotiation time
    async fn record_negotiation(&self, duration: Duration);

    /// Record message adaptation time
    async fn record_adaptation(&self, duration: Duration);

    /// Record circuit breaker trip
    async fn record_circuit_breaker_trip(&self);

    /// Get aggregated metrics
    async fn get_aggregated(&self) -> AggregatedMetrics;

    /// Get sidecar-specific metrics
    async fn get_sidecar_metrics(&self) -> SidecarMetrics;

    /// Reset all metrics
    async fn reset(&self);
}

/// Default metrics collector implementation - Layer 3: Concrete implementation
pub struct DefaultMetricsCollector {
    metrics: Arc<RwLock<Vec<MetricData>>>,
    sidecar_metrics: Arc<RwLock<SidecarMetrics>>,
    /// Adaptive threshold for metric collection (from Phase 9.3.3)
    collection_threshold: Duration,
}

impl DefaultMetricsCollector {
    /// Create a new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            sidecar_metrics: Arc::new(RwLock::new(SidecarMetrics::default())),
            collection_threshold: Duration::from_millis(100), // Adaptive threshold
        }
    }

    /// Create with custom collection threshold
    #[must_use]
    pub fn with_threshold(collection_threshold: Duration) -> Self {
        Self {
            metrics: Arc::new(RwLock::new(Vec::new())),
            sidecar_metrics: Arc::new(RwLock::new(SidecarMetrics::default())),
            collection_threshold,
        }
    }

    /// Calculate percentile from sorted durations
    fn calculate_percentile(durations: &[f64], percentile: f64) -> f64 {
        if durations.is_empty() {
            return 0.0;
        }
        let index = ((percentile / 100.0) * durations.len() as f64) as usize;
        let index = index.min(durations.len() - 1);
        durations[index]
    }
}

impl Default for DefaultMetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MetricsCollector for DefaultMetricsCollector {
    async fn record(&self, metric: MetricData) {
        // Only record if above threshold (adaptive performance)
        if metric.duration >= self.collection_threshold {
            let mut metrics = self.metrics.write().await;

            // Limit metrics storage (prevent memory leak)
            if metrics.len() > 10000 {
                metrics.drain(0..5000); // Keep latest 5000
            }

            metrics.push(metric);
        }
    }

    async fn record_negotiation(&self, duration: Duration) {
        let mut sidecar = self.sidecar_metrics.write().await;
        // Exponential moving average
        let new_time = duration.as_secs_f64() * 1000.0;
        sidecar.negotiation_time_ms = sidecar.negotiation_time_ms * 0.9 + new_time * 0.1;
    }

    async fn record_adaptation(&self, duration: Duration) {
        let mut sidecar = self.sidecar_metrics.write().await;
        // Exponential moving average
        let new_time = duration.as_secs_f64() * 1000.0;
        sidecar.adaptation_time_ms = sidecar.adaptation_time_ms * 0.9 + new_time * 0.1;
    }

    async fn record_circuit_breaker_trip(&self) {
        let mut sidecar = self.sidecar_metrics.write().await;
        sidecar.circuit_breaker_trips += 1;
    }

    async fn get_aggregated(&self) -> AggregatedMetrics {
        let metrics = self.metrics.read().await;

        let total = metrics.len() as u64;
        let successful = metrics.iter().filter(|m| m.success).count() as u64;
        let failed = total - successful;

        // Calculate latencies
        let mut durations: Vec<f64> = metrics
            .iter()
            .map(|m| m.duration.as_secs_f64() * 1000.0)
            .collect();
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let avg_latency = if !durations.is_empty() {
            durations.iter().sum::<f64>() / durations.len() as f64
        } else {
            0.0
        };

        // Count by protocol and channel
        let mut by_protocol = HashMap::new();
        let mut by_channel = HashMap::new();

        for metric in metrics.iter() {
            *by_protocol.entry(metric.protocol.clone()).or_insert(0) += 1;
            *by_channel.entry(metric.channel.clone()).or_insert(0) += 1;
        }

        AggregatedMetrics {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            avg_latency_ms: avg_latency,
            p50_latency_ms: Self::calculate_percentile(&durations, 50.0),
            p95_latency_ms: Self::calculate_percentile(&durations, 95.0),
            p99_latency_ms: Self::calculate_percentile(&durations, 99.0),
            requests_by_protocol: by_protocol,
            requests_by_channel: by_channel,
        }
    }

    async fn get_sidecar_metrics(&self) -> SidecarMetrics {
        self.sidecar_metrics.read().await.clone()
    }

    async fn reset(&self) {
        let mut metrics = self.metrics.write().await;
        metrics.clear();

        let mut sidecar = self.sidecar_metrics.write().await;
        *sidecar = SidecarMetrics::default();
    }
}

/// Null metrics collector for testing - Following Phase 9.3 pattern
pub struct NullMetricsCollector;

#[async_trait]
impl MetricsCollector for NullMetricsCollector {
    async fn record(&self, _metric: MetricData) {}
    async fn record_negotiation(&self, _duration: Duration) {}
    async fn record_adaptation(&self, _duration: Duration) {}
    async fn record_circuit_breaker_trip(&self) {}

    async fn get_aggregated(&self) -> AggregatedMetrics {
        AggregatedMetrics::default()
    }

    async fn get_sidecar_metrics(&self) -> SidecarMetrics {
        SidecarMetrics::default()
    }

    async fn reset(&self) {}
}

/// Helper to time an operation and record metrics
pub struct MetricTimer {
    start: Instant,
    operation: String,
    protocol: String,
    channel: String,
}

impl MetricTimer {
    /// Start timing an operation
    #[must_use]
    pub fn start(
        operation: impl Into<String>,
        protocol: impl Into<String>,
        channel: impl Into<String>,
    ) -> Self {
        Self {
            start: Instant::now(),
            operation: operation.into(),
            protocol: protocol.into(),
            channel: channel.into(),
        }
    }

    /// Complete the timing and create metric data
    #[must_use]
    pub fn complete(self, success: bool, error: Option<String>) -> MetricData {
        MetricData {
            operation: self.operation,
            protocol: self.protocol,
            channel: self.channel,
            duration: self.start.elapsed(),
            success,
            error,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            tags: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collection() {
        let collector = DefaultMetricsCollector::new();

        // Record some metrics
        for i in 0..5 {
            let metric = MetricData {
                operation: "test".to_string(),
                protocol: "LRP".to_string(),
                channel: "shell".to_string(),
                duration: Duration::from_millis(100 + i * 10),
                success: i != 2, // One failure
                error: if i == 2 {
                    Some("test error".to_string())
                } else {
                    None
                },
                timestamp_ms: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as u64,
                tags: HashMap::new(),
            };
            collector.record(metric).await;
        }

        // Check aggregated metrics
        let aggregated = collector.get_aggregated().await;
        assert_eq!(aggregated.total_requests, 5);
        assert_eq!(aggregated.successful_requests, 4);
        assert_eq!(aggregated.failed_requests, 1);
        assert!(aggregated.avg_latency_ms > 100.0);
    }

    #[tokio::test]
    async fn test_sidecar_metrics() {
        let collector = DefaultMetricsCollector::new();

        // Record sidecar-specific metrics
        collector
            .record_negotiation(Duration::from_millis(10))
            .await;
        collector.record_adaptation(Duration::from_millis(5)).await;
        collector.record_circuit_breaker_trip().await;

        let metrics = collector.get_sidecar_metrics().await;
        assert!(metrics.negotiation_time_ms > 0.0);
        assert!(metrics.adaptation_time_ms > 0.0);
        assert_eq!(metrics.circuit_breaker_trips, 1);
    }

    #[tokio::test]
    async fn test_metric_timer() {
        let timer = MetricTimer::start("test_op", "LRP", "shell");
        tokio::time::sleep(Duration::from_millis(10)).await;
        let metric = timer.complete(true, None);

        assert_eq!(metric.operation, "test_op");
        assert_eq!(metric.protocol, "LRP");
        assert!(metric.duration >= Duration::from_millis(10));
        assert!(metric.success);
    }
}
