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
    #[error("Artifact not found: {id} in session {session_id}")]
    ArtifactNotFound {
        /// Artifact ID
        id: String,
        /// Session ID
        session_id: String,
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

    /// Validation error
    #[error("Validation error: {0}")]
    Validation(String),

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
