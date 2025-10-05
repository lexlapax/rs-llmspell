//! ABOUTME: ModelSpecifier for parsing provider/model syntax
//! ABOUTME: Handles "provider/model", "model", and base URL override parsing

use llmspell_core::error::LLMSpellError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Specification for a model with optional provider, backend, and base URL
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelSpecifier {
    /// The provider name (e.g., "openai", "anthropic", "local")
    pub provider: Option<String>,
    /// The model name (e.g., "gpt-4", "claude-3-sonnet", "llama3.1:8b")
    pub model: String,
    /// Backend for local providers (e.g., "ollama", "candle")
    pub backend: Option<String>,
    /// Optional base URL override
    pub base_url: Option<String>,
}

impl ModelSpecifier {
    /// Create a new ModelSpecifier with just a model name
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            provider: None,
            model: model.into(),
            backend: None,
            base_url: None,
        }
    }

    /// Create a new ModelSpecifier with provider and model
    pub fn with_provider(provider: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            provider: Some(provider.into()),
            model: model.into(),
            backend: None,
            base_url: None,
        }
    }

    /// Create a new ModelSpecifier with provider, model, and base URL
    pub fn with_base_url(
        provider: impl Into<String>,
        model: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            provider: Some(provider.into()),
            model: model.into(),
            backend: None,
            base_url: Some(base_url.into()),
        }
    }

    /// Parse a model specification string
    ///
    /// Supported formats:
    /// - "model" -> ModelSpecifier { provider: None, model: "model", backend: None, base_url: None }
    /// - "provider/model" -> ModelSpecifier { provider: Some("provider"), model: "model", backend: None, base_url: None }
    /// - "provider/subprovider/model" -> ModelSpecifier { provider: Some("provider/subprovider"), model: "model", backend: None, base_url: None }
    /// - "model@backend" -> ModelSpecifier { provider: None, model: "model", backend: Some("backend"), base_url: None }
    /// - "provider/model@backend" -> ModelSpecifier { provider: Some("provider"), model: "model", backend: Some("backend"), base_url: None }
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use llmspell_providers::ModelSpecifier;
    /// let spec = ModelSpecifier::parse("gpt-4").unwrap();
    /// assert_eq!(spec.model, "gpt-4");
    /// assert_eq!(spec.provider, None);
    /// assert_eq!(spec.backend, None);
    ///
    /// let spec = ModelSpecifier::parse("openai/gpt-4").unwrap();
    /// assert_eq!(spec.model, "gpt-4");
    /// assert_eq!(spec.provider, Some("openai".to_string()));
    /// assert_eq!(spec.backend, None);
    ///
    /// let spec = ModelSpecifier::parse("local/llama3.1:8b@ollama").unwrap();
    /// assert_eq!(spec.model, "llama3.1:8b");
    /// assert_eq!(spec.provider, Some("local".to_string()));
    /// assert_eq!(spec.backend, Some("ollama".to_string()));
    ///
    /// let spec = ModelSpecifier::parse("llama3.1:8b@candle").unwrap();
    /// assert_eq!(spec.model, "llama3.1:8b");
    /// assert_eq!(spec.provider, None);
    /// assert_eq!(spec.backend, Some("candle".to_string()));
    /// ```
    pub fn parse(spec: &str) -> Result<Self, LLMSpellError> {
        let spec = spec.trim();

        if spec.is_empty() {
            return Err(LLMSpellError::Configuration {
                message: "Model specification cannot be empty".to_string(),
                source: None,
            });
        }

        // First, extract backend if present (split on rightmost '@')
        let (model_part, backend) = if let Some(idx) = spec.rfind('@') {
            (&spec[..idx], Some(spec[idx + 1..].to_string()))
        } else {
            (spec, None)
        };

        // Now parse provider/model from model_part
        let parts: Vec<&str> = model_part.split('/').collect();

        let mut result = match parts.len() {
            1 => {
                // Just a model name
                Self::new(parts[0])
            }
            2 => {
                // provider/model
                Self::with_provider(parts[0], parts[1])
            }
            n if n > 2 => {
                // provider/subprovider/.../model
                // Join all parts except the last as provider
                let provider = parts[..n - 1].join("/");
                let model = parts[n - 1];
                Self::with_provider(provider, model)
            }
            _ => {
                // This shouldn't happen with split, but handle gracefully
                return Err(LLMSpellError::Configuration {
                    message: format!("Invalid model specification format: '{}'", spec),
                    source: None,
                });
            }
        };

        // Set the backend field
        result.backend = backend;

        Ok(result)
    }

    /// Parse a model specification with an optional base URL override
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use llmspell_providers::ModelSpecifier;
    /// let spec = ModelSpecifier::parse_with_base_url(
    ///     "openai/gpt-4",
    ///     Some("https://api.custom.com/v1")
    /// ).unwrap();
    /// assert_eq!(spec.model, "gpt-4");
    /// assert_eq!(spec.provider, Some("openai".to_string()));
    /// assert_eq!(spec.base_url, Some("https://api.custom.com/v1".to_string()));
    /// ```
    pub fn parse_with_base_url(spec: &str, base_url: Option<&str>) -> Result<Self, LLMSpellError> {
        let mut model_spec = Self::parse(spec)?;
        model_spec.base_url = base_url.map(str::to_string);
        Ok(model_spec)
    }

    /// Get the provider name, or return a default
    pub fn provider_or_default<'a>(&'a self, default: &'a str) -> &'a str {
        self.provider.as_deref().unwrap_or(default)
    }

    /// Check if this specifier has a provider
    pub fn has_provider(&self) -> bool {
        self.provider.is_some()
    }

    /// Check if this specifier has a base URL override
    pub fn has_base_url(&self) -> bool {
        self.base_url.is_some()
    }

    /// Check if this specifier has a backend
    pub fn has_backend(&self) -> bool {
        self.backend.is_some()
    }

    /// Get the backend name, or return a default
    pub fn backend_or_default<'a>(&'a self, default: &'a str) -> &'a str {
        self.backend.as_deref().unwrap_or(default)
    }
}

impl std::fmt::Display for ModelSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Build provider/model part
        let model_spec = match &self.provider {
            Some(provider) => format!("{}/{}", provider, self.model),
            None => self.model.clone(),
        };

        // Append backend if present
        match &self.backend {
            Some(backend) => write!(f, "{}@{}", model_spec, backend),
            None => write!(f, "{}", model_spec),
        }
    }
}

impl FromStr for ModelSpecifier {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_model_only() {
        let spec = ModelSpecifier::parse("gpt-4").unwrap();
        assert_eq!(spec.model, "gpt-4");
        assert_eq!(spec.provider, None);
        assert_eq!(spec.base_url, None);
        assert!(!spec.has_provider());
        assert!(!spec.has_base_url());
    }
    #[test]
    fn test_parse_provider_model() {
        let spec = ModelSpecifier::parse("openai/gpt-4").unwrap();
        assert_eq!(spec.model, "gpt-4");
        assert_eq!(spec.provider, Some("openai".to_string()));
        assert_eq!(spec.base_url, None);
        assert!(spec.has_provider());
        assert!(!spec.has_base_url());
    }
    #[test]
    fn test_parse_nested_provider() {
        let spec = ModelSpecifier::parse("openrouter/deepseek/model").unwrap();
        assert_eq!(spec.model, "model");
        assert_eq!(spec.provider, Some("openrouter/deepseek".to_string()));
        assert_eq!(spec.base_url, None);
        assert!(spec.has_provider());
    }
    #[test]
    fn test_parse_deeply_nested() {
        let spec = ModelSpecifier::parse("a/b/c/d/model").unwrap();
        assert_eq!(spec.model, "model");
        assert_eq!(spec.provider, Some("a/b/c/d".to_string()));
    }
    #[test]
    fn test_parse_empty_string() {
        let result = ModelSpecifier::parse("");
        assert!(result.is_err());
        if let Err(LLMSpellError::Configuration { message, .. }) = result {
            assert!(message.contains("empty"));
        }
    }
    #[test]
    fn test_parse_whitespace_only() {
        let result = ModelSpecifier::parse("   ");
        assert!(result.is_err());
    }
    #[test]
    fn test_parse_with_base_url() {
        let spec =
            ModelSpecifier::parse_with_base_url("openai/gpt-4", Some("https://api.custom.com/v1"))
                .unwrap();

        assert_eq!(spec.model, "gpt-4");
        assert_eq!(spec.provider, Some("openai".to_string()));
        assert_eq!(spec.base_url, Some("https://api.custom.com/v1".to_string()));
        assert!(spec.has_base_url());
    }
    #[test]
    fn test_parse_with_base_url_none() {
        let spec = ModelSpecifier::parse_with_base_url("openai/gpt-4", None).unwrap();
        assert_eq!(spec.base_url, None);
        assert!(!spec.has_base_url());
    }
    #[test]
    fn test_constructor_methods() {
        let spec1 = ModelSpecifier::new("gpt-4");
        assert_eq!(spec1.model, "gpt-4");
        assert_eq!(spec1.provider, None);

        let spec2 = ModelSpecifier::with_provider("openai", "gpt-4");
        assert_eq!(spec2.model, "gpt-4");
        assert_eq!(spec2.provider, Some("openai".to_string()));

        let spec3 = ModelSpecifier::with_base_url("openai", "gpt-4", "https://api.custom.com");
        assert_eq!(spec3.model, "gpt-4");
        assert_eq!(spec3.provider, Some("openai".to_string()));
        assert_eq!(spec3.base_url, Some("https://api.custom.com".to_string()));
    }
    #[test]
    fn test_provider_or_default() {
        let spec1 = ModelSpecifier::new("gpt-4");
        assert_eq!(spec1.provider_or_default("default"), "default");

        let spec2 = ModelSpecifier::with_provider("openai", "gpt-4");
        assert_eq!(spec2.provider_or_default("default"), "openai");
    }
    #[test]
    fn test_to_string() {
        let spec1 = ModelSpecifier::new("gpt-4");
        assert_eq!(spec1.to_string(), "gpt-4");

        let spec2 = ModelSpecifier::with_provider("openai", "gpt-4");
        assert_eq!(spec2.to_string(), "openai/gpt-4");

        let spec3 = ModelSpecifier::parse("openrouter/deepseek/model").unwrap();
        assert_eq!(spec3.to_string(), "openrouter/deepseek/model");
    }
    #[test]
    fn test_display_trait() {
        let spec = ModelSpecifier::with_provider("openai", "gpt-4");
        assert_eq!(format!("{}", spec), "openai/gpt-4");
    }
    #[test]
    fn test_from_str_trait() {
        let spec: ModelSpecifier = "openai/gpt-4".parse().unwrap();
        assert_eq!(spec.model, "gpt-4");
        assert_eq!(spec.provider, Some("openai".to_string()));
    }
    #[test]
    fn test_serde_serialization() {
        let spec = ModelSpecifier::with_base_url("openai", "gpt-4", "https://api.custom.com");

        // Test serialization
        let serialized = serde_json::to_string(&spec).unwrap();

        // Test deserialization
        let deserialized: ModelSpecifier = serde_json::from_str(&serialized).unwrap();
        assert_eq!(spec, deserialized);
    }
    #[test]
    fn test_edge_cases() {
        // Test with special characters in model names
        let spec = ModelSpecifier::parse("openai/gpt-4-turbo-preview").unwrap();
        assert_eq!(spec.model, "gpt-4-turbo-preview");

        // Test with numbers and hyphens
        let spec = ModelSpecifier::parse("anthropic/claude-3-opus-20240229").unwrap();
        assert_eq!(spec.model, "claude-3-opus-20240229");
        assert_eq!(spec.provider, Some("anthropic".to_string()));
    }
    #[test]
    fn test_clone_and_eq() {
        let spec1 = ModelSpecifier::with_provider("openai", "gpt-4");
        let spec2 = spec1.clone();
        assert_eq!(spec1, spec2);

        let spec3 = ModelSpecifier::with_provider("anthropic", "claude-3");
        assert_ne!(spec1, spec3);
    }

    // Backend-related tests (Phase 11)
    #[test]
    fn test_parse_model_with_backend() {
        let spec = ModelSpecifier::parse("llama3.1:8b@ollama").unwrap();
        assert_eq!(spec.model, "llama3.1:8b");
        assert_eq!(spec.provider, None);
        assert_eq!(spec.backend, Some("ollama".to_string()));
        assert!(!spec.has_provider());
        assert!(spec.has_backend());
    }

    #[test]
    fn test_parse_provider_model_with_backend() {
        let spec = ModelSpecifier::parse("local/llama3.1:8b@ollama").unwrap();
        assert_eq!(spec.model, "llama3.1:8b");
        assert_eq!(spec.provider, Some("local".to_string()));
        assert_eq!(spec.backend, Some("ollama".to_string()));
        assert!(spec.has_provider());
        assert!(spec.has_backend());
    }

    #[test]
    fn test_parse_candle_backend() {
        let spec = ModelSpecifier::parse("llama3.1:8b@candle").unwrap();
        assert_eq!(spec.model, "llama3.1:8b");
        assert_eq!(spec.provider, None);
        assert_eq!(spec.backend, Some("candle".to_string()));
        assert!(spec.has_backend());
    }

    #[test]
    fn test_parse_local_without_backend() {
        let spec = ModelSpecifier::parse("local/llama3.1:8b").unwrap();
        assert_eq!(spec.model, "llama3.1:8b");
        assert_eq!(spec.provider, Some("local".to_string()));
        assert_eq!(spec.backend, None);
        assert!(spec.has_provider());
        assert!(!spec.has_backend());
    }

    #[test]
    fn test_parse_nested_provider_with_backend() {
        let spec = ModelSpecifier::parse("openrouter/local/model@backend").unwrap();
        assert_eq!(spec.model, "model");
        assert_eq!(spec.provider, Some("openrouter/local".to_string()));
        assert_eq!(spec.backend, Some("backend".to_string()));
        assert!(spec.has_provider());
        assert!(spec.has_backend());
    }

    #[test]
    fn test_backend_or_default() {
        let spec1 = ModelSpecifier::parse("llama3.1:8b@ollama").unwrap();
        assert_eq!(spec1.backend_or_default("default"), "ollama");

        let spec2 = ModelSpecifier::parse("llama3.1:8b").unwrap();
        assert_eq!(spec2.backend_or_default("default"), "default");
    }

    #[test]
    fn test_display_with_backend() {
        let spec1 = ModelSpecifier::parse("local/llama3.1:8b@ollama").unwrap();
        assert_eq!(spec1.to_string(), "local/llama3.1:8b@ollama");

        let spec2 = ModelSpecifier::parse("llama3.1:8b@candle").unwrap();
        assert_eq!(spec2.to_string(), "llama3.1:8b@candle");

        let spec3 = ModelSpecifier::parse("local/model").unwrap();
        assert_eq!(spec3.to_string(), "local/model");
    }

    #[test]
    fn test_backend_backward_compatibility() {
        // Ensure existing tests still pass - no backend in old format
        let spec = ModelSpecifier::parse("openai/gpt-4").unwrap();
        assert_eq!(spec.backend, None);
        assert!(!spec.has_backend());

        let spec2 = ModelSpecifier::parse("gpt-4").unwrap();
        assert_eq!(spec2.backend, None);
    }

    #[test]
    fn test_serde_with_backend() {
        let mut spec = ModelSpecifier::with_provider("local", "llama3.1:8b");
        spec.backend = Some("ollama".to_string());

        let serialized = serde_json::to_string(&spec).unwrap();
        let deserialized: ModelSpecifier = serde_json::from_str(&serialized).unwrap();

        assert_eq!(spec, deserialized);
        assert_eq!(deserialized.backend, Some("ollama".to_string()));
    }
}
