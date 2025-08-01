// ABOUTME: DebuggingHook implementation for comprehensive trace capture and debugging support
// ABOUTME: Provides stack traces, context dumps, and debugging aids for hook execution

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

/// Debug trace entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugTrace {
    pub timestamp: DateTime<Utc>,
    pub hook_point: HookPoint,
    pub component_name: String,
    pub component_type: String,
    pub language: String,
    pub correlation_id: String,
    pub context_data: serde_json::Value,
    pub metadata: HashMap<String, String>,
    pub stack_trace: Option<String>,
    pub execution_duration: Option<f64>,
}

/// Configuration for the debugging hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebuggingConfig {
    /// Whether to capture stack traces
    pub capture_stack_traces: bool,
    /// Whether to include full context data
    pub include_context_data: bool,
    /// Whether to include metadata
    pub include_metadata: bool,
    /// Maximum number of traces to keep in memory
    pub max_traces: usize,
    /// Whether to log traces to console
    pub log_to_console: bool,
    /// Minimum duration (in ms) to capture for performance debugging
    pub min_duration_ms: u64,
}

impl Default for DebuggingConfig {
    fn default() -> Self {
        Self {
            capture_stack_traces: true,
            include_context_data: true,
            include_metadata: true,
            max_traces: 1000,
            log_to_console: true,
            min_duration_ms: 0, // Capture all by default
        }
    }
}

/// Debug trace storage
#[derive(Debug, Default)]
pub struct TraceStorage {
    traces: Arc<RwLock<Vec<DebugTrace>>>,
    config: DebuggingConfig,
}

impl TraceStorage {
    pub fn new(config: DebuggingConfig) -> Self {
        Self {
            traces: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Add a trace entry
    pub fn add_trace(&self, trace: DebugTrace) {
        let mut traces = self.traces.write().unwrap();

        // Only add if it meets duration threshold
        if let Some(duration) = trace.execution_duration {
            if duration < self.config.min_duration_ms as f64 {
                return;
            }
        }

        traces.push(trace.clone());

        // Maintain max traces limit
        if traces.len() > self.config.max_traces {
            traces.remove(0);
        }

        // Log to console if enabled
        if self.config.log_to_console {
            log::debug!(
                "DEBUG TRACE: {:?} at {:?} ({}:{})",
                trace.hook_point,
                trace.timestamp,
                trace.component_type,
                trace.component_name
            );
        }
    }

    /// Get all traces
    pub fn get_traces(&self) -> Vec<DebugTrace> {
        self.traces.read().unwrap().clone()
    }

    /// Get traces for a specific hook point
    pub fn get_traces_for_hook_point(&self, hook_point: &HookPoint) -> Vec<DebugTrace> {
        self.traces
            .read()
            .unwrap()
            .iter()
            .filter(|trace| &trace.hook_point == hook_point)
            .cloned()
            .collect()
    }

    /// Get traces for a specific component
    pub fn get_traces_for_component(&self, component_name: &str) -> Vec<DebugTrace> {
        self.traces
            .read()
            .unwrap()
            .iter()
            .filter(|trace| trace.component_name == component_name)
            .cloned()
            .collect()
    }

    /// Get traces within a time range
    pub fn get_traces_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<DebugTrace> {
        self.traces
            .read()
            .unwrap()
            .iter()
            .filter(|trace| trace.timestamp >= start && trace.timestamp <= end)
            .cloned()
            .collect()
    }

    /// Clear all traces
    pub fn clear_traces(&self) {
        self.traces.write().unwrap().clear();
    }

    /// Get traces statistics
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let traces = self.traces.read().unwrap();
        let mut stats = HashMap::new();

        // Total count
        stats.insert(
            "total_traces".to_string(),
            serde_json::Value::Number(traces.len().into()),
        );

        // Count by hook point
        let mut hook_point_counts = HashMap::new();
        for trace in traces.iter() {
            let point_name = format!("{:?}", trace.hook_point);
            *hook_point_counts.entry(point_name).or_insert(0u64) += 1;
        }

        let hook_point_json: serde_json::Map<String, serde_json::Value> = hook_point_counts
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect();
        stats.insert(
            "by_hook_point".to_string(),
            serde_json::Value::Object(hook_point_json),
        );

        // Count by component type
        let mut component_type_counts = HashMap::new();
        for trace in traces.iter() {
            *component_type_counts
                .entry(trace.component_type.clone())
                .or_insert(0u64) += 1;
        }

        let component_type_json: serde_json::Map<String, serde_json::Value> = component_type_counts
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect();
        stats.insert(
            "by_component_type".to_string(),
            serde_json::Value::Object(component_type_json),
        );

        // Duration statistics
        let durations: Vec<f64> = traces
            .iter()
            .filter_map(|trace| trace.execution_duration)
            .collect();

        if !durations.is_empty() {
            let mut duration_stats = serde_json::Map::new();

            let sum: f64 = durations.iter().sum();
            let mean = sum / durations.len() as f64;

            let mut sorted_durations = durations.clone();
            sorted_durations.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let median = if sorted_durations.len() % 2 == 0 {
                let mid = sorted_durations.len() / 2;
                (sorted_durations[mid - 1] + sorted_durations[mid]) / 2.0
            } else {
                sorted_durations[sorted_durations.len() / 2]
            };

            duration_stats.insert(
                "mean".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(mean)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            duration_stats.insert(
                "median".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(median)
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            duration_stats.insert(
                "min".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*sorted_durations.first().unwrap())
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );
            duration_stats.insert(
                "max".to_string(),
                serde_json::Value::Number(
                    serde_json::Number::from_f64(*sorted_durations.last().unwrap())
                        .unwrap_or_else(|| serde_json::Number::from(0)),
                ),
            );

            stats.insert(
                "duration_stats".to_string(),
                serde_json::Value::Object(duration_stats),
            );
        }

        stats
    }
}

/// Built-in debugging hook for comprehensive trace capture
pub struct DebuggingHook {
    storage: Arc<TraceStorage>,
    metadata: HookMetadata,
    start_time: Arc<RwLock<Option<std::time::Instant>>>,
}

impl DebuggingHook {
    /// Create a new debugging hook with default configuration
    pub fn new() -> Self {
        Self {
            storage: Arc::new(TraceStorage::new(DebuggingConfig::default())),
            metadata: HookMetadata {
                name: "DebuggingHook".to_string(),
                description: Some("Built-in hook for debugging and trace capture".to_string()),
                priority: Priority::LOWEST, // Run last to capture complete state
                language: Language::Native,
                tags: vec!["builtin".to_string(), "debugging".to_string()],
                version: "1.0.0".to_string(),
            },
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Create a new debugging hook with custom configuration
    pub fn with_config(config: DebuggingConfig) -> Self {
        Self {
            storage: Arc::new(TraceStorage::new(config)),
            metadata: HookMetadata {
                name: "DebuggingHook".to_string(),
                description: Some("Built-in hook for debugging and trace capture".to_string()),
                priority: Priority::LOWEST,
                language: Language::Native,
                tags: vec!["builtin".to_string(), "debugging".to_string()],
                version: "1.0.0".to_string(),
            },
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Enable or disable stack trace capture (only works on new instance)
    pub fn with_stack_traces(self, enable: bool) -> Self {
        let mut config = self.storage.config.clone();
        config.capture_stack_traces = enable;
        Self {
            storage: Arc::new(TraceStorage::new(config)),
            metadata: self.metadata,
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Set minimum duration threshold for trace capture (only works on new instance)
    pub fn with_min_duration_ms(self, min_duration_ms: u64) -> Self {
        let mut config = self.storage.config.clone();
        config.min_duration_ms = min_duration_ms;
        Self {
            storage: Arc::new(TraceStorage::new(config)),
            metadata: self.metadata,
            start_time: Arc::new(RwLock::new(None)),
        }
    }

    /// Get the trace storage
    pub fn storage(&self) -> Arc<TraceStorage> {
        self.storage.clone()
    }

    /// Get all debug traces
    pub fn get_traces(&self) -> Vec<DebugTrace> {
        self.storage.get_traces()
    }

    /// Get debug statistics
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        self.storage.get_statistics()
    }

    /// Clear all traces
    pub fn clear_traces(&self) {
        self.storage.clear_traces();
    }

    /// Capture stack trace if enabled
    fn capture_stack_trace(&self) -> Option<String> {
        if !self.storage.config.capture_stack_traces {
            return None;
        }

        // Simple stack trace capture - in a real implementation,
        // you might use backtrace crate or similar
        Some(format!(
            "Stack trace captured at {}",
            Utc::now().to_rfc3339()
        ))
    }

    /// Create a debug trace from context
    fn create_trace(&self, context: &HookContext, execution_duration: Option<f64>) -> DebugTrace {
        let context_data = if self.storage.config.include_context_data {
            serde_json::to_value(&context.data).unwrap_or(serde_json::Value::Null)
        } else {
            serde_json::Value::Null
        };

        let metadata = if self.storage.config.include_metadata {
            context.metadata.clone()
        } else {
            HashMap::new()
        };

        DebugTrace {
            timestamp: Utc::now(),
            hook_point: context.point.clone(),
            component_name: context.component_id.name.clone(),
            component_type: format!("{:?}", context.component_id.component_type),
            language: format!("{:?}", context.language),
            correlation_id: context.correlation_id.to_string(),
            context_data,
            metadata,
            stack_trace: self.capture_stack_trace(),
            execution_duration,
        }
    }
}

impl Default for DebuggingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for DebuggingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let start_time = std::time::Instant::now();

        // Store start time for duration calculation in post_execution
        {
            let mut start_time_guard = self.start_time.write().unwrap();
            *start_time_guard = Some(start_time);
        }

        // Create and store debug trace
        let trace = self.create_trace(context, None);
        self.storage.add_trace(trace);

        // Add debugging metadata to context
        context.insert_metadata("debug_traced_at".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata(
            "debug_hook_version".to_string(),
            self.metadata.version.clone(),
        );
        context.insert_metadata(
            "debug_correlation_id".to_string(),
            context.correlation_id.to_string(),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Execute for all contexts unless specifically disabled
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for DebuggingHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        // Record the start of debugging
        let trace = self.create_trace(context, None);
        self.storage.add_trace(trace);

        log::trace!("DebuggingHook: Starting trace for {:?}", context.point);
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        duration: std::time::Duration,
    ) -> Result<()> {
        // Create final trace with execution duration
        let execution_duration = duration.as_millis() as f64;
        let mut trace = self.create_trace(context, Some(execution_duration));

        // Add result information to metadata
        trace.metadata.insert(
            "result_type".to_string(),
            format!("{:?}", std::mem::discriminant(result)),
        );
        trace
            .metadata
            .insert("result_description".to_string(), result.description());
        trace.metadata.insert(
            "execution_successful".to_string(),
            result.should_continue().to_string(),
        );

        self.storage.add_trace(trace);

        log::trace!(
            "DebuggingHook: Completed trace for {:?} in {:?}",
            context.point,
            duration
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use serde_json::json;
    #[tokio::test]
    async fn test_debugging_hook_basic() {
        let hook = DebuggingHook::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that debug metadata was added
        assert!(context.get_metadata("debug_traced_at").is_some());
        assert!(context.get_metadata("debug_hook_version").is_some());
        assert!(context.get_metadata("debug_correlation_id").is_some());

        // Check that trace was recorded
        let traces = hook.get_traces();
        assert_eq!(traces.len(), 1);
        assert_eq!(traces[0].hook_point, HookPoint::SystemStartup);
    }
    #[tokio::test]
    async fn test_debugging_hook_with_context_data() {
        let hook = DebuggingHook::new();
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add test data
        context.insert_data("test_key".to_string(), json!("test_value"));
        context.insert_metadata("test_meta".to_string(), "meta_value".to_string());

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        let traces = hook.get_traces();
        assert_eq!(traces.len(), 1);

        let trace = &traces[0];
        assert_eq!(trace.component_name, "test-agent");
        assert!(trace.context_data.is_object());
        assert!(trace.metadata.contains_key("test_meta"));
    }
    #[test]
    fn test_debugging_config() {
        let config = DebuggingConfig {
            capture_stack_traces: false,
            include_context_data: false,
            min_duration_ms: 100,
            ..Default::default()
        };

        let hook = DebuggingHook::with_config(config);
        assert_eq!(hook.storage.config.min_duration_ms, 100);
        assert!(!hook.storage.config.capture_stack_traces);
        assert!(!hook.storage.config.include_context_data);
    }
    #[test]
    fn test_trace_storage_filtering() {
        let storage = TraceStorage::new(DebuggingConfig::default());

        // Add test traces
        let trace1 = DebugTrace {
            timestamp: Utc::now(),
            hook_point: HookPoint::SystemStartup,
            component_name: "test1".to_string(),
            component_type: "System".to_string(),
            language: "Native".to_string(),
            correlation_id: "test-id-1".to_string(),
            context_data: serde_json::Value::Null,
            metadata: HashMap::new(),
            stack_trace: None,
            execution_duration: Some(50.0),
        };

        let trace2 = DebugTrace {
            timestamp: Utc::now(),
            hook_point: HookPoint::BeforeAgentInit,
            component_name: "test2".to_string(),
            component_type: "Agent".to_string(),
            language: "Native".to_string(),
            correlation_id: "test-id-2".to_string(),
            context_data: serde_json::Value::Null,
            metadata: HashMap::new(),
            stack_trace: None,
            execution_duration: Some(150.0),
        };

        storage.add_trace(trace1);
        storage.add_trace(trace2);

        // Test filtering by hook point
        let system_traces = storage.get_traces_for_hook_point(&HookPoint::SystemStartup);
        assert_eq!(system_traces.len(), 1);
        assert_eq!(system_traces[0].component_name, "test1");

        // Test filtering by component
        let test2_traces = storage.get_traces_for_component("test2");
        assert_eq!(test2_traces.len(), 1);
        assert_eq!(test2_traces[0].hook_point, HookPoint::BeforeAgentInit);
    }
    #[test]
    fn test_trace_statistics() {
        let storage = TraceStorage::new(DebuggingConfig::default());

        // Add multiple traces with different characteristics
        for i in 0..5 {
            let trace = DebugTrace {
                timestamp: Utc::now(),
                hook_point: if i % 2 == 0 {
                    HookPoint::SystemStartup
                } else {
                    HookPoint::BeforeAgentInit
                },
                component_name: format!("test{}", i),
                component_type: if i % 2 == 0 {
                    "System".to_string()
                } else {
                    "Agent".to_string()
                },
                language: "Native".to_string(),
                correlation_id: format!("test-id-{}", i),
                context_data: serde_json::Value::Null,
                metadata: HashMap::new(),
                stack_trace: None,
                execution_duration: Some((i + 1) as f64 * 10.0),
            };
            storage.add_trace(trace);
        }

        let stats = storage.get_statistics();

        assert_eq!(stats.get("total_traces").unwrap().as_u64().unwrap(), 5);
        assert!(stats.contains_key("by_hook_point"));
        assert!(stats.contains_key("by_component_type"));
        assert!(stats.contains_key("duration_stats"));
    }
    #[tokio::test]
    async fn test_metric_hook_trait() {
        let hook = DebuggingHook::new();
        let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
        let context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

        // Test MetricHook implementation
        hook.record_pre_execution(&context).await.unwrap();

        let result = HookResult::Continue;
        hook.record_post_execution(&context, &result, std::time::Duration::from_millis(25))
            .await
            .unwrap();

        // Verify traces were recorded
        let traces = hook.get_traces();
        assert!(traces.len() >= 2); // At least pre and post execution traces
    }
    #[test]
    fn test_max_traces_limit() {
        let config = DebuggingConfig {
            max_traces: 3,
            ..Default::default()
        };
        let storage = TraceStorage::new(config);

        // Add more traces than the limit
        for i in 0..5 {
            let trace = DebugTrace {
                timestamp: Utc::now(),
                hook_point: HookPoint::SystemStartup,
                component_name: format!("test{}", i),
                component_type: "System".to_string(),
                language: "Native".to_string(),
                correlation_id: format!("test-id-{}", i),
                context_data: serde_json::Value::Null,
                metadata: HashMap::new(),
                stack_trace: None,
                execution_duration: Some(10.0),
            };
            storage.add_trace(trace);
        }

        let traces = storage.get_traces();
        assert_eq!(traces.len(), 3); // Should be limited to max_traces

        // Should contain the last 3 traces (test2, test3, test4)
        assert_eq!(traces[0].component_name, "test2");
        assert_eq!(traces[1].component_name, "test3");
        assert_eq!(traces[2].component_name, "test4");
    }
    #[test]
    fn test_hook_metadata() {
        let hook = DebuggingHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "DebuggingHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::LOWEST);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"debugging".to_string()));
    }
}

#[async_trait]
impl ReplayableHook for DebuggingHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with debugging config
        let mut context_data = ctx.data.clone();

        // Add debugging configuration for replay
        context_data.insert(
            "_debugging_config".to_string(),
            serde_json::to_value(&self.storage.config)?,
        );

        // Add current traces snapshot for debugging
        let traces = self.storage.get_traces();
        let trace_summary = serde_json::json!({
            "total_traces": traces.len(),
            "hook_points": traces.iter().map(|t| format!("{:?}", t.hook_point)).collect::<Vec<_>>(),
            "average_duration": if traces.is_empty() {
                0.0
            } else {
                traces.iter()
                    .filter_map(|t| t.execution_duration)
                    .sum::<f64>() / traces.len() as f64
            },
            "component_types": traces.iter().map(|t| &t.component_type).collect::<std::collections::HashSet<_>>(),
            "languages": traces.iter().map(|t| &t.language).collect::<std::collections::HashSet<_>>(),
        });
        context_data.insert("_debug_trace_summary".to_string(), trace_summary);

        // Add the current trace entry if debugging info should be included
        if self.storage.config.include_context_data {
            let current_trace = DebugTrace {
                timestamp: Utc::now(),
                hook_point: ctx.point.clone(),
                component_name: ctx.component_id.name.clone(),
                component_type: format!("{:?}", ctx.component_id.component_type),
                language: format!("{:?}", ctx.language),
                correlation_id: ctx.correlation_id.to_string(),
                context_data: serde_json::to_value(&ctx.data)?,
                metadata: ctx.metadata.clone(),
                stack_trace: if self.storage.config.capture_stack_traces {
                    self.capture_stack_trace()
                } else {
                    None
                },
                execution_duration: None,
            };
            context_data.insert(
                "_current_debug_trace".to_string(),
                serde_json::to_value(&current_trace)?,
            );
        }

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the debugging-specific data from context
        context.data.remove("_debugging_config");
        context.data.remove("_debug_trace_summary");
        context.data.remove("_current_debug_trace");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}
