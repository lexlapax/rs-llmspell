//! ABOUTME: Alerting framework for monitoring agent health and performance
//! ABOUTME: Provides alert rules, notification channels, and alert management

use crate::monitoring::{
    health::HealthStatus, metrics::MetricValue, performance::PerformanceViolation,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::Duration;

/// Alert severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Critical alert
    Critical,
    /// Emergency alert
    Emergency,
}

impl AlertSeverity {
    /// Get color code for severity
    #[must_use]
    pub const fn color(&self) -> &'static str {
        match self {
            Self::Info => "🟢",
            Self::Warning => "🟡",
            Self::Critical => "🟠",
            Self::Emergency => "🔴",
        }
    }
}

impl std::fmt::Display for AlertSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Warning => write!(f, "Warning"),
            Self::Critical => write!(f, "Critical"),
            Self::Emergency => write!(f, "Emergency"),
        }
    }
}

/// Alert state
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertState {
    /// Alert is active
    Active,
    /// Alert is acknowledged
    Acknowledged,
    /// Alert is resolved
    Resolved,
    /// Alert is silenced
    Silenced,
}

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Rule that triggered the alert
    pub rule_id: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert state
    pub state: AlertState,
    /// Agent ID
    pub agent_id: String,
    /// Alert title
    pub title: String,
    /// Alert description
    pub description: String,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
    /// Timestamp when alert was triggered
    pub triggered_at: DateTime<Utc>,
    /// Timestamp when alert was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when alert was resolved (if applicable)
    pub resolved_at: Option<DateTime<Utc>>,
}

impl Alert {
    /// Create a new alert
    #[must_use]
    pub fn new(
        rule_id: String,
        severity: AlertSeverity,
        agent_id: String,
        title: String,
        description: String,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            rule_id,
            severity,
            state: AlertState::Active,
            agent_id,
            title,
            description,
            details: HashMap::new(),
            triggered_at: now,
            updated_at: now,
            resolved_at: None,
        }
    }

    /// Add detail to the alert
    #[must_use]
    pub fn with_detail(mut self, key: String, value: serde_json::Value) -> Self {
        self.details.insert(key, value);
        self
    }

    /// Acknowledge the alert
    pub fn acknowledge(&mut self) {
        self.state = AlertState::Acknowledged;
        self.updated_at = Utc::now();
    }

    /// Resolve the alert
    pub fn resolve(&mut self) {
        self.state = AlertState::Resolved;
        self.resolved_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// Silence the alert
    pub fn silence(&mut self) {
        self.state = AlertState::Silenced;
        self.updated_at = Utc::now();
    }

    /// Get alert duration
    #[must_use]
    pub fn duration(&self) -> Duration {
        if let Some(resolved_at) = self.resolved_at {
            (resolved_at - self.triggered_at)
                .to_std()
                .unwrap_or_default()
        } else {
            (Utc::now() - self.triggered_at)
                .to_std()
                .unwrap_or_default()
        }
    }
}

/// Alert rule configuration
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// Rule ID
    pub id: String,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Condition that triggers the alert
    pub condition: AlertCondition,
    /// Cool down period before re-triggering
    pub cooldown: Duration,
    /// Whether the rule is enabled
    pub enabled: bool,
    /// Notification channels
    pub channels: Vec<String>,
}

/// Alert conditions
#[derive(Clone)]
pub enum AlertCondition {
    /// Metric threshold condition
    MetricThreshold {
        metric_name: String,
        operator: ThresholdOperator,
        threshold: f64,
        duration: Duration,
    },
    /// Health status condition
    HealthStatus {
        status: HealthStatus,
        duration: Duration,
    },
    /// Error rate condition
    ErrorRate {
        rate_percent: f64,
        duration: Duration,
    },
    /// Custom condition with evaluation function
    Custom(Arc<dyn AlertEvaluator>),
}

impl std::fmt::Debug for AlertCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MetricThreshold {
                metric_name,
                operator,
                threshold,
                duration,
            } => f
                .debug_struct("MetricThreshold")
                .field("metric_name", metric_name)
                .field("operator", operator)
                .field("threshold", threshold)
                .field("duration", duration)
                .finish(),
            Self::HealthStatus { status, duration } => f
                .debug_struct("HealthStatus")
                .field("status", status)
                .field("duration", duration)
                .finish(),
            Self::ErrorRate {
                rate_percent,
                duration,
            } => f
                .debug_struct("ErrorRate")
                .field("rate_percent", rate_percent)
                .field("duration", duration)
                .finish(),
            Self::Custom(_) => f.write_str("Custom(AlertEvaluator)"),
        }
    }
}

/// Threshold operators
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThresholdOperator {
    /// Greater than
    GreaterThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal
    Equal,
    /// Not equal
    NotEqual,
}

impl ThresholdOperator {
    /// Evaluate a value against a threshold
    #[must_use]
    pub fn evaluate(&self, value: f64, threshold: f64) -> bool {
        match self {
            Self::GreaterThan => value > threshold,
            Self::GreaterThanOrEqual => value >= threshold,
            Self::LessThan => value < threshold,
            Self::LessThanOrEqual => value <= threshold,
            Self::Equal => (value - threshold).abs() < f64::EPSILON,
            Self::NotEqual => (value - threshold).abs() >= f64::EPSILON,
        }
    }
}

/// Trait for custom alert evaluation
pub trait AlertEvaluator: Send + Sync {
    /// Evaluate if alert should be triggered
    fn evaluate(&self, context: &AlertContext) -> bool;
}

/// Context for alert evaluation
#[derive(Debug)]
pub struct AlertContext<'a> {
    /// Current metrics
    pub metrics: &'a HashMap<String, MetricValue>,
    /// Current health status
    pub health: Option<&'a HealthStatus>,
    /// Recent performance violations
    pub performance_violations: &'a [PerformanceViolation],
    /// Agent ID
    pub agent_id: &'a str,
}

/// Alert configuration
#[derive(Debug, Clone)]
pub struct AlertConfig {
    /// Maximum active alerts
    pub max_active_alerts: usize,
    /// Alert history retention
    pub history_retention: Duration,
    /// Default cooldown period
    pub default_cooldown: Duration,
    /// Notification retry attempts
    pub notification_retries: u32,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            max_active_alerts: 1000,
            history_retention: Duration::from_secs(86400), // 24 hours
            default_cooldown: Duration::from_secs(300),    // 5 minutes
            notification_retries: 3,
        }
    }
}

/// Alert manager for managing alerts and rules
pub struct AlertManager {
    /// Configuration
    config: AlertConfig,
    /// Alert rules
    rules: Arc<RwLock<HashMap<String, AlertRule>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert history
    history: Arc<RwLock<VecDeque<Alert>>>,
    /// Last trigger times for rules (for cooldown)
    last_trigger_times: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// Notification channels
    channels: Arc<RwLock<HashMap<String, Arc<dyn NotificationChannel>>>>,
}

impl AlertManager {
    /// Create a new alert manager
    #[must_use]
    pub fn new(config: AlertConfig) -> Self {
        Self {
            config,
            rules: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(VecDeque::new())),
            last_trigger_times: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register an alert rule
    pub fn register_rule(&self, rule: AlertRule) {
        self.rules.write().unwrap().insert(rule.id.clone(), rule);
    }

    /// Register a notification channel
    pub fn register_channel(&self, name: String, channel: Arc<dyn NotificationChannel>) {
        self.channels.write().unwrap().insert(name, channel);
    }

    /// Evaluate all rules
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rule evaluation fails
    /// - Alert triggering fails
    /// - Notification sending fails
    pub async fn evaluate_rules(&self, context: AlertContext<'_>) -> Result<()> {
        let rules_to_evaluate: Vec<AlertRule> = {
            let rules = self.rules.read().unwrap();
            rules.values().filter(|r| r.enabled).cloned().collect()
        };

        for rule in rules_to_evaluate {
            // Check cooldown
            if self.is_in_cooldown(&rule.id) {
                continue;
            }

            // Evaluate condition
            let should_trigger = match &rule.condition {
                AlertCondition::MetricThreshold {
                    metric_name,
                    operator,
                    threshold,
                    ..
                } => {
                    if let Some(metric) = context.metrics.get(metric_name) {
                        match metric {
                            MetricValue::Counter(v) => operator.evaluate(*v as f64, *threshold),
                            MetricValue::Gauge(v) => operator.evaluate(*v, *threshold),
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                AlertCondition::HealthStatus { status, .. } => context.health == Some(status),
                AlertCondition::ErrorRate { rate_percent, .. } => {
                    // Calculate error rate from metrics
                    if let (Some(total), Some(failed)) = (
                        context.metrics.get("requests_total"),
                        context.metrics.get("requests_failed"),
                    ) {
                        match (total, failed) {
                            (MetricValue::Counter(t), MetricValue::Counter(f)) => {
                                if *t > 0 {
                                    let rate = (*f as f64 / *t as f64) * 100.0;
                                    rate >= *rate_percent
                                } else {
                                    false
                                }
                            }
                            _ => false,
                        }
                    } else {
                        false
                    }
                }
                AlertCondition::Custom(evaluator) => evaluator.evaluate(&context),
            };

            if should_trigger {
                self.trigger_alert(rule, context.agent_id).await?;
            }
        }

        Ok(())
    }

    /// Check if a rule is in cooldown
    fn is_in_cooldown(&self, rule_id: &str) -> bool {
        let last_triggers = self.last_trigger_times.read().unwrap();
        if let Some(last_trigger) = last_triggers.get(rule_id) {
            let elapsed = (Utc::now() - *last_trigger).to_std().unwrap_or_default();
            elapsed < self.config.default_cooldown
        } else {
            false
        }
    }

    /// Trigger an alert
    async fn trigger_alert(&self, rule: AlertRule, agent_id: &str) -> Result<()> {
        // Create alert
        let alert = Alert::new(
            rule.id.clone(),
            rule.severity,
            agent_id.to_string(),
            rule.name.clone(),
            rule.description.clone(),
        );

        // Store alert
        let alert_id = alert.id.clone();
        self.active_alerts
            .write()
            .unwrap()
            .insert(alert_id.clone(), alert.clone());

        // Update last trigger time
        self.last_trigger_times
            .write()
            .unwrap()
            .insert(rule.id.clone(), Utc::now());

        // Send notifications
        self.send_notifications(&alert, &rule.channels).await?;

        // Add to history
        self.add_to_history(alert);

        Ok(())
    }

    /// Send notifications for an alert
    async fn send_notifications(&self, alert: &Alert, channel_names: &[String]) -> Result<()> {
        for channel_name in channel_names {
            // Get the channel Arc, if it exists
            let channel = {
                let channels = self.channels.read().unwrap();
                channels.get(channel_name).cloned()
            };

            if let Some(channel) = channel {
                // Now we can use the channel without holding the lock
                for attempt in 0..self.config.notification_retries {
                    match channel.notify(alert).await {
                        Ok(()) => break,
                        Err(e) => {
                            if attempt == self.config.notification_retries - 1 {
                                tracing::error!(
                                    "Failed to send alert {} via {} after {} attempts: {}",
                                    alert.id,
                                    channel_name,
                                    self.config.notification_retries,
                                    e
                                );
                            }
                        }
                    }
                }
            } else {
                tracing::warn!("Channel {} not found", channel_name);
            }
        }

        Ok(())
    }

    /// Add alert to history
    fn add_to_history(&self, alert: Alert) {
        let mut history = self.history.write().unwrap();

        // Remove old alerts
        let cutoff = Utc::now()
            - chrono::Duration::from_std(self.config.history_retention).unwrap_or_default();
        history.retain(|a| a.triggered_at > cutoff);

        // Add new alert
        history.push_back(alert);
    }

    /// Get active alerts
    #[must_use]
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        self.active_alerts
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Acknowledge an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn acknowledge_alert(&self, alert_id: &str) -> Result<()> {
        if let Some(alert) = self.active_alerts.write().unwrap().get_mut(alert_id) {
            alert.acknowledge();
            Ok(())
        } else {
            Err(llmspell_core::LLMSpellError::Component {
                message: format!("Alert {alert_id} not found"),
                source: None,
            })
        }
    }

    /// Resolve an alert
    ///
    /// # Errors
    ///
    /// Returns an error if the alert is not found
    pub fn resolve_alert(&self, alert_id: &str) -> Result<()> {
        if let Some(mut alert) = self.active_alerts.write().unwrap().remove(alert_id) {
            alert.resolve();
            self.add_to_history(alert);
            Ok(())
        } else {
            Err(llmspell_core::LLMSpellError::Component {
                message: format!("Alert {alert_id} not found"),
                source: None,
            })
        }
    }

    /// Get alert statistics
    #[must_use]
    pub fn get_statistics(&self) -> AlertStatistics {
        let active = self.active_alerts.read().unwrap();
        let history = self.history.read().unwrap();

        let mut severity_counts = HashMap::new();
        for alert in active.values() {
            *severity_counts.entry(alert.severity).or_insert(0) += 1;
        }

        let avg_resolution_time = if history.is_empty() {
            None
        } else {
            let total_duration: Duration = history
                .iter()
                .filter(|a| a.resolved_at.is_some())
                .map(Alert::duration)
                .sum();
            let resolved_count = history.iter().filter(|a| a.resolved_at.is_some()).count();

            if resolved_count > 0 {
                Some(total_duration / u32::try_from(resolved_count).unwrap_or(1))
            } else {
                None
            }
        };

        AlertStatistics {
            active_count: active.len(),
            severity_counts,
            total_triggered: history.len(),
            avg_resolution_time,
        }
    }
}

/// Alert statistics
#[derive(Debug)]
pub struct AlertStatistics {
    /// Number of active alerts
    pub active_count: usize,
    /// Count by severity
    pub severity_counts: HashMap<AlertSeverity, usize>,
    /// Total alerts triggered
    pub total_triggered: usize,
    /// Average resolution time
    pub avg_resolution_time: Option<Duration>,
}

/// Trait for notification channels
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send notification for an alert
    async fn notify(&self, alert: &Alert) -> Result<()>;
}

/// Console notification channel (for testing)
#[derive(Debug)]
pub struct ConsoleNotificationChannel;

#[async_trait]
impl NotificationChannel for ConsoleNotificationChannel {
    async fn notify(&self, alert: &Alert) -> Result<()> {
        println!(
            "{} ALERT [{}]: {} - {}",
            alert.severity.color(),
            alert.agent_id,
            alert.title,
            alert.description
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_threshold_operators() {
        assert!(ThresholdOperator::GreaterThan.evaluate(10.0, 5.0));
        assert!(!ThresholdOperator::GreaterThan.evaluate(5.0, 10.0));
        assert!(ThresholdOperator::LessThan.evaluate(5.0, 10.0));
        assert!(!ThresholdOperator::LessThan.evaluate(10.0, 5.0));
        assert!(ThresholdOperator::Equal.evaluate(5.0, 5.0));
        assert!(!ThresholdOperator::Equal.evaluate(5.0, 6.0));
    }
    #[test]
    fn test_alert_creation() {
        let mut alert = Alert::new(
            "rule-1".to_string(),
            AlertSeverity::Warning,
            "agent-1".to_string(),
            "High CPU Usage".to_string(),
            "CPU usage exceeded 80%".to_string(),
        )
        .with_detail("cpu_percent".to_string(), serde_json::json!(85.5));

        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.state, AlertState::Active);
        assert!(alert.resolved_at.is_none());

        // Test state transitions
        alert.acknowledge();
        assert_eq!(alert.state, AlertState::Acknowledged);

        alert.resolve();
        assert_eq!(alert.state, AlertState::Resolved);
        assert!(alert.resolved_at.is_some());
    }
    #[test]
    fn test_alert_rule() {
        let rule = AlertRule {
            id: "cpu-high".to_string(),
            name: "High CPU Usage".to_string(),
            description: "Alert when CPU usage is high".to_string(),
            severity: AlertSeverity::Warning,
            condition: AlertCondition::MetricThreshold {
                metric_name: "cpu_percent".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 80.0,
                duration: Duration::from_secs(60),
            },
            cooldown: Duration::from_secs(300),
            enabled: true,
            channels: vec!["console".to_string()],
        };

        assert!(rule.enabled);
        assert_eq!(rule.severity, AlertSeverity::Warning);
    }
    #[tokio::test]
    async fn test_alert_manager() {
        let manager = AlertManager::new(AlertConfig::default());

        // Register a console channel
        manager.register_channel("console".to_string(), Arc::new(ConsoleNotificationChannel));

        // Register a rule
        let rule = AlertRule {
            id: "test-rule".to_string(),
            name: "Test Alert".to_string(),
            description: "Test alert rule".to_string(),
            severity: AlertSeverity::Info,
            condition: AlertCondition::MetricThreshold {
                metric_name: "test_metric".to_string(),
                operator: ThresholdOperator::GreaterThan,
                threshold: 100.0,
                duration: Duration::from_secs(0),
            },
            cooldown: Duration::from_secs(60),
            enabled: true,
            channels: vec!["console".to_string()],
        };
        manager.register_rule(rule);

        // Create context that triggers the alert
        let mut metrics = HashMap::new();
        metrics.insert("test_metric".to_string(), MetricValue::Gauge(150.0));

        let context = AlertContext {
            metrics: &metrics,
            health: None,
            performance_violations: &[],
            agent_id: "test-agent",
        };

        // Evaluate rules
        manager.evaluate_rules(context).await.unwrap();

        // Check active alerts
        let active = manager.get_active_alerts();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].rule_id, "test-rule");

        // Test statistics
        let stats = manager.get_statistics();
        assert_eq!(stats.active_count, 1);
        assert_eq!(stats.severity_counts.get(&AlertSeverity::Info), Some(&1));
    }
}
