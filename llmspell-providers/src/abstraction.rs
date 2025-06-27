//! ABOUTME: Provider abstraction layer defining capabilities and management interfaces
//! ABOUTME: Enables provider-agnostic LLM integration with capability detection

use async_trait::async_trait;
use llmspell_core::{
    error::LLMSpellError,
    types::{AgentInput, AgentOutput, AgentStream},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Capabilities that a provider might support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ProviderCapabilities {
    /// Whether the provider supports streaming responses
    pub supports_streaming: bool,

    /// Whether the provider supports multimodal content (images, audio, etc.)
    pub supports_multimodal: bool,

    /// Maximum context window size in tokens
    pub max_context_tokens: Option<usize>,

    /// Maximum output tokens
    pub max_output_tokens: Option<usize>,

    /// Supported model names
    pub available_models: Vec<String>,

    /// Provider-specific features
    pub custom_features: HashMap<String, serde_json::Value>,
}

/// Configuration for a provider instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider name (e.g., "openai", "anthropic", "local")
    pub name: String,

    /// API endpoint URL (if applicable)
    pub endpoint: Option<String>,

    /// API key or authentication token
    pub api_key: Option<String>,

    /// Model to use
    pub model: String,

    /// Request timeout in seconds
    pub timeout_secs: Option<u64>,

    /// Maximum retries for failed requests
    pub max_retries: Option<u32>,

    /// Provider-specific configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl ProviderConfig {
    /// Create a new provider configuration
    pub fn new(name: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            model: model.into(),
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
            max_retries: Some(3),
            custom_config: HashMap::new(),
        }
    }

    /// Load configuration from environment variables
    pub fn from_env(name: &str) -> Result<Self, LLMSpellError> {
        let env_prefix = format!("LLMSPELL_{}_", name.to_uppercase());

        let api_key = std::env::var(format!("{}API_KEY", env_prefix)).ok();
        let endpoint = std::env::var(format!("{}ENDPOINT", env_prefix)).ok();
        let model =
            std::env::var(format!("{}MODEL", env_prefix)).unwrap_or_else(|_| "default".to_string());

        Ok(Self {
            name: name.to_string(),
            endpoint,
            api_key,
            model,
            timeout_secs: std::env::var(format!("{}TIMEOUT", env_prefix))
                .ok()
                .and_then(|s| s.parse().ok()),
            max_retries: std::env::var(format!("{}MAX_RETRIES", env_prefix))
                .ok()
                .and_then(|s| s.parse().ok()),
            custom_config: HashMap::new(),
        })
    }
}

/// Trait for LLM provider implementations
#[async_trait]
pub trait ProviderInstance: Send + Sync {
    /// Get the provider's capabilities
    fn capabilities(&self) -> &ProviderCapabilities;

    /// Execute a completion request
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput, LLMSpellError>;

    /// Execute a streaming completion request
    async fn complete_streaming(&self, _input: &AgentInput) -> Result<AgentStream, LLMSpellError> {
        // Default implementation returns NotImplemented error
        Err(LLMSpellError::Provider {
            message: "Streaming not implemented for this provider".to_string(),
            provider: Some(self.name().to_string()),
            source: None,
        })
    }

    /// Validate the provider configuration and connectivity
    async fn validate(&self) -> Result<(), LLMSpellError>;

    /// Get provider name
    fn name(&self) -> &str;

    /// Get current model
    fn model(&self) -> &str;
}

/// Factory function type for creating provider instances
pub type ProviderFactory =
    Box<dyn Fn(ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError> + Send + Sync>;

/// Type alias for provider instance storage
pub type ProviderInstanceMap = HashMap<String, Arc<Box<dyn ProviderInstance>>>;

/// Provider registry for managing available providers
pub struct ProviderRegistry {
    factories: HashMap<String, ProviderFactory>,
}

impl ProviderRegistry {
    /// Create a new provider registry
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
        }
    }

    /// Register a provider factory
    pub fn register<F>(&mut self, name: impl Into<String>, factory: F)
    where
        F: Fn(ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError>
            + Send
            + Sync
            + 'static,
    {
        self.factories.insert(name.into(), Box::new(factory));
    }

    /// Create a provider instance
    pub fn create(
        &self,
        config: ProviderConfig,
    ) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
        let factory =
            self.factories
                .get(&config.name)
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: format!("Unknown provider: {}", config.name),
                    source: None,
                })?;

        factory(config)
    }

    /// Get list of registered provider names
    pub fn available_providers(&self) -> Vec<&str> {
        self.factories.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Provider manager for handling multiple provider instances
pub struct ProviderManager {
    registry: Arc<RwLock<ProviderRegistry>>,
    instances: Arc<RwLock<ProviderInstanceMap>>,
    default_provider: Arc<RwLock<Option<String>>>,
}

impl ProviderManager {
    /// Create a new provider manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(RwLock::new(ProviderRegistry::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            default_provider: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a provider factory
    pub async fn register_provider<F>(&self, name: impl Into<String>, factory: F)
    where
        F: Fn(ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError>
            + Send
            + Sync
            + 'static,
    {
        let mut registry = self.registry.write().await;
        registry.register(name, factory);
    }

    /// Initialize a provider instance
    pub async fn init_provider(&self, config: ProviderConfig) -> Result<(), LLMSpellError> {
        let instance_name = format!("{}:{}", config.name, config.model);

        let registry = self.registry.read().await;
        let provider = registry.create(config)?;

        // Validate the provider
        provider.validate().await?;

        let mut instances = self.instances.write().await;
        instances.insert(instance_name.clone(), Arc::new(provider));

        // Set as default if it's the first provider
        let mut default = self.default_provider.write().await;
        if default.is_none() {
            *default = Some(instance_name);
        }

        Ok(())
    }

    /// Get a provider instance
    pub async fn get_provider(
        &self,
        name: Option<&str>,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        let instances = self.instances.read().await;
        let default = self.default_provider.read().await;

        let provider_name = if let Some(name) = name {
            name.to_string()
        } else {
            default
                .as_ref()
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "No default provider configured".to_string(),
                    source: None,
                })?
                .clone()
        };

        instances
            .get(&provider_name)
            .cloned()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: format!("Provider not found: {}", provider_name),
                source: None,
            })
    }

    /// Set the default provider
    pub async fn set_default_provider(&self, name: impl Into<String>) -> Result<(), LLMSpellError> {
        let name = name.into();
        let instances = self.instances.read().await;

        if !instances.contains_key(&name) {
            return Err(LLMSpellError::Configuration {
                message: format!("Cannot set default: provider '{}' not initialized", name),
                source: None,
            });
        }

        let mut default = self.default_provider.write().await;
        *default = Some(name);
        Ok(())
    }

    /// Query capabilities of a provider
    pub async fn query_capabilities(
        &self,
        name: Option<&str>,
    ) -> Result<ProviderCapabilities, LLMSpellError> {
        let instances = self.instances.read().await;
        let default = self.default_provider.read().await;

        let provider_name = if let Some(name) = name {
            name.to_string()
        } else {
            default
                .as_ref()
                .ok_or_else(|| LLMSpellError::Configuration {
                    message: "No default provider configured".to_string(),
                    source: None,
                })?
                .clone()
        };

        instances
            .get(&provider_name)
            .ok_or_else(|| LLMSpellError::Configuration {
                message: format!("Provider not found: {}", provider_name),
                source: None,
            })
            .map(|p| p.capabilities().clone())
    }

    /// List all initialized providers
    pub async fn list_providers(&self) -> Vec<String> {
        let instances = self.instances.read().await;
        instances.keys().cloned().collect()
    }

    /// List all available provider types
    pub async fn available_provider_types(&self) -> Vec<String> {
        let registry = self.registry.read().await;
        registry
            .available_providers()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
}

impl Default for ProviderManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_capabilities_default() {
        let caps = ProviderCapabilities::default();
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_multimodal);
        assert!(caps.max_context_tokens.is_none());
        assert!(caps.available_models.is_empty());
    }

    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new("openai", "gpt-4");
        assert_eq!(config.name, "openai");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.timeout_secs, Some(30));
        assert_eq!(config.max_retries, Some(3));
    }

    #[test]
    fn test_provider_registry() {
        let mut registry = ProviderRegistry::new();

        // Register a mock factory
        registry.register("mock", |_config| {
            Err(LLMSpellError::Provider {
                message: "Mock provider".to_string(),
                provider: Some("mock".to_string()),
                source: None,
            })
        });

        assert_eq!(registry.available_providers(), vec!["mock"]);
    }

    #[tokio::test]
    async fn test_provider_manager_initialization() {
        let manager = ProviderManager::new();

        // Register a mock provider
        manager
            .register_provider("mock", |_config| {
                Err(LLMSpellError::Provider {
                    message: "Mock provider".to_string(),
                    provider: Some("mock".to_string()),
                    source: None,
                })
            })
            .await;

        let types = manager.available_provider_types().await;
        assert!(types.contains(&"mock".to_string()));
    }
}
