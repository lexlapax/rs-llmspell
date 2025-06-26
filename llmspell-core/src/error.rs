//! ABOUTME: Error types and handling for rs-llmspell
//! ABOUTME: Provides LLMSpellError enum and Result type alias

use thiserror::Error;

/// Error severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorSeverity {
    /// Informational - can be ignored
    Info,
    /// Warning - should be addressed but not critical
    Warning,
    /// Error - normal error that can be recovered
    Error,
    /// Critical - severe error that may require intervention
    Critical,
    /// Fatal - unrecoverable error
    Fatal,
}

/// Error category for classification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Configuration-related errors
    Configuration,
    /// Network or provider errors
    Network,
    /// Resource errors (memory, disk, etc.)
    Resource,
    /// Security and permissions
    Security,
    /// Business logic errors
    Logic,
    /// External service errors
    External,
    /// Internal system errors
    Internal,
}

/// Comprehensive error enum for all LLMSpell operations
#[derive(Debug, Error)]
pub enum LLMSpellError {
    #[error("Component error: {message}")]
    Component { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Configuration error: {message}")]
    Configuration { 
        message: String,
        #[source] 
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("LLM provider error: {message}")]
    Provider { 
        message: String,
        provider: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Script execution error: {message}")]
    Script { 
        message: String,
        language: Option<String>,
        line: Option<usize>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Tool execution error: {message}")]
    Tool { 
        message: String,
        tool_name: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Workflow execution error: {message}")]
    Workflow { 
        message: String,
        step: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Storage error: {message}")]
    Storage { 
        message: String,
        operation: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Security violation: {message}")]
    Security { 
        message: String,
        violation_type: Option<String>,
    },
    
    #[error("Validation error: {message}")]
    Validation { 
        message: String,
        field: Option<String>,
    },
    
    #[error("Resource error: {message}")]
    Resource {
        message: String,
        resource_type: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Timeout error: {message}")]
    Timeout {
        message: String,
        duration_ms: Option<u64>,
    },
    
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    #[error("Internal error: {message}")]
    Internal { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl LLMSpellError {
    /// Get the error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Configuration { .. } => ErrorCategory::Configuration,
            Self::Provider { .. } | Self::Network { .. } => ErrorCategory::Network,
            Self::Resource { .. } | Self::Timeout { .. } => ErrorCategory::Resource,
            Self::Security { .. } => ErrorCategory::Security,
            Self::Validation { .. } | Self::Component { .. } => ErrorCategory::Logic,
            Self::Tool { .. } | Self::Script { .. } | Self::Workflow { .. } => ErrorCategory::External,
            Self::Storage { .. } | Self::Internal { .. } => ErrorCategory::Internal,
        }
    }
    
    /// Get the error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Validation { .. } => ErrorSeverity::Warning,
            Self::Configuration { .. } => ErrorSeverity::Error,
            Self::Security { .. } => ErrorSeverity::Critical,
            Self::Internal { .. } => ErrorSeverity::Fatal,
            Self::Timeout { .. } => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
    
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { .. } | Self::Timeout { .. } | Self::Provider { .. } => true,
            Self::Resource { .. } => true,
            Self::Storage { operation, .. } => {
                // Some storage operations are retryable
                operation.as_ref().map_or(false, |op| {
                    op == "read" || op == "write" || op == "lock"
                })
            }
            Self::Security { .. } | Self::Configuration { .. } | Self::Validation { .. } => false,
            Self::Internal { .. } => false,
            _ => false,
        }
    }
    
    /// Get suggested retry delay in milliseconds
    pub fn retry_delay_ms(&self) -> Option<u64> {
        if !self.is_retryable() {
            return None;
        }
        
        match self {
            Self::Network { .. } => Some(1000), // 1 second
            Self::Timeout { duration_ms, .. } => {
                // Retry with double the timeout
                duration_ms.map(|d| d * 2).or(Some(5000))
            }
            Self::Provider { .. } => Some(2000), // 2 seconds
            Self::Resource { .. } => Some(500), // 500ms
            Self::Storage { .. } => Some(100), // 100ms
            _ => Some(1000), // Default 1 second
        }
    }
    
    /// Chain with another error as the source
    pub fn with_source<E>(mut self, source: E) -> Self 
    where 
        E: std::error::Error + Send + Sync + 'static 
    {
        match &mut self {
            Self::Component { source: src, .. } |
            Self::Configuration { source: src, .. } |
            Self::Provider { source: src, .. } |
            Self::Script { source: src, .. } |
            Self::Tool { source: src, .. } |
            Self::Workflow { source: src, .. } |
            Self::Storage { source: src, .. } |
            Self::Resource { source: src, .. } |
            Self::Network { source: src, .. } |
            Self::Internal { source: src, .. } => {
                *src = Some(Box::new(source));
            }
            _ => {}
        }
        self
    }
}

/// Convenience Result type alias
pub type Result<T> = std::result::Result<T, LLMSpellError>;

// Common error conversions
impl From<std::io::Error> for LLMSpellError {
    fn from(err: std::io::Error) -> Self {
        LLMSpellError::Storage {
            message: format!("IO error: {}", err),
            operation: None,
            source: Some(Box::new(err)),
        }
    }
}

impl From<serde_json::Error> for LLMSpellError {
    fn from(err: serde_json::Error) -> Self {
        LLMSpellError::Configuration {
            message: format!("JSON serialization error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

impl From<std::fmt::Error> for LLMSpellError {
    fn from(err: std::fmt::Error) -> Self {
        LLMSpellError::Internal {
            message: format!("Formatting error: {}", err),
            source: Some(Box::new(err)),
        }
    }
}

// Error creation macros
#[macro_export]
macro_rules! component_error {
    ($msg:expr) => {
        $crate::LLMSpellError::Component {
            message: $msg.to_string(),
            source: None,
        }
    };
    ($msg:expr, $source:expr) => {
        $crate::LLMSpellError::Component {
            message: $msg.to_string(),
            source: Some(Box::new($source)),
        }
    };
}

#[macro_export]
macro_rules! validation_error {
    ($msg:expr) => {
        $crate::LLMSpellError::Validation {
            message: $msg.to_string(),
            field: None,
        }
    };
    ($msg:expr, $field:expr) => {
        $crate::LLMSpellError::Validation {
            message: $msg.to_string(),
            field: Some($field.to_string()),
        }
    };
}

#[macro_export]
macro_rules! tool_error {
    ($msg:expr) => {
        $crate::LLMSpellError::Tool {
            message: $msg.to_string(),
            tool_name: None,
            source: None,
        }
    };
    ($msg:expr, $tool:expr) => {
        $crate::LLMSpellError::Tool {
            message: $msg.to_string(),
            tool_name: Some($tool.to_string()),
            source: None,
        }
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr) => {{
        use tracing::error;
        let err = &$error;
        error!(
            error = ?err,
            category = ?err.category(),
            severity = ?err.severity(),
            retryable = err.is_retryable(),
            "Error occurred"
        );
        err
    }};
    ($error:expr, $($field:tt)*) => {{
        use tracing::error;
        let err = &$error;
        error!(
            error = ?err,
            category = ?err.category(),
            severity = ?err.severity(),
            retryable = err.is_retryable(),
            $($field)*
        );
        err
    }};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_error_severity_ordering() {
        assert!(ErrorSeverity::Info < ErrorSeverity::Warning);
        assert!(ErrorSeverity::Warning < ErrorSeverity::Error);
        assert!(ErrorSeverity::Error < ErrorSeverity::Critical);
        assert!(ErrorSeverity::Critical < ErrorSeverity::Fatal);
    }
    
    #[test]
    fn test_error_categorization() {
        let config_err = LLMSpellError::Configuration {
            message: "Invalid config".to_string(),
            source: None,
        };
        assert_eq!(config_err.category(), ErrorCategory::Configuration);
        
        let network_err = LLMSpellError::Network {
            message: "Connection failed".to_string(),
            source: None,
        };
        assert_eq!(network_err.category(), ErrorCategory::Network);
        
        let security_err = LLMSpellError::Security {
            message: "Unauthorized".to_string(),
            violation_type: Some("auth".to_string()),
        };
        assert_eq!(security_err.category(), ErrorCategory::Security);
    }
    
    #[test]
    fn test_error_severity_mapping() {
        let validation_err = LLMSpellError::Validation {
            message: "Invalid input".to_string(),
            field: Some("name".to_string()),
        };
        assert_eq!(validation_err.severity(), ErrorSeverity::Warning);
        
        let security_err = LLMSpellError::Security {
            message: "Access denied".to_string(),
            violation_type: None,
        };
        assert_eq!(security_err.severity(), ErrorSeverity::Critical);
        
        let internal_err = LLMSpellError::Internal {
            message: "System failure".to_string(),
            source: None,
        };
        assert_eq!(internal_err.severity(), ErrorSeverity::Fatal);
    }
    
    #[test]
    fn test_error_retryability() {
        // Retryable errors
        let network_err = LLMSpellError::Network {
            message: "Connection timeout".to_string(),
            source: None,
        };
        assert!(network_err.is_retryable());
        assert_eq!(network_err.retry_delay_ms(), Some(1000));
        
        let timeout_err = LLMSpellError::Timeout {
            message: "Operation timed out".to_string(),
            duration_ms: Some(5000),
        };
        assert!(timeout_err.is_retryable());
        assert_eq!(timeout_err.retry_delay_ms(), Some(10000)); // Double the timeout
        
        // Non-retryable errors
        let validation_err = LLMSpellError::Validation {
            message: "Invalid format".to_string(),
            field: None,
        };
        assert!(!validation_err.is_retryable());
        assert_eq!(validation_err.retry_delay_ms(), None);
        
        let security_err = LLMSpellError::Security {
            message: "Forbidden".to_string(),
            violation_type: None,
        };
        assert!(!security_err.is_retryable());
        assert_eq!(security_err.retry_delay_ms(), None);
    }
    
    #[test]
    fn test_storage_error_retryability() {
        let read_err = LLMSpellError::Storage {
            message: "Read failed".to_string(),
            operation: Some("read".to_string()),
            source: None,
        };
        assert!(read_err.is_retryable());
        
        let delete_err = LLMSpellError::Storage {
            message: "Delete failed".to_string(),
            operation: Some("delete".to_string()),
            source: None,
        };
        assert!(!delete_err.is_retryable());
    }
    
    #[test]
    fn test_error_chaining() {
        use std::io;
        
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let storage_err = LLMSpellError::Storage {
            message: "Failed to read file".to_string(),
            operation: Some("read".to_string()),
            source: None,
        }.with_source(io_error);
        
        // Check that source is set
        match storage_err {
            LLMSpellError::Storage { source, .. } => {
                assert!(source.is_some());
            }
            _ => panic!("Expected Storage error"),
        }
    }
    
    #[test]
    fn test_error_display() {
        let provider_err = LLMSpellError::Provider {
            message: "API rate limit exceeded".to_string(),
            provider: Some("OpenAI".to_string()),
            source: None,
        };
        let display = format!("{}", provider_err);
        assert!(display.contains("LLM provider error"));
        assert!(display.contains("API rate limit exceeded"));
    }
    
    #[test]
    fn test_error_macros() {
        let comp_err = component_error!("Component failed");
        match comp_err {
            LLMSpellError::Component { message, source } => {
                assert_eq!(message, "Component failed");
                assert!(source.is_none());
            }
            _ => panic!("Expected Component error"),
        }
        
        let val_err = validation_error!("Invalid value", "username");
        match val_err {
            LLMSpellError::Validation { message, field } => {
                assert_eq!(message, "Invalid value");
                assert_eq!(field, Some("username".to_string()));
            }
            _ => panic!("Expected Validation error"),
        }
        
        let tool_err = tool_error!("Execution failed", "FileReader");
        match tool_err {
            LLMSpellError::Tool { message, tool_name, .. } => {
                assert_eq!(message, "Execution failed");
                assert_eq!(tool_name, Some("FileReader".to_string()));
            }
            _ => panic!("Expected Tool error"),
        }
    }
    
    #[test]
    fn test_script_error_with_details() {
        let script_err = LLMSpellError::Script {
            message: "Syntax error".to_string(),
            language: Some("lua".to_string()),
            line: Some(42),
            source: None,
        };
        
        match script_err {
            LLMSpellError::Script { language, line, .. } => {
                assert_eq!(language, Some("lua".to_string()));
                assert_eq!(line, Some(42));
            }
            _ => panic!("Expected Script error"),
        }
    }
    
    #[test]
    fn test_workflow_error_with_step() {
        let workflow_err = LLMSpellError::Workflow {
            message: "Step execution failed".to_string(),
            step: Some("data_processing".to_string()),
            source: None,
        };
        
        match workflow_err {
            LLMSpellError::Workflow { step, .. } => {
                assert_eq!(step, Some("data_processing".to_string()));
            }
            _ => panic!("Expected Workflow error"),
        }
    }
    
    #[test]
    fn test_error_serialization() {
        // Test that errors can be converted to strings and back
        let errors: Vec<LLMSpellError> = vec![
            LLMSpellError::Component { message: "Test".to_string(), source: None },
            LLMSpellError::Configuration { message: "Test".to_string(), source: None },
            LLMSpellError::Provider { message: "Test".to_string(), provider: None, source: None },
            LLMSpellError::Security { message: "Test".to_string(), violation_type: None },
        ];
        
        for err in errors {
            let err_string = err.to_string();
            assert!(!err_string.is_empty());
        }
    }
}