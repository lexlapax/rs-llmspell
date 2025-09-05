//! Kernel management module for CLI client
//!
//! Provides kernel connection, discovery, and debug event handling functionality.

pub mod connection;
pub mod debug_handler;

// Re-export commonly used types
pub use connection::{
    CliCircuitBreaker, CliCircuitBreakerTrait, CliKernelDiscovery, CliKernelDiscoveryTrait,
    ExecuteResult, KernelClient, KernelConnectionBuilder, KernelConnectionTrait,
    MonitoredKernelConnection,
};
pub use debug_handler::{
    DebugEventHandler, DebugEventHandlerBuilder, DebugEventHandlerTrait, NullDebugEventHandler,
};
