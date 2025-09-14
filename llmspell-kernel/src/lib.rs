//! # `LLMSpell` Kernel
//!
//! Integrated kernel architecture with REPL and debugging infrastructure.
//!
//! This crate provides the core kernel functionality for `LLMSpell`, including:
//! - Global IO runtime management to prevent "dispatch task is gone" errors
//! - Multi-protocol transport layer (Jupyter, LSP, DAP)
//! - Script execution engine with debugging support
//! - Session and state management
//! - Event correlation and distributed tracing
//!
//! ## Architecture
//!
//! The kernel uses a unified runtime context to ensure all I/O operations
//! share the same Tokio runtime, preventing runtime context mismatches that
//! cause HTTP client failures in long-running operations.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod debug;
pub mod execution;
pub mod io;
pub mod runtime;
pub mod sessions;
pub mod state;
pub mod traits;
pub mod transport;

// Re-export commonly used runtime types
pub use runtime::io_runtime::{
    block_on_global, create_io_bound_resource, ensure_runtime_initialized, global_io_runtime,
    runtime_metrics, spawn_global, RuntimeMetrics,
};
pub use runtime::tracing::{
    OperationCategory, SessionType, TracingInstrumentation, TracingLevel, TracingMetadata,
};

// Re-export I/O types
pub use io::{
    manager::{EnhancedIOManager, IOConfig, IOPubMessage, MessageHeader, StreamType},
    router::{ClientConnection, MessageDestination, MessageRouter},
};

// Re-export transport types
pub use traits::{ChannelConfig, Protocol, Transport, TransportConfig};

#[cfg(feature = "zeromq")]
pub use transport::zeromq::ZmqTransport;

pub use transport::jupyter::{JupyterConnectionInfo, JupyterTransport};

// Re-export execution types
pub use execution::{ExecutionConfig, IntegratedKernel};

// Re-export state types
pub use state::{
    circuit_breaker::{CircuitBreaker, CircuitBreakerStats, CircuitState},
    persistence::StatePersistence,
    DebugState, ExecutionState, KernelState, MemoryBackend, SessionState, SledBackend,
    StorageBackend, VectorBackend,
};

// Re-export session types
pub use sessions::{
    AccessControl, ArtifactStorage, ArtifactType, RateLimitConfig, RateLimitPolicy, Session,
    SessionArtifact, SessionConfig, SessionEvent, SessionEventType, SessionId, SessionManager,
    SessionMetadata, SessionMetrics, SessionPolicy, SessionSecurity, SessionStatus, TimeoutPolicy,
};

// Re-export debug types
pub use debug::{
    Breakpoint, DAPBridge, DapBreakpoint, DapCapabilities, DapScope, DapSource, DapStackFrame,
    DapVariable, DebugAdapter, DebugCoordinator, DebugEvent, DebugResponse, DebugSession,
    DebugSessionManager, ExecutionManager, LuaDebugAdapter, MemoryAwareDebugCoordinator,
    SourceBreakpoint, SourceReference, StackFrame, StepMode, Variable, VariableScope,
};

/// Kernel version information
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Kernel protocol version (Jupyter protocol 5.3)
pub const PROTOCOL_VERSION: &str = "5.3";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        // KERNEL_VERSION is from env! macro so it's always non-empty
        assert_eq!(PROTOCOL_VERSION, "5.3");
    }
}
