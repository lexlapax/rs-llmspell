//! Embedding generation traits
//!
//! Core trait for embedding generation providers, extracted from llmspell-rag
//! to avoid circular dependencies (kernel → memory → rag → kernel).

use crate::error::LLMSpellError;
use async_trait::async_trait;

/// Trait for providers that support embedding generation
///
/// Core embedding provider trait, independent of ProviderInstance to avoid circular dependencies.
/// Implementations in llmspell-rag can extend this to also implement ProviderInstance.
///
/// # Example
///
/// ```rust,ignore
/// use llmspell_core::traits::{EmbeddingProvider, ProviderInstance};
/// use async_trait::async_trait;
///
/// struct MyEmbeddingProvider;
///
/// impl ProviderInstance for MyEmbeddingProvider {
///     fn name(&self) -> &str {
///         "my-provider"
///     }
/// }
///
/// #[async_trait]
/// impl EmbeddingProvider for MyEmbeddingProvider {
///     async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError> {
///         // Generate embeddings
///         Ok(texts.iter().map(|_| vec![0.1, 0.2, 0.3]).collect())
///     }
///
///     fn embedding_dimensions(&self) -> usize {
///         3
///     }
///
///     fn embedding_model(&self) -> Option<&str> {
///         Some("my-model")
///     }
/// }
/// ```
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Get provider name (similar to ProviderInstance::name())
    ///
    /// Returns a unique identifier for this provider (e.g., "openai", "ollama").
    fn name(&self) -> &str;

    /// Generate embeddings for multiple texts in batch
    ///
    /// This is the primary method for embedding generation, supporting batch processing
    /// for optimal performance.
    ///
    /// # Arguments
    ///
    /// * `texts` - Slice of strings to generate embeddings for
    ///
    /// # Returns
    ///
    /// Vector of embeddings, where each embedding is a `Vec<f32>` of fixed dimensions
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - Provider is unavailable or rate-limited
    /// - Text exceeds provider's token limits
    /// - Authentication fails
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError>;

    /// Get the number of dimensions in the embeddings
    ///
    /// Returns the fixed dimensionality of embeddings produced by this provider.
    /// Common values: 384 (MiniLM), 768 (BGE-large), 1536 (OpenAI ada-002), 3072 (text-embedding-3-large).
    fn embedding_dimensions(&self) -> usize;

    /// Check if this provider supports dimension reduction
    ///
    /// Some providers (e.g., OpenAI's Matryoshka embeddings) support configurable output dimensions.
    /// Default implementation returns `false`.
    fn supports_dimension_reduction(&self) -> bool {
        false
    }

    /// Configure output dimensions if supported
    ///
    /// Only works if `supports_dimension_reduction()` returns `true`.
    ///
    /// # Arguments
    ///
    /// * `dims` - Desired output dimensions (must be ≤ model's native dimensions)
    ///
    /// # Errors
    ///
    /// Returns error if dimension configuration is not supported by this provider
    fn set_embedding_dimensions(&mut self, _dims: usize) -> Result<(), LLMSpellError> {
        Err(LLMSpellError::Provider {
            message: "Dimension configuration not supported".to_string(),
            provider: Some(self.name().to_string()),
            source: None,
        })
    }

    /// Get the name of the embedding model
    ///
    /// Returns the specific model identifier (e.g., "text-embedding-3-small", "bge-large-en-v1.5").
    /// Returns `None` if model name is unavailable or not applicable.
    fn embedding_model(&self) -> Option<&str>;

    /// Get estimated cost per token for embeddings (in USD)
    ///
    /// Returns `None` if cost information is unavailable or not applicable (e.g., local models).
    /// Default implementation returns `None`.
    fn embedding_cost_per_token(&self) -> Option<f64> {
        None
    }
}
