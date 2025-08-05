//! ABOUTME: Health monitoring for agents and their components
//! ABOUTME: Provides health checks, status reporting, and component health aggregation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::{ComponentMetadata, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

/// Health status levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Component is healthy and functioning normally
    Healthy,
    /// Component is degraded but still functional
    Degraded,
    /// Component is unhealthy and may not be functioning
    Unhealthy,
    /// Component health is unknown
    Unknown,
}

impl HealthStatus {
    /// Check if the status indicates the component is operational
    #[must_use]
    pub const fn is_operational(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Convert to a numeric score (0-100)
    #[must_use]
    pub const fn score(&self) -> u8 {
        match self {
            Self::Healthy => 100,
            Self::Degraded => 50,
            Self::Unhealthy => 0,
            Self::Unknown => 25,
        }
    }
}

/// Health indicator for a specific aspect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIndicator {
    /// Name of the indicator
    pub name: String,
    /// Current status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Optional details
    pub details: HashMap<String, serde_json::Value>,
    /// Last check time
    pub last_check: DateTime<Utc>,
}

impl HealthIndicator {
    /// Create a healthy indicator
    #[must_use]
    pub fn healthy(name: String) -> Self {
        Self {
            name,
            status: HealthStatus::Healthy,
            message: None,
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Create an unhealthy indicator
    #[must_use]
    pub fn unhealthy(name: String, message: String) -> Self {
        Self {
            name,
            status: HealthStatus::Unhealthy,
            message: Some(message),
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Create a degraded indicator
    #[must_use]
    pub fn degraded(name: String, message: String) -> Self {
        Self {
            name,
            status: HealthStatus::Degraded,
            message: Some(message),
            details: HashMap::new(),
            last_check: Utc::now(),
        }
    }

    /// Add detail to the indicator
    #[must_use]
    pub fn with_detail(mut self, key: String, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }
}

/// Component health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    /// Component metadata
    pub metadata: ComponentMetadata,
    /// Overall status
    pub status: HealthStatus,
    /// Individual health indicators
    pub indicators: Vec<HealthIndicator>,
    /// Timestamp of last health check
    pub last_check: DateTime<Utc>,
    /// Check duration
    pub check_duration: Duration,
}

impl ComponentHealth {
    /// Calculate overall status from indicators
    #[must_use]
    pub fn calculate_status(indicators: &[HealthIndicator]) -> HealthStatus {
        if indicators.is_empty() {
            return HealthStatus::Unknown;
        }

        let has_unhealthy = indicators
            .iter()
            .any(|i| i.status == HealthStatus::Unhealthy);

        if has_unhealthy {
            return HealthStatus::Unhealthy;
        }

        let has_degraded = indicators
            .iter()
            .any(|i| i.status == HealthStatus::Degraded);

        if has_degraded {
            return HealthStatus::Degraded;
        }

        let all_healthy = indicators.iter().all(|i| i.status == HealthStatus::Healthy);

        if all_healthy {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform a health check
    /// 
    /// # Errors
    /// 
    /// Returns an error if the health check cannot be performed due to system failures,
    /// resource unavailability, or other critical issues that prevent health assessment.
    async fn check_health(&self) -> Result<Vec<HealthIndicator>>;

    /// Get component metadata
    fn metadata(&self) -> &ComponentMetadata;
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall system health
    pub overall_status: HealthStatus,
    /// Individual component health
    pub components: HashMap<String, ComponentHealth>,
    /// Total check duration
    pub total_duration: Duration,
    /// Timestamp of the check
    pub timestamp: DateTime<Utc>,
}

impl HealthCheckResult {
    /// Create a new health check result
    #[must_use]
    pub fn new(components: HashMap<String, ComponentHealth>) -> Self {
        let overall_status = Self::calculate_overall_status(&components);

        let total_duration = components.values().map(|c| c.check_duration).sum();

        Self {
            overall_status,
            components,
            total_duration,
            timestamp: Utc::now(),
        }
    }

    /// Calculate overall status from component statuses
    fn calculate_overall_status(components: &HashMap<String, ComponentHealth>) -> HealthStatus {
        if components.is_empty() {
            return HealthStatus::Unknown;
        }

        let statuses: Vec<_> = components.values().map(|c| &c.status).collect();

        // If any component is unhealthy, system is unhealthy
        if statuses.iter().any(|s| **s == HealthStatus::Unhealthy) {
            return HealthStatus::Unhealthy;
        }

        // If any component is degraded, system is degraded
        if statuses.iter().any(|s| **s == HealthStatus::Degraded) {
            return HealthStatus::Degraded;
        }

        // If all components are healthy, system is healthy
        if statuses.iter().all(|s| **s == HealthStatus::Healthy) {
            return HealthStatus::Healthy;
        }

        // Otherwise unknown
        HealthStatus::Unknown
    }

    /// Get a summary of the health check
    #[must_use]
    pub fn summary(&self) -> String {
        let healthy_count = self
            .components
            .values()
            .filter(|c| c.status == HealthStatus::Healthy)
            .count();

        let total_count = self.components.len();

        format!(
            "Overall: {:?}, {}/{} components healthy ({}ms)",
            self.overall_status,
            healthy_count,
            total_count,
            self.total_duration.as_millis()
        )
    }
}

/// Health monitor for managing health checks
pub struct HealthMonitor {
    /// Registered health checks
    checks: Vec<Arc<dyn HealthCheck>>,
    /// Check interval
    check_interval: Duration,
    /// Timeout for individual checks
    check_timeout: Duration,
}

impl HealthMonitor {
    /// Create a new health monitor
    #[must_use]
    pub fn new(check_interval: Duration, check_timeout: Duration) -> Self {
        Self {
            checks: Vec::new(),
            check_interval,
            check_timeout,
        }
    }

    /// Register a health check
    pub fn register(&mut self, check: Arc<dyn HealthCheck>) {
        self.checks.push(check);
    }

    /// Perform all health checks
    /// 
    /// # Errors
    /// 
    /// Currently never returns an error as it handles all individual health check failures
    /// internally and converts them to unhealthy indicators. The Result type is provided
    /// for future extensibility (e.g., system-level health check failures).
    pub async fn check_all(&self) -> Result<HealthCheckResult> {
        let mut components = HashMap::new();
        let _start = std::time::Instant::now();

        for check in &self.checks {
            let check_start = std::time::Instant::now();

            // Perform check with timeout
            let indicators =
                match tokio::time::timeout(self.check_timeout, check.check_health()).await {
                    Ok(Ok(indicators)) => indicators,
                    Ok(Err(e)) => vec![HealthIndicator::unhealthy(
                        "check_failed".to_string(),
                        format!("Health check failed: {e}"),
                    )],
                    Err(_) => vec![HealthIndicator::unhealthy(
                        "check_timeout".to_string(),
                        format!("Health check timed out after {:?}", self.check_timeout),
                    )],
                };

            let status = ComponentHealth::calculate_status(&indicators);
            let component_health = ComponentHealth {
                metadata: check.metadata().clone(),
                status,
                indicators,
                last_check: Utc::now(),
                check_duration: check_start.elapsed(),
            };

            components.insert(check.metadata().id.to_string(), component_health);
        }

        Ok(HealthCheckResult::new(components))
    }

    /// Start periodic health monitoring
    pub async fn start_monitoring(self: Arc<Self>) {
        let monitor = self;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(monitor.check_interval);
            loop {
                interval.tick().await;

                match monitor.check_all().await {
                    Ok(result) => {
                        tracing::info!("Health check completed: {}", result.summary());

                        // Log any unhealthy components
                        for (id, health) in &result.components {
                            if health.status == HealthStatus::Unhealthy {
                                tracing::warn!(
                                    "Component {} is unhealthy: {:?}",
                                    id,
                                    health.indicators
                                );
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Health check failed: {}", e);
                    }
                }
            }
        });
    }
}

/// Agent health check implementation
pub struct AgentHealthCheck {
    metadata: ComponentMetadata,
    /// Minimum free memory (bytes)
    min_memory: u64,
    /// Maximum CPU usage (percentage)
    max_cpu: f64,
    /// Maximum response time (milliseconds)
    max_response_time: u64,
    /// Minimum success rate (percentage)
    min_success_rate: f64,
}

impl AgentHealthCheck {
    /// Create a new agent health check
    #[must_use]
    pub const fn new(metadata: ComponentMetadata) -> Self {
        Self {
            metadata,
            min_memory: 100 * 1024 * 1024, // 100MB
            max_cpu: 80.0,                 // 80%
            max_response_time: 5000,       // 5 seconds
            min_success_rate: 95.0,        // 95%
        }
    }

    /// Configure minimum memory requirement
    #[must_use]
    pub const fn with_min_memory(mut self, bytes: u64) -> Self {
        self.min_memory = bytes;
        self
    }

    /// Configure maximum CPU usage
    #[must_use]
    pub const fn with_max_cpu(mut self, percent: f64) -> Self {
        self.max_cpu = percent;
        self
    }

    /// Configure maximum response time
    #[must_use]
    pub const fn with_max_response_time(mut self, millis: u64) -> Self {
        self.max_response_time = millis;
        self
    }

    /// Configure minimum success rate
    #[must_use]
    pub const fn with_min_success_rate(mut self, percent: f64) -> Self {
        self.min_success_rate = percent;
        self
    }
}

#[async_trait]
impl HealthCheck for AgentHealthCheck {
    async fn check_health(&self) -> Result<Vec<HealthIndicator>> {
        let mut indicators = Vec::new();

        // Check memory usage
        // In a real implementation, this would query actual system metrics
        let memory_available = 200 * 1024 * 1024; // Mock: 200MB available
        let memory_indicator = if memory_available >= self.min_memory {
            HealthIndicator::healthy("memory".to_string())
                .with_detail(
                    "available_bytes".to_string(),
                    serde_json::json!(memory_available),
                )
                .with_detail("min_bytes".to_string(), serde_json::json!(self.min_memory))
        } else {
            HealthIndicator::unhealthy(
                "memory".to_string(),
                format!(
                    "Insufficient memory: {} bytes available, {} required",
                    memory_available, self.min_memory
                ),
            )
        };
        indicators.push(memory_indicator);

        // Check CPU usage
        let cpu_usage = 25.0; // Mock: 25%
        let cpu_indicator = if cpu_usage <= self.max_cpu {
            HealthIndicator::healthy("cpu".to_string())
                .with_detail("usage_percent".to_string(), serde_json::json!(cpu_usage))
                .with_detail("max_percent".to_string(), serde_json::json!(self.max_cpu))
        } else {
            HealthIndicator::degraded(
                "cpu".to_string(),
                format!("CPU usage high: {cpu_usage:.1}%"),
            )
        };
        indicators.push(cpu_indicator);

        // Check response time
        let avg_response_time = 1200; // Mock: 1.2 seconds
        let response_indicator = if avg_response_time <= self.max_response_time {
            HealthIndicator::healthy("response_time".to_string())
                .with_detail("avg_ms".to_string(), serde_json::json!(avg_response_time))
                .with_detail(
                    "max_ms".to_string(),
                    serde_json::json!(self.max_response_time),
                )
        } else {
            HealthIndicator::degraded(
                "response_time".to_string(),
                format!("Response time slow: {avg_response_time}ms"),
            )
        };
        indicators.push(response_indicator);

        // Check success rate
        let success_rate = 98.5; // Mock: 98.5%
        let success_indicator = if success_rate >= self.min_success_rate {
            HealthIndicator::healthy("success_rate".to_string())
                .with_detail("rate_percent".to_string(), serde_json::json!(success_rate))
                .with_detail(
                    "min_percent".to_string(),
                    serde_json::json!(self.min_success_rate),
                )
        } else {
            HealthIndicator::unhealthy(
                "success_rate".to_string(),
                format!("Success rate low: {success_rate:.1}%"),
            )
        };
        indicators.push(success_indicator);

        Ok(indicators)
    }

    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_health_status_score() {
        assert_eq!(HealthStatus::Healthy.score(), 100);
        assert_eq!(HealthStatus::Degraded.score(), 50);
        assert_eq!(HealthStatus::Unhealthy.score(), 0);
        assert_eq!(HealthStatus::Unknown.score(), 25);
    }
    #[test]
    fn test_health_indicator_builders() {
        let healthy = HealthIndicator::healthy("test".to_string());
        assert_eq!(healthy.status, HealthStatus::Healthy);
        assert!(healthy.message.is_none());

        let unhealthy = HealthIndicator::unhealthy("test".to_string(), "Error".to_string());
        assert_eq!(unhealthy.status, HealthStatus::Unhealthy);
        assert_eq!(unhealthy.message, Some("Error".to_string()));

        let degraded = HealthIndicator::degraded("test".to_string(), "Warning".to_string())
            .with_detail("count".to_string(), serde_json::json!(42));
        assert_eq!(degraded.status, HealthStatus::Degraded);
        assert_eq!(degraded.details.get("count"), Some(&serde_json::json!(42)));
    }
    #[test]
    fn test_component_health_status_calculation() {
        let indicators = vec![
            HealthIndicator::healthy("cpu".to_string()),
            HealthIndicator::healthy("memory".to_string()),
        ];
        assert_eq!(
            ComponentHealth::calculate_status(&indicators),
            HealthStatus::Healthy
        );

        let indicators = vec![
            HealthIndicator::healthy("cpu".to_string()),
            HealthIndicator::degraded("memory".to_string(), "High usage".to_string()),
        ];
        assert_eq!(
            ComponentHealth::calculate_status(&indicators),
            HealthStatus::Degraded
        );

        let indicators = vec![
            HealthIndicator::healthy("cpu".to_string()),
            HealthIndicator::unhealthy("memory".to_string(), "Out of memory".to_string()),
        ];
        assert_eq!(
            ComponentHealth::calculate_status(&indicators),
            HealthStatus::Unhealthy
        );
    }
    #[test]
    fn test_health_check_result() {
        let mut components = HashMap::new();

        components.insert(
            "agent1".to_string(),
            ComponentHealth {
                metadata: ComponentMetadata::new("agent1".to_string(), "Agent 1".to_string()),
                status: HealthStatus::Healthy,
                indicators: vec![HealthIndicator::healthy("test".to_string())],
                last_check: Utc::now(),
                check_duration: Duration::from_millis(10),
            },
        );

        components.insert(
            "agent2".to_string(),
            ComponentHealth {
                metadata: ComponentMetadata::new("agent2".to_string(), "Agent 2".to_string()),
                status: HealthStatus::Degraded,
                indicators: vec![HealthIndicator::degraded(
                    "test".to_string(),
                    "Warning".to_string(),
                )],
                last_check: Utc::now(),
                check_duration: Duration::from_millis(20),
            },
        );

        let result = HealthCheckResult::new(components);
        assert_eq!(result.overall_status, HealthStatus::Degraded);
        assert_eq!(result.total_duration, Duration::from_millis(30));
        assert!(result.summary().contains("1/2 components healthy"));
    }
    #[tokio::test]
    async fn test_agent_health_check() {
        let metadata = ComponentMetadata::new("test-agent".to_string(), "Test Agent".to_string());
        let check = AgentHealthCheck::new(metadata.clone())
            .with_min_memory(10 * 1024 * 1024) // 10MB
            .with_max_cpu(90.0)
            .with_max_response_time(10000)
            .with_min_success_rate(90.0);

        let indicators = check.check_health().await.unwrap();
        assert_eq!(indicators.len(), 4);

        // All should be healthy with our mock values
        for indicator in &indicators {
            assert!(indicator.status.is_operational());
        }
    }
}
