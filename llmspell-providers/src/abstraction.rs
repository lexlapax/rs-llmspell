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

use crate::ModelSpecifier;

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

    /// Provider type (e.g., "openai", "anthropic", "cohere")
    /// This is used to determine which provider implementation to use
    pub provider_type: String,

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
        let name_str = name.into();
        // For backward compatibility, use name as provider_type if it's a known provider
        let provider_type = match name_str.as_str() {
            "openai" | "anthropic" | "cohere" => name_str.clone(),
            _ => name_str.clone(), // Default to name for unknown providers
        };

        Self {
            name: name_str,
            provider_type,
            model: model.into(),
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
            max_retries: Some(3),
            custom_config: HashMap::new(),
        }
    }

    /// Create a new provider configuration with explicit provider type
    pub fn new_with_type(
        name: impl Into<String>,
        provider_type: impl Into<String>,
        model: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            provider_type: provider_type.into(),
            model: model.into(),
            endpoint: None,
            api_key: None,
            timeout_secs: Some(30),
            max_retries: Some(3),
            custom_config: HashMap::new(),
        }
    }

    /// Generate hierarchical provider instance name
    /// Format: "{name}/{provider_type}/{model}"
    /// Example: "rig/openai/gpt-4"
    pub fn instance_name(&self) -> String {
        format!("{}/{}/{}", self.name, self.provider_type, self.model)
    }

    /// Load configuration from environment variables
    pub fn from_env(name: &str) -> Result<Self, LLMSpellError> {
        let env_prefix = format!("LLMSPELL_{}_", name.to_uppercase());

        // Try LLMSPELL_<PROVIDER>_API_KEY first, then fall back to standard env vars
        let api_key = std::env::var(format!("{}API_KEY", env_prefix))
            .ok()
            .or_else(|| {
                // Fall back to standard environment variable names
                match name.to_lowercase().as_str() {
                    "openai" => std::env::var("OPENAI_API_KEY").ok(),
                    "anthropic" => std::env::var("ANTHROPIC_API_KEY").ok(),
                    "cohere" => std::env::var("COHERE_API_KEY").ok(),
                    "groq" => std::env::var("GROQ_API_KEY").ok(),
                    "perplexity" => std::env::var("PERPLEXITY_API_KEY").ok(),
                    "together" => std::env::var("TOGETHER_API_KEY").ok(),
                    "gemini" => std::env::var("GEMINI_API_KEY").ok(),
                    "mistral" => std::env::var("MISTRAL_API_KEY").ok(),
                    "replicate" => std::env::var("REPLICATE_API_TOKEN").ok(),
                    "fireworks" => std::env::var("FIREWORKS_API_KEY").ok(),
                    _ => None,
                }
            });

        let endpoint = std::env::var(format!("{}ENDPOINT", env_prefix)).ok();
        let model =
            std::env::var(format!("{}MODEL", env_prefix)).unwrap_or_else(|_| "default".to_string());

        // Try to get provider_type from env, or use name as default
        let provider_type = std::env::var(format!("{}PROVIDER_TYPE", env_prefix))
            .unwrap_or_else(|_| name.to_string());

        Ok(Self {
            name: name.to_string(),
            provider_type,
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
        // Use hierarchical naming: name/provider_type/model
        let instance_name = config.instance_name();

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

    /// Create and initialize a provider from a ModelSpecifier
    ///
    /// This method handles the core functionality for Task 2.1.2:
    /// - Parses the ModelSpecifier to determine provider and model
    /// - Applies base URL overrides if specified
    /// - Creates provider configuration with proper fallbacks
    /// - Initializes the provider and makes it available
    ///
    /// # Arguments
    /// * `spec` - ModelSpecifier containing provider/model information
    /// * `base_url_override` - Optional base URL to override default endpoints
    /// * `api_key` - Optional API key (falls back to environment variables)
    ///
    /// # Returns
    /// Returns the initialized provider instance
    ///
    /// # Examples
    /// ```no_run
    /// # use llmspell_providers::{ProviderManager, ModelSpecifier};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let manager = ProviderManager::new();
    ///
    /// // Basic usage with provider/model syntax
    /// let spec = ModelSpecifier::parse("openai/gpt-4")?;
    /// let provider = manager.create_agent_from_spec(spec, None, None).await?;
    ///
    /// // With base URL override
    /// let spec = ModelSpecifier::parse("openai/gpt-4")?;
    /// let provider = manager.create_agent_from_spec(
    ///     spec,
    ///     Some("https://api.custom.com/v1"),
    ///     Some("custom-api-key")
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_agent_from_spec(
        &self,
        spec: ModelSpecifier,
        base_url_override: Option<&str>,
        api_key: Option<&str>,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        // Determine the provider name
        let provider_name = match &spec.provider {
            Some(provider) => provider.clone(),
            None => {
                // If no provider specified, try to use default
                let default = self.default_provider.read().await;
                if let Some(default_provider) = default.as_ref() {
                    // Extract provider name from hierarchical format "name/provider_type/model"
                    // We want the provider_type (second part)
                    let parts: Vec<&str> = default_provider.split('/').collect();
                    if parts.len() >= 2 {
                        parts[1].to_string() // Return the provider_type
                    } else {
                        return Err(LLMSpellError::Configuration {
                            message:
                                "No provider specified and no valid default provider available"
                                    .to_string(),
                            source: None,
                        });
                    }
                } else {
                    return Err(LLMSpellError::Configuration {
                        message: "No provider specified and no default provider configured"
                            .to_string(),
                        source: None,
                    });
                }
            }
        };

        // Map provider types to implementation names
        // These are all the providers that could potentially be supported by rig
        let implementation_name = match provider_name.as_str() {
            "openai" | "anthropic" | "cohere" | "groq" | "perplexity" | "together" | "gemini"
            | "mistral" | "replicate" | "fireworks" => "rig",
            other => other,
        };

        // Create provider configuration with implementation name but preserve provider type
        let mut config =
            ProviderConfig::new_with_type(implementation_name, &provider_name, &spec.model);

        // Apply base URL override with precedence:
        // 1. Function parameter (highest priority)
        // 2. ModelSpecifier base_url
        // 3. Default provider endpoint (lowest priority)
        if let Some(base_url) = base_url_override {
            config.endpoint = Some(base_url.to_string());
        } else if let Some(base_url) = &spec.base_url {
            config.endpoint = Some(base_url.clone());
        }

        // Set API key with fallback to environment variables
        if let Some(api_key) = api_key {
            config.api_key = Some(api_key.to_string());
        } else {
            // Try to load from environment
            if let Ok(env_config) = ProviderConfig::from_env(&provider_name) {
                if config.api_key.is_none() {
                    config.api_key = env_config.api_key;
                }
                if config.endpoint.is_none() {
                    config.endpoint = env_config.endpoint;
                }
            }
        }

        // Create a unique instance name
        let instance_name = format!("{}:{}", provider_name, spec.model);

        // Check if we already have this instance
        {
            let instances = self.instances.read().await;
            if let Some(existing) = instances.get(&instance_name) {
                return Ok(existing.clone());
            }
        }

        // Create the provider instance
        let registry = self.registry.read().await;
        let provider = registry.create(config)?;

        // Validate the provider
        provider.validate().await?;

        // Store the instance
        let provider_arc = Arc::new(provider);
        let mut instances = self.instances.write().await;
        instances.insert(instance_name, provider_arc.clone());

        Ok(provider_arc)
    }

    /// Get the default provider instance
    pub async fn get_default_provider(
        &self,
    ) -> Result<Arc<Box<dyn ProviderInstance>>, LLMSpellError> {
        self.get_provider(None).await
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

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_provider_capabilities_default() {
        let caps = ProviderCapabilities::default();
        assert!(!caps.supports_streaming);
        assert!(!caps.supports_multimodal);
        assert!(caps.max_context_tokens.is_none());
        assert!(caps.available_models.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_provider_config_creation() {
        let config = ProviderConfig::new("openai", "gpt-4");
        assert_eq!(config.name, "openai");
        assert_eq!(config.model, "gpt-4");
        assert_eq!(config.timeout_secs, Some(30));
        assert_eq!(config.max_retries, Some(3));
    }

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
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

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_create_agent_from_spec_no_provider() {
        use crate::ModelSpecifier;

        let manager = ProviderManager::new();
        let spec = ModelSpecifier::parse("gpt-4").unwrap();

        // Should fail when no provider specified and no default
        let result = manager.create_agent_from_spec(spec, None, None).await;
        assert!(result.is_err());

        if let Err(LLMSpellError::Configuration { message, .. }) = result {
            assert!(message.contains("No provider specified"));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_create_agent_from_spec_unknown_provider() {
        use crate::ModelSpecifier;

        let manager = ProviderManager::new();
        let spec = ModelSpecifier::parse("unknown/model").unwrap();

        // Should fail when provider not registered
        let result = manager.create_agent_from_spec(spec, None, None).await;
        assert!(result.is_err());

        if let Err(LLMSpellError::Configuration { message, .. }) = result {
            assert!(message.contains("Unknown provider"));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_model_specifier_base_url_precedence() {
        use crate::ModelSpecifier;

        let manager = ProviderManager::new();

        // Register a mock provider that tracks configuration
        manager
            .register_provider("test", |config| {
                // Verify base URL is set correctly
                assert_eq!(config.endpoint, Some("https://override.com".to_string()));
                Err(LLMSpellError::Provider {
                    message: "Test validation".to_string(),
                    provider: Some("test".to_string()),
                    source: None,
                })
            })
            .await;

        let spec =
            ModelSpecifier::parse_with_base_url("test/model", Some("https://spec.com")).unwrap();

        // Override parameter should take precedence over spec base_url
        let result = manager
            .create_agent_from_spec(spec, Some("https://override.com"), None)
            .await;

        // Should fail at validation (expected for our mock)
        assert!(result.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_model_specifier_provider_extraction() {
        use crate::ModelSpecifier;

        // Test different provider/model formats
        let spec1 = ModelSpecifier::parse("openai/gpt-4").unwrap();
        assert_eq!(spec1.provider, Some("openai".to_string()));
        assert_eq!(spec1.model, "gpt-4");

        let spec2 = ModelSpecifier::parse("openrouter/deepseek/model").unwrap();
        assert_eq!(spec2.provider, Some("openrouter/deepseek".to_string()));
        assert_eq!(spec2.model, "model");

        let spec3 = ModelSpecifier::parse("claude-3").unwrap();
        assert_eq!(spec3.provider, None);
        assert_eq!(spec3.model, "claude-3");
    }
}
