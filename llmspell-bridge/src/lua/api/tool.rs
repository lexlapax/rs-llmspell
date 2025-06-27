//! ABOUTME: Lua Tool API implementation providing Tool.get() and tool methods
//! ABOUTME: Bridges between Lua scripts and Rust Tool implementations

use crate::engine::types::ToolApiDefinition;
use crate::ComponentRegistry;
use llmspell_core::error::LLMSpellError;
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
    let _registry_clone = registry.clone();

    // Implement Tool.get() function
    let get_fn = lua
        .create_function(move |lua, tool_name: String| -> mlua::Result<mlua::Table> {
            // TODO: Actually get tool from registry
            // For now, return a mock tool
            let tool = lua.create_table()?;
            tool.set("name", tool_name.clone())?;
            tool.set("description", format!("Tool: {}", tool_name))?;

            // Add execute method
            let tool_name_for_execute = tool_name.clone();
            tool.set(
                "execute",
                lua.create_async_function(move |lua, _args: mlua::Table| {
                    let tool_name_in_async = tool_name_for_execute.clone();
                    async move {
                        // Mock execution
                        let result = lua.create_table()?;
                        result.set("success", true)?;
                        result.set("output", format!("Executed tool: {}", tool_name_in_async))?;
                        Ok(result)
                    }
                })?,
            )?;

            // Add getSchema method
            tool.set(
                "getSchema",
                lua.create_function(|lua, _: ()| {
                    let schema = lua.create_table()?;
                    schema.set("parameters", lua.create_table()?)?;
                    Ok(schema)
                })?,
            )?;

            Ok(tool)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Tool.get function: {}", e),
            source: None,
        })?;

    // Implement Tool.list() function
    let list_fn = lua
        .create_function(move |lua, _: ()| -> mlua::Result<mlua::Table> {
            // TODO: Get actual tools from registry
            let tools = lua.create_table()?;
            tools.set(1, "calculator")?;
            tools.set(2, "web_search")?;
            tools.set(3, "file_reader")?;
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
