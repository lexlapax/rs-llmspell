// ABOUTME: Lua script tests for bridge functionality
// ABOUTME: Tests Lua API, tool integration, and script execution

//! Lua test suite for llmspell framework
//! 
//! This module contains tests for Lua scripting functionality,
//! including API tests, tool integration, and script execution.

use mlua::{Lua, Result as LuaResult};
use std::path::PathBuf;

// Lua API tests
#[cfg(test)]
mod api_tests;

// Tool integration tests
#[cfg(test)]
mod tool_integration;

// Hook tests in Lua
#[cfg(test)]
mod hook_tests;

// Workflow tests in Lua
#[cfg(test)]
mod workflow_tests;

// State persistence tests in Lua
#[cfg(test)]
mod state_tests;

// Error handling in Lua
#[cfg(test)]
mod error_handling;

// Performance tests in Lua
#[cfg(test)]
mod performance_tests;

// Cross-language tests
#[cfg(test)]
mod cross_language;

/// Helper function to create a Lua VM with llmspell globals loaded
#[cfg(test)]
pub fn create_test_lua() -> LuaResult<Lua> {
    let lua = Lua::new();
    // Initialize with llmspell globals
    llmspell_bridge::init_lua_globals(&lua)?;
    Ok(lua)
}

/// Helper function to load a Lua test script
#[cfg(test)]
pub fn load_lua_script(lua: &Lua, script_name: &str) -> LuaResult<()> {
    let script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join("lua")
        .join(script_name);
    
    let script = std::fs::read_to_string(script_path)
        .map_err(|e| mlua::Error::RuntimeError(format!("Failed to read script: {}", e)))?;
    
    lua.load(&script).exec()
}