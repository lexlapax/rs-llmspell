// ABOUTME: Integration tests for refactored utility tools
// ABOUTME: Validates consistent response format and error handling

use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use llmspell_tools::util::hash_calculator::{HashCalculatorConfig, HashCalculatorTool};
use llmspell_tools::util::text_manipulator::{TextManipulatorConfig, TextManipulatorTool};
use llmspell_tools::util::uuid_generator::{UuidGeneratorConfig, UuidGeneratorTool};
use llmspell_tools::util::*;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Helper to create test input with wrapped parameters
fn create_test_input(text: &str, params: serde_json::Value) -> AgentInput {
    AgentInput {
        text: text.to_string(),
        media: vec![],
        context: None,
        parameters: {
            let mut map = HashMap::new();
            map.insert("parameters".to_string(), params);
            map
        },
        output_modalities: vec![],
    }
}

/// Helper to extract result from wrapped response
fn extract_result(output: &str) -> Value {
    let parsed: Value = serde_json::from_str(output).expect("Invalid JSON");
    parsed["result"].clone()
}

#[tokio::test]
async fn test_hash_calculator_response_format() {
    let tool = HashCalculatorTool::new(HashCalculatorConfig::default());

    // Test MD5
    let input = create_test_input(
        "test",
        json!({
            "operation": "hash",
            "algorithm": "md5",
            "input": "test"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
    let hash = output["result"]["hash"].as_str().unwrap();
    assert_eq!(hash.len(), 32); // MD5 is 32 hex characters

    // Test SHA256
    let input = create_test_input(
        "test",
        json!({
            "operation": "hash",
            "algorithm": "sha256",
            "input": "test"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    let hash = output["result"]["hash"].as_str().unwrap();
    assert_eq!(hash.len(), 64); // SHA256 is 64 hex characters
}

#[tokio::test]
async fn test_uuid_generator_response_format() {
    let tool = UuidGeneratorTool::new(UuidGeneratorConfig::default());
    let input = create_test_input(
        "test",
        json!({
            "operation": "generate",
            "version": "v4"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
async fn test_text_manipulator_response_format() {
    let tool = TextManipulatorTool::new(TextManipulatorConfig::default());
    let input = create_test_input(
        "test",
        json!({
            "operation": "uppercase",
            "input": "hello world"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
async fn test_calculator_response_format() {
    let tool = CalculatorTool::new();
    let input = create_test_input(
        "test",
        json!({
            "operation": "evaluate",
            "input": "2 + 2"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
async fn test_datetime_handler_response_format() {
    let tool = DateTimeHandlerTool::new();
    let input = create_test_input(
        "test",
        json!({
            "operation": "now",
            "format": "iso"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
async fn test_diff_calculator_response_format() {
    let tool = DiffCalculatorTool::new();
    let input = create_test_input(
        "test",
        json!({
            "operation": "diff",
            "old_text": "line1\nline2\nline3",
            "new_text": "line1\nmodified\nline3"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
async fn test_data_validation_response_format() {
    let tool = DataValidationTool::new();
    let input = create_test_input(
        "test",
        json!({
            "input": {"name": "test", "age": 30},
            "rules": {
                "rules": [{
                    "type": "object",
                    "required": ["name", "age"],
                    "properties": {
                        "name": {"rules": [{"type": "type", "expected": "string"}]},
                        "age": {"rules": [{"type": "type", "expected": "number"}]}
                    },
                    "additional_properties": true
                }]
            }
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}

#[tokio::test]
#[cfg(feature = "templates")]
async fn test_template_engine_response_format() {
    let tool = TemplateEngineTool::new();
    let input = create_test_input(
        "test",
        json!({
            "input": "Hello {{name}}",
            "context": {"name": "World"}
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());
}
#[tokio::test]
async fn test_refactored_tools_error_consistency() {
    // Test that all tools handle errors consistently

    // Test missing required parameters
    let test_cases: Vec<(Box<dyn BaseAgent>, serde_json::Value)> = vec![
        (
            Box::new(HashCalculatorTool::new(HashCalculatorConfig::default())),
            json!({"operation": "hash", "algorithm": "md5"}),
        ), // missing input
        (
            Box::new(Base64EncoderTool::new()),
            json!({"operation": "encode"}),
        ), // missing input
        (
            Box::new(TextManipulatorTool::new(TextManipulatorConfig::default())),
            json!({"operation": "uppercase"}),
        ), // missing text
        (
            Box::new(CalculatorTool::new()),
            json!({"operation": "evaluate"}),
        ), // missing input
        (
            Box::new(DiffCalculatorTool::new()),
            json!({"format": "unified"}),
        ), // missing old_text/new_text
        (Box::new(DataValidationTool::new()), json!({"input": {}})), // missing rules
    ];

    for (i, (tool, params)) in test_cases.into_iter().enumerate() {
        let input = create_test_input("test", params);
        let result = tool.execute(input, ExecutionContext::default()).await;

        // Some tools may still return Err for missing parameters, others return Ok with success=false
        if result.is_err() {
            // Legacy error handling - tool returns Err
            continue;
        }

        // Check that the response indicates failure
        let output = result.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&output.text).unwrap();
        assert!(
            !parsed["success"].as_bool().unwrap_or(true),
            "Tool {i} should have success=false for missing parameters"
        );

        // Error should be in the response and mention missing parameters
        assert!(
            parsed.get("error").is_some(),
            "Tool {i} should have error field when parameters are missing"
        );
    }
}

#[tokio::test]
async fn test_hash_calculator_functionality() {
    let tool = HashCalculatorTool::new(HashCalculatorConfig::default());
    let input = create_test_input(
        "test",
        json!({
            "operation": "hash",
            "algorithm": "md5",
            "input": "Hello World"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(
        result_value["hash"].as_str().unwrap(),
        "b10a8db164e0754105b7a99be72e3fe5"
    );

    // Verify SHA256
    let input = create_test_input(
        "test",
        json!({
            "operation": "hash",
            "algorithm": "sha256",
            "input": "test"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(
        result_value["hash"].as_str().unwrap(),
        "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08"
    );
}

#[tokio::test]
async fn test_uuid_generator_functionality() {
    let tool = UuidGeneratorTool::new(UuidGeneratorConfig::default());
    let input = create_test_input(
        "test",
        json!({
            "operation": "generate",
            "version": "v4"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    let uuid = result_value["uuid"].as_str().unwrap();
    // Verify UUID v4 format (8-4-4-4-12 hex digits)
    assert_eq!(uuid.len(), 36);
    assert!(uuid.chars().nth(8) == Some('-'));
    assert!(uuid.chars().nth(13) == Some('-'));
    assert!(uuid.chars().nth(18) == Some('-'));
    assert!(uuid.chars().nth(23) == Some('-'));
}

#[tokio::test]
async fn test_text_manipulator_functionality() {
    let tool = TextManipulatorTool::new(TextManipulatorConfig::default());

    // Test uppercase
    let input = create_test_input(
        "test",
        json!({
            "operation": "uppercase",
            "input": "hello world"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["result"].as_str().unwrap(), "HELLO WORLD");

    // Test reverse
    let input = create_test_input(
        "test",
        json!({
            "operation": "reverse",
            "input": "hello"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["result"].as_str().unwrap(), "olleh");
}

#[tokio::test]
async fn test_calculator_functionality() {
    let tool = CalculatorTool::new();

    // Test basic arithmetic
    let input = create_test_input(
        "test",
        json!({
            "operation": "evaluate",
            "input": "2 + 2 * 3"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["result"], 8.0);

    // Test with variables
    let input = create_test_input(
        "test",
        json!({
            "operation": "evaluate",
            "input": "x^2 + y^2",
            "variables": {"x": 3, "y": 4}
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["result"], 25.0);
}

#[tokio::test]
async fn test_datetime_handler_functionality() {
    let tool = DateTimeHandlerTool::new();

    // Test now operation
    let input = create_test_input(
        "test",
        json!({
            "operation": "now"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    // Now operation returns datetime, timezone, and format fields
    assert!(result_value["datetime"].is_string());
    assert!(result_value["timezone"].is_string());

    // Test parsing
    let input = create_test_input(
        "test",
        json!({
            "operation": "parse",
            "input": "2024-01-15T12:00:00Z"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert!(result_value["parsed"].is_object());
    assert!(result_value["parsed"]["timestamp"].is_number());
    assert!(result_value["parsed"]["utc"].is_string());
}

#[tokio::test]
async fn test_diff_calculator_functionality() {
    let tool = DiffCalculatorTool::new();
    let input = create_test_input(
        "test",
        json!({
            "type": "text",
            "old_text": "line1\nline2\nline3",
            "new_text": "line1\nmodified\nline3\nadded"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert!(result_value["diff"].is_string());
    let diff_text = result_value["diff"].as_str().unwrap();
    assert!(diff_text.contains("modified") || diff_text.contains('+') || diff_text.contains('-'));
}

#[tokio::test]
async fn test_data_validation_functionality() {
    let tool = DataValidationTool::new();

    // Test valid data
    let input = create_test_input(
        "test",
        json!({
            "input": {"name": "John", "age": 30},
            "rules": {
                "rules": [{
                    "type": "object",
                    "required": ["name", "age"],
                    "properties": {
                        "name": {"rules": [{"type": "type", "expected": "string"}]},
                        "age": {"rules": [{"type": "type", "expected": "number"}]}
                    },
                    "additional_properties": true
                }]
            }
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["valid"], true);

    // Test invalid data - missing required field
    let input = create_test_input(
        "test",
        json!({
            "input": {"name": "John"},  // Missing required field
            "rules": {
                "rules": [{
                    "type": "object",
                    "required": ["name", "age"],
                    "properties": {
                        "name": {"rules": [{"type": "type", "expected": "string"}]},
                        "age": {"rules": [{"type": "type", "expected": "number"}]}
                    },
                    "additional_properties": false
                }]
            }
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["valid"], false);
    assert!(result_value["errors"].is_array());
}

#[tokio::test]
#[cfg(feature = "templates")]
async fn test_template_engine_functionality() {
    let tool = TemplateEngineTool::new();

    // Test basic template
    let input = create_test_input(
        "test",
        json!({
            "input": "Hello {{name}}, you have {{count}} messages",
            "context": {"name": "Alice", "count": 5}
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(
        result_value["rendered"].as_str().unwrap(),
        "Hello Alice, you have 5 messages"
    );

    // Test with conditionals
    let input = create_test_input(
        "test",
        json!({
            "input": "{{#if premium}}Premium User{{else}}Free User{{/if}}",
            "context": {"premium": true}
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let result_value = extract_result(&result.text);
    assert_eq!(result_value["rendered"].as_str().unwrap(), "Premium User");
}
#[tokio::test]
async fn test_tool_chaining_integration() {
    // Test chaining multiple refactored tools together

    // Step 1: Generate a UUID
    let uuid_tool = UuidGeneratorTool::new(UuidGeneratorConfig::default());
    let uuid_input = create_test_input("generate", json!({}));
    let uuid_result = uuid_tool
        .execute(uuid_input, ExecutionContext::default())
        .await
        .unwrap();
    let uuid_output = extract_result(&uuid_result.text);
    let uuid = uuid_output["uuid"].as_str().unwrap();

    // Step 2: Hash the UUID
    let hash_tool = HashCalculatorTool::new(HashCalculatorConfig::default());
    let hash_input = create_test_input(
        "hash",
        json!({
            "operation": "hash",
            "algorithm": "sha256",
            "input": uuid
        }),
    );
    let hash_result = hash_tool
        .execute(hash_input, ExecutionContext::default())
        .await
        .unwrap();
    let hash_output = extract_result(&hash_result.text);
    let hash = hash_output["hash"].as_str().unwrap();

    // Step 3: Encode the hash in base64
    let base64_tool = Base64EncoderTool::new();
    let base64_input = create_test_input(
        "encode",
        json!({
            "operation": "encode",
            "input": hash
        }),
    );
    let base64_result = base64_tool
        .execute(base64_input, ExecutionContext::default())
        .await
        .unwrap();
    let base64_output = extract_result(&base64_result.text);
    let encoded = base64_output["output"].as_str().unwrap();

    // Verify the chain worked
    assert!(!uuid.is_empty());
    assert!(!hash.is_empty());
    assert!(!encoded.is_empty());
    assert_eq!(hash.len(), 64); // SHA-256 produces 64 hex characters
}
