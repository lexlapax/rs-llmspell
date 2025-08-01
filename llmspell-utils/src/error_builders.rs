// ABOUTME: Error construction helpers and builder patterns for consistent error handling
// ABOUTME: Provides ergonomic error building utilities used throughout LLMSpell

//! Error construction and builder utilities
//!
//! This module provides convenient error builders and context helpers
//! for consistent error handling across the framework.

use std::error::Error;
use std::fmt;

/// Error builder for fluent error construction
///
/// # Examples
///
/// ```rust
/// use llmspell_utils::error_builders::ErrorBuilder;
///
/// let error = ErrorBuilder::new("Operation failed")
///     .with_source_string("connection refused")
///     .with_context("key", "value")
///     .with_context("retry_count", 3)
///     .build();
///
/// assert_eq!(error.to_string(), "Operation failed (key: value, retry_count: 3)");
/// ```
#[derive(Debug)]
pub struct ErrorBuilder {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
    context: Vec<(String, String)>,
}

impl ErrorBuilder {
    /// Create a new error builder with the given message
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            source: None,
            context: Vec::new(),
        }
    }

    /// Add a source error
    #[must_use]
    pub fn with_source(mut self, source: impl Into<Box<dyn Error + Send + Sync>>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add a source error from a string
    #[must_use]
    pub fn with_source_string(mut self, source: impl Into<String>) -> Self {
        self.source = Some(Box::new(SimpleError::new(source.into())));
        self
    }

    /// Add context information
    #[must_use]
    pub fn with_context(mut self, key: impl Into<String>, value: impl fmt::Display) -> Self {
        self.context.push((key.into(), value.to_string()));
        self
    }

    /// Add multiple context entries
    #[must_use]
    pub fn with_contexts<I, K, V>(mut self, contexts: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: fmt::Display,
    {
        for (key, value) in contexts {
            self.context.push((key.into(), value.to_string()));
        }
        self
    }

    /// Build the final error
    #[must_use]
    pub fn build(self) -> BuiltError {
        BuiltError {
            message: self.message,
            source: self.source,
            context: self.context,
        }
    }
}

/// Error built by `ErrorBuilder`
#[derive(Debug)]
pub struct BuiltError {
    message: String,
    source: Option<Box<dyn Error + Send + Sync>>,
    context: Vec<(String, String)>,
}

impl BuiltError {
    /// Get the error message
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the error context
    #[must_use]
    pub fn context(&self) -> &[(String, String)] {
        &self.context
    }

    /// Get a specific context value
    #[must_use]
    pub fn get_context(&self, key: &str) -> Option<&str> {
        self.context
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }
}

impl fmt::Display for BuiltError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)?;

        if !self.context.is_empty() {
            write!(f, " (")?;
            for (i, (key, value)) in self.context.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{key}: {value}")?;
            }
            write!(f, ")")?;
        }

        Ok(())
    }
}

impl Error for BuiltError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

/// Simple error type for string errors
#[derive(Debug)]
struct SimpleError {
    message: String,
}

impl SimpleError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for SimpleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SimpleError {}

/// Trait for adding context to errors
pub trait WithContext<T> {
    /// Add context to the error
    ///
    /// # Errors
    ///
    /// Returns an error with additional context
    fn with_context<C, F>(self, f: F) -> Result<T, BuiltError>
    where
        C: fmt::Display,
        F: FnOnce() -> C;

    /// Add context with a static string
    ///
    /// # Errors
    ///
    /// Returns an error with additional context
    fn context(self, context: &'static str) -> Result<T, BuiltError>;
}

impl<T, E> WithContext<T> for Result<T, E>
where
    E: Error + Send + Sync + 'static,
{
    fn with_context<C, F>(self, f: F) -> Result<T, BuiltError>
    where
        C: fmt::Display,
        F: FnOnce() -> C,
    {
        self.map_err(|e| ErrorBuilder::new(f().to_string()).with_source(e).build())
    }

    fn context(self, context: &'static str) -> Result<T, BuiltError> {
        self.with_context(|| context)
    }
}

/// Common error templates
pub mod templates {
    use super::ErrorBuilder;

    /// Create an I/O error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::io_error;
    ///
    /// let error = io_error("Failed to read file", "/path/to/file");
    /// ```
    pub fn io_error(
        operation: impl Into<String>,
        path: impl AsRef<std::path::Path>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(operation.into())
            .with_context("path", path.as_ref().display())
            .with_context("error_type", "io")
    }

    /// Create a validation error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::validation_error;
    ///
    /// let error = validation_error("Invalid email format", "user@", "must contain domain");
    /// ```
    pub fn validation_error(
        message: impl Into<String>,
        value: impl std::fmt::Display,
        reason: impl Into<String>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(message.into())
            .with_context("value", value)
            .with_context("reason", reason.into())
            .with_context("error_type", "validation")
    }

    /// Create a configuration error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::config_error;
    ///
    /// let error = config_error("Missing required field", "database.url");
    /// ```
    pub fn config_error(message: impl Into<String>, field: impl Into<String>) -> ErrorBuilder {
        ErrorBuilder::new(message.into())
            .with_context("field", field.into())
            .with_context("error_type", "configuration")
    }

    /// Create a network error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::network_error;
    ///
    /// let error = network_error("Connection failed", "api.example.com", 443);
    /// ```
    pub fn network_error(
        message: impl Into<String>,
        host: impl Into<String>,
        port: u16,
    ) -> ErrorBuilder {
        ErrorBuilder::new(message.into())
            .with_context("host", host.into())
            .with_context("port", port)
            .with_context("error_type", "network")
    }

    /// Create a timeout error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::timeout_error;
    /// use std::time::Duration;
    ///
    /// let error = timeout_error("Operation timed out", Duration::from_secs(30));
    /// ```
    pub fn timeout_error(
        operation: impl Into<String>,
        duration: std::time::Duration,
    ) -> ErrorBuilder {
        ErrorBuilder::new(format!("{} after {:?}", operation.into(), duration))
            .with_context("timeout", format!("{duration:?}"))
            .with_context("error_type", "timeout")
    }

    /// Create a permission error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::permission_error;
    ///
    /// let error = permission_error("Access denied", "write", "/etc/passwd");
    /// ```
    pub fn permission_error(
        message: impl Into<String>,
        operation: impl Into<String>,
        resource: impl Into<String>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(message.into())
            .with_context("operation", operation.into())
            .with_context("resource", resource.into())
            .with_context("error_type", "permission")
    }

    /// Create a not found error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::not_found_error;
    ///
    /// let error = not_found_error("User", "user123");
    /// ```
    pub fn not_found_error(
        resource_type: impl Into<String>,
        identifier: impl Into<String>,
    ) -> ErrorBuilder {
        let resource_type_str = resource_type.into();
        ErrorBuilder::new(format!("{} not found", &resource_type_str))
            .with_context("resource_type", resource_type_str)
            .with_context("identifier", identifier.into())
            .with_context("error_type", "not_found")
    }

    /// Create a parsing error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_utils::error_builders::templates::parse_error;
    ///
    /// let error = parse_error("Invalid JSON", 10, 5, "expected '}'");
    /// ```
    pub fn parse_error(
        message: impl Into<String>,
        line: usize,
        column: usize,
        expected: impl Into<String>,
    ) -> ErrorBuilder {
        ErrorBuilder::new(message.into())
            .with_context("line", line)
            .with_context("column", column)
            .with_context("expected", expected.into())
            .with_context("error_type", "parse")
    }
}

/// Helper macros for error construction
#[macro_export]
macro_rules! build_error {
    ($msg:expr) => {
        $crate::error_builders::ErrorBuilder::new($msg).build()
    };
    ($msg:expr, source: $source:expr) => {
        $crate::error_builders::ErrorBuilder::new($msg)
            .with_source($source)
            .build()
    };
    ($msg:expr, $($key:ident: $value:expr),+ $(,)?) => {
        $crate::error_builders::ErrorBuilder::new($msg)
            $(.with_context(stringify!($key), $value))+
            .build()
    };
}

/// Bail out with an error
#[macro_export]
macro_rules! bail_error {
    ($($arg:tt)*) => {
        return Err($crate::build_error!($($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_error_builder_basic() {
        let error = ErrorBuilder::new("Test error").build();
        assert_eq!(error.message(), "Test error");
        assert_eq!(error.to_string(), "Test error");
    }
    #[test]
    fn test_error_builder_with_source() {
        let source = SimpleError::new("Source error".to_string());
        let error = ErrorBuilder::new("Main error").with_source(source).build();

        assert_eq!(error.message(), "Main error");
        assert!(error.source().is_some());
    }
    #[test]
    fn test_error_builder_with_context() {
        let error = ErrorBuilder::new("Error with context")
            .with_context("key1", "value1")
            .with_context("key2", 42)
            .build();

        assert_eq!(error.get_context("key1"), Some("value1"));
        assert_eq!(error.get_context("key2"), Some("42"));
        assert_eq!(error.get_context("nonexistent"), None);

        let display = error.to_string();
        assert!(display.contains("Error with context"));
        assert!(display.contains("key1: value1"));
        assert!(display.contains("key2: 42"));
    }
    #[test]
    fn test_with_context_trait() {
        let result: Result<(), _> = Err(SimpleError::new("Original error".to_string()));
        let error = result.with_context(|| "Additional context").unwrap_err();

        assert_eq!(error.message(), "Additional context");
        assert!(error.source().is_some());
    }
    #[test]
    fn test_error_templates() {
        let io_err = templates::io_error("Failed to read", "/tmp/test.txt").build();
        assert_eq!(io_err.get_context("error_type"), Some("io"));
        assert_eq!(io_err.get_context("path"), Some("/tmp/test.txt"));

        let val_err = templates::validation_error("Invalid", "test@", "missing domain").build();
        assert_eq!(val_err.get_context("error_type"), Some("validation"));
        assert_eq!(val_err.get_context("value"), Some("test@"));

        let config_err = templates::config_error("Missing field", "db.url").build();
        assert_eq!(config_err.get_context("error_type"), Some("configuration"));
        assert_eq!(config_err.get_context("field"), Some("db.url"));

        let net_err = templates::network_error("Connection failed", "localhost", 8080).build();
        assert_eq!(net_err.get_context("error_type"), Some("network"));
        assert_eq!(net_err.get_context("host"), Some("localhost"));
        assert_eq!(net_err.get_context("port"), Some("8080"));

        let timeout_err =
            templates::timeout_error("Read timeout", std::time::Duration::from_secs(30)).build();
        assert_eq!(timeout_err.get_context("error_type"), Some("timeout"));

        let perm_err = templates::permission_error("Denied", "write", "/etc/passwd").build();
        assert_eq!(perm_err.get_context("error_type"), Some("permission"));
        assert_eq!(perm_err.get_context("operation"), Some("write"));

        let not_found = templates::not_found_error("User", "john_doe").build();
        assert_eq!(not_found.get_context("error_type"), Some("not_found"));
        assert_eq!(not_found.get_context("identifier"), Some("john_doe"));

        let parse_err = templates::parse_error("Syntax error", 10, 5, "closing brace").build();
        assert_eq!(parse_err.get_context("error_type"), Some("parse"));
        assert_eq!(parse_err.get_context("line"), Some("10"));
        assert_eq!(parse_err.get_context("column"), Some("5"));
    }
    #[test]
    fn test_build_error_macro() {
        let error1 = build_error!("Simple error");
        assert_eq!(error1.message(), "Simple error");

        let source = SimpleError::new("Source".to_string());
        let error2 = build_error!("With source", source: source);
        assert_eq!(error2.message(), "With source");
        assert!(error2.source().is_some());

        let error3 = build_error!("With context", foo: "bar", count: 42);
        assert_eq!(error3.get_context("foo"), Some("bar"));
        assert_eq!(error3.get_context("count"), Some("42"));
    }
    #[test]
    fn test_error_display_formatting() {
        let error = ErrorBuilder::new("Main message")
            .with_context("code", "E001")
            .with_context("line", 42)
            .build();

        let display = error.to_string();
        assert_eq!(display, "Main message (code: E001, line: 42)");
    }
}

/// LLMSpellError-specific builders
pub mod llmspell {
    use llmspell_core::LLMSpellError;

    /// Create a validation error
    pub fn validation_error(message: impl Into<String>, field: Option<String>) -> LLMSpellError {
        LLMSpellError::Validation {
            message: message.into(),
            field,
        }
    }

    /// Create a storage error
    pub fn storage_error(message: impl Into<String>, operation: Option<String>) -> LLMSpellError {
        LLMSpellError::Storage {
            message: message.into(),
            operation,
            source: None,
        }
    }

    /// Create a tool error
    pub fn tool_error(message: impl Into<String>, tool_name: Option<String>) -> LLMSpellError {
        LLMSpellError::Tool {
            message: message.into(),
            tool_name,
            source: None,
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> LLMSpellError {
        LLMSpellError::Configuration {
            message: message.into(),
            source: None,
        }
    }

    /// Create a component error
    pub fn component_error(message: impl Into<String>) -> LLMSpellError {
        LLMSpellError::Component {
            message: message.into(),
            source: None,
        }
    }

    /// Create a workflow error
    pub fn workflow_error(message: impl Into<String>, step: Option<String>) -> LLMSpellError {
        LLMSpellError::Workflow {
            message: message.into(),
            step,
            source: None,
        }
    }

    /// Create a provider error
    pub fn provider_error(message: impl Into<String>, provider: Option<String>) -> LLMSpellError {
        LLMSpellError::Provider {
            message: message.into(),
            provider,
            source: None,
        }
    }

    /// Create a security error
    pub fn security_error(
        message: impl Into<String>,
        violation_type: Option<String>,
    ) -> LLMSpellError {
        LLMSpellError::Security {
            message: message.into(),
            violation_type,
        }
    }

    // Note: Hook error variant doesn't exist in LLMSpellError
    // This is a placeholder for future implementation

    /// Create a script error
    pub fn script_error(
        message: impl Into<String>,
        language: Option<String>,
        line: Option<usize>,
    ) -> LLMSpellError {
        LLMSpellError::Script {
            message: message.into(),
            language,
            line,
            source: None,
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        #[test]
        fn test_validation_error() {
            let err = validation_error("Invalid input", Some("email".to_string()));
            match err {
                LLMSpellError::Validation { message, field } => {
                    assert_eq!(message, "Invalid input");
                    assert_eq!(field, Some("email".to_string()));
                }
                _ => panic!("Wrong error type"),
            }
        }
        #[test]
        fn test_tool_error() {
            let err = tool_error("Tool failed", Some("json_processor".to_string()));
            match err {
                LLMSpellError::Tool {
                    message, tool_name, ..
                } => {
                    assert_eq!(message, "Tool failed");
                    assert_eq!(tool_name, Some("json_processor".to_string()));
                }
                _ => panic!("Wrong error type"),
            }
        }
    }
}
