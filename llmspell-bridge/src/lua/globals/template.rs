//! ABOUTME: Lua-specific Template global implementation
//! ABOUTME: Provides Lua bindings for Template functionality via `TemplateBridge`

use crate::globals::GlobalContext;
use crate::lua::conversion::{
    config_schema_to_lua_table, lua_table_to_template_params, template_metadata_to_lua_table,
    template_output_to_lua_table,
};
use crate::lua::sync_utils::block_on_async_lua;
use crate::template_bridge::TemplateBridge;
use llmspell_templates::TemplateCategory;
use mlua::{Lua, Table, Value};
use std::sync::Arc;
use tracing::{info, instrument};

/// Inject Template global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
#[allow(clippy::too_many_lines)]
#[instrument(
    level = "info",
    skip(lua, _context, bridge),
    fields(global_name = "Template")
)]
pub fn inject_template_global(
    lua: &Lua,
    _context: &GlobalContext,
    bridge: Arc<TemplateBridge>,
) -> mlua::Result<()> {
    info!("Injecting Template global API");
    let template_table = lua.create_table()?;

    // Create Template.list([category]) function
    let bridge_clone = bridge.clone();
    let list_fn = lua.create_function(move |lua, category_str: Option<String>| {
        let bridge = bridge_clone.clone();

        // Parse category if provided
        let category = category_str.map(|cat_str| parse_template_category(&cat_str));

        // Call bridge.list_templates()
        let templates = bridge.list_templates(category);

        // Convert Vec<TemplateMetadata> to Lua array
        let list_table = lua.create_table()?;
        for (i, metadata) in templates.into_iter().enumerate() {
            let metadata_table = template_metadata_to_lua_table(lua, &metadata)?;
            list_table.set(i + 1, metadata_table)?;
        }
        Ok(list_table)
    })?;

    // Create Template.info(name, [show_schema]) function
    let bridge_clone = bridge.clone();
    let info_fn =
        lua.create_function(move |lua, (name, show_schema): (String, Option<bool>)| {
            let bridge = bridge_clone.clone();
            let include_schema = show_schema.unwrap_or(false);

            // Call bridge.get_template_info()
            let template_info = bridge
                .get_template_info(&name, include_schema)
                .map_err(|e| {
                    mlua::Error::RuntimeError(format!("Failed to get template info: {e}"))
                })?;

            // Convert TemplateInfo to Lua table
            let info_table = lua.create_table()?;

            // Add metadata
            let metadata_table = template_metadata_to_lua_table(lua, &template_info.metadata)?;
            info_table.set("metadata", metadata_table)?;

            // Add schema if requested
            if let Some(schema) = template_info.schema {
                let schema_table = config_schema_to_lua_table(lua, &schema)?;
                info_table.set("schema", schema_table)?;
            }

            Ok(info_table)
        })?;

    // Create Template.execute(name, params) function - ASYNC
    let bridge_clone = bridge.clone();
    let execute_fn = lua.create_function(move |lua, (name, params_table): (String, Table)| {
        let bridge = bridge_clone.clone();

        // Use block_on_async_lua for async execution
        let result = block_on_async_lua(
            "template_execute",
            async move {
                // Convert Lua table to TemplateParams
                let params = lua_table_to_template_params(lua, params_table)?;

                // Call bridge.execute_template() - ALL validation/context building in bridge!
                let output = bridge.execute_template(&name, params).await.map_err(|e| {
                    mlua::Error::RuntimeError(format!("Template '{name}' execution failed: {e}"))
                })?;

                // Convert TemplateOutput to Lua table
                let output_table = template_output_to_lua_table(lua, &output)?;
                Ok(Value::Table(output_table))
            },
            None,
        )?;

        Ok(result)
    })?;

    // Create Template.search(query, [category]) function
    let bridge_clone = bridge.clone();
    let search_fn = lua.create_function(
        move |lua, (query, category_str): (String, Option<String>)| {
            let bridge = bridge_clone.clone();

            // Parse category if provided
            let category = category_str.map(|cat_str| parse_template_category(&cat_str));

            // Call bridge.search_templates()
            let templates = bridge.search_templates(&query, category);

            // Convert Vec<TemplateMetadata> to Lua array
            let list_table = lua.create_table()?;
            for (i, metadata) in templates.into_iter().enumerate() {
                let metadata_table = template_metadata_to_lua_table(lua, &metadata)?;
                list_table.set(i + 1, metadata_table)?;
            }
            Ok(list_table)
        },
    )?;

    // Create Template.schema(name) function
    let bridge_clone = bridge.clone();
    let schema_fn = lua.create_function(move |lua, name: String| {
        let bridge = bridge_clone.clone();

        // Call bridge.get_template_schema()
        let schema = bridge.get_template_schema(&name).map_err(|e| {
            mlua::Error::RuntimeError(format!("Failed to get template schema: {e}"))
        })?;

        // Convert ConfigSchema to Lua table
        config_schema_to_lua_table(lua, &schema)
    })?;

    // Create Template.estimate_cost(name, params) function - ASYNC (optional bonus method)
    let bridge_clone = bridge;
    let estimate_cost_fn =
        lua.create_function(move |lua, (name, params_table): (String, Table)| {
            let bridge = bridge_clone.clone();

            // Use block_on_async_lua for async execution
            let result = block_on_async_lua(
                "template_estimate_cost",
                async move {
                    // Convert Lua table to TemplateParams
                    let params = lua_table_to_template_params(lua, params_table)?;

                    // Call bridge.estimate_cost()
                    let estimate_opt = bridge.estimate_cost(&name, &params).await.map_err(|e| {
                        mlua::Error::RuntimeError(format!(
                            "Failed to estimate cost for template '{name}': {e}"
                        ))
                    })?;

                    // Convert Option<CostEstimate> to Lua
                    if let Some(estimate) = estimate_opt {
                        let estimate_table = lua.create_table()?;
                        if let Some(tokens) = estimate.estimated_tokens {
                            #[allow(clippy::cast_precision_loss)]
                            estimate_table.set("estimated_tokens", tokens as f64)?;
                        }
                        if let Some(cost) = estimate.estimated_cost_usd {
                            estimate_table.set("estimated_cost_usd", cost)?;
                        }
                        if let Some(duration) = estimate.estimated_duration_ms {
                            #[allow(clippy::cast_precision_loss)]
                            estimate_table.set("estimated_duration_ms", duration as f64)?;
                        }
                        estimate_table.set("confidence", estimate.confidence)?;
                        Ok(Value::Table(estimate_table))
                    } else {
                        Ok(Value::Nil)
                    }
                },
                None,
            )?;

            Ok(result)
        })?;

    // Set functions on Template table
    template_table.set("list", list_fn)?;
    template_table.set("info", info_fn)?;
    template_table.set("execute", execute_fn)?;
    template_table.set("search", search_fn)?;
    template_table.set("schema", schema_fn)?;
    template_table.set("estimate_cost", estimate_cost_fn)?;

    // Set Template as global
    lua.globals().set("Template", template_table)?;

    Ok(())
}

/// Parse category string to `TemplateCategory` enum
///
/// Supports: "research", "chat", "analysis", "codegen", "document", "workflow"
/// Unknown categories are treated as Custom
fn parse_template_category(category_str: &str) -> TemplateCategory {
    match category_str.to_lowercase().as_str() {
        "research" => TemplateCategory::Research,
        "chat" => TemplateCategory::Chat,
        "analysis" => TemplateCategory::Analysis,
        "codegen" => TemplateCategory::CodeGen,
        "document" => TemplateCategory::Document,
        "workflow" => TemplateCategory::Workflow,
        custom => TemplateCategory::Custom(custom.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_category() {
        assert!(matches!(
            parse_template_category("research"),
            TemplateCategory::Research
        ));
        assert!(matches!(
            parse_template_category("Chat"),
            TemplateCategory::Chat
        ));
        assert!(matches!(
            parse_template_category("ANALYSIS"),
            TemplateCategory::Analysis
        ));

        // Custom category
        if let TemplateCategory::Custom(name) = parse_template_category("my-custom") {
            assert_eq!(name, "my-custom");
        } else {
            panic!("Expected Custom category");
        }
    }
}
