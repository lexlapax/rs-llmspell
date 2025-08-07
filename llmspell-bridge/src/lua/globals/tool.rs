//! ABOUTME: Lua-specific Tool global implementation
//! ABOUTME: Provides Lua bindings for Tool functionality

use crate::globals::GlobalContext;
use crate::lua::conversion::json_to_lua_value;
use crate::lua::sync_utils::block_on_async_lua;
use crate::ComponentRegistry;
use mlua::{Lua, Table, Value};
use std::sync::Arc;

/// Inject Tool global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
#[allow(clippy::too_many_lines)]
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
            tool_table.set("name", name.clone())?;
            tool_table.set("description", metadata.description.clone())?;
            tool_table.set("version", metadata.version.to_string())?;

            // Add schema
            let schema = tool.schema();
            let schema_table = lua.create_table()?;
            schema_table.set("name", schema.name)?;
            schema_table.set("description", schema.description)?;

            // Convert parameters to Lua table
            let parameters_table = lua.create_table()?;
            for (i, param) in schema.parameters.into_iter().enumerate() {
                let param_entry = lua.create_table()?;
                param_entry.set("name", param.name)?;
                param_entry.set("type", format!("{:?}", param.param_type).to_lowercase())?;
                param_entry.set("description", param.description)?;
                param_entry.set("required", param.required)?;
                if let Some(default) = param.default {
                    param_entry.set("default", json_to_lua_value(lua, &default)?)?;
                }
                parameters_table.set(i + 1, param_entry)?;
            }
            schema_table.set("parameters", parameters_table)?;
            tool_table.set("schema", schema_table)?;

            // Add execute method to the returned tool
            let tool_arc = tool.clone();
            let tool_name = name.clone();
            tool_table.set(
                "execute",
                lua.create_function(move |lua, (_self, params): (Table, Table)| {
                    let tool_instance = tool_arc.clone();
                    let name = tool_name.clone();

                    // Use shared sync utility to execute async code
                    let result = block_on_async_lua(
                        "tool_execute",
                        async move {
                            // Create AgentInput with parameters wrapped correctly for extract_parameters
                            let params_table = lua.create_table()?;
                            params_table.set("text", "Tool invocation")?; // Required by AgentInput

                            // Create nested parameters structure that extract_parameters expects
                            let nested_params = lua.create_table()?;
                            nested_params.set("parameters", params)?;
                            params_table.set("parameters", nested_params)?;

                            // Convert params to agent input
                            let agent_input = crate::lua::conversion::lua_table_to_agent_input(
                                lua,
                                params_table,
                            )?;

                            // Execute the tool
                            let context = llmspell_core::ExecutionContext::default();
                            let output = tool_instance
                                .execute(agent_input, context)
                                .await
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Tool '{name}' execution failed: {e}"
                                    ))
                                })?;

                            // Convert output to Lua table
                            let table =
                                crate::lua::conversion::agent_output_to_lua_table(lua, output)?;
                            Ok(Value::Table(table))
                        },
                        None,
                    )?;

                    Ok(result)
                })?,
            )?;

            Ok(Some(tool_table))
        } else {
            Ok(None)
        }
    })?;

    // Create Tool.invoke() function - synchronous wrapper
    let registry_clone = registry.clone();
    let invoke_fn = lua.create_function(move |lua, (name, input): (String, Table)| {
        let registry = registry_clone.clone();

        // Use shared sync utility to execute async code
        let result = block_on_async_lua(
            "tool_invoke",
            async move {
                // Get the tool
                let tool = registry
                    .get_tool(&name)
                    .ok_or_else(|| mlua::Error::RuntimeError(format!("Tool '{name}' not found")))?;

                // Create AgentInput with parameters wrapped correctly for extract_parameters
                let params_table = lua.create_table()?;
                params_table.set("text", "Tool invocation")?; // Required by AgentInput

                // Create nested parameters structure that extract_parameters expects
                let nested_params = lua.create_table()?;
                nested_params.set("parameters", input)?;
                params_table.set("parameters", nested_params)?;

                // Convert input to agent input
                let agent_input =
                    crate::lua::conversion::lua_table_to_agent_input(lua, params_table)?;

                // Execute the tool
                let context = llmspell_core::ExecutionContext::default();
                let output = tool.execute(agent_input, context).await.map_err(|e| {
                    mlua::Error::RuntimeError(format!("Tool execution failed: {e}"))
                })?;

                // Convert output to Lua table
                let table = crate::lua::conversion::agent_output_to_lua_table(lua, output)?;
                Ok(Value::Table(table))
            },
            None,
        )?;

        Ok(result)
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

    // Add direct tool access via Tool.tool_name pattern
    let registry_for_index = registry.clone();
    let tool_metatable = lua.create_table()?;
    tool_metatable.set(
        "__index",
        lua.create_function(move |lua, (_table, key): (Table, String)| {
            // Check if it's a built-in method first
            let methods = [
                "list",
                "get",
                "invoke",
                "exists",
                "categories",
                "discover",
                "executeAsync",
            ];
            if methods.contains(&key.as_str()) {
                return Ok(mlua::Value::Nil);
            }

            // Otherwise, try to get the tool
            if let Some(tool) = registry_for_index.get_tool(&key) {
                // Create tool instance table
                let tool_instance = lua.create_table()?;
                tool_instance.set("name", key.clone())?;
                let metadata = tool.metadata();
                tool_instance.set("description", metadata.description.clone())?;

                // Add execute method
                let tool_arc = tool.clone();
                let tool_name = key.clone();
                tool_instance.set(
                    "execute",
                    lua.create_async_function(move |lua, (_self, params): (Table, Table)| {
                        let tool_instance = tool_arc.clone();
                        let name = tool_name.clone();
                        async move {
                            // Create AgentInput with parameters wrapped correctly for extract_parameters
                            let params_table = lua.create_table()?;
                            params_table.set("text", "Tool invocation")?; // Required by AgentInput

                            // Create nested parameters structure that extract_parameters expects
                            let nested_params = lua.create_table()?;
                            nested_params.set("parameters", params)?;
                            params_table.set("parameters", nested_params)?;

                            // Convert params to agent input
                            let agent_input = crate::lua::conversion::lua_table_to_agent_input(
                                lua,
                                params_table,
                            )?;

                            // Execute the tool
                            let context = llmspell_core::ExecutionContext::default();
                            let output = tool_instance
                                .execute(agent_input, context)
                                .await
                                .map_err(|e| {
                                    mlua::Error::RuntimeError(format!(
                                        "Tool '{name}' execution failed: {e}"
                                    ))
                                })?;

                            // Convert output to Lua table
                            crate::lua::conversion::agent_output_to_lua_table(lua, output)
                        }
                    })?,
                )?;

                // Add getSchema method
                let schema = tool.schema();
                tool_instance.set(
                    "getSchema",
                    lua.create_function(move |lua, (): ()| {
                        let schema_table = lua.create_table()?;
                        schema_table.set("name", schema.name.clone())?;
                        schema_table.set("description", schema.description.clone())?;

                        let parameters_table = lua.create_table()?;
                        for (i, param) in schema.parameters.iter().enumerate() {
                            let param_entry = lua.create_table()?;
                            param_entry.set("name", param.name.clone())?;
                            param_entry
                                .set("type", format!("{:?}", param.param_type).to_lowercase())?;
                            param_entry.set("description", param.description.clone())?;
                            param_entry.set("required", param.required)?;
                            if let Some(default) = &param.default {
                                param_entry.set("default", json_to_lua_value(lua, default)?)?;
                            }
                            parameters_table.set(i + 1, param_entry)?;
                        }
                        schema_table.set("parameters", parameters_table)?;
                        Ok(schema_table)
                    })?,
                )?;

                Ok(mlua::Value::Table(tool_instance))
            } else {
                Ok(mlua::Value::Nil)
            }
        })?,
    )?;
    tool_table.set_metatable(Some(tool_metatable));

    // Add discover function for tool discovery
    let registry_clone = registry;
    let discover_fn = lua.create_function(move |lua, filter: Option<Table>| {
        let tools = registry_clone.list_tools();
        let discover_table = lua.create_table()?;

        // Extract filter criteria
        let category_filter = filter
            .as_ref()
            .and_then(|f| f.get::<_, Option<String>>("category").ok())
            .flatten();
        let tag_filter = filter
            .as_ref()
            .and_then(|f| f.get::<_, Option<String>>("tag").ok())
            .flatten();

        let mut index = 1;
        for name in tools {
            if let Some(tool) = registry_clone.get_tool(&name) {
                let metadata = tool.metadata();
                let category = tool.category();

                // Apply filters
                if let Some(cat) = &category_filter {
                    if category.to_string() != *cat {
                        continue;
                    }
                }

                if let Some(tag) = &tag_filter {
                    // Tag filtering would require metadata to have tags
                    // For now, skip if tag filter is provided
                    if !metadata
                        .description
                        .to_lowercase()
                        .contains(&tag.to_lowercase())
                    {
                        continue;
                    }
                }

                let tool_info = lua.create_table()?;
                tool_info.set("name", name)?;
                tool_info.set("description", metadata.description.clone())?;
                tool_info.set("category", category.to_string())?;
                tool_info.set("version", metadata.version.to_string())?;
                discover_table.set(index, tool_info)?;
                index += 1;
            }
        }
        Ok(discover_table)
    })?;

    tool_table.set("discover", discover_fn)?;

    // Tool.executeAsync removed - all tools now use synchronous API

    // Set Tool as global
    lua.globals().set("Tool", tool_table)?;

    Ok(())
}
