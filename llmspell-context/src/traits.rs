//! Trait definitions for pluggable components in the context engineering pipeline

use crate::error::Result;
use crate::types::{Chunk, QueryUnderstanding, RankedChunk};
use async_trait::async_trait;

/// Trait for query understanding components
#[async_trait]
pub trait QueryAnalyzer: Send + Sync {
    /// Analyze a query to extract intent, entities, and keywords
    async fn understand(&self, query: &str) -> Result<QueryUnderstanding>;
}

/// Trait for retrieval components
#[async_trait]
pub trait Retriever: Send + Sync {
    /// Retrieve candidate chunks based on query
    async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<Chunk>>;
}

/// Trait for reranking components
#[async_trait]
pub trait Reranker: Send + Sync {
    /// Rerank chunks based on relevance to query
    async fn rerank(
        &self,
        chunks: Vec<Chunk>,
        query: &str,
        top_k: usize,
    ) -> Result<Vec<RankedChunk>>;
}

/// Trait for context assembly components
pub trait Assembler: Send + Sync {
    /// Assemble ranked chunks into coherent context
    ///
    /// # Errors
    ///
    /// Returns `ContextError::AssemblyError` if context assembly fails
    fn assemble(&self, chunks: Vec<RankedChunk>, query: &QueryUnderstanding) -> Result<String>;
}
