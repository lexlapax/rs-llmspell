//! Hybrid retrieval traits combining vector, keyword, and metadata search

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use llmspell_core::state::StateScope;
use llmspell_core::traits::storage::VectorStorage;
use llmspell_core::types::storage::VectorResult;

/// Hybrid storage supporting multiple retrieval methods
#[async_trait]
pub trait HybridStorage: VectorStorage {
    /// Perform hybrid search combining vector, keyword, and metadata
    async fn hybrid_search(&self, query: &HybridQuery) -> Result<Vec<HybridResult>>;

    /// Configure retrieval weights for different methods
    fn set_weights(&mut self, weights: RetrievalWeights);

    /// Get current retrieval weights
    fn get_weights(&self) -> &RetrievalWeights;

    /// Perform keyword-only search
    async fn keyword_search(
        &self,
        keywords: &[String],
        scope: Option<&StateScope>,
    ) -> Result<Vec<HybridResult>>;

    /// Perform metadata-only filtering
    async fn metadata_search(
        &self,
        filters: &HashMap<String, Value>,
        scope: Option<&StateScope>,
    ) -> Result<Vec<HybridResult>>;

    /// Rerank results using a different strategy
    async fn rerank(
        &self,
        results: Vec<HybridResult>,
        strategy: RerankingStrategy,
    ) -> Result<Vec<HybridResult>>;
}

/// Query for hybrid retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridQuery {
    /// Optional vector for semantic search
    pub vector: Option<Vec<f32>>,

    /// Optional keywords for keyword search
    pub keywords: Option<Vec<String>>,

    /// Optional metadata filters
    pub metadata_filters: Option<HashMap<String, Value>>,

    /// Number of results to return
    pub k: usize,

    /// Optional scope for multi-tenant isolation
    pub scope: Option<StateScope>,

    /// Minimum score threshold
    pub threshold: Option<f32>,

    /// Retrieval strategy to use
    pub strategy: RetrievalStrategy,

    /// Whether to include explanations in results
    pub include_explanations: bool,
}

impl HybridQuery {
    /// Create a new hybrid query
    #[must_use]
    pub fn new(k: usize) -> Self {
        Self {
            vector: None,
            keywords: None,
            metadata_filters: None,
            k,
            scope: None,
            threshold: None,
            strategy: RetrievalStrategy::default(),
            include_explanations: false,
        }
    }

    /// Add vector search
    #[must_use]
    pub fn with_vector(mut self, vector: Vec<f32>) -> Self {
        self.vector = Some(vector);
        self
    }

    /// Add keyword search
    #[must_use]
    pub fn with_keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = Some(keywords);
        self
    }

    /// Add metadata filters
    #[must_use]
    pub fn with_metadata_filters(mut self, filters: HashMap<String, Value>) -> Self {
        self.metadata_filters = Some(filters);
        self
    }

    /// Set retrieval strategy
    #[must_use]
    pub const fn with_strategy(mut self, strategy: RetrievalStrategy) -> Self {
        self.strategy = strategy;
        self
    }
}

/// Result from hybrid search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridResult {
    /// Document/vector ID
    pub id: String,

    /// Combined score
    pub score: f32,

    /// Individual scores from different methods
    pub component_scores: ComponentScores,

    /// Retrieved content
    pub content: Option<String>,

    /// Metadata
    pub metadata: Option<HashMap<String, Value>>,

    /// Explanation of how the result was retrieved
    pub explanation: Option<String>,
}

/// Component scores from different retrieval methods
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentScores {
    /// Vector similarity score
    pub vector_score: Option<f32>,

    /// Keyword relevance score
    pub keyword_score: Option<f32>,

    /// Metadata match score
    pub metadata_score: Option<f32>,

    /// Reranking score if applied
    pub rerank_score: Option<f32>,
}

/// Weights for different retrieval methods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalWeights {
    /// Weight for vector similarity (0.0 to 1.0)
    pub vector_weight: f32,

    /// Weight for keyword matching (0.0 to 1.0)
    pub keyword_weight: f32,

    /// Weight for metadata filtering (0.0 to 1.0)
    pub metadata_weight: f32,
}

impl Default for RetrievalWeights {
    fn default() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.2,
            metadata_weight: 0.1,
        }
    }
}

impl RetrievalWeights {
    /// Create weights for vector-only search
    #[must_use]
    pub const fn vector_only() -> Self {
        Self {
            vector_weight: 1.0,
            keyword_weight: 0.0,
            metadata_weight: 0.0,
        }
    }

    /// Create weights for keyword-only search
    #[must_use]
    pub const fn keyword_only() -> Self {
        Self {
            vector_weight: 0.0,
            keyword_weight: 1.0,
            metadata_weight: 0.0,
        }
    }

    /// Create balanced weights
    #[must_use]
    pub const fn balanced() -> Self {
        Self {
            vector_weight: 0.33,
            keyword_weight: 0.33,
            metadata_weight: 0.34,
        }
    }

    /// Normalize weights to sum to 1.0
    pub fn normalize(&mut self) {
        let sum = self.vector_weight + self.keyword_weight + self.metadata_weight;
        if sum > 0.0 {
            self.vector_weight /= sum;
            self.keyword_weight /= sum;
            self.metadata_weight /= sum;
        }
    }
}

/// Retrieval strategy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum RetrievalStrategy {
    /// Combine all methods with weighted scores
    #[default]
    Fusion,

    /// Use vector search, fall back to keyword if no results
    VectorFirst,

    /// Use keyword search, enhance with vectors
    KeywordFirst,

    /// Apply methods in sequence as filters
    Sequential,

    /// Use all methods in parallel and merge
    Parallel,
}

/// Reranking strategy for results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RerankingStrategy {
    /// No reranking
    None,

    /// Use a cross-encoder model
    CrossEncoder {
        /// Model name or path for cross-encoding
        model: String,
    },

    /// Use reciprocal rank fusion
    ReciprocalRankFusion,

    /// Custom scoring function
    Custom {
        /// Function name or code for custom scoring
        function: String,
    },
}

impl From<VectorResult> for HybridResult {
    fn from(result: VectorResult) -> Self {
        Self {
            id: result.id,
            score: result.score,
            component_scores: ComponentScores {
                vector_score: Some(result.score),
                ..Default::default()
            },
            content: None,
            metadata: result.metadata,
            explanation: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_query_builder() {
        let query = HybridQuery::new(10)
            .with_vector(vec![1.0, 2.0, 3.0])
            .with_keywords(vec!["test".to_string(), "query".to_string()])
            .with_strategy(RetrievalStrategy::VectorFirst);

        assert_eq!(query.k, 10);
        assert!(query.vector.is_some());
        assert!(query.keywords.is_some());
        assert_eq!(query.strategy, RetrievalStrategy::VectorFirst);
    }

    #[test]
    fn test_retrieval_weights_normalization() {
        let mut weights = RetrievalWeights {
            vector_weight: 2.0,
            keyword_weight: 1.0,
            metadata_weight: 1.0,
        };

        weights.normalize();

        assert!((weights.vector_weight - 0.5).abs() < 0.001);
        assert!((weights.keyword_weight - 0.25).abs() < 0.001);
        assert!((weights.metadata_weight - 0.25).abs() < 0.001);
    }
}
