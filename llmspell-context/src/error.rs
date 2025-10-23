//! Error types for the context engineering pipeline

use thiserror::Error;

/// Result type for context operations
pub type Result<T> = std::result::Result<T, ContextError>;

/// Errors that can occur in the context engineering pipeline
#[derive(Error, Debug)]
pub enum ContextError {
    /// Query understanding failed
    #[error("Query understanding failed: {0}")]
    QueryUnderstandingError(String),

    /// Retrieval failed
    #[error("Retrieval failed: {0}")]
    RetrievalError(String),

    /// Reranking failed
    #[error("Reranking failed: {0}")]
    RerankingError(String),

    /// Assembly failed
    #[error("Assembly failed: {0}")]
    AssemblyError(String),

    /// Model loading failed
    #[error("Model loading failed: {0}")]
    ModelLoadError(String),

    /// Model download failed
    #[error("Model download failed: {0}")]
    ModelDownloadError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Memory error (from llmspell-memory)
    #[error("Memory error: {0}")]
    MemoryError(String),

    /// Graph error (from llmspell-graph)
    #[error("Graph error: {0}")]
    GraphError(String),

    /// Other error
    #[error("Other error: {0}")]
    Other(#[from] anyhow::Error),
}
