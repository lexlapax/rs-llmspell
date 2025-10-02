//! Candle provider for embedded GGUF inference
//!
//! This module provides local LLM inference using the Candle framework.
//! Models are loaded as GGUF files and run directly in Rust with no external dependencies.

mod provider;

pub use provider::CandleProvider;

use crate::abstraction::ProviderConfig;
use crate::abstraction::ProviderInstance;
use llmspell_core::error::LLMSpellError;

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
        .map(|s| std::path::PathBuf::from(s));

    let provider = CandleProvider::new(
        config.model.clone(),
        model_directory,
        device,
    ).map_err(|e| LLMSpellError::Component {
        message: format!("Failed to create Candle provider: {}", e),
        source: None,
    })?;

    Ok(Box::new(provider))
}
