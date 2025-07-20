//! ABOUTME: Lua-specific JSON global implementation
//! ABOUTME: Provides JSON.parse() and JSON.stringify() for Lua scripts

use llmspell_core::error::LLMSpellError;
use mlua::Lua;

/// Inject JSON global into Lua environment
pub fn inject_json_global(lua: &Lua) -> Result<(), LLMSpellError> {
    // Use the existing JSON API implementation
    let api_def = crate::engine::types::JsonApiDefinition {
        global_name: "JSON".to_string(),
        parse_function: "parse".to_string(),
        stringify_function: "stringify".to_string(),
    };

    crate::lua::api::inject_json_api(lua, &api_def)?;
    Ok(())
}
