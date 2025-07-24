//! ABOUTME: Provider management for LLM providers accessible from scripts
//! ABOUTME: Manages provider configuration, capability detection, and access control

use llmspell_core::error::LLMSpellError;
use llmspell_providers::{
    ModelSpecifier, ProviderCapabilities, ProviderConfig as ProviderInstanceConfig,
    ProviderInstance, ProviderManager as CoreProviderManager,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Manages LLM providers for script access
pub struct ProviderManager {
    core_manager: CoreProviderManager,
    config: ProviderManagerConfig,
}

impl ProviderManager {
    /// Create a new provider manager with configuration
    pub async fn new(config: ProviderManagerConfig) -> Result<Self, LLMSpellError> {
        let core_manager = CoreProviderManager::new();
        let manager = Self {
            core_manager,
            config: config.clone(),
        };

        // Register the rig provider factory
        manager.register_rig_provider().await?;

        // Initialize configured providers
        manager.initialize_providers().await?;

        Ok(manager)
    }

    /// Register the rig provider factory
    async fn register_rig_provider(&self) -> Result<(), LLMSpellError> {
        self.core_manager
            .register_provider("rig", llmspell_providers::create_rig_provider)
            .await;
        Ok(())
    }

    /// Initialize providers from configuration
    async fn initialize_providers(&self) -> Result<(), LLMSpellError> {
        // Initialize each configured provider
        for (name, config) in &self.config.providers {
            let provider_config = self.create_provider_config(name, config)?;
            self.core_manager.init_provider(provider_config).await?;
        }

        // Set default provider if specified
        if let Some(ref default) = self.config.default_provider {
            if !self.config.providers.contains_key(default) {
                return Err(LLMSpellError::Validation {
                    field: Some("default_provider".to_string()),
                    message: format!("Default provider '{}' not found in configuration", default),
                });
            }
            // The default will be set based on the provider:model format
            let model = self.config.providers[default]
                .model
                .as_ref()
                .ok_or_else(|| LLMSpellError::Validation {
                    field: Some("model".to_string()),
                    message: format!("Model not specified for provider '{}'", default),
                })?;
            // Get the provider config to determine the actual provider name
            let provider_config = &self.config.providers[default];
            let provider_name = match provider_config.provider_type.as_str() {
                "openai" | "anthropic" | "cohere" | "groq" | "perplexity" | "together"
                | "gemini" | "mistral" | "replicate" | "fireworks" => "rig",
                other => other,
            };

            // Use hierarchical naming: name/provider_type/model
            self.core_manager
                .set_default_provider(format!(
                    "{}/{}/{}",
                    provider_name, provider_config.provider_type, model
                ))
                .await?;
        }

        Ok(())
    }

    /// Create a provider config from our configuration
    fn create_provider_config(
        &self,
        name: &str,
        config: &ProviderConfig,
    ) -> Result<ProviderInstanceConfig, LLMSpellError> {
        // Map provider_type to the actual provider name
        let provider_name = match config.provider_type.as_str() {
            "openai" | "anthropic" | "cohere" | "groq" | "perplexity" | "together" | "gemini"
            | "mistral" | "replicate" | "fireworks" => "rig",
            other => other,
        };

        let model = config
            .model
            .as_ref()
            .ok_or_else(|| LLMSpellError::Validation {
                field: Some("model".to_string()),
                message: format!("Model not specified for provider '{}'", name),
            })?;

        // Use new_with_type to preserve provider_type information
        let mut provider_config =
            ProviderInstanceConfig::new_with_type(provider_name, &config.provider_type, model);

        // Set API key from environment if specified
        if let Some(ref api_key_env) = config.api_key_env {
            let api_key = std::env::var(api_key_env).map_err(|_| LLMSpellError::Configuration {
                message: format!(
                    "Environment variable '{}' not found for provider '{}'",
                    api_key_env, name
                ),
                source: None,
            })?;
            provider_config.api_key = Some(api_key);
        }

        // Set other configuration
        if let Some(ref base_url) = config.base_url {
            provider_config.endpoint = Some(base_url.clone());
        }

        // Add max_tokens to custom config if specified
        if let Some(max_tokens) = config.max_tokens {
            provider_config
                .custom_config
                .insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }

        // Add extra configuration
        for (key, value) in &config.extra {
            provider_config
                .custom_config
                .insert(key.clone(), value.clone());
        }

        Ok(provider_config)
    }

    /// Get a provider by name
    pub async fn get_provider(
        &self,
        name: Option<&str>,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        self.core_manager.get_provider(name).await
    }

    /// Get the default provider
    pub async fn get_default_provider(
        &self,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        self.core_manager.get_provider(None).await
    }

    /// Set the default provider
    pub async fn set_default_provider(&self, name: String) -> Result<(), LLMSpellError> {
        self.core_manager.set_default_provider(name).await
    }

    /// Create and initialize a provider from a ModelSpecifier
    ///
    /// This is a bridge method that delegates to the core provider manager's
    /// create_agent_from_spec method. It supports the new "provider/model" syntax.
    pub async fn create_agent_from_spec(
        &self,
        spec: ModelSpecifier,
        base_url_override: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        self.core_manager
            .create_agent_from_spec(spec, base_url_override, api_key)
            .await
    }

    /// List all registered providers
    pub async fn list_providers(&self) -> Vec<ProviderInfo> {
        let providers = self.core_manager.list_providers().await;
        let mut infos = Vec::new();

        for provider_name in providers {
            if let Ok(capabilities) = self
                .core_manager
                .query_capabilities(Some(&provider_name))
                .await
            {
                infos.push(ProviderInfo {
                    name: provider_name,
                    capabilities,
                });
            }
        }

        infos
    }

    /// Check if a provider supports a specific capability
    pub async fn provider_supports(&self, provider_name: &str, capability: &str) -> bool {
        if let Ok(caps) = self
            .core_manager
            .query_capabilities(Some(provider_name))
            .await
        {
            match capability {
                "streaming" => caps.supports_streaming,
                "multimodal" => caps.supports_multimodal,
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get the core provider manager
    pub fn core_manager(&self) -> &CoreProviderManager {
        &self.core_manager
    }

    /// Create an Arc to a new core provider manager with the same configuration
    /// This is needed for components that require ownership of the core manager
    pub async fn create_core_manager_arc(&self) -> Result<Arc<CoreProviderManager>, LLMSpellError> {
        // Create a new core manager
        let core_manager = CoreProviderManager::new();

        // Register the rig provider factory
        core_manager
            .register_provider("rig", llmspell_providers::create_rig_provider)
            .await;

        // Initialize providers from our configuration
        for (name, config) in &self.config.providers {
            let provider_config = self.create_provider_config(name, config)?;
            core_manager.init_provider(provider_config).await?;
        }

        // Set default provider if specified
        if let Some(ref default) = self.config.default_provider {
            if let Some(provider_config) = self.config.providers.get(default) {
                let model =
                    provider_config
                        .model
                        .as_ref()
                        .ok_or_else(|| LLMSpellError::Validation {
                            field: Some("model".to_string()),
                            message: format!("Model not specified for provider '{}'", default),
                        })?;
                let provider_name = match provider_config.provider_type.as_str() {
                    "openai" | "anthropic" | "cohere" => "rig",
                    other => other,
                };
                core_manager
                    .set_default_provider(format!(
                        "{}/{}/{}",
                        provider_name, provider_config.provider_type, model
                    ))
                    .await?;
            }
        }

        Ok(Arc::new(core_manager))
    }
}

/// Information about a registered provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub capabilities: ProviderCapabilities,
}

/// Configuration for the provider manager
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ProviderManagerConfig {
    /// Default provider to use
    pub default_provider: Option<String>,
    /// Provider-specific configurations
    pub providers: HashMap<String, ProviderConfig>,
}

/// Configuration for a specific provider
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderConfig {
    /// Provider type (e.g., "openai", "anthropic", "local")
    pub provider_type: String,
    /// API key or authentication
    pub api_key_env: Option<String>,
    /// Base URL override
    pub base_url: Option<String>,
    /// Model to use
    pub model: Option<String>,
    /// Maximum tokens
    pub max_tokens: Option<u32>,
    /// Additional provider-specific settings
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_manager_creation() {
        let config = ProviderManagerConfig::default();
        let manager = ProviderManager::new(config).await.unwrap();
        assert!(manager.get_default_provider().await.is_err());
    }

    #[tokio::test]
    async fn test_provider_config_validation() {
        let mut config = ProviderManagerConfig::default();
        config.default_provider = Some("nonexistent".to_string());

        let result = ProviderManager::new(config).await;
        assert!(result.is_err());
    }
}
