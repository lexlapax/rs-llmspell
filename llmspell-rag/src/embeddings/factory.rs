//! Factory for creating embedding providers

use super::provider::{EmbeddingModel, EmbeddingProviderConfig};
use anyhow::Result;
use std::sync::Arc;

/// Factory for creating embedding model instances
#[derive(Debug)]
pub struct EmbeddingFactory {
    #[allow(dead_code)]
    config: EmbeddingProviderConfig,
}

impl EmbeddingFactory {
    /// Create a new embedding factory
    #[must_use]
    pub const fn new(config: EmbeddingProviderConfig) -> Self {
        Self { config }
    }

    /// Create an embedding model instance
    ///
    /// This will eventually create the appropriate provider instance
    /// based on the configuration (`OpenAI`, Cohere, local models, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if model creation is not yet implemented
    #[allow(clippy::unused_async)]
    pub async fn create_model(&self) -> Result<Arc<dyn EmbeddingModel>> {
        // TODO: Implement actual provider creation based on config
        // For now, return an error indicating not implemented
        anyhow::bail!("Embedding model creation not yet implemented")
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
