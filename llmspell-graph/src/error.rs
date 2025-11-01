//! Error types for knowledge graph operations

use thiserror::Error;

/// Result type alias for graph operations
pub type Result<T> = std::result::Result<T, GraphError>;

/// Errors that can occur in the knowledge graph system
#[derive(Debug, Error)]
pub enum GraphError {
    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Relationship not found
    #[error("Relationship not found: {0}")]
    RelationshipNotFound(String),

    /// Invalid temporal query
    #[error("Invalid temporal query: {0}")]
    InvalidTemporalQuery(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Invalid input
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Core error
    #[error("Core error: {0}")]
    Core(#[from] llmspell_core::LLMSpellError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// `SurrealDB` error
    #[error("SurrealDB error: {0}")]
    SurrealDB(String),

    /// Other error
    #[error("{0}")]
    Other(String),
}

impl From<String> for GraphError {
    fn from(s: String) -> Self {
        Self::Other(s)
    }
}

impl From<&str> for GraphError {
    fn from(s: &str) -> Self {
        Self::Other(s.to_string())
    }
}

impl From<surrealdb::Error> for GraphError {
    fn from(e: surrealdb::Error) -> Self {
        Self::SurrealDB(e.to_string())
    }
}
