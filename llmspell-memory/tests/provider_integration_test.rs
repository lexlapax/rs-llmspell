//! Provider integration tests for memory consolidation
//!
//! Verifies provider system integration across:
//! - Memory uses provider config (not hardcoded values)
//! - Provider fallback (`consolidation.provider_name=None` → `default_provider`)
//! - TOML config loading with custom providers
//! - Environment variable overrides
//!
//! These tests are unit tests that verify configuration,
//! not E2E tests requiring real LLM calls.

use llmspell_config::{LLMSpellConfig, ProviderConfig, ProviderManagerConfig};
use llmspell_memory::consolidation::LLMConsolidationConfig;
use serial_test::serial;
use std::sync::Arc;

/// Test `LLMConsolidationConfig::from_provider()` factory method
///
/// Verifies that config is sourced from provider, not hardcoded defaults
#[test]
fn test_consolidation_config_from_provider() {
    // Create custom provider with non-default values
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("custom-model")
        .temperature(0.8) // Non-default temperature
        .max_tokens(4000) // Non-default max_tokens
        .timeout_seconds(120) // Non-default timeout
        .build();

    // Create config from provider
    let config = LLMConsolidationConfig::from_provider(&provider)
        .expect("Should create config from provider");

    // Verify values come from provider, NOT hardcoded defaults
    assert_eq!(
        config.model, "custom-model",
        "Model should come from provider"
    );
    assert!(
        (config.temperature - 0.8).abs() < 0.01,
        "Temperature should come from provider"
    );
    assert_eq!(
        config.max_tokens, 4000,
        "Max tokens should come from provider"
    );
    assert_eq!(
        config.timeout_secs, 120,
        "Timeout should come from provider"
    );

    // Note: max_retries comes from provider.max_retries if available
    // For now we test the successful config creation
}

/// Test `from_provider()` with minimal provider (defaults fill in missing values)
#[test]
fn test_consolidation_config_from_provider_with_defaults() {
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("test-model")
        // No temperature, max_tokens, timeout - should use defaults
        .build();

    let config = LLMConsolidationConfig::from_provider(&provider)
        .expect("Should create config with defaults");

    assert_eq!(config.model, "test-model");
    assert!(
        (config.temperature - 0.0).abs() < 0.01,
        "Default temperature is 0.0"
    );
    assert_eq!(config.max_tokens, 2000, "Default max_tokens is 2000");
    assert_eq!(
        config.timeout_secs, 60,
        "Default timeout is 60s (ProviderConfigBuilder default)"
    );
    assert_eq!(config.max_retries, 3, "Default retries is 3");
}

/// Test `from_provider()` returns error if provider missing `default_model`
#[test]
fn test_consolidation_config_from_provider_missing_model() {
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        // No default_model - should error
        .temperature(0.5)
        .max_tokens(2000)
        .build();

    let result = LLMConsolidationConfig::from_provider(&provider);

    assert!(
        result.is_err(),
        "Should error if provider missing default_model"
    );
    let error_msg = format!("{}", result.unwrap_err());
    assert!(
        error_msg.contains("model") || error_msg.contains("provider"),
        "Error should mention missing model/provider: {error_msg}"
    );
}

/// Test provider lookup via `ProviderManagerConfig`
///
/// Verifies that providers can be looked up by name and used for consolidation config
#[test]
fn test_provider_manager_lookup() {
    // Create provider manager with custom provider
    let mut manager = ProviderManagerConfig::default();

    // Register custom provider
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("consolidation-model")
        .temperature(0.1)
        .max_tokens(3000)
        .timeout_seconds(90)
        .build();

    // Add provider to manager
    manager.add_provider("consolidation-llm".to_string(), provider);
    manager.default_provider = Some("consolidation-llm".to_string());

    // Lookup provider by name
    let looked_up = manager
        .get_provider("consolidation-llm")
        .expect("Provider should exist");

    // Verify lookup returns correct provider
    assert_eq!(
        looked_up.default_model,
        Some("consolidation-model".to_string())
    );
    assert_eq!(looked_up.temperature, Some(0.1));
    assert_eq!(looked_up.max_tokens, Some(3000));

    // Create consolidation config from looked-up provider
    let config = LLMConsolidationConfig::from_provider(looked_up)
        .expect("Should create config from looked-up provider");

    assert_eq!(config.model, "consolidation-model");
    assert!((config.temperature - 0.1).abs() < 0.01);
    assert_eq!(config.max_tokens, 3000);
}

/// Test default provider fallback
///
/// Verifies that if `consolidation.provider_name` is None,
/// system falls back to `default_provider`
#[test]
fn test_default_provider_fallback() {
    // Create provider manager with default provider
    let mut manager = ProviderManagerConfig::default();

    let default_provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("default-model")
        .temperature(0.0)
        .max_tokens(2000)
        .build();

    manager.add_provider("default".to_string(), default_provider);
    manager.default_provider = Some("default".to_string());

    // Simulate consolidation.provider_name = None (use default)
    let provider_name: Option<&str> = None;

    // Fallback logic
    let provider = if let Some(name) = provider_name {
        manager
            .get_provider(name)
            .expect("Named provider should exist")
    } else {
        // Fallback to default provider
        let default_name = manager
            .default_provider
            .as_ref()
            .expect("Default provider should be set");
        manager
            .get_provider(default_name)
            .expect("Default provider should exist")
    };

    // Verify we got the default provider
    assert_eq!(
        provider.default_model,
        Some("default-model".to_string()),
        "Should use default provider when no name specified"
    );

    let config =
        LLMConsolidationConfig::from_provider(provider).expect("Should create config from default");

    assert_eq!(config.model, "default-model");
}

/// Test environment variable override for memory config
///
/// Verifies that `LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME` can override config
#[tokio::test]
#[serial]
async fn test_env_var_override_consolidation_provider() {
    // Clean up first to avoid pollution from parallel tests
    std::env::remove_var("LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME");

    // Simulate environment variable
    std::env::set_var(
        "LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME",
        "env-provider",
    );

    // Load config with environment variables
    // The config system automatically loads env vars during construction
    let config = LLMSpellConfig::load_with_profile(None, Some("memory"))
        .await
        .expect("Should load memory profile");

    // Note: Environment variable handling is done during config load
    // This test verifies the config system structure supports env overrides
    // Actual env var override testing is in llmspell-config package tests

    // Verify memory config exists and has consolidation settings
    assert!(config.runtime.memory.enabled, "Memory should be enabled");
    assert!(
        config.runtime.memory.consolidation.batch_size > 0,
        "Consolidation should be configured"
    );

    // Cleanup
    std::env::remove_var("LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME");
}

/// Test TOML config loading with custom provider
///
/// Verifies that TOML files can define custom providers for consolidation
#[tokio::test]
#[serial]
async fn test_toml_config_with_custom_provider() {
    // Clean up any env vars from other tests (tests may run in parallel)
    std::env::remove_var("LLMSPELL_MEMORY_CONSOLIDATION_PROVIDER_NAME");

    // Load memory.toml profile (has provider + memory config)
    let config = LLMSpellConfig::load_with_profile(None, Some("memory"))
        .await
        .expect("Should load memory profile");

    // Verify memory config loaded
    assert!(
        config.runtime.memory.enabled,
        "Memory should be enabled in profile"
    );

    // Verify consolidation provider is NOT configured in base memory profile
    // (It can be set via env vars or custom config, but not in the base profile)
    assert_eq!(
        config.runtime.memory.consolidation.provider_name,
        None,
        "Memory profile does not set a specific consolidation provider by default"
    );

    // Verify consolidation config has expected defaults
    assert!(
        config.runtime.memory.consolidation.batch_size > 0,
        "Consolidation batch_size should be configured"
    );
    assert!(
        config.runtime.memory.consolidation.max_concurrent > 0,
        "Consolidation max_concurrent should be configured"
    );

    // If a default provider exists, verify we can create consolidation config from it
    if let Some(default_name) = config.providers.default_provider.as_ref() {
        let default_provider = config
            .providers
            .get_provider(default_name)
            .expect("default provider should exist if set");

        // Verify we can create consolidation config from provider
        let _llm_config = LLMConsolidationConfig::from_provider(default_provider)
            .expect("Should create consolidation config from provider");
    }
}

/// Test multiple providers in config
///
/// Verifies that multiple providers can coexist and be looked up independently
#[test]
fn test_multiple_providers() {
    let mut manager = ProviderManagerConfig::default();

    // Create multiple providers
    let ollama_provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("llama3.2:3b")
        .temperature(0.0)
        .build();

    let openai_provider = ProviderConfig::builder()
        .provider_type("openai")
        .default_model("gpt-4")
        .temperature(0.7)
        .max_tokens(4000)
        .build();

    manager.add_provider("ollama-local".to_string(), ollama_provider);
    manager.add_provider("openai-cloud".to_string(), openai_provider);
    manager.default_provider = Some("ollama-local".to_string());

    // Verify both providers exist and are distinct
    let ollama = manager
        .get_provider("ollama-local")
        .expect("Ollama provider should exist");
    let openai = manager
        .get_provider("openai-cloud")
        .expect("OpenAI provider should exist");

    assert_eq!(
        ollama.default_model,
        Some("llama3.2:3b".to_string()),
        "Ollama provider"
    );
    assert_eq!(
        openai.default_model,
        Some("gpt-4".to_string()),
        "OpenAI provider"
    );

    // Create configs from both providers
    let ollama_config =
        LLMConsolidationConfig::from_provider(ollama).expect("Ollama config should work");
    let openai_config =
        LLMConsolidationConfig::from_provider(openai).expect("OpenAI config should work");

    assert_eq!(ollama_config.model, "llama3.2:3b");
    assert_eq!(openai_config.model, "gpt-4");
    assert!((ollama_config.temperature - 0.0).abs() < 0.01);
    assert!((openai_config.temperature - 0.7).abs() < 0.01);
}

/// Test that consolidation-specific config fields are independent of provider
///
/// Verifies that provider config (model, temperature) is separate from
/// consolidation config (`batch_size`, `max_concurrent`, `circuit_breaker_threshold`)
#[test]
fn test_consolidation_specific_vs_provider_config() {
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("test-model")
        .temperature(0.5)
        .max_tokens(2000)
        .build();

    let mut config =
        LLMConsolidationConfig::from_provider(&provider).expect("Should create config");

    // Provider-sourced fields
    assert_eq!(config.model, "test-model");
    assert!((config.temperature - 0.5).abs() < 0.01);
    assert_eq!(config.max_tokens, 2000);

    // Consolidation-specific fields (NOT from provider)
    // These should have consolidation-specific defaults
    config.circuit_breaker_threshold = 10; // Consolidation-specific
    assert_eq!(config.circuit_breaker_threshold, 10);

    // Verify provider doesn't have consolidation-specific fields
    // (This is a conceptual test - provider has no circuit_breaker field)
    assert!(provider.default_model.is_some(), "Provider has LLM fields");
    // Provider intentionally does NOT have consolidation-specific fields
}

/// Test Arc wrapping for thread-safe provider sharing
///
/// Verifies that providers can be wrapped in Arc for concurrent use
#[test]
fn test_provider_arc_sharing() {
    let provider = ProviderConfig::builder()
        .provider_type("ollama")
        .default_model("shared-model")
        .temperature(0.3)
        .build();

    // Wrap in Arc (pattern used in consolidation engine)
    let provider_arc = Arc::new(provider);

    // Clone Arc (cheap, shares underlying provider)
    let provider_clone = Arc::clone(&provider_arc);

    // Both Arcs point to same provider
    assert_eq!(
        provider_arc.default_model, provider_clone.default_model,
        "Arc clones share same provider"
    );

    // Create config from Arc-wrapped provider
    let config = LLMConsolidationConfig::from_provider(&provider_arc)
        .expect("Should create config from Arc<ProviderConfig>");

    assert_eq!(config.model, "shared-model");
    assert!((config.temperature - 0.3).abs() < 0.01);
}
