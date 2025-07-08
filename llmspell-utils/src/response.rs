// ABOUTME: Response builder utilities for consistent output formatting across tools
// ABOUTME: Provides fluent API for building successful and error responses

//! Response building utilities
//!
//! This module provides a fluent API for building consistent response
//! objects across all LLMSpell tools, ensuring uniform output structure.

use serde_json::{json, Value};
use std::collections::HashMap;

/// A builder for creating consistent response objects
#[derive(Debug, Clone)]
pub struct ResponseBuilder {
    operation: String,
    success: bool,
    message: Option<String>,
    result: Option<Value>,
    error: Option<String>,
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
            error: Some(error.into()),
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
            response["error"] = json!(error);
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
                self.error.as_deref().unwrap_or("Unknown error")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_response() {
        let response = ResponseBuilder::success("test_op")
            .with_message("Test completed")
            .build();

        assert_eq!(response["operation"], "test_op");
        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "Test completed");
    }

    #[test]
    fn test_error_response() {
        let response = ResponseBuilder::error("test_op", "Something went wrong").build();

        assert_eq!(response["operation"], "test_op");
        assert_eq!(response["success"], false);
        assert_eq!(response["error"], "Something went wrong");
    }

    #[test]
    fn test_with_result() {
        let response = ResponseBuilder::success("test_op")
            .with_result(json!({ "data": [1, 2, 3] }))
            .build();

        assert_eq!(response["result"]["data"], json!([1, 2, 3]));
    }

    #[test]
    fn test_with_metadata() {
        let response = ResponseBuilder::success("test_op")
            .with_metadata("key1", json!("value1"))
            .with_metadata("key2", json!(42))
            .build();

        assert_eq!(response["metadata"]["key1"], "value1");
        assert_eq!(response["metadata"]["key2"], 42);
    }

    #[test]
    fn test_with_file_info() {
        let response = ResponseBuilder::success("read_file")
            .with_file_info("/path/to/file.txt", Some(1024))
            .build();

        assert_eq!(response["metadata"]["file_path"], "/path/to/file.txt");
        assert_eq!(response["metadata"]["file_size"], 1024);
    }

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

    #[test]
    fn test_helper_functions() {
        let response = success_response("test", "All good");
        assert_eq!(response["success"], true);
        assert_eq!(response["message"], "All good");

        let response = error_response("test", "Failed");
        assert_eq!(response["success"], false);
        assert_eq!(response["error"], "Failed");

        let response =
            file_operation_response("write", "/file.txt", true, Some("Written".to_string()));
        assert_eq!(response["metadata"]["file_path"], "/file.txt");
        assert_eq!(response["message"], "Written");

        let items = vec!["a", "b", "c"];
        let response = list_response("list", &items, None);
        assert_eq!(response["result"], json!(["a", "b", "c"]));
        assert_eq!(response["metadata"]["count"], 3);
    }
}
