//! Hybrid RAG + Memory Retrieval
//!
//! Combines RAG vector search with episodic memory retrieval using weighted merge.
//! Supports configurable token budget allocation across sources.

use anyhow::Result;
use llmspell_core::state::StateScope;
use llmspell_memory::traits::MemoryManager;
use llmspell_rag::pipeline::RAGRetriever;
use std::sync::Arc;
use tracing::{debug, error, info, trace};

use super::rag_adapter::rag_results_to_ranked_chunks;
use crate::types::{Chunk, RankedChunk};

/// Retrieval source weights for hybrid retrieval
///
/// Weights control how scores are combined from different sources.
/// Must sum to 1.0 ±0.01 for validation to pass.
#[derive(Debug, Clone, Copy)]
pub struct RetrievalWeights {
    /// Weight for RAG vector search results (0.0-1.0)
    pub rag_weight: f32,
    /// Weight for memory episodic search results (0.0-1.0)
    pub memory_weight: f32,
}

impl RetrievalWeights {
    /// Create new retrieval weights
    ///
    /// # Arguments
    /// * `rag_weight` - Weight for RAG results (0.0-1.0)
    /// * `memory_weight` - Weight for memory results (0.0-1.0)
    ///
    /// # Returns
    /// Validated weights if they sum to ~1.0, error otherwise
    ///
    /// # Errors
    /// Returns error if weights don't sum to 1.0 ±0.01
    pub fn new(rag_weight: f32, memory_weight: f32) -> Result<Self> {
        let sum = rag_weight + memory_weight;
        if (sum - 1.0).abs() > 0.01 {
            anyhow::bail!(
                "Retrieval weights must sum to 1.0 ±0.01, got {:.3} (rag={:.3}, memory={:.3})",
                sum,
                rag_weight,
                memory_weight
            );
        }

        Ok(Self {
            rag_weight,
            memory_weight,
        })
    }

    /// Balanced preset: Equal weight to RAG and memory (50/50)
    #[must_use]
    pub const fn balanced() -> Self {
        Self {
            rag_weight: 0.5,
            memory_weight: 0.5,
        }
    }

    /// RAG-focused preset: Emphasize RAG results (70/30)
    #[must_use]
    pub const fn rag_focused() -> Self {
        Self {
            rag_weight: 0.7,
            memory_weight: 0.3,
        }
    }

    /// Memory-focused preset: Emphasize memory results (40/60)
    #[must_use]
    pub const fn memory_focused() -> Self {
        Self {
            rag_weight: 0.4,
            memory_weight: 0.6,
        }
    }
}

impl Default for RetrievalWeights {
    fn default() -> Self {
        Self::memory_focused()
    }
}

/// Hybrid retriever combining RAG and episodic memory
///
/// Queries both RAG vector search (if available) and episodic memory,
/// then combines results using weighted merge and token budget allocation.
pub struct HybridRetriever {
    /// Optional RAG pipeline for vector search (None = memory-only fallback)
    rag_pipeline: Option<Arc<dyn RAGRetriever>>,
    /// Memory manager for episodic search
    memory_manager: Arc<dyn MemoryManager>,
    /// Weights for combining results
    weights: RetrievalWeights,
}

impl HybridRetriever {
    /// Create new hybrid retriever
    ///
    /// # Arguments
    /// * `rag_pipeline` - Optional RAG pipeline (None for memory-only)
    /// * `memory_manager` - Memory manager for episodic retrieval
    /// * `weights` - Weights for combining results
    #[must_use]
    pub fn new(
        rag_pipeline: Option<Arc<dyn RAGRetriever>>,
        memory_manager: Arc<dyn MemoryManager>,
        weights: RetrievalWeights,
    ) -> Self {
        info!(
            "Created HybridRetriever: rag={}, memory={}, weights=(rag={:.2}, mem={:.2})",
            rag_pipeline.is_some(),
            true,
            weights.rag_weight,
            weights.memory_weight
        );

        Self {
            rag_pipeline,
            memory_manager,
            weights,
        }
    }

    /// Retrieve and merge results from RAG + Memory with weighted scoring
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `session_id` - Session identifier for scoping
    /// * `token_budget` - Maximum tokens to retrieve (allocated across sources)
    ///
    /// # Returns
    /// Merged and ranked chunks respecting token budget
    ///
    /// # Errors
    /// Returns error if retrieval or merging fails
    pub async fn retrieve_hybrid(
        &self,
        query: &str,
        session_id: &str,
        token_budget: usize,
    ) -> Result<Vec<RankedChunk>> {
        info!(
            "Starting hybrid retrieval: query=\"{}\" session={} budget={}",
            query, session_id, token_budget
        );

        // Allocate token budget across sources based on weights
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let rag_budget = if self.rag_pipeline.is_some() {
            (token_budget as f32 * self.weights.rag_weight) as usize
        } else {
            0
        };
        let memory_budget = token_budget - rag_budget;

        debug!(
            "Token budget allocation: rag={} memory={} (total={})",
            rag_budget, memory_budget, token_budget
        );

        // Query RAG if available
        let mut rag_chunks = Vec::new();
        if let Some(ref rag) = self.rag_pipeline {
            debug!("Querying RAG pipeline...");
            let scope = Some(StateScope::Custom(format!("session:{session_id}")));

            // Estimate ~100 tokens per result for k calculation
            let rag_k = (rag_budget / 100).max(1);

            match rag.retrieve(query, rag_k, scope).await {
                Ok(results) => {
                    debug!("RAG returned {} results", results.len());
                    rag_chunks = rag_results_to_ranked_chunks(results);
                    debug!("Converted to {} RankedChunks", rag_chunks.len());
                }
                Err(e) => {
                    error!("RAG retrieval failed: {}", e);
                    // Continue with memory-only
                }
            }
        } else {
            debug!("No RAG pipeline configured, using memory-only");
        }

        // Query episodic memory
        debug!("Querying episodic memory...");
        let memory_k = (memory_budget / 100).max(1);
        let episodic_results = self
            .memory_manager
            .episodic()
            .search(query, memory_k)
            .await
            .map_err(|e| anyhow::anyhow!("Episodic search failed: {}", e))?;

        debug!("Episodic memory returned {} results", episodic_results.len());

        // Convert episodic entries to ranked chunks
        let memory_chunks: Vec<RankedChunk> = episodic_results
            .into_iter()
            .filter(|entry| entry.session_id == session_id) // Filter by session
            .map(|entry| {
                let metadata = if entry.metadata.is_null() {
                    None
                } else {
                    Some(entry.metadata.clone())
                };

                let chunk = Chunk {
                    id: entry.id,
                    content: entry.content,
                    source: format!("memory:{}", entry.session_id),
                    timestamp: entry.timestamp,
                    metadata,
                };

                RankedChunk {
                    chunk,
                    score: 1.0, // Episodic search doesn't return scores, default to 1.0
                    ranker: "episodic_vector_search".to_string(),
                }
            })
            .collect();

        debug!("Converted to {} memory RankedChunks", memory_chunks.len());

        // Apply weighted merge
        let mut merged = self.weighted_merge(rag_chunks, memory_chunks);

        debug!("Merged results: {} chunks", merged.len());
        trace!("Merged chunk scores: {:?}", merged.iter().map(|c| c.score).collect::<Vec<_>>());

        // Sort by score descending
        merged.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Truncate to token budget (estimate ~100 tokens per chunk)
        let max_chunks = (token_budget / 100).max(1);
        merged.truncate(max_chunks);

        info!("Hybrid retrieval complete: returning {} chunks", merged.len());

        Ok(merged)
    }

    /// Weighted merge of RAG and memory results
    ///
    /// Applies weights to scores and combines both result sets.
    fn weighted_merge(
        &self,
        mut rag_chunks: Vec<RankedChunk>,
        mut memory_chunks: Vec<RankedChunk>,
    ) -> Vec<RankedChunk> {
        debug!(
            "Applying weighted merge: {} RAG + {} memory chunks",
            rag_chunks.len(),
            memory_chunks.len()
        );

        // Apply RAG weight to scores
        for chunk in &mut rag_chunks {
            chunk.score *= self.weights.rag_weight;
        }

        // Apply memory weight to scores
        for chunk in &mut memory_chunks {
            chunk.score *= self.weights.memory_weight;
        }

        // Combine
        rag_chunks.extend(memory_chunks);
        rag_chunks
    }

    /// Get reference to RAG pipeline (if configured)
    #[must_use]
    pub const fn rag_pipeline(&self) -> Option<&Arc<dyn RAGRetriever>> {
        self.rag_pipeline.as_ref()
    }

    /// Get reference to memory manager
    #[must_use]
    pub const fn memory_manager(&self) -> &Arc<dyn MemoryManager> {
        &self.memory_manager
    }

    /// Get current retrieval weights
    #[must_use]
    pub const fn weights(&self) -> RetrievalWeights {
        self.weights
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retrieval_weights_validation() {
        // Valid weights
        assert!(RetrievalWeights::new(0.4, 0.6).is_ok());
        assert!(RetrievalWeights::new(0.5, 0.5).is_ok());

        // Invalid weights (don't sum to 1.0)
        assert!(RetrievalWeights::new(0.3, 0.5).is_err());
        assert!(RetrievalWeights::new(0.6, 0.6).is_err());
    }

    #[test]
    fn test_retrieval_weights_presets() {
        let balanced = RetrievalWeights::balanced();
        assert_eq!(balanced.rag_weight, 0.5);
        assert_eq!(balanced.memory_weight, 0.5);

        let rag_focused = RetrievalWeights::rag_focused();
        assert_eq!(rag_focused.rag_weight, 0.7);
        assert_eq!(rag_focused.memory_weight, 0.3);

        let memory_focused = RetrievalWeights::memory_focused();
        assert_eq!(memory_focused.rag_weight, 0.4);
        assert_eq!(memory_focused.memory_weight, 0.6);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Test needs exact equality
    fn test_default_weights() {
        let default = RetrievalWeights::default();
        assert_eq!(default.rag_weight, 0.4);
        assert_eq!(default.memory_weight, 0.6);
    }
}
