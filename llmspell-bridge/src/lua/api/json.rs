//! ABOUTME: JSON API implementation for Lua scripting engine
//! ABOUTME: Provides JSON.parse() and JSON.stringify() functions for Lua scripts

use crate::engine::types::JsonApiDefinition;
use llmspell_core::error::LLMSpellError;
use serde_json;

/// Inject JSON API into Lua engine
pub fn inject_json_api(lua: &mlua::Lua, api_def: &JsonApiDefinition) -> Result<(), LLMSpellError> {
    let json_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create JSON table: {}", e),
        source: None,
    })?;

    // JSON.parse(string) -> table/value
    let parse_fn = lua
        .create_function(|lua, json_str: String| {
            let json_value = serde_json::from_str::<serde_json::Value>(&json_str)
                .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;
            json_value_to_lua(lua, json_value)
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
        .set(api_def.parse_function.as_str(), parse_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set parse function: {}", e),
            source: None,
        })?;

    json_table
        .set(api_def.stringify_function.as_str(), stringify_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set stringify function: {}", e),
            source: None,
        })?;

    lua.globals()
        .set(api_def.global_name.as_str(), json_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set JSON global: {}", e),
            source: None,
        })?;

    Ok(())
}

/// Convert serde_json::Value to mlua::Value
pub(crate) fn json_value_to_lua(
    lua: &mlua::Lua,
    value: serde_json::Value,
) -> mlua::Result<mlua::Value> {
    match value {
        serde_json::Value::Null => Ok(mlua::Value::Nil),
        serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(mlua::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(mlua::Value::Number(f))
            } else {
                Ok(mlua::Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, val) in arr.into_iter().enumerate() {
                table.set(i + 1, json_value_to_lua(lua, val)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.set(k, json_value_to_lua(lua, v)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
    }
}

/// Convert mlua::Value to serde_json::Value
pub(crate) fn lua_value_to_json(value: mlua::Value) -> mlua::Result<serde_json::Value> {
    match value {
        mlua::Value::Nil => Ok(serde_json::Value::Null),
        mlua::Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        mlua::Value::Integer(i) => Ok(serde_json::Value::Number(i.into())),
        mlua::Value::Number(f) => serde_json::Number::from_f64(f)
            .map(serde_json::Value::Number)
            .ok_or_else(|| mlua::Error::RuntimeError("Invalid number for JSON".to_string())),
        mlua::Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        mlua::Value::Table(table) => {
            // Check if this is an array or object
            if is_lua_array(&table)? {
                let mut arr = Vec::new();
                for i in 1.. {
                    match table.get::<_, mlua::Value>(i)? {
                        mlua::Value::Nil => break,
                        val => arr.push(lua_value_to_json(val)?),
                    }
                }
                Ok(serde_json::Value::Array(arr))
            } else {
                let mut map = serde_json::Map::new();
                for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
                    let (k, v) = pair?;
                    let key = match k {
                        mlua::Value::String(s) => s.to_str()?.to_string(),
                        mlua::Value::Integer(i) => i.to_string(),
                        mlua::Value::Number(f) => f.to_string(),
                        _ => {
                            return Err(mlua::Error::RuntimeError(
                                "Table key must be string or number for JSON".to_string(),
                            ))
                        }
                    };
                    map.insert(key, lua_value_to_json(v)?);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Err(mlua::Error::RuntimeError(format!(
            "Cannot convert {:?} to JSON",
            value
        ))),
    }
}

/// Check if a Lua table is an array (sequential integer keys starting from 1)
fn is_lua_array(table: &mlua::Table) -> mlua::Result<bool> {
    let len = table.len()?;
    if len == 0 {
        // Empty table, check if it has any non-integer keys
        for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, _) = pair?;
            if !matches!(k, mlua::Value::Integer(_)) {
                return Ok(false);
            }
        }
        Ok(true)
    } else {
        // Check if all keys from 1 to len exist
        for i in 1..=len {
            if table.get::<_, mlua::Value>(i)?.is_nil() {
                return Ok(false);
            }
        }
        // Check if there are any keys beyond len or non-integer keys
        let mut count = 0;
        for pair in table.clone().pairs::<mlua::Value, mlua::Value>() {
            let (k, _) = pair?;
            match k {
                mlua::Value::Integer(i) if i >= 1 && i <= len => count += 1,
                _ => return Ok(false),
            }
        }
        Ok(count == len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_roundtrip() {
        let lua = mlua::Lua::new();
        let api_def = JsonApiDefinition::standard();

        // Inject API
        inject_json_api(&lua, &api_def).unwrap();

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
