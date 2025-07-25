// ABOUTME: State-specific error types for persistent state management
// ABOUTME: Provides detailed error handling for state operations

use thiserror::Error;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("Storage backend error: {0}")]
    StorageError(#[from] anyhow::Error),
    
    #[error("State serialization failed: {0}")]
    SerializationError(String),
    
    #[error("State deserialization failed: {0}")]
    DeserializationError(String),
    
    #[error("State key validation failed: {0}")]
    InvalidKey(String),
    
    #[error("State scope access denied: {0}")]
    AccessDenied(String),
    
    #[error("State migration failed: {0}")]
    MigrationError(String),
    
    #[error("State schema version mismatch: expected {expected}, got {actual}")]
    SchemaVersionMismatch { expected: u32, actual: u32 },
    
    #[error("State lock acquisition failed: {0}")]
    LockError(String),
    
    #[error("Hook execution failed during state operation: {0}")]
    HookError(String),
    
    #[error("State operation timeout after {0:?}")]
    Timeout(std::time::Duration),
    
    #[error("State corruption detected: {0}")]
    CorruptedState(String),
}

impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> Self {
        StateError::StorageError(err.into())
    }
}

pub type StateResult<T> = Result<T, StateError>;