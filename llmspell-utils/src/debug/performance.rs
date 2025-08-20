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
    laps: Arc<RwLock<Vec<Lap>>>,
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
}

impl PerformanceTracker {
    /// Create a new performance tracker
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            start: Instant::now(),
            laps: Arc::new(RwLock::new(Vec::new())),
            parent: None,
            children: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Create a child tracker
    #[must_use]
    pub fn child(&self, name: impl Into<String>) -> Arc<PerformanceTracker> {
        let child = Arc::new(Self {
            name: name.into(),
            start: Instant::now(),
            laps: Arc::new(RwLock::new(Vec::new())),
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
        });
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

        let lap_times: Vec<Duration> = laps.iter().map(|l| l.duration).collect();

        TimingStats {
            name: self.name.clone(),
            total,
            count: laps.len(),
            min: lap_times.iter().min().copied(),
            max: lap_times.iter().max().copied(),
            mean: if lap_times.is_empty() {
                None
            } else {
                Some(Duration::from_nanos(
                    u64::try_from(lap_times.iter().map(Duration::as_nanos).sum::<u128>())
                        .unwrap_or(u64::MAX)
                        / lap_times.len() as u64,
                ))
            },
            laps: laps.into_iter().map(|l| (l.name, l.duration)).collect(),
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
    /// List of lap names and durations
    pub laps: Vec<(String, Duration)>,
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
