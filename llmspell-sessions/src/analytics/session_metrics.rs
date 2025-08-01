//! ABOUTME: Session-specific analytics implementation using `MetricsHook`
//! ABOUTME: Extends `MetricsHook` with session duration, operation tracking, and resource usage analytics

#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::missing_panics_doc)]

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use llmspell_hooks::{
    builtin::metrics::{MetricsHook, MetricsStorage},
    traits::Hook,
    types::{HookMetadata, Language, Priority},
    HookContext, HookPoint, HookResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Session-specific metric types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SessionMetricType {
    /// Session duration metrics
    SessionDuration,
    /// Operation count metrics
    OperationCount,
    /// Resource usage metrics
    ResourceUsage,
    /// Success rate metrics
    SuccessRate,
    /// Checkpoint frequency
    CheckpointFrequency,
    /// State size metrics
    StateSize,
    /// User activity metrics
    UserActivity,
}

/// Session analytics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalyticsConfig {
    /// Enable detailed metrics collection
    pub enable_detailed_metrics: bool,
    /// Enable operation tracking
    pub enable_operation_tracking: bool,
    /// Enable resource usage tracking
    pub enable_resource_tracking: bool,
    /// Metrics retention period
    pub retention_period: Duration,
    /// Privacy mode - anonymize sensitive data
    pub privacy_mode: bool,
}

impl Default for SessionAnalyticsConfig {
    fn default() -> Self {
        Self {
            enable_detailed_metrics: true,
            enable_operation_tracking: true,
            enable_resource_tracking: true,
            retention_period: Duration::from_secs(86400), // 24 hours
            privacy_mode: false,
        }
    }
}

/// Session metrics collector
pub struct SessionMetricsCollector {
    /// Base metrics hook
    metrics_hook: Arc<MetricsHook>,
    /// Configuration
    config: SessionAnalyticsConfig,
    /// Session start times
    session_start_times: Arc<tokio::sync::RwLock<HashMap<String, Instant>>>,
    /// Operation counts by session
    operation_counts: Arc<tokio::sync::RwLock<HashMap<String, HashMap<String, u64>>>>,
    /// Hook metadata
    metadata: HookMetadata,
}

impl SessionMetricsCollector {
    /// Create new session metrics collector
    pub fn new(storage: Arc<MetricsStorage>, config: SessionAnalyticsConfig) -> Self {
        let metrics_hook = Arc::new(
            MetricsHook::with_storage(storage).with_custom_metrics(config.enable_detailed_metrics),
        );

        Self {
            metrics_hook,
            config,
            session_start_times: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            operation_counts: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
            metadata: HookMetadata {
                name: "SessionMetricsCollector".to_string(),
                description: Some("Session-specific metrics collection".to_string()),
                priority: Priority::LOW,
                language: Language::Native,
                tags: vec!["session".to_string(), "analytics".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Record session start
    async fn record_session_start(&self, session_id: &str) -> Result<()> {
        let mut start_times = self.session_start_times.write().await;
        start_times.insert(session_id.to_string(), Instant::now());

        // Record custom metric for session start
        if self.config.enable_detailed_metrics {
            let mut labels = HashMap::new();
            labels.insert("event_type".to_string(), "session_start".to_string());
            labels.insert(
                "session_id".to_string(),
                self.anonymize_if_needed(session_id),
            );

            self.metrics_hook.storage().record_custom_metric(
                "session_lifecycle".to_string(),
                1.0,
                labels,
            );
        }

        Ok(())
    }

    /// Record session end and calculate duration
    async fn record_session_end(&self, session_id: &str) -> Result<()> {
        let duration = {
            let mut start_times = self.session_start_times.write().await;
            if let Some(start_time) = start_times.remove(session_id) {
                start_time.elapsed()
            } else {
                Duration::from_secs(0)
            }
        };

        // Record session duration metric
        if self.config.enable_detailed_metrics {
            let mut labels = HashMap::new();
            labels.insert("event_type".to_string(), "session_end".to_string());
            labels.insert(
                "session_id".to_string(),
                self.anonymize_if_needed(session_id),
            );
            labels.insert("metric_type".to_string(), "duration_seconds".to_string());

            self.metrics_hook.storage().record_custom_metric(
                "session_duration".to_string(),
                duration.as_secs_f64(),
                labels,
            );
        }

        // Clean up operation counts
        self.operation_counts.write().await.remove(session_id);

        Ok(())
    }

    /// Record operation for a session
    async fn record_operation(&self, session_id: &str, operation: &str) -> Result<()> {
        if !self.config.enable_operation_tracking {
            return Ok(());
        }

        let mut counts = self.operation_counts.write().await;
        let session_ops = counts.entry(session_id.to_string()).or_default();
        *session_ops.entry(operation.to_string()).or_insert(0) += 1;

        // Record custom metric
        let mut labels = HashMap::new();
        labels.insert(
            "session_id".to_string(),
            self.anonymize_if_needed(session_id),
        );
        labels.insert("operation".to_string(), operation.to_string());

        self.metrics_hook.storage().record_custom_metric(
            "session_operation".to_string(),
            1.0,
            labels,
        );

        Ok(())
    }

    /// Anonymize session ID if privacy mode is enabled
    fn anonymize_if_needed(&self, session_id: &str) -> String {
        if self.config.privacy_mode {
            // Hash the session ID for privacy
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            session_id.hash(&mut hasher);
            format!("session_{:x}", hasher.finish())
        } else {
            session_id.to_string()
        }
    }

    /// Get storage reference
    pub fn storage(&self) -> Arc<MetricsStorage> {
        self.metrics_hook.storage()
    }
}

#[async_trait]
impl Hook for SessionMetricsCollector {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // First execute the base metrics hook
        let result = self.metrics_hook.execute(context).await?;

        // Extract session ID from context
        let session_id = context
            .data
            .get("session_id")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        // Record session-specific metrics based on hook point
        match &context.point {
            HookPoint::SessionStart => {
                self.record_session_start(session_id).await?;
            }
            HookPoint::SessionEnd => {
                self.record_session_end(session_id).await?;
            }
            HookPoint::SessionCheckpoint => {
                self.record_operation(session_id, "checkpoint").await?;
            }
            HookPoint::SessionRestore => {
                self.record_operation(session_id, "restore").await?;
            }
            HookPoint::SessionSave => {
                self.record_operation(session_id, "save").await?;

                // Record state size if available
                if self.config.enable_resource_tracking {
                    if let Some(state_size) =
                        context.data.get("state_size").and_then(|v| v.as_u64())
                    {
                        let mut labels = HashMap::new();
                        labels.insert(
                            "session_id".to_string(),
                            self.anonymize_if_needed(session_id),
                        );
                        labels.insert("metric_type".to_string(), "state_size_bytes".to_string());

                        self.metrics_hook.storage().record_custom_metric(
                            "session_resource".to_string(),
                            #[allow(clippy::cast_precision_loss)]
                            {
                                state_size as f64
                            },
                            labels,
                        );
                    }
                }
            }
            _ => {
                // Record general operation
                if self.config.enable_operation_tracking {
                    let operation = format!("{:?}", context.point);
                    self.record_operation(session_id, &operation).await?;
                }
            }
        }

        Ok(result)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Session analytics facade
pub struct SessionAnalytics {
    /// Metrics collector
    collector: Arc<SessionMetricsCollector>,
    /// Shared storage
    storage: Arc<MetricsStorage>,
    /// Configuration
    config: SessionAnalyticsConfig,
}

impl SessionAnalytics {
    /// Create new session analytics
    pub fn new(config: SessionAnalyticsConfig) -> Self {
        let storage = Arc::new(MetricsStorage::new());
        let collector = Arc::new(SessionMetricsCollector::new(
            storage.clone(),
            config.clone(),
        ));

        Self {
            collector,
            storage,
            config,
        }
    }

    /// Get the hook for registration
    pub fn as_hook(&self) -> Arc<dyn Hook> {
        self.collector.clone()
    }

    /// Get session metrics summary
    pub fn get_session_summary(&self, session_id: &str) -> SessionMetricsSummary {
        let anonymized_id = if self.config.privacy_mode {
            self.collector.anonymize_if_needed(session_id)
        } else {
            session_id.to_string()
        };

        // Get metrics from storage
        let custom_metrics = self.storage.custom_metrics.read().unwrap();

        let mut summary = SessionMetricsSummary {
            session_id: anonymized_id.clone(),
            total_operations: 0,
            operation_breakdown: HashMap::new(),
            average_operation_time: Duration::from_secs(0),
            success_rate: 0.0,
            resource_usage: HashMap::new(),
            session_duration: None,
            last_activity: None,
        };

        // Calculate operation counts
        if let Some(ops) = custom_metrics.get("session_operation") {
            for metric in ops {
                if metric.labels.get("session_id").map(|s| s.as_str()) == Some(&anonymized_id) {
                    summary.total_operations += 1;
                    if let Some(op) = metric.labels.get("operation") {
                        *summary.operation_breakdown.entry(op.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Get session duration
        if let Some(durations) = custom_metrics.get("session_duration") {
            for metric in durations {
                if metric.labels.get("session_id").map(|s| s.as_str()) == Some(&anonymized_id) {
                    summary.session_duration = Some(Duration::from_secs_f64(metric.value));
                    summary.last_activity = Some(metric.timestamp);
                }
            }
        }

        // Calculate success rate from hook execution metrics
        let hook_points = vec![
            HookPoint::SessionStart,
            HookPoint::SessionEnd,
            HookPoint::SessionCheckpoint,
            HookPoint::SessionRestore,
            HookPoint::SessionSave,
        ];

        let mut total_executions = 0;
        let mut successful_executions = 0;

        for hook_point in &hook_points {
            let success_count = self
                .storage
                .success_counts
                .read()
                .unwrap()
                .get(hook_point)
                .copied()
                .unwrap_or(0);
            let error_count = self
                .storage
                .error_counts
                .read()
                .unwrap()
                .get(hook_point)
                .copied()
                .unwrap_or(0);

            successful_executions += success_count;
            total_executions += success_count + error_count;
        }

        if total_executions > 0 {
            #[allow(clippy::cast_precision_loss)]
            let success_rate = successful_executions as f64 / total_executions as f64;
            summary.success_rate = success_rate;
        }

        // Get resource usage
        if let Some(resources) = custom_metrics.get("session_resource") {
            for metric in resources {
                if metric.labels.get("session_id").map(|s| s.as_str()) == Some(&anonymized_id) {
                    if let Some(metric_type) = metric.labels.get("metric_type") {
                        summary
                            .resource_usage
                            .insert(metric_type.clone(), metric.value);
                    }
                }
            }
        }

        summary
    }

    /// Get aggregated metrics across all sessions
    pub fn get_aggregated_metrics(&self) -> HashMap<String, serde_json::Value> {
        let mut aggregated = self.storage.get_summary();

        // Add session-specific aggregations
        let custom_metrics = self.storage.custom_metrics.read().unwrap();

        // Count total sessions
        let mut unique_sessions = std::collections::HashSet::new();
        if let Some(lifecycle) = custom_metrics.get("session_lifecycle") {
            for metric in lifecycle {
                if let Some(session_id) = metric.labels.get("session_id") {
                    unique_sessions.insert(session_id.clone());
                }
            }
        }

        aggregated.insert(
            "total_sessions".to_string(),
            serde_json::json!(unique_sessions.len()),
        );

        // Calculate average session duration
        let mut total_duration = 0.0;
        let mut duration_count = 0;
        if let Some(durations) = custom_metrics.get("session_duration") {
            for metric in durations {
                total_duration += metric.value;
                duration_count += 1;
            }
        }

        if duration_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let avg_duration = total_duration / f64::from(duration_count);
            aggregated.insert(
                "average_session_duration_seconds".to_string(),
                serde_json::json!(avg_duration),
            );
        }

        aggregated
    }

    /// Clean up old metrics based on retention period
    pub fn cleanup_old_metrics(&self) -> Result<()> {
        let cutoff = Utc::now() - chrono::Duration::from_std(self.config.retention_period)?;

        let mut custom_metrics = self.storage.custom_metrics.write().unwrap();
        for metrics in custom_metrics.values_mut() {
            metrics.retain(|m| m.timestamp > cutoff);
        }

        Ok(())
    }
}

/// Session metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetricsSummary {
    /// Session ID (anonymized if privacy mode enabled)
    pub session_id: String,
    /// Total number of operations
    pub total_operations: u64,
    /// Operations breakdown by type
    pub operation_breakdown: HashMap<String, u64>,
    /// Average operation time
    pub average_operation_time: Duration,
    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Resource usage metrics
    pub resource_usage: HashMap<String, f64>,
    /// Session duration if ended
    pub session_duration: Option<Duration>,
    /// Last activity timestamp
    pub last_activity: Option<DateTime<Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::types::{ComponentId, ComponentType};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_metrics_collector() {
        let storage = Arc::new(MetricsStorage::new());
        let config = SessionAnalyticsConfig::default();
        let collector = SessionMetricsCollector::new(storage.clone(), config);

        let mut context = HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(ComponentType::Agent, "test-session".to_string()),
        );
        context.insert_data(
            "session_id".to_string(),
            serde_json::json!("test-session-123"),
        );

        // Execute hook
        let result = collector.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Verify session start was recorded
        let start_times = collector.session_start_times.read().await;
        assert!(start_times.contains_key("test-session-123"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_session_analytics() {
        let analytics = SessionAnalytics::new(SessionAnalyticsConfig::default());

        let mut context = HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(ComponentType::Agent, "analytics-test".to_string()),
        );
        context.insert_data("session_id".to_string(), serde_json::json!("session-456"));

        // Start session
        analytics.as_hook().execute(&mut context).await.unwrap();

        // Perform some operations
        context.point = HookPoint::SessionCheckpoint;
        analytics.as_hook().execute(&mut context).await.unwrap();

        context.point = HookPoint::SessionSave;
        context.insert_data("state_size".to_string(), serde_json::json!(1024));
        analytics.as_hook().execute(&mut context).await.unwrap();

        // End session
        context.point = HookPoint::SessionEnd;
        analytics.as_hook().execute(&mut context).await.unwrap();

        // Get summary
        let summary = analytics.get_session_summary("session-456");
        assert!(summary.total_operations > 0);
        assert!(summary.session_duration.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_privacy_mode() {
        let mut config = SessionAnalyticsConfig::default();
        config.privacy_mode = true;

        let analytics = SessionAnalytics::new(config);

        let mut context = HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(ComponentType::Agent, "private-session".to_string()),
        );
        context.insert_data(
            "session_id".to_string(),
            serde_json::json!("sensitive-id-789"),
        );

        analytics.as_hook().execute(&mut context).await.unwrap();

        // Get summary - should have anonymized ID
        let summary = analytics.get_session_summary("sensitive-id-789");
        assert_ne!(summary.session_id, "sensitive-id-789");
        assert!(summary.session_id.starts_with("session_"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_aggregated_metrics() {
        let analytics = SessionAnalytics::new(SessionAnalyticsConfig::default());

        let aggregated = analytics.get_aggregated_metrics();
        assert!(aggregated.contains_key("execution_counts"));
        assert!(aggregated.contains_key("success_rates"));
        assert!(aggregated.contains_key("total_sessions"));
    }
}
