//! Retrieval strategies and implementations
//!
//! Provides episodic (vector), semantic (graph), BM25 (keyword), and hybrid retrieval.

pub mod bm25;

pub use bm25::BM25Retriever;
