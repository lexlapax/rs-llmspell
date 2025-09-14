//! Runtime management module
//!
//! Provides global IO runtime and tracing infrastructure for the kernel.

pub mod io_runtime;
pub mod tracing;

pub use io_runtime::{create_io_bound_resource, global_io_runtime, RuntimeMetrics};
pub use tracing::{TracingInstrumentation, TracingLevel};