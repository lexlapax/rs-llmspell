//! ABOUTME: Lua Tool API implementation providing Tool.get() and tool methods
//! ABOUTME: Bridges between Lua scripts and Rust Tool implementations

use crate::engine::types::ToolApiDefinition;
use crate::ComponentRegistry;
use llmspell_core::error::LLMSpellError;
use llmspell_core::types::{AgentInput, ExecutionContext};
use mlua::Lua;
use std::sync::Arc;

/// Inject the Tool API into the Lua environment
pub fn inject_tool_api(
    lua: &Lua,
    _api_def: &ToolApiDefinition,
    registry: Arc<ComponentRegistry>,
) -> Result<(), LLMSpellError> {
    // Create the Tool global table
    let tool_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Tool table: {}", e),
        source: None,
    })?;

    // Clone registry for closures
    let registry_for_get = registry.clone();

    // Implement Tool.get() function
    let get_fn = lua
        .create_function(move |lua, tool_name: String| -> mlua::Result<mlua::Table> {
            // Get tool from registry
            let tool_arc = registry_for_get.get_tool(&tool_name);
            if tool_arc.is_none() {
                return Err(mlua::Error::runtime(format!(
                    "Tool '{}' not found",
                    tool_name
                )));
            }
            let tool_arc = tool_arc.unwrap();

            // Create Lua table for tool
            let tool_table = lua.create_table()?;
            tool_table.set("name", tool_name.clone())?;
            tool_table.set("description", format!("Tool: {}", tool_name))?;

            // Add execute method
            let tool_arc_for_execute = tool_arc.clone();
            tool_table.set(
                "execute",
                lua.create_async_function(move |lua, args: mlua::Table| {
                    let tool_instance = tool_arc_for_execute.clone();
                    async move {
                        // Convert table to parameters
                        let mut parameters = serde_json::Map::new();

                        // If args has a "parameters" key, use its contents
                        if let Ok(params_table) = args.get::<_, mlua::Table>("parameters") {
                            for (key, value) in
                                params_table.pairs::<mlua::Value, mlua::Value>().flatten()
                            {
                                if let mlua::Value::String(key_str) = key {
                                    let key_string = key_str.to_str().unwrap_or("").to_string();
                                    let json_value = lua_value_to_json(value)?;
                                    parameters.insert(key_string, json_value);
                                }
                            }
                        } else {
                            // Otherwise, use the whole table as parameters
                            // Convert Lua table to JSON
                            for (key, value) in args.pairs::<mlua::Value, mlua::Value>().flatten() {
                                if let mlua::Value::String(key_str) = key {
                                    let key_string = key_str.to_str().unwrap_or("").to_string();
                                    let json_value = lua_value_to_json(value)?;
                                    parameters.insert(key_string, json_value);
                                }
                            }
                        }

                        // Create AgentInput
                        let mut params_map = std::collections::HashMap::new();
                        params_map.insert(
                            "parameters".to_string(),
                            serde_json::Value::Object(parameters),
                        );

                        let input = AgentInput {
                            text: String::new(),
                            media: vec![],
                            context: None,
                            parameters: params_map,
                            output_modalities: vec![],
                        };

                        // Create ExecutionContext
                        let context = ExecutionContext::default();

                        // Execute tool
                        let result = tool_instance.execute(input, context).await;

                        match result {
                            Ok(output) => {
                                let result_table = lua.create_table()?;
                                result_table.set("success", true)?;
                                result_table.set("output", output.text)?;
                                if !output.metadata.extra.is_empty() {
                                    result_table
                                        .set("metadata", format!("{:?}", output.metadata))?;
                                }
                                if !output.tool_calls.is_empty() {
                                    result_table.set("tool_calls", output.tool_calls.len())?;
                                }
                                Ok(result_table)
                            }
                            Err(e) => {
                                let result_table = lua.create_table()?;
                                result_table.set("success", false)?;
                                result_table.set("error", e.to_string())?;
                                Ok(result_table)
                            }
                        }
                    }
                })?,
            )?;

            // Add getSchema method (stub for now)
            tool_table.set(
                "getSchema",
                lua.create_function(move |lua, _: ()| -> mlua::Result<mlua::Table> {
                    let schema_table = lua.create_table()?;
                    schema_table.set("name", "unknown")?;
                    schema_table.set("description", "no description")?;
                    let params_table = lua.create_table()?;
                    schema_table.set("parameters", params_table)?;
                    Ok(schema_table)
                })?,
            )?;

            Ok(tool_table)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Tool.get function: {}", e),
            source: None,
        })?;

    tool_table
        .set("get", get_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.get: {}", e),
            source: None,
        })?;

    // Implement Tool.list() function
    let registry_for_list = registry.clone();
    let list_fn = lua
        .create_function(move |lua, _: ()| -> mlua::Result<mlua::Table> {
            let tools = registry_for_list.list_tools();
            let list_table = lua.create_table()?;
            for (i, tool_name) in tools.iter().enumerate() {
                list_table.set(i + 1, tool_name.clone())?;
            }
            Ok(list_table)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Tool.list function: {}", e),
            source: None,
        })?;

    tool_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.list: {}", e),
            source: None,
        })?;

    // Add async-aware helper function
    let execute_async_code = r#"
        -- Helper to execute tool functions within a coroutine
        function(tool_name, params)
            local tool = Tool.get(tool_name)
            if not tool then
                return {success = false, error = "Tool not found: " .. tool_name}
            end
            
            -- Create coroutine for async execution
            local co = coroutine.create(function()
                -- Use dot notation to call execute as a function, not a method
                return tool.execute(params or {})
            end)
            
            -- Execute the coroutine
            local success, result = coroutine.resume(co)
            
            -- Handle async operations that yield
            while success and coroutine.status(co) ~= "dead" do
                success, result = coroutine.resume(co, result)
            end
            
            if not success then
                return {success = false, error = tostring(result)}
            end
            
            return result
        end
    "#;

    let execute_async_fn = lua
        .load(execute_async_code)
        .eval::<mlua::Function>()
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create executeAsync helper: {}", e),
            source: None,
        })?;

    tool_table
        .set("executeAsync", execute_async_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.executeAsync: {}", e),
            source: None,
        })?;

    // Add sync wrapper for backward compatibility
    let execute_sync_code = r#"
        -- Synchronous wrapper (just calls executeAsync)
        function(tool_name, params)
            return Tool.executeAsync(tool_name, params)
        end
    "#;

    let execute_sync_fn = lua
        .load(execute_sync_code)
        .eval::<mlua::Function>()
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create executeSync helper: {}", e),
            source: None,
        })?;

    tool_table
        .set("executeSync", execute_sync_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.executeSync: {}", e),
            source: None,
        })?;

    // Set Tool global
    lua.globals()
        .set("Tool", tool_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool global: {}", e),
            source: None,
        })?;

    Ok(())
}

/// Check if a Lua table is an array (has sequential numeric keys starting at 1)
fn is_lua_array(table: &mlua::Table) -> bool {
    if let Ok(len) = table.len() {
        if len == 0 {
            return false;
        }
        // Check if all keys from 1 to len exist
        for i in 1..=len {
            if table.get::<_, mlua::Value>(i).is_err() {
                return false;
            }
        }
        // Check if there are any non-numeric keys
        for (k, _) in table.clone().pairs::<mlua::Value, mlua::Value>().flatten() {
            match k {
                mlua::Value::Integer(i) if i >= 1 && i <= len => continue,
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}

/// Helper function to convert Lua values to JSON values
fn lua_value_to_json(value: mlua::Value) -> mlua::Result<serde_json::Value> {
    match value {
        mlua::Value::Nil => Ok(serde_json::Value::Null),
        mlua::Value::Boolean(b) => Ok(serde_json::Value::Bool(b)),
        mlua::Value::Integer(i) => Ok(serde_json::Value::Number(serde_json::Number::from(i))),
        mlua::Value::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(n) {
                Ok(serde_json::Value::Number(num))
            } else {
                Ok(serde_json::Value::Null)
            }
        }
        mlua::Value::String(s) => Ok(serde_json::Value::String(s.to_str()?.to_string())),
        mlua::Value::Table(t) => {
            // Check if it's an array
            if is_lua_array(&t) {
                let mut array = Vec::new();
                for i in 1..=t.len()? {
                    let value = t.get::<_, mlua::Value>(i)?;
                    array.push(lua_value_to_json(value)?);
                }
                Ok(serde_json::Value::Array(array))
            } else {
                // It's an object
                let mut map = serde_json::Map::new();
                for (key, value) in t.pairs::<mlua::Value, mlua::Value>().flatten() {
                    let key_str = match key {
                        mlua::Value::String(s) => s.to_str()?.to_string(),
                        mlua::Value::Integer(i) => i.to_string(),
                        mlua::Value::Number(n) => n.to_string(),
                        _ => continue,
                    };
                    let json_value = lua_value_to_json(value)?;
                    map.insert(key_str, json_value);
                }
                Ok(serde_json::Value::Object(map))
            }
        }
        _ => Ok(serde_json::Value::Null),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mlua::Lua;

    #[test]
    fn test_is_lua_array() {
        let lua = Lua::new();

        // Test array table
        let array_table = lua.create_table().unwrap();
        array_table.set(1, "first").unwrap();
        array_table.set(2, "second").unwrap();
        array_table.set(3, "third").unwrap();
        assert!(is_lua_array(&array_table));

        // Test object table
        let object_table = lua.create_table().unwrap();
        object_table.set("key1", "value1").unwrap();
        object_table.set("key2", "value2").unwrap();
        assert!(!is_lua_array(&object_table));

        // Test mixed table (should be object)
        let mixed_table = lua.create_table().unwrap();
        mixed_table.set(1, "first").unwrap();
        mixed_table.set("key", "value").unwrap();
        assert!(!is_lua_array(&mixed_table));

        // Test non-sequential array (should be object)
        let non_seq_table = lua.create_table().unwrap();
        non_seq_table.set(1, "first").unwrap();
        non_seq_table.set(3, "third").unwrap();
        assert!(!is_lua_array(&non_seq_table));

        // Test empty table
        let empty_table = lua.create_table().unwrap();
        assert!(!is_lua_array(&empty_table));
    }

    #[test]
    fn test_lua_value_to_json() {
        let lua = Lua::new();

        // Test primitives
        assert_eq!(
            lua_value_to_json(mlua::Value::Nil).unwrap(),
            serde_json::Value::Null
        );
        assert_eq!(
            lua_value_to_json(mlua::Value::Boolean(true)).unwrap(),
            serde_json::Value::Bool(true)
        );
        assert_eq!(
            lua_value_to_json(mlua::Value::Integer(42)).unwrap(),
            serde_json::json!(42)
        );
        assert_eq!(
            lua_value_to_json(mlua::Value::Number(3.14)).unwrap(),
            serde_json::json!(3.14)
        );

        // Test string
        let lua_str = lua.create_string("hello").unwrap();
        assert_eq!(
            lua_value_to_json(mlua::Value::String(lua_str)).unwrap(),
            serde_json::json!("hello")
        );

        // Test array
        let array_table = lua.create_table().unwrap();
        array_table.set(1, "a").unwrap();
        array_table.set(2, "b").unwrap();
        array_table.set(3, "c").unwrap();
        let json_array = lua_value_to_json(mlua::Value::Table(array_table)).unwrap();
        assert_eq!(json_array, serde_json::json!(["a", "b", "c"]));

        // Test object
        let object_table = lua.create_table().unwrap();
        object_table.set("name", "Alice").unwrap();
        object_table.set("age", 30).unwrap();
        let json_object = lua_value_to_json(mlua::Value::Table(object_table)).unwrap();
        assert!(json_object.is_object());
        assert_eq!(json_object["name"], "Alice");
        assert_eq!(json_object["age"], 30);

        // Test nested array in object
        let nested_table = lua.create_table().unwrap();
        let inner_array = lua.create_table().unwrap();
        inner_array.set(1, "id").unwrap();
        inner_array.set(2, "name").unwrap();
        inner_array.set(3, "age").unwrap();
        nested_table.set("required", inner_array).unwrap();
        let json_nested = lua_value_to_json(mlua::Value::Table(nested_table)).unwrap();
        assert!(json_nested.is_object());
        assert_eq!(
            json_nested["required"],
            serde_json::json!(["id", "name", "age"])
        );
    }
}
