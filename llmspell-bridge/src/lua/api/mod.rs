//! ABOUTME: Lua-specific API injection modules for Agent, Tool, and Workflow access
//! ABOUTME: Handles type conversions between Rust and Lua for LLMSpell components

mod agent;
mod json;
mod streaming;
mod tool;
mod workflow;

pub use agent::inject_agent_api;
pub use json::inject_json_api;
pub use streaming::{create_lua_stream_bridge, inject_streaming_api};
pub use tool::inject_tool_api;
pub use workflow::inject_workflow_api;
