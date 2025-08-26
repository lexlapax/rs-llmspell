//! ABOUTME: Distributed tracing for agent operations
//! ABOUTME: Provides span tracking, context propagation, and trace collection

#![allow(clippy::significant_drop_tightening)]

use chrono::{DateTime, Utc};
use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use uuid::Uuid;

/// Trace span representing a unit of work
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceSpan {
    /// Span ID
    pub span_id: String,
    /// Trace ID (shared across all spans in a trace)
    pub trace_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Operation name
    pub operation: String,
    /// Service name
    pub service: String,
    /// Start time
    pub start_time: DateTime<Utc>,
    /// End time (if completed)
    pub end_time: Option<DateTime<Utc>>,
    /// Duration (if completed)
    pub duration: Option<Duration>,
    /// Tags/labels
    pub tags: HashMap<String, String>,
    /// Events that occurred during the span
    pub events: Vec<TraceEvent>,
    /// Status
    pub status: SpanStatus,
}

impl TraceSpan {
    /// Create a new root span
    #[must_use]
    pub fn new_root(operation: String, service: String) -> Self {
        let trace_id = Uuid::new_v4().to_string();
        Self::new(trace_id, None, operation, service)
    }

    /// Create a new child span
    #[must_use]
    pub fn new_child(&self, operation: String) -> Self {
        Self::new(
            self.trace_id.clone(),
            Some(self.span_id.clone()),
            operation,
            self.service.clone(),
        )
    }

    /// Create a new span
    fn new(
        trace_id: String,
        parent_span_id: Option<String>,
        operation: String,
        service: String,
    ) -> Self {
        Self {
            span_id: Uuid::new_v4().to_string(),
            trace_id,
            parent_span_id,
            operation,
            service,
            start_time: Utc::now(),
            end_time: None,
            duration: None,
            tags: HashMap::new(),
            events: Vec::new(),
            status: SpanStatus::InProgress,
        }
    }

    /// Add a tag to the span
    pub fn add_tag(&mut self, key: String, value: String) {
        self.tags.insert(key, value);
    }

    /// Add an event to the span
    pub fn add_event(&mut self, event: TraceEvent) {
        self.events.push(event);
    }

    /// Complete the span
    ///
    /// # Panics
    ///
    /// Panics if the `end_time` calculation results in a negative `duration`
    pub fn complete(&mut self, status: SpanStatus) {
        self.end_time = Some(Utc::now());
        self.duration = Some(
            (self.end_time.unwrap() - self.start_time)
                .to_std()
                .unwrap_or_default(),
        );
        self.status = status;
    }

    /// Get span duration in milliseconds
    #[must_use]
    pub fn duration_ms(&self) -> Option<f64> {
        self.duration.map(|d| d.as_secs_f64() * 1000.0)
    }
}

/// Span status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpanStatus {
    /// Span is still in progress
    InProgress,
    /// Span completed successfully
    Ok,
    /// Span completed with error
    Error,
    /// Span was cancelled
    Cancelled,
}

/// Event that occurred during a span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceEvent {
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Event name
    pub name: String,
    /// Event attributes
    pub attributes: HashMap<String, String>,
}

impl TraceEvent {
    /// Create a new trace event
    #[must_use]
    pub fn new(name: String) -> Self {
        Self {
            timestamp: Utc::now(),
            name,
            attributes: HashMap::new(),
        }
    }

    /// Add an attribute to the event
    #[must_use]
    pub fn with_attribute(mut self, key: String, value: String) -> Self {
        self.attributes.insert(key, value);
        self
    }
}

/// Span context for propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: String,
    /// Span ID
    pub span_id: String,
    /// Service name
    pub service: String,
    /// Baggage items for propagation
    pub baggage: HashMap<String, String>,
}

impl SpanContext {
    /// Create from a span
    #[must_use]
    pub fn from_span(span: &TraceSpan) -> Self {
        Self {
            trace_id: span.trace_id.clone(),
            span_id: span.span_id.clone(),
            service: span.service.clone(),
            baggage: HashMap::new(),
        }
    }

    /// Add baggage item
    pub fn add_baggage(&mut self, key: String, value: String) {
        self.baggage.insert(key, value);
    }

    /// Create a child span from this context
    #[must_use]
    pub fn child_span(&self, operation: String) -> TraceSpan {
        TraceSpan::new(
            self.trace_id.clone(),
            Some(self.span_id.clone()),
            operation,
            self.service.clone(),
        )
    }
}

/// Trace collector for managing spans
pub struct TraceCollector {
    /// Active spans
    active_spans: Arc<RwLock<HashMap<String, TraceSpan>>>,
    /// Completed spans (ring buffer)
    completed_spans: Arc<RwLock<Vec<TraceSpan>>>,
    /// Maximum completed spans to keep
    max_completed_spans: usize,
    /// Trace exporters
    exporters: Vec<Box<dyn TraceExporter>>,
}

impl std::fmt::Debug for TraceCollector {
    /// # Panics
    ///
    /// Panics if any `RwLock` is poisoned
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TraceCollector")
            .field(
                "active_spans_count",
                &self.active_spans.read().unwrap().len(),
            )
            .field(
                "completed_spans_count",
                &self.completed_spans.read().unwrap().len(),
            )
            .field("max_completed_spans", &self.max_completed_spans)
            .field("exporters_count", &self.exporters.len())
            .finish()
    }
}

impl TraceCollector {
    /// Create a new trace collector
    #[must_use]
    pub fn new(max_completed_spans: usize) -> Self {
        Self {
            active_spans: Arc::new(RwLock::new(HashMap::new())),
            completed_spans: Arc::new(RwLock::new(Vec::with_capacity(max_completed_spans))),
            max_completed_spans,
            exporters: Vec::new(),
        }
    }

    /// Add a trace exporter
    pub fn add_exporter(&mut self, exporter: Box<dyn TraceExporter>) {
        self.exporters.push(exporter);
    }

    /// Start a new span
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn start_span(self: &Arc<Self>, span: TraceSpan) -> SpanHandle {
        let span_id = span.span_id.clone();
        self.active_spans
            .write()
            .unwrap()
            .insert(span_id.clone(), span);

        SpanHandle {
            span_id,
            collector: Arc::clone(self),
        }
    }

    /// Complete a span
    ///
    /// # Panics
    ///
    /// Panics if any `RwLock` is poisoned
    fn complete_span(&self, span_id: &str, status: SpanStatus) -> Result<()> {
        let mut active = self.active_spans.write().unwrap();

        if let Some(mut span) = active.remove(span_id) {
            span.complete(status);

            // Export to all exporters
            for exporter in &self.exporters {
                exporter.export(&span)?;
            }

            // Store in completed spans
            let mut completed = self.completed_spans.write().unwrap();
            if completed.len() >= self.max_completed_spans {
                completed.remove(0);
            }
            completed.push(span);
        }

        Ok(())
    }

    /// Get active span count
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn active_span_count(&self) -> usize {
        self.active_spans.read().unwrap().len()
    }

    /// Get completed spans for a trace
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    #[must_use]
    pub fn get_trace(&self, trace_id: &str) -> Vec<TraceSpan> {
        self.completed_spans
            .read()
            .unwrap()
            .iter()
            .filter(|span| span.trace_id == trace_id)
            .cloned()
            .collect()
    }

    /// Get all traces
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    pub fn get_all_traces(&self) -> HashMap<String, Vec<TraceSpan>> {
        let mut traces = HashMap::new();

        for span in self.completed_spans.read().unwrap().iter() {
            traces
                .entry(span.trace_id.clone())
                .or_insert_with(Vec::new)
                .push(span.clone());
        }

        traces
    }

    /// Clear all completed spans
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    pub fn clear_completed(&self) {
        self.completed_spans.write().unwrap().clear();
    }
}

/// Handle to an active span
pub struct SpanHandle {
    span_id: String,
    collector: Arc<TraceCollector>,
}

impl SpanHandle {
    /// Add a tag to the span
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    pub fn add_tag(&self, key: String, value: String) {
        if let Some(span) = self
            .collector
            .active_spans
            .write()
            .unwrap()
            .get_mut(&self.span_id)
        {
            span.add_tag(key, value);
        }
    }

    /// Add an event to the span
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned
    pub fn add_event(&self, event: TraceEvent) {
        if let Some(span) = self
            .collector
            .active_spans
            .write()
            .unwrap()
            .get_mut(&self.span_id)
        {
            span.add_event(event);
        }
    }

    /// Complete the span successfully
    pub fn complete_ok(self) {
        let _ = self.collector.complete_span(&self.span_id, SpanStatus::Ok);
    }

    /// Complete the span with error
    pub fn complete_error(self) {
        let _ = self
            .collector
            .complete_span(&self.span_id, SpanStatus::Error);
    }

    /// Complete the span as cancelled
    pub fn complete_cancelled(self) {
        let _ = self
            .collector
            .complete_span(&self.span_id, SpanStatus::Cancelled);
    }
}

/// Trait for exporting traces
pub trait TraceExporter: Send + Sync {
    /// Export a completed span
    ///
    /// # Errors
    ///
    /// Returns an error if the span cannot be exported due to network issues,
    /// serialization failures, or backend service unavailability.
    fn export(&self, span: &TraceSpan) -> Result<()>;
}

/// Simple console trace exporter
#[derive(Debug)]
pub struct ConsoleTraceExporter;

impl TraceExporter for ConsoleTraceExporter {
    fn export(&self, span: &TraceSpan) -> Result<()> {
        let duration_ms = span.duration_ms().unwrap_or(0.0);
        tracing::info!(
            "[TRACE] {} {} - {} ({:.2}ms) - {:?}",
            span.service,
            span.operation,
            span.span_id,
            duration_ms,
            span.status
        );
        Ok(())
    }
}

/// Trace analysis utilities
pub struct TraceAnalyzer;

impl TraceAnalyzer {
    /// Calculate critical path in a trace
    #[must_use]
    pub fn critical_path(spans: &[TraceSpan]) -> Vec<String> {
        if spans.is_empty() {
            return Vec::new();
        }

        // Find root span
        let root = spans.iter().find(|s| s.parent_span_id.is_none()).cloned();

        if let Some(root) = root {
            let mut path = vec![root.span_id.clone()];
            Self::find_longest_path(&root, spans, &mut path);
            path
        } else {
            Vec::new()
        }
    }

    fn find_longest_path(current: &TraceSpan, all_spans: &[TraceSpan], path: &mut Vec<String>) {
        // Find children
        let children: Vec<_> = all_spans
            .iter()
            .filter(|s| s.parent_span_id.as_ref() == Some(&current.span_id))
            .collect();

        if children.is_empty() {
            return;
        }

        // Find child with longest duration
        let longest_child = children
            .iter()
            .max_by_key(|s| s.duration.unwrap_or(Duration::ZERO))
            .unwrap();

        path.push(longest_child.span_id.clone());
        Self::find_longest_path(longest_child, all_spans, path);
    }

    /// Calculate trace statistics
    #[must_use]
    pub fn trace_stats(spans: &[TraceSpan]) -> TraceStatistics {
        if spans.is_empty() {
            return TraceStatistics::default();
        }

        let total_duration = spans.iter().filter_map(|s| s.duration).sum::<Duration>();

        let error_count = spans
            .iter()
            .filter(|s| s.status == SpanStatus::Error)
            .count();

        let service_durations = Self::calculate_service_durations(spans);

        #[allow(clippy::cast_precision_loss)]
        let error_count_f64 = error_count as f64;
        #[allow(clippy::cast_precision_loss)]
        let span_count_f64 = spans.len() as f64;

        TraceStatistics {
            span_count: spans.len(),
            total_duration,
            error_count,
            error_rate: (error_count_f64 / span_count_f64) * 100.0,
            service_durations,
        }
    }

    fn calculate_service_durations(spans: &[TraceSpan]) -> HashMap<String, Duration> {
        let mut durations = HashMap::new();

        for span in spans {
            if let Some(duration) = span.duration {
                *durations
                    .entry(span.service.clone())
                    .or_insert(Duration::ZERO) += duration;
            }
        }

        durations
    }
}

/// Trace statistics
#[derive(Debug, Default)]
pub struct TraceStatistics {
    /// Total number of spans
    pub span_count: usize,
    /// Total duration of all spans
    pub total_duration: Duration,
    /// Number of error spans
    pub error_count: usize,
    /// Error rate percentage
    pub error_rate: f64,
    /// Duration by service
    pub service_durations: HashMap<String, Duration>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_span_creation() {
        let root = TraceSpan::new_root("operation".to_string(), "service".to_string());
        assert!(root.parent_span_id.is_none());
        assert_eq!(root.operation, "operation");
        assert_eq!(root.service, "service");

        let child = root.new_child("child-op".to_string());
        assert_eq!(child.trace_id, root.trace_id);
        assert_eq!(child.parent_span_id, Some(root.span_id.clone()));
        assert_eq!(child.service, root.service);
    }
    #[test]
    fn test_span_completion() {
        let mut span = TraceSpan::new_root("test".to_string(), "service".to_string());

        // Add tag and event
        span.add_tag("key".to_string(), "value".to_string());
        span.add_event(TraceEvent::new("event".to_string()));

        // Complete span
        std::thread::sleep(Duration::from_millis(10));
        span.complete(SpanStatus::Ok);

        assert!(span.end_time.is_some());
        assert!(span.duration.is_some());
        assert!(span.duration.unwrap() >= Duration::from_millis(10));
        assert_eq!(span.status, SpanStatus::Ok);
    }
    #[test]
    fn test_trace_collector() {
        let collector = Arc::new(TraceCollector::new(10));

        // Start a span
        let span = TraceSpan::new_root("test-op".to_string(), "test-service".to_string());
        let span_id = span.span_id.clone();
        let trace_id = span.trace_id.clone();

        let handle = collector.start_span(span);
        assert_eq!(collector.active_span_count(), 1);

        // Add tag and event
        handle.add_tag("test".to_string(), "value".to_string());
        handle.add_event(TraceEvent::new("test-event".to_string()));

        // Complete span
        handle.complete_ok();
        assert_eq!(collector.active_span_count(), 0);

        // Check completed spans
        let trace = collector.get_trace(&trace_id);
        assert_eq!(trace.len(), 1);
        assert_eq!(trace[0].span_id, span_id);
        assert_eq!(trace[0].status, SpanStatus::Ok);
    }
    #[test]
    fn test_span_context() {
        let span = TraceSpan::new_root("op".to_string(), "service".to_string());
        let mut context = SpanContext::from_span(&span);

        context.add_baggage("user_id".to_string(), "123".to_string());

        let child = context.child_span("child-op".to_string());
        assert_eq!(child.trace_id, span.trace_id);
        assert_eq!(child.parent_span_id, Some(span.span_id));
    }
    #[test]
    fn test_trace_analyzer() {
        let root = TraceSpan::new_root("root".to_string(), "service-a".to_string());
        let mut child1 = root.new_child("child1".to_string());
        let mut child2 = root.new_child("child2".to_string());

        // Simulate different durations
        child1.duration = Some(Duration::from_millis(100));
        child2.duration = Some(Duration::from_millis(200));

        let spans = vec![root.clone(), child1, child2];

        // Test critical path
        let path = TraceAnalyzer::critical_path(&spans);
        assert_eq!(path.len(), 2); // root + longest child
        assert_eq!(path[0], root.span_id);

        // Test statistics
        let stats = TraceAnalyzer::trace_stats(&spans);
        assert_eq!(stats.span_count, 3);
        assert_eq!(stats.error_count, 0);
        assert_eq!(stats.total_duration, Duration::from_millis(300));
    }
}
