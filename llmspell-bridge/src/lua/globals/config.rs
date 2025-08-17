//! ABOUTME: Lua-specific configuration global implementation
//! ABOUTME: Provides Config table with methods for configuration access

use crate::config_bridge::ConfigBridge;
use llmspell_config::providers::ProviderConfig;
use llmspell_core::error::LLMSpellError;
use mlua::{Lua, LuaSerdeExt, Result as LuaResult, Table, Value};
use std::sync::Arc;

/// Inject the Config global into Lua
///
/// # Errors
///
/// Returns an error if Lua table creation or function injection fails
#[allow(clippy::too_many_lines)]
pub fn inject_config_global(lua: &Lua, bridge: &Arc<ConfigBridge>) -> LuaResult<()> {
    let bridge = Arc::clone(bridge);
    let config_table = lua.create_table()?;

    // Config.get() - Get full configuration as table
    let bridge_clone = bridge.clone();
    config_table.set(
        "get",
        lua.create_function(move |lua, ()| {
            let json = bridge_clone.get().map_err(mlua::Error::external)?;

            lua.to_value(&json)
        })?,
    )?;

    // Config.getDefaultEngine() - Get default engine
    let bridge_clone = bridge.clone();
    config_table.set(
        "getDefaultEngine",
        lua.create_function(move |_, ()| {
            bridge_clone
                .get_default_engine()
                .map_err(mlua::Error::external)
        })?,
    )?;

    // Config.getProvider(name) - Get provider configuration
    let bridge_clone = bridge.clone();
    config_table.set(
        "getProvider",
        lua.create_function(move |lua, name: String| {
            let provider = bridge_clone
                .get_provider(&name)
                .map_err(mlua::Error::external)?;

            provider.map_or_else(|| Ok(Value::Nil), |json| lua.to_value(&json))
        })?,
    )?;

    // Config.listProviders() - List all provider names
    let bridge_clone = bridge.clone();
    config_table.set(
        "listProviders",
        lua.create_function(move |lua, ()| {
            let providers = bridge_clone
                .list_providers()
                .map_err(mlua::Error::external)?;

            let table = lua.create_table()?;
            for (i, name) in providers.iter().enumerate() {
                table.set(i + 1, name.clone())?;
            }
            Ok(table)
        })?,
    )?;

    // Config.getSecurity() - Get security settings
    let bridge_clone = bridge.clone();
    config_table.set(
        "getSecurity",
        lua.create_function(move |lua, ()| {
            let security = bridge_clone.get_security().map_err(mlua::Error::external)?;

            let json = serde_json::to_value(&security).map_err(|e| {
                mlua::Error::external(LLMSpellError::Configuration {
                    message: format!("Failed to serialize security: {e}"),
                    source: None,
                })
            })?;

            lua.to_value(&json)
        })?,
    )?;

    // Config.getTools() - Get tools configuration
    let bridge_clone = bridge.clone();
    config_table.set(
        "getTools",
        lua.create_function(move |lua, ()| {
            let tools = bridge_clone
                .get_tools_config()
                .map_err(mlua::Error::external)?;

            let json = serde_json::to_value(&tools).map_err(|e| {
                mlua::Error::external(LLMSpellError::Configuration {
                    message: format!("Failed to serialize tools: {e}"),
                    source: None,
                })
            })?;

            lua.to_value(&json)
        })?,
    )?;

    // Config.isFileAccessAllowed() - Check file access permission
    let bridge_clone = bridge.clone();
    config_table.set(
        "isFileAccessAllowed",
        lua.create_function(move |_, ()| {
            bridge_clone
                .is_file_access_allowed()
                .map_err(mlua::Error::external)
        })?,
    )?;

    // Config.isNetworkAccessAllowed() - Check network access permission
    let bridge_clone = bridge.clone();
    config_table.set(
        "isNetworkAccessAllowed",
        lua.create_function(move |_, ()| {
            bridge_clone
                .is_network_access_allowed()
                .map_err(mlua::Error::external)
        })?,
    )?;

    // Config.getSection(name) - Get specific configuration section
    let bridge_clone = bridge.clone();
    config_table.set(
        "getSection",
        lua.create_function(move |lua, section: String| {
            let json = bridge_clone
                .section_to_json(&section)
                .map_err(mlua::Error::external)?;

            lua.to_value(&json)
        })?,
    )?;

    // Config.getPermissions() - Get current permissions
    let bridge_clone = bridge.clone();
    config_table.set(
        "getPermissions",
        lua.create_function(move |lua, ()| {
            let permissions = bridge_clone.permissions();
            let table = lua.create_table()?;
            table.set("read", permissions.read)?;
            table.set("modify_providers", permissions.modify_providers)?;
            table.set("modify_tools", permissions.modify_tools)?;
            table.set("modify_runtime", permissions.modify_runtime)?;
            table.set("modify_security", permissions.modify_security)?;
            table.set("access_secrets", permissions.access_secrets)?;

            if let Some(ref providers) = permissions.allowed_providers {
                let providers_table = lua.create_table()?;
                for (i, name) in providers.iter().enumerate() {
                    providers_table.set(i + 1, name.clone())?;
                }
                table.set("allowed_providers", providers_table)?;
            }

            if let Some(ref tools) = permissions.allowed_tools {
                let tools_table = lua.create_table()?;
                for (i, name) in tools.iter().enumerate() {
                    tools_table.set(i + 1, name.clone())?;
                }
                table.set("allowed_tools", tools_table)?;
            }

            Ok(table)
        })?,
    )?;

    // Config.addAllowedPath(path) - Add allowed path for file operations
    let bridge_clone = bridge.clone();
    config_table.set(
        "addAllowedPath",
        lua.create_function(move |_, path: String| {
            bridge_clone
                .add_allowed_path(&path)
                .map_err(mlua::Error::external)?;
            Ok(())
        })?,
    )?;

    // Config.setProvider(name, config) - Set provider configuration
    let bridge_clone = bridge.clone();
    config_table.set(
        "setProvider",
        lua.create_function(move |_, (name, config): (String, Table)| {
            // Convert Lua table to JSON string then to ProviderConfig
            let json_str = serde_json::to_string(&config).map_err(|e| {
                mlua::Error::external(LLMSpellError::Configuration {
                    message: format!("Invalid provider config: {e}"),
                    source: None,
                })
            })?;

            let provider: ProviderConfig = serde_json::from_str(&json_str).map_err(|e| {
                mlua::Error::external(LLMSpellError::Configuration {
                    message: format!("Invalid provider config: {e}"),
                    source: None,
                })
            })?;

            bridge_clone
                .set_provider(&name, &provider)
                .map_err(mlua::Error::external)?;
            Ok(())
        })?,
    )?;

    // Config.setSecurity(config) - Set security configuration (DANGEROUS!)
    let bridge_clone = bridge.clone();
    config_table.set(
        "setSecurity",
        lua.create_function(move |_, config: Table| {
            // Convert Lua table to SecurityConfig
            let json_str = serde_json::to_string(&config).map_err(|e| {
                mlua::Error::external(LLMSpellError::Configuration {
                    message: format!("Invalid security config: {e}"),
                    source: None,
                })
            })?;

            let security: llmspell_config::SecurityConfig = serde_json::from_str(&json_str)
                .map_err(|e| {
                    mlua::Error::external(LLMSpellError::Configuration {
                        message: format!("Invalid security config: {e}"),
                        source: None,
                    })
                })?;

            bridge_clone
                .set_security(&security)
                .map_err(mlua::Error::external)?;
            Ok(())
        })?,
    )?;

    // Config.snapshot() - Create a configuration snapshot
    let bridge_clone = bridge.clone();
    config_table.set(
        "snapshot",
        lua.create_function(move |_, ()| {
            bridge_clone.snapshot().map_err(mlua::Error::external)?;
            Ok(())
        })?,
    )?;

    // Config.restoreSnapshot(timestamp) - Restore from snapshot
    let bridge_clone = bridge.clone();
    config_table.set(
        "restoreSnapshot",
        lua.create_function(move |_, timestamp: u64| {
            bridge_clone
                .restore_snapshot(timestamp)
                .map_err(mlua::Error::external)?;
            Ok(())
        })?,
    )?;

    // Config.toJson() - Export full config as JSON
    config_table.set(
        "toJson",
        lua.create_function(move |lua, ()| {
            let json = bridge.to_json().map_err(mlua::Error::external)?;
            lua.to_value(&json)
        })?,
    )?;

    // Register the Config global
    lua.globals().set("Config", config_table)?;

    Ok(())
}

/// Create example Lua code for Config usage
#[must_use]
pub const fn config_lua_examples() -> &'static str {
    r#"
-- Get full configuration
local config = Config.get()
print("Default engine:", config.default_engine)

-- Get specific provider
local openai = Config.getProvider("openai")
if openai then
    print("OpenAI model:", openai.model)
end

-- List all providers
local providers = Config.listProviders()
for i, name in ipairs(providers) do
    print("Provider " .. i .. ":", name)
end

-- Check security settings
local security = Config.getSecurity()
print("File access allowed:", security.allow_file_access)
print("Network access allowed:", security.allow_network_access)

-- Check permissions
local perms = Config.getPermissions()
print("Can modify providers:", perms.modify_providers)
print("Can modify security:", perms.modify_security)

-- Get tools configuration
local tools = Config.getTools()
if tools.file_operations then
    print("Allowed paths:", table.concat(tools.file_operations.allowed_paths, ", "))
end

-- Modify configuration (if permitted)
if perms.modify_providers then
    -- Add a new provider
    Config.setProvider("custom", {
        provider_type = "openai",
        api_key_env = "CUSTOM_API_KEY",
        model = "gpt-4"
    })
end

if perms.modify_tools then
    -- Add allowed path for file operations
    Config.addAllowedPath("/tmp/myapp")
end

-- Create snapshot before changes
Config.snapshot()

-- Make changes...

-- Restore if needed
-- Config.restoreSnapshot(timestamp)

-- Export configuration
local json = Config.toJson()
print("Configuration exported")
"#
}
