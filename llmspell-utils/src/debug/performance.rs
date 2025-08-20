//! Performance tracking utilities for debug profiling
//!
//! Provides hierarchical timers and performance statistics collection
//! for profiling script execution.

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Write;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Performance tracker for timing operations
#[derive(Debug, Clone)]
pub struct PerformanceTracker {
    name: String,
    start: Instant,
    start_memory: Option<u64>,
    laps: Arc<RwLock<Vec<Lap>>>,
    events: Arc<RwLock<Vec<TimingEvent>>>,
    #[allow(dead_code)]
    parent: Option<Arc<PerformanceTracker>>,
    children: Arc<RwLock<Vec<Arc<PerformanceTracker>>>>,
}

/// A lap or checkpoint in performance tracking
#[derive(Debug, Clone)]
pub struct Lap {
    /// Name of the lap/checkpoint
    pub name: String,
    /// Duration since last lap
    pub duration: Duration,
    /// Timestamp when lap was recorded
    pub timestamp: Instant,
    /// Memory usage when lap was recorded (bytes)
    pub memory_bytes: Option<u64>,
}

/// Custom event recorded during timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingEvent {
    /// Event name
    pub name: String,
    /// When the event occurred (relative to timer start)
    pub offset: Duration,
    /// Event metadata
    pub metadata: Option<serde_json::Value>,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            start_memory: Self::get_memory_usage(),
            laps: Arc::new(RwLock::new(Vec::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            parent: None,
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current memory usage (placeholder - would use real allocator stats in production)
    fn get_memory_usage() -> Option<u64> {
        // In a real implementation, this would query the allocator
        // For now, return None as a placeholder
        None
    }

    /// Create a child tracker
    #[must_use]
    pub fn child(&self, name: impl Into<String>) -> Arc<PerformanceTracker> {
        let child = Arc::new(Self {
            name: name.into(),
            start: Instant::now(),
            start_memory: Self::get_memory_usage(),
            laps: Arc::new(RwLock::new(Vec::new())),
            events: Arc::new(RwLock::new(Vec::new())),
            parent: None, // Avoid circular reference
            children: Arc::new(RwLock::new(Vec::new())),
        });

        self.children.write().push(child.clone());
        child
    }

    /// Record a lap/checkpoint
    pub fn lap(&self, name: impl Into<String>) {
        let now = Instant::now();
        let last_time = self.laps.read().last().map_or(self.start, |l| l.timestamp);

        self.laps.write().push(Lap {
            name: name.into(),
            duration: now - last_time,
            timestamp: now,
            memory_bytes: Self::get_memory_usage(),
        });
    }

    /// Record a custom event
    pub fn event(&self, name: impl Into<String>, metadata: Option<serde_json::Value>) {
        let now = Instant::now();
        self.events.write().push(TimingEvent {
            name: name.into(),
            offset: now - self.start,
            metadata,
        });
    }

    /// Get all events
    #[must_use]
    pub fn get_events(&self) -> Vec<TimingEvent> {
        self.events.read().clone()
    }

    /// Stop the tracker and return total duration
    #[must_use]
    pub fn stop(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get the elapsed time without stopping
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Get all laps
    #[must_use]
    pub fn get_laps(&self) -> Vec<Lap> {
        self.laps.read().clone()
    }

    /// Get statistics for this tracker
    #[must_use]
    pub fn get_stats(&self) -> TimingStats {
        let total = self.elapsed();
        let laps = self.get_laps();
        let events = self.get_events();

        let lap_times: Vec<Duration> = laps.iter().map(|l| l.duration).collect();
        let mut sorted_times = lap_times.clone();
        sorted_times.sort();

        // Calculate percentiles
        let (median, p95, p99) = if sorted_times.is_empty() {
            (None, None, None)
        } else {
            let len = sorted_times.len();
            let median = sorted_times[len / 2];
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let p95_idx = ((len as f64) * 0.95).floor() as usize;
            #[allow(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                clippy::cast_precision_loss
            )]
            let p99_idx = ((len as f64) * 0.99).floor() as usize;
            (
                Some(median),
                Some(sorted_times[p95_idx.min(len - 1)]),
                Some(sorted_times[p99_idx.min(len - 1)]),
            )
        };

        // Calculate mean and standard deviation
        let (mean, std_dev) = if lap_times.is_empty() {
            (None, None)
        } else {
            let sum_nanos: u128 = lap_times.iter().map(Duration::as_nanos).sum();
            #[allow(clippy::cast_precision_loss)]
            let mean_nanos = sum_nanos / lap_times.len() as u128;
            let mean_duration = Duration::from_nanos(u64::try_from(mean_nanos).unwrap_or(u64::MAX));

            // Calculate variance
            #[allow(clippy::cast_precision_loss)]
            let variance: f64 = lap_times
                .iter()
                .map(|d| {
                    #[allow(clippy::cast_precision_loss)]
                    let diff = d.as_nanos() as f64 - mean_nanos as f64;
                    diff * diff
                })
                .sum::<f64>()
                / lap_times.len() as f64;

            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let std_dev_duration = Duration::from_nanos(variance.sqrt() as u64);

            (Some(mean_duration), Some(std_dev_duration))
        };

        TimingStats {
            name: self.name.clone(),
            total,
            count: laps.len(),
            min: lap_times.iter().min().copied(),
            max: lap_times.iter().max().copied(),
            mean,
            median,
            p95,
            p99,
            std_dev,
            laps: laps.into_iter().map(|l| (l.name, l.duration)).collect(),
            memory_start: self.start_memory,
            memory_end: Self::get_memory_usage(),
            events,
        }
    }

    /// Format as a report string
    #[must_use]
    pub fn format_report(&self) -> String {
        let stats = self.get_stats();
        let mut report = format!("Performance Report: {}\n", self.name);
        let _ = writeln!(report, "Total: {:?}", stats.total);

        if !stats.laps.is_empty() {
            report.push_str("Laps:\n");
            for (name, duration) in &stats.laps {
                let _ = writeln!(report, "  - {name}: {duration:?}");
            }
        }

        if !self.children.read().is_empty() {
            report.push_str("Children:\n");
            for child in self.children.read().iter() {
                let child_stats = child.get_stats();
                let _ = writeln!(report, "  - {}: {:?}", child_stats.name, child_stats.total);
            }
        }

        report
    }
}

/// Statistics for timing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingStats {
    /// Name of the operation
    pub name: String,
    /// Total duration
    pub total: Duration,
    /// Number of laps
    pub count: usize,
    /// Minimum lap duration
    pub min: Option<Duration>,
    /// Maximum lap duration
    pub max: Option<Duration>,
    /// Mean lap duration
    pub mean: Option<Duration>,
    /// Median lap duration
    pub median: Option<Duration>,
    /// 95th percentile lap duration
    pub p95: Option<Duration>,
    /// 99th percentile lap duration
    pub p99: Option<Duration>,
    /// Standard deviation of lap durations
    pub std_dev: Option<Duration>,
    /// List of lap names and durations
    pub laps: Vec<(String, Duration)>,
    /// Memory usage at start (bytes)
    pub memory_start: Option<u64>,
    /// Memory usage at end (bytes)
    pub memory_end: Option<u64>,
    /// Custom events recorded during timing
    pub events: Vec<TimingEvent>,
}

/// Global performance profiler for collecting all timings
#[derive(Debug, Clone)]
pub struct Profiler {
    trackers: Arc<RwLock<HashMap<String, Arc<PerformanceTracker>>>>,
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler {
    /// Create a new profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            trackers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start a new timer
    pub fn start_timer(&self, name: impl Into<String>) -> Arc<PerformanceTracker> {
        let name = name.into();
        let tracker = Arc::new(PerformanceTracker::new(name.clone()));
        self.trackers.write().insert(name, tracker.clone());
        tracker
    }

    /// Get a tracker by name
    #[must_use]
    pub fn get_tracker(&self, name: &str) -> Option<Arc<PerformanceTracker>> {
        self.trackers.read().get(name).cloned()
    }

    /// Get all trackers
    #[must_use]
    pub fn get_all_trackers(&self) -> Vec<Arc<PerformanceTracker>> {
        self.trackers.read().values().cloned().collect()
    }

    /// Generate a full performance report
    #[must_use]
    pub fn generate_report(&self) -> ProfileReport {
        let trackers = self.get_all_trackers();
        let timings: HashMap<String, TimingStats> = trackers
            .iter()
            .map(|t| (t.name.clone(), t.get_stats()))
            .collect();

        ProfileReport {
            timings,
            generated_at: Instant::now(),
        }
    }

    /// Clear all trackers
    pub fn clear(&self) {
        self.trackers.write().clear();
    }

    /// Generate flame graph compatible output
    #[must_use]
    pub fn generate_flame_graph(&self) -> String {
        let mut output = String::new();
        let trackers = self.get_all_trackers();

        for tracker in trackers {
            Self::flame_graph_recursive(&tracker, &mut output, "");
        }

        output
    }

    /// Recursively generate flame graph entries
    fn flame_graph_recursive(tracker: &PerformanceTracker, output: &mut String, stack: &str) {
        let stats = tracker.get_stats();
        let stack_name = if stack.is_empty() {
            stats.name.clone()
        } else {
            format!("{stack};{}", stats.name)
        };

        // Add main entry for this tracker
        let _ = writeln!(output, "{} {}", stack_name, stats.total.as_micros());

        // Add entries for each lap
        for (lap_name, duration) in &stats.laps {
            let lap_stack = format!("{stack_name};{lap_name}");
            let _ = writeln!(output, "{} {}", lap_stack, duration.as_micros());
        }

        // Process children recursively
        for child in tracker.children.read().iter() {
            Self::flame_graph_recursive(child, output, &stack_name);
        }
    }

    /// Generate JSON performance report
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails
    pub fn generate_json_report(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        serde_json::to_string_pretty(&JsonReport {
            version: "1.0".to_string(),
            generated_at: format!("timestamp_{now}"),
            summary: JsonSummary::from_report(&report),
            timings: report.timings,
        })
    }

    /// Generate memory usage snapshot
    #[must_use]
    pub fn generate_memory_snapshot(&self) -> MemorySnapshot {
        let trackers = self.get_all_trackers();
        let total_trackers = trackers.len();

        // Calculate total memory delta if available
        let memory_delta = trackers
            .iter()
            .filter_map(|t| {
                let stats = t.get_stats();
                match (stats.memory_start, stats.memory_end) {
                    #[allow(clippy::cast_possible_wrap)]
                    (Some(start), Some(end)) => Some(end as i64 - start as i64),
                    _ => None,
                }
            })
            .sum::<i64>();

        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        MemorySnapshot {
            timestamp_secs,
            active_trackers: total_trackers,
            total_memory_delta_bytes: if memory_delta != 0 {
                Some(memory_delta)
            } else {
                None
            },
            tracker_memory_usage: trackers
                .iter()
                .map(|t| {
                    let stats = t.get_stats();
                    (
                        stats.name.clone(),
                        TrackerMemoryInfo {
                            start_bytes: stats.memory_start,
                            end_bytes: stats.memory_end,
                            delta_bytes: match (stats.memory_start, stats.memory_end) {
                                #[allow(clippy::cast_possible_wrap)]
                                (Some(start), Some(end)) => Some(end as i64 - start as i64),
                                _ => None,
                            },
                        },
                    )
                })
                .collect(),
        }
    }
}

/// Complete profile report
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Map of operation names to timing statistics
    pub timings: HashMap<String, TimingStats>,
    /// When the report was generated
    pub generated_at: Instant,
}

impl ProfileReport {
    /// Format as a human-readable string
    #[must_use]
    pub fn format(&self) -> String {
        let mut report = String::from("=== Performance Profile Report ===\n\n");

        for (name, stats) in &self.timings {
            let _ = writeln!(report, "{name}: {stats:?}", stats = stats.total);

            if stats.count > 0 {
                let _ = writeln!(report, "  Laps: {}", stats.count);
                if let Some(min) = stats.min {
                    let _ = writeln!(report, "  Min: {min:?}");
                }
                if let Some(max) = stats.max {
                    let _ = writeln!(report, "  Max: {max:?}");
                }
                if let Some(mean) = stats.mean {
                    let _ = writeln!(report, "  Mean: {mean:?}");
                }
            }
            report.push('\n');
        }

        report
    }
}

/// JSON-serializable performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonReport {
    /// Report format version
    pub version: String,
    /// When the report was generated (RFC3339)
    pub generated_at: String,
    /// Summary statistics
    pub summary: JsonSummary,
    /// Detailed timing statistics
    pub timings: HashMap<String, TimingStats>,
}

/// Summary statistics for JSON report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSummary {
    /// Total number of tracked operations
    pub total_operations: usize,
    /// Total time tracked across all operations
    pub total_time_ms: f64,
    /// Average operation time
    pub avg_time_ms: f64,
    /// Slowest operation
    pub slowest_operation: Option<String>,
    /// Fastest operation
    pub fastest_operation: Option<String>,
}

impl JsonSummary {
    fn from_report(report: &ProfileReport) -> Self {
        let total_operations = report.timings.len();
        let total_time: Duration = report.timings.values().map(|stats| stats.total).sum();

        let avg_time = if total_operations > 0 {
            #[allow(clippy::cast_precision_loss)]
            let total_ops_f64 = total_operations as f64;
            total_time.as_secs_f64() * 1000.0 / total_ops_f64
        } else {
            0.0
        };

        let (slowest_operation, fastest_operation) = if report.timings.is_empty() {
            (None, None)
        } else {
            let mut min_time = Duration::MAX;
            let mut max_time = Duration::ZERO;
            let mut fastest = None;
            let mut slowest = None;

            for (name, stats) in &report.timings {
                if stats.total < min_time {
                    min_time = stats.total;
                    fastest = Some(name.clone());
                }
                if stats.total > max_time {
                    max_time = stats.total;
                    slowest = Some(name.clone());
                }
            }

            (slowest, fastest)
        };

        Self {
            total_operations,
            total_time_ms: total_time.as_secs_f64() * 1000.0,
            avg_time_ms: avg_time,
            slowest_operation,
            fastest_operation,
        }
    }
}

/// Memory usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    /// When the snapshot was taken (seconds since epoch)
    pub timestamp_secs: u64,
    /// Number of active trackers
    pub active_trackers: usize,
    /// Total memory delta across all trackers (bytes)
    pub total_memory_delta_bytes: Option<i64>,
    /// Per-tracker memory information
    pub tracker_memory_usage: HashMap<String, TrackerMemoryInfo>,
}

/// Memory information for a specific tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerMemoryInfo {
    /// Memory usage at tracker start (bytes)
    pub start_bytes: Option<u64>,
    /// Memory usage at tracker end (bytes)
    pub end_bytes: Option<u64>,
    /// Memory delta (end - start, can be negative)
    pub delta_bytes: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new("test_operation");

        thread::sleep(Duration::from_millis(10));
        tracker.lap("step1");

        thread::sleep(Duration::from_millis(10));
        tracker.lap("step2");

        let stats = tracker.get_stats();
        assert_eq!(stats.name, "test_operation");
        assert_eq!(stats.count, 2);
        assert!(stats.total >= Duration::from_millis(20));
    }

    #[test]
    fn test_profiler() {
        let profiler = Profiler::new();

        let timer1 = profiler.start_timer("operation1");
        thread::sleep(Duration::from_millis(5));
        let _ = timer1.stop();

        let timer2 = profiler.start_timer("operation2");
        thread::sleep(Duration::from_millis(5));
        let _ = timer2.stop();

        let report = profiler.generate_report();
        assert_eq!(report.timings.len(), 2);
        assert!(report.timings.contains_key("operation1"));
        assert!(report.timings.contains_key("operation2"));
    }
}
