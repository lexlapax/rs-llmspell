//! ABOUTME: Error handling and conversion for session script bridge
//! ABOUTME: Maps session errors to script-friendly error representations

use crate::SessionError;
use std::fmt;

/// Script-friendly error representation
#[derive(Debug, Clone)]
pub struct ScriptError {
    /// Error code (e.g., `SESSION_NOT_FOUND`)
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error details
    pub details: Option<serde_json::Value>,
}

impl ScriptError {
    /// Create a new script error
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Add details to the error
    #[must_use]
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl std::error::Error for ScriptError {}

/// Convert `SessionError` to `ScriptError`
impl From<SessionError> for ScriptError {
    fn from(err: SessionError) -> Self {
        match err {
            SessionError::SessionNotFound { id } => {
                ScriptError::new("SESSION_NOT_FOUND", format!("Session not found: {}", id))
            }

            SessionError::SessionAlreadyExists { id } => ScriptError::new(
                "SESSION_ALREADY_EXISTS",
                format!("Session already exists: {}", id),
            ),

            SessionError::InvalidStateTransition { from, to } => ScriptError::new(
                "INVALID_STATE_TRANSITION",
                format!("Invalid state transition from {:?} to {:?}", from, to),
            )
            .with_details(serde_json::json!({
                "from_state": format!("{:?}", from),
                "to_state": format!("{:?}", to),
            })),

            SessionError::InvalidOperation { reason } => {
                ScriptError::new("INVALID_OPERATION", reason)
            }

            SessionError::InvalidSessionState {
                id,
                state,
                operation,
            } => ScriptError::new(
                "INVALID_SESSION_STATE",
                format!(
                    "Session {} is in invalid state {:?} for operation {}",
                    id, state, operation
                ),
            )
            .with_details(serde_json::json!({
                "session_id": id,
                "state": format!("{:?}", state),
                "operation": operation,
            })),

            SessionError::ArtifactNotFound { id } => {
                ScriptError::new("ARTIFACT_NOT_FOUND", format!("Artifact not found: {}", id))
            }

            SessionError::ArtifactAlreadyExists { id, session_id } => ScriptError::new(
                "ARTIFACT_ALREADY_EXISTS",
                format!("Artifact {} already exists in session {}", id, session_id),
            ),

            SessionError::Storage(msg) => ScriptError::new("STORAGE_ERROR", msg),

            SessionError::State(state_err) => {
                ScriptError::new("STATE_ERROR", state_err.to_string())
            }

            SessionError::Hook(msg) => ScriptError::new("HOOK_EXECUTION_ERROR", msg),

            SessionError::Event(msg) => ScriptError::new("EVENT_DISPATCH_ERROR", msg),

            SessionError::Serialization(msg) => ScriptError::new("SERIALIZATION_ERROR", msg),

            SessionError::Deserialization(msg) => ScriptError::new("DESERIALIZATION_ERROR", msg),

            SessionError::Validation(msg) => ScriptError::new("VALIDATION_ERROR", msg),

            SessionError::Io(io_err) => ScriptError::new("IO_ERROR", io_err.to_string()),

            SessionError::ReplayError { message, source: _ } => {
                ScriptError::new("REPLAY_ERROR", message)
            }

            SessionError::Configuration(msg) => ScriptError::new("CONFIGURATION_ERROR", msg),

            SessionError::AccessDenied { message } => ScriptError::new("ACCESS_DENIED", message),

            SessionError::ResourceLimitExceeded { resource, message } => ScriptError::new(
                "RESOURCE_LIMIT_EXCEEDED",
                format!("Resource limit exceeded for {}: {}", resource, message),
            )
            .with_details(serde_json::json!({
                "resource": resource,
                "message": message,
            })),

            SessionError::Timeout { operation } => {
                ScriptError::new("TIMEOUT", format!("Operation timed out: {}", operation))
            }

            SessionError::IntegrityError { message } => {
                ScriptError::new("INTEGRITY_ERROR", message)
            }

            SessionError::General { message, source: _ } => {
                ScriptError::new("GENERAL_ERROR", message)
            }
        }
    }
}

/// Error codes for script bridge operations
pub mod error_codes {
    /// Session not found error
    pub const SESSION_NOT_FOUND: &str = "SESSION_NOT_FOUND";
    /// Session already exists error
    pub const SESSION_ALREADY_EXISTS: &str = "SESSION_ALREADY_EXISTS";
    /// Invalid operation error
    pub const INVALID_OPERATION: &str = "INVALID_OPERATION";
    /// Configuration error
    pub const CONFIGURATION_ERROR: &str = "CONFIGURATION_ERROR";
    /// Storage error
    pub const STORAGE_ERROR: &str = "STORAGE_ERROR";
    /// Serialization error
    pub const SERIALIZATION_ERROR: &str = "SERIALIZATION_ERROR";
    /// IO error
    pub const IO_ERROR: &str = "IO_ERROR";
    /// Permission denied error
    pub const PERMISSION_DENIED: &str = "PERMISSION_DENIED";
    /// Resource limit exceeded error
    pub const RESOURCE_LIMIT_EXCEEDED: &str = "RESOURCE_LIMIT_EXCEEDED";
    /// Invalid input error
    pub const INVALID_INPUT: &str = "INVALID_INPUT";
    /// Invalid state transition error
    pub const INVALID_STATE_TRANSITION: &str = "INVALID_STATE_TRANSITION";
    /// Hook execution error
    pub const HOOK_EXECUTION_ERROR: &str = "HOOK_EXECUTION_ERROR";
    /// Event dispatch error
    pub const EVENT_DISPATCH_ERROR: &str = "EVENT_DISPATCH_ERROR";
    /// Artifact error
    pub const ARTIFACT_ERROR: &str = "ARTIFACT_ERROR";
    /// Internal error
    pub const INTERNAL_ERROR: &str = "INTERNAL_ERROR";
    /// Script conversion error
    pub const SCRIPT_CONVERSION_ERROR: &str = "SCRIPT_CONVERSION_ERROR";
}

/// Helper to create script errors for common cases
pub struct ErrorBuilder;

impl ErrorBuilder {
    /// Create a not found error
    pub fn not_found(resource: &str, id: &str) -> ScriptError {
        ScriptError::new(
            error_codes::SESSION_NOT_FOUND,
            format!("{} not found: {}", resource, id),
        )
    }

    /// Create an invalid input error
    pub fn invalid_input(field: &str, reason: &str) -> ScriptError {
        ScriptError::new(
            error_codes::INVALID_INPUT,
            format!("Invalid {}: {}", field, reason),
        )
        .with_details(serde_json::json!({
            "field": field,
            "reason": reason,
        }))
    }

    /// Create a permission denied error
    pub fn permission_denied(action: &str, resource: &str) -> ScriptError {
        ScriptError::new(
            error_codes::PERMISSION_DENIED,
            format!("Permission denied: {} on {}", action, resource),
        )
        .with_details(serde_json::json!({
            "action": action,
            "resource": resource,
        }))
    }

    /// Create a conversion error
    pub fn conversion_error(from_type: &str, to_type: &str, reason: &str) -> ScriptError {
        ScriptError::new(
            error_codes::SCRIPT_CONVERSION_ERROR,
            format!("Failed to convert {} to {}: {}", from_type, to_type, reason),
        )
        .with_details(serde_json::json!({
            "from_type": from_type,
            "to_type": to_type,
            "reason": reason,
        }))
    }
}
