//! ABOUTME: Resource monitoring and enforcement for sandbox execution
//! ABOUTME: Tracks CPU, memory, network bandwidth, and custom resource usage with limits

use super::{SandboxContext, SandboxViolation};
use llmspell_core::{error::LLMSpellError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::time::interval;
use tracing::{debug, warn};

/// Resource usage statistics
#[derive(Debug, Clone)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time used in milliseconds
    pub cpu_time_ms: u64,
    /// Network bytes transferred
    pub network_bytes: u64,
    /// File operations performed
    pub file_operations: u32,
    /// Custom resource usage
    pub custom_usage: HashMap<String, u64>,
    /// Timestamp of measurement
    pub timestamp: Instant,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_bytes: 0,
            cpu_time_ms: 0,
            network_bytes: 0,
            file_operations: 0,
            custom_usage: HashMap::new(),
            timestamp: Instant::now(),
        }
    }
}

/// Resource monitor for tracking and enforcing limits
pub struct ResourceMonitor {
    context: SandboxContext,
    violations: Vec<SandboxViolation>,
    current_usage: Arc<RwLock<ResourceUsage>>,
    monitoring_active: Arc<RwLock<bool>>,
    start_time: Instant,
}

impl ResourceMonitor {
    /// Create a new resource monitor
    pub fn new(context: SandboxContext) -> Result<Self> {
        Ok(Self {
            context,
            violations: Vec::new(),
            current_usage: Arc::new(RwLock::new(ResourceUsage::default())),
            monitoring_active: Arc::new(RwLock::new(false)),
            start_time: Instant::now(),
        })
    }

    /// Start resource monitoring
    pub async fn start(&mut self) -> Result<()> {
        {
            let mut active = self.monitoring_active.write().await;
            *active = true;
        }

        self.start_time = Instant::now();
        debug!(
            "Resource monitoring started for sandbox: {}",
            self.context.id
        );

        // Start background monitoring task
        let usage = Arc::clone(&self.current_usage);
        let active = Arc::clone(&self.monitoring_active);
        let resource_limits = self.context.resource_limits.clone();
        let violations = Arc::new(RwLock::new(Vec::new()));
        let violations_clone = Arc::clone(&violations);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // Monitor every 100ms

            while *active.read().await {
                interval.tick().await;

                // Update resource usage
                let new_usage = Self::collect_system_usage().await;
                {
                    let mut current = usage.write().await;
                    *current = new_usage;
                }

                // Check limits
                let current = usage.read().await;
                let mut viols = violations_clone.write().await;

                if let Some(limit) = resource_limits.max_memory_bytes {
                    if current.memory_bytes > limit {
                        viols.push(SandboxViolation::ResourceLimit {
                            resource: "memory".to_string(),
                            limit,
                            actual: current.memory_bytes,
                            reason: "Memory usage exceeded limit".to_string(),
                        });
                    }
                }

                if let Some(limit) = resource_limits.max_cpu_time_ms {
                    if current.cpu_time_ms > limit {
                        viols.push(SandboxViolation::ResourceLimit {
                            resource: "cpu_time".to_string(),
                            limit,
                            actual: current.cpu_time_ms,
                            reason: "CPU time exceeded limit".to_string(),
                        });
                    }
                }

                if let Some(limit) = resource_limits.max_file_ops_per_sec {
                    let ops_per_sec = f64::from(current.file_operations)
                        / current.timestamp.elapsed().as_secs_f64().max(1.0);
                    #[allow(clippy::cast_precision_loss)]
                    let limit_f64 = limit as f64;
                    if ops_per_sec > limit_f64 {
                        viols.push(SandboxViolation::ResourceLimit {
                            resource: "file_operations".to_string(),
                            limit: u64::from(limit),
                            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            actual: ops_per_sec as u64,
                            reason: "File operations per second exceeded limit".to_string(),
                        });
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop resource monitoring
    pub async fn stop(&mut self) -> Result<()> {
        {
            let mut active = self.monitoring_active.write().await;
            *active = false;
        }

        debug!(
            "Resource monitoring stopped for sandbox: {}",
            self.context.id
        );
        Ok(())
    }

    /// Check if monitoring is active
    pub async fn is_monitoring(&self) -> bool {
        *self.monitoring_active.read().await
    }

    /// Get current resource usage
    pub async fn get_current_usage(&self) -> ResourceUsage {
        self.current_usage.read().await.clone()
    }

    /// Record network usage
    pub async fn record_network_usage(&self, bytes: u64) -> Result<()> {
        {
            let mut usage = self.current_usage.write().await;
            usage.network_bytes += bytes;
            usage.timestamp = Instant::now();
        }

        // Check network bandwidth limit
        if let Some(limit) = self.context.resource_limits.max_network_bps {
            let usage = self.current_usage.read().await;
            let duration = self.start_time.elapsed().as_secs_f64().max(1.0);
            #[allow(clippy::cast_precision_loss)]
            let bps = usage.network_bytes as f64 / duration;

            #[allow(clippy::cast_precision_loss)]
            let limit_f64 = limit as f64;
            if bps > limit_f64 {
                let violation = SandboxViolation::ResourceLimit {
                    resource: "network_bandwidth".to_string(),
                    limit,
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    actual: bps as u64,
                    reason: "Network bandwidth exceeded limit".to_string(),
                };
                warn!("Resource violation: {}", violation);
                return Err(LLMSpellError::Security {
                    message: violation.to_string(),
                    violation_type: Some("resource_limit".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Record file operation
    pub async fn record_file_operation(&self) -> Result<()> {
        {
            let mut usage = self.current_usage.write().await;
            usage.file_operations += 1;
            usage.timestamp = Instant::now();
        }

        // Check file operations per second limit
        if let Some(limit) = self.context.resource_limits.max_file_ops_per_sec {
            let usage = self.current_usage.read().await;
            let duration = self.start_time.elapsed().as_secs_f64().max(1.0);
            let ops_per_sec = f64::from(usage.file_operations) / duration;

            #[allow(clippy::cast_precision_loss)]
            let limit_f64 = limit as f64;
            if ops_per_sec > limit_f64 {
                let violation = SandboxViolation::ResourceLimit {
                    resource: "file_operations".to_string(),
                    limit: u64::from(limit),
                    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                    actual: ops_per_sec as u64,
                    reason: "File operations per second exceeded limit".to_string(),
                };
                warn!("Resource violation: {}", violation);
                return Err(LLMSpellError::Security {
                    message: violation.to_string(),
                    violation_type: Some("resource_limit".to_string()),
                });
            }
        }

        Ok(())
    }

    /// Record custom resource usage
    pub async fn record_custom_usage(&self, resource: &str, amount: u64) -> Result<()> {
        {
            let mut usage = self.current_usage.write().await;
            *usage.custom_usage.entry(resource.to_string()).or_insert(0) += amount;
            usage.timestamp = Instant::now();
        }

        // Check custom resource limits
        if let Some(limit) = self.context.resource_limits.custom_limits.get(resource) {
            let usage = self.current_usage.read().await;
            if let Some(current) = usage.custom_usage.get(resource) {
                if *current > *limit {
                    let violation = SandboxViolation::ResourceLimit {
                        resource: resource.to_string(),
                        limit: *limit,
                        actual: *current,
                        reason: format!("Custom resource '{}' exceeded limit", resource),
                    };
                    warn!("Resource violation: {}", violation);
                    return Err(LLMSpellError::Security {
                        message: violation.to_string(),
                        violation_type: Some("resource_limit".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Collect system resource usage (simplified implementation)
    async fn collect_system_usage() -> ResourceUsage {
        // In a real implementation, this would use system APIs to get actual usage
        // For testing, we'll simulate some usage
        let now = Instant::now();

        ResourceUsage {
            memory_bytes: (now.elapsed().as_secs() * 1024 * 1024).min(50 * 1024 * 1024),
            #[allow(clippy::cast_possible_truncation)]
            cpu_time_ms: now.elapsed().as_millis() as u64 / 10,
            network_bytes: 0,
            file_operations: 0,
            custom_usage: HashMap::new(),
            timestamp: now,
        }
    }

    /// Check if any violations occurred
    pub async fn has_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Get all violations
    pub async fn get_violations(&self) -> Vec<String> {
        self.violations.iter().map(|v| v.to_string()).collect()
    }

    /// Get detailed resource statistics
    pub async fn get_resource_stats(&self) -> ResourceStats {
        let usage = self.current_usage.read().await;
        let elapsed = self.start_time.elapsed();

        ResourceStats {
            current_usage: usage.clone(),
            limits: self.context.resource_limits.clone(),
            uptime_seconds: elapsed.as_secs(),
            #[allow(clippy::cast_possible_truncation)]
            violations_count: self.violations.len(),
            efficiency_metrics: EfficiencyMetrics {
                memory_efficiency: self.calculate_memory_efficiency(&usage).await,
                cpu_efficiency: self.calculate_cpu_efficiency(&usage).await,
                network_efficiency: self.calculate_network_efficiency(&usage).await,
            },
        }
    }

    /// Calculate memory efficiency (0.0 to 1.0)
    async fn calculate_memory_efficiency(&self, usage: &ResourceUsage) -> f64 {
        if let Some(limit) = self.context.resource_limits.max_memory_bytes {
            #[allow(clippy::cast_precision_loss)]
            let efficiency = 1.0 - (usage.memory_bytes as f64 / limit as f64).min(1.0);
            efficiency
        } else {
            1.0 // No limit means perfect efficiency
        }
    }

    /// Calculate CPU efficiency (0.0 to 1.0)
    async fn calculate_cpu_efficiency(&self, usage: &ResourceUsage) -> f64 {
        if let Some(limit) = self.context.resource_limits.max_cpu_time_ms {
            #[allow(clippy::cast_precision_loss)]
            let efficiency = 1.0 - (usage.cpu_time_ms as f64 / limit as f64).min(1.0);
            efficiency
        } else {
            1.0
        }
    }

    /// Calculate network efficiency (0.0 to 1.0)
    async fn calculate_network_efficiency(&self, usage: &ResourceUsage) -> f64 {
        if let Some(limit) = self.context.resource_limits.max_network_bps {
            let duration = self.start_time.elapsed().as_secs_f64().max(1.0);
            #[allow(clippy::cast_precision_loss)]
            let actual_bps = usage.network_bytes as f64 / duration;
            #[allow(clippy::cast_precision_loss)]
            let efficiency = 1.0 - (actual_bps / limit as f64).min(1.0);
            efficiency
        } else {
            1.0
        }
    }

    /// Reset resource counters
    pub async fn reset_counters(&mut self) -> Result<()> {
        {
            let mut usage = self.current_usage.write().await;
            *usage = ResourceUsage::default();
        }
        self.start_time = Instant::now();
        self.violations.clear();
        Ok(())
    }
}

/// Resource statistics
#[derive(Debug)]
pub struct ResourceStats {
    pub current_usage: ResourceUsage,
    pub limits: llmspell_core::traits::tool::ResourceLimits,
    pub uptime_seconds: u64,
    pub violations_count: usize,
    pub efficiency_metrics: EfficiencyMetrics,
}

/// Efficiency metrics
#[derive(Debug)]
pub struct EfficiencyMetrics {
    pub memory_efficiency: f64,
    pub cpu_efficiency: f64,
    pub network_efficiency: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};

    fn create_test_monitor() -> ResourceMonitor {
        let security_reqs = SecurityRequirements::safe();
        let resource_limits = ResourceLimits::strict();

        let context =
            SandboxContext::new("test-monitor".to_string(), security_reqs, resource_limits);

        ResourceMonitor::new(context).unwrap()
    }
    #[tokio::test]
    async fn test_monitor_lifecycle() {
        let mut monitor = create_test_monitor();

        // Start monitoring
        assert!(monitor.start().await.is_ok());
        assert!(monitor.is_monitoring().await);

        // Stop monitoring
        assert!(monitor.stop().await.is_ok());
        assert!(!monitor.is_monitoring().await);
    }
    #[tokio::test]
    async fn test_usage_recording() {
        let monitor = create_test_monitor();

        // Record network usage
        assert!(monitor.record_network_usage(1024).await.is_ok());

        // Record file operation
        assert!(monitor.record_file_operation().await.is_ok());

        // Record custom usage
        assert!(monitor.record_custom_usage("api_calls", 5).await.is_ok());

        // Check usage
        let usage = monitor.get_current_usage().await;
        assert_eq!(usage.network_bytes, 1024);
        assert_eq!(usage.file_operations, 1);
        assert_eq!(usage.custom_usage.get("api_calls"), Some(&5));
    }
    #[tokio::test]
    async fn test_resource_stats() {
        let monitor = create_test_monitor();

        // Record some usage
        let _ = monitor.record_network_usage(512).await;
        let _ = monitor.record_file_operation().await;

        let stats = monitor.get_resource_stats().await;
        assert_eq!(stats.current_usage.network_bytes, 512);
        assert_eq!(stats.current_usage.file_operations, 1);
        assert!(stats.efficiency_metrics.memory_efficiency >= 0.0);
        assert!(stats.efficiency_metrics.memory_efficiency <= 1.0);
    }
    #[tokio::test]
    async fn test_counter_reset() {
        let mut monitor = create_test_monitor();

        // Record some usage
        let _ = monitor.record_network_usage(1024).await;
        let _ = monitor.record_file_operation().await;

        // Reset counters
        assert!(monitor.reset_counters().await.is_ok());

        // Check that usage is reset
        let usage = monitor.get_current_usage().await;
        assert_eq!(usage.network_bytes, 0);
        assert_eq!(usage.file_operations, 0);
    }
    #[tokio::test]
    async fn test_limit_enforcement() {
        let security_reqs = SecurityRequirements::safe();
        let resource_limits = ResourceLimits::strict().with_network_limit(512); // Very low limit

        let context = SandboxContext::new("test-limit".to_string(), security_reqs, resource_limits);

        let monitor = ResourceMonitor::new(context).unwrap();

        // This should exceed the network limit
        let result = monitor.record_network_usage(1024).await;
        assert!(result.is_err());

        // Should be a security violation
        match result.unwrap_err() {
            LLMSpellError::Security { violation_type, .. } => {
                assert_eq!(violation_type, Some("resource_limit".to_string()));
            }
            _ => panic!("Expected SecurityViolation"),
        }
    }
}
