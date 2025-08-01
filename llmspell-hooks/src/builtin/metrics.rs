// ABOUTME: MetricsHook implementation for comprehensive hook execution metrics collection
// ABOUTME: Provides histogram support, performance tracking, and aggregation for all hook points

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, HookPoint, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: HashMap<String, String>,
}

/// Histogram bucket for timing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBucket {
    pub upper_bound: f64,
    pub count: u64,
}

/// Histogram data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Histogram {
    pub buckets: Vec<HistogramBucket>,
    pub count: u64,
    pub sum: f64,
}

impl Histogram {
    pub fn new() -> Self {
        // Default buckets: 1ms, 5ms, 10ms, 25ms, 50ms, 100ms, 250ms, 500ms, 1s, 2.5s, 5s, 10s, +Inf
        let buckets = vec![
            HistogramBucket {
                upper_bound: 0.001,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.005,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.01,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.025,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.05,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.1,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.25,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 0.5,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 1.0,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 2.5,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 5.0,
                count: 0,
            },
            HistogramBucket {
                upper_bound: 10.0,
                count: 0,
            },
            HistogramBucket {
                upper_bound: f64::INFINITY,
                count: 0,
            },
        ];

        Self {
            buckets,
            count: 0,
            sum: 0.0,
        }
    }

    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;

        for bucket in &mut self.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
            }
        }
    }

    pub fn mean(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum / self.count as f64
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target_count = (self.count as f64 * p / 100.0).ceil() as u64;

        for bucket in &self.buckets {
            if bucket.count >= target_count {
                return bucket.upper_bound;
            }
        }

        f64::INFINITY
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new()
    }
}

/// Metrics storage for the MetricsHook
#[derive(Debug, Default)]
pub struct MetricsStorage {
    /// Execution count by hook point
    pub execution_counts: Arc<RwLock<HashMap<HookPoint, u64>>>,
    /// Execution duration histograms by hook point
    pub duration_histograms: Arc<RwLock<HashMap<HookPoint, Histogram>>>,
    /// Error counts by hook point
    pub error_counts: Arc<RwLock<HashMap<HookPoint, u64>>>,
    /// Success counts by hook point
    pub success_counts: Arc<RwLock<HashMap<HookPoint, u64>>>,
    /// Custom metrics
    pub custom_metrics: Arc<RwLock<HashMap<String, Vec<MetricPoint>>>>,
}

impl MetricsStorage {
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an execution
    pub fn record_execution(&self, hook_point: &HookPoint, duration: Duration, success: bool) {
        // Update execution count
        {
            let mut counts = self.execution_counts.write().unwrap();
            *counts.entry(hook_point.clone()).or_insert(0) += 1;
        }

        // Update duration histogram
        {
            let mut histograms = self.duration_histograms.write().unwrap();
            let histogram = histograms.entry(hook_point.clone()).or_default();
            histogram.observe(duration.as_secs_f64());
        }

        // Update success/error counts
        if success {
            let mut counts = self.success_counts.write().unwrap();
            *counts.entry(hook_point.clone()).or_insert(0) += 1;
        } else {
            let mut counts = self.error_counts.write().unwrap();
            *counts.entry(hook_point.clone()).or_insert(0) += 1;
        }
    }

    /// Record a custom metric
    pub fn record_custom_metric(&self, name: String, value: f64, labels: HashMap<String, String>) {
        let metric_point = MetricPoint {
            timestamp: Utc::now(),
            value,
            labels,
        };

        let mut metrics = self.custom_metrics.write().unwrap();
        metrics.entry(name).or_default().push(metric_point);
    }

    /// Get execution count for a hook point
    pub fn get_execution_count(&self, hook_point: &HookPoint) -> u64 {
        self.execution_counts
            .read()
            .unwrap()
            .get(hook_point)
            .copied()
            .unwrap_or(0)
    }

    /// Get duration histogram for a hook point
    pub fn get_duration_histogram(&self, hook_point: &HookPoint) -> Option<Histogram> {
        self.duration_histograms
            .read()
            .unwrap()
            .get(hook_point)
            .cloned()
    }

    /// Get success rate for a hook point
    pub fn get_success_rate(&self, hook_point: &HookPoint) -> f64 {
        let success_count = self
            .success_counts
            .read()
            .unwrap()
            .get(hook_point)
            .copied()
            .unwrap_or(0);

        let error_count = self
            .error_counts
            .read()
            .unwrap()
            .get(hook_point)
            .copied()
            .unwrap_or(0);

        let total = success_count + error_count;
        if total == 0 {
            0.0
        } else {
            success_count as f64 / total as f64
        }
    }

    /// Get all metrics summary
    pub fn get_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();

        // Execution counts
        {
            let counts = self.execution_counts.read().unwrap();
            let mut exec_counts = HashMap::new();
            for (point, count) in counts.iter() {
                exec_counts.insert(
                    format!("{:?}", point),
                    serde_json::Value::Number((*count).into()),
                );
            }
            summary.insert(
                "execution_counts".to_string(),
                serde_json::Value::Object(exec_counts.into_iter().collect()),
            );
        }

        // Success rates
        {
            let success_counts = self.success_counts.read().unwrap();
            let error_counts = self.error_counts.read().unwrap();
            let mut rates = HashMap::new();

            for point in success_counts.keys().chain(error_counts.keys()) {
                rates.insert(
                    format!("{:?}", point),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(self.get_success_rate(point))
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
            }
            summary.insert(
                "success_rates".to_string(),
                serde_json::Value::Object(rates.into_iter().collect()),
            );
        }

        // Duration statistics
        {
            let histograms = self.duration_histograms.read().unwrap();
            let mut duration_stats = HashMap::new();

            for (point, histogram) in histograms.iter() {
                let mut stats = serde_json::Map::new();
                stats.insert(
                    "mean".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(histogram.mean())
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                stats.insert(
                    "p50".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(histogram.percentile(50.0))
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                stats.insert(
                    "p95".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(histogram.percentile(95.0))
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );
                stats.insert(
                    "p99".to_string(),
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(histogram.percentile(99.0))
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    ),
                );

                duration_stats.insert(format!("{:?}", point), serde_json::Value::Object(stats));
            }
            summary.insert(
                "duration_stats".to_string(),
                serde_json::Value::Object(duration_stats.into_iter().collect()),
            );
        }

        summary
    }
}

/// Built-in metrics hook for comprehensive performance tracking
pub struct MetricsHook {
    storage: Arc<MetricsStorage>,
    metadata: HookMetadata,
    collect_custom_metrics: bool,
}

impl MetricsHook {
    /// Create a new metrics hook
    pub fn new() -> Self {
        Self {
            storage: Arc::new(MetricsStorage::new()),
            metadata: HookMetadata {
                name: "MetricsHook".to_string(),
                description: Some("Built-in hook for collecting execution metrics".to_string()),
                priority: Priority::LOW, // Run after other hooks
                language: Language::Native,
                tags: vec!["builtin".to_string(), "metrics".to_string()],
                version: "1.0.0".to_string(),
            },
            collect_custom_metrics: true,
        }
    }

    /// Create a new metrics hook with shared storage
    pub fn with_storage(storage: Arc<MetricsStorage>) -> Self {
        Self {
            storage,
            metadata: HookMetadata {
                name: "MetricsHook".to_string(),
                description: Some("Built-in hook for collecting execution metrics".to_string()),
                priority: Priority::LOW,
                language: Language::Native,
                tags: vec!["builtin".to_string(), "metrics".to_string()],
                version: "1.0.0".to_string(),
            },
            collect_custom_metrics: true,
        }
    }

    /// Enable or disable custom metrics collection
    pub fn with_custom_metrics(mut self, enable: bool) -> Self {
        self.collect_custom_metrics = enable;
        self
    }

    /// Get the metrics storage
    pub fn storage(&self) -> Arc<MetricsStorage> {
        self.storage.clone()
    }

    /// Get metrics summary
    pub fn get_summary(&self) -> HashMap<String, serde_json::Value> {
        self.storage.get_summary()
    }
}

impl Default for MetricsHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for MetricsHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let start_time = Instant::now();

        // Record custom metrics if enabled
        if self.collect_custom_metrics {
            let mut labels = HashMap::new();
            labels.insert("hook_point".to_string(), format!("{:?}", context.point));
            labels.insert(
                "component_type".to_string(),
                format!("{:?}", context.component_id.component_type),
            );
            labels.insert(
                "component_name".to_string(),
                context.component_id.name.clone(),
            );
            labels.insert("language".to_string(), format!("{:?}", context.language));

            self.storage
                .record_custom_metric("hook_execution".to_string(), 1.0, labels);
        }

        let duration = start_time.elapsed();

        // Record execution metrics (always successful for this hook)
        self.storage
            .record_execution(&context.point, duration, true);

        // Add metrics metadata to context
        context.insert_metadata("metrics_recorded_at".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata(
            "metrics_hook_version".to_string(),
            self.metadata.version.clone(),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Always execute metrics hook
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for MetricsHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        // Record that we're about to execute
        let mut labels = HashMap::new();
        labels.insert("hook_point".to_string(), format!("{:?}", context.point));
        labels.insert("phase".to_string(), "pre_execution".to_string());

        if self.collect_custom_metrics {
            self.storage
                .record_custom_metric("hook_lifecycle".to_string(), 1.0, labels);
        }

        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> Result<()> {
        // Record completion metrics
        let success = result.should_continue();
        self.storage
            .record_execution(&context.point, duration, success);

        // Record result type metrics
        if self.collect_custom_metrics {
            let mut labels = HashMap::new();
            labels.insert("hook_point".to_string(), format!("{:?}", context.point));
            labels.insert(
                "result_type".to_string(),
                format!("{:?}", std::mem::discriminant(result)),
            );
            labels.insert("success".to_string(), success.to_string());

            self.storage.record_custom_metric(
                "hook_result".to_string(),
                if success { 1.0 } else { 0.0 },
                labels,
            );
        }

        Ok(())
    }
}

#[async_trait]
impl ReplayableHook for MetricsHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context
        // Note: We don't serialize the actual metrics storage as it's runtime state
        let mut context_data = ctx.data.clone();

        // Add metadata about the metrics hook configuration
        context_data.insert(
            "_metrics_config".to_string(),
            serde_json::json!({
                "collect_custom_metrics": self.collect_custom_metrics,
                "hook_name": self.metadata.name,
                "version": self.metadata.version,
            }),
        );

        // Add current metrics snapshot for context
        context_data.insert(
            "_metrics_snapshot".to_string(),
            serde_json::to_value(self.get_summary())?,
        );

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the metrics-specific data from context
        context.data.remove("_metrics_config");
        context.data.remove("_metrics_snapshot");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    #[test]
    fn test_histogram_basic() {
        let mut histogram = Histogram::new();

        histogram.observe(0.05); // 50ms
        histogram.observe(0.1); // 100ms
        histogram.observe(0.2); // 200ms

        assert_eq!(histogram.count, 3);
        assert!((histogram.sum - 0.35).abs() < 1e-10);
        assert!((histogram.mean() - 0.11666666666666667).abs() < 1e-10);
    }
    #[test]
    fn test_histogram_percentiles() {
        let mut histogram = Histogram::new();

        // Add 100 observations from 0.001 to 0.1 seconds
        for i in 1..=100 {
            histogram.observe(i as f64 * 0.001);
        }

        assert_eq!(histogram.count, 100);

        // Test percentiles
        let p50 = histogram.percentile(50.0);
        let p95 = histogram.percentile(95.0);
        let p99 = histogram.percentile(99.0);

        assert!(p50 > 0.0);
        assert!(p95 >= p50); // Allow equal values due to bucket boundaries
        assert!(p99 >= p95); // Allow equal values due to bucket boundaries

        // Test that we get reasonable bucket values
        assert!(p50 <= 0.1); // Should be within our bucket range
        assert!(p95 <= 0.25); // Should be within our bucket range
        assert!(p99 <= 0.25); // Should be within our bucket range
    }
    #[tokio::test]
    async fn test_metrics_hook_basic() {
        let hook = MetricsHook::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that metrics were recorded
        let count = hook.storage.get_execution_count(&HookPoint::SystemStartup);
        assert_eq!(count, 1);

        // Check success rate
        let success_rate = hook.storage.get_success_rate(&HookPoint::SystemStartup);
        assert_eq!(success_rate, 1.0);
    }
    #[tokio::test]
    async fn test_metrics_hook_multiple_executions() {
        let storage = Arc::new(MetricsStorage::new());
        let hook = MetricsHook::with_storage(storage.clone());
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());

        // Execute multiple times
        for _ in 0..5 {
            let mut context =
                HookContext::new(HookPoint::BeforeAgentExecution, component_id.clone());
            let result = hook.execute(&mut context).await.unwrap();
            assert!(matches!(result, HookResult::Continue));
        }

        // Check metrics
        let count = storage.get_execution_count(&HookPoint::BeforeAgentExecution);
        assert_eq!(count, 5);

        let histogram = storage.get_duration_histogram(&HookPoint::BeforeAgentExecution);
        assert!(histogram.is_some());
        assert_eq!(histogram.unwrap().count, 5);
    }
    #[test]
    fn test_metrics_storage_custom_metrics() {
        let storage = MetricsStorage::new();

        let mut labels = HashMap::new();
        labels.insert("test_label".to_string(), "test_value".to_string());

        storage.record_custom_metric("test_metric".to_string(), 42.0, labels);

        let custom_metrics = storage.custom_metrics.read().unwrap();
        let metric_points = custom_metrics.get("test_metric").unwrap();
        assert_eq!(metric_points.len(), 1);
        assert_eq!(metric_points[0].value, 42.0);
    }
    #[test]
    fn test_metrics_summary() {
        let storage = MetricsStorage::new();

        // Record some test metrics
        storage.record_execution(&HookPoint::SystemStartup, Duration::from_millis(50), true);
        storage.record_execution(&HookPoint::SystemStartup, Duration::from_millis(75), true);
        storage.record_execution(
            &HookPoint::BeforeAgentInit,
            Duration::from_millis(100),
            false,
        );

        let summary = storage.get_summary();

        assert!(summary.contains_key("execution_counts"));
        assert!(summary.contains_key("success_rates"));
        assert!(summary.contains_key("duration_stats"));
    }
    #[tokio::test]
    async fn test_metric_hook_trait() {
        let hook = MetricsHook::new();
        let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
        let context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

        // Test MetricHook implementation
        hook.record_pre_execution(&context).await.unwrap();

        let result = HookResult::Continue;
        hook.record_post_execution(&context, &result, Duration::from_millis(10))
            .await
            .unwrap();

        // Verify metrics were recorded
        let count = hook
            .storage
            .get_execution_count(&HookPoint::BeforeToolExecution);
        assert_eq!(count, 1);
    }
    #[test]
    fn test_hook_metadata() {
        let hook = MetricsHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "MetricsHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::LOW);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"metrics".to_string()));
    }
    #[tokio::test]
    async fn test_replayable_hook_implementation() {
        let hook = MetricsHook::new().with_custom_metrics(true);
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add test data
        context.insert_data("test_key".to_string(), serde_json::json!("test_value"));

        // Execute the hook to generate some metrics
        hook.execute(&mut context).await.unwrap();

        // Test serialization
        let serialized = hook.serialize_context(&context).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized = hook.deserialize_context(&serialized).unwrap();
        assert_eq!(deserialized.point, context.point);
        assert_eq!(deserialized.component_id, context.component_id);
        assert_eq!(
            deserialized.data.get("test_key"),
            context.data.get("test_key")
        );

        // Ensure metrics-specific data was removed
        assert!(deserialized.data.get("_metrics_config").is_none());
        assert!(deserialized.data.get("_metrics_snapshot").is_none());

        // Test replay ID
        assert_eq!(hook.replay_id(), "MetricsHook:1.0.0");
        assert!(hook.is_replayable());
    }
}
