//! Configuration types for RAG pipeline

use crate::{
    chunking::ChunkingConfig,
    embeddings::{CacheConfig, EmbeddingProviderConfig},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete RAG pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    /// Document ingestion configuration
    pub ingestion: IngestionConfig,

    /// Retrieval configuration
    pub retrieval: RetrievalConfig,

    /// Optional state scope for tenant isolation
    pub scope_prefix: Option<String>,

    /// Maximum concurrent operations
    pub max_concurrency: usize,

    /// Operation timeouts in seconds
    pub timeouts: TimeoutConfig,
}

impl Default for RAGConfig {
    fn default() -> Self {
        Self {
            ingestion: IngestionConfig::default(),
            retrieval: RetrievalConfig::default(),
            scope_prefix: None,
            max_concurrency: 10,
            timeouts: TimeoutConfig::default(),
        }
    }
}

/// Document ingestion configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestionConfig {
    /// Chunking strategy configuration
    pub chunking: ChunkingConfig,

    /// Embedding provider configuration
    pub embedding: EmbeddingProviderConfig,

    /// Embedding cache configuration
    pub cache: CacheConfig,

    /// Whether to store document text in metadata
    pub store_text: bool,

    /// Additional metadata to extract/compute
    pub metadata_extractors: Vec<String>,

    /// Whether to deduplicate chunks by content hash
    pub deduplicate: bool,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self {
            chunking: ChunkingConfig::default(),
            embedding: EmbeddingProviderConfig::default(),
            cache: CacheConfig::default(),
            store_text: true,
            metadata_extractors: Vec::new(),
            deduplicate: true,
        }
    }
}

/// Retrieval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Hybrid retrieval weights
    pub hybrid_weights: HybridWeights,

    /// Maximum number of results to return
    pub max_results: usize,

    /// Minimum similarity score threshold
    pub min_score: f32,

    /// Reranking configuration
    pub reranking: RerankingConfig,

    /// Whether to include chunk metadata in results
    pub include_metadata: bool,

    /// Whether to include original text in results
    pub include_text: bool,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            hybrid_weights: HybridWeights::default(),
            max_results: 10,
            min_score: 0.0,
            reranking: RerankingConfig::default(),
            include_metadata: true,
            include_text: true,
        }
    }
}

/// Weights for hybrid retrieval methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridWeights {
    /// Vector similarity weight (0.0-1.0)
    pub vector: f32,

    /// Keyword/BM25 weight (0.0-1.0)  
    pub keyword: f32,

    /// Metadata filter boost (0.0-1.0)
    pub metadata: f32,

    /// Recency boost weight (0.0-1.0)
    pub recency: f32,
}

impl Default for HybridWeights {
    fn default() -> Self {
        Self {
            vector: 0.7,
            keyword: 0.2,
            metadata: 0.05,
            recency: 0.05,
        }
    }
}

impl HybridWeights {
    /// Validate weights are reasonable
    ///
    /// # Errors
    ///
    /// Returns an error if weights are not between 0.0 and 1.0 or don't sum to a positive value
    pub fn validate(&self) -> Result<(), String> {
        let weights = [self.vector, self.keyword, self.metadata, self.recency];

        // Check all weights are non-negative
        for (i, &weight) in weights.iter().enumerate() {
            if !(0.0..=1.0).contains(&weight) {
                return Err(format!(
                    "Weight {} ({}) must be between 0.0 and 1.0",
                    ["vector", "keyword", "metadata", "recency"][i],
                    weight
                ));
            }
        }

        // Check weights sum to reasonable total (allow some flexibility)
        let total: f32 = weights.iter().sum();
        if !(0.1..=2.0).contains(&total) {
            return Err(format!(
                "Total weight sum ({total:.2}) should be between 0.1 and 2.0"
            ));
        }

        Ok(())
    }

    /// Normalize weights to sum to 1.0
    #[must_use]
    pub fn normalized(mut self) -> Self {
        let total = self.vector + self.keyword + self.metadata + self.recency;
        if total > 0.0 {
            self.vector /= total;
            self.keyword /= total;
            self.metadata /= total;
            self.recency /= total;
        }
        self
    }
}

/// Reranking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankingConfig {
    /// Whether to enable reranking
    pub enabled: bool,

    /// Reranking strategy
    pub strategy: RerankingStrategy,

    /// Number of candidates to rerank (should be > final results)
    pub candidates: usize,

    /// Diversity parameter for MMR (0.0 = relevance only, 1.0 = diversity only)
    pub diversity_lambda: f32,
}

impl Default for RerankingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            strategy: RerankingStrategy::MMR,
            candidates: 50,
            diversity_lambda: 0.3,
        }
    }
}

/// Reranking strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RerankingStrategy {
    /// No reranking, use original scores
    None,

    /// Maximal Marginal Relevance for diversity
    MMR,

    /// Score-based reranking
    Score,

    /// Custom reranking (placeholder for future ML models)
    Custom(String),
}

/// Timeout configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Embedding generation timeout (seconds)
    pub embedding: u64,

    /// Vector search timeout (seconds)
    pub search: u64,

    /// Total pipeline operation timeout (seconds)
    pub pipeline: u64,
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            embedding: 30,
            search: 10,
            pipeline: 60,
        }
    }
}

/// Query-time configuration overrides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryConfig {
    /// Override max results for this query
    pub max_results: Option<usize>,

    /// Override minimum score threshold
    pub min_score: Option<f32>,

    /// Override hybrid weights
    pub hybrid_weights: Option<HybridWeights>,

    /// Additional metadata filters
    pub metadata_filters: HashMap<String, serde_json::Value>,

    /// Override reranking for this query
    pub reranking: Option<RerankingConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_weights_validation() {
        let valid_weights = HybridWeights::default();
        assert!(valid_weights.validate().is_ok());

        let invalid_weights = HybridWeights {
            vector: -0.1,
            ..HybridWeights::default()
        };
        assert!(invalid_weights.validate().is_err());

        let zero_weights = HybridWeights {
            vector: 0.0,
            keyword: 0.0,
            metadata: 0.0,
            recency: 0.0,
        };
        assert!(zero_weights.validate().is_err());
    }

    #[test]
    fn test_hybrid_weights_normalization() {
        let weights = HybridWeights {
            vector: 2.0,
            keyword: 1.0,
            metadata: 0.5,
            recency: 0.5,
        };

        let normalized = weights.normalized();
        let sum = normalized.vector + normalized.keyword + normalized.metadata + normalized.recency;
        assert!((sum - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_config_serialization() {
        let config = RAGConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: RAGConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.max_concurrency, deserialized.max_concurrency);
    }
}
