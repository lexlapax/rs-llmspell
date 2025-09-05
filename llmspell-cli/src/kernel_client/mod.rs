//! Kernel client module for connecting to kernel servers
//!
//! Provides kernel connection, discovery, and debug event handling for CLI commands
//! that need to communicate with a kernel server (either local or remote).

pub mod connection;
pub mod debug_handler;
pub mod in_process;

// Re-export commonly used types
pub use connection::{
    CliCircuitBreaker, CliCircuitBreakerTrait, CliKernelDiscovery, CliKernelDiscoveryTrait,
    ExecuteResult, KernelClient, KernelConnectionBuilder, KernelConnectionTrait,
    MonitoredKernelConnection,
};
pub use debug_handler::{
    DebugEventHandler, DebugEventHandlerBuilder, DebugEventHandlerTrait, NullDebugEventHandler,
};
pub use in_process::InProcessKernel;
