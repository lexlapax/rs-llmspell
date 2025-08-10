//! ABOUTME: Lua hook adapter for cross-language hook execution
//! ABOUTME: Implements `HookAdapter` trait to convert between Rust and Lua types

use crate::lua::conversion::{json_to_lua_value, lua_table_to_json};
use llmspell_hooks::{HookAdapter, HookContext, HookResult};
use mlua::{Lua, Table, Value};
use serde_json::json;
use std::any::Any;

/// Lua-specific hook adapter
pub struct LuaHookAdapter;

impl Default for LuaHookAdapter {
    fn default() -> Self {
        Self
    }
}

impl LuaHookAdapter {
    /// Create a new Lua hook adapter
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Convert `HookContext` to Lua table
    ///
    /// # Errors
    ///
    /// Returns an error if Lua table creation or field setting fails
    pub fn hook_context_to_lua_table<'lua>(
        &self,
        lua: &'lua Lua,
        context: &HookContext,
    ) -> mlua::Result<Table<'lua>> {
        let table = lua.create_table()?;

        // Set basic fields
        table.set("point", format!("{:?}", context.point))?;
        table.set("correlation_id", context.correlation_id.to_string())?;

        // Component ID
        let component_table = lua.create_table()?;
        component_table.set("type", format!("{:?}", context.component_id.component_type))?;
        component_table.set("name", context.component_id.name.clone())?;
        table.set("component_id", component_table)?;

        // Language
        table.set("language", context.language.to_string())?;

        // Metadata
        let metadata_table = lua.create_table()?;
        for (key, value) in &context.metadata {
            metadata_table.set(key.as_str(), value.as_str())?;
        }
        table.set("metadata", metadata_table)?;

        // Data (convert JSON to Lua)
        let data_table = lua.create_table()?;
        for (key, value) in &context.data {
            if let Ok(lua_value) = json_to_lua_value(lua, value) {
                data_table.set(key.as_str(), lua_value)?;
            }
        }
        table.set("data", data_table)?;

        Ok(table)
    }

    /// Convert Lua value to `HookResult`
    ///
    /// # Errors
    ///
    /// Returns an error if the Lua value cannot be converted to a `HookResult`
    pub fn lua_value_to_hook_result(&self, _lua: &Lua, value: Value) -> mlua::Result<HookResult> {
        match value {
            Value::String(s) => {
                let s = s.to_str()?;
                match s {
                    "continue" => Ok(HookResult::Continue),
                    "skip" | "skipped" => {
                        Ok(HookResult::Skipped("Skipped by Lua hook".to_string()))
                    }
                    _ => Ok(HookResult::Cancel(s.to_string())),
                }
            }
            Value::Table(table) => {
                // Check for result type
                if let Ok(Value::String(result_type)) = table.get::<_, Value>("type") {
                    let result_type = result_type.to_str()?;
                    match result_type {
                        "modified" => {
                            // Get the modified data
                            if let Ok(data) = table.get::<_, Table>("data") {
                                let json_data = lua_table_to_json(data)?;
                                Ok(HookResult::Modified(json_data))
                            } else {
                                Ok(HookResult::Modified(json!({})))
                            }
                        }
                        "cancel" => {
                            let reason = table
                                .get::<_, String>("reason")
                                .unwrap_or_else(|_| "Cancelled by Lua hook".to_string());
                            Ok(HookResult::Cancel(reason))
                        }
                        "redirect" => {
                            let target = table.get::<_, String>("target").map_err(|_| {
                                mlua::Error::FromLuaConversionError {
                                    from: "table",
                                    to: "HookResult::Redirect",
                                    message: Some(
                                        "Missing 'target' field for redirect".to_string(),
                                    ),
                                }
                            })?;
                            Ok(HookResult::Redirect(target))
                        }
                        "replace" => {
                            if let Ok(data) = table.get::<_, Table>("data") {
                                let json_data = lua_table_to_json(data)?;
                                Ok(HookResult::Replace(json_data))
                            } else {
                                Ok(HookResult::Replace(json!({})))
                            }
                        }
                        "retry" => {
                            let delay_ms = table.get::<_, u64>("delay_ms").unwrap_or(1000);
                            let max_attempts = table.get::<_, u32>("max_attempts").unwrap_or(3);
                            Ok(HookResult::Retry {
                                delay: std::time::Duration::from_millis(delay_ms),
                                max_attempts,
                            })
                        }
                        "skipped" => {
                            let reason = table
                                .get::<_, String>("reason")
                                .unwrap_or_else(|_| "Skipped by Lua hook".to_string());
                            Ok(HookResult::Skipped(reason))
                        }
                        _ => Ok(HookResult::Continue),
                    }
                } else {
                    Ok(HookResult::Continue)
                }
            }
            _ => Ok(HookResult::Continue),
        }
    }
}

impl HookAdapter for LuaHookAdapter {
    type Context = Box<dyn Any>;
    type Result = Box<dyn Any>;

    fn adapt_context(&self, ctx: &HookContext) -> Self::Context {
        // We can't hold Lua reference here, so just pass a simplified version
        // The actual conversion will happen in the hook execution
        Box::new(serde_json::json!({
            "point": format!("{:?}", ctx.point),
            "component_id": {
                "type": format!("{:?}", ctx.component_id.component_type),
                "name": ctx.component_id.name.clone()
            },
            "correlation_id": ctx.correlation_id.to_string(),
            "language": ctx.language.to_string(),
            "metadata": ctx.metadata.clone(),
            "data": ctx.data.clone()
        }))
    }

    fn adapt_result(&self, result: Self::Result) -> HookResult {
        result
            .downcast_ref::<HookResult>()
            .map_or(HookResult::Continue, Clone::clone)
    }

    fn extract_error(&self, result: &Self::Result) -> Option<String> {
        result.downcast_ref::<String>().cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::types::{ComponentId, ComponentType, HookPoint};
    #[test]
    fn test_lua_hook_adapter() {
        let lua = Lua::new();
        let adapter = LuaHookAdapter::new();

        // Create a test context
        let mut context = HookContext::new(
            HookPoint::BeforeToolExecution,
            ComponentId::new(ComponentType::Tool, "test-tool".to_string()),
        );
        context.insert_metadata("test_key".to_string(), "test_value".to_string());

        // Test context adaptation
        let lua_context = adapter.hook_context_to_lua_table(&lua, &context).unwrap();
        assert_eq!(
            lua_context.get::<_, String>("point").unwrap(),
            "BeforeToolExecution"
        );

        // Test result adaptation
        let boxed_result = Box::new(HookResult::Continue) as Box<dyn Any>;
        let hook_result = adapter.adapt_result(boxed_result);
        assert!(matches!(hook_result, HookResult::Continue));
    }
}
