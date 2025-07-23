//! ABOUTME: Lua-specific API injection modules for Tool and Workflow access  
//! ABOUTME: Handles type conversions between Rust and Lua for LLMSpell components

mod json;
mod streaming;
mod workflow;

pub use json::inject_json_api;
pub use streaming::{create_lua_stream_bridge, inject_streaming_api};
pub use workflow::inject_workflow_api;
