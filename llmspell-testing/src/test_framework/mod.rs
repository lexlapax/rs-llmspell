//! Unified Test Execution Framework
//!
//! Provides a single execution engine for tests, benchmarks, and stress tests
//! with adaptive workloads and built-in telemetry.

pub mod adapters;
pub mod collectors;
pub mod executor;
pub mod telemetry;
pub mod workload;

// Re-export main types
pub use executor::{ExecutionContext, ExecutionMode, TestExecutor, TestResult};
pub use telemetry::{Metrics, TelemetryCollector};
pub use workload::{WorkloadAdapter, WorkloadClass};
