//! Diagnostics bridge for script engines
//!
//! Provides a unified interface for all script engines to access
//! the centralized diagnostics infrastructure (logging, profiling, metrics)
//! and distributed tracing via OpenTelemetry.

use crate::execution_context::{ExecutionContextBridge, SharedExecutionContext};
use crate::tracing::{DefaultTraceEnricher, SpanHandle, TraceEnricher, TracingConfig};
use llmspell_utils::debug::{global_debug_manager, DebugEntry, DebugLevel, PerformanceTracker};
use opentelemetry::{
    global,
    trace::{SpanKind, Tracer},
    Context, KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{self, TracerProvider as SdkTracerProvider};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

/// Diagnostics bridge that script engines interact with for logging, profiling, and tracing
#[derive(Clone)]
pub struct DiagnosticsBridge {
    /// Reference to the global debug manager
    manager: Arc<llmspell_utils::debug::DebugManager>,
    /// Active performance trackers by ID (using interior mutability)
    trackers: Arc<Mutex<HashMap<String, Arc<PerformanceTracker>>>>,
    /// OpenTelemetry tracer for distributed tracing
    tracer: Option<Arc<opentelemetry::global::BoxedTracer>>,
    /// Trace enricher for adding context to spans
    trace_enricher: Arc<dyn TraceEnricher>,
    /// Tracing configuration
    tracing_config: TracingConfig,
    /// Shared execution context for trace enrichment
    shared_context: Arc<Mutex<SharedExecutionContext>>,
}

impl DiagnosticsBridge {
    /// Create a new diagnostics bridge
    #[must_use]
    pub fn new() -> Self {
        Self {
            manager: global_debug_manager(),
            trackers: Arc::new(Mutex::new(HashMap::new())),
            tracer: None,
            trace_enricher: Arc::new(DefaultTraceEnricher),
            tracing_config: TracingConfig::default(),
            shared_context: Arc::new(Mutex::new(SharedExecutionContext::new())),
        }
    }

    /// Create a new diagnostics bridge with distributed tracing
    #[must_use]
    pub fn with_distributed_tracing(mut self, config: TracingConfig) -> Self {
        if config.enabled {
            // Initialize OpenTelemetry tracer
            if let Ok(tracer) = Self::init_tracer(&config) {
                self.tracer = Some(Arc::new(tracer));
            }
        }
        self.tracing_config = config;
        self
    }

    /// Initialize OpenTelemetry tracer
    fn init_tracer(config: &TracingConfig) -> Result<opentelemetry::global::BoxedTracer, String> {
        let exporter = opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(&config.otlp_endpoint)
            .build()
            .map_err(|e| format!("Failed to create OTLP exporter: {e}"))?;

        let provider = SdkTracerProvider::builder()
            .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
            .with_sampler(trace::Sampler::TraceIdRatioBased(config.sampling_rate))
            .with_resource(opentelemetry_sdk::Resource::new(vec![KeyValue::new(
                "service.name",
                config.service_name.clone(),
            )]))
            .build();

        global::set_tracer_provider(provider);
        Ok(global::tracer("llmspell-diagnostics"))
    }

    /// Start a trace span for script execution
    #[must_use]
    pub fn trace_execution(
        &self,
        operation: &str,
        context: &SharedExecutionContext,
    ) -> Option<Box<dyn SpanHandle>> {
        self.tracer.as_ref().map(|tracer| {
            let span = tracer
                .span_builder(operation.to_string())
                .with_kind(SpanKind::Internal)
                .start(tracer.as_ref());

            // Enrich span with context
            let mut handle = OpenTelemetrySpanHandle { span };
            self.trace_enricher.enrich_span(&mut handle, context);

            Box::new(handle) as Box<dyn SpanHandle>
        })
    }

    /// Trace a diagnostic event
    pub fn trace_diagnostic(&self, message: &str, level: &str) {
        if let Some(ref tracer) = self.tracer {
            let span = tracer
                .span_builder("diagnostic")
                .with_kind(SpanKind::Internal)
                .with_attributes(vec![
                    KeyValue::new("diagnostic.message", message.to_string()),
                    KeyValue::new("diagnostic.level", level.to_string()),
                ])
                .start(tracer.as_ref());
            drop(span); // End span by dropping
        }
    }

    /// Log a message at the specified level
    pub fn log(&self, level: &str, message: &str, module: Option<&str>) {
        if let Ok(debug_level) = level.parse::<DebugLevel>() {
            self.manager
                .log(debug_level, message, module.map(String::from));
        }
    }

    /// Log with metadata and trace the diagnostic
    pub fn log_with_metadata(
        &self,
        level: &str,
        message: &str,
        module: Option<&str>,
        metadata: Value,
    ) {
        // Trace the diagnostic event
        self.trace_diagnostic(message, level);

        if let Ok(debug_level) = level.parse::<DebugLevel>() {
            self.manager.log_with_metadata(
                debug_level,
                message,
                module.map(String::from),
                metadata,
            );
        }
    }

    /// Start a performance timer
    #[must_use]
    pub fn start_timer(&self, name: &str) -> String {
        let tracker = self.manager.start_timer(name);
        let id = format!("timer_{}", uuid::Uuid::new_v4());
        self.trackers.lock().insert(id.clone(), tracker);
        id
    }

    /// Stop a timer and get the duration in milliseconds
    #[must_use]
    pub fn stop_timer(&self, id: &str) -> Option<f64> {
        self.trackers
            .lock()
            .remove(id)
            .map(|tracker| tracker.stop().as_secs_f64() * 1000.0)
    }

    /// Record a lap for a timer
    #[must_use]
    pub fn lap_timer(&self, id: &str, lap_name: &str) -> bool {
        self.trackers.lock().get(id).is_some_and(|tracker| {
            tracker.lap(lap_name);
            true
        })
    }

    /// Get the elapsed time for a timer without stopping it
    #[must_use]
    pub fn elapsed_timer(&self, id: &str) -> Option<f64> {
        self.trackers
            .lock()
            .get(id)
            .map(|tracker| tracker.elapsed().as_secs_f64() * 1000.0)
    }

    /// Set the debug level
    #[must_use]
    pub fn set_level(&self, level: &str) -> bool {
        level.parse::<DebugLevel>().is_ok_and(|debug_level| {
            self.manager.set_level(debug_level);
            true
        })
    }

    /// Get the current debug level
    #[must_use]
    pub fn get_level(&self) -> String {
        self.manager.get_level().to_string()
    }

    /// Enable or disable debugging
    pub fn set_enabled(&self, enabled: bool) {
        self.manager.set_enabled(enabled);
    }

    /// Check if debugging is enabled
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.manager.is_enabled()
    }

    /// Add a module filter
    pub fn add_module_filter(&self, pattern: &str, enabled: bool) {
        self.manager.add_module_filter(pattern, enabled);
    }

    /// Clear all module filters
    pub fn clear_module_filters(&self) {
        self.manager.clear_module_filters();
    }

    /// Get module filter summary
    #[must_use]
    pub fn get_filter_summary(&self) -> llmspell_utils::debug::FilterSummary {
        self.manager.get_filter_summary()
    }

    /// Remove a specific filter pattern
    #[must_use]
    pub fn remove_module_filter(&self, pattern: &str) -> bool {
        self.manager.remove_module_filter(pattern)
    }

    /// Set default filter behavior  
    pub fn set_default_filter_enabled(&self, enabled: bool) {
        self.manager.set_default_filter_enabled(enabled);
    }

    /// Add advanced filter rule
    #[must_use]
    pub fn add_filter_rule(&self, pattern: &str, pattern_type: &str, enabled: bool) -> bool {
        use llmspell_utils::debug::{FilterPattern, FilterRule};

        let filter_pattern = match pattern_type {
            "exact" => FilterPattern::Exact(pattern.to_string()),
            "wildcard" => FilterPattern::Wildcard(pattern.to_string()),
            "regex" => FilterPattern::Regex(pattern.to_string()),
            "hierarchical" => FilterPattern::Hierarchical(pattern.to_string()),
            _ => return false,
        };

        let rule = FilterRule {
            pattern: filter_pattern,
            enabled,
            description: None,
        };

        self.manager.add_filter_rule(rule);
        true
    }

    /// Get captured debug entries
    pub fn get_captured_entries(&self, limit: Option<usize>) -> Vec<DebugEntryInfo> {
        let entries = limit.map_or_else(
            || self.manager.get_captured_entries(),
            |n| self.manager.get_last_entries(n),
        );

        entries.into_iter().map(Into::into).collect()
    }

    /// Clear captured entries
    pub fn clear_captured(&self) {
        self.manager.clear_captured();
    }

    /// Generate a performance report
    #[must_use]
    pub fn generate_performance_report(&self) -> String {
        self.manager.generate_performance_report()
    }

    /// Dump a value for debugging (pretty-print) - JSON fallback
    #[must_use]
    pub fn dump_value(&self, value: &Value, label: Option<&str>) -> String {
        let pretty = serde_json::to_string_pretty(value)
            .unwrap_or_else(|_| "Failed to serialize".to_string());

        if let Some(label) = label {
            format!("{label}: {pretty}")
        } else {
            pretty
        }
    }

    /// Dump a value with enhanced formatting options (for script engines with advanced dumping)
    #[must_use]
    pub fn dump_value_enhanced(
        &self,
        value: &Value,
        label: Option<&str>,
        _compact: bool,
    ) -> String {
        // This will be used by script-specific implementations
        self.dump_value(value, label)
    }

    /// Get memory statistics (placeholder for future implementation)
    #[must_use]
    pub const fn get_memory_stats(&self) -> MemoryStats {
        MemoryStats {
            used_bytes: 0,
            allocated_bytes: 0,
            resident_bytes: 0,
            collections: 0,
        }
    }

    /// Generate JSON performance report
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn generate_json_report(&self) -> Result<String, String> {
        // Use the global debug manager's profiler
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler
            .generate_json_report()
            .map_err(|e| format!("JSON serialization failed: {e}"))
    }

    /// Generate flame graph compatible output
    #[must_use]
    pub fn generate_flame_graph(&self) -> String {
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler.generate_flame_graph()
    }

    /// Get memory usage snapshot
    #[must_use]
    pub fn get_memory_snapshot(&self) -> llmspell_utils::debug::performance::MemorySnapshot {
        let profiler = llmspell_utils::debug::performance::Profiler::new();
        profiler.generate_memory_snapshot()
    }

    /// Record a custom event on a timer
    #[must_use]
    pub fn record_event(
        &self,
        timer_id: &str,
        event_name: &str,
        metadata: Option<serde_json::Value>,
    ) -> bool {
        self.trackers.lock().get(timer_id).is_some_and(|tracker| {
            tracker.event(event_name, metadata);
            true
        })
    }

    /// Get stack trace options for different debug levels
    #[must_use]
    pub fn stack_trace_options_for_level(
        &self,
        level: &str,
    ) -> crate::lua::output::StackTraceOptions {
        match level {
            "trace" => crate::lua::output::StackTraceOptions::for_trace(),
            "error" => crate::lua::output::StackTraceOptions::for_error(),
            _ => crate::lua::output::StackTraceOptions::default(),
        }
    }
}

impl Default for DiagnosticsBridge {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for DiagnosticsBridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiagnosticsBridge")
            .field("enabled", &self.is_enabled())
            .field("level", &self.get_level())
            .field("tracker_count", &self.trackers.lock().len())
            .field("tracing_enabled", &self.tracing_config.enabled)
            .finish_non_exhaustive()
    }
}

/// Implementation of `ExecutionContextBridge` for `DiagnosticsBridge`
impl ExecutionContextBridge for DiagnosticsBridge {
    fn get_context(&self) -> SharedExecutionContext {
        self.shared_context.lock().clone()
    }

    fn update_context(&self, context: SharedExecutionContext) {
        *self.shared_context.lock() = context;
    }

    fn enrich_diagnostic(&self, message: &str) -> String {
        let context = self.get_context();
        let enriched = context
            .location
            .as_ref()
            .map_or_else(
                || message.to_string(),
                |location| format!("{} [{}:{}]", message, location.source, location.line),
            );

        // Create trace span for this diagnostic if tracing is enabled
        if let Some(span) = self.trace_execution("diagnostic", &context) {
            // Span will be ended when dropped
            drop(span);
        }

        enriched
    }
}

/// OpenTelemetry span handle implementation
struct OpenTelemetrySpanHandle {
    span: opentelemetry::global::BoxedSpan,
}

impl SpanHandle for OpenTelemetrySpanHandle {
    fn end(self: Box<Self>) {
        drop(self.span); // Span ends when dropped
    }

    fn record_exception(&mut self, exception: &str, stacktrace: Option<&str>) {
        // Record exception as an event instead of error
        use opentelemetry::trace::Span;
        self.span.add_event(
            "exception",
            vec![
                KeyValue::new("exception.message", exception.to_string()),
                KeyValue::new("exception.stacktrace", stacktrace.unwrap_or("").to_string()),
            ],
        );
    }

    fn set_attribute(&mut self, key: &str, value: String) {
        use opentelemetry::trace::Span;
        self.span
            .set_attribute(KeyValue::new(key.to_string(), value));
    }

    fn context(&self) -> Context {
        // Return current context - span is already active
        Context::current()
    }
}

/// Simplified debug entry for script consumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugEntryInfo {
    pub timestamp: String,
    pub level: String,
    pub module: Option<String>,
    pub message: String,
    pub metadata: Option<Value>,
}

impl From<DebugEntry> for DebugEntryInfo {
    fn from(entry: DebugEntry) -> Self {
        Self {
            timestamp: entry.timestamp.to_rfc3339(),
            level: entry.level.to_string(),
            module: entry.module,
            message: entry.message,
            metadata: entry.metadata,
        }
    }
}

/// Memory statistics for script debugging
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub used_bytes: u64,
    pub allocated_bytes: u64,
    pub resident_bytes: u64,
    pub collections: u32,
}

/// Timer handle for script usage
#[derive(Debug, Clone)]
pub struct TimerHandle {
    pub id: String,
    pub name: String,
}

impl TimerHandle {
    /// Create a new timer handle
    #[must_use]
    pub const fn new(id: String, name: String) -> Self {
        Self { id, name }
    }
}

#[cfg(test)]
mod tests {
    use super::DiagnosticsBridge;

    #[test]
    fn test_debug_bridge_logging() {
        let bridge = DiagnosticsBridge::new();

        // Test basic logging
        bridge.log("info", "Test message", Some("test_module"));
        bridge.log("debug", "Debug message", None);

        // Test invalid level
        bridge.log("invalid", "Should not log", None);
    }

    #[test]
    fn test_debug_bridge_timer() {
        let bridge = DiagnosticsBridge::new();

        // Start a timer
        let timer_id = bridge.start_timer("test_timer");
        assert!(!timer_id.is_empty());

        // Check elapsed time
        let elapsed = bridge.elapsed_timer(&timer_id);
        assert!(elapsed.is_some());

        // Record a lap
        assert!(bridge.lap_timer(&timer_id, "checkpoint"));

        // Stop the timer
        let duration = bridge.stop_timer(&timer_id);
        assert!(duration.is_some());

        // Timer should no longer exist
        assert!(bridge.elapsed_timer(&timer_id).is_none());
    }

    #[test]
    fn test_debug_bridge_configuration() {
        let bridge = DiagnosticsBridge::new();

        // Test level setting
        assert!(bridge.set_level("debug"));
        assert_eq!(bridge.get_level(), "DEBUG");

        // Test invalid level
        assert!(!bridge.set_level("invalid"));

        // Test enable/disable
        bridge.set_enabled(false);
        assert!(!bridge.is_enabled());
        bridge.set_enabled(true);
        assert!(bridge.is_enabled());
    }

    #[test]
    fn test_module_filters() {
        let bridge = DiagnosticsBridge::new();

        // Add filters
        bridge.add_module_filter("workflow", true);
        bridge.add_module_filter("agent.internal", false);

        // Clear filters
        bridge.clear_module_filters();
    }

    #[test]
    fn test_value_dumping() {
        let bridge = DiagnosticsBridge::new();

        let value = serde_json::json!({
            "key": "value",
            "nested": {
                "array": [1, 2, 3]
            }
        });

        let dump = bridge.dump_value(&value, Some("test_object"));
        assert!(dump.contains("test_object"));
        assert!(dump.contains("\"key\": \"value\""));
    }
}
