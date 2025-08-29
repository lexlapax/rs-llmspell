//! Provider integration types for embedding generation

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use llmspell_providers::abstraction::ProviderInstance;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait for providers that support embedding generation
///
/// This extends the base `ProviderInstance` trait to add embedding-specific functionality,
/// allowing providers like `OpenAI`, Google, and Cohere to generate vector embeddings.
#[async_trait]
pub trait EmbeddingProvider: ProviderInstance {
    /// Generate embeddings for text
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LLMSpellError>;

    /// Get the number of dimensions in the embeddings
    fn embedding_dimensions(&self) -> usize;

    /// Check if this provider supports dimension reduction (e.g., `OpenAI`'s Matryoshka)
    fn supports_dimension_reduction(&self) -> bool {
        false
    }

    /// Configure output dimensions if supported
    ///
    /// # Errors
    ///
    /// Returns an error if dimension configuration is not supported
    fn set_embedding_dimensions(&mut self, _dims: usize) -> Result<(), LLMSpellError> {
        Err(LLMSpellError::Provider {
            message: "Dimension configuration not supported".to_string(),
            provider: Some(self.name().to_string()),
            source: None,
        })
    }

    /// Get the name of the embedding model
    fn embedding_model(&self) -> Option<&str>;

    /// Get estimated cost per token for embeddings (in USD)
    fn embedding_cost_per_token(&self) -> Option<f64> {
        None
    }
}

/// Configuration for embedding providers
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmbeddingProviderConfig {
    /// Type of embedding provider
    pub provider_type: EmbeddingProviderType,

    /// Model name or identifier
    pub model: String,

    /// Optional fixed dimensions (None = use model default)
    pub dimensions: Option<usize>,

    /// Environment variable containing API key
    pub api_key_env: Option<String>,

    /// Optional base URL override
    pub base_url: Option<String>,

    /// Maximum batch size for embedding requests
    pub max_batch_size: usize,

    /// Additional provider-specific configuration
    pub custom_config: HashMap<String, serde_json::Value>,
}

impl Default for EmbeddingProviderConfig {
    fn default() -> Self {
        Self {
            provider_type: EmbeddingProviderType::OpenAI,
            model: "text-embedding-3-small".to_string(),
            dimensions: None,
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            base_url: None,
            max_batch_size: 100,
            custom_config: HashMap::new(),
        }
    }
}

/// Supported embedding provider types
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub enum EmbeddingProviderType {
    /// `OpenAI` embeddings (text-embedding-3-large/small, ada-002)
    OpenAI,

    /// Google Vertex AI embeddings (text-embedding-004, gecko)
    Google,

    /// Cohere embeddings (embed-v3-english, embed-v3-multilingual)
    Cohere,

    /// Voyage AI embeddings (voyage-2, voyage-large-2, voyage-code-2)
    VoyageAI,

    /// AWS Bedrock embeddings
    AWSBedrock,

    /// `HuggingFace` models (BGE-M3, E5, etc) via local inference
    HuggingFace,

    /// `FastEmbed` ONNX-optimized models
    FastEmbed,

    /// Custom provider implementation
    Custom(String),
}

/// Generic embedding model trait
#[async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Generate embeddings for input texts
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the number of dimensions
    fn dimensions(&self) -> usize;

    /// Get model identifier
    fn model_id(&self) -> &str;

    /// Check if model supports variable dimensions
    fn supports_dimension_reduction(&self) -> bool {
        false
    }

    /// Set output dimensions if supported
    ///
    /// # Errors
    ///
    /// Returns an error if dimension configuration is not supported
    fn set_dimensions(&mut self, _dims: usize) -> Result<()> {
        anyhow::bail!("Dimension configuration not supported")
    }

    /// Get cost per token if known
    fn cost_per_token(&self) -> Option<f64> {
        None
    }
}

/// Token-level embeddings for late interaction models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenEmbeddings {
    /// Token IDs from tokenization
    pub token_ids: Vec<u32>,

    /// One embedding vector per token
    pub embeddings: Vec<Vec<f32>>,

    /// Dimension of each embedding vector
    pub dimensions: usize,

    /// Original text (optional)
    pub text: Option<String>,
}

impl TokenEmbeddings {
    /// Create new token embeddings
    #[must_use]
    pub const fn new(token_ids: Vec<u32>, embeddings: Vec<Vec<f32>>, dimensions: usize) -> Self {
        Self {
            token_ids,
            embeddings,
            dimensions,
            text: None,
        }
    }

    /// Add original text
    #[must_use]
    pub fn with_text(mut self, text: String) -> Self {
        self.text = Some(text);
        self
    }

    /// Get the number of tokens
    #[must_use]
    pub const fn num_tokens(&self) -> usize {
        self.token_ids.len()
    }
}

/// Late interaction model trait (e.g., `ColBERT`)
///
/// These models generate token-level embeddings and compute similarity
/// using late interaction (`MaxSim`) between query and document tokens.
#[async_trait]
pub trait LateInteractionModel: EmbeddingModel {
    /// Generate token-level embeddings
    async fn embed_tokens(&self, texts: &[String]) -> Result<Vec<TokenEmbeddings>>;

    /// Compute late interaction score between query and document
    ///
    /// This typically uses `MaxSim`: for each query token, find the maximum
    /// similarity with any document token, then sum these maxima.
    fn late_interaction_score(&self, query: &TokenEmbeddings, doc: &TokenEmbeddings) -> f32 {
        // Default MaxSim implementation
        let mut total_score = 0.0;

        for q_emb in &query.embeddings {
            let mut max_sim = f32::NEG_INFINITY;

            for d_emb in &doc.embeddings {
                let sim = Self::cosine_similarity(q_emb, d_emb);
                if sim > max_sim {
                    max_sim = sim;
                }
            }

            total_score += max_sim;
        }

        // Normalize by query length
        #[allow(clippy::cast_precision_loss)]
        {
            total_score / query.embeddings.len() as f32
        }
    }

    /// Compute cosine similarity between two vectors
    #[must_use]
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        let mut dot = 0.0;
        let mut norm_a = 0.0;
        let mut norm_b = 0.0;

        for i in 0..a.len().min(b.len()) {
            dot += a[i] * b[i];
            norm_a += a[i] * a[i];
            norm_b += b[i] * b[i];
        }

        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }

        dot / (norm_a.sqrt() * norm_b.sqrt())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_embeddings_builder() {
        let embeddings = TokenEmbeddings::new(
            vec![1, 2, 3],
            vec![vec![0.1, 0.2], vec![0.3, 0.4], vec![0.5, 0.6]],
            2,
        )
        .with_text("test text".to_string());

        assert_eq!(embeddings.num_tokens(), 3);
        assert_eq!(embeddings.dimensions, 2);
        assert_eq!(embeddings.text, Some("test text".to_string()));
    }

    #[test]
    fn test_embedding_provider_config_default() {
        let config = EmbeddingProviderConfig::default();
        assert_eq!(config.provider_type, EmbeddingProviderType::OpenAI);
        assert_eq!(config.model, "text-embedding-3-small");
        assert_eq!(config.max_batch_size, 100);
    }
}
