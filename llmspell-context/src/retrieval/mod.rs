//! Retrieval strategies and implementations
//!
//! Provides episodic (vector), semantic (graph), BM25 (keyword), RAG, and hybrid retrieval.

pub mod bm25;
pub mod hybrid_rag_memory;
pub mod rag_adapter;
pub mod strategy;

pub use bm25::BM25Retriever;
pub use hybrid_rag_memory::{HybridRetriever, RetrievalWeights};
pub use rag_adapter::{rag_result_to_ranked_chunk, rag_results_to_ranked_chunks, RAGAdapter};
pub use strategy::StrategySelector;
