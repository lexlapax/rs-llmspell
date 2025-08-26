//! Retrieval flow with hybrid search and reranking

use anyhow::Result;
use llmspell_state_traits::StateScope;
use llmspell_storage::vector_storage::{VectorQuery, VectorResult, VectorStorage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};

use super::config::{HybridWeights, QueryConfig, RetrievalConfig};
use crate::embeddings::{EmbeddingCache, EmbeddingFactory};

/// Retrieval flow that orchestrates hybrid search and reranking
pub struct RetrievalFlow {
    /// Vector storage backend
    storage: Arc<dyn VectorStorage>,

    /// Embedding generation for query vectors
    embedding_factory: Arc<EmbeddingFactory>,

    /// Embedding cache for query optimization
    embedding_cache: Arc<EmbeddingCache>,

    /// Retrieval configuration
    config: RetrievalConfig,
}

impl std::fmt::Debug for RetrievalFlow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RetrievalFlow")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl RetrievalFlow {
    /// Create new retrieval flow
    pub fn new(
        storage: Arc<dyn VectorStorage>,
        embedding_factory: Arc<EmbeddingFactory>,
        embedding_cache: Arc<EmbeddingCache>,
        config: RetrievalConfig,
    ) -> Self {
        Self {
            storage,
            embedding_factory,
            embedding_cache,
            config,
        }
    }

    /// Search with hybrid retrieval and reranking
    ///
    /// # Errors
    ///
    /// Returns an error if embedding generation or storage operations fail
    pub async fn search(
        &self,
        query: String,
        scope: Option<StateScope>,
        query_config: Option<QueryConfig>,
    ) -> Result<RetrievalResult> {
        debug!("Starting hybrid search for query: {}", query);

        // Merge configurations
        let effective_config = self.merge_configs(query_config);

        // Generate query embedding
        let query_embedding = self.embed_query(&query).await?;

        // Execute vector search
        let vector_results = self
            .vector_search(&query_embedding, scope.as_ref(), &effective_config)
            .await?;

        // Execute keyword search (placeholder for now)
        let keyword_results = Self::keyword_search(&query, scope.as_ref(), &effective_config);

        // Fuse scores from different retrieval methods
        let hybrid_weights = effective_config
            .hybrid_weights
            .as_ref()
            .unwrap_or(&self.config.hybrid_weights);
        let fused_results = Self::fuse_scores(vector_results, keyword_results, hybrid_weights);

        // Apply metadata filtering
        let filtered_results =
            Self::apply_metadata_filters(fused_results, &effective_config.metadata_filters);

        // Rerank results if configured
        let final_results = if effective_config
            .reranking
            .as_ref()
            .unwrap_or(&self.config.reranking)
            .enabled
        {
            self.rerank_results(filtered_results, &query, &effective_config)
        } else {
            filtered_results
        };

        // Limit results
        let max_results = effective_config
            .max_results
            .unwrap_or(self.config.max_results);
        let limited_results: Vec<SearchResult> =
            final_results.into_iter().take(max_results).collect();

        info!(
            "Hybrid search completed with {} results",
            limited_results.len()
        );

        Ok(RetrievalResult {
            results: limited_results,
            query: query.clone(),
            total_candidates: 0,  // TODO: Track this properly
            retrieval_time_ms: 0, // TODO: Add timing
            reranked: effective_config
                .reranking
                .as_ref()
                .unwrap_or(&self.config.reranking)
                .enabled,
        })
    }

    /// Generate embedding for query with caching
    async fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let cache_key = EmbeddingCache::generate_key(query);

        // Try cache first
        if let Some(cached) = self.embedding_cache.get(&cache_key) {
            debug!("Cache hit for query embedding");
            return Ok(cached);
        }

        // Generate new embedding
        let model = self.embedding_factory.create_model()?;
        let embeddings = model.embed(&[query.to_string()]).await?;

        embeddings.into_iter().next().map_or_else(
            || Err(anyhow::anyhow!("Failed to generate query embedding")),
            |embedding| {
                // Cache the result
                if let Err(e) = self.embedding_cache.put(cache_key, embedding.clone()) {
                    warn!("Failed to cache query embedding: {}", e);
                }
                Ok(embedding)
            },
        )
    }

    /// Execute vector similarity search
    async fn vector_search(
        &self,
        query_embedding: &[f32],
        scope: Option<&StateScope>,
        config: &QueryConfig,
    ) -> Result<Vec<VectorResult>> {
        let k = config.max_results.unwrap_or(self.config.max_results * 2); // Get more candidates

        let mut vector_query = VectorQuery::new(query_embedding.to_vec(), k)
            .with_filter(config.metadata_filters.clone());

        if let Some(threshold) = config.min_score.or(Some(self.config.min_score)) {
            vector_query = vector_query.with_threshold(threshold);
        }

        let results = match scope {
            Some(scope) => self
                .storage
                .search_scoped(&vector_query, scope)
                .await
                .unwrap_or_default(),
            None => self.storage.search(&vector_query).await.unwrap_or_default(),
        };

        debug!("Vector search returned {} results", results.len());
        Ok(results)
    }

    /// Execute keyword/BM25 search (placeholder implementation)
    fn keyword_search(
        _query: &str,
        _scope: Option<&StateScope>,
        _config: &QueryConfig,
    ) -> Vec<VectorResult> {
        // TODO: Implement actual keyword search
        // This would typically use a full-text search index like Tantivy
        debug!("Keyword search not yet implemented");
        Vec::new()
    }

    /// Fuse scores from different retrieval methods
    fn fuse_scores(
        vector_results: Vec<VectorResult>,
        keyword_results: Vec<VectorResult>,
        weights: &HybridWeights,
    ) -> Vec<SearchResult> {
        let mut score_map: HashMap<String, SearchResult> = HashMap::new();

        // Process vector results
        for result in vector_results {
            let search_result = SearchResult {
                id: result.id.clone(),
                content: result
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("text"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: result.score * weights.vector,
                vector_score: Some(result.score),
                keyword_score: None,
                metadata_score: None,
                recency_score: None,
                metadata: result.metadata.unwrap_or_default(),
                distance: result.distance,
            };
            score_map.insert(result.id, search_result);
        }

        // Process keyword results and merge scores
        for result in keyword_results {
            if let Some(existing) = score_map.get_mut(&result.id) {
                existing.keyword_score = Some(result.score);
                existing.score += result.score * weights.keyword;
            } else {
                let search_result = SearchResult {
                    id: result.id.clone(),
                    content: result
                        .metadata
                        .as_ref()
                        .and_then(|m| m.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    score: result.score * weights.keyword,
                    vector_score: None,
                    keyword_score: Some(result.score),
                    metadata_score: None,
                    recency_score: None,
                    metadata: result.metadata.unwrap_or_default(),
                    distance: result.distance,
                };
                score_map.insert(result.id, search_result);
            }
        }

        // Apply metadata and recency boosts
        for result in score_map.values_mut() {
            // Metadata boost (based on metadata richness)
            let metadata_boost = Self::calculate_metadata_boost(&result.metadata);
            result.metadata_score = Some(metadata_boost);
            result.score += metadata_boost * weights.metadata;

            // Recency boost
            let recency_boost = Self::calculate_recency_boost(&result.metadata);
            result.recency_score = Some(recency_boost);
            result.score += recency_boost * weights.recency;
        }

        // Convert to sorted vector
        let mut results: Vec<SearchResult> = score_map.into_values().collect();
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Apply metadata filters to results
    fn apply_metadata_filters(
        results: Vec<SearchResult>,
        filters: &HashMap<String, serde_json::Value>,
    ) -> Vec<SearchResult> {
        if filters.is_empty() {
            return results;
        }

        let filtered: Vec<SearchResult> = results
            .into_iter()
            .filter(|result| {
                filters
                    .iter()
                    .all(|(key, expected_value)| result.metadata.get(key) == Some(expected_value))
            })
            .collect();

        debug!("Filtered {} results based on metadata", filtered.len());
        filtered
    }

    /// Rerank results using configured strategy
    fn rerank_results(
        &self,
        results: Vec<SearchResult>,
        _query: &str,
        config: &QueryConfig,
    ) -> Vec<SearchResult> {
        let rerank_config = config.reranking.as_ref().unwrap_or(&self.config.reranking);

        match rerank_config.strategy {
            super::config::RerankingStrategy::None => results,
            super::config::RerankingStrategy::MMR => {
                Self::mmr_rerank(results, rerank_config.diversity_lambda)
            }
            super::config::RerankingStrategy::Score => {
                // Simple score-based reranking (already sorted by score)
                results
            }
            super::config::RerankingStrategy::Custom(_) => {
                warn!("Custom reranking not implemented, falling back to score-based");
                results
            }
        }
    }

    /// Epsilon for floating point comparisons
    const EPSILON: f32 = 1e-6;

    /// Maximal Marginal Relevance reranking for diversity
    fn mmr_rerank(results: Vec<SearchResult>, lambda: f32) -> Vec<SearchResult> {
        if results.len() <= 1 {
            return results;
        }

        let mut reranked = Vec::with_capacity(results.len());
        let mut remaining = results;

        // Start with highest-scoring result
        let max_score = remaining
            .iter()
            .map(|r| r.score)
            .fold(f32::NEG_INFINITY, f32::max);
        if let Some(first) = remaining
            .iter()
            .position(|r| (r.score - max_score).abs() < Self::EPSILON)
        {
            reranked.push(remaining.remove(first));
        }

        // Iteratively select results balancing relevance and diversity
        while !remaining.is_empty() {
            let mut best_idx = 0;
            let mut best_mmr_score = f32::NEG_INFINITY;

            for (i, candidate) in remaining.iter().enumerate() {
                // Calculate similarity to already selected items
                let max_similarity = reranked
                    .iter()
                    .map(|selected| {
                        Self::calculate_similarity(&candidate.content, &selected.content)
                    })
                    .fold(0.0f32, f32::max);

                // MMR score: lambda * relevance - (1-lambda) * max_similarity
                let mmr_score = lambda.mul_add(candidate.score, -((1.0 - lambda) * max_similarity));

                if mmr_score > best_mmr_score {
                    best_mmr_score = mmr_score;
                    best_idx = i;
                }
            }

            reranked.push(remaining.remove(best_idx));
        }

        debug!("MMR reranking completed with lambda={}", lambda);
        reranked
    }

    /// Calculate similarity between two text strings (simple heuristic)
    #[allow(clippy::cast_precision_loss)]
    fn calculate_similarity(text1: &str, text2: &str) -> f32 {
        // Simple Jaccard similarity based on word overlap
        let words1: std::collections::HashSet<&str> = text1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = text2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count() as f32;
        let union = words1.union(&words2).count() as f32;

        if union > 0.0 {
            intersection / union
        } else {
            0.0
        }
    }

    /// Calculate metadata boost score
    #[allow(clippy::cast_precision_loss)]
    fn calculate_metadata_boost(metadata: &HashMap<String, serde_json::Value>) -> f32 {
        // Simple heuristic: more metadata fields = higher boost
        let field_count = metadata.len() as f32;
        (field_count / 10.0).min(1.0) // Normalize to 0-1 range
    }

    /// Calculate recency boost score
    #[allow(clippy::cast_precision_loss)]
    fn calculate_recency_boost(metadata: &HashMap<String, serde_json::Value>) -> f32 {
        // Try to extract timestamp and calculate recency boost
        if let Some(timestamp_val) = metadata
            .get("ingested_at")
            .or_else(|| metadata.get("created_at"))
        {
            if let Some(timestamp_str) = timestamp_val.as_str() {
                if let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str) {
                    let now = chrono::Utc::now();
                    let age_days = (now - timestamp.with_timezone(&chrono::Utc)).num_days() as f32;

                    // Exponential decay: more recent = higher score
                    return (-age_days / 30.0).exp().min(1.0);
                }
            }
        }

        // Default neutral boost if no timestamp found
        0.5
    }

    /// Merge query config with default config
    fn merge_configs(&self, query_config: Option<QueryConfig>) -> QueryConfig {
        let query_config = query_config.unwrap_or_default();

        QueryConfig {
            max_results: query_config.max_results.or(Some(self.config.max_results)),
            min_score: query_config.min_score.or(Some(self.config.min_score)),
            hybrid_weights: query_config
                .hybrid_weights
                .or_else(|| Some(self.config.hybrid_weights.clone())),
            metadata_filters: query_config.metadata_filters,
            reranking: query_config
                .reranking
                .or_else(|| Some(self.config.reranking.clone())),
        }
    }
}

/// Result of a retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Search results
    pub results: Vec<SearchResult>,

    /// Original query
    pub query: String,

    /// Total number of candidates considered
    pub total_candidates: usize,

    /// Retrieval time in milliseconds
    pub retrieval_time_ms: u64,

    /// Whether results were reranked
    pub reranked: bool,
}

/// Individual search result with hybrid scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: String,

    /// Text content (if available)
    pub content: String,

    /// Final combined score
    pub score: f32,

    /// Individual component scores
    pub vector_score: Option<f32>,
    /// Keyword/BM25 search score
    pub keyword_score: Option<f32>,
    /// Metadata match score
    pub metadata_score: Option<f32>,
    /// Recency-based score
    pub recency_score: Option<f32>,

    /// Result metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Distance metric value
    pub distance: f32,
}

/// Re-export reranking strategy from config
pub use super::config::RerankingStrategy;

/// Score fusion strategies for hybrid retrieval
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScoreFusion {
    /// Weighted linear combination
    WeightedSum,

    /// Reciprocal rank fusion
    ReciprocalRankFusion,

    /// Normalized score combination
    Normalized,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embeddings::{CacheConfig, EmbeddingProviderConfig, EmbeddingProviderType};
    use llmspell_storage::backends::vector::HNSWVectorStorage;
    use llmspell_storage::vector_storage::HNSWConfig;

    fn create_test_retrieval_flow() -> RetrievalFlow {
        let storage = Arc::new(HNSWVectorStorage::new(384, HNSWConfig::default()));
        let embedding_config = EmbeddingProviderConfig {
            provider_type: EmbeddingProviderType::HuggingFace,
            model: "test-model".to_string(),
            dimensions: Some(384),
            ..Default::default()
        };
        let embedding_factory =
            Arc::new(crate::embeddings::EmbeddingFactory::new(embedding_config));
        let embedding_cache = Arc::new(EmbeddingCache::new(CacheConfig::default()));

        RetrievalFlow::new(
            storage,
            embedding_factory,
            embedding_cache,
            RetrievalConfig::default(),
        )
    }

    #[test]
    fn test_similarity_calculation() {
        let _flow = create_test_retrieval_flow();

        let similarity =
            RetrievalFlow::calculate_similarity("the quick brown fox", "quick brown fox jumps");

        assert!(similarity > 0.0);
        assert!(similarity < 1.0);
    }

    #[test]
    fn test_metadata_boost() {
        let _flow = create_test_retrieval_flow();
        let mut metadata = HashMap::new();
        metadata.insert(
            "title".to_string(),
            serde_json::Value::String("test".to_string()),
        );
        metadata.insert(
            "author".to_string(),
            serde_json::Value::String("test".to_string()),
        );

        let boost = RetrievalFlow::calculate_metadata_boost(&metadata);
        assert!(boost > 0.0);
        assert!(boost <= 1.0);
    }

    #[test]
    fn test_config_merging() {
        let flow = create_test_retrieval_flow();

        let query_config = QueryConfig {
            max_results: Some(20),
            ..Default::default()
        };

        let merged = flow.merge_configs(Some(query_config));
        assert_eq!(merged.max_results, Some(20));
    }
}
