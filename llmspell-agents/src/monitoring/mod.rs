//! ABOUTME: Monitoring and observability infrastructure for agents
//! ABOUTME: Provides metrics, health checks, tracing, logging, and alerting capabilities

pub mod alerts;
pub mod events;
pub mod health;
pub mod metrics;
pub mod performance;
pub mod tracing;

// Re-export main monitoring types
pub use alerts::{Alert, AlertConfig, AlertManager, AlertRule, AlertSeverity};
pub use events::{EventLogger, LogEvent, LogLevel};
pub use health::{ComponentHealth, HealthCheck, HealthCheckResult, HealthIndicator, HealthStatus};
pub use metrics::{
    AgentMetrics, Counter, Gauge, Histogram, MetricLabel, MetricRegistry, MetricType, MetricValue,
};
pub use performance::{PerformanceMonitor, PerformanceReport, PerformanceSnapshot, ResourceUsage};
pub use tracing::{SpanContext, TraceCollector, TraceEvent, TraceSpan};
