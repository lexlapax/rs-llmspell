//! RAG (Retrieval-Augmented Generation) pipeline implementation
//!
//! This module provides a complete RAG pipeline that orchestrates:
//! - Document chunking and preprocessing
//! - Embedding generation and caching  
//! - Vector storage with multi-tenant support
//! - Hybrid retrieval (vector + keyword + metadata)
//! - Result ranking and filtering

pub mod builder;
pub mod config;
pub mod ingestion;
pub mod rag_pipeline;
pub mod rag_trait;
pub mod retrieval_flow;
pub mod session_adapter;

// Re-export main types
pub use builder::{RAGPipelineBuilder, RAGPipelineBuilderError};
pub use config::{HybridWeights, IngestionConfig, RAGConfig, RerankingConfig, RetrievalConfig};
pub use ingestion::{
    DocumentMetadata, DocumentProcessor, IngestionFlow, IngestionResult, ProcessedDocument,
};
pub use rag_pipeline::{RAGPipeline, RAGPipelineError};
pub use rag_trait::{RAGResult, RAGRetriever};
pub use retrieval_flow::{
    RerankingStrategy, RetrievalFlow, RetrievalResult, ScoreFusion, SearchResult,
};
pub use session_adapter::SessionRAGAdapter;
