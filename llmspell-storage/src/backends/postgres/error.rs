//! ABOUTME: PostgreSQL-specific error types
//! ABOUTME: Wraps tokio-postgres, deadpool, and refinery errors for unified error handling

use thiserror::Error;

/// Result type for PostgreSQL operations
pub type Result<T> = std::result::Result<T, PostgresError>;

/// PostgreSQL-specific errors
#[derive(Error, Debug)]
pub enum PostgresError {
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

    /// Generic error
    #[error("PostgreSQL error: {0}")]
    Other(String),
}

impl From<tokio_postgres::Error> for PostgresError {
    fn from(err: tokio_postgres::Error) -> Self {
        PostgresError::Query(err.to_string())
    }
}

impl From<deadpool_postgres::PoolError> for PostgresError {
    fn from(err: deadpool_postgres::PoolError) -> Self {
        PostgresError::Pool(err.to_string())
    }
}

impl From<deadpool_postgres::BuildError> for PostgresError {
    fn from(err: deadpool_postgres::BuildError) -> Self {
        PostgresError::Config(err.to_string())
    }
}
