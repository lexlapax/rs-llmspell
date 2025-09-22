//! Observability framework traits for Phase 18/20: Production Optimization
//!
//! Provides traits and types for comprehensive observability including metrics,
//! tracing, and monitoring. Designed to integrate with Prometheus, OpenTelemetry,
//! and other standard observability tools.

use crate::error::LLMSpellError;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;

/// Metric type for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricType {
    /// Counter that only goes up
    Counter(u64),
    /// Gauge that can go up or down
    Gauge(f64),
    /// Histogram of values with buckets
    Histogram {
        /// The actual values
        values: Vec<f64>,
        /// Bucket boundaries for aggregation
        buckets: Vec<f64>,
    },
    /// Summary statistics
    Summary {
        /// Number of observations
        count: u64,
        /// Sum of all observations
        sum: f64,
        /// Minimum value
        min: f64,
        /// Maximum value
        max: f64,
        /// Average value
        avg: f64,
        /// Percentiles (50th, 90th, 95th, 99th)
        percentiles: HashMap<String, f64>,
    },
}

/// Performance metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// Metric name (e.g., "llmspell_requests_total")
    pub name: String,
    /// Metric type and value
    pub value: MetricType,
    /// Labels for the metric
    pub labels: HashMap<String, String>,
    /// Help text describing the metric
    pub help: Option<String>,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Handle to an active trace span
#[derive(Debug, Clone)]
pub struct SpanHandle {
    /// Span ID (unique identifier)
    pub id: String,
    /// Trace ID (groups related spans)
    pub trace_id: String,
    /// Parent span ID if nested
    pub parent_id: Option<String>,
    /// Operation name
    pub operation: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// Span attributes
    pub attributes: HashMap<String, String>,
}

impl SpanHandle {
    /// End the span and record duration
    #[allow(clippy::must_use_candidate)]
    pub fn end(self) -> std::time::Duration {
        let now = chrono::Utc::now();
        let duration = now.signed_duration_since(self.start_time);
        duration.to_std().unwrap_or_default()
    }
}

/// Health check status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System is unhealthy
    Unhealthy,
}

/// Health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Component name
    pub component: String,
    /// Health status
    pub status: HealthStatus,
    /// Optional message
    pub message: Option<String>,
    /// Check timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Additional details
    pub details: HashMap<String, serde_json::Value>,
}

/// Alert severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning alert
    Warning,
    /// Error alert
    Error,
    /// Critical alert requiring immediate attention
    Critical,
}

/// Alert definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Severity level
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Component that triggered the alert
    pub source: String,
    /// When the alert was triggered
    pub triggered_at: chrono::DateTime<chrono::Utc>,
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

/// Observability framework trait for Phase 20
///
/// This trait defines the interface for implementing comprehensive observability
/// including metrics collection, distributed tracing, health checks, and alerting.
/// Implementations should integrate with standard tools like Prometheus and OpenTelemetry.
#[async_trait]
pub trait ObservabilityFramework: Send + Sync + Debug {
    // === Metrics ===

    /// Record a metric
    async fn record_metric(&self, metric: Metric) -> Result<(), LLMSpellError>;

    /// Increment a counter
    async fn increment_counter(
        &self,
        name: &str,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError>;

    /// Set a gauge value
    async fn set_gauge(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError>;

    /// Record a histogram observation
    async fn observe_histogram(
        &self,
        name: &str,
        value: f64,
        labels: HashMap<String, String>,
    ) -> Result<(), LLMSpellError>;

    /// Get metrics matching filter
    async fn get_metrics(&self, name_prefix: &str) -> Result<Vec<Metric>, LLMSpellError>;

    // === Tracing ===

    /// Start a trace span
    async fn start_span(
        &self,
        operation_name: &str,
        parent_id: Option<&str>,
    ) -> Result<SpanHandle, LLMSpellError>;

    /// Add attributes to a span
    async fn add_span_attributes(
        &self,
        span_id: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), LLMSpellError>;

    /// Record an event on a span
    async fn add_span_event(
        &self,
        span_id: &str,
        event_name: &str,
        attributes: HashMap<String, String>,
    ) -> Result<(), LLMSpellError>;

    // === Health Checks ===

    /// Register a health check
    async fn register_health_check(&self, component: &str) -> Result<(), LLMSpellError>;

    /// Report health status
    async fn report_health(&self, check: HealthCheck) -> Result<(), LLMSpellError>;

    /// Get overall system health
    async fn get_system_health(&self) -> Result<Vec<HealthCheck>, LLMSpellError>;

    // === Alerting ===

    /// Trigger an alert
    async fn trigger_alert(&self, alert: Alert) -> Result<(), LLMSpellError>;

    /// Acknowledge an alert
    async fn acknowledge_alert(&self, alert_id: &str) -> Result<(), LLMSpellError>;

    /// Get active alerts
    async fn get_active_alerts(
        &self,
        severity: Option<AlertSeverity>,
    ) -> Result<Vec<Alert>, LLMSpellError>;

    // === Export ===

    /// Export metrics in Prometheus format
    async fn export_prometheus(&self) -> Result<String, LLMSpellError>;

    /// Export traces in OpenTelemetry format
    async fn export_opentelemetry(&self) -> Result<Vec<u8>, LLMSpellError>;

    /// Export metrics and traces to external collector
    async fn export_to_collector(&self, endpoint: &str) -> Result<(), LLMSpellError>;
}
