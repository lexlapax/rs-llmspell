// ABOUTME: Integration tests for refactored utility tools
// ABOUTME: Validates consistent response format and error handling

use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::{AgentInput, ExecutionContext};
use llmspell_tools::util::*;
use serde_json::{json, Value};

/// Helper to extract result from wrapped response
fn extract_result(output: &str) -> Value {
    let parsed: Value = serde_json::from_str(output).expect("Invalid JSON");
    parsed["result"].clone()
}

#[tokio::test]
async fn test_all_refactored_tools_response_format() {
    // Test that all refactored tools use consistent response format

    // HashCalculatorTool
    let tool = HashCalculatorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "hash",
            "algorithm": "md5",
            "data": "test"
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

    // Base64EncoderTool
    let tool = Base64EncoderTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
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

    // UuidGeneratorTool
    let tool = UuidGeneratorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter("parameters", json!({}));
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());

    // TextManipulatorTool
    let tool = TextManipulatorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "uppercase",
            "text": "test"
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

    // CalculatorTool
    let tool = CalculatorTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "expression": "2 + 2"
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

    // DateTimeHandlerTool
    let tool = DateTimeHandlerTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "now"
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

    // DiffCalculatorTool
    let tool = DiffCalculatorTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "old_text": "a",
            "new_text": "b",
            "format": "unified"
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

    // DataValidationTool
    let tool = DataValidationTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "data": {"field1": "test@example.com"},
            "rules": {
                "rules": [
                    {
                        "field": "field1",
                        "type": "email"
                    }
                ]
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

    // TemplateEngineTool
    let tool = TemplateEngineTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "template": "Hello {{name}}",
            "context": {"name": "World"},
            "engine": "handlebars"
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
            Box::new(HashCalculatorTool::new(Default::default())),
            json!({"operation": "hash", "algorithm": "md5"}),
        ), // missing data
        (
            Box::new(Base64EncoderTool::new()),
            json!({"operation": "encode"}),
        ), // missing input
        (
            Box::new(TextManipulatorTool::new(Default::default())),
            json!({"operation": "uppercase"}),
        ), // missing text
        (
            Box::new(CalculatorTool::new()),
            json!({"operation": "evaluate"}),
        ), // missing expression
        (
            Box::new(DiffCalculatorTool::new()),
            json!({"format": "unified"}),
        ), // missing old_text/new_text
        (Box::new(DataValidationTool::new()), json!({"data": {}})), // missing rules
        (
            Box::new(TemplateEngineTool::new()),
            json!({"engine": "handlebars"}),
        ), // missing template
    ];

    for (i, (tool, params)) in test_cases.into_iter().enumerate() {
        let input = AgentInput::text("test").with_parameter("parameters", params);
        let result = tool.execute(input, ExecutionContext::default()).await;

        // Should fail with missing parameters
        assert!(
            result.is_err(),
            "Tool {} should fail with missing parameters",
            i
        );

        let err = result.unwrap_err();
        let err_str = err.to_string();

        // Error should be a validation error mentioning the missing field
        assert!(
            err_str.contains("required")
                || err_str.contains("missing")
                || err_str.contains("Missing")
                || err_str.contains("must be provided")
                || err_str.contains("Either"),
            "Error should indicate missing parameter: {}",
            err_str
        );
    }
}

#[tokio::test]
async fn test_refactored_tools_functionality() {
    // Test actual functionality of refactored tools

    // HashCalculatorTool - verify hash values
    let tool = HashCalculatorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "hash",
            "algorithm": "md5",
            "data": "Hello, World!"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["hash"], "65a8e27d8879283831b664bd8b7f0ad4");

    // Base64EncoderTool - verify encoding/decoding
    let tool = Base64EncoderTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "input": "Hello, Base64!"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["output"], "SGVsbG8sIEJhc2U2NCE=");

    // UuidGeneratorTool - verify UUID format
    let tool = UuidGeneratorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter("parameters", json!({}));
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    let uuid_str = output["uuid"].as_str().unwrap();
    assert_eq!(uuid_str.len(), 36); // Standard UUID length

    // TextManipulatorTool - verify text operations
    let tool = TextManipulatorTool::new(Default::default());
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "uppercase",
            "text": "hello world"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["result"], "HELLO WORLD");

    // CalculatorTool - verify calculations
    let tool = CalculatorTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "expression": "2 + 3 * 4"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["result"], 14.0);

    // DateTimeHandlerTool - verify date operations
    let tool = DateTimeHandlerTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "operation": "parse",
            "input": "2023-12-25T10:30:00Z"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["parsed"]["year"], 2023);
    assert_eq!(output["parsed"]["month"], 12);
    assert_eq!(output["parsed"]["day"], 25);

    // DiffCalculatorTool - verify diff detection
    let tool = DiffCalculatorTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "old_text": "Line 1\nLine 2",
            "new_text": "Line 1\nLine 2 modified",
            "format": "unified"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert!(output["diff"].as_str().unwrap().contains("modified"));

    // DataValidationTool - verify validation
    let tool = DataValidationTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "data": {"email": "test@example.com"},
            "rules": {
                "rules": [
                    {
                        "field": "email",
                        "type": "email"
                    }
                ]
            }
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["valid"], true);

    // TemplateEngineTool - verify template rendering
    let tool = TemplateEngineTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({
            "template": "Hello, {{name}}!",
            "context": {"name": "World"},
            "engine": "handlebars"
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    assert_eq!(output["rendered"], "Hello, World!");
}

#[tokio::test]
async fn test_tool_chaining_integration() {
    // Test chaining multiple refactored tools together

    // Step 1: Generate a UUID
    let uuid_tool = UuidGeneratorTool::new(Default::default());
    let uuid_input = AgentInput::text("generate").with_parameter("parameters", json!({}));
    let uuid_result = uuid_tool
        .execute(uuid_input, ExecutionContext::default())
        .await
        .unwrap();
    let uuid_output = extract_result(&uuid_result.text);
    let uuid = uuid_output["uuid"].as_str().unwrap();

    // Step 2: Hash the UUID
    let hash_tool = HashCalculatorTool::new(Default::default());
    let hash_input = AgentInput::text("hash").with_parameter(
        "parameters",
        json!({
            "operation": "hash",
            "algorithm": "sha256",
            "data": uuid
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
    let base64_input = AgentInput::text("encode").with_parameter(
        "parameters",
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
