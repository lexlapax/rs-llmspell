//! ABOUTME: Lua-specific JSON global implementation
//! ABOUTME: Provides JSON.parse() and JSON.stringify() for Lua scripts

use crate::lua::conversion::{json_to_lua_value, lua_value_to_json};
use llmspell_core::error::LLMSpellError;
use mlua::Lua;
use serde_json;

/// Inject JSON global into Lua environment
pub fn inject_json_global(lua: &Lua) -> Result<(), LLMSpellError> {
    let json_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create JSON table: {}", e),
        source: None,
    })?;

    // JSON.parse(string) -> table/value
    let parse_fn = lua
        .create_function(|lua, json_str: String| {
            let json_value = serde_json::from_str::<serde_json::Value>(&json_str)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;
            json_to_lua_value(lua, &json_value)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create parse function: {}", e),
            source: None,
        })?;

    // JSON.stringify(value) -> string
    let stringify_fn = lua
        .create_function(|_lua, value: mlua::Value| {
            let json_value = lua_value_to_json(value)?;
            serde_json::to_string(&json_value)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON stringify error: {}", e)))
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create stringify function: {}", e),
            source: None,
        })?;

    json_table
        .set("parse", parse_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set parse function: {}", e),
            source: None,
        })?;

    json_table
        .set("stringify", stringify_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set stringify function: {}", e),
            source: None,
        })?;

    lua.globals()
        .set("JSON", json_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set JSON global: {}", e),
            source: None,
        })?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip() {
        let lua = mlua::Lua::new();

        // Inject global
        inject_json_global(&lua).unwrap();

        // Test roundtrip
        lua.load(
            r#"
            local data = {
                string = "hello",
                number = 42,
                float = 3.14,
                bool = true,
                null = nil,
                array = {1, 2, 3},
                object = {
                    nested = "value"
                }
            }
            
            local json_str = JSON.stringify(data)
            local parsed = JSON.parse(json_str)
            
            assert(parsed.string == "hello")
            assert(parsed.number == 42)
            assert(parsed.bool == true)
            assert(parsed.null == nil)
            assert(parsed.array[1] == 1)
            assert(parsed.object.nested == "value")
        "#,
        )
        .exec()
        .unwrap();
    }
}
