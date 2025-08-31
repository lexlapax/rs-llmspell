//! Diagnostics bridge for script engines
//!
//! Provides a unified interface for all script engines to access
//! the centralized diagnostics infrastructure (logging, profiling, metrics)
//! and distributed tracing via OpenTelemetry.

use crate::condition_evaluator::ConditionEvaluator;
use crate::execution_context::{ExecutionContextBridge, SharedExecutionContext, SourceLocation};
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
            hot_reload_watcher: Arc::new(Mutex::new(None)),
            hot_reload_receiver: Arc::new(Mutex::new(None)),
            hot_reload_config: HotReloadConfig::default(),
            watched_files: Arc::new(Mutex::new(HashMap::new())),
            condition_evaluator: None,
            variable_inspector: None,
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
