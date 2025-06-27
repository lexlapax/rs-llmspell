//! ABOUTME: Lua-specific API injection modules for Agent, Tool, and Workflow access
//! ABOUTME: Handles type conversions between Rust and Lua for LLMSpell components

mod agent;
mod tool;
mod workflow;
mod streaming;

pub use agent::inject_agent_api;
pub use tool::inject_tool_api;
pub use workflow::inject_workflow_api;
pub use streaming::{inject_streaming_api, create_lua_stream_bridge};