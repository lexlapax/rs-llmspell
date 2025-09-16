//! ABOUTME: Lua-specific ARGS global implementation for passing command-line arguments
//! ABOUTME: Provides `ARGS` table with both named and positional argument access

use llmspell_core::error::LLMSpellError;
use mlua::Lua;
use std::collections::HashMap;
use std::hash::BuildHasher;
use tracing::{instrument, trace};

/// Inject ARGS global into Lua environment
///
/// Creates a global `ARGS` table containing script arguments.
/// Arguments can be accessed as:
/// - Named: `ARGS.input` or `ARGS["input"]`
/// - Positional: `ARGS[1]`, `ARGS[2]`, etc.
/// - Script name: `ARGS[0]` (for Lua compatibility)
///
/// # Errors
///
/// Returns an error if ARGS global injection or table creation fails
#[instrument(level = "trace", skip(lua, args), fields(
    global_name = "ARGS",
    arg_count = args.len()
))]
pub fn inject_args_global<S: BuildHasher>(
    lua: &Lua,
    args: &HashMap<String, String, S>,
) -> Result<(), LLMSpellError> {
    trace!(arg_count = args.len(), "Injecting ARGS global");
    let args_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create ARGS table: {e}"),
        source: None,
    })?;

    // Add all arguments to the table
    for (key, value) in args {
        // Try to parse the key as a number for positional arguments
        if let Ok(index) = key.parse::<i32>() {
            // It's a positional argument, use numeric index
            args_table
                .set(index, value.clone())
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set positional argument {index}: {e}"),
                    source: None,
                })?;
        } else {
            // It's a named argument, use string key
            args_table
                .set(key.as_str(), value.clone())
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set named argument '{key}': {e}"),
                    source: None,
                })?;
        }
    }

    // Set the ARGS global
    lua.globals()
        .set("ARGS", args_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set ARGS global: {e}"),
            source: None,
        })?;

    // Also create the traditional 'arg' table for compatibility
    let traditional_arg_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create arg table: {e}"),
        source: None,
    })?;

    // Copy positional arguments to arg table
    for (key, value) in args {
        if let Ok(index) = key.parse::<i32>() {
            traditional_arg_table
                .set(index, value.clone())
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to set arg[{index}]: {e}"),
                    source: None,
                })?;
        }
    }

    // Set the arg global (lowercase for traditional Lua compatibility)
    lua.globals()
        .set("arg", traditional_arg_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set arg global: {e}"),
            source: None,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_injection() {
        let lua = mlua::Lua::new();
        let mut args = HashMap::new();

        // Add some test arguments
        args.insert("0".to_string(), "script.lua".to_string());
        args.insert("1".to_string(), "positional1".to_string());
        args.insert("2".to_string(), "positional2".to_string());
        args.insert("input".to_string(), "input-file.lua".to_string());
        args.insert("debug".to_string(), "true".to_string());
        args.insert("max-cost".to_string(), "20".to_string());

        // Inject the arguments
        inject_args_global(&lua, &args).unwrap();

        // Test that we can access them from Lua
        lua.load(
            r#"
            -- Test positional access
            assert(ARGS[0] == "script.lua", "ARGS[0] should be script name")
            assert(ARGS[1] == "positional1", "ARGS[1] should be first positional arg")
            assert(ARGS[2] == "positional2", "ARGS[2] should be second positional arg")
            
            -- Test named access
            assert(ARGS.input == "input-file.lua", "ARGS.input should work")
            assert(ARGS["input"] == "input-file.lua", "ARGS['input'] should work")
            assert(ARGS.debug == "true", "ARGS.debug should work")
            assert(ARGS["max-cost"] == "20", "ARGS['max-cost'] should work for hyphenated keys")
            
            -- Test traditional arg table
            assert(arg[0] == "script.lua", "arg[0] should be script name")
            assert(arg[1] == "positional1", "arg[1] should be first positional arg")
            assert(arg[2] == "positional2", "arg[2] should be second positional arg")
            "#,
        )
        .exec()
        .unwrap();
    }

    #[test]
    fn test_empty_args() {
        let lua = mlua::Lua::new();
        let args = HashMap::new();

        // Should work with empty args
        inject_args_global(&lua, &args).unwrap();

        // ARGS and arg should exist but be empty
        lua.load(
            r#"
            assert(type(ARGS) == "table", "ARGS should be a table")
            assert(type(arg) == "table", "arg should be a table")
            
            -- Should be able to safely check for missing args
            local input = ARGS.input or "default.lua"
            assert(input == "default.lua", "Should use default when arg missing")
            "#,
        )
        .exec()
        .unwrap();
    }
}
