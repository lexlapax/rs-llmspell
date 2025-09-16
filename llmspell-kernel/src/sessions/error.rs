//! ABOUTME: Error types for session management operations including session lifecycle, artifact storage, and replay
//! ABOUTME: Provides comprehensive error handling with context and recovery information

use llmspell_state_traits::StateError;
use thiserror::Error;

/// Result type alias for session operations
pub type Result<T> = std::result::Result<T, SessionError>;

/// Comprehensive error type for session operations
#[derive(Debug, Error)]
pub enum SessionError {
    /// Session not found
    #[error("Session not found: {id}")]
    SessionNotFound {
        /// Session ID that was not found
        id: String,
    },

    /// Session already exists
    #[error("Session already exists: {id}")]
    SessionAlreadyExists {
        /// Session ID that already exists
        id: String,
    },

    /// Invalid session state transition
    #[error("Invalid session state transition from {from:?} to {to:?}")]
    InvalidStateTransition {
        /// Current state
        from: crate::SessionStatus,
        /// Attempted new state
        to: crate::SessionStatus,
    },

    /// Invalid operation
    #[error("Invalid operation: {reason}")]
    InvalidOperation {
        /// Reason for the invalid operation
        reason: String,
    },

    /// Session is in invalid state for operation
    #[error("Session {id} is in invalid state {state:?} for operation {operation}")]
    InvalidSessionState {
        /// Session ID
        id: String,
        /// Current session state
        state: crate::SessionStatus,
        /// Operation that was attempted
        operation: String,
    },

    /// Artifact not found
    #[error("Artifact not found: {id}")]
    ArtifactNotFound {
        /// Artifact ID
        id: String,
    },

    /// Artifact already exists
    #[error("Artifact already exists: {id} in session {session_id}")]
    ArtifactAlreadyExists {
        /// Artifact ID
        id: String,
        /// Session ID
        session_id: String,
    },

    /// Storage backend error
    #[error("Storage error: {0}")]
    Storage(String), // TODO: Add #[from] StorageError when available

    /// State persistence error
    #[error("State error: {0}")]
    State(#[from] StateError),

    /// Hook execution error
    #[error("Hook error: {0}")]
    Hook(String), // TODO: Add #[from] HookError when available

    /// Event system error
    #[error("Event error: {0}")]
    Event(String), // TODO: Add #[from] EventError when available

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Deserialization error
    #[error("Deserialization error: {0}")]
    Deserialization(String),

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

    /// I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Replay error
    #[error("Replay error: {message}")]
    ReplayError {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Access denied error
    #[error("Access denied: {message}")]
    AccessDenied {
        /// Error message
        message: String,
    },

    /// Resource limit exceeded
    #[error("Resource limit exceeded: {resource} - {message}")]
    ResourceLimitExceeded {
        /// Resource type
        resource: String,
        /// Error message
        message: String,
    },

    /// Timeout error
    #[error("Operation timed out: {operation}")]
    Timeout {
        /// Operation that timed out
        operation: String,
    },

    /// Data integrity error
    #[error("Data integrity error: {message}")]
    IntegrityError {
        /// Error message
        message: String,
    },

    /// General error with context
    #[error("{message}")]
    General {
        /// Error message
        message: String,
        /// Optional source error
        #[source]
        source: Option<anyhow::Error>,
    },
}

impl SessionError {
    /// Create a general error with context
    pub fn general(message: impl Into<String>) -> Self {
        Self::General {
            message: message.into(),
            source: None,
        }
    }

    /// Create a general error with source
    pub fn general_with_source(message: impl Into<String>, source: anyhow::Error) -> Self {
        Self::General {
            message: message.into(),
            source: Some(source),
        }
    }

    /// Create a replay error
    pub fn replay(message: impl Into<String>) -> Self {
        Self::ReplayError {
            message: message.into(),
            source: None,
        }
    }

    /// Create a replay error with source
    pub fn replay_with_source(
        message: impl Into<String>,
        source: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::ReplayError {
            message: message.into(),
            source: Some(source),
        }
    }
}

// Implement From for serde_json errors
impl From<serde_json::Error> for SessionError {
    fn from(err: serde_json::Error) -> Self {
        if err.is_data() || err.is_eof() {
            Self::Deserialization(err.to_string())
        } else {
            Self::Serialization(err.to_string())
        }
    }
}

// Implement From for bincode errors
impl From<bincode::Error> for SessionError {
    fn from(err: bincode::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sessions::SessionStatus;
    #[test]
    fn test_session_error_display() {
        // Test each error variant's display output
        let error = SessionError::SessionNotFound {
            id: "test-id".to_string(),
        };
        assert_eq!(error.to_string(), "Session not found: test-id");

        let error = SessionError::SessionAlreadyExists {
            id: "test-id".to_string(),
        };
        assert_eq!(error.to_string(), "Session already exists: test-id");

        let error = SessionError::InvalidStateTransition {
            from: SessionStatus::Active,
            to: SessionStatus::Completed,
        };
        assert_eq!(
            error.to_string(),
            "Invalid session state transition from Active to Completed"
        );

        let error = SessionError::InvalidOperation {
            reason: "test reason".to_string(),
        };
        assert_eq!(error.to_string(), "Invalid operation: test reason");

        let error = SessionError::Storage("storage error".to_string());
        assert_eq!(error.to_string(), "Storage error: storage error");

        let error = SessionError::Configuration("bad config".to_string());
        assert_eq!(error.to_string(), "Configuration error: bad config");

        let error = SessionError::Validation("invalid data".to_string());
        assert_eq!(error.to_string(), "Validation error: invalid data");
    }
    #[test]
    fn test_invalid_session_state_error() {
        let error = SessionError::InvalidSessionState {
            id: "session-123".to_string(),
            state: SessionStatus::Failed,
            operation: "save".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Session session-123 is in invalid state Failed for operation save"
        );
    }
    #[test]
    fn test_artifact_errors() {
        let error = SessionError::ArtifactNotFound {
            id: "artifact-456".to_string(),
        };
        assert_eq!(error.to_string(), "Artifact not found: artifact-456");

        let error = SessionError::ArtifactAlreadyExists {
            id: "artifact-789".to_string(),
            session_id: "session-123".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Artifact already exists: artifact-789 in session session-123"
        );
    }
    #[test]
    fn test_resource_and_access_errors() {
        let error = SessionError::AccessDenied {
            message: "insufficient permissions".to_string(),
        };
        assert_eq!(error.to_string(), "Access denied: insufficient permissions");

        let error = SessionError::ResourceLimitExceeded {
            resource: "memory".to_string(),
            message: "exceeded 1GB limit".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "Resource limit exceeded: memory - exceeded 1GB limit"
        );

        let error = SessionError::Timeout {
            operation: "session save".to_string(),
        };
        assert_eq!(error.to_string(), "Operation timed out: session save");

        let error = SessionError::IntegrityError {
            message: "checksum mismatch".to_string(),
        };
        assert_eq!(error.to_string(), "Data integrity error: checksum mismatch");
    }
    #[test]
    fn test_general_error_constructors() {
        let error = SessionError::general("general error");
        match error {
            SessionError::General { message, source } => {
                assert_eq!(message, "general error");
                assert!(source.is_none());
            }
            _ => panic!("Expected General error"),
        }

        let source_err = anyhow::anyhow!("source error");
        let error = SessionError::general_with_source("wrapped error", source_err);
        match error {
            SessionError::General { message, source } => {
                assert_eq!(message, "wrapped error");
                assert!(source.is_some());
            }
            _ => panic!("Expected General error"),
        }
    }
    #[test]
    fn test_replay_error_constructors() {
        let error = SessionError::replay("replay failed");
        match error {
            SessionError::ReplayError { message, source } => {
                assert_eq!(message, "replay failed");
                assert!(source.is_none());
            }
            _ => panic!("Expected ReplayError"),
        }

        let source_err: Box<dyn std::error::Error + Send + Sync> =
            Box::new(std::io::Error::other("io error"));
        let error = SessionError::replay_with_source("replay io error", source_err);
        match error {
            SessionError::ReplayError { message, source } => {
                assert_eq!(message, "replay io error");
                assert!(source.is_some());
            }
            _ => panic!("Expected ReplayError"),
        }
    }
    #[test]
    fn test_from_serde_json_error() {
        // Test deserialization error (EOF)
        let json_str = r#"{"incomplete":"#;
        let result: std::result::Result<serde_json::Value, _> = serde_json::from_str(json_str);
        let error: SessionError = result.unwrap_err().into();
        match error {
            SessionError::Deserialization(msg) => {
                assert!(msg.contains("EOF"));
            }
            _ => panic!("Expected Deserialization error"),
        }

        // Test serialization error (would need a type that can't be serialized)
        // Most serde_json errors are actually data/parsing errors
    }
    #[test]
    fn test_from_io_error() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let error: SessionError = io_error.into();
        match error {
            SessionError::Io(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
                assert_eq!(e.to_string(), "file not found");
            }
            _ => panic!("Expected Io error"),
        }
    }
    #[test]
    fn test_from_bincode_error() {
        // Create a bincode error by trying to deserialize invalid data
        let bad_data = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result: std::result::Result<String, _> = bincode::deserialize(&bad_data);
        let error: SessionError = result.unwrap_err().into();
        match error {
            SessionError::Serialization(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected Serialization error"),
        }
    }
    #[test]
    fn test_error_is_send_sync() {
        // Verify that SessionError implements Send + Sync
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SessionError>();
    }
    #[test]
    fn test_result_type_alias() {
        // Test that Result<T> works as expected
        fn test_function() -> String {
            "success".to_string()
        }

        fn test_error_function() -> Result<String> {
            Err(SessionError::general("test error"))
        }

        assert_eq!(test_function(), "success");
        assert!(test_error_function().is_err());
    }
    #[test]
    fn test_from_state_error() {
        use llmspell_state_traits::StateError;

        let state_error = StateError::NotFound {
            scope: "test_scope".to_string(),
            key: "test_key".to_string(),
        };
        let error: SessionError = state_error.into();
        match error {
            SessionError::State(e) => match e {
                StateError::NotFound { scope, key } => {
                    assert_eq!(scope, "test_scope");
                    assert_eq!(key, "test_key");
                }
                _ => panic!("Expected NotFound state error"),
            },
            _ => panic!("Expected State error"),
        }
    }
    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        // Test that errors with sources properly implement Error trait
        let io_error = std::io::Error::other("root cause");
        let source_err = anyhow::Error::new(io_error);
        let error = SessionError::general_with_source("high level error", source_err);

        // Verify error source chain works
        assert!(error.source().is_some());
    }
    #[test]
    fn test_hook_and_event_errors() {
        let error = SessionError::Hook("hook failed".to_string());
        assert_eq!(error.to_string(), "Hook error: hook failed");

        let error = SessionError::Event("event dispatch failed".to_string());
        assert_eq!(error.to_string(), "Event error: event dispatch failed");
    }
    #[test]
    fn test_serialization_deserialization_errors() {
        let error = SessionError::Serialization("failed to serialize".to_string());
        assert_eq!(
            error.to_string(),
            "Serialization error: failed to serialize"
        );

        let error = SessionError::Deserialization("failed to deserialize".to_string());
        assert_eq!(
            error.to_string(),
            "Deserialization error: failed to deserialize"
        );
    }
}
