//! Service mesh sidecar pattern for protocol complexity isolation
//!
//! Provides a sidecar that intercepts and manages protocol messages,
//! handles service discovery, and integrates circuit breaker patterns.

mod core;
mod discovery;
mod metrics;

pub use core::{
    ProcessedMessage, ProcessingMetadata, RawMessage, Sidecar, SidecarConfig, SidecarError,
};
pub use discovery::{
    HealthStatus, LocalServiceDiscovery, NullServiceDiscovery, ServiceDiscovery, ServiceInfo,
    ServiceQuery,
};
pub use metrics::{
    DefaultMetricsCollector, MetricsCollector, NullMetricsCollector, SidecarMetrics,
};
