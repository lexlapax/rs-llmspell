//! ABOUTME: Lua script engine implementation of `ScriptEngineBridge`
//! ABOUTME: Provides Lua 5.4 scripting with coroutine-based streaming

pub mod conversion;
pub mod engine;
pub mod globals;
pub mod hook_adapter;
pub mod output;
pub mod sync_utils;

pub use engine::LuaEngine;
