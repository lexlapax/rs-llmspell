//! Kernel management module for CLI client
//!
//! Provides kernel connection, discovery, and debug event handling functionality.

pub mod connection;
pub mod debug_handler;

// Re-export commonly used types
pub use connection::{
    DebugExecutionHandle, KernelConnection, KernelConnectionBuilder, KernelConnectionTrait,
    KernelDiscoveryTrait, NullKernelConnection, NullKernelDiscovery, RealKernelDiscovery,
};
pub use debug_handler::{
    DebugEventHandler, DebugEventHandlerBuilder, DebugEventHandlerTrait, NullDebugEventHandler,
};
