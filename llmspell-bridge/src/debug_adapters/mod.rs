//! Protocol adapters for debug infrastructure
//!
//! This module provides adapters that wrap existing debug components
//! to implement the `DebugCapability` trait for protocol-based access.

pub mod execution_manager_adapter;
pub mod session_manager_adapter;
pub mod stack_navigator_adapter;
pub mod variable_inspector_adapter;

pub use execution_manager_adapter::ExecutionManagerAdapter;
pub use session_manager_adapter::DebugSessionManagerAdapter;
pub use stack_navigator_adapter::StackNavigatorAdapter;
pub use variable_inspector_adapter::VariableInspectorAdapter;
