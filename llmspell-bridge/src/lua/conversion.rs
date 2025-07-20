//! ABOUTME: Consolidated Lua-specific type conversions
//! ABOUTME: All Lua <-> Rust type conversions in one place

use crate::conversion::{FromScriptValue, ScriptValue};
use llmspell_core::types::{
    AgentInput, AgentOutput, ColorSpace, ImageFormat, ImageMetadata, MediaContent, MediaType,
    ToolOutput,
};
use llmspell_core::{ExecutionContext, LLMSpellError, Result};
use mlua::{Error as LuaError, Lua, Table, Value as LuaValue};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

// ===== Core Lua <-> JSON conversions =====

/// Convert Lua value to JSON value
pub fn lua_value_to_json(value: LuaValue) -> mlua::Result<JsonValue> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(i.into())),
        LuaValue::Number(n) => {
            if n.is_finite() {
                Ok(serde_json::Number::from_f64(n)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null))
            } else {
                Ok(JsonValue::Null)
            }
        }
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(table) => lua_table_to_json(table),
        _ => Err(LuaError::FromLuaConversionError {
            from: value.type_name(),
            to: "JSON",
            message: Some("Unsupported Lua type for JSON conversion".to_string()),
        }),
    }
}

/// Convert Lua table to JSON value
pub fn lua_table_to_json(table: Table) -> mlua::Result<JsonValue> {
    // Check if it's an array by looking for numeric keys starting at 1
    let len = table.raw_len();
    if len > 0 {
        let mut is_array = true;
        for i in 1..=len {
            if table.get::<_, LuaValue>(i).is_err() {
                is_array = false;
                break;
            }
        }

        if is_array {
            let mut array = Vec::with_capacity(len);
            for i in 1..=len {
                let value: LuaValue = table.get(i)?;
                array.push(lua_value_to_json(value)?);
            }
            return Ok(JsonValue::Array(array));
        }
    }

    // Otherwise, treat as object
    let mut map = serde_json::Map::new();
    for pair in table.pairs::<LuaValue, LuaValue>() {
        let (key, value) = pair?;
        let key_str = match key {
            LuaValue::String(s) => s.to_str()?.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            LuaValue::Number(n) => n.to_string(),
            _ => continue, // Skip non-string keys
        };
        map.insert(key_str, lua_value_to_json(value)?);
    }
    Ok(JsonValue::Object(map))
}

/// Convert JSON value to Lua value
pub fn json_to_lua_value<'lua>(lua: &'lua Lua, json: &JsonValue) -> mlua::Result<LuaValue<'lua>> {
    match json {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        JsonValue::String(s) => lua.create_string(s).map(LuaValue::String),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, value) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (key, value) in obj {
                table.set(key.as_str(), json_to_lua_value(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

// ===== Agent conversions =====

/// Convert Lua table to AgentInput
pub fn lua_table_to_agent_input(lua: &Lua, table: Table) -> mlua::Result<AgentInput> {
    // Extract text (required)
    let text: String = table.get("text").unwrap_or_default();

    // Extract parameters (optional)
    let mut parameters = HashMap::new();
    if let Ok(params_table) = table.get::<_, Table>("parameters") {
        for (key, value) in params_table.pairs::<String, LuaValue>().flatten() {
            if let Ok(json_value) = lua_value_to_json(value) {
                parameters.insert(key, json_value);
            }
        }
    }

    // Extract context (optional) - for now we don't support full ExecutionContext from Lua
    let _context: Option<ExecutionContext> = None;

    // Extract output_modalities (optional)
    let mut output_modalities = Vec::new();
    if let Ok(modalities_table) = table.get::<_, Table>("output_modalities") {
        for value in modalities_table.sequence_values::<String>().flatten() {
            // Map string to MediaType
            match value.as_str() {
                "text" => output_modalities.push(MediaType::Text),
                "image" => output_modalities.push(MediaType::Image),
                "audio" => output_modalities.push(MediaType::Audio),
                "video" => output_modalities.push(MediaType::Video),
                "binary" => output_modalities.push(MediaType::Binary),
                _ => {} // Skip unknown types
            }
        }
    }

    // Extract media content (optional)
    let mut media = Vec::new();
    if let Ok(media_table) = table.get::<_, Table>("media") {
        // Handle array of media items
        for entry in media_table.sequence_values::<Table>().flatten() {
            if let Ok(media_content) = parse_media_content(lua, entry) {
                media.push(media_content);
            }
        }
    } else if let Ok(media_table) = table.get::<_, Table>("image") {
        // Legacy single image support
        if let Ok(media_content) = parse_media_content(lua, media_table) {
            media.push(media_content);
        }
    }

    Ok(AgentInput {
        text,
        parameters,
        context: None,
        output_modalities,
        media,
    })
}

/// Parse media content from Lua table
fn parse_media_content(_lua: &Lua, table: Table) -> mlua::Result<MediaContent> {
    let base64: String = table.get("base64")?;
    let data = base64::Engine::decode(&base64::engine::general_purpose::STANDARD, &base64)
        .map_err(|e| LuaError::RuntimeError(format!("Failed to decode base64 image: {}", e)))?;

    let format = table
        .get::<_, String>("format")
        .ok()
        .and_then(|f| match f.as_str() {
            "png" => Some(ImageFormat::Png),
            "jpeg" | "jpg" => Some(ImageFormat::Jpeg),
            "webp" => Some(ImageFormat::Webp),
            "gif" => Some(ImageFormat::Gif),
            "svg" => Some(ImageFormat::Svg),
            "tiff" => Some(ImageFormat::Tiff),
            _ => None,
        })
        .unwrap_or(ImageFormat::Png);

    let metadata = if let Ok(width) = table.get::<_, u32>("width") {
        if let Ok(height) = table.get::<_, u32>("height") {
            let color_space = table
                .get::<_, String>("color_space")
                .ok()
                .and_then(|cs| match cs.as_str() {
                    "rgb" => Some(ColorSpace::RGB),
                    "rgba" => Some(ColorSpace::RGBA),
                    "grayscale" => Some(ColorSpace::Grayscale),
                    "cmyk" => Some(ColorSpace::CMYK),
                    _ => None,
                })
                .unwrap_or(ColorSpace::RGB);

            Some(ImageMetadata {
                width,
                height,
                color_space,
                has_transparency: false,
                dpi: None,
            })
        } else {
            None
        }
    } else {
        None
    };

    Ok(MediaContent::Image {
        data,
        format,
        metadata: metadata.unwrap_or(ImageMetadata {
            width: 0,
            height: 0,
            color_space: ColorSpace::RGB,
            has_transparency: false,
            dpi: None,
        }),
    })
}

/// Convert AgentOutput to Lua table
pub fn agent_output_to_lua_table(lua: &Lua, output: AgentOutput) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    // Set text
    table.set("text", output.text)?;

    // Set metadata
    let metadata_table = lua.create_table()?;
    if let Some(model) = output.metadata.model {
        metadata_table.set("model", model)?;
    }
    if let Some(token_count) = output.metadata.token_count {
        metadata_table.set("token_count", token_count)?;
    }
    if let Some(execution_time_ms) = output.metadata.execution_time_ms {
        metadata_table.set("execution_time_ms", execution_time_ms)?;
    }
    if let Some(confidence) = output.metadata.confidence {
        metadata_table.set("confidence", confidence)?;
    }
    for (key, value) in output.metadata.extra {
        metadata_table.set(key, json_to_lua_value(lua, &value)?)?;
    }
    table.set("metadata", metadata_table)?;

    // Set media if present
    if !output.media.is_empty() {
        let media_array = lua.create_table()?;
        for (i, media) in output.media.iter().enumerate() {
            let media_table = lua.create_table()?;
            match media {
                MediaContent::Image {
                    data,
                    format,
                    metadata,
                } => {
                    media_table.set(
                        "base64",
                        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, data),
                    )?;
                    media_table.set("type", "image")?;
                    media_table.set(
                        "format",
                        match format {
                            ImageFormat::Png => "png",
                            ImageFormat::Jpeg => "jpeg",
                            ImageFormat::Webp => "webp",
                            ImageFormat::Gif => "gif",
                            ImageFormat::Svg => "svg",
                            ImageFormat::Tiff => "tiff",
                        },
                    )?;
                    media_table.set("width", metadata.width)?;
                    media_table.set("height", metadata.height)?;
                    media_table.set(
                        "color_space",
                        match metadata.color_space {
                            ColorSpace::RGB => "rgb",
                            ColorSpace::RGBA => "rgba",
                            ColorSpace::Grayscale => "grayscale",
                            ColorSpace::CMYK => "cmyk",
                        },
                    )?;
                }
                _ => {} // Other media types not yet supported
            }

            media_array.set(i + 1, media_table)?;
        }
        table.set("media", media_array)?;
    }

    Ok(table)
}

// ===== Tool conversions =====

/// Convert Lua table to tool input (JSON)
pub fn lua_table_to_tool_input(_lua: &Lua, table: Table) -> mlua::Result<JsonValue> {
    lua_table_to_json(table)
}

/// Convert tool output to Lua table
pub fn tool_output_to_lua_table(lua: &Lua, output: ToolOutput) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    // Set success status
    table.set("success", output.success)?;

    // Convert the data to Lua
    let lua_value = json_to_lua_value(lua, &output.data)?;
    table.set("data", lua_value)?;

    // Add error if present
    if let Some(error) = output.error {
        table.set("error", error)?;
    }

    // Add execution time if present
    if let Some(execution_time_ms) = output.execution_time_ms {
        table.set("execution_time_ms", execution_time_ms)?;
    }

    Ok(table)
}

// ===== Workflow conversions =====

/// Convert Lua table to workflow parameters
pub fn lua_table_to_workflow_params(_lua: &Lua, table: Table) -> Result<JsonValue> {
    lua_table_to_json(table).map_err(|e| LLMSpellError::Component {
        message: format!("Failed to convert Lua table to workflow params: {}", e),
        source: None,
    })
}

/// Convert workflow result to Lua table  
pub fn workflow_result_to_lua_table(lua: &Lua, result: serde_json::Value) -> mlua::Result<Table> {
    match json_to_lua_value(lua, &result)? {
        LuaValue::Table(table) => Ok(table),
        _ => {
            // If it's not a table, wrap it in one
            let table = lua.create_table()?;
            table.set("result", json_to_lua_value(lua, &result)?)?;
            Ok(table)
        }
    }
}

/// Convert ScriptWorkflowResult to Lua table
pub fn script_workflow_result_to_lua_table(
    lua: &Lua,
    result: crate::conversion::ScriptWorkflowResult,
) -> mlua::Result<Table> {
    // Convert to JSON first, then to Lua
    let json_value = serde_json::to_value(result).map_err(|e| {
        LuaError::RuntimeError(format!("Failed to serialize workflow result: {}", e))
    })?;
    workflow_result_to_lua_table(lua, json_value)
}

// ===== Trait implementations =====

// Note: We cannot implement ToScriptValue for LuaValue because it requires a Lua context
// Use the json_to_lua_value function directly when you have a Lua context

impl FromScriptValue<LuaValue<'_>> for ScriptValue {
    fn from_script_value(value: LuaValue<'_>) -> Result<Self> {
        match value {
            LuaValue::Nil => Ok(ScriptValue::Null),
            LuaValue::Boolean(b) => Ok(ScriptValue::Bool(b)),
            LuaValue::Integer(i) => Ok(ScriptValue::Number(i as f64)),
            LuaValue::Number(n) => Ok(ScriptValue::Number(n)),
            LuaValue::String(s) => Ok(ScriptValue::String(
                s.to_str()
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to convert Lua string: {}", e),
                        source: None,
                    })?
                    .to_string(),
            )),
            LuaValue::Table(_) => {
                // Need Lua context to properly convert tables
                Err(LLMSpellError::Component {
                    message: "Cannot convert Lua table without Lua context".to_string(),
                    source: None,
                })
            }
            _ => Err(LLMSpellError::Component {
                message: format!(
                    "Cannot convert Lua type {} to ScriptValue",
                    value.type_name()
                ),
                source: None,
            }),
        }
    }
}
