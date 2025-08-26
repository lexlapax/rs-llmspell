//! Factory for creating embedding providers

use super::local::LocalEmbedding;
use super::openai::OpenAIEmbedding;
use super::provider::{EmbeddingModel, EmbeddingProviderConfig, EmbeddingProviderType};
use anyhow::Result;
use std::sync::Arc;

/// Factory for creating embedding model instances
#[derive(Debug, Clone)]
pub struct EmbeddingFactory {
    config: EmbeddingProviderConfig,

    /// Cost tracking accumulator
    total_tokens_used: Arc<std::sync::atomic::AtomicUsize>,
}

impl EmbeddingFactory {
    /// Create a new embedding factory
    #[must_use]
    pub fn new(config: EmbeddingProviderConfig) -> Self {
        Self {
            config,
            total_tokens_used: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        }
    }

    /// Create an embedding model instance
    ///
    /// Routes to the appropriate provider based on configuration:
    /// - `OpenAI`: text-embedding-3-small/large, ada-002
    /// - Local: BGE-M3, E5 models (mock implementation for now)
    /// - Others: Return error (not yet implemented)
    ///
    /// # Errors
    ///
    /// Returns an error if the provider is not supported or configuration is invalid
    pub fn create_model(&self) -> Result<Arc<dyn EmbeddingModel>> {
        match &self.config.provider_type {
            EmbeddingProviderType::OpenAI => {
                let model = OpenAIEmbedding::new(&self.config)?;
                Ok(Arc::new(model) as Arc<dyn EmbeddingModel>)
            }

            EmbeddingProviderType::HuggingFace | EmbeddingProviderType::FastEmbed => {
                // Use local mock implementation for now
                let model = match self.config.model.as_str() {
                    "BAAI/bge-m3" => LocalEmbedding::bge_m3(),
                    "intfloat/e5-large-v2" => LocalEmbedding::e5_large(),
                    "intfloat/multilingual-e5-large" => LocalEmbedding::multilingual_e5(),
                    _ => LocalEmbedding::new(
                        &self.config.model,
                        self.config.dimensions.unwrap_or(1024),
                    ),
                };

                // Apply dimension configuration if specified
                let mut model = model.with_deterministic(true); // Deterministic for testing
                if let Some(dims) = self.config.dimensions {
                    if model.supports_dimension_reduction() {
                        model.set_dimensions(dims)?;
                    }
                }

                Ok(Arc::new(model) as Arc<dyn EmbeddingModel>)
            }

            EmbeddingProviderType::Google => {
                anyhow::bail!("Google Vertex AI embeddings not yet implemented")
            }

            EmbeddingProviderType::Cohere => {
                anyhow::bail!("Cohere embeddings not yet implemented")
            }

            EmbeddingProviderType::VoyageAI => {
                anyhow::bail!("Voyage AI embeddings not yet implemented")
            }

            EmbeddingProviderType::AWSBedrock => {
                anyhow::bail!("AWS Bedrock embeddings not yet implemented")
            }

            EmbeddingProviderType::Custom(name) => {
                anyhow::bail!("Custom provider '{}' not yet implemented", name)
            }
        }
    }

    /// Get total tokens used across all embedding calls
    #[must_use]
    pub fn total_tokens_used(&self) -> usize {
        self.total_tokens_used
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Estimate total cost based on tokens used
    /// Calculate the estimated cost based on token usage
    ///
    /// # Errors
    ///
    /// Returns an error if model creation fails
    pub fn estimated_cost(&self) -> Result<f64> {
        let model = self.create_model()?;
        let tokens = self.total_tokens_used();
        let cost_per_token = model.cost_per_token().unwrap_or(0.0);
        #[allow(clippy::cast_precision_loss)]
        Ok(tokens as f64 * cost_per_token)
    }

    /// Reset cost tracking
    pub fn reset_tracking(&self) {
        self.total_tokens_used
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

/// Builder for embedding factory
#[derive(Debug)]
pub struct EmbeddingFactoryBuilder {
    config: EmbeddingProviderConfig,
}

impl EmbeddingFactoryBuilder {
    /// Create a new builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: EmbeddingProviderConfig::default(),
        }
    }

    /// Set the provider configuration
    #[must_use]
    pub fn with_config(mut self, config: EmbeddingProviderConfig) -> Self {
        self.config = config;
        self
    }

    /// Build the factory
    #[must_use]
    pub fn build(self) -> EmbeddingFactory {
        EmbeddingFactory::new(self.config)
    }
}

impl Default for EmbeddingFactoryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_creates_openai_model() {
        std::env::set_var("TEST_OPENAI_KEY", "test_key");

        let config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::OpenAI,
            model: "text-embedding-3-small".to_string(),
            api_key_env: Some("TEST_OPENAI_KEY".to_string()),
            ..Default::default()
        };

        let factory = EmbeddingFactory::new(config);
        let model = factory.create_model().unwrap();

        assert_eq!(model.model_id(), "text-embedding-3-small");
        assert_eq!(model.dimensions(), 1536);
        assert!(model.supports_dimension_reduction());

        std::env::remove_var("TEST_OPENAI_KEY");
    }

    #[tokio::test]
    async fn test_factory_creates_local_model() {
        let config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::HuggingFace,
            model: "BAAI/bge-m3".to_string(),
            dimensions: Some(512),
            ..Default::default()
        };

        let factory = EmbeddingFactory::new(config);
        let model = factory.create_model().unwrap();

        assert_eq!(model.model_id(), "BAAI/bge-m3");
        assert_eq!(model.dimensions(), 512);
        assert!(model.supports_dimension_reduction());
    }

    #[tokio::test]
    async fn test_factory_handles_unsupported_provider() {
        let config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::Google,
            model: "text-embedding-004".to_string(),
            ..Default::default()
        };

        let factory = EmbeddingFactory::new(config);
        let result = factory.create_model();

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("not yet implemented"));
    }

    #[tokio::test]
    async fn test_cost_tracking() {
        let config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::HuggingFace,
            model: "BAAI/bge-m3".to_string(),
            ..Default::default()
        };

        let factory = EmbeddingFactory::new(config);

        // Initially no tokens used
        assert_eq!(factory.total_tokens_used(), 0);

        // Local models have zero cost
        let cost = factory.estimated_cost().unwrap();
        assert!((cost - 0.0).abs() < f64::EPSILON);

        // Reset should work
        factory.reset_tracking();
        assert_eq!(factory.total_tokens_used(), 0);
    }

    #[test]
    fn test_builder_pattern() {
        let factory = EmbeddingFactoryBuilder::new()
            .with_config(EmbeddingProviderConfig {
                provider_type: EmbeddingProviderType::FastEmbed,
                model: "e5-small".to_string(),
                dimensions: Some(384),
                ..Default::default()
            })
            .build();

        assert_eq!(
            factory.config.provider_type,
            EmbeddingProviderType::FastEmbed
        );
        assert_eq!(factory.config.model, "e5-small");
        assert_eq!(factory.config.dimensions, Some(384));
    }
}
