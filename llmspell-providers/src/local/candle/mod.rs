//! Candle provider for embedded GGUF inference
//!
//! This module provides local LLM inference using the Candle framework.
//! Models are loaded as GGUF files and run directly in Rust with no external dependencies.

mod gguf_loader;
mod hf_downloader;
mod model_wrapper;
mod provider;
mod sampling;
mod tokenizer_loader;

pub use gguf_loader::{GGUFLoader, GGUFMetadata};
pub use hf_downloader::{HFDownloader, HFModelRepo};
pub use model_wrapper::ModelWrapper;
pub use provider::CandleProvider;
pub use sampling::{sample_token, SamplingConfig};
pub use tokenizer_loader::TokenizerLoader;

use crate::abstraction::ProviderConfig;
use crate::abstraction::ProviderInstance;
use llmspell_core::error::LLMSpellError;
use llmspell_utils::file_utils::expand_path;
use tracing::warn;

/// Factory function to create a Candle provider instance
///
/// This function creates a CandleProvider that runs GGUF models directly in Rust.
/// It requires:
/// - Model directory path (defaults to ~/.llmspell/models/candle)
/// - Device configuration (auto/cpu/cuda/metal)
/// - Default model for inference
///
/// # Arguments
/// * `config` - Provider configuration with model and optional settings
///
/// # Returns
/// * `Ok(Box<dyn ProviderInstance>)` - Initialized Candle provider
/// * `Err(LLMSpellError)` - Configuration or initialization error
///
/// # Examples
///
/// ```no_run
/// # use llmspell_providers::abstraction::ProviderConfig;
/// # use llmspell_providers::local::candle::create_candle_provider;
/// let mut config = ProviderConfig::new_with_type("candle", "local", "llama3.1:8b");
/// let provider = create_candle_provider(config).unwrap();
/// ```
pub fn create_candle_provider(
    config: ProviderConfig,
) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
    // Extract Candle-specific configuration from options HashMap
    let device = config
        .custom_config
        .get("device")
        .and_then(|v| v.as_str())
        .unwrap_or("auto")
        .to_string();

    let model_directory = config
        .custom_config
        .get("model_directory")
        .and_then(|v| v.as_str())
        .and_then(|path_str| {
            expand_path(path_str)
                .map_err(|e| {
                    warn!("Failed to expand path '{}': {}", path_str, e);
                    e
                })
                .ok()
        });

    let provider =
        CandleProvider::new(config.model.clone(), model_directory, device).map_err(|e| {
            LLMSpellError::Component {
                message: format!("Failed to create Candle provider: {}", e),
                source: None,
            }
        })?;

    Ok(Box::new(provider))
}
