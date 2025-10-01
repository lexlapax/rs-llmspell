//! Health monitoring and system metrics
//!
//! Provides comprehensive health monitoring for the kernel including:
//! - System resource monitoring (CPU, memory, uptime)
//! - Connection metrics from `MessageRouter`
//! - Performance metrics from `KernelState`
//! - Health status evaluation
//! - Optional HTTP health endpoint

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use sysinfo::{MemoryRefreshKind, Pid, RefreshKind, System};
use tokio::sync::RwLock;
use tracing::{debug, error, warn};

use crate::io::router::MessageRouter;
use crate::state::{KernelState, StateMetrics};

/// Overall health status of the kernel
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// All systems operating normally
    Healthy,
    /// Some issues detected but kernel is functional
    Degraded,
    /// Critical issues detected, kernel may not be functional
    Unhealthy,
}

impl HealthStatus {
    /// Convert to HTTP status code equivalent
    pub fn to_http_status(&self) -> u16 {
        match self {
            HealthStatus::Healthy | HealthStatus::Degraded => 200, // Still serving requests
            HealthStatus::Unhealthy => 503,                        // Service unavailable
        }
    }
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    /// Process ID
    pub pid: u32,
    /// Process uptime in seconds
    pub uptime_secs: u64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Memory usage in MB
    pub memory_usage_mb: u64,
    /// CPU usage percentage (0.0-100.0)
    pub cpu_usage_percent: f32,
    /// System total memory in bytes
    pub system_total_memory_bytes: u64,
    /// System available memory in bytes
    pub system_available_memory_bytes: u64,
    /// Number of CPU cores
    pub cpu_cores: usize,
    /// System load average (1, 5, 15 minutes) - Unix only
    #[cfg(unix)]
    pub load_average: Option<(f64, f64, f64)>,
}

/// Connection metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMetrics {
    /// Number of active client connections
    pub active_connections: usize,
    /// Total number of registered clients (active + inactive)
    pub total_registered_clients: usize,
    /// Connection IDs
    pub client_ids: Vec<String>,
}

/// Performance thresholds for health evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthThresholds {
    /// Maximum acceptable memory usage in MB
    pub max_memory_mb: u64,
    /// Maximum acceptable CPU usage percentage
    pub max_cpu_percent: f32,
    /// Maximum acceptable number of connections
    pub max_connections: usize,
    /// Maximum acceptable average latency in microseconds
    pub max_avg_latency_us: u64,
    /// Maximum acceptable error rate (errors per minute)
    pub max_error_rate_per_minute: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            max_memory_mb: 1024,             // 1 GB
            max_cpu_percent: 80.0,           // 80%
            max_connections: 100,            // 100 concurrent connections
            max_avg_latency_us: 5000,        // 5ms
            max_error_rate_per_minute: 10.0, // 10 errors per minute
        }
    }
}

/// Comprehensive health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    /// Overall health status
    pub status: HealthStatus,
    /// Timestamp of health check
    pub timestamp: DateTime<Utc>,
    /// System resource metrics
    pub system: SystemMetrics,
    /// Connection metrics
    pub connections: ConnectionMetrics,
    /// Performance metrics from kernel state
    pub performance: StateMetrics,
    /// Health issues detected
    pub issues: Vec<String>,
    /// Health thresholds used for evaluation
    pub thresholds: HealthThresholds,
    /// Kernel version and name
    pub kernel_info: KernelInfo,
}

/// Kernel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    /// Kernel name
    pub name: String,
    /// Kernel version
    pub version: String,
    /// Session ID
    pub session_id: Option<String>,
    /// Execution count
    pub execution_count: Option<u64>,
}

/// Health monitor with system tracking
pub struct HealthMonitor {
    /// System information tracker
    system: Arc<RwLock<System>>,
    /// Process start time for uptime calculation
    start_time: Instant,
    /// Process ID
    pid: Pid,
    /// Health thresholds
    thresholds: HealthThresholds,
    /// Last CPU measurement for accurate calculation
    last_cpu_measurement: Arc<RwLock<Option<Instant>>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(thresholds: Option<HealthThresholds>) -> Self {
        let mut system = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(sysinfo::CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::new().with_ram()),
        );
        system.refresh_all();

        let pid = sysinfo::get_current_pid().unwrap_or_else(|_| Pid::from_u32(0));

        Self {
            system: Arc::new(RwLock::new(system)),
            start_time: Instant::now(),
            pid,
            thresholds: thresholds.unwrap_or_default(),
            last_cpu_measurement: Arc::new(RwLock::new(None)),
        }
    }

    /// Update health thresholds
    pub fn set_thresholds(&mut self, thresholds: HealthThresholds) {
        self.thresholds = thresholds;
    }

    /// Get current system metrics
    ///
    /// # Errors
    ///
    /// Returns an error if system information cannot be retrieved
    pub async fn get_system_metrics(&self) -> Result<SystemMetrics> {
        let mut system = self.system.write().await;

        // Refresh system information
        system.refresh_cpu_all();
        system.refresh_memory();
        system.refresh_processes(sysinfo::ProcessesToUpdate::All);

        // Get process information
        let process = system
            .process(self.pid)
            .context("Failed to get current process information")?;

        let memory_usage_bytes = process.memory();
        let memory_usage_mb = memory_usage_bytes / (1024 * 1024);
        let cpu_usage_percent = process.cpu_usage();

        // Update CPU measurement timing
        {
            let mut last_measurement = self.last_cpu_measurement.write().await;
            *last_measurement = Some(Instant::now());
        }

        let uptime_secs = self.start_time.elapsed().as_secs();

        let system_total_memory_bytes = system.total_memory();
        let system_available_memory_bytes = system.available_memory();
        let cpu_cores = system.cpus().len();

        #[cfg(unix)]
        let load_average = {
            let load_avg = System::load_average();
            Some((load_avg.one, load_avg.five, load_avg.fifteen))
        };

        Ok(SystemMetrics {
            pid: self.pid.as_u32(),
            uptime_secs,
            memory_usage_bytes,
            memory_usage_mb,
            cpu_usage_percent,
            system_total_memory_bytes,
            system_available_memory_bytes,
            cpu_cores,
            #[cfg(unix)]
            load_average,
        })
    }

    /// Get connection metrics from message router
    pub fn get_connection_metrics(&self, router: &MessageRouter) -> ConnectionMetrics {
        let active_connections = router.active_client_count();
        let client_ids = router.get_client_ids();
        let total_registered_clients = client_ids.len();

        ConnectionMetrics {
            active_connections,
            total_registered_clients,
            client_ids,
        }
    }

    /// Evaluate health status based on metrics and thresholds
    pub fn evaluate_health(
        &self,
        system: &SystemMetrics,
        connections: &ConnectionMetrics,
        performance: &StateMetrics,
    ) -> (HealthStatus, Vec<String>) {
        let mut issues = Vec::new();
        let mut critical_issues = 0;
        let mut warning_issues = 0;

        // Check memory usage
        if system.memory_usage_mb > self.thresholds.max_memory_mb {
            let issue = format!(
                "High memory usage: {}MB > {}MB threshold",
                system.memory_usage_mb, self.thresholds.max_memory_mb
            );
            issues.push(issue);
            critical_issues += 1;
        } else if system.memory_usage_mb > self.thresholds.max_memory_mb * 8 / 10 {
            let issue = format!(
                "Elevated memory usage: {}MB ({}% of threshold)",
                system.memory_usage_mb,
                (system.memory_usage_mb * 100) / self.thresholds.max_memory_mb
            );
            issues.push(issue);
            warning_issues += 1;
        }

        // Check CPU usage
        if system.cpu_usage_percent > self.thresholds.max_cpu_percent {
            let issue = format!(
                "High CPU usage: {:.1}% > {:.1}% threshold",
                system.cpu_usage_percent, self.thresholds.max_cpu_percent
            );
            issues.push(issue);
            critical_issues += 1;
        } else if system.cpu_usage_percent > self.thresholds.max_cpu_percent * 0.8 {
            let issue = format!(
                "Elevated CPU usage: {:.1}% ({:.1}% of threshold)",
                system.cpu_usage_percent,
                (system.cpu_usage_percent * 100.0) / self.thresholds.max_cpu_percent
            );
            issues.push(issue);
            warning_issues += 1;
        }

        // Check connection count
        if connections.active_connections > self.thresholds.max_connections {
            let issue = format!(
                "High connection count: {} > {} threshold",
                connections.active_connections, self.thresholds.max_connections
            );
            issues.push(issue);
            warning_issues += 1;
        }

        // Check average latency
        if performance.avg_read_latency_us > self.thresholds.max_avg_latency_us {
            let issue = format!(
                "High read latency: {}μs > {}μs threshold",
                performance.avg_read_latency_us, self.thresholds.max_avg_latency_us
            );
            issues.push(issue);
            warning_issues += 1;
        }

        if performance.avg_write_latency_us > self.thresholds.max_avg_latency_us {
            let issue = format!(
                "High write latency: {}μs > {}μs threshold",
                performance.avg_write_latency_us, self.thresholds.max_avg_latency_us
            );
            issues.push(issue);
            warning_issues += 1;
        }

        // Determine overall status
        let status = if critical_issues > 0 {
            HealthStatus::Unhealthy
        } else if warning_issues > 0 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        (status, issues)
    }

    /// Generate comprehensive health report
    ///
    /// # Errors
    ///
    /// Returns an error if system metrics cannot be retrieved
    pub async fn health_check(
        &self,
        kernel_state: &KernelState,
        message_router: &MessageRouter,
        session_id: Option<String>,
    ) -> Result<HealthReport> {
        debug!("Performing health check");

        let system = self.get_system_metrics().await?;
        let connections = self.get_connection_metrics(message_router);
        let performance = kernel_state.metrics();

        let (status, issues) = self.evaluate_health(&system, &connections, &performance);

        let kernel_info = KernelInfo {
            name: env!("CARGO_PKG_NAME").to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            session_id,
            execution_count: Some(kernel_state.execution_count()),
        };

        let report = HealthReport {
            status,
            timestamp: Utc::now(),
            system,
            connections,
            performance,
            issues,
            thresholds: self.thresholds.clone(),
            kernel_info,
        };

        match report.status {
            HealthStatus::Healthy => debug!("Health check: HEALTHY"),
            HealthStatus::Degraded => {
                warn!("Health check: DEGRADED - {} issues", report.issues.len());
            }
            HealthStatus::Unhealthy => {
                error!("Health check: UNHEALTHY - {} issues", report.issues.len());
            }
        }

        Ok(report)
    }

    /// Get a simple health status without full report
    ///
    /// # Errors
    ///
    /// Returns an error if health check fails
    pub async fn quick_health_check(
        &self,
        kernel_state: &KernelState,
        message_router: &MessageRouter,
    ) -> Result<HealthStatus> {
        let system = self.get_system_metrics().await?;
        let connections = self.get_connection_metrics(message_router);
        let performance = kernel_state.metrics();

        let (status, _) = self.evaluate_health(&system, &connections, &performance);
        Ok(status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_health_monitor_creation() {
        let monitor = HealthMonitor::new(None);
        assert!(monitor.pid.as_u32() > 0);
        assert_eq!(monitor.thresholds.max_memory_mb, 1024);
    }

    #[tokio::test]
    async fn test_custom_thresholds() {
        let custom_thresholds = HealthThresholds {
            max_memory_mb: 2048,
            max_cpu_percent: 90.0,
            max_connections: 200,
            max_avg_latency_us: 10000,
            max_error_rate_per_minute: 20.0,
        };

        let monitor = HealthMonitor::new(Some(custom_thresholds.clone()));
        assert_eq!(monitor.thresholds.max_memory_mb, 2048);
        assert!((monitor.thresholds.max_cpu_percent - 90.0).abs() < f32::EPSILON);
    }

    #[tokio::test]
    async fn test_system_metrics() {
        let monitor = HealthMonitor::new(None);
        let metrics = monitor.get_system_metrics().await.unwrap();

        assert!(metrics.pid > 0);
        // Uptime should be a reasonable value (we just started, so should be small)
        assert!(metrics.uptime_secs < 3600); // Less than 1 hour
        assert!(metrics.memory_usage_bytes > 0);
        assert!(metrics.memory_usage_mb > 0);
        assert!(metrics.cpu_cores > 0);
        assert!(metrics.system_total_memory_bytes > 0);
    }

    #[test]
    fn test_health_status_http_codes() {
        assert_eq!(HealthStatus::Healthy.to_http_status(), 200);
        assert_eq!(HealthStatus::Degraded.to_http_status(), 200);
        assert_eq!(HealthStatus::Unhealthy.to_http_status(), 503);
    }

    #[test]
    fn test_health_evaluation() {
        let monitor = HealthMonitor::new(None);

        // Test healthy metrics
        let system = SystemMetrics {
            pid: 1234,
            uptime_secs: 3600,
            memory_usage_bytes: 100 * 1024 * 1024, // 100MB
            memory_usage_mb: 100,
            cpu_usage_percent: 10.0,
            system_total_memory_bytes: 8 * 1024 * 1024 * 1024, // 8GB
            system_available_memory_bytes: 4 * 1024 * 1024 * 1024, // 4GB
            cpu_cores: 4,
            #[cfg(unix)]
            load_average: Some((0.5, 0.3, 0.2)),
        };

        let connections = ConnectionMetrics {
            active_connections: 10,
            total_registered_clients: 15,
            client_ids: vec!["client1".to_string(), "client2".to_string()],
        };

        let performance = StateMetrics {
            reads: 1000,
            writes: 500,
            avg_read_latency_us: 1000,
            avg_write_latency_us: 2000,
            persistence_ops: 100,
            circuit_breaker_trips: 0,
            read_errors: 0,
            write_errors: 0,
            persistence_errors: 0,
            last_error_at: None,
            last_update: Some(std::time::Instant::now()),
        };

        let (status, issues) = monitor.evaluate_health(&system, &connections, &performance);
        assert_eq!(status, HealthStatus::Healthy);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_health_evaluation_unhealthy() {
        let monitor = HealthMonitor::new(None);

        // Test unhealthy metrics
        let system = SystemMetrics {
            pid: 1234,
            uptime_secs: 3600,
            memory_usage_bytes: 2 * 1024 * 1024 * 1024, // 2GB
            memory_usage_mb: 2048,                      // Exceeds default 1024MB threshold
            cpu_usage_percent: 95.0,                    // Exceeds default 80% threshold
            system_total_memory_bytes: 8 * 1024 * 1024 * 1024,
            system_available_memory_bytes: 1024 * 1024 * 1024,
            cpu_cores: 4,
            #[cfg(unix)]
            load_average: Some((5.0, 4.0, 3.0)),
        };

        let connections = ConnectionMetrics {
            active_connections: 10,
            total_registered_clients: 15,
            client_ids: vec![],
        };

        let performance = StateMetrics::default();

        let (status, issues) = monitor.evaluate_health(&system, &connections, &performance);
        assert_eq!(status, HealthStatus::Unhealthy);
        assert!(!issues.is_empty());
        assert!(issues
            .iter()
            .any(|issue| issue.contains("High memory usage")));
        assert!(issues.iter().any(|issue| issue.contains("High CPU usage")));
    }
}
