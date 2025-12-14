//! SQLite-specific error types

use thiserror::Error;

/// Result type for SQLite operations
pub type Result<T> = std::result::Result<T, SqliteError>;

/// SQLite-specific errors
#[derive(Error, Debug)]
pub enum SqliteError {
    /// Database connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query execution error
    #[error("Query error: {0}")]
    Query(String),

    /// Connection pool error
    #[error("Pool error: {0}")]
    Pool(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Extension loading error
    #[error("Extension error: {0}")]
    Extension(String),

    /// Generic error
    #[error("SQLite error: {0}")]
    Other(String),
}

impl From<rusqlite::Error> for SqliteError {
    fn from(err: rusqlite::Error) -> Self {
        SqliteError::Query(err.to_string()) // Assuming Query is the closest match for a generic rusqlite error
    }
}

impl From<r2d2::Error> for SqliteError {
    fn from(err: r2d2::Error) -> Self {
        SqliteError::Pool(err.to_string())
    }
}
