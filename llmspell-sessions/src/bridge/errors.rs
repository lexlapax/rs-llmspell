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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SessionId, SessionStatus};
    use serde_json::json;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_script_error_new() {
        let error = ScriptError::new("TEST_ERROR", "Test message");
        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_script_error_with_details() {
        let error =
            ScriptError::new("TEST_ERROR", "Test message").with_details(json!({ "key": "value" }));

        assert_eq!(error.code, "TEST_ERROR");
        assert_eq!(error.message, "Test message");
        assert_eq!(error.details, Some(json!({ "key": "value" })));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_script_error_display() {
        let error = ScriptError::new("TEST_ERROR", "Test message");
        assert_eq!(error.to_string(), "[TEST_ERROR] Test message");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_session_error_to_script_error() {
        // Test SessionNotFound
        let session_id = SessionId::new();
        let error = SessionError::SessionNotFound {
            id: session_id.to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "SESSION_NOT_FOUND");
        assert!(script_error.message.contains(&session_id.to_string()));

        // Test SessionAlreadyExists
        let error = SessionError::SessionAlreadyExists {
            id: session_id.to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "SESSION_ALREADY_EXISTS");
        assert!(script_error.message.contains(&session_id.to_string()));

        // Test InvalidStateTransition
        let error = SessionError::InvalidStateTransition {
            from: SessionStatus::Active,
            to: SessionStatus::Completed,
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "INVALID_STATE_TRANSITION");
        assert!(script_error.details.is_some());
        let details = script_error.details.unwrap();
        assert_eq!(details["from_state"], "Active");
        assert_eq!(details["to_state"], "Completed");

        // Test InvalidOperation
        let error = SessionError::InvalidOperation {
            reason: "Test reason".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "INVALID_OPERATION");
        assert_eq!(script_error.message, "Test reason");

        // Test Storage error
        let error = SessionError::Storage("Storage failure".to_string());
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "STORAGE_ERROR");
        assert_eq!(script_error.message, "Storage failure");

        // Test Hook error
        let error = SessionError::Hook("Hook failed".to_string());
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "HOOK_EXECUTION_ERROR");
        assert_eq!(script_error.message, "Hook failed");

        // Test Event error
        let error = SessionError::Event("Event dispatch failed".to_string());
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "EVENT_DISPATCH_ERROR");
        assert_eq!(script_error.message, "Event dispatch failed");

        // Test Serialization error
        let error = SessionError::Serialization("JSON error".to_string());
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "SERIALIZATION_ERROR");
        assert_eq!(script_error.message, "JSON error");

        // Test Validation error
        let error = SessionError::Validation("Invalid input".to_string());
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "VALIDATION_ERROR");
        assert_eq!(script_error.message, "Invalid input");

        // Test AccessDenied
        let error = SessionError::AccessDenied {
            message: "No permission".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "ACCESS_DENIED");
        assert_eq!(script_error.message, "No permission");

        // Test ResourceLimitExceeded
        let error = SessionError::ResourceLimitExceeded {
            resource: "memory".to_string(),
            message: "Out of memory".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "RESOURCE_LIMIT_EXCEEDED");
        assert!(script_error.message.contains("memory"));
        assert!(script_error.message.contains("Out of memory"));
        assert!(script_error.details.is_some());

        // Test Timeout
        let error = SessionError::Timeout {
            operation: "save".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "TIMEOUT");
        assert!(script_error.message.contains("save"));

        // Test IntegrityError
        let error = SessionError::IntegrityError {
            message: "Checksum mismatch".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "INTEGRITY_ERROR");
        assert_eq!(script_error.message, "Checksum mismatch");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_invalid_session_state_error() {
        let session_id = SessionId::new();
        let error = SessionError::InvalidSessionState {
            id: session_id.to_string(),
            state: SessionStatus::Failed,
            operation: "save".to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "INVALID_SESSION_STATE");
        assert!(script_error.message.contains(&session_id.to_string()));
        assert!(script_error.message.contains("Failed"));
        assert!(script_error.message.contains("save"));

        let details = script_error.details.unwrap();
        assert_eq!(details["session_id"], session_id.to_string());
        assert_eq!(details["state"], "Failed");
        assert_eq!(details["operation"], "save");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_artifact_errors() {
        // Test ArtifactNotFound
        let artifact_id = crate::ArtifactId::new("test_hash".to_string(), SessionId::new(), 1);
        let error = SessionError::ArtifactNotFound {
            id: artifact_id.to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "ARTIFACT_NOT_FOUND");
        assert!(script_error.message.contains(&artifact_id.to_string()));

        // Test ArtifactAlreadyExists
        let session_id = SessionId::new();
        let error = SessionError::ArtifactAlreadyExists {
            id: artifact_id.to_string(),
            session_id: session_id.to_string(),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "ARTIFACT_ALREADY_EXISTS");
        assert!(script_error.message.contains(&artifact_id.to_string()));
        assert!(script_error.message.contains(&session_id.to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_io_error_conversion() {
        use std::io;
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = SessionError::Io(io_error);
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "IO_ERROR");
        assert!(script_error.message.contains("File not found"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_builder_not_found() {
        let error = ErrorBuilder::not_found("Session", "12345");
        assert_eq!(error.code, error_codes::SESSION_NOT_FOUND);
        assert!(error.message.contains("Session"));
        assert!(error.message.contains("12345"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_builder_invalid_input() {
        let error = ErrorBuilder::invalid_input("session_id", "not a valid UUID");
        assert_eq!(error.code, error_codes::INVALID_INPUT);
        assert!(error.message.contains("session_id"));
        assert!(error.message.contains("not a valid UUID"));

        let details = error.details.unwrap();
        assert_eq!(details["field"], "session_id");
        assert_eq!(details["reason"], "not a valid UUID");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_builder_permission_denied() {
        let error = ErrorBuilder::permission_denied("write", "session_123");
        assert_eq!(error.code, error_codes::PERMISSION_DENIED);
        assert!(error.message.contains("write"));
        assert!(error.message.contains("session_123"));

        let details = error.details.unwrap();
        assert_eq!(details["action"], "write");
        assert_eq!(details["resource"], "session_123");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_builder_conversion_error() {
        let error =
            ErrorBuilder::conversion_error("Lua table", "SessionConfig", "missing field 'name'");
        assert_eq!(error.code, error_codes::SCRIPT_CONVERSION_ERROR);
        assert!(error.message.contains("Lua table"));
        assert!(error.message.contains("SessionConfig"));
        assert!(error.message.contains("missing field 'name'"));

        let details = error.details.unwrap();
        assert_eq!(details["from_type"], "Lua table");
        assert_eq!(details["to_type"], "SessionConfig");
        assert_eq!(details["reason"], "missing field 'name'");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_codes_constants() {
        // Verify error code constants are correctly defined
        assert_eq!(error_codes::SESSION_NOT_FOUND, "SESSION_NOT_FOUND");
        assert_eq!(
            error_codes::SESSION_ALREADY_EXISTS,
            "SESSION_ALREADY_EXISTS"
        );
        assert_eq!(error_codes::INVALID_OPERATION, "INVALID_OPERATION");
        assert_eq!(error_codes::CONFIGURATION_ERROR, "CONFIGURATION_ERROR");
        assert_eq!(error_codes::STORAGE_ERROR, "STORAGE_ERROR");
        assert_eq!(error_codes::SERIALIZATION_ERROR, "SERIALIZATION_ERROR");
        assert_eq!(error_codes::IO_ERROR, "IO_ERROR");
        assert_eq!(error_codes::PERMISSION_DENIED, "PERMISSION_DENIED");
        assert_eq!(
            error_codes::RESOURCE_LIMIT_EXCEEDED,
            "RESOURCE_LIMIT_EXCEEDED"
        );
        assert_eq!(error_codes::INVALID_INPUT, "INVALID_INPUT");
        assert_eq!(
            error_codes::INVALID_STATE_TRANSITION,
            "INVALID_STATE_TRANSITION"
        );
        assert_eq!(error_codes::HOOK_EXECUTION_ERROR, "HOOK_EXECUTION_ERROR");
        assert_eq!(error_codes::EVENT_DISPATCH_ERROR, "EVENT_DISPATCH_ERROR");
        assert_eq!(error_codes::ARTIFACT_ERROR, "ARTIFACT_ERROR");
        assert_eq!(error_codes::INTERNAL_ERROR, "INTERNAL_ERROR");
        assert_eq!(
            error_codes::SCRIPT_CONVERSION_ERROR,
            "SCRIPT_CONVERSION_ERROR"
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_general_error_with_source() {
        let source_error = std::io::Error::new(std::io::ErrorKind::Other, "source error");
        let error = SessionError::General {
            message: "General failure".to_string(),
            source: Some(anyhow::Error::new(source_error)),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "GENERAL_ERROR");
        assert_eq!(script_error.message, "General failure");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_replay_error_with_source() {
        let source_error = std::io::Error::new(std::io::ErrorKind::Other, "replay source");
        let error = SessionError::ReplayError {
            message: "Replay failed".to_string(),
            source: Some(Box::new(source_error)),
        };
        let script_error: ScriptError = error.into();
        assert_eq!(script_error.code, "REPLAY_ERROR");
        assert_eq!(script_error.message, "Replay failed");
    }
}
