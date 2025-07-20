//! ABOUTME: Lua-specific workflow result conversion
//! ABOUTME: Handles transformation of workflow results for Lua script consumption

use crate::workflow_results::ScriptWorkflowResult;
use llmspell_core::{LLMSpellError, Result};
use mlua::{Lua, Table, Value as LuaValue};
use serde_json::Value;

// Helper to convert mlua errors
fn lua_error_to_llmspell(e: mlua::Error) -> LLMSpellError {
    LLMSpellError::Script {
        message: format!("Lua error: {}", e),
        language: Some("lua".to_string()),
        line: None,
        source: None,
    }
}

/// Convert script workflow result to Lua table
pub fn script_result_to_lua_table(lua: &Lua, result: ScriptWorkflowResult) -> Result<Table> {
    let table = lua.create_table().map_err(lua_error_to_llmspell)?;

    // Basic fields
    table
        .set("success", result.success)
        .map_err(lua_error_to_llmspell)?;
    table
        .set("workflow_type", result.workflow_type)
        .map_err(lua_error_to_llmspell)?;
    table
        .set("workflow_name", result.workflow_name)
        .map_err(lua_error_to_llmspell)?;
    table
        .set("duration_ms", result.duration_ms)
        .map_err(lua_error_to_llmspell)?;

    // Convert data
    let data_value = serde_json_to_lua_value(lua, result.data)?;
    table
        .set("data", data_value)
        .map_err(lua_error_to_llmspell)?;

    // Convert error if present
    if let Some(error) = result.error {
        let error_table = lua.create_table().map_err(lua_error_to_llmspell)?;
        error_table
            .set("type", error.error_type)
            .map_err(lua_error_to_llmspell)?;
        error_table
            .set("message", error.message)
            .map_err(lua_error_to_llmspell)?;
        if let Some(location) = error.location {
            error_table
                .set("location", location)
                .map_err(lua_error_to_llmspell)?;
        }
        if let Some(details) = error.details {
            let details_value = serde_json_to_lua_value(lua, details)?;
            error_table
                .set("details", details_value)
                .map_err(lua_error_to_llmspell)?;
        }
        table
            .set("error", error_table)
            .map_err(lua_error_to_llmspell)?;
    }

    // Convert metadata
    let metadata_table = lua.create_table().map_err(lua_error_to_llmspell)?;
    metadata_table
        .set("start_time", result.metadata.start_time)
        .map_err(lua_error_to_llmspell)?;
    metadata_table
        .set("end_time", result.metadata.end_time)
        .map_err(lua_error_to_llmspell)?;
    if let Some(steps) = result.metadata.steps_executed {
        metadata_table
            .set("steps_executed", steps)
            .map_err(lua_error_to_llmspell)?;
    }
    if let Some(succeeded) = result.metadata.steps_succeeded {
        metadata_table
            .set("steps_succeeded", succeeded)
            .map_err(lua_error_to_llmspell)?;
    }
    if let Some(failed) = result.metadata.steps_failed {
        metadata_table
            .set("steps_failed", failed)
            .map_err(lua_error_to_llmspell)?;
    }
    if let Some(extra) = result.metadata.extra {
        let extra_value = serde_json_to_lua_value(lua, extra)?;
        metadata_table
            .set("extra", extra_value)
            .map_err(lua_error_to_llmspell)?;
    }
    table
        .set("metadata", metadata_table)
        .map_err(lua_error_to_llmspell)?;

    Ok(table)
}

fn serde_json_to_lua_value(lua: &Lua, value: Value) -> Result<LuaValue> {
    match value {
        Value::Null => Ok(LuaValue::Nil),
        Value::Bool(b) => Ok(LuaValue::Boolean(b)),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        Value::String(s) => Ok(LuaValue::String(
            lua.create_string(&s).map_err(lua_error_to_llmspell)?,
        )),
        Value::Array(arr) => {
            let table = lua.create_table().map_err(lua_error_to_llmspell)?;
            for (i, val) in arr.into_iter().enumerate() {
                table
                    .set(i + 1, serde_json_to_lua_value(lua, val)?)
                    .map_err(lua_error_to_llmspell)?;
            }
            Ok(LuaValue::Table(table))
        }
        Value::Object(obj) => {
            let table = lua.create_table().map_err(lua_error_to_llmspell)?;
            for (key, val) in obj {
                table
                    .set(key, serde_json_to_lua_value(lua, val)?)
                    .map_err(lua_error_to_llmspell)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}
