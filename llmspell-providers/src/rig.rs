//! ABOUTME: Rig provider implementation for LLM completions
//! ABOUTME: Wraps the rig-core crate to provide LLM capabilities

use crate::abstraction::{ProviderCapabilities, ProviderConfig, ProviderInstance};
use async_trait::async_trait;
use llmspell_core::{
    error::LLMSpellError,
    types::{AgentInput, AgentOutput, AgentStream},
};
use rig::{completion::CompletionModel, providers};
use serde_json::json;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;
use tracing::{debug, info, instrument, span, warn, Level};

/// Enum to hold different provider models
enum RigModel {
    OpenAI(providers::openai::CompletionModel),
    Anthropic(providers::anthropic::completion::CompletionModel),
    Cohere(providers::cohere::CompletionModel),
}

/// Rig provider implementation with cost tracking and tracing
pub struct RigProvider {
    config: ProviderConfig,
    capabilities: ProviderCapabilities,
    model: RigModel,
    max_tokens: u64,
    /// Total cost accrued by this provider instance
    total_cost: std::sync::atomic::AtomicU64,
    /// Total tokens used by this provider instance
    total_tokens: std::sync::atomic::AtomicU64,
    /// Total requests made by this provider instance
    total_requests: std::sync::atomic::AtomicU64,
}

impl RigProvider {
    /// Create a new Rig provider instance
    pub fn new(config: ProviderConfig) -> Result<Self, LLMSpellError> {
        // Create the appropriate model based on provider type
        let model = match config.provider_type.as_str() {
            "openai" => {
                let api_key =
                    config
                        .api_key
                        .as_ref()
                        .ok_or_else(|| LLMSpellError::Configuration {
                            message: "OpenAI API key required".to_string(),
                            source: None,
                        })?;

                let client = providers::openai::Client::new(api_key);
                let model = client.completion_model(&config.model);
                RigModel::OpenAI(model)
            }
            "anthropic" => {
                let api_key =
                    config
                        .api_key
                        .as_ref()
                        .ok_or_else(|| LLMSpellError::Configuration {
                            message: "Anthropic API key required".to_string(),
                            source: None,
                        })?;

                // Anthropic client requires more parameters
                let base_url = config
                    .endpoint
                    .as_deref()
                    .unwrap_or("https://api.anthropic.com");
                let version = "2023-06-01"; // Default API version

                let client = providers::anthropic::Client::new(api_key, base_url, None, version);
                let model = client.completion_model(&config.model);
                RigModel::Anthropic(model)
            }
            "cohere" => {
                let api_key =
                    config
                        .api_key
                        .as_ref()
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
                    message: format!("Unsupported provider type: {}", config.provider_type),
                    source: None,
                });
            }
        };

        // Set capabilities based on provider type and model
        let capabilities = ProviderCapabilities {
            supports_streaming: false, // Rig doesn't expose streaming yet
            supports_multimodal: matches!(config.provider_type.as_str(), "openai" | "anthropic"),
            max_context_tokens: Some(match config.provider_type.as_str() {
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
            custom_features: HashMap::default(),
        };

        // Extract max_tokens from custom_config, with defaults based on provider
        let max_tokens = config
            .custom_config
            .get("max_tokens")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(match config.provider_type.as_str() {
                "anthropic" => 4096, // Anthropic requires max_tokens
                _ => 2048,           // Default for others
            });

        Ok(Self {
            config,
            capabilities,
            model,
            max_tokens,
            total_cost: AtomicU64::new(0),
            total_tokens: AtomicU64::new(0),
            total_requests: AtomicU64::new(0),
        })
    }

    /// Estimate cost in cents based on provider pricing
    /// These are rough estimates based on published pricing as of 2024
    fn estimate_cost_cents(&self, input_tokens: u64, output_tokens: u64) -> u64 {
        match self.config.provider_type.as_str() {
            "openai" => {
                match self.config.model.as_str() {
                    "gpt-4" | "gpt-4-0613" => {
                        // GPT-4: $0.03/1K input, $0.06/1K output
                        let input_cost = (input_tokens as f64 / 1000.0) * 3.0;
                        let output_cost = (output_tokens as f64 / 1000.0) * 6.0;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                    "gpt-3.5-turbo" | "gpt-3.5-turbo-0613" => {
                        // GPT-3.5: $0.001/1K input, $0.002/1K output
                        let input_cost = (input_tokens as f64 / 1000.0) * 0.1;
                        let output_cost = (output_tokens as f64 / 1000.0) * 0.2;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                    _ => {
                        // Default to GPT-3.5 pricing
                        let input_cost = (input_tokens as f64 / 1000.0) * 0.1;
                        let output_cost = (output_tokens as f64 / 1000.0) * 0.2;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                }
            }
            "anthropic" => {
                match self.config.model.as_str() {
                    "claude-3-opus-20240229" => {
                        // Claude 3 Opus: $0.015/1K input, $0.075/1K output
                        let input_cost = (input_tokens as f64 / 1000.0) * 1.5;
                        let output_cost = (output_tokens as f64 / 1000.0) * 7.5;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                    "claude-3-sonnet-20240229" => {
                        // Claude 3 Sonnet: $0.003/1K input, $0.015/1K output
                        let input_cost = (input_tokens as f64 / 1000.0) * 0.3;
                        let output_cost = (output_tokens as f64 / 1000.0) * 1.5;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                    _ => {
                        // Default to Claude 3 Sonnet pricing
                        let input_cost = (input_tokens as f64 / 1000.0) * 0.3;
                        let output_cost = (output_tokens as f64 / 1000.0) * 1.5;
                        ((input_cost + output_cost) * 100.0).round() as u64
                    }
                }
            }
            "cohere" => {
                // Cohere Command: $0.0015/1K input, $0.002/1K output (estimate)
                let input_cost = (input_tokens as f64 / 1000.0) * 0.15;
                let output_cost = (output_tokens as f64 / 1000.0) * 0.2;
                ((input_cost + output_cost) * 100.0).round() as u64
            }
            _ => {
                // Unknown provider, use conservative estimate
                let input_cost = (input_tokens as f64 / 1000.0) * 0.1;
                let output_cost = (output_tokens as f64 / 1000.0) * 0.2;
                ((input_cost + output_cost) * 100.0).round() as u64
            }
        }
    }

    /// Get current total cost in cents
    pub fn total_cost_cents(&self) -> u64 {
        self.total_cost.load(Ordering::SeqCst)
    }

    /// Get current total tokens used
    pub fn total_tokens_used(&self) -> u64 {
        self.total_tokens.load(Ordering::SeqCst)
    }

    /// Get current total requests made
    pub fn total_requests_made(&self) -> u64 {
        self.total_requests.load(Ordering::SeqCst)
    }

    async fn execute_completion(&self, prompt: String) -> Result<String, LLMSpellError> {
        match &self.model {
            RigModel::OpenAI(model) => model
                .completion_request(&prompt)
                .max_tokens(self.max_tokens)
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
                }),
            RigModel::Anthropic(model) => model
                .completion_request(&prompt)
                .max_tokens(self.max_tokens)
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
                }),
            RigModel::Cohere(model) => model
                .completion_request(&prompt)
                .max_tokens(self.max_tokens)
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
                }),
        }
    }
}

#[async_trait]
impl ProviderInstance for RigProvider {
    fn capabilities(&self) -> &ProviderCapabilities {
        &self.capabilities
    }

    #[instrument(level = "info", skip(self, input), fields(
        provider_type = %self.config.provider_type,
        model = %self.config.model,
        input_length = tracing::field::Empty,
        output_length = tracing::field::Empty,
        tokens_used = tracing::field::Empty,
        estimated_cost_cents = tracing::field::Empty,
        request_duration_ms = tracing::field::Empty,
        total_requests = tracing::field::Empty
    ))]
    async fn complete(&self, input: &AgentInput) -> Result<AgentOutput, LLMSpellError> {
        let start_time = Instant::now();
        let span = span!(Level::INFO, "provider_completion");
        let _enter = span.enter();

        // Record input metrics
        let input_length = input.text.len();
        span.record("input_length", input_length);
        info!(
            "Starting LLM completion request with {} character input",
            input_length
        );

        // Build the prompt
        let mut prompt = input.text.clone();

        // Add context if available
        if let Some(context) = &input.context {
            // Add context data as prefix
            if !context.data.is_empty() {
                let context_text = context
                    .data
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect::<Vec<_>>()
                    .join("\n");

                prompt = format!("{}\n\n{}", context_text, prompt);
                debug!("Added {} context items to prompt", context.data.len());
            }
        }

        debug!("Final prompt length: {} characters", prompt.len());

        // Execute the completion with error tracking
        let output_text = match self.execute_completion(prompt).await {
            Ok(text) => {
                info!("LLM completion succeeded");
                text
            }
            Err(e) => {
                warn!("LLM completion failed: {}", e);
                return Err(e);
            }
        };

        // Record output metrics
        let output_length = output_text.len();
        span.record("output_length", output_length);

        // Estimate tokens and cost (rough approximation)
        let estimated_input_tokens = (input_length as f64 / 4.0).ceil() as u64; // ~4 chars per token
        let estimated_output_tokens = (output_length as f64 / 4.0).ceil() as u64;
        let total_tokens = estimated_input_tokens + estimated_output_tokens;

        // Estimate cost in cents based on provider type and model
        let estimated_cost_cents =
            self.estimate_cost_cents(estimated_input_tokens, estimated_output_tokens);

        // Record metrics
        span.record("tokens_used", total_tokens);
        span.record("estimated_cost_cents", estimated_cost_cents);

        // Update provider-level metrics
        let request_count = self.total_requests.fetch_add(1, Ordering::SeqCst) + 1;
        self.total_tokens.fetch_add(total_tokens, Ordering::SeqCst);
        self.total_cost
            .fetch_add(estimated_cost_cents, Ordering::SeqCst);

        span.record("total_requests", request_count);

        // Record timing
        let duration = start_time.elapsed();
        span.record("request_duration_ms", duration.as_millis() as u64);

        info!(
            "Completion finished: {} -> {} chars, ~{} tokens, ~{:.2}¢, {}ms",
            input_length,
            output_length,
            total_tokens,
            estimated_cost_cents as f64 / 100.0,
            duration.as_millis()
        );

        // Build the output
        let mut output = AgentOutput::text(output_text);

        // Add provider metadata with cost tracking
        output.metadata.model = Some(self.config.model.clone());
        output
            .metadata
            .extra
            .insert("provider".to_string(), json!(self.config.name));
        output
            .metadata
            .extra
            .insert("estimated_tokens".to_string(), json!(total_tokens));
        output.metadata.extra.insert(
            "estimated_cost_cents".to_string(),
            json!(estimated_cost_cents),
        );
        output
            .metadata
            .extra
            .insert("duration_ms".to_string(), json!(duration.as_millis()));
        output
            .metadata
            .extra
            .insert("provider_total_requests".to_string(), json!(request_count));

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

    #[allow(clippy::misnamed_getters)]
    fn name(&self) -> &str {
        // Return the provider type for now, but consider using instance_name() for hierarchical naming
        &self.config.provider_type
    }

    fn model(&self) -> &str {
        &self.config.model
    }
}

/// Factory function for creating Rig providers
pub fn create_rig_provider(
    config: ProviderConfig,
) -> Result<Box<dyn ProviderInstance>, LLMSpellError> {
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
