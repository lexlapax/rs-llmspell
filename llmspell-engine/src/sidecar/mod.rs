//! Service mesh sidecar pattern for protocol complexity isolation
//!
//! Provides a sidecar that intercepts and manages protocol messages,
//! handles service discovery, and integrates circuit breaker patterns.

mod discovery;
mod metrics;
mod sidecar;

pub use discovery::{
    HealthStatus, LocalServiceDiscovery, NullServiceDiscovery, ServiceDiscovery, ServiceInfo,
    ServiceQuery,
};
pub use metrics::{
    DefaultMetricsCollector, MetricsCollector, NullMetricsCollector, SidecarMetrics,
};
pub use sidecar::{ProcessedMessage, RawMessage, Sidecar, SidecarConfig, SidecarError};
