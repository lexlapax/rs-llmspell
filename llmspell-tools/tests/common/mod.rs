//! ABOUTME: Common test utilities and helpers for integration tests
//! ABOUTME: Provides standardized test context creation and validation helpers

use llmspell_core::{
    types::{AgentInput, AgentOutput},
    ExecutionContext, LLMSpellError,
};
use serde_json::{json, Value};

/// Create a standard test execution context
pub fn create_test_context() -> ExecutionContext {
    ExecutionContext::new()
}

/// Create an agent input with the given parameters
pub fn create_agent_input(params: Value) -> Result<AgentInput, LLMSpellError> {
    // AgentInput expects parameters to be wrapped in a "parameters" object
    let mut input = AgentInput::text("");
    let wrapped_params = json!({ "parameters": params });
    if let Value::Object(map) = wrapped_params {
        // Convert serde_json::Map to HashMap
        input.parameters = map.into_iter().collect();
    }
    Ok(input)
}

/// Create an agent input with a single "input" parameter
pub fn create_simple_input(input: &str) -> Result<AgentInput, LLMSpellError> {
    create_agent_input(json!({ "input": input }))
}

/// Validate that output is successful with expected fields
#[allow(dead_code)]
pub fn assert_success_output(output: &AgentOutput, expected_fields: &[&str]) {
    let output_value: Value = serde_json::from_str(&output.text).unwrap();

    assert!(
        output_value["success"].as_bool().unwrap_or(false),
        "Expected success=true, got: {}",
        output_value
    );

    for field in expected_fields {
        assert!(
            output_value.get(field).is_some(),
            "Expected field '{}' in output, got: {}",
            field,
            output_value
        );
    }
}

/// Validate that output is an error with expected message pattern
#[allow(dead_code)]
pub fn assert_error_output(output: &AgentOutput, error_pattern: &str) {
    let output_value: Value = serde_json::from_str(&output.text).unwrap();

    assert!(
        !output_value["success"].as_bool().unwrap_or(true),
        "Expected success=false, got: {}",
        output_value
    );

    // Try different error message formats
    let error_msg = if let Some(error_str) = output_value["error"].as_str() {
        // Simple string error format
        error_str.to_string()
    } else if let Some(error_obj) = output_value["error"].as_object() {
        // Complex error object format - try message field first
        if let Some(msg) = error_obj.get("message").and_then(|m| m.as_str()) {
            msg.to_string()
        } else {
            // Fallback to serializing the entire error object
            serde_json::to_string(error_obj).unwrap_or_else(|_| format!("{:?}", error_obj))
        }
    } else {
        panic!("Expected error field in output, got: {}", output_value);
    };

    assert!(
        error_msg
            .to_lowercase()
            .contains(&error_pattern.to_lowercase()),
        "Expected error to contain '{}', got: '{}'",
        error_pattern,
        error_msg
    );
}

/// Common test URLs and endpoints
#[allow(dead_code)]
pub mod test_endpoints {
    pub const HTTPBIN_BASE: &str = "https://httpbin.org";
    pub const HTTPBIN_GET: &str = "https://httpbin.org/get";
    pub const HTTPBIN_POST: &str = "https://httpbin.org/post";
    pub const HTTPBIN_STATUS: &str = "https://httpbin.org/status";
    pub const HTTPBIN_DELAY: &str = "https://httpbin.org/delay";
    pub const HTTPBIN_HEADERS: &str = "https://httpbin.org/headers";

    pub const EXAMPLE_WEBSITE: &str = "https://example.com";
    pub const INVALID_URL: &str = "https://this-domain-definitely-does-not-exist-12345.com";
}

/// Common test data fixtures
#[allow(dead_code)]
pub mod fixtures {
    use serde_json::json;

    pub fn sample_json() -> serde_json::Value {
        json!({
            "name": "Test User",
            "email": "test@example.com",
            "age": 30,
            "active": true
        })
    }

    pub fn sample_html() -> &'static str {
        r#"<!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
            <h1>Test Header</h1>
            <p>Test paragraph</p>
            <a href="/link1">Link 1</a>
            <a href="/link2">Link 2</a>
        </body>
        </html>"#
    }

    pub fn sample_api_key() -> &'static str {
        "test_api_key_12345"
    }
}

/// Helper to create test configuration
#[allow(dead_code)]
pub fn create_test_config() -> serde_json::Map<String, serde_json::Value> {
    let mut config = serde_json::Map::new();
    config.insert("timeout".to_string(), json!(30));
    config.insert("max_retries".to_string(), json!(3));
    config
}

/// Assert that a value contains expected JSON structure
#[allow(dead_code)]
pub fn assert_json_contains(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Object(actual_map), Value::Object(expected_map)) => {
            for (key, expected_value) in expected_map {
                assert!(
                    actual_map.contains_key(key),
                    "Expected key '{}' not found in actual JSON",
                    key
                );
                assert_json_contains(&actual_map[key], expected_value);
            }
        }
        (actual_val, expected_val) => {
            assert_eq!(actual_val, expected_val, "JSON values don't match");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_context() {
        let context = create_test_context();
        // Context should be created successfully
        let _ = context;
    }

    #[test]
    fn test_create_agent_input() {
        let input = create_agent_input(json!({"test": "value"})).unwrap();
        // AgentInput stores JSON data internally
        let _ = input;
    }

    #[test]
    fn test_create_simple_input() {
        let input = create_simple_input("test input").unwrap();
        // AgentInput stores JSON data internally
        let _ = input;
    }
}
