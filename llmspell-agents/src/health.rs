//! ABOUTME: Health monitoring system for agent lifecycle management and operational status tracking
//! ABOUTME: Provides comprehensive health checks, metrics collection, and alerting for agent wellness

use crate::lifecycle::{
    events::{LifecycleEvent, LifecycleEventData, LifecycleEventSystem, LifecycleEventType},
    resources::ResourceManager,
    state_machine::{AgentState, AgentStateMachine},
};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::interval;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Health status levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Everything is operating normally
    Healthy,
    /// Minor issues detected, but agent is still functional
    Warning,
    /// Significant issues, agent functionality may be degraded
    Critical,
    /// Agent is not responding or completely non-functional
    Unhealthy,
    /// Health status cannot be determined
    Unknown,
}

impl HealthStatus {
    /// Check if status indicates agent is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Warning)
    }

    /// Check if status requires immediate attention
    pub fn needs_attention(&self) -> bool {
        matches!(self, HealthStatus::Critical | HealthStatus::Unhealthy)
    }

    /// Get numeric severity (higher is worse)
    pub fn severity(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Warning => 1,
            HealthStatus::Critical => 2,
            HealthStatus::Unhealthy => 3,
            HealthStatus::Unknown => 4,
        }
    }
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Unique check ID
    pub id: String,
    /// Agent being checked
    pub agent_id: String,
    /// Type of health check
    pub check_type: String,
    /// Overall health status
    pub status: HealthStatus,
    /// Timestamp of the check
    pub timestamp: SystemTime,
    /// Duration of the health check
    pub duration: Duration,
    /// Detailed status message
    pub message: String,
    /// Additional metrics and data
    pub metrics: HashMap<String, f64>,
    /// Specific issues found
    pub issues: Vec<HealthIssue>,
    /// Recommendations for improvement
    pub recommendations: Vec<String>,
}

impl HealthCheckResult {
    pub fn new(agent_id: String, check_type: String, status: HealthStatus) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            check_type,
            status,
            timestamp: SystemTime::now(),
            duration: Duration::from_millis(0),
            message: String::new(),
            metrics: HashMap::new(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn with_metric(mut self, name: &str, value: f64) -> Self {
        self.metrics.insert(name.to_string(), value);
        self
    }

    pub fn with_issue(mut self, issue: HealthIssue) -> Self {
        self.issues.push(issue);
        self
    }

    pub fn with_recommendation(mut self, recommendation: String) -> Self {
        self.recommendations.push(recommendation);
        self
    }
}

/// Health issue details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue severity
    pub severity: HealthStatus,
    /// Issue category
    pub category: String,
    /// Description of the issue
    pub description: String,
    /// Possible causes
    pub possible_causes: Vec<String>,
    /// Suggested remediation steps
    pub remediation: Vec<String>,
}

impl HealthIssue {
    pub fn new(severity: HealthStatus, category: String, description: String) -> Self {
        Self {
            severity,
            category,
            description,
            possible_causes: Vec::new(),
            remediation: Vec::new(),
        }
    }

    pub fn with_causes(mut self, causes: Vec<String>) -> Self {
        self.possible_causes = causes;
        self
    }

    pub fn with_remediation(mut self, remediation: Vec<String>) -> Self {
        self.remediation = remediation;
        self
    }
}

/// Health check trait
#[async_trait]
pub trait HealthCheck: Send + Sync {
    /// Perform health check
    async fn check(&self, agent_id: &str) -> Result<HealthCheckResult>;

    /// Get check type name
    fn check_type(&self) -> String;

    /// Get check interval
    fn interval(&self) -> Duration {
        Duration::from_secs(60) // Default 1 minute
    }

    /// Check if this health check is enabled
    fn is_enabled(&self) -> bool {
        true
    }

    /// Get check priority (lower numbers run first)
    fn priority(&self) -> u8 {
        50
    }
}

/// Agent health monitor
pub struct AgentHealthMonitor {
    /// Agent being monitored
    agent_id: String,
    /// State machine for status checks
    state_machine: Arc<AgentStateMachine>,
    /// Resource manager for resource checks
    resource_manager: Arc<ResourceManager>,
    /// Event system for notifications
    event_system: Arc<LifecycleEventSystem>,
    /// Registered health checks
    health_checks: Arc<RwLock<Vec<Arc<dyn HealthCheck>>>>,
    /// Health check results history
    check_history: Arc<Mutex<Vec<HealthCheckResult>>>,
    /// Current overall health status
    current_status: Arc<RwLock<HealthStatus>>,
    /// Health monitoring configuration
    config: HealthMonitorConfig,
    /// Last health check timestamp
    last_check: Arc<Mutex<Option<SystemTime>>>,
}

/// Health monitor configuration
#[derive(Debug, Clone)]
pub struct HealthMonitorConfig {
    /// Enable continuous monitoring
    pub enable_monitoring: bool,
    /// Default check interval
    pub default_interval: Duration,
    /// Maximum check history size
    pub max_history_size: usize,
    /// Threshold for marking agent as unhealthy
    pub unhealthy_threshold: Duration,
    /// Enable health check logging
    pub enable_logging: bool,
    /// Alert on status changes
    pub alert_on_status_change: bool,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            enable_monitoring: true,
            default_interval: Duration::from_secs(60),
            max_history_size: 100,
            unhealthy_threshold: Duration::from_secs(300), // 5 minutes
            enable_logging: true,
            alert_on_status_change: true,
        }
    }
}

impl AgentHealthMonitor {
    /// Create new health monitor for agent
    pub fn new(
        agent_id: String,
        state_machine: Arc<AgentStateMachine>,
        resource_manager: Arc<ResourceManager>,
        event_system: Arc<LifecycleEventSystem>,
        config: HealthMonitorConfig,
    ) -> Self {
        Self {
            agent_id,
            state_machine,
            resource_manager,
            event_system,
            health_checks: Arc::new(RwLock::new(Vec::new())),
            check_history: Arc::new(Mutex::new(Vec::new())),
            current_status: Arc::new(RwLock::new(HealthStatus::Unknown)),
            config,
            last_check: Arc::new(Mutex::new(None)),
        }
    }

    /// Add health check
    pub async fn add_health_check(&self, check: Arc<dyn HealthCheck>) {
        let mut checks = self.health_checks.write().await;
        checks.push(check);

        // Sort by priority
        checks.sort_by_key(|c| c.priority());

        if self.config.enable_logging {
            debug!("Added health check for agent {}", self.agent_id);
        }
    }

    /// Perform immediate health check
    pub async fn check_health(&self) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        if self.config.enable_logging {
            debug!("Performing health check for agent {}", self.agent_id);
        }

        // Run all health checks
        let mut overall_status = HealthStatus::Healthy;
        let mut all_metrics = HashMap::new();
        let mut all_issues = Vec::new();
        let mut all_recommendations = Vec::new();
        let mut check_messages = Vec::new();

        let checks = self.health_checks.read().await;
        for check in checks.iter() {
            if !check.is_enabled() {
                continue;
            }

            match check.check(&self.agent_id).await {
                Ok(result) => {
                    // Update overall status to worst status found
                    if result.status.severity() > overall_status.severity() {
                        overall_status = result.status;
                    }

                    // Collect metrics, issues, and recommendations
                    all_metrics.extend(result.metrics);
                    all_issues.extend(result.issues);
                    all_recommendations.extend(result.recommendations);

                    if !result.message.is_empty() {
                        check_messages.push(format!("{}: {}", result.check_type, result.message));
                    }
                }
                Err(e) => {
                    warn!(
                        "Health check {} failed for agent {}: {}",
                        check.check_type(),
                        self.agent_id,
                        e
                    );
                    overall_status = HealthStatus::Unknown;
                    all_issues.push(HealthIssue::new(
                        HealthStatus::Critical,
                        "health_check".to_string(),
                        format!("Health check {} failed: {}", check.check_type(), e),
                    ));
                }
            }
        }

        // Create overall result
        let overall_message = if check_messages.is_empty() {
            format!("Health check completed with status: {:?}", overall_status)
        } else {
            check_messages.join("; ")
        };

        let result = HealthCheckResult::new(
            self.agent_id.clone(),
            "comprehensive".to_string(),
            overall_status,
        )
        .with_message(overall_message)
        .with_duration(start_time.elapsed());

        let mut final_result = result;
        for (key, value) in &all_metrics {
            final_result = final_result.with_metric(key, *value);
        }
        for issue in all_issues {
            final_result = final_result.with_issue(issue);
        }
        for recommendation in all_recommendations {
            final_result = final_result.with_recommendation(recommendation);
        }

        // Update current status
        let previous_status = {
            let mut status = self.current_status.write().await;
            let prev = *status;
            *status = overall_status;
            prev
        };

        // Record check time
        {
            let mut last_check = self.last_check.lock().await;
            *last_check = Some(SystemTime::now());
        }

        // Add to history
        {
            let mut history = self.check_history.lock().await;
            history.push(final_result.clone());

            if history.len() > self.config.max_history_size {
                history.remove(0);
            }
        }

        // Emit health check event
        if self.config.alert_on_status_change && previous_status != overall_status {
            let event = LifecycleEvent::new(
                LifecycleEventType::HealthCheck,
                self.agent_id.clone(),
                LifecycleEventData::Health {
                    is_healthy: overall_status.is_operational(),
                    status: format!("{:?}", overall_status),
                    metrics: all_metrics
                        .into_iter()
                        .map(|(k, v)| (k, v.to_string()))
                        .collect(),
                },
                "health_monitor".to_string(),
            );

            if let Err(e) = self.event_system.emit(event).await {
                warn!("Failed to emit health check event: {}", e);
            }
        }

        if self.config.enable_logging {
            match overall_status {
                HealthStatus::Healthy => {
                    debug!("Agent {} health check passed", self.agent_id);
                }
                HealthStatus::Warning => {
                    warn!("Agent {} health check found warnings", self.agent_id);
                }
                HealthStatus::Critical | HealthStatus::Unhealthy => {
                    error!(
                        "Agent {} health check failed with status {:?}",
                        self.agent_id, overall_status
                    );
                }
                HealthStatus::Unknown => {
                    warn!("Agent {} health status unknown", self.agent_id);
                }
            }
        }

        Ok(final_result)
    }

    /// Start continuous health monitoring
    pub async fn start_monitoring(&self) -> Result<()> {
        if !self.config.enable_monitoring {
            return Ok(());
        }

        let agent_id = self.agent_id.clone();
        let monitor = self.clone();

        tokio::spawn(async move {
            let mut interval_timer = interval(monitor.config.default_interval);

            info!("Started health monitoring for agent {}", agent_id);

            loop {
                interval_timer.tick().await;

                if let Err(e) = monitor.check_health().await {
                    error!("Health monitoring failed for agent {}: {}", agent_id, e);
                }
            }
        });

        Ok(())
    }

    /// Get current health status
    pub async fn get_current_status(&self) -> HealthStatus {
        *self.current_status.read().await
    }

    /// Get health check history
    pub async fn get_health_history(&self) -> Vec<HealthCheckResult> {
        let history = self.check_history.lock().await;
        history.clone()
    }

    /// Get latest health check result
    pub async fn get_latest_result(&self) -> Option<HealthCheckResult> {
        let history = self.check_history.lock().await;
        history.last().cloned()
    }

    /// Check if agent is healthy
    pub async fn is_healthy(&self) -> bool {
        self.get_current_status().await.is_operational()
    }

    /// Get time since last health check
    pub async fn time_since_last_check(&self) -> Option<Duration> {
        let last_check = self.last_check.lock().await;
        last_check.as_ref().map(|t| t.elapsed().unwrap_or_default())
    }
}

// Implement Clone manually for AgentHealthMonitor
impl Clone for AgentHealthMonitor {
    fn clone(&self) -> Self {
        Self {
            agent_id: self.agent_id.clone(),
            state_machine: self.state_machine.clone(),
            resource_manager: self.resource_manager.clone(),
            event_system: self.event_system.clone(),
            health_checks: self.health_checks.clone(),
            check_history: self.check_history.clone(),
            current_status: self.current_status.clone(),
            config: self.config.clone(),
            last_check: self.last_check.clone(),
        }
    }
}

/// Built-in health checks
/// State machine health check
pub struct StateMachineHealthCheck {
    state_machine: Arc<AgentStateMachine>,
}

impl StateMachineHealthCheck {
    pub fn new(state_machine: Arc<AgentStateMachine>) -> Self {
        Self { state_machine }
    }
}

#[async_trait]
impl HealthCheck for StateMachineHealthCheck {
    async fn check(&self, agent_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();
        let current_state = self.state_machine.current_state().await;
        let metrics = self.state_machine.get_metrics().await;

        let (status, message) = match current_state {
            AgentState::Terminated => (HealthStatus::Unhealthy, "Agent is terminated".to_string()),
            AgentState::Error => (
                HealthStatus::Critical,
                "Agent is in error state".to_string(),
            ),
            AgentState::Recovering => (
                HealthStatus::Warning,
                "Agent is recovering from error".to_string(),
            ),
            AgentState::Terminating => (HealthStatus::Warning, "Agent is terminating".to_string()),
            _ if current_state.is_healthy() => (
                HealthStatus::Healthy,
                "State machine is healthy".to_string(),
            ),
            _ => (
                HealthStatus::Unknown,
                format!("Unknown state: {:?}", current_state),
            ),
        };

        let mut result =
            HealthCheckResult::new(agent_id.to_string(), "state_machine".to_string(), status)
                .with_message(message)
                .with_duration(start_time.elapsed())
                .with_metric("total_transitions", metrics.total_transitions as f64)
                .with_metric("recovery_attempts", metrics.recovery_attempts as f64)
                .with_metric("uptime_seconds", metrics.uptime.as_secs() as f64);

        // Add issues for problematic states
        if metrics.recovery_attempts > 0 {
            result = result.with_issue(
                HealthIssue::new(
                    HealthStatus::Warning,
                    "recovery".to_string(),
                    format!("Agent has {} recovery attempts", metrics.recovery_attempts),
                )
                .with_remediation(vec![
                    "Check agent logs for error patterns".to_string(),
                    "Consider agent restart if recovery attempts are high".to_string(),
                ]),
            );
        }

        if let Some(error) = &metrics.last_error {
            result = result.with_issue(HealthIssue::new(
                HealthStatus::Critical,
                "error".to_string(),
                format!("Last error: {}", error),
            ));
        }

        Ok(result)
    }

    fn check_type(&self) -> String {
        "state_machine".to_string()
    }

    fn priority(&self) -> u8 {
        10 // High priority
    }
}

/// Resource usage health check
pub struct ResourceHealthCheck {
    resource_manager: Arc<ResourceManager>,
}

impl ResourceHealthCheck {
    pub fn new(resource_manager: Arc<ResourceManager>) -> Self {
        Self { resource_manager }
    }
}

#[async_trait]
impl HealthCheck for ResourceHealthCheck {
    async fn check(&self, agent_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();
        let allocations = self.resource_manager.get_agent_allocations(agent_id).await;
        let stats = self.resource_manager.get_usage_stats().await;

        // Check for resource leaks or excessive usage
        let allocation_count = allocations.len();
        let (status, message) = if allocation_count == 0 {
            (HealthStatus::Healthy, "No resource allocations".to_string())
        } else if allocation_count > 50 {
            (
                HealthStatus::Critical,
                format!("Excessive resource allocations: {}", allocation_count),
            )
        } else if allocation_count > 20 {
            (
                HealthStatus::Warning,
                format!("High resource allocation count: {}", allocation_count),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Resource allocations: {}", allocation_count),
            )
        };

        let mut result =
            HealthCheckResult::new(agent_id.to_string(), "resources".to_string(), status)
                .with_message(message)
                .with_duration(start_time.elapsed())
                .with_metric("allocation_count", allocation_count as f64)
                .with_metric("total_allocations", stats.total_allocations as f64);

        // Add resource-specific metrics
        for (resource_type, usage) in &stats.current_usage_by_type {
            result = result.with_metric(&format!("usage_{}", resource_type.name()), *usage as f64);
        }

        // Add recommendations for high resource usage
        if allocation_count > 20 {
            result = result
                .with_recommendation("Consider reviewing resource allocation patterns".to_string());
        }

        Ok(result)
    }

    fn check_type(&self) -> String {
        "resources".to_string()
    }

    fn priority(&self) -> u8 {
        20 // Medium priority
    }
}

/// Responsiveness health check
pub struct ResponsivenessHealthCheck;

#[async_trait]
impl HealthCheck for ResponsivenessHealthCheck {
    async fn check(&self, agent_id: &str) -> Result<HealthCheckResult> {
        let start_time = Instant::now();

        // Simple ping test - measure response time
        tokio::time::sleep(Duration::from_millis(1)).await;

        let response_time = start_time.elapsed();
        let response_ms = response_time.as_millis() as f64;

        let (status, message) = if response_ms > 1000.0 {
            (
                HealthStatus::Critical,
                format!("Very slow response: {:.1}ms", response_ms),
            )
        } else if response_ms > 100.0 {
            (
                HealthStatus::Warning,
                format!("Slow response: {:.1}ms", response_ms),
            )
        } else {
            (
                HealthStatus::Healthy,
                format!("Good response time: {:.1}ms", response_ms),
            )
        };

        Ok(
            HealthCheckResult::new(agent_id.to_string(), "responsiveness".to_string(), status)
                .with_message(message)
                .with_duration(response_time)
                .with_metric("response_time_ms", response_ms),
        )
    }

    fn check_type(&self) -> String {
        "responsiveness".to_string()
    }

    fn interval(&self) -> Duration {
        Duration::from_secs(30) // Check every 30 seconds
    }

    fn priority(&self) -> u8 {
        30 // Lower priority
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::{events::EventSystemConfig, resources::ResourceLimits};

    #[tokio::test]
    async fn test_health_monitor_basic() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            ResourceLimits::default(),
            event_system.clone(),
        ));
        let state_machine = Arc::new(AgentStateMachine::default("test-agent".to_string()));
        state_machine.initialize().await.unwrap();

        let monitor = AgentHealthMonitor::new(
            "test-agent".to_string(),
            state_machine.clone(),
            resource_manager.clone(),
            event_system,
            HealthMonitorConfig::default(),
        );

        monitor
            .add_health_check(Arc::new(StateMachineHealthCheck::new(state_machine)))
            .await;
        monitor
            .add_health_check(Arc::new(ResourceHealthCheck::new(resource_manager)))
            .await;
        monitor
            .add_health_check(Arc::new(ResponsivenessHealthCheck))
            .await;

        let result = monitor.check_health().await.unwrap();
        assert_eq!(result.agent_id, "test-agent");
        assert_eq!(result.check_type, "comprehensive");
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_status_severity() {
        assert_eq!(HealthStatus::Healthy.severity(), 0);
        assert_eq!(HealthStatus::Warning.severity(), 1);
        assert_eq!(HealthStatus::Critical.severity(), 2);
        assert_eq!(HealthStatus::Unhealthy.severity(), 3);
        assert_eq!(HealthStatus::Unknown.severity(), 4);

        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Warning.is_operational());
        assert!(!HealthStatus::Critical.is_operational());

        assert!(HealthStatus::Critical.needs_attention());
        assert!(HealthStatus::Unhealthy.needs_attention());
        assert!(!HealthStatus::Healthy.needs_attention());
    }

    #[tokio::test]
    async fn test_state_machine_health_check() {
        let state_machine = Arc::new(AgentStateMachine::default("test-agent".to_string()));
        state_machine.initialize().await.unwrap();

        let check = StateMachineHealthCheck::new(state_machine.clone());
        let result = check.check("test-agent").await.unwrap();

        assert_eq!(result.check_type, "state_machine");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.metrics.contains_key("total_transitions"));

        // Test error state
        state_machine.error("Test error".to_string()).await.unwrap();
        let result = check.check("test-agent").await.unwrap();
        assert_eq!(result.status, HealthStatus::Critical);
    }

    #[tokio::test]
    async fn test_resource_health_check() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let resource_manager = Arc::new(ResourceManager::new(
            ResourceLimits::default(),
            event_system,
        ));

        let check = ResourceHealthCheck::new(resource_manager);
        let result = check.check("test-agent").await.unwrap();

        assert_eq!(result.check_type, "resources");
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.metrics.contains_key("allocation_count"));
    }

    #[tokio::test]
    async fn test_responsiveness_health_check() {
        let check = ResponsivenessHealthCheck;
        let result = check.check("test-agent").await.unwrap();

        assert_eq!(result.check_type, "responsiveness");
        assert!(result.metrics.contains_key("response_time_ms"));
        // Should be healthy for a simple test
        assert_eq!(result.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_health_check_result_builder() {
        let result = HealthCheckResult::new(
            "test-agent".to_string(),
            "test".to_string(),
            HealthStatus::Warning,
        )
        .with_message("Test warning".to_string())
        .with_duration(Duration::from_millis(100))
        .with_metric("test_metric", 42.0)
        .with_issue(HealthIssue::new(
            HealthStatus::Warning,
            "test".to_string(),
            "Test issue".to_string(),
        ))
        .with_recommendation("Test recommendation".to_string());

        assert_eq!(result.agent_id, "test-agent");
        assert_eq!(result.status, HealthStatus::Warning);
        assert_eq!(result.message, "Test warning");
        assert_eq!(result.metrics.get("test_metric"), Some(&42.0));
        assert_eq!(result.issues.len(), 1);
        assert_eq!(result.recommendations.len(), 1);
    }
}
