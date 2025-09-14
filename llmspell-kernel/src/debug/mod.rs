//! Debug Infrastructure for Interactive REPL and Debugging
//!
//! This module consolidates the debug infrastructure from Phase-9 branch,
//! including execution bridge, debug coordinator, DAP bridge, and language-specific debug support.

pub mod coordinator;
pub mod dap;
pub mod execution_bridge;
pub mod lua;
pub mod session;

// Re-export main types
pub use coordinator::{DebugCoordinator, DebugEvent, DebugResponse, MemoryAwareDebugCoordinator};
pub use dap::{
    Capabilities as DapCapabilities, DAPBridge, DapBreakpoint, DapStackFrame, DapVariable,
    DebugAdapter, LuaDebugAdapter, Scope as DapScope, Source as DapSource, SourceBreakpoint,
    SourceReference,
};
pub use execution_bridge::{
    Breakpoint, ExecutionManager, StackFrame, StepMode, Variable, VariableScope,
};
pub use session::{DebugSession, DebugSessionManager};
