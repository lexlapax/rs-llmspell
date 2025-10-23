//! Convenience re-exports for common use cases

pub use crate::error::{ContextError, Result};
pub use crate::query::RegexQueryAnalyzer;
pub use crate::retrieval::BM25Retriever;
pub use crate::traits::{Assembler, QueryAnalyzer, Reranker, Retriever};
pub use crate::types::{
    AssembledContext, BM25Config, Chunk, QueryIntent, QueryUnderstanding, RankedChunk,
    RetrievalStrategy,
};
