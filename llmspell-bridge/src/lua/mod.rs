//! ABOUTME: Lua script engine implementation of `ScriptEngineBridge`
//! ABOUTME: Provides Lua 5.4 scripting with coroutine-based streaming

pub mod conversion;
pub mod engine;
pub mod globals;
pub mod hook_adapter;
pub mod object_dump;
pub mod output_capture;
pub mod stacktrace;
pub mod sync_utils;

pub use engine::LuaEngine;
