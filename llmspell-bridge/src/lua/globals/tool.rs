//! ABOUTME: Lua-specific Tool global implementation
//! ABOUTME: Provides Lua bindings for Tool functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::json_to_lua_value;
use crate::ComponentRegistry;
use mlua::{Lua, Table};
use std::sync::Arc;

/// Inject Tool global into Lua environment
pub fn inject_tool_global(
    lua: &Lua,
    _context: &GlobalContext,
    registry: Arc<ComponentRegistry>,
) -> mlua::Result<()> {
    let tool_table = lua.create_table()?;

    // Create Tool.list() function
    let registry_clone = registry.clone();
    let list_fn = lua.create_function(move |lua, ()| {
        let tools = registry_clone.list_tools();
        let list_table = lua.create_table()?;
        for (i, name) in tools.into_iter().enumerate() {
            if let Some(tool) = registry_clone.get_tool(&name) {
                let metadata = tool.metadata();
                let tool_table = lua.create_table()?;
                tool_table.set("name", name)?;
                tool_table.set("description", metadata.description.clone())?;
                tool_table.set("version", metadata.version.to_string())?;
                list_table.set(i + 1, tool_table)?;
            }
        }
        Ok(list_table)
    })?;

    // Create Tool.get() function
    let registry_clone = registry.clone();
    let get_fn = lua.create_function(move |lua, name: String| {
        if let Some(tool) = registry_clone.get_tool(&name) {
            let metadata = tool.metadata();
            let tool_table = lua.create_table()?;
            tool_table.set("name", name)?;
            tool_table.set("description", metadata.description.clone())?;
            tool_table.set("version", metadata.version.to_string())?;

            // Add schema
            let schema = tool.schema();
            let schema_table = lua.create_table()?;
            schema_table.set("name", schema.name)?;
            schema_table.set("description", schema.description)?;

            // Convert parameters to Lua table
            let params_table = lua.create_table()?;
            for (i, param) in schema.parameters.into_iter().enumerate() {
                let param_table = lua.create_table()?;
                param_table.set("name", param.name)?;
                param_table.set("type", format!("{:?}", param.param_type).to_lowercase())?;
                param_table.set("description", param.description)?;
                param_table.set("required", param.required)?;
                if let Some(default) = param.default {
                    param_table.set("default", json_to_lua_value(lua, &default)?)?;
                }
                params_table.set(i + 1, param_table)?;
            }
            schema_table.set("parameters", params_table)?;
            tool_table.set("schema", schema_table)?;

            Ok(Some(tool_table))
        } else {
            Ok(None)
        }
    })?;

    // Create Tool.invoke() function
    let registry_clone = registry.clone();
    let invoke_fn = lua.create_async_function(move |lua, (name, input): (String, Table)| {
        let registry = registry_clone.clone();

        async move {
            // Get the tool
            let tool = registry
                .get_tool(&name)
                .ok_or_else(|| mlua::Error::RuntimeError(format!("Tool '{}' not found", name)))?;

            // Convert input to agent input
            let agent_input = crate::lua::conversion::lua_table_to_agent_input(lua, input)?;

            // Execute the tool
            let context = llmspell_core::ExecutionContext::default();
            let output = tool
                .execute(agent_input, context)
                .await
                .map_err(|e| mlua::Error::RuntimeError(format!("Tool execution failed: {}", e)))?;

            // Convert output to Lua table
            crate::lua::conversion::agent_output_to_lua_table(lua, output)
        }
    })?;

    // Create Tool.exists() function
    let registry_clone = registry.clone();
    let exists_fn =
        lua.create_function(move |_, name: String| Ok(registry_clone.get_tool(&name).is_some()))?;

    // Create Tool.categories() function
    let registry_clone = registry.clone();
    let categories_fn = lua.create_function(move |lua, ()| {
        let mut categories = std::collections::HashSet::new();
        let tools = registry_clone.list_tools();
        for name in tools {
            if let Some(tool) = registry_clone.get_tool(&name) {
                let category = tool.category();
                categories.insert(category.to_string());
            }
        }

        let categories_table = lua.create_table()?;
        for (i, category) in categories.into_iter().enumerate() {
            categories_table.set(i + 1, category)?;
        }
        Ok(categories_table)
    })?;

    // Set functions on Tool table
    tool_table.set("list", list_fn)?;
    tool_table.set("get", get_fn)?;
    tool_table.set("invoke", invoke_fn)?;
    tool_table.set("exists", exists_fn)?;
    tool_table.set("categories", categories_fn)?;

    // Set Tool as global
    lua.globals().set("Tool", tool_table)?;

    Ok(())
}
