// ABOUTME: Event metrics collection and reporting
// ABOUTME: Tracks event throughput, latency, and system health metrics

use crate::universal_event::{Language, UniversalEvent};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Event metrics collector
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
}

/// Metrics collector implementation
pub struct MetricsCollector {
    metrics: Arc<RwLock<EventMetrics>>,
    start_time: Instant,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(EventMetrics::default())),
            start_time: Instant::now(),
        }
    }

    /// Record an event
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

        // Update average processing time (simple moving average)
        let current_avg = metrics.avg_processing_time.as_nanos() as f64;
        let new_time = processing_time.as_nanos() as f64;
        let count = metrics.total_events as f64;
        let new_avg = (current_avg * (count - 1.0) + new_time) / count;
        metrics.avg_processing_time = Duration::from_nanos(new_avg as u64);

        // Update events per second
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            metrics.current_events_per_second = metrics.total_events as f64 / elapsed;
            if metrics.current_events_per_second > metrics.peak_events_per_second {
                metrics.peak_events_per_second = metrics.current_events_per_second;
            }
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
        }
    }
}
