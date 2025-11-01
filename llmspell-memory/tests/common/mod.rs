//! Common test utilities for llmspell-memory
//!
//! Provides test helpers for provider configuration, mock objects, and fixtures.

use llmspell_config::ProviderConfig;

/// Create a standard test provider configuration
///
/// Returns a ProviderConfig suitable for testing with:
/// - provider_type: "ollama"
/// - default_model: "ollama/llama3.2:3b"
/// - temperature: 0.0 (deterministic)
/// - max_tokens: 2000
/// - timeout: 30s
/// - max_retries: 3
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_memory_tests::common::test_provider_config;
/// use llmspell_memory::consolidation::LLMConsolidationConfig;
///
/// let provider = test_provider_config();
/// let config = LLMConsolidationConfig::from_provider(&provider).unwrap();
/// assert_eq!(config.model, "ollama/llama3.2:3b");
/// ```
pub fn test_provider_config() -> ProviderConfig {
    ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("ollama/llama3.2:3b")
        .temperature(0.0)
        .max_tokens(2000)
        .timeout_seconds(30)
        .build()
}

/// Create a test provider with custom model
///
/// Useful for testing fallback behavior or multi-model scenarios.
pub fn test_provider_config_with_model(model: &str) -> ProviderConfig {
    ProviderConfig::builder()
        .provider_type("ollama")
        .default_model(model)
        .temperature(0.0)
        .max_tokens(2000)
        .timeout_seconds(30)
        .build()
}
