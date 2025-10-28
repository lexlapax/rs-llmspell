//! RAG Result Adapter - converts RAG retrieval results to context format
//!
//! Bridges between llmspell-rag's `RAGResult` and llmspell-context's `RankedChunk`.

use llmspell_rag::pipeline::{RAGResult, RAGRetriever};
use crate::types::{Chunk, RankedChunk};

/// Default source identifier for RAG results
const DEFAULT_RAG_SOURCE: &str = "rag";

/// Default ranker identifier for RAG vector search
const RAG_RANKER_NAME: &str = "rag_vector_search";

/// Convert `RAGResult` to `RankedChunk`
///
/// Maps RAG retrieval result to context pipeline format:
/// - RAGResult.id → Chunk.id
/// - RAGResult.content → Chunk.content
/// - RAGResult.timestamp → Chunk.timestamp
/// - RAGResult.metadata → Chunk.metadata (converts `HashMap` to Option<Value>)
/// - RAGResult.score → RankedChunk.score
///
/// # Arguments
/// * `rag_result` - RAG retrieval result to convert
///
/// # Returns
/// `RankedChunk` ready for context assembly pipeline
#[must_use]
pub fn rag_result_to_ranked_chunk(rag_result: RAGResult) -> RankedChunk {
    let source = extract_source_from_metadata(&rag_result);

    let metadata = if rag_result.metadata.is_empty() {
        None
    } else {
        Some(serde_json::to_value(&rag_result.metadata).unwrap_or(serde_json::Value::Null))
    };

    let chunk = Chunk {
        id: rag_result.id,
        content: rag_result.content,
        source,
        timestamp: rag_result.timestamp,
        metadata,
    };

    RankedChunk {
        chunk,
        score: rag_result.score,
        ranker: RAG_RANKER_NAME.to_string(),
    }
}

/// Extract source identifier from RAG result metadata
///
/// Checks metadata for "source" field, falls back to default if not present.
fn extract_source_from_metadata(rag_result: &RAGResult) -> String {
    rag_result
        .metadata
        .get("source")
        .and_then(|v| v.as_str())
        .map_or_else(|| DEFAULT_RAG_SOURCE.to_string(), String::from)
}

/// Convert batch of RAG results to ranked chunks
///
/// Convenience function for converting multiple results at once.
#[must_use]
pub fn rag_results_to_ranked_chunks(rag_results: Vec<RAGResult>) -> Vec<RankedChunk> {
    rag_results
        .into_iter()
        .map(rag_result_to_ranked_chunk)
        .collect()
}

/// RAG-backed retriever adapter
///
/// Wraps any `RAGRetriever` implementation to provide chunks in context format.
pub struct RAGAdapter<R: RAGRetriever> {
    retriever: R,
}

impl<R: RAGRetriever> RAGAdapter<R> {
    /// Create new RAG adapter
    #[must_use]
    pub const fn new(retriever: R) -> Self {
        Self { retriever }
    }

    /// Retrieve and convert RAG results to ranked chunks
    ///
    /// # Arguments
    /// * `query` - Search query
    /// * `k` - Number of results
    /// * `scope` - Optional scope for filtering
    ///
    /// # Returns
    /// Ranked chunks ready for context assembly
    ///
    /// # Errors
    /// Returns error if RAG retrieval fails
    pub async fn retrieve_chunks(
        &self,
        query: &str,
        k: usize,
        scope: Option<llmspell_core::state::StateScope>,
    ) -> anyhow::Result<Vec<RankedChunk>> {
        let rag_results = self.retriever.retrieve(query, k, scope).await?;
        Ok(rag_results_to_ranked_chunks(rag_results))
    }

    /// Get reference to underlying retriever
    #[must_use]
    pub const fn retriever(&self) -> &R {
        &self.retriever
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::collections::HashMap;

    #[test]
    fn test_rag_result_to_ranked_chunk_basic() {
        let rag_result = RAGResult {
            id: "result-1".to_string(),
            content: "test content".to_string(),
            score: 0.85,
            metadata: HashMap::new(),
            timestamp: Utc::now(),
        };

        let ranked_chunk = rag_result_to_ranked_chunk(rag_result.clone());

        assert_eq!(ranked_chunk.chunk.id, "result-1");
        assert_eq!(ranked_chunk.chunk.content, "test content");
        assert_eq!(ranked_chunk.chunk.source, DEFAULT_RAG_SOURCE);
        assert_eq!(ranked_chunk.score, 0.85);
        assert_eq!(ranked_chunk.ranker, RAG_RANKER_NAME);
        assert!(ranked_chunk.chunk.metadata.is_none());
    }

    #[test]
    fn test_rag_result_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), serde_json::json!("custom_source"));
        metadata.insert("doc_type".to_string(), serde_json::json!("technical"));

        let rag_result = RAGResult {
            id: "result-2".to_string(),
            content: "content with metadata".to_string(),
            score: 0.92,
            metadata,
            timestamp: Utc::now(),
        };

        let ranked_chunk = rag_result_to_ranked_chunk(rag_result);

        assert_eq!(ranked_chunk.chunk.source, "custom_source");
        assert!(ranked_chunk.chunk.metadata.is_some());

        let metadata_val = ranked_chunk.chunk.metadata.unwrap();
        assert_eq!(
            metadata_val.get("doc_type"),
            Some(&serde_json::json!("technical"))
        );
    }

    #[test]
    fn test_rag_results_to_ranked_chunks() {
        let results = vec![
            RAGResult {
                id: "r1".to_string(),
                content: "content 1".to_string(),
                score: 0.9,
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            },
            RAGResult {
                id: "r2".to_string(),
                content: "content 2".to_string(),
                score: 0.7,
                metadata: HashMap::new(),
                timestamp: Utc::now(),
            },
        ];

        let chunks = rag_results_to_ranked_chunks(results);

        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].chunk.id, "r1");
        assert_eq!(chunks[1].chunk.id, "r2");
        assert_eq!(chunks[0].score, 0.9);
        assert_eq!(chunks[1].score, 0.7);
    }
}
