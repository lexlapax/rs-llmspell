//! ABOUTME: `LocalLLM` global object for local model management
//! ABOUTME: Provides access to Ollama and Candle backends

#[cfg(any(feature = "lua", feature = "javascript"))]
use super::types::GlobalContext;
use super::types::{GlobalMetadata, GlobalObject};
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::Result;
use llmspell_providers::ProviderManager as CoreProviderManager;
use std::sync::Arc;

/// `LocalLLM` global object for script engines
///
/// This global provides access to local LLM model management operations
/// including health checks, model listing, downloading, and information queries.
/// It supports both Ollama and Candle backends.
///
/// # Available Operations
///
/// - **`status()`**: Check backend availability and health
/// - **`list()`**: List installed local models
/// - **`pull()`**: Download models from backend libraries
/// - **`info()`**: Get detailed model information
///
/// # Backend Support
///
/// - **Ollama**: GGUF models via Ollama backend (<http://localhost:11434>)
/// - **Candle**: Native Rust inference (future)
///
/// # Examples
///
/// ```lua
/// -- Check backend status
/// local status = LocalLLM.status()
/// print("Ollama running:", status.ollama.running)
///
/// -- List local models
/// local models = LocalLLM.list()
/// for _, model in ipairs(models) do
///     print(model.id, model.size_bytes)
/// end
///
/// -- Download a model
/// local progress = LocalLLM.pull("llama3.1:8b@ollama")
/// print("Downloaded:", progress.percent_complete .. "%")
/// ```
pub struct LocalLLMGlobal {
    provider_manager: Arc<CoreProviderManager>,
}

impl LocalLLMGlobal {
    /// Create a new `LocalLLM` global with the given provider manager
    ///
    /// # Arguments
    /// * `provider_manager` - The core provider manager for accessing local providers
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use llmspell_bridge::globals::local_llm_global::LocalLLMGlobal;
    /// # use llmspell_providers::ProviderManager;
    /// # use std::sync::Arc;
    /// let manager = Arc::new(ProviderManager::new());
    /// let global = LocalLLMGlobal::new(manager);
    /// ```
    #[must_use]
    pub const fn new(provider_manager: Arc<CoreProviderManager>) -> Self {
        Self { provider_manager }
    }

    /// Get the provider manager
    ///
    /// This provides access to the underlying provider manager for
    /// implementing language-specific bindings.
    #[must_use]
    pub const fn provider_manager(&self) -> &Arc<CoreProviderManager> {
        &self.provider_manager
    }
}

impl GlobalObject for LocalLLMGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "LocalLLM".to_string(),
            description: "Local LLM model management (Ollama, Candle)".to_string(),
            dependencies: vec![],
            required: false, // Optional - only if local providers configured
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<()> {
        crate::lua::globals::local_llm::inject_local_llm_global(
            lua,
            context,
            self.provider_manager.clone(),
        )
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Failed to inject LocalLLM global: {e}"),
            source: None,
        })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // TODO: Implement JavaScript bindings for LocalLLM in Phase 12+
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_llm_global_creation() {
        let manager = Arc::new(CoreProviderManager::new());
        let global = LocalLLMGlobal::new(manager);

        let metadata = global.metadata();
        assert_eq!(metadata.name, "LocalLLM");
        assert!(!metadata.required);
    }

    #[test]
    fn test_local_llm_global_metadata() {
        let manager = Arc::new(CoreProviderManager::new());
        let global = LocalLLMGlobal::new(manager);

        let metadata = global.metadata();
        assert_eq!(metadata.version, "1.0.0");
        assert!(metadata.description.contains("Ollama"));
        assert!(metadata.description.contains("Candle"));
    }
}
