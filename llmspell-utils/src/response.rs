// ABOUTME: Response builder utilities for consistent output formatting across tools
// ABOUTME: Provides fluent API for building successful and error responses

//! Response building utilities
//!
//! This module provides a fluent API for building consistent response
//! objects across all LLMSpell tools, ensuring uniform output structure.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

/// Error details for standardized error responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    /// The error message
    pub message: String,
    /// Optional error code for categorization
    pub code: Option<String>,
    /// Optional additional error details
    pub details: Option<Value>,
}

impl ErrorDetails {
    /// Create a new error with just a message
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            details: None,
        }
    }

    /// Add an error code
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// Add additional error details
    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }
}

/// A builder for creating consistent response objects
#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    operation: String,
    success: bool,
    message: Option<String>,
    result: Option<Value>,
    error: Option<ErrorDetails>,
    metadata: HashMap<String, Value>,
}

impl ResponseBuilder {
    /// Create a new successful response builder
    #[must_use]
    pub fn success(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            success: true,
            message: None,
            result: None,
            error: None,
            metadata: HashMap::new(),
        }
    }

    /// Create a new error response builder
    #[must_use]
    pub fn error(operation: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            success: false,
            message: None,
            result: None,
            error: Some(ErrorDetails::new(error)),
            metadata: HashMap::new(),
        }
    }

    /// Add a human-readable message
    #[must_use]
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Add the main result data
    #[must_use]
    pub fn with_result(mut self, result: Value) -> Self {
        self.result = Some(result);
        self
    }

    /// Add detailed error information
    #[must_use]
    pub fn with_error_details(mut self, error: ErrorDetails) -> Self {
        self.success = false;
        self.error = Some(error);
        self
    }

    /// Add a metadata field
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Add multiple metadata fields
    #[must_use]
    pub fn with_metadata_map(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Add file information to metadata
    #[must_use]
    pub fn with_file_info(mut self, path: impl Into<String>, size: Option<u64>) -> Self {
        self.metadata
            .insert("file_path".to_string(), json!(path.into()));
        if let Some(size) = size {
            self.metadata.insert("file_size".to_string(), json!(size));
        }
        self
    }

    /// Add timing information to metadata
    #[must_use]
    pub fn with_duration_ms(mut self, duration_ms: u64) -> Self {
        self.metadata
            .insert("duration_ms".to_string(), json!(duration_ms));
        self
    }

    /// Add count information to metadata
    #[must_use]
    pub fn with_count(mut self, count: usize) -> Self {
        self.metadata.insert("count".to_string(), json!(count));
        self
    }

    /// Build the final response as a JSON value
    #[must_use]
    pub fn build(self) -> Value {
        let mut response = json!({
            "operation": self.operation,
            "success": self.success,
        });

        if let Some(message) = self.message {
            response["message"] = json!(message);
        }

        if let Some(result) = self.result {
            response["result"] = result;
        }

        if let Some(error) = self.error {
            response["error"] = serde_json::to_value(error).unwrap_or_else(|_| {
                json!({
                    "message": "Failed to serialize error details"
                })
            });
        }

        if !self.metadata.is_empty() {
            response["metadata"] = json!(self.metadata);
        }

        response
    }

    /// Build the response and extract specific fields for tool output
    #[must_use]
    pub fn build_for_output(self) -> (String, Value) {
        // Generate appropriate text output
        let text = if self.success {
            self.message
                .clone()
                .unwrap_or_else(|| format!("Operation '{}' completed successfully", self.operation))
        } else {
            format!(
                "Operation '{}' failed: {}",
                self.operation,
                self.error
                    .as_ref()
                    .map_or("Unknown error", |e| e.message.as_str())
            )
        };

        let response = self.build();
        (text, response)
    }
}

/// Helper function to create a simple success response
#[must_use]
pub fn success_response(operation: impl Into<String>, message: impl Into<String>) -> Value {
    ResponseBuilder::success(operation)
        .with_message(message)
        .build()
}

/// Helper function to create a simple error response
#[must_use]
pub fn error_response(operation: impl Into<String>, error: impl Into<String>) -> Value {
    ResponseBuilder::error(operation, error).build()
}

/// Helper function to format file operation response
#[must_use]
pub fn file_operation_response(
    operation: impl Into<String>,
    file_path: impl Into<String>,
    success: bool,
    message: Option<String>,
) -> Value {
    let mut builder = if success {
        ResponseBuilder::success(operation)
    } else {
        ResponseBuilder::error(operation, message.as_deref().unwrap_or("Operation failed"))
    };

    builder = builder.with_file_info(file_path, None);

    if let Some(msg) = message {
        builder = builder.with_message(msg);
    }

    builder.build()
}

/// Helper function to format list operation response
#[must_use]
pub fn list_response<T: serde::Serialize>(
    operation: impl Into<String>,
    items: &[T],
    message: Option<String>,
) -> Value {
    let count = items.len();
    let mut builder = ResponseBuilder::success(operation)
        .with_result(json!(items))
        .with_count(count);

    if let Some(msg) = message {
        builder = builder.with_message(msg);
    } else {
        builder = builder.with_message(format!("Found {count} items"));
    }

    builder.build()
}

/// Represents a validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// The field that failed validation (if applicable)
    pub field: Option<String>,
    /// The validation error message
    pub message: String,
    /// Optional error code for categorization
    pub code: Option<String>,
}

impl ValidationError {
    /// Create a new validation error
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            field: None,
            message: message.into(),
            code: None,
        }
    }

    /// Add a field name
    #[must_use]
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    /// Add an error code
    #[must_use]
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

/// Helper function to create a validation response
#[must_use]
pub fn validation_response(valid: bool, errors: &Option<Vec<ValidationError>>) -> Value {
    ResponseBuilder::success("validate")
        .with_result(json!({
            "valid": valid,
            "errors": errors
        }))
        .build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_success_response() {
        let response = ResponseBuilder::success("test_op")
            .with_message("Test completed")
            .build();

        assert_eq!(response["operation"], "test_op");
        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "Test completed");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_response() {
        let response = ResponseBuilder::error("test_op", "Something went wrong").build();

        assert_eq!(response["operation"], "test_op");
        assert_eq!(response["success"], false);
        assert_eq!(response["error"]["message"], "Something went wrong");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_with_result() {
        let response = ResponseBuilder::success("test_op")
            .with_result(json!({ "data": [1, 2, 3] }))
            .build();

        assert_eq!(response["result"]["data"], json!([1, 2, 3]));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_with_metadata() {
        let response = ResponseBuilder::success("test_op")
            .with_metadata("key1", json!("value1"))
            .with_metadata("key2", json!(42))
            .build();

        assert_eq!(response["metadata"]["key1"], "value1");
        assert_eq!(response["metadata"]["key2"], 42);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_with_file_info() {
        let response = ResponseBuilder::success("read_file")
            .with_file_info("/path/to/file.txt", Some(1024))
            .build();

        assert_eq!(response["metadata"]["file_path"], "/path/to/file.txt");
        assert_eq!(response["metadata"]["file_size"], 1024);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_build_for_output() {
        let (text, response) = ResponseBuilder::success("test_op")
            .with_message("Custom message")
            .build_for_output();

        assert_eq!(text, "Custom message");
        assert_eq!(response["success"], true);

        let (text, response) =
            ResponseBuilder::error("test_op", "Error occurred").build_for_output();

        assert_eq!(text, "Operation 'test_op' failed: Error occurred");
        assert_eq!(response["success"], false);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_helper_functions() {
        let response = success_response("test", "All good");
        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "All good");

        let response = error_response("test", "Failed");
        assert_eq!(response["success"], false);
        assert_eq!(response["error"]["message"], "Failed");

        let response =
            file_operation_response("write", "/file.txt", true, Some("Written".to_string()));
        assert_eq!(response["metadata"]["file_path"], "/file.txt");
        assert_eq!(response["message"], "Written");

        let items = vec!["a", "b", "c"];
        let response = list_response("list", &items, None);
        assert_eq!(response["result"], json!(["a", "b", "c"]));
        assert_eq!(response["metadata"]["count"], 3);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_error_details() {
        let error_details = ErrorDetails::new("Test error")
            .with_code("ERR_001")
            .with_details(json!({"field": "test"}));

        let response = ResponseBuilder::success("test")
            .with_error_details(error_details)
            .build();

        assert_eq!(response["success"], false);
        assert_eq!(response["error"]["message"], "Test error");
        assert_eq!(response["error"]["code"], "ERR_001");
        assert_eq!(response["error"]["details"]["field"], "test");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_validation_response() {
        let errors = vec![
            ValidationError::new("Field is required")
                .with_field("name")
                .with_code("REQUIRED"),
            ValidationError::new("Invalid format")
                .with_field("email")
                .with_code("FORMAT"),
        ];

        let response = validation_response(false, &Some(errors));

        assert_eq!(response["operation"], "validate");
        assert_eq!(response["success"], true);
        assert_eq!(response["result"]["valid"], false);

        let errors = &response["result"]["errors"];
        assert_eq!(errors[0]["field"], "name");
        assert_eq!(errors[0]["message"], "Field is required");
        assert_eq!(errors[0]["code"], "REQUIRED");
        assert_eq!(errors[1]["field"], "email");
        assert_eq!(errors[1]["message"], "Invalid format");
        assert_eq!(errors[1]["code"], "FORMAT");
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_validation_response_success() {
        let response = validation_response(true, &None);

        assert_eq!(response["operation"], "validate");
        assert_eq!(response["success"], true);
        assert_eq!(response["result"]["valid"], true);
        assert!(response["result"]["errors"].is_null());
    }
}
