//! OpenAI embedding provider implementation

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::error::LLMSpellError;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;

use super::provider::{EmbeddingModel, EmbeddingProviderConfig};

/// OpenAI embedding model implementation
#[derive(Debug)]
pub struct OpenAIEmbedding {
    /// Model name (e.g., text-embedding-3-small, text-embedding-3-large)
    model: String,

    /// API key
    api_key: String,

    /// HTTP client
    client: Client,

    /// Base URL
    base_url: String,

    /// Dimensions (optional, for text-embedding-3-* models)
    dimensions: Option<usize>,

    /// Default dimensions for the model
    default_dimensions: usize,

    /// Cost per 1K tokens in USD
    cost_per_1k_tokens: f64,
}

impl OpenAIEmbedding {
    /// Create new OpenAI embedding provider
    pub fn new(config: &EmbeddingProviderConfig) -> Result<Self> {
        let api_key = if let Some(env_var) = &config.api_key_env {
            env::var(env_var).map_err(|_| LLMSpellError::Configuration {
                message: format!("API key environment variable '{}' not set", env_var),
                source: None,
            })?
        } else {
            return Err(LLMSpellError::Configuration {
                message: "API key environment variable not configured".to_string(),
                source: None,
            }
            .into());
        };

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        // Determine model defaults
        let (default_dimensions, cost_per_1k_tokens) = match config.model.as_str() {
            "text-embedding-3-small" => (1536, 0.00002),
            "text-embedding-3-large" => (3072, 0.00013),
            "text-embedding-ada-002" => (1536, 0.00010),
            _ => (1536, 0.00010), // Default to ada-002 pricing
        };

        Ok(Self {
            model: config.model.clone(),
            api_key,
            client: Client::new(),
            base_url,
            dimensions: config.dimensions,
            default_dimensions,
            cost_per_1k_tokens,
        })
    }

    /// Create from environment variables
    pub fn from_env(model: &str) -> Result<Self> {
        let config = EmbeddingProviderConfig {
            model: model.to_string(),
            api_key_env: Some("OPENAI_API_KEY".to_string()),
            ..Default::default()
        };
        Self::new(&config)
    }
}

#[async_trait]
impl EmbeddingModel for OpenAIEmbedding {
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let mut request = EmbeddingRequest {
            input: texts.to_vec(),
            model: self.model.clone(),
            dimensions: None,
            encoding_format: Some("float".to_string()),
            user: None,
        };

        // Add dimensions if configured and supported
        if let Some(dims) = self.dimensions {
            if self.model.starts_with("text-embedding-3-") {
                request.dimensions = Some(dims);
            }
        }

        let url = format!("{}/embeddings", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| LLMSpellError::Provider {
                message: format!("Failed to send embedding request: {}", e),
                provider: Some("OpenAI".to_string()),
                source: Some(Box::new(e)),
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LLMSpellError::Provider {
                message: format!("OpenAI API error ({}): {}", status, error_text),
                provider: Some("OpenAI".to_string()),
                source: None,
            }
            .into());
        }

        let result: EmbeddingResponse =
            response.json().await.map_err(|e| LLMSpellError::Provider {
                message: format!("Failed to parse embedding response: {}", e),
                provider: Some("OpenAI".to_string()),
                source: Some(Box::new(e)),
            })?;

        // Sort by index to ensure correct order
        let mut sorted_data = result.data;
        sorted_data.sort_by_key(|e| e.index);

        Ok(sorted_data.into_iter().map(|e| e.embedding).collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions.unwrap_or(self.default_dimensions)
    }

    fn model_id(&self) -> &str {
        &self.model
    }

    fn supports_dimension_reduction(&self) -> bool {
        self.model.starts_with("text-embedding-3-")
    }

    fn set_dimensions(&mut self, dims: usize) -> Result<()> {
        if !self.supports_dimension_reduction() {
            anyhow::bail!(
                "Model {} does not support dimension configuration",
                self.model
            );
        }

        // text-embedding-3-* models support dimensions from 256 to their max
        let max_dims = self.default_dimensions;
        if dims < 256 || dims > max_dims {
            anyhow::bail!(
                "Invalid dimensions {} for model {} (must be between 256 and {})",
                dims,
                self.model,
                max_dims
            );
        }

        self.dimensions = Some(dims);
        Ok(())
    }

    fn cost_per_token(&self) -> Option<f64> {
        Some(self.cost_per_1k_tokens / 1000.0)
    }
}

/// OpenAI embedding request structure
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    input: Vec<String>,
    model: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    encoding_format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user: Option<String>,
}

/// OpenAI embedding response structure
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    #[allow(dead_code)]
    model: String,
    #[allow(dead_code)]
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
    index: usize,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Usage {
    prompt_tokens: usize,
    total_tokens: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openai_embedding_creation() {
        let config = EmbeddingProviderConfig {
            model: "text-embedding-3-small".to_string(),
            api_key_env: Some("TEST_API_KEY".to_string()),
            dimensions: Some(512),
            ..Default::default()
        };

        // Set test environment variable
        env::set_var("TEST_API_KEY", "test_key");

        let embedding = OpenAIEmbedding::new(&config).unwrap();
        assert_eq!(embedding.model, "text-embedding-3-small");
        assert_eq!(embedding.dimensions, Some(512));
        assert_eq!(embedding.default_dimensions, 1536);
        assert!(embedding.supports_dimension_reduction());

        // Clean up
        env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_dimension_validation() {
        let config = EmbeddingProviderConfig {
            model: "text-embedding-3-large".to_string(),
            api_key_env: Some("TEST_API_KEY".to_string()),
            ..Default::default()
        };

        env::set_var("TEST_API_KEY", "test_key");

        let mut embedding = OpenAIEmbedding::new(&config).unwrap();

        // Valid dimension
        assert!(embedding.set_dimensions(1024).is_ok());
        assert_eq!(embedding.dimensions, Some(1024));

        // Invalid dimensions
        assert!(embedding.set_dimensions(100).is_err()); // Too small
        assert!(embedding.set_dimensions(4096).is_err()); // Too large

        env::remove_var("TEST_API_KEY");
    }

    #[test]
    fn test_ada_no_dimension_support() {
        let config = EmbeddingProviderConfig {
            model: "text-embedding-ada-002".to_string(),
            api_key_env: Some("TEST_API_KEY".to_string()),
            ..Default::default()
        };

        env::set_var("TEST_API_KEY", "test_key");

        let mut embedding = OpenAIEmbedding::new(&config).unwrap();
        assert!(!embedding.supports_dimension_reduction());
        assert!(embedding.set_dimensions(512).is_err());

        env::remove_var("TEST_API_KEY");
    }
}
