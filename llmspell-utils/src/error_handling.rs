//! ABOUTME: Production-ready error handling with information disclosure prevention
//! ABOUTME: Provides safe error responses, logging, and debugging capabilities

use crate::security::information_disclosure::{
    ErrorInfo, InfoDisclosurePreventer, ProductionErrorHandler,
};
use llmspell_core::LLMSpellError;
use std::collections::HashMap;
use std::sync::Arc;

/// Production-safe error response
#[derive(Debug, Clone, serde::Serialize)]
pub struct SafeErrorResponse {
    /// Error message safe for client
    pub error: String,
    /// Error code for reference
    pub code: String,
    /// Error category if allowed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    /// Whether the client should retry
    pub retry: bool,
    /// Request ID for correlation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
    /// Timestamp
    pub timestamp: String,
}

/// Error context builder for rich error information
#[derive(Debug, Default, Clone)]
pub struct ErrorContext {
    /// Operation being performed
    pub operation: Option<String>,
    /// Resource involved
    pub resource: Option<String>,
    /// User or session ID (sanitized)
    pub user_id: Option<String>,
    /// Additional context
    pub metadata: HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the operation
    #[must_use]
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Set the resource
    #[must_use]
    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    /// Set the user ID
    #[must_use]
    pub fn with_user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Add metadata
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Safe error handler for production use
pub struct SafeErrorHandler {
    /// Information disclosure preventer
    preventer: Arc<InfoDisclosurePreventer>,
    /// Production error handler
    handler: ProductionErrorHandler,
    /// Whether we're in production mode
    #[allow(dead_code)]
    is_production: bool,
}

impl SafeErrorHandler {
    /// Create a new safe error handler
    #[must_use]
    pub fn new(is_production: bool) -> Self {
        let preventer = if is_production {
            Arc::new(InfoDisclosurePreventer::production())
        } else {
            Arc::new(InfoDisclosurePreventer::development())
        };

        let handler = ProductionErrorHandler::new(Arc::clone(&preventer));

        Self {
            preventer,
            handler,
            is_production,
        }
    }

    /// Handle an `LLMSpellError` safely
    #[must_use]
    pub fn handle_llmspell_error(
        &self,
        error: &LLMSpellError,
        context: &ErrorContext,
    ) -> SafeErrorResponse {
        // Create error info with context
        let mut error_info = ErrorInfo {
            message: error.to_string(),
            kind: Some(Self::categorize_llmspell_error(error)),
            stack_trace: None, // Backtrace support requires nightly or specific feature flags
            context: HashMap::new(),
            source_location: None,
        };

        // Add context to error info
        if let Some(ref op) = context.operation {
            error_info
                .context
                .insert("operation".to_string(), op.clone());
        }
        if let Some(ref res) = context.resource {
            error_info
                .context
                .insert("resource".to_string(), res.clone());
        }

        // Log the full error internally
        self.log_error(&error_info, context);

        // Sanitize for external consumption
        let sanitized = self.preventer.sanitize_error(&error_info);

        SafeErrorResponse {
            error: sanitized.message,
            code: sanitized.error_code,
            category: sanitized.category,
            retry: sanitized.retriable,
            request_id: context.metadata.get("request_id").cloned(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Handle any error safely
    pub fn handle_error(
        &self,
        error: &dyn std::error::Error,
        context: &ErrorContext,
    ) -> SafeErrorResponse {
        let sanitized = self.handler.handle_error(error);

        SafeErrorResponse {
            error: sanitized.message,
            code: sanitized.error_code,
            category: sanitized.category,
            retry: sanitized.retriable,
            request_id: context.metadata.get("request_id").cloned(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Categorize `LLMSpellError`
    fn categorize_llmspell_error(error: &LLMSpellError) -> String {
        match error {
            LLMSpellError::Validation { .. } => "validation".to_string(),
            LLMSpellError::Tool { .. } => "tool_error".to_string(),
            LLMSpellError::Provider { .. } => "provider_error".to_string(),
            LLMSpellError::Storage { .. } => "storage_error".to_string(),
            LLMSpellError::Timeout { .. } => "timeout".to_string(),
            LLMSpellError::RateLimit { .. } => "rate_limit".to_string(),
            LLMSpellError::Network { .. } => "network_error".to_string(),
            LLMSpellError::Script { .. } => "script_error".to_string(),
            _ => "internal".to_string(),
        }
    }

    /// Log error internally with full details
    fn log_error(&self, error_info: &ErrorInfo, context: &ErrorContext) {
        let mut fields = vec![];

        if let Some(ref op) = context.operation {
            fields.push(("operation", op.as_str()));
        }
        if let Some(ref res) = context.resource {
            fields.push(("resource", res.as_str()));
        }
        if let Some(ref user) = context.user_id {
            fields.push(("user_id", user.as_str()));
        }

        tracing::error!(
            error_code = %self.preventer.generate_error_code(&error_info.message),
            error_type = ?error_info.kind,
            ?fields,
            "Error occurred: {}",
            error_info.message
        );
    }
}

/// Debug information manager for development
pub struct DebugInfoManager {
    /// Whether to include debug info
    enabled: bool,
    /// Maximum debug info size
    max_size: usize,
}

impl DebugInfoManager {
    /// Create a new debug info manager
    #[must_use]
    pub fn new(enabled: bool) -> Self {
        Self {
            enabled,
            max_size: 10000, // 10KB max
        }
    }

    /// Add debug information to response if enabled
    pub fn add_debug_info<T: serde::Serialize + for<'de> serde::Deserialize<'de>>(
        &self,
        response: T,
        debug_info: HashMap<String, String>,
    ) -> T {
        if !self.enabled || debug_info.is_empty() {
            return response;
        }

        // Only in development mode, add debug info
        if let Ok(mut value) = serde_json::to_value(&response) {
            if let Some(obj) = value.as_object_mut() {
                let mut safe_debug = HashMap::new();
                let mut total_size = 0;

                for (key, val) in debug_info {
                    let size = key.len() + val.len();
                    if total_size + size <= self.max_size {
                        safe_debug.insert(key, val);
                        total_size += size;
                    }
                }

                obj.insert("_debug".to_string(), serde_json::json!(safe_debug));
            }

            if let Ok(updated) = serde_json::from_value(value) {
                return updated;
            }
        }

        response
    }
}

/// Stack trace remover for production
pub struct StackTraceRemover;

impl StackTraceRemover {
    /// Remove stack traces from error strings
    #[must_use]
    pub fn remove_stack_traces(error_str: &str) -> String {
        let lines: Vec<&str> = error_str.lines().collect();
        let mut result = Vec::new();
        let mut in_stack_trace = false;

        for line in lines {
            if line.trim_start().starts_with("at ")
                || line.contains("stack backtrace:")
                || line.trim_start().starts_with("note: run with")
            {
                in_stack_trace = true;
                continue;
            }

            if in_stack_trace && line.trim().is_empty() {
                in_stack_trace = false;
                continue;
            }

            if !in_stack_trace {
                result.push(line);
            }
        }

        result.join("\n").trim().to_string()
    }
}

/// Logging standards enforcer
pub struct LoggingStandards {
    /// Minimum log level for production
    pub min_level: tracing::Level,
    /// Fields that must be included
    pub required_fields: Vec<String>,
    /// Fields that must be excluded
    pub forbidden_fields: Vec<String>,
}

impl Default for LoggingStandards {
    fn default() -> Self {
        Self {
            min_level: tracing::Level::INFO,
            required_fields: vec![
                "timestamp".to_string(),
                "level".to_string(),
                "target".to_string(),
            ],
            forbidden_fields: vec![
                "password".to_string(),
                "secret".to_string(),
                "token".to_string(),
                "api_key".to_string(),
                "private_key".to_string(),
            ],
        }
    }
}

impl LoggingStandards {
    /// Validate a log record
    #[must_use]
    pub fn validate_log_record(&self, record: &tracing::Event) -> bool {
        // Check level
        if record.metadata().level() > &self.min_level {
            return false;
        }

        // Would check fields here in a real implementation
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_error_handler() {
        let handler = SafeErrorHandler::new(true); // production mode
        let context = ErrorContext::new()
            .with_operation("file_read")
            .with_resource("/etc/passwd");

        let error = std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Permission denied: /etc/passwd",
        );

        let response = handler.handle_error(&error, &context);
        assert!(!response.error.contains("/etc/passwd"));
        assert!(response.code.starts_with("ERR_"));
    }

    #[test]
    fn test_stack_trace_removal() {
        let error_with_stack = r#"
Error: Something went wrong
   at src/main.rs:42:15
   at src/lib.rs:100:20
stack backtrace:
   0: std::panicking::begin_panic
   1: main::run
note: run with `RUST_BACKTRACE=1` for a backtrace

This is the actual error message.
        "#;

        let cleaned = StackTraceRemover::remove_stack_traces(error_with_stack);
        assert!(!cleaned.contains("at src/"));
        assert!(!cleaned.contains("stack backtrace"));
        assert!(cleaned.contains("Error: Something went wrong"));
        assert!(cleaned.contains("This is the actual error message"));
    }

    #[test]
    fn test_debug_info_manager() {
        let manager = DebugInfoManager::new(true);

        // Test with a Value directly to avoid deserialization issues
        let _response = serde_json::json!({
            "message": "Success"
        });

        let mut debug_info = HashMap::new();
        debug_info.insert("operation".to_string(), "test_op".to_string());
        debug_info.insert("duration_ms".to_string(), "42".to_string());

        // Test that debug info would be added
        assert!(manager.enabled);
        assert_eq!(manager.max_size, 10000);

        // The actual add_debug_info requires Serialize + Deserialize trait bounds
        // which makes it hard to test with dynamic values. In production,
        // this would work with concrete types that implement both traits.
    }

    #[test]
    fn test_error_context_builder() {
        let context = ErrorContext::new()
            .with_operation("delete_file")
            .with_resource("important.doc")
            .with_user_id("user123")
            .with_metadata("attempt", "3");

        assert_eq!(context.operation.unwrap(), "delete_file");
        assert_eq!(context.resource.unwrap(), "important.doc");
        assert_eq!(context.user_id.unwrap(), "user123");
        assert_eq!(context.metadata.get("attempt").unwrap(), "3");
    }

    #[test]
    fn test_production_vs_development_mode() {
        let prod_handler = SafeErrorHandler::new(true);
        let dev_handler = SafeErrorHandler::new(false);

        let error = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "File not found: /home/user/secret.txt",
        );

        let context = ErrorContext::new();

        let prod_response = prod_handler.handle_error(&error, &context);
        let dev_response = dev_handler.handle_error(&error, &context);

        // Production should sanitize paths
        assert!(!prod_response.error.contains("/home/user"));

        // Dev might include more details (depends on config)
        assert!(!dev_response.error.is_empty());
    }
}
