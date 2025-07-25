//! ABOUTME: Tool lifecycle management with hook integration
//! ABOUTME: Provides enhanced tool execution with hooks, state management, and performance monitoring

pub mod hook_integration;
pub mod state_machine;

pub use hook_integration::{ExecutionMetrics, ToolExecutor, ToolHookContext, ToolLifecycleConfig};
pub use state_machine::{ToolExecutionState, ToolStateMachine};

/// Tool lifecycle exports for public API
pub use hook_integration::HookableToolExecution;
