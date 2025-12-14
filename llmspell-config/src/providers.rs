//! ABOUTME: Provider configuration definitions for llmspell  
//! ABOUTME: Manages LLM provider configurations and credentials

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

/// Provider manager configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct ProviderManagerConfig {
    /// Default provider to use
    #[serde(default)]
    pub default_provider: Option<String>,
    /// Provider-specific configurations
    #[serde(default, flatten)]
    pub providers: HashMap<String, ProviderConfig>,
}

impl ProviderManagerConfig {
    /// Create a new builder for `ProviderManagerConfig`
    #[must_use]
    pub fn builder() -> ProviderManagerConfigBuilder {
        ProviderManagerConfigBuilder::new()
    }

    /// Get a provider configuration by name
    #[must_use]
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.providers.get(name)
    }

    /// Add a provider configuration
    pub fn add_provider(&mut self, name: String, config: ProviderConfig) {
        self.providers.insert(name, config);
    }

    /// Remove a provider configuration
    pub fn remove_provider(&mut self, name: &str) -> Option<ProviderConfig> {
        self.providers.remove(name)
    }

    /// Get the default provider configuration
    #[must_use]
    pub fn get_default_provider(&self) -> Option<&ProviderConfig> {
        self.default_provider
            .as_ref()
            .and_then(|name| self.providers.get(name))
    }
}

/// Builder for `ProviderManagerConfig`
#[derive(Debug, Clone)]
pub struct ProviderManagerConfigBuilder {
    config: ProviderManagerConfig,
}

impl ProviderManagerConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProviderManagerConfig::default(),
        }
    }

    /// Set the default provider
    #[must_use]
    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.default_provider = Some(provider.into());
        self
    }

    /// Add a provider configuration
    #[must_use]
    pub fn add_provider(mut self, name: impl Into<String>, config: ProviderConfig) -> Self {
        self.config.providers.insert(name.into(), config);
        self
    }

    /// Set all providers at once
    #[must_use]
    pub fn providers(mut self, providers: HashMap<String, ProviderConfig>) -> Self {
        self.config.providers = providers;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> ProviderManagerConfig {
        self.config
    }
}

impl Default for ProviderManagerConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Individual provider configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
pub struct ProviderConfig {
    /// Provider name identifier
    #[serde(default)]
    pub name: String,
    /// Provider type (e.g., "openai", "anthropic", "ollama")
    #[serde(default)]
    pub provider_type: String,
    /// Whether this provider is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Base URL for the provider API
    pub base_url: Option<String>,
    /// Environment variable name for API key
    pub api_key_env: Option<String>,
    /// Direct API key (not recommended for production)
    pub api_key: Option<String>,
    /// Default model to use (aliased from model for compatibility)
    #[serde(alias = "model")]
    pub default_model: Option<String>,
    /// Maximum tokens per request
    pub max_tokens: Option<u32>,
    /// Request timeout in seconds
    pub timeout_seconds: Option<u64>,
    /// Temperature for LLM sampling (0.0-2.0)
    pub temperature: Option<f32>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
    /// Retry configuration
    pub retry: Option<RetryConfig>,
    /// Maximum retries (shorthand, overrides retry.max_retries if set)
    pub max_retries: Option<u32>,
    /// Provider-specific options
    #[serde(flatten)]
    pub options: HashMap<String, serde_json::Value>,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            provider_type: String::new(),
            enabled: true,
            base_url: None,
            api_key_env: None,
            api_key: None,
            default_model: None,
            max_tokens: None,
            timeout_seconds: None,
            temperature: None,
            rate_limit: None,
            retry: None,
            max_retries: None,
            options: HashMap::new(),
        }
    }
}

impl ProviderConfig {
    /// Create a new builder for `ProviderConfig`
    #[must_use]
    pub fn builder() -> ProviderConfigBuilder {
        ProviderConfigBuilder::new()
    }

    /// Check if this provider has valid credentials configured
    #[must_use]
    pub fn has_credentials(&self) -> bool {
        self.api_key.is_some()
            || self
                .api_key_env
                .as_ref()
                .is_some_and(|env_var| std::env::var(env_var).is_ok())
    }

    /// Get the API key, either direct or from environment
    #[must_use]
    pub fn get_api_key(&self) -> Option<String> {
        if let Some(key) = &self.api_key {
            return Some(key.clone());
        }

        if let Some(env_var) = &self.api_key_env {
            return std::env::var(env_var).ok();
        }

        None
    }
}

/// Builder for `ProviderConfig`
#[derive(Debug, Clone)]
pub struct ProviderConfigBuilder {
    config: ProviderConfig,
}

impl ProviderConfigBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ProviderConfig {
                name: String::new(),
                provider_type: String::new(),
                enabled: true,
                base_url: None,
                api_key_env: None,
                api_key: None,
                default_model: None,
                max_tokens: None,
                timeout_seconds: Some(60),
                temperature: None,
                rate_limit: None,
                retry: None,
                max_retries: None,
                options: HashMap::new(),
            },
        }
    }

    /// Set the provider type
    #[must_use]
    pub fn provider_type(mut self, provider_type: impl Into<String>) -> Self {
        self.config.provider_type = provider_type.into();
        self
    }

    /// Set the base URL
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.config.base_url = Some(url.into());
        self
    }

    /// Set the API key environment variable
    #[must_use]
    pub fn api_key_env(mut self, env_var: impl Into<String>) -> Self {
        self.config.api_key_env = Some(env_var.into());
        self
    }

    /// Set the API key directly (not recommended for production)
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.config.api_key = Some(key.into());
        self
    }

    /// Set the default model
    #[must_use]
    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = Some(model.into());
        self
    }

    /// Set the default model (explicit name)
    #[must_use]
    pub fn default_model(mut self, model: impl Into<String>) -> Self {
        self.config.default_model = Some(model.into());
        self
    }

    /// Set the maximum tokens
    #[must_use]
    pub const fn max_tokens(mut self, tokens: u32) -> Self {
        self.config.max_tokens = Some(tokens);
        self
    }

    /// Set the timeout in seconds
    #[must_use]
    pub const fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = Some(seconds);
        self
    }

    /// Set the temperature for LLM sampling
    #[must_use]
    pub const fn temperature(mut self, temp: f32) -> Self {
        self.config.temperature = Some(temp);
        self
    }

    /// Set rate limiting configuration
    #[must_use]
    pub fn rate_limit(mut self, rate_limit: RateLimitConfig) -> Self {
        self.config.rate_limit = Some(rate_limit);
        self
    }

    /// Set retry configuration
    #[must_use]
    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.config.retry = Some(retry);
        self
    }

    /// Add a custom option
    #[must_use]
    pub fn option(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.config.options.insert(key.into(), value);
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> ProviderConfig {
        self.config
    }
}

impl Default for ProviderConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: u32,
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
    /// Enable automatic retry on rate limit
    pub auto_retry: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            tokens_per_minute: None,
            auto_retry: true,
        }
    }
}

/// Retry configuration
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct RetryConfig {
    /// Maximum number of retries
    pub max_retries: u32,
    /// Initial delay in milliseconds
    pub initial_delay_ms: u64,
    /// Maximum delay in milliseconds
    pub max_delay_ms: u64,
    /// Backoff multiplier
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::builder()
            .provider_type("openai")
            .api_key_env("OPENAI_API_KEY")
            .model("gpt-4")
            .max_tokens(4096)
            .temperature(0.7)
            .build();

        assert_eq!(config.provider_type, "openai");
        assert_eq!(config.api_key_env, Some("OPENAI_API_KEY".to_string()));
        assert_eq!(config.default_model, Some("gpt-4".to_string()));
        assert_eq!(config.max_tokens, Some(4096));
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_provider_config_temperature_builder() {
        let config = ProviderConfig::builder()
            .provider_type("ollama")
            .temperature(0.0)
            .build();

        assert_eq!(config.temperature, Some(0.0));

        let config_high_temp = ProviderConfig::builder()
            .provider_type("ollama")
            .temperature(2.0)
            .build();

        assert_eq!(config_high_temp.temperature, Some(2.0));
    }

    #[test]
    fn test_provider_manager_config_builder() {
        let openai_config = ProviderConfig::builder()
            .provider_type("openai")
            .api_key_env("OPENAI_API_KEY")
            .build();

        let config = ProviderManagerConfig::builder()
            .default_provider("openai")
            .add_provider("openai", openai_config)
            .build();

        assert_eq!(config.default_provider, Some("openai".to_string()));
        assert!(config.providers.contains_key("openai"));
    }

    #[test]
    fn test_provider_config_credentials() {
        // Test with environment variable (mocked as not set)
        let config = ProviderConfig::builder()
            .provider_type("openai")
            .api_key_env("NONEXISTENT_KEY")
            .build();

        assert!(!config.has_credentials());

        // Test with direct API key
        let config = ProviderConfig::builder()
            .provider_type("openai")
            .api_key("test-key")
            .build();

        assert!(config.has_credentials());
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_provider_manager_operations() {
        let mut config = ProviderManagerConfig::default();

        let openai_config = ProviderConfig::builder().provider_type("openai").build();

        // Test add provider
        config.add_provider("openai".to_string(), openai_config);
        assert!(config.get_provider("openai").is_some());

        // Test remove provider
        let removed = config.remove_provider("openai");
        assert!(removed.is_some());
        assert!(config.get_provider("openai").is_none());
    }

    #[test]
    fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_minute, 60);
        assert!(config.auto_retry);
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 1000);
        assert_eq!(config.backoff_multiplier, 2.0);
    }

    #[test]
    fn test_provider_config_serialization() {
        let config = ProviderConfig::builder()
            .provider_type("anthropic")
            .api_key_env("ANTHROPIC_API_KEY")
            .model("claude-3-5-haiku-latest")
            .temperature(0.7)
            .option(
                "custom_option".to_string(),
                serde_json::json!("custom_value"),
            )
            .build();

        let serialized = serde_json::to_string(&config).expect("Serialization should work");
        let deserialized: ProviderConfig =
            serde_json::from_str(&serialized).expect("Deserialization should work");

        assert_eq!(deserialized.provider_type, "anthropic");
        assert_eq!(
            deserialized.default_model,
            Some("claude-3-5-haiku-latest".to_string())
        );
        assert_eq!(deserialized.temperature, Some(0.7));
        assert!(deserialized.options.contains_key("custom_option"));
    }
}
