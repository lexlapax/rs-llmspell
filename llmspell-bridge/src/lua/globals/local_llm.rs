//! ABOUTME: Lua-specific LocalLLM global implementation
//! ABOUTME: Provides Lua bindings for local model management

use crate::globals::GlobalContext;
use crate::lua::sync_utils::block_on_async_lua;
use llmspell_providers::local::{DownloadStatus, HealthStatus, ModelSpec};
use llmspell_providers::ProviderManager as CoreProviderManager;
use mlua::{Lua, Table, Value};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// Inject LocalLLM global into Lua environment
///
/// This function creates the LocalLLM table with all local model management
/// methods and injects it into the Lua global namespace.
///
/// # Errors
///
/// Returns an error if:
/// - Lua table creation fails
/// - Function binding fails
/// - Provider access fails
///
/// # Examples
///
/// ```lua
/// -- Check backend status
/// local status = LocalLLM.status()
/// print("Ollama:", status.ollama.running)
/// print("Models:", status.ollama.models)
///
/// -- List local models
/// local models = LocalLLM.list()
/// for _, model in ipairs(models) do
///     print(model.id, model.backend, model.size_bytes)
/// end
///
/// -- Download a model
/// local spec = "llama3.1:8b@ollama"
/// local progress = LocalLLM.pull(spec)
/// print("Download:", progress.percent_complete .. "%")
///
/// -- Get model info
/// local info = LocalLLM.info("llama3.1:8b")
/// print("Format:", info.format)
/// print("Params:", info.parameter_count)
/// ```
#[instrument(
    level = "info",
    skip(lua, _context, provider_manager),
    fields(global_name = "LocalLLM")
)]
pub fn inject_local_llm_global(
    lua: &Lua,
    _context: &GlobalContext,
    provider_manager: Arc<CoreProviderManager>,
) -> mlua::Result<()> {
    info!("Injecting LocalLLM global API");

    let local_llm_table = lua.create_table()?;

    // Register methods
    register_status_method(lua, &local_llm_table, provider_manager.clone())?;
    register_list_method(lua, &local_llm_table, provider_manager.clone())?;
    register_pull_method(lua, &local_llm_table, provider_manager.clone())?;
    register_info_method(lua, &local_llm_table, provider_manager)?;

    // Inject into Lua globals
    lua.globals().set("LocalLLM", local_llm_table)?;

    debug!("LocalLLM global registered successfully");
    Ok(())
}

/// Register the status() method
///
/// Returns health status for all local backends (Ollama, Candle)
fn register_status_method(
    lua: &Lua,
    table: &Table,
    provider_manager: Arc<CoreProviderManager>,
) -> mlua::Result<()> {
    let status_fn = lua.create_async_function(move |lua, ()| {
        let pm = provider_manager.clone();
        async move {
            info!("LocalLLM.status() called from Lua");

            // Use shared sync utility to execute async code
            let result = block_on_async_lua(
                "local_llm_status",
                async move {
                    let result_table = lua.create_table()?;

                    // Check Ollama backend
                    debug!("Checking Ollama status");
                    let ollama_table = lua.create_table()?;

                    match pm.get_provider_for_backend("ollama").await {
                        Ok(Some(provider)) => {
                            if let Some(local_provider) = provider.as_local() {
                                match local_provider.health_check().await {
                                    Ok(HealthStatus::Healthy {
                                        available_models,
                                        version,
                                    }) => {
                                        ollama_table.set("running", true)?;
                                        ollama_table.set("models", available_models)?;
                                        if let Some(v) = version {
                                            ollama_table.set("version", v)?;
                                        }
                                    }
                                    Ok(HealthStatus::Unhealthy { reason }) => {
                                        ollama_table.set("running", false)?;
                                        ollama_table.set("error", reason)?;
                                        ollama_table.set("models", 0)?;
                                    }
                                    Ok(HealthStatus::Unknown) => {
                                        ollama_table.set("running", false)?;
                                        ollama_table.set("error", "Status unknown")?;
                                        ollama_table.set("models", 0)?;
                                    }
                                    Err(e) => {
                                        warn!("Ollama health check failed: {}", e);
                                        ollama_table.set("running", false)?;
                                        ollama_table.set("error", e.to_string())?;
                                        ollama_table.set("models", 0)?;
                                    }
                                }
                            } else {
                                ollama_table.set("running", false)?;
                                ollama_table.set("error", "Provider not a LocalProviderInstance")?;
                                ollama_table.set("models", 0)?;
                            }
                        }
                        Ok(None) => {
                            debug!("No Ollama provider initialized");
                            ollama_table.set("running", false)?;
                            ollama_table.set("error", "Not configured")?;
                            ollama_table.set("models", 0)?;
                        }
                        Err(e) => {
                            warn!("Failed to get Ollama provider: {}", e);
                            ollama_table.set("running", false)?;
                            ollama_table.set("error", e.to_string())?;
                            ollama_table.set("models", 0)?;
                        }
                    }
                    result_table.set("ollama", ollama_table)?;

                    // Check Candle backend
                    debug!("Checking Candle status");
                    let candle_table = lua.create_table()?;

                    match pm.get_provider_for_backend("candle").await {
                        Ok(Some(provider)) => {
                            if let Some(local_provider) = provider.as_local() {
                                match local_provider.health_check().await {
                                    Ok(HealthStatus::Healthy {
                                        available_models,
                                        version,
                                    }) => {
                                        candle_table.set("ready", true)?;
                                        candle_table.set("models", available_models)?;
                                        if let Some(v) = version {
                                            candle_table.set("version", v)?;
                                        }
                                    }
                                    Ok(HealthStatus::Unhealthy { reason }) => {
                                        candle_table.set("ready", false)?;
                                        candle_table.set("error", reason)?;
                                        candle_table.set("models", 0)?;
                                    }
                                    Ok(HealthStatus::Unknown) => {
                                        candle_table.set("ready", false)?;
                                        candle_table.set("error", "Status unknown")?;
                                        candle_table.set("models", 0)?;
                                    }
                                    Err(e) => {
                                        warn!("Candle health check failed: {}", e);
                                        candle_table.set("ready", false)?;
                                        candle_table.set("error", e.to_string())?;
                                        candle_table.set("models", 0)?;
                                    }
                                }
                            } else {
                                candle_table.set("ready", false)?;
                                candle_table.set("error", "Provider not a LocalProviderInstance")?;
                                candle_table.set("models", 0)?;
                            }
                        }
                        Ok(None) => {
                            debug!("No Candle provider initialized (expected in Phase 11)");
                            candle_table.set("ready", false)?;
                            candle_table.set("error", "Not configured")?;
                            candle_table.set("models", 0)?;
                        }
                        Err(e) => {
                            warn!("Failed to get Candle provider: {}", e);
                            candle_table.set("ready", false)?;
                            candle_table.set("error", e.to_string())?;
                            candle_table.set("models", 0)?;
                        }
                    }
                    result_table.set("candle", candle_table)?;

                    debug!("Status check complete");
                    Ok(Value::Table(result_table))
                },
                None,
            )?;

            Ok(result)
        }
    })?;

    table.set("status", status_fn)?;
    Ok(())
}

/// Register the list() method
///
/// Returns array of all local models
fn register_list_method(
    lua: &Lua,
    table: &Table,
    provider_manager: Arc<CoreProviderManager>,
) -> mlua::Result<()> {
    let list_fn = lua.create_async_function(move |lua, backend_opt: Option<String>| {
        let pm = provider_manager.clone();
        async move {
            info!("LocalLLM.list({:?}) called from Lua", backend_opt);

            // Use shared sync utility to execute async code
            let result = block_on_async_lua(
                "local_llm_list",
                async move {
                    let result_table = lua.create_table()?;
                    let mut index = 1;

                    // Determine which backends to query
                    let backends = match backend_opt.as_deref() {
                        Some("ollama") => vec!["ollama"],
                        Some("candle") => vec!["candle"],
                        Some("all") | None => vec!["ollama", "candle"],
                        Some(other) => vec![other],
                    };

                    // Query each backend
                    for backend in backends {
                        debug!("Listing models from {} backend", backend);

                        match pm.get_provider_for_backend(backend).await {
                            Ok(Some(provider)) => {
                                if let Some(local_provider) = provider.as_local() {
                                    match local_provider.list_local_models().await {
                                        Ok(models) => {
                                            for model in models {
                                                let model_table = lua.create_table()?;
                                                model_table.set("id", model.id)?;
                                                model_table.set("backend", model.backend)?;
                                                #[allow(clippy::cast_precision_loss)] // Acceptable for file sizes (>4PB for precision loss)
                                                model_table.set("size_bytes", model.size_bytes as f64)?;

                                                if let Some(quant) = model.quantization {
                                                    model_table.set("quantization", quant)?;
                                                }

                                                if let Some(modified) = model.modified_at {
                                                    if let Ok(timestamp) = modified
                                                        .duration_since(std::time::UNIX_EPOCH)
                                                    {
                                                        model_table.set(
                                                            "modified_at",
                                                            timestamp.as_secs() as f64,
                                                        )?;
                                                    }
                                                }

                                                result_table.set(index, model_table)?;
                                                index += 1;
                                            }
                                        }
                                        Err(e) => {
                                            warn!("Failed to list {} models: {}", backend, e);
                                        }
                                    }
                                }
                            }
                            Ok(None) => {
                                debug!("{} backend not configured", backend);
                            }
                            Err(e) => {
                                warn!("Failed to get {} provider: {}", backend, e);
                            }
                        }
                    }

                    debug!("Model list complete: {} models", index - 1);
                    Ok(Value::Table(result_table))
                },
                None,
            )?;

            Ok(result)
        }
    })?;

    table.set("list", list_fn)?;
    Ok(())
}

/// Register the pull() method
///
/// Downloads a model from the backend library
fn register_pull_method(
    lua: &Lua,
    table: &Table,
    provider_manager: Arc<CoreProviderManager>,
) -> mlua::Result<()> {
    let pull_fn = lua.create_async_function(move |lua, spec_str: String| {
        let pm = provider_manager.clone();
        async move {
            info!("LocalLLM.pull('{}') called from Lua", spec_str);

            // Use shared sync utility to execute async code
            let result = block_on_async_lua(
                "local_llm_pull",
                async move {
                    // Parse model specification
                    let spec = ModelSpec::parse(&spec_str).map_err(|e| {
                        mlua::Error::RuntimeError(format!("Invalid model spec '{}': {}", spec_str, e))
                    })?;

                    // Determine backend (from spec or default to ollama)
                    let backend = spec.backend.as_deref().unwrap_or("ollama");
                    debug!("Pulling model from {} backend", backend);

                    // Get the provider for the backend
                    match pm.get_provider_for_backend(backend).await {
                        Ok(Some(provider)) => {
                            if let Some(local_provider) = provider.as_local() {
                                // Pull the model
                                match local_provider.pull_model(&spec).await {
                                    Ok(progress) => {
                                        let result_table = lua.create_table()?;
                                        result_table.set("model_id", progress.model_id)?;
                                        result_table
                                            .set("percent_complete", progress.percent_complete as f64)?;
                                        result_table.set(
                                            "bytes_downloaded",
                                            progress.bytes_downloaded as f64,
                                        )?;

                                        if let Some(total) = progress.bytes_total {
                                            result_table.set("bytes_total", total as f64)?;
                                        }

                                        // Convert status
                                        let status_str = match progress.status {
                                            DownloadStatus::Starting => "starting",
                                            DownloadStatus::Downloading => "downloading",
                                            DownloadStatus::Verifying => "verifying",
                                            DownloadStatus::Complete => "complete",
                                            DownloadStatus::Failed { .. } => "failed",
                                        };
                                        result_table.set("status", status_str)?;

                                        // Add error if failed
                                        if let DownloadStatus::Failed { error } = progress.status {
                                            result_table.set("error", error)?;
                                        }

                                        debug!("Model pull: {}%", progress.percent_complete);
                                        Ok(Value::Table(result_table))
                                    }
                                    Err(e) => Err(mlua::Error::RuntimeError(format!(
                                        "Failed to pull model: {}",
                                        e
                                    ))),
                                }
                            } else {
                                Err(mlua::Error::RuntimeError(format!(
                                    "Provider '{}' does not support model management",
                                    backend
                                )))
                            }
                        }
                        Ok(None) => Err(mlua::Error::RuntimeError(format!(
                            "Backend '{backend}' not configured"
                        ))),
                        Err(e) => Err(mlua::Error::RuntimeError(format!(
                            "Failed to get provider: {e}"
                        ))),
                    }
                },
                None,
            )?;

            Ok(result)
        }
    })?;

    table.set("pull", pull_fn)?;
    Ok(())
}

/// Register the `info()` method
///
/// Returns detailed information about a specific model
fn register_info_method(
    lua: &Lua,
    table: &Table,
    provider_manager: Arc<CoreProviderManager>,
) -> mlua::Result<()> {
    let info_fn = lua.create_async_function(move |lua, model_id: String| {
        let pm = provider_manager.clone();
        async move {
            info!("LocalLLM.info('{}') called from Lua", model_id);

            // Use shared sync utility to execute async code
            let result = block_on_async_lua(
                "local_llm_info",
                async move {
                    // Try both backends to find the model
                    for backend in &["ollama", "candle"] {
                        match pm.get_provider_for_backend(backend).await {
                            Ok(Some(provider)) => {
                                if let Some(local_provider) = provider.as_local() {
                                    match local_provider.model_info(&model_id).await {
                                        Ok(info) => {
                                            let result_table = lua.create_table()?;
                                            result_table.set("id", info.id)?;
                                            result_table.set("backend", info.backend)?;
                                            #[allow(clippy::cast_precision_loss)] // Acceptable for file sizes (>4PB for precision loss)
                                            result_table.set("size_bytes", info.size_bytes as f64)?;
                                            result_table.set("format", info.format)?;
                                            result_table.set("loaded", info.loaded)?;

                                            if let Some(param_count) = info.parameter_count {
                                                result_table.set("parameter_count", param_count)?;
                                            }

                                            if let Some(quant) = info.quantization {
                                                result_table.set("quantization", quant)?;
                                            }

                                            debug!("Model info found in {}", backend);
                                            return Ok(Value::Table(result_table));
                                        }
                                        Err(_) => {
                                            // Model not found in this backend, try next
                                            continue;
                                        }
                                    }
                                }
                            }
                            _ => continue,
                        }
                    }

                    // Model not found in any backend
                    Err(mlua::Error::RuntimeError(format!(
                        "Model '{model_id}' not found in any backend"
                    )))
                },
                None,
            )?;

            Ok(result)
        }
    })?;

    table.set("info", info_fn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::globals::types::GlobalContext;
    use crate::ComponentRegistry;
    use llmspell_providers::ProviderManager as CoreProviderManager;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_local_llm_injection() {
        let lua = Lua::new();
        let registry = Arc::new(ComponentRegistry::new());
        let core_provider_manager = Arc::new(CoreProviderManager::new());

        // Create minimal config for testing
        let config = crate::providers::ProviderManagerConfig {
            providers: std::collections::HashMap::new(),
        };
        let bridge_provider_manager = Arc::new(
            crate::providers::ProviderManager::new(config)
                .await
                .expect("Failed to create provider manager")
        );
        let context = GlobalContext::new(registry, bridge_provider_manager);

        inject_local_llm_global(&lua, &context, core_provider_manager).expect("Injection should succeed");

        // Verify LocalLLM table exists
        let result: mlua::Result<Table> = lua.globals().get("LocalLLM");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_status_method_exists() {
        let lua = Lua::new();
        let registry = Arc::new(ComponentRegistry::new());
        let core_provider_manager = Arc::new(CoreProviderManager::new());

        let config = crate::providers::ProviderManagerConfig {
            providers: std::collections::HashMap::new(),
        };
        let bridge_provider_manager = Arc::new(
            crate::providers::ProviderManager::new(config)
                .await
                .expect("Failed to create provider manager")
        );
        let context = GlobalContext::new(registry, bridge_provider_manager);

        inject_local_llm_global(&lua, &context, core_provider_manager).expect("Injection should succeed");

        // Verify status() method exists
        let local_llm: Table = lua.globals().get("LocalLLM").unwrap();
        let status_fn: mlua::Result<mlua::Function> = local_llm.get("status");
        assert!(status_fn.is_ok());
    }

    #[tokio::test]
    async fn test_list_method_exists() {
        let lua = Lua::new();
        let registry = Arc::new(ComponentRegistry::new());
        let core_provider_manager = Arc::new(CoreProviderManager::new());

        let config = crate::providers::ProviderManagerConfig {
            providers: std::collections::HashMap::new(),
        };
        let bridge_provider_manager = Arc::new(
            crate::providers::ProviderManager::new(config)
                .await
                .expect("Failed to create provider manager")
        );
        let context = GlobalContext::new(registry, bridge_provider_manager);

        inject_local_llm_global(&lua, &context, core_provider_manager).expect("Injection should succeed");

        // Verify list() method exists
        let local_llm: Table = lua.globals().get("LocalLLM").unwrap();
        let list_fn: mlua::Result<mlua::Function> = local_llm.get("list");
        assert!(list_fn.is_ok());
    }
}
