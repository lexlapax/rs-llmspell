//! Retrieval strategies and implementations
//!
//! Provides episodic (vector), semantic (graph), BM25 (keyword), and hybrid retrieval.

pub mod bm25;
pub mod strategy;

pub use bm25::BM25Retriever;
pub use strategy::StrategySelector;
