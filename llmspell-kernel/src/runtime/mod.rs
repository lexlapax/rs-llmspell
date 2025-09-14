//! Runtime management module
//!
//! Provides global IO runtime and tracing infrastructure for the kernel.

pub mod io_runtime;
pub mod tracing;

pub use io_runtime::{
    block_on_global, create_io_bound_resource, ensure_runtime_initialized, global_io_runtime,
    runtime_metrics, spawn_global, RuntimeMetrics,
};
pub use tracing::{TracingInstrumentation, TracingLevel};
