//! ABOUTME: Lua-specific global object implementations
//! ABOUTME: Contains Lua bindings for global objects

pub mod agent;
pub mod event;
pub mod hook;
pub mod json;
pub mod streaming;
pub mod tool;
pub mod workflow;

pub use agent::inject_agent_global;
pub use event::inject_event_global;
pub use hook::inject_hook_global;
pub use json::inject_json_global;
pub use streaming::inject_streaming_global;
pub use tool::inject_tool_global;
pub use workflow::inject_workflow_global;
