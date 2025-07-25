// ABOUTME: Event metrics collection and reporting
// ABOUTME: Tracks event throughput, latency, and system health metrics

use crate::universal_event::{Language, UniversalEvent};
use parking_lot::RwLock;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::time::interval;

/// Real-time event metrics with windowed analytics
#[derive(Debug, Default)]
pub struct EventMetrics {
    /// Total events processed
    pub total_events: u64,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
    /// Events by language
    pub events_by_language: HashMap<Language, u64>,
    /// Average processing time
    pub avg_processing_time: Duration,
    /// Peak events per second
    pub peak_events_per_second: f64,
    /// Current events per second
    pub current_events_per_second: f64,
    /// Real-time analytics
    pub real_time_analytics: RealTimeAnalytics,
}

/// Real-time analytics data
#[derive(Debug)]
pub struct RealTimeAnalytics {
    /// Events per second over last 60 seconds (sliding window)
    pub eps_history: VecDeque<f64>,
    /// Memory usage over time
    pub memory_usage_bytes: u64,
    /// Processing latency percentiles
    pub latency_percentiles: LatencyPercentiles,
    /// Error rate tracking
    pub error_rate: f64,
    /// Throughput trend (increasing/decreasing)
    pub throughput_trend: ThroughputTrend,
    /// Last update timestamp
    pub last_updated: SystemTime,
}

/// Latency percentile metrics
#[derive(Debug, Default)]
pub struct LatencyPercentiles {
    pub p50: Duration,
    pub p90: Duration,
    pub p95: Duration,
    pub p99: Duration,
}

/// Throughput trend indicator
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ThroughputTrend {
    Increasing,
    Stable,
    Decreasing,
    Unknown,
}

impl Default for ThroughputTrend {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Default for RealTimeAnalytics {
    fn default() -> Self {
        Self {
            eps_history: VecDeque::new(),
            memory_usage_bytes: 0,
            latency_percentiles: LatencyPercentiles::default(),
            error_rate: 0.0,
            throughput_trend: ThroughputTrend::Unknown,
            last_updated: SystemTime::now(),
        }
    }
}

/// Enhanced metrics collector with real-time analytics
pub struct MetricsCollector {
    metrics: Arc<RwLock<EventMetrics>>,
    start_time: Instant,
    /// Processing time samples for percentile calculation
    processing_samples: Arc<RwLock<VecDeque<Duration>>>,
    /// Error count tracking
    error_count: Arc<RwLock<u64>>,
    /// Real-time update task handle
    _update_task: Option<tokio::task::JoinHandle<()>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        let metrics = Arc::new(RwLock::new(EventMetrics::default()));
        let processing_samples = Arc::new(RwLock::new(VecDeque::with_capacity(10000)));
        let error_count = Arc::new(RwLock::new(0u64));

        // Start real-time analytics update task
        let update_task = Self::start_analytics_task(
            Arc::clone(&metrics),
            Arc::clone(&processing_samples),
            Arc::clone(&error_count),
        );

        Self {
            metrics,
            start_time: Instant::now(),
            processing_samples,
            error_count,
            _update_task: Some(update_task),
        }
    }

    /// Start background task for real-time analytics updates
    fn start_analytics_task(
        metrics: Arc<RwLock<EventMetrics>>,
        processing_samples: Arc<RwLock<VecDeque<Duration>>>,
        error_count: Arc<RwLock<u64>>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            let mut last_event_count = 0u64;
            let mut last_update = Instant::now();

            loop {
                interval.tick().await;

                let current_time = Instant::now();
                let elapsed = current_time.duration_since(last_update).as_secs_f64();

                let mut metrics_guard = metrics.write();
                let current_event_count = metrics_guard.total_events;

                // Calculate current EPS
                let current_eps = if elapsed > 0.0 {
                    (current_event_count - last_event_count) as f64 / elapsed
                } else {
                    0.0
                };

                // Update sliding window (keep last 60 seconds)
                metrics_guard
                    .real_time_analytics
                    .eps_history
                    .push_back(current_eps);
                if metrics_guard.real_time_analytics.eps_history.len() > 60 {
                    metrics_guard.real_time_analytics.eps_history.pop_front();
                }

                // Calculate throughput trend
                let trend = if metrics_guard.real_time_analytics.eps_history.len() >= 10 {
                    let recent: f64 = metrics_guard
                        .real_time_analytics
                        .eps_history
                        .iter()
                        .rev()
                        .take(5)
                        .sum::<f64>()
                        / 5.0;
                    let older: f64 = metrics_guard
                        .real_time_analytics
                        .eps_history
                        .iter()
                        .rev()
                        .skip(5)
                        .take(5)
                        .sum::<f64>()
                        / 5.0;

                    if recent > older * 1.1 {
                        ThroughputTrend::Increasing
                    } else if recent < older * 0.9 {
                        ThroughputTrend::Decreasing
                    } else {
                        ThroughputTrend::Stable
                    }
                } else {
                    ThroughputTrend::Unknown
                };

                metrics_guard.real_time_analytics.throughput_trend = trend;

                // Update latency percentiles
                let samples = processing_samples.read();
                if !samples.is_empty() {
                    let mut sorted_samples: Vec<Duration> = samples.iter().cloned().collect();
                    sorted_samples.sort();

                    let len = sorted_samples.len();
                    metrics_guard.real_time_analytics.latency_percentiles = LatencyPercentiles {
                        p50: sorted_samples[len * 50 / 100],
                        p90: sorted_samples[len * 90 / 100],
                        p95: sorted_samples[len * 95 / 100],
                        p99: sorted_samples[len * 99 / 100],
                    };
                }

                // Calculate error rate
                let total_errors = *error_count.read();
                metrics_guard.real_time_analytics.error_rate = if current_event_count > 0 {
                    total_errors as f64 / current_event_count as f64 * 100.0
                } else {
                    0.0
                };

                // Update memory usage (approximate)
                metrics_guard.real_time_analytics.memory_usage_bytes =
                    (samples.len() * std::mem::size_of::<Duration>()) as u64;

                metrics_guard.real_time_analytics.last_updated = SystemTime::now();

                last_event_count = current_event_count;
                last_update = current_time;
            }
        })
    }

    /// Record an event with enhanced analytics
    pub fn record_event(&self, event: &UniversalEvent, processing_time: Duration) {
        let mut metrics = self.metrics.write();

        metrics.total_events += 1;
        *metrics
            .events_by_type
            .entry(event.event_type.clone())
            .or_insert(0) += 1;
        *metrics
            .events_by_language
            .entry(event.language)
            .or_insert(0) += 1;

        // Update average processing time (exponential moving average)
        let alpha = 0.1; // Smoothing factor
        let current_avg = metrics.avg_processing_time.as_nanos() as f64;
        let new_time = processing_time.as_nanos() as f64;
        let new_avg = alpha * new_time + (1.0 - alpha) * current_avg;
        metrics.avg_processing_time = Duration::from_nanos(new_avg as u64);

        // Update events per second
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.current_events_per_second = metrics.total_events as f64 / elapsed;
            if metrics.current_events_per_second > metrics.peak_events_per_second {
                metrics.peak_events_per_second = metrics.current_events_per_second;
            }
        }

        // Store processing time sample for percentile calculation
        let mut samples = self.processing_samples.write();
        samples.push_back(processing_time);
        // Keep only recent samples (last 10,000)
        if samples.len() > 10000 {
            samples.pop_front();
        }
    }

    /// Record an error for error rate calculation
    pub fn record_error(&self) {
        let mut error_count = self.error_count.write();
        *error_count += 1;
    }

    /// Get real-time dashboard data
    pub fn get_dashboard_metrics(&self) -> DashboardMetrics {
        let metrics = self.metrics.read();

        DashboardMetrics {
            total_events: metrics.total_events,
            current_eps: metrics.current_events_per_second,
            peak_eps: metrics.peak_events_per_second,
            avg_latency: metrics.avg_processing_time,
            p99_latency: metrics.real_time_analytics.latency_percentiles.p99,
            error_rate: metrics.real_time_analytics.error_rate,
            throughput_trend: metrics.real_time_analytics.throughput_trend,
            memory_usage_mb: metrics.real_time_analytics.memory_usage_bytes as f64
                / 1024.0
                / 1024.0,
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> EventMetrics {
        self.metrics.read().clone()
    }

    /// Reset all metrics
    pub fn reset(&self) {
        *self.metrics.write() = EventMetrics::default();
    }
}

/// Dashboard metrics for real-time monitoring
#[derive(Debug)]
pub struct DashboardMetrics {
    pub total_events: u64,
    pub current_eps: f64,
    pub peak_eps: f64,
    pub avg_latency: Duration,
    pub p99_latency: Duration,
    pub error_rate: f64,
    pub throughput_trend: ThroughputTrend,
    pub memory_usage_mb: f64,
    pub uptime_seconds: u64,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventMetrics {
    fn clone(&self) -> Self {
        Self {
            total_events: self.total_events,
            events_by_type: self.events_by_type.clone(),
            events_by_language: self.events_by_language.clone(),
            avg_processing_time: self.avg_processing_time,
            peak_events_per_second: self.peak_events_per_second,
            current_events_per_second: self.current_events_per_second,
            real_time_analytics: RealTimeAnalytics::default(),
        }
    }
}
