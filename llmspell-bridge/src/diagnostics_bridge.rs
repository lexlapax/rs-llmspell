//! Diagnostics bridge for script engines
//!
//! Provides a unified interface for all script engines to access
//! the centralized diagnostics infrastructure (logging, profiling, metrics)
//! and distributed tracing via OpenTelemetry.

use crate::circuit_breaker::{CircuitBreaker, ExponentialBackoffBreaker};
use crate::condition_evaluator::ConditionEvaluator;
use crate::execution_bridge::StackFrame;
use crate::execution_context::{ExecutionContextBridge, SharedExecutionContext, SourceLocation};
use crate::hook_profiler::{HookProfiler, RealHookProfiler};
use crate::profiler::{PprofProfiler, Profiler};
use crate::session_recorder::{JsonFileRecorder, SessionRecorder};
use crate::stack_navigator::StackNavigator;
use crate::tracing::{DefaultTraceEnricher, SpanHandle, TraceEnricher, TracingConfig};
use crate::variable_inspector::VariableInspector;
use llmspell_tools::util::ValidationResult;
use llmspell_utils::debug::{global_debug_manager, DebugEntry, DebugLevel, PerformanceTracker};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
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
use std::path::PathBuf;
use std::sync::{mpsc, Arc};

/// Hot reload event type
#[derive(Debug, Clone)]
pub enum HotReloadEvent {
    /// File was modified
    Modified { path: PathBuf, content: String },
    /// File was created
    Created { path: PathBuf, content: String },
    /// File was deleted
    Deleted { path: PathBuf },
    /// Reload was successful
    ReloadSuccess { path: PathBuf, duration_ms: u64 },
    /// Reload failed with validation errors
    ReloadFailed { path: PathBuf, errors: Vec<String> },
}

/// Hot reload configuration
#[derive(Debug, Clone)]
pub struct HotReloadConfig {
    /// Debounce delay in milliseconds
    pub debounce_ms: u64,
    /// Enable validation before reload
    pub validate_before_reload: bool,
    /// Maximum reload attempts
    pub max_reload_attempts: u32,
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self {
            debounce_ms: 100,
            validate_before_reload: true,
            max_reload_attempts: 3,
        }
    }
}

/// Comprehensive validation report for script analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Syntax and compilation errors (critical issues)
    pub errors: Vec<ValidationIssue>,
    /// API usage and security warnings (non-critical issues)
    pub warnings: Vec<ValidationIssue>,
    /// Performance and best practice suggestions
    pub suggestions: Vec<ValidationIssue>,
    /// Overall validation success status
    pub is_valid: bool,
    /// Validation duration in microseconds
    pub validation_duration_us: u64,
}

/// Individual validation issue with context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue severity level
    pub level: ValidationLevel,
    /// Issue category (syntax, security, performance, etc.)
    pub category: String,
    /// Human-readable issue description
    pub message: String,
    /// Source location if available
    pub location: Option<SourceLocation>,
    /// Suggested fix if available
    pub suggestion: Option<String>,
}

/// Validation issue severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValidationLevel {
    /// Critical error - prevents execution
    Error,
    /// Warning - execution possible but risky
    Warning,
    /// Suggestion - improvement opportunity
    Suggestion,
}

impl ValidationReport {
    /// Create a new empty validation report
    #[must_use]
    pub const fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            is_valid: true,
            validation_duration_us: 0,
        }
    }

    /// Add an error to the report
    pub fn add_error(&mut self, message: String, location: Option<SourceLocation>) {
        self.errors.push(ValidationIssue {
            level: ValidationLevel::Error,
            category: "error".to_string(),
            message,
            location,
            suggestion: None,
        });
        self.is_valid = false;
    }

    /// Add a warning to the report
    pub fn add_warning(
        &mut self,
        category: String,
        message: String,
        location: Option<SourceLocation>,
    ) {
        self.warnings.push(ValidationIssue {
            level: ValidationLevel::Warning,
            category,
            message,
            location,
            suggestion: None,
        });
    }

    /// Add a suggestion to the report
    pub fn add_suggestion(&mut self, category: String, message: String, suggestion: String) {
        self.suggestions.push(ValidationIssue {
            level: ValidationLevel::Suggestion,
            category,
            message,
            location: None,
            suggestion: Some(suggestion),
        });
    }

    /// Get total issue count
    #[must_use]
    pub const fn total_issues(&self) -> usize {
        self.errors.len() + self.warnings.len() + self.suggestions.len()
    }

    /// Set validation duration
    pub const fn set_duration(&mut self, duration_us: u64) {
        self.validation_duration_us = duration_us;
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU sample data for profiling
#[derive(Debug, Clone)]
pub struct CpuSample {
    /// Timestamp when sample was taken
    pub timestamp: std::time::Instant,
    /// Stack frames at the time of sampling
    pub stack: Vec<StackFrame>,
    /// Thread ID (optional)
    pub thread_id: Option<u64>,
}

/// Flamegraph frame data enhanced with execution context
#[derive(Debug, Clone)]
pub struct FlameGraphFrame {
    /// Function name
    pub function: String,
    /// Source file path
    pub file: String,
    /// Line number
    pub line: u32,
    /// Number of times this frame was executed
    pub execution_count: u64,
    /// Total time spent in this frame (microseconds)
    pub total_time_us: u64,
    /// Self time spent in this frame (microseconds)
    pub self_time_us: u64,
}

/// Memory allocation sample
#[derive(Debug, Clone)]
pub struct MemorySample {
    /// Timestamp when sample was taken
    pub timestamp: std::time::Instant,
    /// Memory usage in bytes
    pub bytes_allocated: u64,
    /// Stack trace at time of allocation (if available)
    pub stack: Vec<StackFrame>,
}

/// Profiling session data
#[derive(Debug)]
pub struct ProfilingSession {
    /// CPU samples collected during the session
    pub cpu_samples: Vec<CpuSample>,
    /// Memory samples collected during the session
    pub memory_samples: Vec<MemorySample>,
    /// Session start time
    pub start_time: std::time::Instant,
    /// Session end time (None if still active)
    pub end_time: Option<std::time::Instant>,
    /// Profiling configuration used
    pub sample_rate_hz: u32,
}

/// Diagnostics bridge that script engines interact with for logging, profiling, and tracing
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
    /// Hot reload watcher (optional)
    hot_reload_watcher: Arc<Mutex<Option<RecommendedWatcher>>>,
    /// Hot reload event receiver
    hot_reload_receiver: Arc<Mutex<Option<mpsc::Receiver<notify::Result<Event>>>>>,
    /// Hot reload configuration
    hot_reload_config: HotReloadConfig,
    /// Watched file paths with their execution contexts
    watched_files: Arc<Mutex<HashMap<PathBuf, Arc<Mutex<SharedExecutionContext>>>>>,
    /// Condition evaluator for syntax validation
    condition_evaluator: Option<Arc<dyn ConditionEvaluator>>,
    /// Variable inspector for API validation
    variable_inspector: Option<Arc<dyn VariableInspector>>,
    /// Stack navigator for enhanced flamegraph generation
    stack_navigator: Option<Arc<dyn StackNavigator>>,
    /// CPU profiler (trait-based for testability)
    profiler: Box<dyn Profiler>,
    /// Hook profiler for monitoring hook execution performance (trait-based for testability)
    hook_profiler: Box<dyn HookProfiler>,
    /// Circuit breaker for hook fault tolerance (trait-based for testability)
    circuit_breaker: Box<dyn CircuitBreaker>,
    /// Session recorder for debugging replay (trait-based for testability)
    session_recorder: Box<dyn SessionRecorder>,
    /// Profiling session data
    profiling_session: Arc<Mutex<Option<ProfilingSession>>>,
    /// Shared execution context for profiling (separate from trace enrichment)
    profiling_context: Option<Arc<tokio::sync::RwLock<SharedExecutionContext>>>,
}

/// Builder for `DiagnosticsBridge` with clean dependency injection
pub struct DiagnosticsBridgeBuilder {
    /// Optional profiler implementation
    profiler: Option<Box<dyn Profiler>>,
    /// Optional hook profiler implementation
    hook_profiler: Option<Box<dyn HookProfiler>>,
    /// Optional circuit breaker implementation
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    /// Optional session recorder implementation
    session_recorder: Option<Box<dyn SessionRecorder>>,
    /// Optional condition evaluator
    condition_evaluator: Option<Arc<dyn ConditionEvaluator>>,
    /// Optional variable inspector
    variable_inspector: Option<Arc<dyn VariableInspector>>,
    /// Optional stack navigator
    stack_navigator: Option<Arc<dyn StackNavigator>>,
    /// Optional trace enricher
    trace_enricher: Option<Arc<dyn TraceEnricher>>,
    /// Optional tracing configuration
    tracing_config: Option<TracingConfig>,
    /// Optional hot reload configuration
    hot_reload_config: Option<HotReloadConfig>,
}

#[allow(clippy::missing_const_for_fn)] // Builder methods cannot be const due to Box<dyn Trait> and Option
impl DiagnosticsBridgeBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            profiler: None,
            hook_profiler: None,
            circuit_breaker: None,
            session_recorder: None,
            condition_evaluator: None,
            variable_inspector: None,
            stack_navigator: None,
            trace_enricher: None,
            tracing_config: None,
            hot_reload_config: None,
        }
    }

    /// Set the profiler implementation
    #[must_use]
    pub fn profiler(mut self, profiler: Box<dyn Profiler>) -> Self {
        self.profiler = Some(profiler);
        self
    }

    /// Set the hook profiler implementation
    #[must_use]
    pub fn hook_profiler(mut self, hook_profiler: Box<dyn HookProfiler>) -> Self {
        self.hook_profiler = Some(hook_profiler);
        self
    }

    /// Set the circuit breaker implementation
    #[must_use]
    pub fn circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    /// Set the session recorder implementation
    #[must_use]
    pub fn session_recorder(mut self, session_recorder: Box<dyn SessionRecorder>) -> Self {
        self.session_recorder = Some(session_recorder);
        self
    }

    /// Set the condition evaluator
    #[must_use]
    pub fn condition_evaluator(mut self, evaluator: Arc<dyn ConditionEvaluator>) -> Self {
        self.condition_evaluator = Some(evaluator);
        self
    }

    /// Set the variable inspector
    #[must_use]
    pub fn variable_inspector(mut self, inspector: Arc<dyn VariableInspector>) -> Self {
        self.variable_inspector = Some(inspector);
        self
    }

    /// Set the stack navigator
    #[must_use]
    pub fn stack_navigator(mut self, navigator: Arc<dyn StackNavigator>) -> Self {
        self.stack_navigator = Some(navigator);
        self
    }

    /// Set the trace enricher
    #[must_use]
    pub fn trace_enricher(mut self, enricher: Arc<dyn TraceEnricher>) -> Self {
        self.trace_enricher = Some(enricher);
        self
    }

    /// Set the tracing configuration
    #[must_use]
    pub fn tracing_config(mut self, config: TracingConfig) -> Self {
        self.tracing_config = Some(config);
        self
    }

    /// Set the hot reload configuration
    #[must_use]
    pub fn hot_reload_config(mut self, config: HotReloadConfig) -> Self {
        self.hot_reload_config = Some(config);
        self
    }

    /// Build the `DiagnosticsBridge` with defaults for missing components
    #[must_use]
    pub fn build(self) -> DiagnosticsBridge {
        let tracing_config = self.tracing_config.unwrap_or_default();
        let tracer = if tracing_config.enabled {
            DiagnosticsBridge::init_tracer(&tracing_config)
                .ok()
                .map(Arc::new)
        } else {
            None
        };

        DiagnosticsBridge {
            manager: global_debug_manager(),
            trackers: Arc::new(Mutex::new(HashMap::new())),
            tracer,
            trace_enricher: self
                .trace_enricher
                .unwrap_or_else(|| Arc::new(DefaultTraceEnricher)),
            tracing_config,
            shared_context: Arc::new(Mutex::new(SharedExecutionContext::new())),
            hot_reload_watcher: Arc::new(Mutex::new(None)),
            hot_reload_receiver: Arc::new(Mutex::new(None)),
            hot_reload_config: self.hot_reload_config.unwrap_or_default(),
            watched_files: Arc::new(Mutex::new(HashMap::new())),
            condition_evaluator: self.condition_evaluator,
            variable_inspector: self.variable_inspector,
            stack_navigator: self.stack_navigator,
            profiler: self
                .profiler
                .unwrap_or_else(|| Box::new(PprofProfiler::new())),
            hook_profiler: self
                .hook_profiler
                .unwrap_or_else(|| Box::new(RealHookProfiler::new())),
            circuit_breaker: self
                .circuit_breaker
                .unwrap_or_else(|| Box::new(ExponentialBackoffBreaker::default())),
            session_recorder: self
                .session_recorder
                .unwrap_or_else(|| Box::new(JsonFileRecorder::new())),
            profiling_session: Arc::new(Mutex::new(None)),
            profiling_context: None,
        }
    }
}

impl Default for DiagnosticsBridgeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl DiagnosticsBridge {
    /// Create a new diagnostics bridge
    #[must_use]
    pub fn new() -> Self {
        Self::with_profiler(Box::new(PprofProfiler::new()))
    }

    /// Create a builder for constructing a `DiagnosticsBridge` with custom components
    #[must_use]
    pub fn builder() -> DiagnosticsBridgeBuilder {
        DiagnosticsBridgeBuilder::new()
    }

    /// Create a new diagnostics bridge with custom profiler (for dependency injection)
    #[must_use]
    pub fn with_profiler(profiler: Box<dyn Profiler>) -> Self {
        Self {
            manager: global_debug_manager(),
            trackers: Arc::new(Mutex::new(HashMap::new())),
            tracer: None,
            trace_enricher: Arc::new(DefaultTraceEnricher),
            tracing_config: TracingConfig::default(),
            shared_context: Arc::new(Mutex::new(SharedExecutionContext::new())),
            hot_reload_watcher: Arc::new(Mutex::new(None)),
            hot_reload_receiver: Arc::new(Mutex::new(None)),
            hot_reload_config: HotReloadConfig::default(),
            watched_files: Arc::new(Mutex::new(HashMap::new())),
            condition_evaluator: None,
            variable_inspector: None,
            stack_navigator: None,
            profiler,
            hook_profiler: Box::new(RealHookProfiler::new()),
            circuit_breaker: Box::new(ExponentialBackoffBreaker::default()),
            session_recorder: Box::new(JsonFileRecorder::new()),
            profiling_session: Arc::new(Mutex::new(None)),
            profiling_context: None,
        }
    }

    /// Set the hook profiler for dependency injection
    #[must_use]
    pub fn with_hook_profiler(mut self, hook_profiler: Box<dyn HookProfiler>) -> Self {
        self.hook_profiler = hook_profiler;
        self
    }

    /// Set the circuit breaker for dependency injection
    #[must_use]
    pub fn with_circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker>) -> Self {
        self.circuit_breaker = circuit_breaker;
        self
    }

    /// Set the session recorder for dependency injection
    #[must_use]
    pub fn with_session_recorder(mut self, session_recorder: Box<dyn SessionRecorder>) -> Self {
        self.session_recorder = session_recorder;
        self
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

    /// Set the condition evaluator for syntax validation
    #[must_use]
    pub fn with_condition_evaluator(mut self, evaluator: Arc<dyn ConditionEvaluator>) -> Self {
        self.condition_evaluator = Some(evaluator);
        self
    }

    /// Set the variable inspector for API validation
    #[must_use]
    pub fn with_variable_inspector(mut self, inspector: Arc<dyn VariableInspector>) -> Self {
        self.variable_inspector = Some(inspector);
        self
    }

    /// Set the stack navigator for enhanced flamegraph generation
    #[must_use]
    pub fn with_stack_navigator(mut self, navigator: Arc<dyn StackNavigator>) -> Self {
        self.stack_navigator = Some(navigator);
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

    /// Comprehensive script validation using Phase 9.2 three-layer architecture
    ///
    /// # Errors
    ///
    /// Returns an error if validation cannot be performed due to internal failures
    #[allow(clippy::cognitive_complexity)]
    #[tracing::instrument(skip(self, context))]
    pub fn validate_script_comprehensive(
        &self,
        script: &str,
        context: &mut SharedExecutionContext,
    ) -> Result<ValidationReport, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        let span = tracing::info_span!("script_validation", script_length = script.len());
        let _enter = span.enter();

        let mut report = ValidationReport::new();

        // Basic content validation
        if script.trim().is_empty() {
            report.add_error("Script content is empty".to_string(), None);
            report.set_duration(
                start_time
                    .elapsed()
                    .as_micros()
                    .min(u128::from(u64::MAX))
                    .try_into()
                    .unwrap_or(u64::MAX),
            );
            return Ok(report);
        }

        if script.len() > 1_000_000 {
            report.add_error("Script content too large (>1MB)".to_string(), None);
        }

        // Syntax validation using ConditionEvaluator trait patterns
        if let Some(condition_evaluator) = &self.condition_evaluator {
            match condition_evaluator.compile_condition(script) {
                Err(compilation_error) => {
                    let enriched_error = self.enrich_diagnostic(&compilation_error.to_string());
                    report.add_error(enriched_error, None);
                    tracing::error!(error = %compilation_error, "Syntax validation failed");
                }
                Ok(_) => {
                    tracing::info!("Syntax validation passed");
                }
            }
        }

        // API validation using VariableInspector trait patterns
        if let Some(variable_inspector) = &self.variable_inspector {
            match variable_inspector.validate_api_usage(script, context) {
                Ok(api_violations) => {
                    for violation in api_violations {
                        report.add_warning("api".to_string(), violation.clone(), None);
                        tracing::warn!(violation = %violation, "API validation warning");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "API validation failed");
                    report.add_error(format!("API validation error: {e}"), None);
                }
            }
        }

        // Performance validation using established metrics
        let metrics = &context.performance_metrics;
        if metrics.execution_count > 10000 {
            let perf_warning = format!(
                "High execution count ({}) - consider optimization",
                metrics.execution_count
            );
            report.add_warning("performance".to_string(), perf_warning, None);
            tracing::warn!(
                execution_count = metrics.execution_count,
                "Performance validation warning"
            );
        }

        if metrics.memory_allocated > 100_000_000 {
            // 100MB
            let mem_warning = format!(
                "High memory allocation ({} bytes) - check for memory leaks",
                metrics.memory_allocated
            );
            report.add_warning("performance".to_string(), mem_warning, None);
        }

        // Security validation with trace enrichment
        let security_issues = self.detect_security_patterns(script);
        for issue in security_issues {
            report.add_warning("security".to_string(), issue.clone(), None);
            tracing::warn!(security_issue = %issue, "Security validation warning");
        }

        // Performance suggestions
        if script.lines().count() > 1000 {
            report.add_suggestion(
                "performance".to_string(),
                "Large script detected - consider splitting into modules".to_string(),
                "Break script into smaller, focused functions or modules".to_string(),
            );
        }

        report.set_duration(
            start_time
                .elapsed()
                .as_micros()
                .min(u128::from(u64::MAX))
                .try_into()
                .unwrap_or(u64::MAX),
        );
        tracing::info!(
            duration_us = report.validation_duration_us,
            errors = report.errors.len(),
            warnings = report.warnings.len(),
            suggestions = report.suggestions.len(),
            "Script validation completed"
        );

        Ok(report)
    }

    /// Basic script validation for hot reload (legacy method)
    #[must_use]
    pub fn validate_script(&self, content: &str, _script_type: &str) -> ValidationResult {
        use llmspell_tools::util::ValidationError;

        // Basic validation for backwards compatibility
        if content.trim().is_empty() {
            ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    field: "content".to_string(),
                    value: serde_json::Value::String(content.to_string()),
                    rule: "not_empty".to_string(),
                    message: "Script content is empty".to_string(),
                }],
            }
        } else if content.len() > 1_000_000 {
            ValidationResult {
                valid: false,
                errors: vec![ValidationError {
                    field: "content".to_string(),
                    value: serde_json::Value::String("<<TRUNCATED>>".to_string()),
                    rule: "max_size".to_string(),
                    message: "Script content too large (>1MB)".to_string(),
                }],
            }
        } else {
            ValidationResult {
                valid: true,
                errors: vec![],
            }
        }
    }

    /// Detect security patterns in script content with distributed tracing
    #[allow(clippy::cognitive_complexity)]
    fn detect_security_patterns(&self, script: &str) -> Vec<String> {
        let span = tracing::debug_span!("security_pattern_detection", script_length = script.len());
        let _enter = span.enter();

        let mut issues = Vec::new();

        // Common security patterns across languages
        let dangerous_patterns = [
            ("eval(", "Dynamic code evaluation detected"),
            ("exec(", "Code execution function detected"),
            ("system(", "System command execution detected"),
            ("shell_exec", "Shell execution detected"),
            ("passthru", "Pass-through execution detected"),
            ("file_get_contents(", "File access function detected"),
            ("fopen(", "File opening detected"),
            ("file_put_contents(", "File writing detected"),
            ("curl_exec", "HTTP request execution detected"),
            ("fsockopen", "Socket connection detected"),
            ("popen(", "Process execution detected"),
        ];

        for (pattern, message) in &dangerous_patterns {
            if script.contains(pattern) {
                let enriched_message = self.enrich_diagnostic(message);
                issues.push(enriched_message);
                tracing::warn!(
                    pattern = pattern,
                    message = message,
                    "Security pattern detected"
                );
            }
        }

        // Check for SQL injection patterns
        if (script.contains("SELECT ")
            || script.contains("INSERT ")
            || script.contains("UPDATE ")
            || script.contains("DELETE "))
            && !script.contains("prepare")
            && !script.contains("bind")
        {
            let sql_warning =
                "SQL queries detected without prepared statements - potential injection risk";
            issues.push(self.enrich_diagnostic(sql_warning));
            tracing::warn!(message = sql_warning, "SQL injection pattern detected");
        }

        // Check for hardcoded credentials patterns
        let credential_patterns = ["password=", "pwd=", "secret=", "key=", "token=", "api_key="];

        for pattern in &credential_patterns {
            if script.contains(pattern) {
                let cred_warning =
                    format!("Potential hardcoded credential pattern '{pattern}' detected");
                issues.push(self.enrich_diagnostic(&cred_warning));
                tracing::warn!(pattern = pattern, "Credential pattern detected");
            }
        }

        // Check for path traversal patterns
        if script.contains("../") || script.contains("..\\") {
            let path_warning =
                "Path traversal pattern detected - potential directory traversal attack";
            issues.push(self.enrich_diagnostic(path_warning));
            tracing::warn!(message = path_warning, "Path traversal pattern detected");
        }

        // Check for XSS patterns
        if script.contains("<script")
            || script.contains("javascript:")
            || script.contains("onclick=")
        {
            let xss_warning = "Potential XSS pattern detected in script content";
            issues.push(self.enrich_diagnostic(xss_warning));
            tracing::warn!(message = xss_warning, "XSS pattern detected");
        }

        tracing::debug!(
            issues_found = issues.len(),
            "Security pattern detection completed"
        );
        issues
    }

    /// Enable hot reload for specified files
    ///
    /// # Errors
    /// Returns error if file watcher cannot be created or files cannot be watched
    #[tracing::instrument(skip(self))]
    pub async fn enable_hot_reload(&self, files: Vec<PathBuf>) -> Result<(), anyhow::Error> {
        let span = tracing::info_span!("hot_reload_enable", file_count = files.len());
        let _enter = span.enter();

        let (tx, rx) = mpsc::channel();
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;

        // Watch all specified files
        let file_count = {
            let mut watched_files = self.watched_files.lock();
            for file in files {
                if file.exists() {
                    watcher.watch(&file, RecursiveMode::NonRecursive)?;
                    watched_files.insert(
                        file.clone(),
                        Arc::new(Mutex::new(SharedExecutionContext::new())),
                    );
                    tracing::info!(file = %file.display(), "File added to hot reload watch");
                } else {
                    tracing::warn!(file = %file.display(), "File does not exist, skipping watch");
                }
            }
            watched_files.len()
        };

        // Store watcher and receiver
        *self.hot_reload_watcher.lock() = Some(watcher);
        *self.hot_reload_receiver.lock() = Some(rx);

        tracing::info!("Hot reload enabled for {} files", file_count);
        Ok(())
    }

    /// Handle file change events with context preservation
    ///
    /// # Errors
    /// Returns error if file content cannot be read or validation fails
    #[tracing::instrument(skip(self, event))]
    pub async fn handle_file_change(&self, event: notify::Event) -> Result<(), anyhow::Error> {
        let span = tracing::info_span!("hot_reload_handle_change",
            event_kind = ?event.kind,
            paths = ?event.paths
        );
        let _enter = span.enter();

        for path in &event.paths {
            // Check if path is watched (without holding lock)
            let context_ref = {
                let watched = self.watched_files.lock();
                watched.get(path).cloned()
            };

            if let Some(context_ref) = context_ref {
                // Preserve context before async operations
                let snapshot = {
                    let context = context_ref.lock();
                    context.preserve_across_async_boundary()
                };

                match event.kind {
                    notify::EventKind::Modify(_) => {
                        if let Ok(content) = tokio::fs::read_to_string(path).await {
                            self.emit_hot_reload_event(HotReloadEvent::Modified {
                                path: path.clone(),
                                content,
                            })
                            .await?;
                        }
                    }
                    notify::EventKind::Create(_) => {
                        if let Ok(content) = tokio::fs::read_to_string(path).await {
                            self.emit_hot_reload_event(HotReloadEvent::Created {
                                path: path.clone(),
                                content,
                            })
                            .await?;
                        }
                    }
                    notify::EventKind::Remove(_) => {
                        self.emit_hot_reload_event(HotReloadEvent::Deleted { path: path.clone() })
                            .await?;
                    }
                    _ => {
                        tracing::debug!(path = %path.display(), event_kind = ?event.kind, "Ignoring file event");
                    }
                }

                // Restore context after handling
                {
                    let mut context = context_ref.lock();
                    context.restore_from_async_boundary(snapshot);
                }
            }
        }

        Ok(())
    }

    /// Emit hot reload event with distributed tracing
    #[tracing::instrument(skip(self, event))]
    async fn emit_hot_reload_event(&self, event: HotReloadEvent) -> Result<(), anyhow::Error> {
        let start_time = std::time::Instant::now();

        match &event {
            HotReloadEvent::Modified { path, .. } | HotReloadEvent::Created { path, .. } => {
                // Validate script content if configured to do so
                if let Ok(content) = tokio::fs::read_to_string(path).await {
                    let should_validate = self.hot_reload_config.validate_before_reload;
                    let validation_result = if should_validate {
                        self.validate_script(&content, "lua")
                    } else {
                        ValidationResult {
                            valid: true,
                            errors: vec![],
                        }
                    };

                    if validation_result.valid {
                        let duration_ms = u64::try_from(
                            start_time.elapsed().as_millis().min(u128::from(u64::MAX)),
                        )
                        .unwrap_or(u64::MAX);

                        tracing::info!(
                            path = %path.display(),
                            duration_ms = duration_ms,
                            "Hot reload successful"
                        );
                    } else {
                        tracing::error!(
                            path = %path.display(),
                            errors = ?validation_result.errors.iter().map(|e| &e.message).collect::<Vec<_>>(),
                            "Hot reload failed validation"
                        );
                    }
                }
            }
            HotReloadEvent::Deleted { path } => {
                tracing::info!(path = %path.display(), "File deleted, removing from watch");
                self.watched_files.lock().remove(path);
            }
            HotReloadEvent::ReloadSuccess { path, duration_ms } => {
                tracing::info!(
                    path = %path.display(),
                    duration_ms = duration_ms,
                    "Hot reload completed successfully"
                );
            }
            HotReloadEvent::ReloadFailed { path, errors } => {
                tracing::error!(
                    path = %path.display(),
                    errors = ?errors,
                    "Hot reload failed"
                );
            }
        }

        Ok(())
    }

    /// Start hot reload event processing loop
    ///
    /// # Errors
    /// Returns error if processing loop cannot be started
    pub fn start_hot_reload_processing(&self) -> Result<(), anyhow::Error> {
        let receiver = self.hot_reload_receiver.lock().take();

        if let Some(receiver) = receiver {
            let self_clone = self.clone();
            tokio::spawn(async move {
                loop {
                    if let Ok(result) = receiver.recv() {
                        match result {
                            Ok(event) => {
                                if let Err(e) = self_clone.handle_file_change(event).await {
                                    tracing::error!(error = %e, "Failed to handle file change");
                                }
                            }
                            Err(error) => {
                                tracing::error!(error = %error, "File watcher error");
                            }
                        }
                    } else {
                        tracing::info!("Hot reload receiver closed");
                        break;
                    }
                }
            });
        }

        Ok(())
    }

    /// Disable hot reload and cleanup resources
    pub fn disable_hot_reload(&self) {
        *self.hot_reload_watcher.lock() = None;
        *self.hot_reload_receiver.lock() = None;
        self.watched_files.lock().clear();
        tracing::info!("Hot reload disabled");
    }

    // === PROFILING METHODS ===

    /// Start CPU profiling with distributed tracing integration
    ///
    /// # Arguments
    /// * `context` - Shared execution context for profiling data coordination
    /// * `sample_rate_hz` - CPU sampling rate (default 100Hz for <5% overhead)
    ///
    /// # Errors
    /// Returns error if profiling cannot be started or is already active
    pub fn start_profiling(
        &mut self,
        context: Arc<tokio::sync::RwLock<SharedExecutionContext>>,
        sample_rate_hz: Option<u32>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if profiling is already active
        if self.profiler.is_active() {
            return Err("Profiling is already active".into());
        }

        let sample_rate = sample_rate_hz.unwrap_or(100);

        // Create trace span for profiling session
        let _span = tracing::info_span!(
            "profiling_session_start",
            sample_rate_hz = sample_rate,
            "profiler.type" = "cpu"
        );

        // Start CPU profiler with specified sample rate
        self.profiler
            .start(i32::try_from(sample_rate).unwrap_or(100))?;

        // Initialize profiling session
        let session = ProfilingSession {
            cpu_samples: Vec::new(),
            memory_samples: Vec::new(),
            start_time: std::time::Instant::now(),
            end_time: None,
            sample_rate_hz: sample_rate,
        };

        // Store session data
        self.profiling_context = Some(context);
        *self.profiling_session.lock() = Some(session);

        tracing::info!(
            sample_rate_hz = sample_rate,
            "CPU profiling started with distributed tracing integration"
        );

        Ok(())
    }

    /// Stop CPU profiling and finalize session
    ///
    /// # Errors
    /// Returns error if profiling is not active or cannot be stopped
    ///
    /// # Panics
    /// Panics if profiling session end time is not set (should not happen in practice)
    pub fn stop_profiling(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if profiling is active
        if !self.profiler.is_active() {
            return Err("Profiling is not active".into());
        }

        // Stop profiler and get report data
        let _profiling_data = self.profiler.stop()?;

        // Finalize profiling session
        {
            let mut session_guard = self.profiling_session.lock();
            if let Some(mut session) = session_guard.take() {
                session.end_time = Some(std::time::Instant::now());
                let duration = session.end_time.unwrap() - session.start_time;

                tracing::info!(
                    duration_ms = duration.as_millis(),
                    cpu_samples = session.cpu_samples.len(),
                    memory_samples = session.memory_samples.len(),
                    "CPU profiling session completed"
                );

                // Update SharedExecutionContext with profiling summary
                if let Some(context_ref) = &self.profiling_context {
                    let context = context_ref.clone();
                    let duration_us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);
                    tokio::task::spawn(async move {
                        let mut ctx = context.write().await;
                        // Use existing fields - store profiling time in function_time_us
                        ctx.performance_metrics.function_time_us += duration_us;
                    });
                }

                // Store final session data
                *session_guard = Some(session);
            }
        }

        // Clean up profiling context
        self.profiling_context = None;

        tracing::info!("CPU profiling stopped and session finalized");
        Ok(())
    }

    /// Generate flamegraph enhanced with `StackNavigator` trait
    ///
    /// # Returns
    /// Flamegraph data as SVG bytes
    ///
    /// # Errors
    /// Returns error if profiling data is not available or flamegraph generation fails
    pub fn generate_flamegraph(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        // Hold lock only as long as necessary to access session
        let session_guard = self.profiling_session.lock();
        let session = session_guard
            .as_ref()
            .ok_or("No profiling session available")?;

        let cpu_samples_len = session.cpu_samples.len();

        // Generate enhanced frames using StackNavigator if available
        let enhanced_frames = self.stack_navigator.as_ref().map_or_else(
            || Self::generate_basic_frames(session),
            |stack_navigator| Self::generate_enhanced_frames(session, stack_navigator.as_ref()),
        );

        drop(session_guard); // Explicitly drop the lock

        // Create trace span for flamegraph generation
        let _span = tracing::info_span!(
            "flamegraph_generation",
            cpu_samples = cpu_samples_len,
            "flamegraph.enhanced" = self.stack_navigator.is_some()
        );

        // Create flamegraph from enhanced data
        let mut flamegraph_data = Vec::new();
        Self::build_flamegraph(&enhanced_frames, &mut flamegraph_data)?;

        tracing::info!(
            flamegraph_size_bytes = flamegraph_data.len(),
            enhanced_frames = enhanced_frames.len(),
            "Flamegraph generated successfully"
        );

        Ok(flamegraph_data)
    }

    /// Sample stack for profiling (called from debug hooks)
    pub fn sample_stack_for_profiling(&self, stack: Vec<StackFrame>) {
        if let Some(session) = self.profiling_session.lock().as_mut() {
            let sample = CpuSample {
                timestamp: std::time::Instant::now(),
                stack,
                thread_id: None, // Skip thread ID for now since it requires unstable API
            };

            session.cpu_samples.push(sample);

            // Update context with sample count periodically
            if session.cpu_samples.len() % 100 == 0 {
                if let Some(context_ref) = &self.profiling_context {
                    let context = context_ref.clone();
                    let sample_count = session.cpu_samples.len();
                    tokio::task::spawn(async move {
                        let mut ctx = context.write().await;
                        // Use existing execution_count field to track samples
                        ctx.performance_metrics.execution_count =
                            u32::try_from(sample_count).unwrap_or(u32::MAX);
                    });
                }
            }
        }
    }

    /// Sample memory usage for profiling
    ///
    /// # Arguments
    /// * `bytes_allocated` - Current memory allocation in bytes
    /// * `stack` - Optional stack trace for the allocation
    ///
    /// # Errors
    /// Returns error if memory sampling fails
    pub fn sample_memory(
        &self,
        bytes_allocated: u64,
        stack: Option<Vec<StackFrame>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(session) = self.profiling_session.lock().as_mut() {
            let sample = MemorySample {
                timestamp: std::time::Instant::now(),
                bytes_allocated,
                stack: stack.unwrap_or_default(),
            };

            session.memory_samples.push(sample);

            // Update SharedExecutionContext with memory data
            if let Some(context_ref) = &self.profiling_context {
                let context = context_ref.clone();
                let memory_allocated = bytes_allocated.try_into().unwrap_or(usize::MAX);
                tokio::task::spawn(async move {
                    {
                        let mut ctx = context.write().await;
                        ctx.performance_metrics.memory_allocated = memory_allocated;
                    } // Lock released here

                    // Log potential memory leak warning (simplified heuristic)
                    if bytes_allocated > 500_000_000 {
                        // 500MB threshold
                        tracing::warn!(
                            memory_mb = bytes_allocated / 1_000_000,
                            "High memory usage detected - potential leak"
                        );
                    }
                });
            }

            tracing::trace!(
                bytes_allocated = bytes_allocated,
                total_memory_samples = session.memory_samples.len(),
                "Memory sample collected"
            );
        }

        Ok(())
    }

    /// Update performance metrics with profiling data
    pub fn update_performance_metrics(&self, operation: &str, duration: std::time::Duration) {
        if let Some(context_ref) = &self.profiling_context {
            let context = context_ref.clone();
            let op_name = operation.to_string();
            let duration_us = u64::try_from(duration.as_micros()).unwrap_or(u64::MAX);

            tokio::task::spawn(async move {
                let mut ctx = context.write().await;

                // Update execution metrics using existing fields
                ctx.performance_metrics.execution_count += 1;
                ctx.performance_metrics.function_time_us += duration_us;

                tracing::trace!(
                    operation = op_name,
                    duration_us = duration_us,
                    total_executions = ctx.performance_metrics.execution_count,
                    cumulative_time_us = ctx.performance_metrics.function_time_us,
                    "Performance metrics updated via profiling"
                );
            });
        }
    }

    /// Generate enhanced frames using `StackNavigator` trait
    fn generate_enhanced_frames(
        session: &ProfilingSession,
        stack_navigator: &dyn StackNavigator,
    ) -> Vec<FlameGraphFrame> {
        let mut frame_map: std::collections::HashMap<String, FlameGraphFrame> =
            std::collections::HashMap::new();

        for sample in &session.cpu_samples {
            for frame in &sample.stack {
                let formatted = stack_navigator.format_frame(frame);
                let key = format!("{}:{}:{}", frame.source, frame.line, frame.name);

                frame_map
                    .entry(key)
                    .and_modify(|f| {
                        f.execution_count += 1;
                        f.total_time_us += 1000; // Approximate time per sample
                    })
                    .or_insert_with(|| FlameGraphFrame {
                        function: formatted,
                        file: frame.source.clone(),
                        line: frame.line,
                        execution_count: 1,
                        total_time_us: 1000,
                        self_time_us: 1000,
                    });
            }
        }

        frame_map.into_values().collect()
    }

    /// Generate basic frames without `StackNavigator` enhancement
    fn generate_basic_frames(session: &ProfilingSession) -> Vec<FlameGraphFrame> {
        let mut frame_map: std::collections::HashMap<String, FlameGraphFrame> =
            std::collections::HashMap::new();

        for sample in &session.cpu_samples {
            for frame in &sample.stack {
                let key = format!("{}:{}:{}", frame.source, frame.line, frame.name);

                frame_map
                    .entry(key)
                    .and_modify(|f| {
                        f.execution_count += 1;
                        f.total_time_us += 1000;
                    })
                    .or_insert_with(|| FlameGraphFrame {
                        function: format!("{}:{} in {}", frame.source, frame.line, frame.name),
                        file: frame.source.clone(),
                        line: frame.line,
                        execution_count: 1,
                        total_time_us: 1000,
                        self_time_us: 1000,
                    });
            }
        }

        frame_map.into_values().collect()
    }

    /// Build flamegraph from enhanced frames
    fn build_flamegraph(
        frames: &[FlameGraphFrame],
        output: &mut Vec<u8>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::fmt::Write;

        // Simple SVG flamegraph generation (basic implementation)
        let mut svg = String::new();
        writeln!(&mut svg, r#"<?xml version="1.0" standalone="no"?>"#)?;
        writeln!(
            &mut svg,
            r#"<svg version="1.1" width="1200" height="600" xmlns="http://www.w3.org/2000/svg">"#
        )?;

        let mut y = 50;
        for frame in frames {
            let width = (frame.execution_count * 10).min(1000); // Scale width
            let color = format!("hsl({}, 60%, 60%)", (frame.line * 13) % 360); // Deterministic color

            writeln!(
                &mut svg,
                r#"<rect x="50" y="{y}" width="{width}" height="20" fill="{color}" stroke="black"/>"#
            )?;

            writeln!(
                &mut svg,
                r#"<text x="{}" y="{}" font-family="monospace" font-size="12" fill="black">{}</text>"#,
                55,
                y + 15,
                frame.function
            )?;

            y += 25;
        }

        writeln!(&mut svg, "</svg>")?;
        *output = svg.into_bytes();

        Ok(())
    }

    /// Get profiling session data for testing and inspection
    pub fn get_profiling_session(&self) -> parking_lot::MutexGuard<'_, Option<ProfilingSession>> {
        self.profiling_session.lock()
    }

    /// Check if profiling is currently active
    #[must_use]
    pub fn is_profiling_active(&self) -> bool {
        self.profiler.is_active()
    }
}

impl Clone for DiagnosticsBridge {
    fn clone(&self) -> Self {
        Self {
            manager: self.manager.clone(),
            trackers: self.trackers.clone(),
            tracer: self.tracer.clone(),
            trace_enricher: self.trace_enricher.clone(),
            tracing_config: self.tracing_config.clone(),
            shared_context: self.shared_context.clone(),
            hot_reload_watcher: self.hot_reload_watcher.clone(),
            hot_reload_receiver: self.hot_reload_receiver.clone(),
            hot_reload_config: self.hot_reload_config.clone(),
            watched_files: self.watched_files.clone(),
            condition_evaluator: self.condition_evaluator.clone(),
            variable_inspector: self.variable_inspector.clone(),
            stack_navigator: self.stack_navigator.clone(),
            // Profiling state is not cloned - each clone starts without active profiling
            profiler: Box::new(PprofProfiler::new()),
            hook_profiler: Box::new(RealHookProfiler::new()),
            circuit_breaker: Box::new(ExponentialBackoffBreaker::default()),
            session_recorder: Box::new(JsonFileRecorder::new()),
            profiling_session: Arc::new(Mutex::new(None)),
            profiling_context: None,
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
        let enriched = context.location.as_ref().map_or_else(
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
