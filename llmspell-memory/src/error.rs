//! Error types for memory system

use thiserror::Error;

/// Result type alias for memory operations
pub type Result<T> = std::result::Result<T, MemoryError>;

/// Errors that can occur in the memory system
#[derive(Debug, Error)]
pub enum MemoryError {
    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Vector search error
    #[error("Vector search error: {0}")]
    VectorSearch(String),

    /// Knowledge graph error
    #[error("Knowledge graph error: {0}")]
    KnowledgeGraph(String),

    /// Consolidation error
    #[error("Consolidation error: {0}")]
    Consolidation(String),

    /// LLM call error
    #[error("LLM call error: {0}")]
    LLMCall(String),

    /// Embedding generation error
    #[error("Embedding generation error: {0}")]
    EmbeddingError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Entry not found
    #[error("Entry not found: {0}")]
    NotFound(String),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] llmspell_core::LLMSpellError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<String> for MemoryError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for MemoryError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}
