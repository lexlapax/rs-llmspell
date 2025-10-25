//! Error types for template system

use thiserror::Error;

/// Result type for template operations
pub type Result<T> = std::result::Result<T, TemplateError>;

/// Errors that can occur during template operations
#[derive(Debug, Error)]
pub enum TemplateError {
    /// Template not found in registry
    #[error("Template not found: {0}")]
    NotFound(String),

    /// Template with this ID already registered
    #[error("Template already registered: {0}")]
    AlreadyRegistered(String),

    /// Parameter validation failed
    #[error("Parameter validation failed: {0}")]
    ValidationFailed(#[from] ValidationError),

    /// Template execution failed
    #[error("Template execution failed: {0}")]
    ExecutionFailed(String),

    /// Required infrastructure not available
    #[error("Required infrastructure not available: {0}")]
    InfrastructureUnavailable(String),

    /// Agent execution failed
    #[error("Agent execution failed: {0}")]
    AgentFailed(String),

    /// Workflow execution failed
    #[error("Workflow execution failed: {0}")]
    WorkflowFailed(String),

    /// Tool execution failed
    #[error("Tool execution failed: {0}")]
    ToolFailed(String),

    /// RAG operation failed
    #[error("RAG operation failed: {0}")]
    RAGFailed(String),

    /// LLM provider error
    #[error("LLM provider error: {0}")]
    ProviderError(String),

    /// Configuration error (Task 13.5.7d)
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// IO error
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// Generic error
    #[error("Template error: {0}")]
    Other(#[from] anyhow::Error),
}

/// Convert StateError to TemplateError
impl From<llmspell_core::state::StateError> for TemplateError {
    fn from(err: llmspell_core::state::StateError) -> Self {
        TemplateError::ExecutionFailed(format!("State operation failed: {}", err))
    }
}

/// Parameter validation errors with detailed context
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Required parameter missing
    #[error("Required parameter missing: {parameter}")]
    MissingRequired { parameter: String },

    /// Parameter type mismatch
    #[error("Parameter type mismatch for '{parameter}': expected {expected}, got {actual}")]
    TypeMismatch {
        parameter: String,
        expected: String,
        actual: String,
    },

    /// Parameter value out of range
    #[error("Parameter '{parameter}' out of range: {message}")]
    OutOfRange { parameter: String, message: String },

    /// Invalid parameter value
    #[error("Invalid value for parameter '{parameter}': {message}")]
    InvalidValue { parameter: String, message: String },

    /// Unsupported parameter
    #[error("Unsupported parameter: {0}")]
    UnsupportedParameter(String),

    /// Multiple validation errors
    #[error("Multiple validation errors: {}", .errors.join("; "))]
    Multiple { errors: Vec<String> },

    /// JSON schema validation failed
    #[error("JSON schema validation failed: {0}")]
    SchemaValidation(String),
}

impl ValidationError {
    /// Create a missing required parameter error
    pub fn missing(parameter: impl Into<String>) -> Self {
        Self::MissingRequired {
            parameter: parameter.into(),
        }
    }

    /// Create a type mismatch error
    pub fn type_mismatch(
        parameter: impl Into<String>,
        expected: impl Into<String>,
        actual: impl Into<String>,
    ) -> Self {
        Self::TypeMismatch {
            parameter: parameter.into(),
            expected: expected.into(),
            actual: actual.into(),
        }
    }

    /// Create an out of range error
    pub fn out_of_range(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        Self::OutOfRange {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// Create an invalid value error
    pub fn invalid_value(parameter: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidValue {
            parameter: parameter.into(),
            message: message.into(),
        }
    }

    /// Create a multiple errors container
    pub fn multiple(errors: Vec<String>) -> Self {
        Self::Multiple { errors }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_error_missing() {
        let err = ValidationError::missing("topic");
        assert_eq!(err.to_string(), "Required parameter missing: topic");
    }

    #[test]
    fn test_validation_error_type_mismatch() {
        let err = ValidationError::type_mismatch("max_sources", "number", "string");
        assert_eq!(
            err.to_string(),
            "Parameter type mismatch for 'max_sources': expected number, got string"
        );
    }

    #[test]
    fn test_validation_error_out_of_range() {
        let err = ValidationError::out_of_range("max_sources", "must be between 1 and 100");
        assert_eq!(
            err.to_string(),
            "Parameter 'max_sources' out of range: must be between 1 and 100"
        );
    }

    #[test]
    fn test_validation_error_invalid_value() {
        let err = ValidationError::invalid_value("model", "model not found");
        assert_eq!(
            err.to_string(),
            "Invalid value for parameter 'model': model not found"
        );
    }

    #[test]
    fn test_validation_error_multiple() {
        let err = ValidationError::multiple(vec![
            "Missing parameter: topic".to_string(),
            "Invalid value: max_sources".to_string(),
        ]);
        assert_eq!(
            err.to_string(),
            "Multiple validation errors: Missing parameter: topic; Invalid value: max_sources"
        );
    }

    #[test]
    fn test_template_error_from_validation() {
        let validation_err = ValidationError::missing("topic");
        let template_err: TemplateError = validation_err.into();
        assert!(matches!(template_err, TemplateError::ValidationFailed(_)));
    }

    #[test]
    fn test_template_error_not_found() {
        let err = TemplateError::NotFound("research-assistant".to_string());
        assert_eq!(err.to_string(), "Template not found: research-assistant");
    }

    #[test]
    fn test_template_error_already_registered() {
        let err = TemplateError::AlreadyRegistered("research-assistant".to_string());
        assert_eq!(
            err.to_string(),
            "Template already registered: research-assistant"
        );
    }
}
