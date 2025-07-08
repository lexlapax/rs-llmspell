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
    api_def: &ToolApiDefinition,
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
                        // Convert Lua table to AgentInput
                        let mut parameters = serde_json::Map::new();

                        // Convert Lua table to JSON
                        for (key, value) in args.pairs::<mlua::Value, mlua::Value>().flatten() {
                            if let mlua::Value::String(key_str) = key {
                                let key_string = key_str.to_str().unwrap_or("").to_string();
                                let json_value = lua_value_to_json(value)?;
                                parameters.insert(key_string, json_value);
                            }
                        }

                        let agent_input = AgentInput::text("Tool execution from Lua".to_string())
                            .with_parameter(
                                "parameters".to_string(),
                                serde_json::Value::Object(parameters),
                            );

                        // Create execution context
                        let context = ExecutionContext::with_conversation("lua-bridge".to_string());

                        // Execute the tool
                        let execution_result = tool_instance.execute(agent_input, context).await;

                        // Convert result to Lua table
                        let result_table = lua.create_table()?;
                        match execution_result {
                            Ok(agent_output) => {
                                result_table.set("success", true)?;
                                result_table.set("output", agent_output.text)?;
                            }
                            Err(e) => {
                                result_table.set("success", false)?;
                                result_table.set("error", e.to_string())?;
                            }
                        }

                        Ok(result_table)
                    }
                })?,
            )?;

            // Add getSchema method
            tool_table.set(
                "getSchema",
                lua.create_function(|lua, _: ()| {
                    let schema = lua.create_table()?;
                    schema.set("parameters", lua.create_table()?)?;
                    Ok(schema)
                })?,
            )?;

            Ok(tool_table)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Tool.get function: {}", e),
            source: None,
        })?;

    // Implement Tool.list() function
    let registry_for_list = registry.clone();
    let list_fn = lua
        .create_function(move |lua, _: ()| -> mlua::Result<mlua::Table> {
            // Get actual tools from registry
            let tool_names = registry_for_list.list_tools();
            let tools = lua.create_table()?;

            for (i, tool_name) in tool_names.iter().enumerate() {
                tools.set(i + 1, tool_name.clone())?;
            }

            Ok(tools)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Tool.list function: {}", e),
            source: None,
        })?;

    // Add functions to Tool table
    tool_table
        .set(&api_def.get_function[..], get_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.get: {}", e),
            source: None,
        })?;

    tool_table
        .set(&api_def.list_function[..], list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool.list: {}", e),
            source: None,
        })?;

    // Set the Tool table as a global
    lua.globals()
        .set(&api_def.global_name[..], tool_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Tool global: {}", e),
            source: None,
        })?;

    Ok(())
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
        _ => Ok(serde_json::Value::Null),
    }
}
