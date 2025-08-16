//! ABOUTME: Lua-specific Provider global implementation
//! ABOUTME: Provides Lua bindings for LLM provider information and capabilities

use crate::globals::GlobalContext;
use crate::ProviderManager;
use llmspell_core::error::LLMSpellError;
use mlua::{Lua, Result as LuaResult, Table, Value};
use std::sync::Arc;

/// Inject Provider global into Lua environment
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function injection fails
/// - Global setting fails
#[allow(clippy::too_many_lines)]
pub fn inject_provider_global(
    lua: &Lua,
    _context: &GlobalContext,
    providers: &Arc<ProviderManager>,
) -> Result<(), LLMSpellError> {
    // Create the Provider table
    let provider_table = lua.create_table().map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Provider table: {e}"),
        source: None,
    })?;

    // Clone providers for closure capture
    let providers_for_list = providers.clone();

    // Provider.list() - List all available providers
    let list_fn = lua
        .create_function(move |lua, ()| -> LuaResult<Table> {
            let result = lua.create_table()?;

            // Get provider list from manager
            let provider_list = futures::executor::block_on(providers_for_list.list_providers());

            for (idx, provider_info) in provider_list.iter().enumerate() {
                let provider_table = lua.create_table()?;
                provider_table.set("name", provider_info.name.clone())?;
                provider_table.set("enabled", provider_info.enabled)?;

                // Add capabilities if available
                if let Some(caps) = &provider_info.capabilities {
                    let caps_table = lua.create_table()?;
                    caps_table.set("supports_streaming", caps.supports_streaming)?;
                    caps_table.set("supports_multimodal", caps.supports_multimodal)?;
                    caps_table.set("max_context_tokens", caps.max_context_tokens)?;

                    // Add available models
                    if !caps.available_models.is_empty() {
                        let models_table = lua.create_table()?;
                        for (model_idx, model) in caps.available_models.iter().enumerate() {
                            models_table.set(model_idx + 1, model.clone())?;
                        }
                        caps_table.set("available_models", models_table)?;
                    }

                    provider_table.set("capabilities", caps_table)?;
                }

                result.set(idx + 1, provider_table)?;
            }

            Ok(result)
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Provider.list function: {e}"),
            source: None,
        })?;

    provider_table
        .set("list", list_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Provider.list: {e}"),
            source: None,
        })?;

    // Provider.get(name) - Get specific provider information
    let providers_get = providers.clone();
    let get_fn = lua
        .create_function(move |lua, name: String| -> LuaResult<Value> {
            let provider_info = futures::executor::block_on(providers_get.get_provider_info(&name));

            match provider_info {
                Some(info) => {
                    let provider_table = lua.create_table()?;
                    provider_table.set("name", info.name)?;
                    provider_table.set("enabled", info.enabled)?;

                    if let Some(caps) = info.capabilities {
                        let caps_table = lua.create_table()?;
                        caps_table.set("supports_streaming", caps.supports_streaming)?;
                        caps_table.set("supports_multimodal", caps.supports_multimodal)?;
                        caps_table.set("max_context_tokens", caps.max_context_tokens)?;
                        provider_table.set("capabilities", caps_table)?;
                    }

                    Ok(Value::Table(provider_table))
                }
                None => Ok(Value::Nil),
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Provider.get function: {e}"),
            source: None,
        })?;

    provider_table
        .set("get", get_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Provider.get: {e}"),
            source: None,
        })?;

    // Provider.getCapabilities(name) - Get provider capabilities
    let providers_caps = providers.clone();
    let get_caps_fn = lua
        .create_function(move |lua, name: String| -> LuaResult<Value> {
            let provider_info =
                futures::executor::block_on(providers_caps.get_provider_info(&name));

            match provider_info.and_then(|info| info.capabilities) {
                Some(caps) => {
                    let caps_table = lua.create_table()?;
                    caps_table.set("supports_streaming", caps.supports_streaming)?;
                    caps_table.set("supports_multimodal", caps.supports_multimodal)?;
                    caps_table.set("max_context_tokens", caps.max_context_tokens)?;

                    if !caps.available_models.is_empty() {
                        let models_table = lua.create_table()?;
                        for (idx, model) in caps.available_models.iter().enumerate() {
                            models_table.set(idx + 1, model.clone())?;
                        }
                        caps_table.set("available_models", models_table)?;
                    }

                    Ok(Value::Table(caps_table))
                }
                None => Ok(Value::Nil),
            }
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Provider.getCapabilities function: {e}"),
            source: None,
        })?;

    provider_table
        .set("getCapabilities", get_caps_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Provider.getCapabilities: {e}"),
            source: None,
        })?;

    // Provider.isAvailable(name) - Check if provider is configured and available
    let providers_avail = providers.clone();
    let is_available_fn = lua
        .create_function(move |_lua, name: String| -> LuaResult<bool> {
            let provider_info =
                futures::executor::block_on(providers_avail.get_provider_info(&name));
            Ok(provider_info.is_some_and(|info| info.enabled))
        })
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to create Provider.isAvailable function: {e}"),
            source: None,
        })?;

    provider_table
        .set("isAvailable", is_available_fn)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Provider.isAvailable: {e}"),
            source: None,
        })?;

    // Set the Provider table as a global
    lua.globals()
        .set("Provider", provider_table)
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to set Provider global: {e}"),
            source: None,
        })?;

    Ok(())
}
