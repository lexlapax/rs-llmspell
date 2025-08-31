//! Unified Test Execution Framework
//! 
//! Provides a single execution engine for tests, benchmarks, and stress tests
//! with adaptive workloads and built-in telemetry.

pub mod executor;
pub mod workload;
pub mod telemetry;
pub mod adapters;
pub mod collectors;

// Re-export main types
pub use executor::{TestExecutor, TestResult, ExecutionContext, ExecutionMode};
pub use workload::{WorkloadClass, WorkloadAdapter};
pub use telemetry::{TelemetryCollector, Metrics};