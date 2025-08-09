//! ABOUTME: Performance monitoring and profiling for agents
//! ABOUTME: Tracks resource usage, response times, throughput, and generates performance reports

#![allow(clippy::significant_drop_tightening)]

use crate::monitoring::metrics::AgentMetrics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Resource usage snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// CPU usage percentage (0-100)
    pub cpu_percent: f64,
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// Thread count
    pub thread_count: usize,
    /// File descriptor count (Unix)
    pub fd_count: Option<usize>,
    /// Network bytes sent
    pub network_sent_bytes: u64,
    /// Network bytes received
    pub network_recv_bytes: u64,
}

impl ResourceUsage {
    /// Create a resource usage snapshot
    #[must_use]
    pub const fn snapshot() -> Self {
        // In a real implementation, this would use system APIs
        // For now, we'll return mock data
        Self {
            cpu_percent: 25.5,
            memory_bytes: 100 * 1024 * 1024, // 100MB
            thread_count: 8,
            fd_count: Some(42),
            network_sent_bytes: 1024 * 1024,     // 1MB
            network_recv_bytes: 2 * 1024 * 1024, // 2MB
        }
    }
}

/// Performance snapshot at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    /// Timestamp of the snapshot
    pub timestamp: DateTime<Utc>,
    /// Resource usage
    pub resources: ResourceUsage,
    /// Request rate (requests per second)
    pub request_rate: f64,
    /// Average response time (milliseconds)
    pub avg_response_time: f64,
    /// Error rate (percentage)
    pub error_rate: f64,
    /// Active requests
    pub active_requests: usize,
    /// Queue depth
    pub queue_depth: usize,
}

/// Performance report over a time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// Start time of the report period
    pub start_time: DateTime<Utc>,
    /// End time of the report period
    pub end_time: DateTime<Utc>,
    /// Average CPU usage
    pub avg_cpu_percent: f64,
    /// Peak CPU usage
    pub peak_cpu_percent: f64,
    /// Average memory usage
    pub avg_memory_bytes: u64,
    /// Peak memory usage
    pub peak_memory_bytes: u64,
    /// Total requests processed
    pub total_requests: u64,
    /// Total failed requests
    pub failed_requests: u64,
    /// Average response time
    pub avg_response_time: f64,
    /// 95th percentile response time
    pub p95_response_time: f64,
    /// 99th percentile response time
    pub p99_response_time: f64,
    /// Throughput (requests per second)
    pub throughput: f64,
    /// Availability (percentage)
    pub availability: f64,
}

impl PerformanceReport {
    /// Generate a report from snapshots
    ///
    /// # Panics
    ///
    /// Panics if snapshots is empty (though this is guarded against).
    #[must_use]
    pub fn from_snapshots(snapshots: &[PerformanceSnapshot]) -> Self {
        if snapshots.is_empty() {
            return Self::empty();
        }

        let start_time = snapshots.first().unwrap().timestamp;
        let end_time = snapshots.last().unwrap().timestamp;
        let duration = (end_time - start_time).to_std().unwrap_or_default();

        // Calculate averages and peaks
        let mut total_cpu = 0.0;
        let mut peak_cpu: f64 = 0.0;
        let mut total_memory = 0u64;
        let mut peak_memory = 0u64;
        let mut total_response_time = 0.0;
        let mut response_times = Vec::new();

        for snapshot in snapshots {
            total_cpu += snapshot.resources.cpu_percent;
            peak_cpu = peak_cpu.max(snapshot.resources.cpu_percent);
            total_memory += snapshot.resources.memory_bytes;
            peak_memory = peak_memory.max(snapshot.resources.memory_bytes);
            total_response_time += snapshot.avg_response_time;

            // Collect response times for percentile calculation
            response_times.push(snapshot.avg_response_time);
        }

        #[allow(clippy::cast_precision_loss)]
        let count = snapshots.len() as f64;
        let avg_cpu_percent = total_cpu / count;
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let avg_memory_bytes = (total_memory as f64 / count).round() as u64;
        let avg_response_time = total_response_time / count;

        // Calculate percentiles
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let p95_index = usize::try_from((response_times.len() as f64 * 0.95).round() as u64)
            .unwrap_or(0)
            .min(response_times.len() - 1);
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let p99_index = usize::try_from((response_times.len() as f64 * 0.99).round() as u64)
            .unwrap_or(0)
            .min(response_times.len() - 1);
        let p95_response_time = response_times[p95_index];
        let p99_response_time = response_times[p99_index];

        // Calculate totals from first and last snapshots
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let total_requests = snapshots.iter().map(|s| s.request_rate).sum::<f64>() as u64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let failed_requests = snapshots
            .iter()
            .map(|s| (s.error_rate * s.request_rate / 100.0) as u64)
            .sum();

        let throughput = if duration.as_secs() > 0 {
            #[allow(clippy::cast_precision_loss)]
            let throughput_val = total_requests as f64 / duration.as_secs_f64();
            throughput_val
        } else {
            0.0
        };

        let availability = if total_requests > 0 {
            #[allow(clippy::cast_precision_loss)]
            let avail_val =
                ((total_requests - failed_requests) as f64 / total_requests as f64) * 100.0;
            avail_val
        } else {
            100.0
        };

        Self {
            start_time,
            end_time,
            avg_cpu_percent,
            peak_cpu_percent: peak_cpu,
            avg_memory_bytes,
            peak_memory_bytes: peak_memory,
            total_requests,
            failed_requests,
            avg_response_time,
            p95_response_time,
            p99_response_time,
            throughput,
            availability,
        }
    }

    /// Create an empty report
    fn empty() -> Self {
        let now = Utc::now();
        Self {
            start_time: now,
            end_time: now,
            avg_cpu_percent: 0.0,
            peak_cpu_percent: 0.0,
            avg_memory_bytes: 0,
            peak_memory_bytes: 0,
            total_requests: 0,
            failed_requests: 0,
            avg_response_time: 0.0,
            p95_response_time: 0.0,
            p99_response_time: 0.0,
            throughput: 0.0,
            availability: 100.0,
        }
    }

    /// Generate a summary string
    #[must_use]
    pub fn summary(&self) -> String {
        let duration = (self.end_time - self.start_time)
            .to_std()
            .unwrap_or_default();
        format!(
            "Performance Report ({:.1}s):\n\
             - CPU: avg {:.1}%, peak {:.1}%\n\
             - Memory: avg {:.1}MB, peak {:.1}MB\n\
             - Requests: {} total, {} failed ({:.1}% success)\n\
             - Response Time: avg {:.1}ms, p95 {:.1}ms, p99 {:.1}ms\n\
             - Throughput: {:.1} req/s\n\
             - Availability: {:.1}%",
            duration.as_secs_f64(),
            self.avg_cpu_percent,
            self.peak_cpu_percent,
            {
                #[allow(clippy::cast_precision_loss)]
                let avg_mb = self.avg_memory_bytes as f64 / (1024.0 * 1024.0);
                avg_mb
            },
            {
                #[allow(clippy::cast_precision_loss)]
                let peak_mb = self.peak_memory_bytes as f64 / (1024.0 * 1024.0);
                peak_mb
            },
            self.total_requests,
            self.failed_requests,
            {
                #[allow(clippy::cast_precision_loss)]
                let success_rate = (self.failed_requests as f64
                    / self.total_requests.max(1) as f64)
                    .mul_add(-100.0, 100.0);
                success_rate
            },
            self.avg_response_time,
            self.p95_response_time,
            self.p99_response_time,
            self.throughput,
            self.availability
        )
    }
}

/// Performance monitor for tracking agent performance
pub struct PerformanceMonitor {
    /// Agent ID
    agent_id: String,
    /// Agent metrics
    metrics: Arc<AgentMetrics>,
    /// Performance snapshots (ring buffer)
    snapshots: Arc<RwLock<VecDeque<PerformanceSnapshot>>>,
    /// Maximum snapshots to keep
    max_snapshots: usize,
    /// Snapshot interval
    snapshot_interval: Duration,
    /// Custom thresholds for alerting
    thresholds: PerformanceThresholds,
}

/// Performance thresholds for alerting
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Maximum CPU usage percentage
    pub max_cpu_percent: f64,
    /// Maximum memory usage bytes
    pub max_memory_bytes: u64,
    /// Maximum response time milliseconds
    pub max_response_time_ms: u64,
    /// Minimum throughput (req/s)
    pub min_throughput: f64,
    /// Minimum availability percentage
    pub min_availability: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            max_cpu_percent: 80.0,
            max_memory_bytes: 1024 * 1024 * 1024, // 1GB
            max_response_time_ms: 5000,           // 5 seconds
            min_throughput: 10.0,                 // 10 req/s
            min_availability: 99.0,               // 99%
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    #[must_use]
    pub fn new(
        agent_id: String,
        metrics: Arc<AgentMetrics>,
        max_snapshots: usize,
        snapshot_interval: Duration,
    ) -> Self {
        Self {
            agent_id,
            metrics,
            snapshots: Arc::new(RwLock::new(VecDeque::with_capacity(max_snapshots))),
            max_snapshots,
            snapshot_interval,
            thresholds: PerformanceThresholds::default(),
        }
    }

    /// Set custom thresholds
    #[must_use]
    pub const fn with_thresholds(mut self, thresholds: PerformanceThresholds) -> Self {
        self.thresholds = thresholds;
        self
    }

    /// Take a performance snapshot
    #[must_use]
    pub fn take_snapshot(&self) -> PerformanceSnapshot {
        let resources = ResourceUsage::snapshot();

        // Calculate rates from metrics
        let total_requests = self.metrics.requests_total.get();
        let failed_requests = self.metrics.requests_failed.get();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let active_requests = self.metrics.requests_active.get() as usize;

        #[allow(clippy::cast_precision_loss)]
        let request_rate = total_requests as f64 / self.snapshot_interval.as_secs_f64();
        let error_rate = if total_requests > 0 {
            #[allow(clippy::cast_precision_loss)]
            let rate = (failed_requests as f64 / total_requests as f64) * 100.0;
            rate
        } else {
            0.0
        };

        // Get average response time from histogram
        let avg_response_time = match self.metrics.request_duration.get() {
            crate::monitoring::metrics::MetricValue::Histogram { sum, count, .. } => {
                if count > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let avg_time = (sum / count as f64) * 1000.0; // Convert to milliseconds
                    avg_time
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        PerformanceSnapshot {
            timestamp: Utc::now(),
            resources,
            request_rate,
            avg_response_time,
            error_rate,
            active_requests,
            queue_depth: 0, // Would need queue metrics
        }
    }

    /// Store a snapshot
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    fn store_snapshot(&self, snapshot: PerformanceSnapshot) {
        let mut snapshots = self.snapshots.write().unwrap();

        // Remove oldest if at capacity
        if snapshots.len() >= self.max_snapshots {
            snapshots.pop_front();
        }

        snapshots.push_back(snapshot);
    }

    /// Generate a performance report
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn generate_report(&self) -> PerformanceReport {
        let snapshots = self.snapshots.read().unwrap();
        let snapshots_vec: Vec<_> = snapshots.iter().cloned().collect();
        PerformanceReport::from_snapshots(&snapshots_vec)
    }

    /// Check if performance is within thresholds
    #[must_use]
    pub fn check_thresholds(&self, snapshot: &PerformanceSnapshot) -> Vec<PerformanceViolation> {
        let mut violations = Vec::new();

        if snapshot.resources.cpu_percent > self.thresholds.max_cpu_percent {
            violations.push(PerformanceViolation {
                metric: "cpu_percent".to_string(),
                current_value: snapshot.resources.cpu_percent,
                threshold_value: self.thresholds.max_cpu_percent,
                severity: ViolationSeverity::Warning,
            });
        }

        if snapshot.resources.memory_bytes > self.thresholds.max_memory_bytes {
            violations.push(PerformanceViolation {
                metric: "memory_bytes".to_string(),
                #[allow(clippy::cast_precision_loss)]
                current_value: snapshot.resources.memory_bytes as f64,
                #[allow(clippy::cast_precision_loss)]
                threshold_value: self.thresholds.max_memory_bytes as f64,
                severity: ViolationSeverity::Critical,
            });
        }

        #[allow(clippy::cast_precision_loss)]
        let max_response_time = self.thresholds.max_response_time_ms as f64;
        if snapshot.avg_response_time > max_response_time {
            violations.push(PerformanceViolation {
                metric: "response_time_ms".to_string(),
                current_value: snapshot.avg_response_time,
                threshold_value: max_response_time,
                severity: ViolationSeverity::Warning,
            });
        }

        if snapshot.request_rate < self.thresholds.min_throughput {
            violations.push(PerformanceViolation {
                metric: "throughput".to_string(),
                current_value: snapshot.request_rate,
                threshold_value: self.thresholds.min_throughput,
                severity: ViolationSeverity::Info,
            });
        }

        violations
    }

    /// Start performance monitoring
    pub fn start_monitoring(self: Arc<Self>) {
        let monitor = self;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor.snapshot_interval);

            loop {
                interval.tick().await;

                // Take and store snapshot
                let snapshot = monitor.take_snapshot();

                // Update metrics
                #[allow(clippy::cast_precision_loss)]
                let memory_f64 = snapshot.resources.memory_bytes as f64;
                monitor
                    .metrics
                    .update_resources(memory_f64, snapshot.resources.cpu_percent);

                // Check thresholds
                let violations = monitor.check_thresholds(&snapshot);
                for violation in &violations {
                    tracing::warn!(
                        "Performance threshold violation for {}: {} (current: {:.2}, threshold: {:.2})",
                        monitor.agent_id,
                        violation.metric,
                        violation.current_value,
                        violation.threshold_value
                    );
                }

                // Store snapshot
                monitor.store_snapshot(snapshot);
            }
        });
    }
}

/// Performance threshold violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceViolation {
    /// Metric name
    pub metric: String,
    /// Current value
    pub current_value: f64,
    /// Threshold value
    pub threshold_value: f64,
    /// Severity
    pub severity: ViolationSeverity,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Critical
    Critical,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_resource_usage_snapshot() {
        let usage = ResourceUsage::snapshot();
        assert!(usage.cpu_percent >= 0.0 && usage.cpu_percent <= 100.0);
        assert!(usage.memory_bytes > 0);
        assert!(usage.thread_count > 0);
    }
    #[test]
    #[allow(clippy::float_cmp)] // Test assertions on float values
    fn test_performance_report_from_snapshots() {
        let mut snapshots = Vec::new();

        for i in 0..5 {
            let snapshot = PerformanceSnapshot {
                #[allow(clippy::cast_possible_wrap)]
                timestamp: Utc::now() + chrono::Duration::seconds(i64::from(i)),
                resources: ResourceUsage {
                    cpu_percent: f64::from(i).mul_add(5.0, 20.0),
                    memory_bytes: 100 * 1024 * 1024
                        + (u64::try_from(i).unwrap_or(0) * 10 * 1024 * 1024),
                    thread_count: 8,
                    fd_count: Some(42),
                    network_sent_bytes: 0,
                    network_recv_bytes: 0,
                },
                request_rate: 100.0,
                avg_response_time: f64::from(i).mul_add(10.0, 50.0),
                error_rate: 1.0,
                active_requests: 10,
                queue_depth: 5,
            };
            snapshots.push(snapshot);
        }

        let report = PerformanceReport::from_snapshots(&snapshots);

        assert_eq!(report.avg_cpu_percent, 30.0); // (20 + 25 + 30 + 35 + 40) / 5
        assert_eq!(report.peak_cpu_percent, 40.0);
        assert_eq!(report.avg_memory_bytes, 120 * 1024 * 1024); // Average of the memory values
        assert_eq!(report.peak_memory_bytes, 140 * 1024 * 1024);
        assert_eq!(report.p95_response_time, 90.0); // 5th element (index 4)
        assert_eq!(report.p99_response_time, 90.0); // Same for small dataset

        let summary = report.summary();
        assert!(summary.contains("CPU: avg 30.0%, peak 40.0%"));
    }
    #[test]
    fn test_performance_threshold_violations() {
        let metrics = Arc::new(AgentMetrics::new("test-agent".to_string()));
        let monitor = PerformanceMonitor::new(
            "test-agent".to_string(),
            metrics,
            100,
            Duration::from_secs(1),
        );

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            resources: ResourceUsage {
                cpu_percent: 90.0,                    // Above default threshold of 80%
                memory_bytes: 2 * 1024 * 1024 * 1024, // Above 1GB threshold
                thread_count: 8,
                fd_count: Some(42),
                network_sent_bytes: 0,
                network_recv_bytes: 0,
            },
            request_rate: 5.0,         // Below minimum of 10 req/s
            avg_response_time: 6000.0, // Above 5 second threshold
            error_rate: 5.0,
            active_requests: 10,
            queue_depth: 5,
        };

        let violations = monitor.check_thresholds(&snapshot);
        assert_eq!(violations.len(), 4);

        // Check specific violations
        assert!(violations.iter().any(|v| v.metric == "cpu_percent"));
        assert!(violations.iter().any(|v| v.metric == "memory_bytes"));
        assert!(violations.iter().any(|v| v.metric == "response_time_ms"));
        assert!(violations.iter().any(|v| v.metric == "throughput"));
    }
    #[test]
    #[allow(clippy::float_cmp)] // Test assertions on float values
    fn test_snapshot_storage() {
        let metrics = Arc::new(AgentMetrics::new("test-agent".to_string()));
        let monitor = PerformanceMonitor::new(
            "test-agent".to_string(),
            metrics,
            3, // Max 3 snapshots
            Duration::from_secs(1),
        );

        // Store 5 snapshots
        for i in 0..5 {
            let snapshot = PerformanceSnapshot {
                timestamp: Utc::now(),
                resources: ResourceUsage::snapshot(),
                request_rate: f64::from(i),
                avg_response_time: 50.0,
                error_rate: 0.0,
                active_requests: 0,
                queue_depth: 0,
            };
            monitor.store_snapshot(snapshot);
        }

        // Should only have 3 snapshots (oldest removed)
        let snapshots = monitor.snapshots.read().unwrap();
        assert_eq!(snapshots.len(), 3);
        assert_eq!(snapshots[0].request_rate, 2.0); // First two were removed
        assert_eq!(snapshots[1].request_rate, 3.0);
        assert_eq!(snapshots[2].request_rate, 4.0);
    }
}
