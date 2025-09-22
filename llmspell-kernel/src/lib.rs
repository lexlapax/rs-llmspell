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

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod api;
pub mod connection;
pub mod daemon;
pub mod debug;
pub mod events;
pub mod execution;
pub mod hooks;
pub mod io;
pub mod protocols;
pub mod repl;
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

// Re-export event correlation types
pub use events::correlation::{
    ErrorInfo as KernelErrorInfo, ExecutionState as KernelExecutionState,
    ExecutionStatus as KernelExecutionStatus, LanguageInfo,
};
pub use events::{EventBroadcaster, KernelEvent, KernelEventCorrelator};

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
    events::{SessionEvent, SessionEventType},
    ArtifactId, ArtifactQuery, ArtifactStorage, ArtifactStorageConfig, ArtifactStorageOps,
    ArtifactType, CreateSessionOptions, CreateSessionOptionsBuilder, Result as SessionResult,
    Session, SessionArtifact, SessionConfig, SessionError, SessionId, SessionManager,
    SessionManagerConfig, SessionManagerConfigBuilder, SessionMetadata, SessionStatus,
};

// Re-export debug types
pub use debug::{
    Breakpoint, DAPBridge, DapBreakpoint, DapCapabilities, DapScope, DapSource, DapStackFrame,
    DapVariable, DebugAdapter, DebugCoordinator, DebugEvent, DebugResponse, DebugSession,
    DebugSessionManager, ExecutionManager, LuaDebugAdapter, MemoryAwareDebugCoordinator,
    SourceBreakpoint, SourceReference, StackFrame, StepMode, Variable, VariableScope,
};

// Re-export hook types
pub use hooks::{
    ComponentId,
    ComponentType,
    Condition,
    ConditionBuilder,
    ConditionalHook,
    DebugContext,
    ExecutionContext,
    // Re-export core hook infrastructure
    Hook,
    HookContext,
    HookExecutor,
    HookPerformanceMetrics,
    HookPoint,
    HookRegistry,
    HookResult,
    KernelHook,
    KernelHookManager,
    KernelHookPoint,
    KernelHookSystem,
    KernelPerformanceMonitor,
    PostExecuteHook,
    PreDebugHook,
    PreExecuteHook,
    Priority,
    RetryHook,
    StateChangeHook,
    StateContext,
};

/// Kernel version information
pub const KERNEL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Kernel protocol version (Jupyter protocol 5.3)
pub const PROTOCOL_VERSION: &str = "5.3";

// Re-export high-level API
pub use api::{
    connect_to_kernel, start_embedded_kernel, start_embedded_kernel_with_executor,
    start_kernel_service, ClientHandle, KernelHandle, ServiceHandle,
};

// Re-export REPL types
pub use repl::{DebugCommand, InteractiveSession, MetaCommand, ReplCommand, ReplSessionConfig};

// Re-export daemon types
pub use daemon::{DaemonConfig, DaemonManager, PidFile, SignalBridge, SignalHandler};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        // KERNEL_VERSION is from env! macro so it's always non-empty
        assert_eq!(PROTOCOL_VERSION, "5.3");
    }
}
