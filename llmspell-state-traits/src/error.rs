// ABOUTME: State management error types and result aliases
// ABOUTME: Provides common error handling for state operations across all components

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for state operations
pub type StateResult<T> = Result<T, StateError>;

/// Errors that can occur during state operations
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum StateError {
    /// State not found for the given scope and key
    #[error("State not found: scope={scope}, key={key}")]
    NotFound { scope: String, key: String },

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Storage backend errors
    #[error("Storage error: {0}")]
    StorageError(String),

    /// Invalid state scope or key format
    #[error("Invalid format: {0}")]
    InvalidFormat(String),

    /// Concurrent access conflicts
    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    /// Migration errors
    #[error("Migration error: {0}")]
    MigrationError(String),

    /// IO errors from storage operations
    #[error("IO error: {0}")]
    IoError(String),

    /// Permission/security errors
    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    /// Resource limit errors
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    /// Generic internal errors
    #[error("Internal error: {0}")]
    Internal(String),
}

impl StateError {
    /// Create a not found error
    pub fn not_found(scope: impl Into<String>, key: impl Into<String>) -> Self {
        Self::NotFound {
            scope: scope.into(),
            key: key.into(),
        }
    }

    /// Create a serialization error
    pub fn serialization(message: impl Into<String>) -> Self {
        Self::SerializationError(message.into())
    }

    /// Create a storage error
    pub fn storage(message: impl Into<String>) -> Self {
        Self::StorageError(message.into())
    }

    /// Create an invalid format error
    pub fn invalid_format(message: impl Into<String>) -> Self {
        Self::InvalidFormat(message.into())
    }

    /// Create a concurrency error
    pub fn concurrency(message: impl Into<String>) -> Self {
        Self::ConcurrencyError(message.into())
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::ConfigurationError(message.into())
    }

    /// Create a migration error
    pub fn migration(message: impl Into<String>) -> Self {
        Self::MigrationError(message.into())
    }

    /// Create an IO error
    pub fn io(message: impl Into<String>) -> Self {
        Self::IoError(message.into())
    }

    /// Create a permission denied error
    pub fn permission_denied(message: impl Into<String>) -> Self {
        Self::PermissionDenied(message.into())
    }

    /// Create a resource limit exceeded error
    pub fn resource_limit_exceeded(message: impl Into<String>) -> Self {
        Self::ResourceLimitExceeded(message.into())
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    /// Create a hook error (maps to internal error)
    pub fn hook_error(message: impl Into<String>) -> Self {
        Self::Internal(format!("Hook error: {}", message.into()))
    }

    /// Create a compression error (maps to storage error)
    pub fn compression_error(message: impl Into<String>) -> Self {
        Self::StorageError(format!("Compression error: {}", message.into()))
    }

    /// Create a lock error (maps to concurrency error)
    pub fn lock_error(message: impl Into<String>) -> Self {
        Self::ConcurrencyError(format!("Lock error: {}", message.into()))
    }

    /// Create a validation error (maps to invalid format error)
    pub fn validation_error(message: impl Into<String>) -> Self {
        Self::InvalidFormat(format!("Validation error: {}", message.into()))
    }

    /// Create a timeout error (maps to internal error)
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Internal(format!("Timeout: {}", message.into()))
    }

    /// Create an already exists error (maps to invalid format error)
    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::InvalidFormat(format!("Already exists: {}", message.into()))
    }

    /// Create a backup error (maps to storage error)
    pub fn backup_error(message: impl Into<String>) -> Self {
        Self::StorageError(format!("Backup error: {}", message.into()))
    }

    /// Create a background task error (maps to internal error)
    pub fn background_task_error(message: impl Into<String>) -> Self {
        Self::Internal(format!("Background task error: {}", message.into()))
    }

    /// Check if this error indicates a missing/not found state
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound { .. })
    }

    /// Check if this error is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::StorageError(_) | Self::ConcurrencyError(_) | Self::IoError(_)
        )
    }

    /// Check if this error indicates a permanent failure
    #[must_use]
    pub fn is_permanent(&self) -> bool {
        matches!(
            self,
            Self::SerializationError(_)
                | Self::InvalidFormat(_)
                | Self::ConfigurationError(_)
                | Self::PermissionDenied(_)
        )
    }
}

// Conversion from serde_json::Error
impl From<serde_json::Error> for StateError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_creation() {
        let err = StateError::not_found("global", "test_key");
        assert!(err.is_not_found());
        assert!(!err.is_retryable());

        let err = StateError::storage("connection failed");
        assert!(!err.is_not_found());
        assert!(err.is_retryable());

        let err = StateError::serialization("invalid json");
        assert!(err.is_permanent());
    }
    #[test]
    fn test_error_display() {
        let err = StateError::not_found("global", "test_key");
        assert_eq!(
            err.to_string(),
            "State not found: scope=global, key=test_key"
        );

        let err = StateError::serialization("invalid data");
        assert_eq!(err.to_string(), "Serialization error: invalid data");
    }
    #[test]
    fn test_error_conversion() {
        // Create a JSON parse error
        let invalid_json = "{ invalid json }";
        let json_err = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let state_err: StateError = json_err.into();
        assert!(matches!(state_err, StateError::SerializationError(_)));
    }
}
