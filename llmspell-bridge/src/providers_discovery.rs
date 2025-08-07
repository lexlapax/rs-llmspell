//! ABOUTME: Provider discovery for LLM providers
//! ABOUTME: Provides discovery of available LLM providers and their capabilities

use crate::discovery::BridgeDiscovery;
use crate::providers::{ProviderInfo, ProviderManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Information about a provider type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderTypeInfo {
    /// Provider type name
    pub name: String,
    /// Description of the provider
    pub description: String,
    /// Provider category (cloud, local, etc.)
    pub category: String,
    /// Whether the provider supports streaming
    pub supports_streaming: bool,
    /// Whether the provider supports multimodal content
    pub supports_multimodal: bool,
    /// Common models available
    pub common_models: Vec<String>,
    /// Required configuration
    pub required_config: Vec<String>,
    /// Optional configuration
    pub optional_config: Vec<String>,
    /// Provider-specific features
    pub features: Vec<String>,
}

/// Provider discovery service
pub struct ProviderDiscovery {
    /// Provider manager reference
    provider_manager: Option<Arc<ProviderManager>>,
    /// Static provider information
    provider_types: HashMap<String, ProviderTypeInfo>,
}

impl ProviderDiscovery {
    /// Create a new provider discovery service
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        let mut provider_types = HashMap::new();

        // OpenAI
        provider_types.insert(
            "openai".to_string(),
            ProviderTypeInfo {
                name: "openai".to_string(),
                description: "OpenAI API for GPT models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: true,
                common_models: vec![
                    "gpt-4".to_string(),
                    "gpt-4-turbo".to_string(),
                    "gpt-3.5-turbo".to_string(),
                    "gpt-4o".to_string(),
                    "gpt-4o-mini".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                    "top_p".to_string(),
                ],
                features: vec![
                    "function_calling".to_string(),
                    "json_mode".to_string(),
                    "vision".to_string(),
                ],
            },
        );

        // Anthropic
        provider_types.insert(
            "anthropic".to_string(),
            ProviderTypeInfo {
                name: "anthropic".to_string(),
                description: "Anthropic API for Claude models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: true,
                common_models: vec![
                    "claude-3-opus-20240229".to_string(),
                    "claude-3-sonnet-20240229".to_string(),
                    "claude-3-haiku-20240307".to_string(),
                    "claude-2.1".to_string(),
                    "claude-instant-1.2".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "large_context".to_string(),
                    "vision".to_string(),
                    "artifacts".to_string(),
                ],
            },
        );

        // Cohere
        provider_types.insert(
            "cohere".to_string(),
            ProviderTypeInfo {
                name: "cohere".to_string(),
                description: "Cohere API for language models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "command".to_string(),
                    "command-light".to_string(),
                    "command-nightly".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec!["retrieval_augmented".to_string(), "search".to_string()],
            },
        );

        // Groq
        provider_types.insert(
            "groq".to_string(),
            ProviderTypeInfo {
                name: "groq".to_string(),
                description: "Groq high-speed inference API".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "llama-3.1-70b-versatile".to_string(),
                    "llama-3.1-8b-instant".to_string(),
                    "mixtral-8x7b-32768".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "ultra_fast_inference".to_string(),
                    "low_latency".to_string(),
                ],
            },
        );

        // Perplexity
        provider_types.insert(
            "perplexity".to_string(),
            ProviderTypeInfo {
                name: "perplexity".to_string(),
                description: "Perplexity AI with real-time web search".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "pplx-7b-online".to_string(),
                    "pplx-70b-online".to_string(),
                    "pplx-7b-chat".to_string(),
                    "pplx-70b-chat".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "web_search".to_string(),
                    "real_time_data".to_string(),
                    "citations".to_string(),
                ],
            },
        );

        // Together
        provider_types.insert(
            "together".to_string(),
            ProviderTypeInfo {
                name: "together".to_string(),
                description: "Together AI for open-source models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "meta-llama/Llama-3-70b-chat-hf".to_string(),
                    "mistralai/Mixtral-8x7B-Instruct-v0.1".to_string(),
                    "NousResearch/Nous-Hermes-2-Mixtral-8x7B-DPO".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "open_source_models".to_string(),
                    "custom_models".to_string(),
                ],
            },
        );

        // Gemini
        provider_types.insert(
            "gemini".to_string(),
            ProviderTypeInfo {
                name: "gemini".to_string(),
                description: "Google Gemini models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: true,
                common_models: vec![
                    "gemini-pro".to_string(),
                    "gemini-pro-vision".to_string(),
                    "gemini-ultra".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "multimodal".to_string(),
                    "long_context".to_string(),
                    "vision".to_string(),
                ],
            },
        );

        // Mistral
        provider_types.insert(
            "mistral".to_string(),
            ProviderTypeInfo {
                name: "mistral".to_string(),
                description: "Mistral AI models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "mistral-tiny".to_string(),
                    "mistral-small".to_string(),
                    "mistral-medium".to_string(),
                    "mistral-large".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec![
                    "efficient_inference".to_string(),
                    "multilingual".to_string(),
                ],
            },
        );

        // Replicate
        provider_types.insert(
            "replicate".to_string(),
            ProviderTypeInfo {
                name: "replicate".to_string(),
                description: "Replicate platform for various models".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: true,
                common_models: vec![
                    "meta/llama-2-70b-chat".to_string(),
                    "stability-ai/sdxl".to_string(),
                    "replicate/flan-t5-xl".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "webhook".to_string(),
                ],
                features: vec![
                    "model_zoo".to_string(),
                    "custom_deployments".to_string(),
                    "webhooks".to_string(),
                ],
            },
        );

        // Fireworks
        provider_types.insert(
            "fireworks".to_string(),
            ProviderTypeInfo {
                name: "fireworks".to_string(),
                description: "Fireworks AI for fast inference".to_string(),
                category: "cloud".to_string(),
                supports_streaming: true,
                supports_multimodal: false,
                common_models: vec![
                    "accounts/fireworks/models/llama-v3-70b-instruct".to_string(),
                    "accounts/fireworks/models/mixtral-8x7b-instruct".to_string(),
                ],
                required_config: vec!["api_key_env".to_string(), "model".to_string()],
                optional_config: vec![
                    "base_url".to_string(),
                    "max_tokens".to_string(),
                    "temperature".to_string(),
                ],
                features: vec!["fast_inference".to_string(), "serverless".to_string()],
            },
        );

        Self {
            provider_manager: None,
            provider_types,
        }
    }

    /// Create with a provider manager for dynamic discovery
    #[must_use]
    pub fn with_provider_manager(provider_manager: Arc<ProviderManager>) -> Self {
        let mut discovery = Self::new();
        discovery.provider_manager = Some(provider_manager);
        discovery
    }

    /// Get information about a specific provider type
    #[must_use]
    pub fn get_provider_info(&self, provider_type: &str) -> Option<ProviderTypeInfo> {
        self.provider_types.get(provider_type).cloned()
    }

    /// List all available provider types
    #[must_use]
    pub fn list_provider_types(&self) -> Vec<String> {
        self.provider_types.keys().cloned().collect()
    }

    /// Get providers by category
    #[must_use]
    pub fn get_providers_by_category(&self, category: &str) -> Vec<(String, ProviderTypeInfo)> {
        self.provider_types
            .iter()
            .filter(|(_, info)| info.category == category)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get providers that support streaming
    #[must_use]
    pub fn get_streaming_providers(&self) -> Vec<(String, ProviderTypeInfo)> {
        self.provider_types
            .iter()
            .filter(|(_, info)| info.supports_streaming)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get providers that support multimodal
    #[must_use]
    pub fn get_multimodal_providers(&self) -> Vec<(String, ProviderTypeInfo)> {
        self.provider_types
            .iter()
            .filter(|(_, info)| info.supports_multimodal)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }

    /// Get runtime provider information if provider manager is available
    pub async fn get_runtime_providers(&self) -> Vec<ProviderInfo> {
        if let Some(ref manager) = self.provider_manager {
            manager.list_providers().await
        } else {
            Vec::new()
        }
    }
}

impl Default for ProviderDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Implementation of unified BridgeDiscovery trait for ProviderDiscovery
#[async_trait::async_trait]
impl BridgeDiscovery<ProviderTypeInfo> for ProviderDiscovery {
    async fn discover_types(&self) -> Vec<(String, ProviderTypeInfo)> {
        self.provider_types
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<ProviderTypeInfo> {
        self.get_provider_info(type_name)
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.provider_types.contains_key(type_name)
    }

    async fn list_types(&self) -> Vec<String> {
        self.list_provider_types()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, ProviderTypeInfo)>
    where
        F: Fn(&str, &ProviderTypeInfo) -> bool + Send,
    {
        self.provider_types
            .iter()
            .filter(|(name, info)| predicate(name, info))
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_discovery() {
        let discovery = ProviderDiscovery::new();

        // Test listing provider types
        let providers = discovery.list_provider_types();
        assert!(providers.len() >= 10);
        assert!(providers.contains(&"openai".to_string()));
        assert!(providers.contains(&"anthropic".to_string()));

        // Test getting provider info
        let openai_info = discovery.get_provider_info("openai").unwrap();
        assert_eq!(openai_info.name, "openai");
        assert!(openai_info.supports_streaming);
        assert!(openai_info.supports_multimodal);

        // Test cloud providers
        let cloud_providers = discovery.get_providers_by_category("cloud");
        assert!(cloud_providers.len() >= 10);

        // Test streaming providers
        let streaming_providers = discovery.get_streaming_providers();
        assert!(streaming_providers.len() >= 10);

        // Test multimodal providers
        let multimodal_providers = discovery.get_multimodal_providers();
        assert!(multimodal_providers.len() >= 3);
    }

    #[tokio::test]
    async fn test_provider_bridge_discovery() {
        let discovery = ProviderDiscovery::new();

        // Test discover_types
        let types = discovery.discover_types().await;
        assert!(types.len() >= 10);

        // Test get_type_info
        let anthropic_info = discovery.get_type_info("anthropic").await.unwrap();
        assert_eq!(anthropic_info.category, "cloud");
        assert!(anthropic_info
            .features
            .contains(&"large_context".to_string()));

        // Test has_type
        assert!(discovery.has_type("groq").await);
        assert!(discovery.has_type("mistral").await);
        assert!(!discovery.has_type("nonexistent").await);

        // Test filter_types
        let vision_providers = discovery
            .filter_types(|_, info| info.features.iter().any(|f| f.contains("vision")))
            .await;
        assert!(vision_providers.len() >= 2); // openai, anthropic, gemini
    }
}
