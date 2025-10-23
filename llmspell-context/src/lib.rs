// ABOUTME: Context engineering pipeline for LLMSpell framework
// ABOUTME: Provides query understanding, retrieval strategy selection, reranking, and context assembly

//! # `LLMSpell` Context Engineering
//!
//! This crate implements the context engineering pipeline for the `LLMSpell` framework,
//! providing intelligent retrieval, reranking, and assembly of relevant context for LLM prompts.
//!
//! ## Features
//!
//! - **Query Understanding**: Intent classification, entity extraction, keyword detection
//! - **Retrieval Strategies**: Episodic (vector), Semantic (graph), BM25 (keyword), Hybrid
//! - **Reranking**: `DeBERTa` cross-encoder (Candle) and BM25 fallback
//! - **Context Assembly**: Temporal ordering, token budget management, confidence scoring
//!
//! ## Pipeline Stages
//!
//! 1. **Query Understanding** → Analyze user query (intent, entities, keywords)
//! 2. **Retrieval Strategy Selection** → Choose retrieval approach based on query
//! 3. **Retrieval** → Fetch candidate chunks from episodic memory or knowledge graph
//! 4. **Reranking** → Score and reorder chunks using `DeBERTa` or BM25
//! 5. **Assembly** → Structure top chunks into coherent context with metadata
//!
//! ## Usage
//!
//! ```rust,ignore
//! use llmspell_context::prelude::*;
//!
//! let pipeline = ContextPipeline::builder()
//!     .with_reranker(DeBERTaReranker::new().await?)
//!     .with_assembler(ContextAssembler::default())
//!     .build()?;
//!
//! let context = pipeline.process_query("What is Rust?", &memory, &graph).await?;
//! ```

#![warn(missing_docs)]
#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

/// Trait definitions for pluggable components
pub mod traits;

/// Query understanding and analysis
pub mod query;

/// Retrieval strategies and implementations
pub mod retrieval;

/// Reranking algorithms (DeBERTa, BM25)
pub mod reranking;

/// Context assembly and structuring
pub mod assembly;

/// End-to-end pipeline orchestration
pub mod pipeline;

/// Shared types and data structures
pub mod types;

/// Error types and handling
pub mod error;

/// Convenience re-exports
pub mod prelude;
