//! Error types for vectorlite-rs

use thiserror::Error;

/// Result type for vectorlite operations
pub type Result<T> = std::result::Result<T, Error>;

/// Vectorlite error types
#[derive(Error, Debug)]
pub enum Error {
    /// SQLite error
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// HNSW index error
    #[error("HNSW index error: {0}")]
    Hnsw(String),

    /// Invalid vector dimension
    #[error("Invalid dimension: expected {expected}, got {actual}")]
    InvalidDimension { expected: usize, actual: usize },

    /// Invalid distance metric
    #[error("Invalid distance metric: {0}")]
    InvalidMetric(String),

    /// Invalid parameter
    #[error("Invalid parameter {name}: {reason}")]
    InvalidParameter { name: String, reason: String },

    /// Vector not found
    #[error("Vector not found: rowid {0}")]
    VectorNotFound(i64),

    /// Index not initialized
    #[error("HNSW index not initialized")]
    IndexNotInitialized,

    /// Generic error
    #[error("{0}")]
    Other(String),
}

impl From<Error> for rusqlite::Error {
    fn from(err: Error) -> Self {
        rusqlite::Error::ModuleError(err.to_string())
    }
}
