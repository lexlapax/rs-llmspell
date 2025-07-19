//! ABOUTME: Parameter conversion between script types and agent types
//! ABOUTME: Handles transformation of data between Lua/JS and Rust agent interfaces

use llmspell_core::types::{AgentInput, AgentOutput, MediaContent, MediaType};
use llmspell_core::{ExecutionContext, LLMSpellError};
use mlua::{Lua, Table, Value as LuaValue};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

/// Convert Lua table to AgentInput
pub fn lua_table_to_agent_input(_lua: &Lua, table: Table) -> mlua::Result<AgentInput> {
    use llmspell_core::types::ColorSpace;
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
    let context: Option<ExecutionContext> = None;

    // Extract output_modalities (optional)
    let mut output_modalities = Vec::new();
    if let Ok(modalities_table) = table.get::<_, Table>("output_modalities") {
        for modality_str in modalities_table.sequence_values::<String>().flatten() {
            // Convert string to MediaType
            let media_type = match modality_str.as_str() {
                "text" => MediaType::Text,
                "image" => MediaType::Image,
                "audio" => MediaType::Audio,
                "video" => MediaType::Video,
                _ => MediaType::Text, // Default to text
            };
            output_modalities.push(media_type);
        }
    }

    // Extract media content (optional)
    let mut media = Vec::new();
    if let Ok(media_table) = table.get::<_, Table>("media") {
        for media_item in media_table.sequence_values::<Table>().flatten() {
            if let Ok(media_type) = media_item.get::<_, String>("type") {
                match media_type.as_str() {
                    "text" => {
                        if let Ok(content) = media_item.get::<_, String>("content") {
                            media.push(MediaContent::Text(content));
                        }
                    }
                    "image" => {
                        // For now, we'll create a placeholder image
                        // In a real implementation, we'd handle base64 data or file paths
                        if let Ok(data) = media_item.get::<_, String>("data") {
                            // Assume data is base64 encoded
                            use base64::Engine as _;
                            if let Ok(bytes) =
                                base64::engine::general_purpose::STANDARD.decode(&data)
                            {
                                media.push(MediaContent::Image {
                                    data: bytes,
                                    format: llmspell_core::types::ImageFormat::Png,
                                    metadata: llmspell_core::types::ImageMetadata {
                                        width: media_item.get("width").unwrap_or(0),
                                        height: media_item.get("height").unwrap_or(0),
                                        color_space: ColorSpace::RGB, // Default to RGB
                                        has_transparency: media_item
                                            .get("has_transparency")
                                            .unwrap_or(false),
                                        dpi: media_item.get("dpi").ok(),
                                    },
                                });
                            }
                        }
                    }
                    // Add more media types as needed
                    _ => {}
                }
            }
        }
    }

    // Create AgentInput
    let mut input = AgentInput::text(text);
    input.parameters = parameters;
    input.context = context;
    input.output_modalities = output_modalities;
    input.media = media;

    Ok(input)
}

/// Convert AgentOutput to Lua table
pub fn agent_output_to_lua_table(lua: &Lua, output: AgentOutput) -> mlua::Result<Table> {
    let table = lua.create_table()?;

    // Add text
    table.set("text", output.text)?;

    // Add media if present - simplified for now
    if !output.media.is_empty() {
        let media_table = lua.create_table()?;
        for (i, media) in output.media.iter().enumerate() {
            let media_item = lua.create_table()?;
            match media {
                MediaContent::Text(text) => {
                    media_item.set("type", "text")?;
                    media_item.set("content", text.clone())?;
                }
                MediaContent::Image {
                    format, metadata, ..
                } => {
                    media_item.set("type", "image")?;
                    media_item.set("format", format!("{:?}", format))?;
                    media_item.set("width", metadata.width)?;
                    media_item.set("height", metadata.height)?;
                }
                MediaContent::Audio {
                    format, metadata, ..
                } => {
                    media_item.set("type", "audio")?;
                    media_item.set("format", format!("{:?}", format))?;
                    media_item.set("duration_ms", metadata.duration_ms)?;
                }
                MediaContent::Video {
                    format, metadata, ..
                } => {
                    media_item.set("type", "video")?;
                    media_item.set("format", format!("{:?}", format))?;
                    media_item.set("duration_ms", metadata.duration_ms)?;
                }
                MediaContent::Binary { mime_type, .. } => {
                    media_item.set("type", "binary")?;
                    media_item.set("mime_type", mime_type.clone())?;
                }
            }
            media_table.set(i + 1, media_item)?;
        }
        table.set("media", media_table)?;
    }

    // Add metadata
    let metadata_table = lua.create_table()?;
    if let Some(model) = &output.metadata.model {
        metadata_table.set("model", model.clone())?;
    }
    if let Some(token_count) = output.metadata.token_count {
        metadata_table.set("token_count", token_count)?;
    }
    if let Some(execution_time) = output.metadata.execution_time_ms {
        metadata_table.set("execution_time_ms", execution_time)?;
    }
    if let Some(confidence) = output.metadata.confidence {
        metadata_table.set("confidence", confidence)?;
    }

    // Add extra metadata if present
    if !output.metadata.extra.is_empty() {
        let extra_table = lua.create_table()?;
        for (key, value) in &output.metadata.extra {
            if let Ok(lua_value) = json_to_lua_value(lua, value.clone()) {
                extra_table.set(key.clone(), lua_value)?;
            }
        }
        metadata_table.set("extra", extra_table)?;
    }

    table.set("metadata", metadata_table)?;

    // Add tool calls if present
    if !output.tool_calls.is_empty() {
        let tool_calls_table = lua.create_table()?;
        for (i, call) in output.tool_calls.iter().enumerate() {
            let call_table = lua.create_table()?;
            call_table.set("tool_name", call.tool_name.clone())?;

            // Convert parameters
            let params_table = lua.create_table()?;
            for (key, value) in &call.parameters {
                if let Ok(lua_value) = json_to_lua_value(lua, value.clone()) {
                    params_table.set(key.clone(), lua_value)?;
                }
            }
            call_table.set("parameters", params_table)?;

            tool_calls_table.set(i + 1, call_table)?;
        }
        table.set("tool_calls", tool_calls_table)?;
    }

    Ok(table)
}

/// Convert JSON value to Lua value
pub fn json_to_lua_value(lua: &Lua, value: JsonValue) -> mlua::Result<LuaValue<'_>> {
    match value {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Ok(LuaValue::Nil)
            }
        }
        JsonValue::String(s) => Ok(LuaValue::String(lua.create_string(&s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, item) in arr.into_iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, item)?)?;
            }
            Ok(LuaValue::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (key, value) in obj {
                table.set(key, json_to_lua_value(lua, value)?)?;
            }
            Ok(LuaValue::Table(table))
        }
    }
}

/// Convert Lua value to JSON value
pub fn lua_value_to_json(value: LuaValue) -> mlua::Result<JsonValue> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number(serde_json::Number::from(i))),
        LuaValue::Number(n) => {
            if let Some(num) = serde_json::Number::from_f64(n) {
                Ok(JsonValue::Number(num))
            } else {
                Ok(JsonValue::Null)
            }
        }
        LuaValue::String(s) => Ok(JsonValue::String(s.to_str()?.to_string())),
        LuaValue::Table(t) => {
            // Check if it's an array
            if is_lua_array(&t) {
                let mut array = Vec::new();
                for i in 1..=t.len()? {
                    let value = t.get::<_, LuaValue>(i)?;
                    array.push(lua_value_to_json(value)?);
                }
                Ok(JsonValue::Array(array))
            } else {
                // It's an object
                let mut map = serde_json::Map::new();
                for pair in t.pairs::<LuaValue, LuaValue>() {
                    let (key, value) = pair?;
                    let key_str = match key {
                        LuaValue::String(s) => s.to_str()?.to_string(),
                        LuaValue::Integer(i) => i.to_string(),
                        LuaValue::Number(n) => n.to_string(),
                        _ => continue,
                    };
                    let json_value = lua_value_to_json(value)?;
                    map.insert(key_str, json_value);
                }
                Ok(JsonValue::Object(map))
            }
        }
        _ => Ok(JsonValue::Null),
    }
}

/// Check if a Lua table is an array
fn is_lua_array(table: &Table) -> bool {
    if let Ok(len) = table.len() {
        if len == 0 {
            return false;
        }
        // Check if all keys from 1 to len exist
        for i in 1..=len {
            if table.get::<_, LuaValue>(i).is_err() {
                return false;
            }
        }
        // Check if there are any non-numeric keys
        for (k, _) in table.clone().pairs::<LuaValue, LuaValue>().flatten() {
            match k {
                LuaValue::Integer(i) if i >= 1 && i <= len => continue,
                _ => return false,
            }
        }
        true
    } else {
        false
    }
}

/// Convert script error to LLMSpellError
pub fn script_error_to_llmspell(error: String) -> LLMSpellError {
    LLMSpellError::Component {
        message: format!("Script execution error: {}", error),
        source: None,
    }
}

/// Convert LLMSpellError to script-friendly format
pub fn llmspell_error_to_script(error: LLMSpellError) -> String {
    format!("Agent error: {}", error)
}

/// Convert Lua table to ToolInput (using AgentInput for tools)
pub fn lua_table_to_tool_input(_lua: &Lua, table: Table) -> mlua::Result<AgentInput> {
    // For tools, we convert the Lua table to AgentInput
    // The main difference is that tool parameters go into the `parameters` field
    let mut parameters = std::collections::HashMap::new();

    // Convert all table entries to parameters
    for pair in table.pairs::<LuaValue, LuaValue>().flatten() {
        let key = match pair.0 {
            LuaValue::String(s) => s.to_str()?.to_string(),
            LuaValue::Integer(i) => i.to_string(),
            LuaValue::Number(n) => n.to_string(),
            _ => continue,
        };
        let value = lua_value_to_json(pair.1)?;
        parameters.insert(key, value);
    }

    // Create AgentInput with parameters
    let mut agent_input = AgentInput::text(""); // Tools typically don't need text
    agent_input.parameters = parameters;

    Ok(agent_input)
}

/// Convert ToolOutput to Lua table (using AgentOutput for tools)
pub fn tool_output_to_lua_table(lua: &Lua, output: AgentOutput) -> mlua::Result<Table> {
    // Tools use AgentOutput, so we just reuse the existing conversion
    agent_output_to_lua_table(lua, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_json_conversion() {
        let lua = Lua::new();

        // Test primitives
        assert_eq!(lua_value_to_json(LuaValue::Nil).unwrap(), JsonValue::Null);
        assert_eq!(
            lua_value_to_json(LuaValue::Boolean(true)).unwrap(),
            JsonValue::Bool(true)
        );
        assert_eq!(
            lua_value_to_json(LuaValue::Integer(42)).unwrap(),
            JsonValue::Number(serde_json::Number::from(42))
        );

        // Test round-trip
        let original = JsonValue::Object(serde_json::Map::from_iter([
            ("name".to_string(), JsonValue::String("test".to_string())),
            (
                "value".to_string(),
                JsonValue::Number(serde_json::Number::from(123)),
            ),
        ]));

        let lua_value = json_to_lua_value(&lua, original.clone()).unwrap();
        let back_to_json = lua_value_to_json(lua_value).unwrap();

        assert_eq!(original, back_to_json);
    }

    #[test]
    fn test_agent_input_conversion() {
        let lua = Lua::new();

        let table = lua.create_table().unwrap();
        table.set("text", "Hello, agent!").unwrap();

        let params = lua.create_table().unwrap();
        params.set("temperature", 0.7).unwrap();
        params.set("max_tokens", 100).unwrap();
        table.set("parameters", params).unwrap();

        let input = lua_table_to_agent_input(&lua, table).unwrap();

        assert_eq!(input.text, "Hello, agent!");
        assert_eq!(input.parameters.len(), 2);
        assert_eq!(
            input.parameters.get("temperature"),
            Some(&JsonValue::Number(
                serde_json::Number::from_f64(0.7).unwrap()
            ))
        );
    }

    #[test]
    fn test_agent_output_conversion() {
        let lua = Lua::new();

        let mut output = AgentOutput::text("Response text");
        output.metadata = llmspell_core::types::OutputMetadata {
            model: Some("test-model".to_string()),
            token_count: Some(50),
            execution_time_ms: Some(100),
            confidence: Some(0.95),
            extra: HashMap::new(),
        };

        let table = agent_output_to_lua_table(&lua, output).unwrap();

        assert_eq!(table.get::<_, String>("text").unwrap(), "Response text");

        let metadata: Table = table.get("metadata").unwrap();
        assert_eq!(metadata.get::<_, String>("model").unwrap(), "test-model");
        assert_eq!(metadata.get::<_, u32>("token_count").unwrap(), 50);
    }
}
