//! Script-agnostic tracing traits for distributed observability
//!
//! This module provides trait definitions for tracing that are independent
//! of any specific script engine implementation (no mlua/v8/pyo3 imports).
//! It follows the three-layer bridge architecture pattern.

use crate::execution_context::SharedExecutionContext;
use opentelemetry::{
    trace::{SpanKind, Status},
    Context, KeyValue,
};
use std::sync::Arc;

/// Trait for tracing script execution and diagnostics
///
/// This trait provides script-agnostic methods for distributed tracing,
/// designed to work with diagnostics infrastructure (not execution debugging).
pub trait ScriptTracer: Send + Sync {
    /// Start a new trace span for a script operation
    fn start_span(&self, operation: &str, context: &SharedExecutionContext) -> Box<dyn SpanHandle>;

    /// Record an event within the current span
    fn record_event(&self, name: &str, attributes: Vec<KeyValue>);

    /// Set span status to indicate success or failure
    fn set_status(&self, status: Status);

    /// Add attributes to the current span
    fn add_attributes(&self, attributes: Vec<KeyValue>);

    /// Get the current trace context
    fn current_context(&self) -> Context;

    /// Extract trace context from `SharedExecutionContext` correlation ID
    fn extract_context(&self, context: &SharedExecutionContext) -> Option<Context>;

    /// Inject trace context into `SharedExecutionContext`
    fn inject_context(&self, context: &mut SharedExecutionContext, trace_context: &Context);
}

/// Handle for managing a trace span lifecycle
pub trait SpanHandle: Send + Sync {
    /// End the span
    fn end(self: Box<Self>);

    /// Record an exception in the span
    fn record_exception(&mut self, exception: &str, stacktrace: Option<&str>);

    /// Add an attribute to the span
    fn set_attribute(&mut self, key: &str, value: String);

    /// Get the span context for propagation
    fn context(&self) -> Context;
}

/// Trait for enriching traces with diagnostic context
///
/// This trait is used by `DiagnosticsBridge` to enrich traces with
/// execution context information.
pub trait TraceEnricher: Send + Sync {
    /// Enrich a span with `SharedExecutionContext` data
    fn enrich_span(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext);

    /// Create attributes from `SharedExecutionContext`
    fn context_to_attributes(&self, context: &SharedExecutionContext) -> Vec<KeyValue>;

    /// Enrich with performance metrics
    fn enrich_with_performance(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext);

    /// Enrich with location information
    fn enrich_with_location(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext);
}

/// Factory trait for creating tracers
///
/// This allows different script engines to create their own tracer implementations
/// while maintaining the same interface.
pub trait TracerFactory: Send + Sync {
    /// Create a new tracer instance
    fn create_tracer(&self, name: &str) -> Arc<dyn ScriptTracer>;

    /// Configure OTLP exporter
    ///
    /// # Errors
    ///
    /// Returns an error if OTLP configuration fails
    fn configure_otlp(&self, endpoint: &str) -> Result<(), String>;

    /// Shutdown the tracer provider
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails
    fn shutdown(&self) -> Result<(), String>;
}

/// Default implementation of `TraceEnricher`
pub struct DefaultTraceEnricher;

impl TraceEnricher for DefaultTraceEnricher {
    fn enrich_span(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext) {
        self.enrich_with_location(span, context);
        self.enrich_with_performance(span, context);

        // Add correlation ID if present
        if let Some(correlation_id) = context.correlation_id {
            span.set_attribute("correlation.id", correlation_id.to_string());
        }

        // Add variables count if present
        if !context.variables.is_empty() {
            span.set_attribute("variables.count", context.variables.len().to_string());
        }
    }

    fn context_to_attributes(&self, context: &SharedExecutionContext) -> Vec<KeyValue> {
        let mut attributes = Vec::new();

        // Add location attributes
        if let Some(ref location) = context.location {
            attributes.push(KeyValue::new("source.file", location.source.clone()));
            attributes.push(KeyValue::new("source.line", i64::from(location.line)));
            if let Some(column) = location.column {
                attributes.push(KeyValue::new("source.column", i64::from(column)));
            }
        }

        // Add correlation ID
        if let Some(correlation_id) = context.correlation_id {
            attributes.push(KeyValue::new("correlation.id", correlation_id.to_string()));
        }

        // Add performance metrics
        attributes.push(KeyValue::new(
            "performance.execution_count",
            i64::from(context.performance_metrics.execution_count),
        ));
        attributes.push(KeyValue::new(
            "performance.function_time_ms",
            #[allow(clippy::cast_precision_loss)]
            (context.performance_metrics.function_time_us as f64 / 1000.0).to_string(),
        ));

        attributes
    }

    fn enrich_with_performance(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext) {
        let metrics = &context.performance_metrics;
        span.set_attribute(
            "performance.execution_count",
            metrics.execution_count.to_string(),
        );
        span.set_attribute(
            "performance.function_time_ms",
            #[allow(clippy::cast_precision_loss)]
            (metrics.function_time_us as f64 / 1000.0).to_string(),
        );
        span.set_attribute(
            "performance.memory_allocated",
            metrics.memory_allocated.to_string(),
        );

        if metrics.function_time_us > 0 && metrics.execution_count > 0 {
            #[allow(clippy::cast_precision_loss)]
            let avg_time =
                (metrics.function_time_us as f64 / 1000.0) / f64::from(metrics.execution_count);
            span.set_attribute("performance.avg_time_ms", avg_time.to_string());
        }
    }

    fn enrich_with_location(&self, span: &mut dyn SpanHandle, context: &SharedExecutionContext) {
        if let Some(ref location) = context.location {
            span.set_attribute("source.file", location.source.clone());
            span.set_attribute("source.line", location.line.to_string());
            if let Some(column) = location.column {
                span.set_attribute("source.column", column.to_string());
            }
        }
    }
}

/// Tracing configuration
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Enable distributed tracing
    pub enabled: bool,
    /// OTLP endpoint URL
    pub otlp_endpoint: String,
    /// Service name for traces
    pub service_name: String,
    /// Sampling rate (0.0 to 1.0)
    pub sampling_rate: f64,
    /// Maximum attributes per span
    pub max_attributes_per_span: u32,
    /// Maximum events per span
    pub max_events_per_span: u32,
    /// Enable automatic context propagation
    pub auto_propagate: bool,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "llmspell".to_string(),
            sampling_rate: 1.0,
            max_attributes_per_span: 128,
            max_events_per_span: 128,
            auto_propagate: true,
        }
    }
}

/// Span kind for script operations
#[derive(Debug, Clone, Copy)]
pub enum ScriptSpanKind {
    /// Script execution
    Execution,
    /// Tool invocation
    ToolInvocation,
    /// Agent execution
    AgentExecution,
    /// Diagnostic event
    Diagnostic,
    /// Debug event (not breakpoint)
    DebugEvent,
}

impl From<ScriptSpanKind> for SpanKind {
    fn from(kind: ScriptSpanKind) -> Self {
        match kind {
            ScriptSpanKind::ToolInvocation => Self::Client,
            ScriptSpanKind::Execution
            | ScriptSpanKind::AgentExecution
            | ScriptSpanKind::Diagnostic
            | ScriptSpanKind::DebugEvent => Self::Internal,
        }
    }
}
