//! ABOUTME: Provider management for LLM providers accessible from scripts
//! ABOUTME: Manages provider configuration, capability detection, and access control

use llmspell_core::error::LLMSpellError;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

// Placeholder traits until llmspell-providers is implemented
#[async_trait]
pub trait Provider: Send + Sync {
    fn capabilities(&self) -> ProviderCapabilities;
    async fn complete(&self, prompt: &str) -> Result<String, LLMSpellError>;
}

#[derive(Debug, Clone)]
pub struct ProviderCapabilities {
    pub streaming: bool,
    pub multimodal: bool,
    pub tool_calling: bool,
    pub embeddings: bool,
}

/// Manages LLM providers for script access
pub struct ProviderManager {
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    default_provider: Arc<RwLock<Option<String>>>,
    config: ProviderManagerConfig,
}

impl ProviderManager {
    /// Create a new provider manager with configuration
    pub fn new(config: ProviderManagerConfig) -> Result<Self, LLMSpellError> {
        let manager = Self {
            providers: Arc::new(RwLock::new(HashMap::new())),
            default_provider: Arc::new(RwLock::new(config.default_provider.clone())),
            config,
        };
        
        // Initialize configured providers
        manager.initialize_providers()?;
        
        Ok(manager)
    }
    
    /// Initialize providers from configuration
    fn initialize_providers(&self) -> Result<(), LLMSpellError> {
        // This will be implemented when we have actual provider implementations
        // For now, just validate configuration
        if let Some(ref default) = self.config.default_provider {
            if !self.config.providers.contains_key(default) {
                return Err(LLMSpellError::Validation {
                    field: Some("default_provider".to_string()),
                    message: format!("Default provider '{}' not found in configuration", default),
                });
            }
        }
        Ok(())
    }
    
    /// Register a provider
    pub fn register_provider(&self, name: String, provider: Arc<dyn Provider>) -> Result<(), LLMSpellError> {
        let mut providers = self.providers.write().unwrap();
        if providers.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("provider_name".to_string()),
                message: format!("Provider '{}' already registered", name),
            });
        }
        providers.insert(name.clone(), provider);
        
        // If no default provider set, use the first one registered
        let mut default = self.default_provider.write().unwrap();
        if default.is_none() {
            *default = Some(name);
        }
        
        Ok(())
    }
    
    /// Get a provider by name
    pub fn get_provider(&self, name: &str) -> Option<Arc<dyn Provider>> {
        let providers = self.providers.read().unwrap();
        providers.get(name).cloned()
    }
    
    /// Get the default provider
    pub fn get_default_provider(&self) -> Option<Arc<dyn Provider>> {
        let default = self.default_provider.read().unwrap();
        if let Some(ref name) = *default {
            self.get_provider(name)
        } else {
            None
        }
    }
    
    /// Set the default provider
    pub fn set_default_provider(&self, name: String) -> Result<(), LLMSpellError> {
        let providers = self.providers.read().unwrap();
        if !providers.contains_key(&name) {
            return Err(LLMSpellError::Validation {
                field: Some("provider_name".to_string()),
                message: format!("Provider '{}' not found", name),
            });
        }
        
        let mut default = self.default_provider.write().unwrap();
        *default = Some(name);
        Ok(())
    }
    
    /// List all registered providers
    pub fn list_providers(&self) -> Vec<ProviderInfo> {
        let providers = self.providers.read().unwrap();
        providers.iter().map(|(name, provider)| {
            ProviderInfo {
                name: name.clone(),
                capabilities: provider.capabilities(),
            }
        }).collect()
    }
    
    /// Check if a provider supports a specific capability
    pub fn provider_supports(&self, provider_name: &str, capability: &str) -> bool {
        if let Some(provider) = self.get_provider(provider_name) {
            let caps = provider.capabilities();
            match capability {
                "streaming" => caps.streaming,
                "multimodal" => caps.multimodal,
                "tool_calling" => caps.tool_calling,
                "embeddings" => caps.embeddings,
                _ => false,
            }
        } else {
            false
        }
    }
}

/// Information about a registered provider
#[derive(Debug, Clone)]
pub struct ProviderInfo {
    pub name: String,
    pub capabilities: ProviderCapabilities,
}

/// Configuration for the provider manager
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProviderManagerConfig {
    /// Default provider to use
    pub default_provider: Option<String>,
    /// Provider-specific configurations
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for ProviderManagerConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            providers: HashMap::new(),
        }
    }
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
    
    #[test]
    fn test_provider_manager_creation() {
        let config = ProviderManagerConfig::default();
        let manager = ProviderManager::new(config).unwrap();
        assert!(manager.get_default_provider().is_none());
    }
    
    #[test]
    fn test_provider_config_validation() {
        let mut config = ProviderManagerConfig::default();
        config.default_provider = Some("nonexistent".to_string());
        
        let result = ProviderManager::new(config);
        assert!(result.is_err());
    }
}