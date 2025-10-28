//! Retrieval strategies and implementations
//!
//! Provides episodic (vector), semantic (graph), BM25 (keyword), and hybrid retrieval.

pub mod bm25;
pub mod rag_adapter;
pub mod strategy;

pub use bm25::BM25Retriever;
pub use rag_adapter::{rag_result_to_ranked_chunk, rag_results_to_ranked_chunks, RAGAdapter};
pub use strategy::StrategySelector;
