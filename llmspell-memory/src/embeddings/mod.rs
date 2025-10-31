//! ABOUTME: Embedding generation integration with llmspell-rag providers
//!
//! This module integrates llmspell-rag's `EmbeddingProvider` trait into the memory system,
//! providing a foundation for optimization layers (caching, batching).

use llmspell_core::traits::embedding::EmbeddingProvider;
use std::sync::Arc;
use tracing::{debug, info};

pub mod cached;
pub use cached::{CachedEmbeddingService, CacheStats};

/// Wrapper for embedding provider integration
///
/// Provides a clean interface between memory components and RAG embedding providers.
/// Can be enhanced with caching, batching, and other optimizations.
#[derive(Clone)]
pub struct EmbeddingService {
    provider: Arc<dyn EmbeddingProvider>,
}

impl std::fmt::Debug for EmbeddingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbeddingService")
            .field("provider", &self.provider.name())
            .finish()
    }
}

impl EmbeddingService {
    /// Create new embedding service from provider
    pub fn new(provider: Arc<dyn EmbeddingProvider>) -> Self {
        info!(
            "Creating EmbeddingService with provider: {}, dimensions: {}",
            provider.name(),
            provider.embedding_dimensions()
        );
        Self { provider }
    }

    /// Generate embedding for single text
    ///
    /// Wraps the provider's batch API for convenience
    ///
    /// # Errors
    ///
    /// Returns error if embedding generation fails or provider is unavailable
    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>, crate::error::MemoryError> {
        debug!("Generating embedding for text (length: {})", text.len());

        let texts = vec![text.to_string()];
        let mut embeddings = self
            .provider
            .embed(&texts)
            .await
            .map_err(|e| crate::error::MemoryError::EmbeddingError(e.to_string()))?;

        embeddings
            .pop()
            .ok_or_else(|| crate::error::MemoryError::EmbeddingError("No embedding returned".to_string()))
    }

    /// Generate embeddings for multiple texts in batch
    ///
    /// Uses provider's native batching for optimal performance
    ///
    /// # Errors
    ///
    /// Returns error if embedding generation fails or provider is unavailable
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, crate::error::MemoryError> {
        info!("Generating batch of {} embeddings", texts.len());

        self.provider
            .embed(texts)
            .await
            .map_err(|e| crate::error::MemoryError::EmbeddingError(e.to_string()))
    }

    /// Get embedding dimensions
    #[must_use] 
    pub fn dimensions(&self) -> usize {
        self.provider.embedding_dimensions()
    }

    /// Get provider name
    #[must_use] 
    pub fn provider_name(&self) -> &str {
        self.provider.name()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use llmspell_core::error::LLMSpellError;

    /// Mock embedding provider for testing
    struct MockEmbeddingProvider;

    #[async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        fn name(&self) -> &str {
            "mock"
        }

        async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError> {
            Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
        }

        fn embedding_dimensions(&self) -> usize {
            3
        }

        fn supports_dimension_reduction(&self) -> bool {
            false
        }

        fn set_embedding_dimensions(&mut self, _dims: usize) -> Result<(), LLMSpellError> {
            Err(LLMSpellError::Provider {
                message: "Dimension configuration not supported".to_string(),
                provider: Some(self.name().to_string()),
                source: None,
            })
        }

        fn embedding_model(&self) -> Option<&str> {
            Some("mock-model")
        }

        fn embedding_cost_per_token(&self) -> Option<f64> {
            None
        }
    }

    #[tokio::test]
    async fn test_embed_single() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = EmbeddingService::new(provider);

        let embedding = service.embed_single("test").await.unwrap();
        assert_eq!(embedding.len(), 3);
        assert_eq!(embedding, vec![0.1, 0.2, 0.3]);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = EmbeddingService::new(provider);

        let texts = vec!["test1".to_string(), "test2".to_string()];
        let embeddings = service.embed_batch(&texts).await.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0], vec![0.1, 0.2, 0.3]);
        assert_eq!(embeddings[1], vec![0.1, 0.2, 0.3]);
    }

    #[test]
    fn test_dimensions() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = EmbeddingService::new(provider);
        assert_eq!(service.dimensions(), 3);
    }
}
