//! Reranking algorithms for relevance scoring
//!
//! Provides pluggable reranking via the `Reranker` trait.
//! Current implementations:
//! - `DeBERTaReranker`: Neural cross-encoder (Candle, ~30ms)
//! - `BM25Reranker`: Lexical keyword matching (fast, <5ms)
//!
//! Future implementations can be added by implementing the `Reranker` trait:
//! - ColBERT late interaction
//! - T5 sequence-to-sequence
//! - LLM-based scoring (Ollama/OpenAI)

pub mod deberta;

pub use deberta::DeBERTaReranker;
