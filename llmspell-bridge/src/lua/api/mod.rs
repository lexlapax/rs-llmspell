//! ABOUTME: Lua-specific API injection modules for Tool and Workflow access  
//! ABOUTME: Handles type conversions between Rust and Lua for LLMSpell components

mod streaming;

pub use streaming::{create_lua_stream_bridge, inject_streaming_api};
