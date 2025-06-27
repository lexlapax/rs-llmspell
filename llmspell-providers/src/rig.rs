//! ABOUTME: Rig provider implementation for LLM completions
//! ABOUTME: Wraps the rig-core crate to provide LLM capabilities

use crate::abstraction::{ProviderCapabilities, ProviderConfig, ProviderInstance};
use async_trait::async_trait;
use llmspell_core::{
    error::LLMSpellError,
    types::{AgentInput, AgentOutput, AgentStream},
};
use rig::{
    completion::CompletionModel,
    providers,
};
use serde_json::json;

/// Enum to hold different provider models
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
}

/// Rig provider implementation
pub struct RigProvider {
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    model: RigModel,
}

impl RigProvider {
    /// Create a new Rig provider instance
    pub fn new(config: ProviderConfig) -> Result<Self, LLMSpellError> {
        // Create the appropriate model based on provider name
        let model = match config.name.as_str() {
            "openai" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| LLMSpellError::Configuration {
                        message: "OpenAI API key required".to_string(),
                        source: None,
                    })?;
                
                let client = providers::openai::Client::new(api_key);
                let model = client.completion_model(&config.model);
                RigModel::OpenAI(model)
            }
            "anthropic" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| LLMSpellError::Configuration {
                        message: "Anthropic API key required".to_string(),
                        source: None,
                    })?;
                
                // Anthropic client requires more parameters
                let base_url = config.endpoint.as_deref()
                    .unwrap_or("https://api.anthropic.com");
                let version = "2023-06-01"; // Default API version
                
                let client = providers::anthropic::Client::new(api_key, base_url, None, version);
                let model = client.completion_model(&config.model);
                RigModel::Anthropic(model)
            }
            "cohere" => {
                let api_key = config.api_key.as_ref()
                    .ok_or_else(|| LLMSpellError::Configuration {
                        message: "Cohere API key required".to_string(),
                        source: None,
                    })?;
                
                let client = providers::cohere::Client::new(api_key);
                let model = client.completion_model(&config.model);
                RigModel::Cohere(model)
            }
            _ => {
                return Err(LLMSpellError::Configuration {
                    message: format!("Unsupported provider: {}", config.name),
                    source: None,
                });
            }
        };
        
        // Set capabilities based on provider and model
        let capabilities = ProviderCapabilities {
            supports_streaming: false, // Rig doesn't expose streaming yet
            supports_multimodal: matches!(config.name.as_str(), "openai" | "anthropic"),
            max_context_tokens: Some(match config.name.as_str() {
                "openai" => match config.model.as_str() {
                    "gpt-4" | "gpt-4-turbo" => 128000,
                    "gpt-3.5-turbo" => 16384,
                    _ => 8192,
                },
                "anthropic" => match config.model.as_str() {
                    "claude-3-opus" | "claude-3-sonnet" => 200000,
                    "claude-2.1" => 100000,
                    _ => 100000,
                },
                "cohere" => 4096,
                _ => 4096,
            }),
            max_output_tokens: Some(4096),
            available_models: vec![config.model.clone()],
            custom_features: Default::default(),
        };
        
        Ok(Self {
            config,
            capabilities,
            model,
        })
    }
    
    async fn execute_completion(&self, prompt: String) -> Result<String, LLMSpellError> {
        match &self.model {
            RigModel::OpenAI(model) => {
                model.completion_request(&prompt)
                    .send()
                    .await
                    .map_err(|e| LLMSpellError::Provider {
                        message: format!("OpenAI completion failed: {}", e),
                        provider: Some(self.config.name.clone()),
                        source: None,
                    })
                    .and_then(|response| match response.choice {
                        rig::completion::ModelChoice::Message(text) => Ok(text),
                        rig::completion::ModelChoice::ToolCall(name, _params) => {
                            Err(LLMSpellError::Provider {
                                message: format!("Unexpected tool call response: {}", name),
                                provider: Some(self.config.name.clone()),
                                source: None,
                            })
                        }
                    })
            }
            RigModel::Anthropic(model) => {
                model.completion_request(&prompt)
                    .send()
                    .await
                    .map_err(|e| LLMSpellError::Provider {
                        message: format!("Anthropic completion failed: {}", e),
                        provider: Some(self.config.name.clone()),
                        source: None,
                    })
                    .and_then(|response| match response.choice {
                        rig::completion::ModelChoice::Message(text) => Ok(text),
                        rig::completion::ModelChoice::ToolCall(name, _params) => {
                            Err(LLMSpellError::Provider {
                                message: format!("Unexpected tool call response: {}", name),
                                provider: Some(self.config.name.clone()),
                                source: None,
                            })
                        }
                    })
            }
            RigModel::Cohere(model) => {
                model.completion_request(&prompt)
                    .send()
                    .await
                    .map_err(|e| LLMSpellError::Provider {
                        message: format!("Cohere completion failed: {}", e),
                        provider: Some(self.config.name.clone()),
                        source: None,
                    })
                    .and_then(|response| match response.choice {
                        rig::completion::ModelChoice::Message(text) => Ok(text),
                        rig::completion::ModelChoice::ToolCall(name, _params) => {
                            Err(LLMSpellError::Provider {
                                message: format!("Unexpected tool call response: {}", name),
                                provider: Some(self.config.name.clone()),
                                source: None,
                            })
                        }
                    })
            }
        }
    }
}

#[async_trait]
impl ProviderInstance for RigProvider {
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }
    
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput, LLMSpellError> {
        // Build the prompt
        let mut prompt = input.text.clone();
        
        // Add context if available
        if let Some(context) = &input.context {
            // Add context data as prefix
            if !context.data.is_empty() {
                let context_text = context.data.iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");
                
                prompt = format!("{}\n\n{}", context_text, prompt);
            }
        }
        
        // TODO: In a real implementation, we would handle parameters like max_tokens, temperature, etc.
        // For now, we'll use defaults since Rig's simple completion API doesn't expose these
        
        // Execute the completion
        let output_text = self.execute_completion(prompt).await?;
        
        // Build the output
        let mut output = AgentOutput::text(output_text);
        
        // Add provider metadata
        output.metadata.model = Some(self.config.model.clone());
        output.metadata.extra.insert("provider".to_string(), json!(self.config.name));
        
        // Note: Rig's simple completion API doesn't return usage information
        // In a real implementation, we might need to use more advanced APIs
        
        Ok(output)
    }
    
    async fn complete_streaming(&self, _input: &AgentInput) -> Result<AgentStream, LLMSpellError> {
        // Rig doesn't expose streaming yet, use default implementation
        Err(LLMSpellError::Provider {
            message: "Streaming not yet supported in Rig provider".to_string(),
            provider: Some(self.name().to_string()),
            source: None,
        })
    }
    
    async fn validate(&self) -> Result<(), LLMSpellError> {
        // Try a simple completion to validate the configuration
        let test_input = AgentInput::text("Say 'test'");
        
        match self.complete(&test_input).await {
            Ok(_) => Ok(()),
            Err(e) => Err(LLMSpellError::Configuration {
                message: format!("Provider validation failed: {}", e),
                source: Some(Box::new(e)),
            }),
        }
    }
    
    fn name(&self) -> &str {
        &self.config.name
    }
    
    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Factory function for creating Rig providers
pub fn create_rig_provider(config: ProviderConfig) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
    Ok(Box::new(RigProvider::new(config)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rig_provider_capabilities() {
        let config = ProviderConfig::new("openai", "gpt-4");
        
        // Note: This will fail without API key, but we can test the error handling
        match RigProvider::new(config) {
            Err(LLMSpellError::Configuration { message, .. }) => {
                assert!(message.contains("API key required"));
            }
            _ => panic!("Expected configuration error"),
        }
    }
    
    #[test]
    fn test_unsupported_provider() {
        let config = ProviderConfig::new("unsupported", "model");
        
        match RigProvider::new(config) {
            Err(LLMSpellError::Configuration { message, .. }) => {
                assert!(message.contains("Unsupported provider"));
            }
            _ => panic!("Expected configuration error"),
        }
    }
    
    #[test]
    fn test_provider_capabilities_settings() {
        let mut config = ProviderConfig::new("openai", "gpt-4");
        config.api_key = Some("test-key".to_string());
        
        // Create provider and check capabilities
        if let Ok(provider) = RigProvider::new(config) {
            let caps = provider.capabilities();
            assert!(!caps.supports_streaming); // Rig doesn't support streaming yet
            assert!(caps.supports_multimodal); // OpenAI supports multimodal
            assert_eq!(caps.max_context_tokens, Some(128000)); // GPT-4 context size
            assert_eq!(caps.max_output_tokens, Some(4096));
            assert_eq!(caps.available_models, vec!["gpt-4"]);
        }
    }
    
    #[test]
    fn test_anthropic_capabilities() {
        let mut config = ProviderConfig::new("anthropic", "claude-3-opus");
        config.api_key = Some("test-key".to_string());
        
        if let Ok(provider) = RigProvider::new(config) {
            let caps = provider.capabilities();
            assert!(caps.supports_multimodal); // Anthropic supports multimodal
            assert_eq!(caps.max_context_tokens, Some(200000)); // Claude 3 Opus context size
        }
    }
    
    #[test]
    fn test_cohere_capabilities() {
        let mut config = ProviderConfig::new("cohere", "command");
        config.api_key = Some("test-key".to_string());
        
        if let Ok(provider) = RigProvider::new(config) {
            let caps = provider.capabilities();
            assert!(!caps.supports_multimodal); // Cohere doesn't support multimodal
            assert_eq!(caps.max_context_tokens, Some(4096)); // Cohere context size
        }
    }
}