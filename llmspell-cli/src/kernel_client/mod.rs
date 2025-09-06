//! Kernel client module for connecting to kernel servers
//!
//! Provides kernel connection, discovery, and debug event handling for CLI commands
//! that need to communicate with a kernel server (either local or remote).

pub mod connection;
pub mod debug_handler;
pub mod embedded_kernel;

// Re-export commonly used types
pub use connection::{
    CliCircuitBreaker, CliCircuitBreakerTrait, ExecuteResult, KernelClient,
    KernelConnectionBuilder, KernelConnectionTrait, MonitoredKernelConnection,
};
pub use embedded_kernel::EmbeddedKernel;
