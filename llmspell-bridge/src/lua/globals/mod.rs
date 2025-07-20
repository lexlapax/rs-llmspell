//! ABOUTME: Lua-specific global object implementations
//! ABOUTME: Contains Lua bindings for global objects

pub mod agent;
pub mod tool;
pub mod workflow;

pub use agent::inject_agent_global;
pub use tool::inject_tool_global;
pub use workflow::inject_workflow_global;
