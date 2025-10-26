//! Provider Config Lookup Tests
//!
//! Tests for Task 13.5.7h: Fix Agent Provider Config Lookup (Bridge/Kernel Gap)
//!
//! Verifies that `ProviderManager.create_agent_from_spec()` correctly implements
//! the 3-tier lookup strategy:
//! 1. Exact match: Reuse cached instance (provider_type + model match)
//! 2. Provider type match: Reuse initialized provider with matching type
//! 3. Ephemeral config: Create new instance when no match found

use async_trait::async_trait;
use llmspell_core::{
    error::LLMSpellError,
    types::{AgentInput, AgentOutput},
};
use llmspell_providers::{
    abstraction::{ProviderCapabilities, ProviderConfig, ProviderInstance, ProviderManager},
    ModelSpecifier,
};
use std::sync::Arc;

/// Mock provider for testing
struct MockProvider {
    name: String,
    model: String,
    capabilities: ProviderCapabilities,
}

#[async_trait]
impl ProviderInstance for MockProvider {
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    async fn complete(&self, _input: &AgentInput) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: "Mock response".to_string(),
            metadata: Default::default(),
            media: vec![],
            tool_calls: vec![],
            transfer_to: None,
        })
    }

    async fn validate(&self) -> Result<(), LLMSpellError> {
        Ok(())
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn model(&self) -> &str {
        &self.model
    }
}

/// Create a mock provider factory
fn create_mock_factory(
) -> impl Fn(ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError> + Send + Sync + 'static
{
    move |config: ProviderConfig| {
        let capabilities = ProviderCapabilities {
            supports_streaming: false,
            supports_multimodal: false,
            max_context_tokens: Some(4096),
            max_output_tokens: Some(2000),
            available_models: vec![config.model.clone()],
            custom_features: Default::default(),
        };

        Ok(Box::new(MockProvider {
            name: config.name.clone(),
            model: config.model.clone(),
            capabilities,
        }))
    }
}

/// Test Tier 1: Exact match - reuse cached instance
#[tokio::test]
async fn test_tier1_exact_match_cache_hit() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Initialize a provider with specific config
    let config = ProviderConfig::new_with_type("mock", "mock", "test-model");

    manager
        .init_provider(config)
        .await
        .expect("Should init provider");

    // Create agent from spec - should hit cache (exact match)
    let spec = ModelSpecifier::parse("mock/test-model").expect("Should parse spec");
    let provider1 = manager
        .create_agent_from_spec(spec.clone(), None, None)
        .await
        .expect("Should create agent");

    // Create another agent - should reuse cached instance
    let provider2 = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent");

    // Verify same instance (Arc pointer equality)
    assert!(
        Arc::ptr_eq(&provider1, &provider2),
        "Should reuse exact same cached instance"
    );
}

/// Test Tier 2: Provider type match - reuse initialized provider
#[tokio::test]
async fn test_tier2_provider_type_match() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Initialize a provider from config (format: name/provider_type/model)
    // Note: config.name must match registered provider factory name ("mock")
    let config = ProviderConfig::new_with_type("mock", "mock", "config-model");

    manager
        .init_provider(config)
        .await
        .expect("Should init provider");

    // Create agent with same provider_type and model
    let spec = ModelSpecifier::parse("mock/config-model").expect("Should parse spec");
    let provider = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent");

    // Verify we got the initialized provider (temperature from config)
    assert_eq!(provider.model(), "config-model", "Should have config model");
}

/// Test Tier 3: Ephemeral config - create new instance
#[tokio::test]
async fn test_tier3_ephemeral_config() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Create agent without any initialized provider (ephemeral)
    let spec = ModelSpecifier::parse("mock/ephemeral-model").expect("Should parse spec");
    let provider = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent");

    // Verify ephemeral provider was created
    assert_eq!(
        provider.model(),
        "ephemeral-model",
        "Should have ephemeral model"
    );
}

/// Test provider type match with different model (future enhancement)
#[tokio::test]
async fn test_provider_type_match_different_model() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Initialize a provider with model A
    let config = ProviderConfig::new_with_type("mock", "mock", "model-a");

    manager
        .init_provider(config)
        .await
        .expect("Should init provider");

    // Request model B with same provider type
    // Current implementation: Falls back to ephemeral config (model mismatch)
    // Future enhancement: Clone config and override model
    let spec = ModelSpecifier::parse("mock/model-b").expect("Should parse spec");
    let provider = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent");

    // Verify we got model B (ephemeral config for now)
    assert_eq!(provider.model(), "model-b", "Should have requested model");
}

/// Test exact match takes precedence over provider type match
#[tokio::test]
async fn test_exact_match_precedence() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Initialize provider A (from config)
    let config_a = ProviderConfig::new_with_type("mock", "mock", "shared-model");

    manager
        .init_provider(config_a)
        .await
        .expect("Should init provider A");

    // Create an ephemeral instance (this will be cached with instance_name format)
    let spec = ModelSpecifier::parse("mock/shared-model").expect("Should parse spec");
    let provider_first = manager
        .create_agent_from_spec(spec.clone(), None, None)
        .await
        .expect("Should create first agent");

    // Second request should hit exact cache, not provider type match
    let provider_second = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create second agent");

    // Verify same instance (cache hit)
    assert!(
        Arc::ptr_eq(&provider_first, &provider_second),
        "Exact match should take precedence over provider type match"
    );
}

/// Test multiple providers with same type
#[tokio::test]
async fn test_multiple_providers_same_type() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Initialize two providers with same provider_type but different models
    let config1 = ProviderConfig::new_with_type("mock", "mock", "model-1");
    let config2 = ProviderConfig::new_with_type("mock", "mock", "model-2");

    manager
        .init_provider(config1)
        .await
        .expect("Should init provider 1");
    manager
        .init_provider(config2)
        .await
        .expect("Should init provider 2");

    // Request with provider type - should find FIRST matching provider
    let spec = ModelSpecifier::parse("mock/model-1").expect("Should parse spec");
    let provider = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent");

    // Verify we got the first provider (provider-1)
    assert_eq!(
        provider.model(),
        "model-1",
        "Should match first provider's model"
    );
}

/// Test backward compatibility - agents without config providers still work
#[tokio::test]
async fn test_backward_compat_no_config_provider() {
    let manager = ProviderManager::new();

    // Register mock provider
    manager
        .register_provider("mock", create_mock_factory())
        .await;

    // Create agent WITHOUT any initialized provider (pure ephemeral)
    // This is the backward-compatible path for agents that don't use config
    let spec = ModelSpecifier::parse("mock/ephemeral-model").expect("Should parse spec");
    let provider = manager
        .create_agent_from_spec(spec, None, None)
        .await
        .expect("Should create agent via ephemeral config");

    // Verify ephemeral provider works
    assert_eq!(provider.name(), "mock", "Should have mock provider");
    assert_eq!(
        provider.model(),
        "ephemeral-model",
        "Should have ephemeral model"
    );

    // Verify it can execute (validate succeeds)
    provider
        .validate()
        .await
        .expect("Should validate successfully");
}
