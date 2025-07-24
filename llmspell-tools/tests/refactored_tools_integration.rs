// ABOUTME: Integration tests for refactored utility tools
// ABOUTME: Validates consistent response format and error handling

use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
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
async fn test_all_refactored_tools_response_format() {
    // Test that all refactored tools use consistent response format

    // HashCalculatorTool
    let tool = HashCalculatorTool::new(Default::default());
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

    // Base64EncoderTool
    let tool = Base64EncoderTool::new();
    let input = create_test_input(
        "test",
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
    let input = create_test_input("test", json!({}));
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
    let input = create_test_input(
        "test",
        json!({
            "operation": "uppercase",
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

    // CalculatorTool
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

    // DateTimeHandlerTool
    let tool = DateTimeHandlerTool::new();
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
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert_eq!(output["success"], true);
    assert!(output["operation"].is_string());
    assert!(output["result"].is_object());

    // DiffCalculatorTool
    let tool = DiffCalculatorTool::new();
    let input = create_test_input(
        "test",
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
    let input = create_test_input(
        "test",
        json!({
            "input": {"field1": "test@example.com"},
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
    let input = create_test_input(
        "test",
        json!({
            "input": "Hello {{name}}",
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
        ), // missing input
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
        ), // missing input
        (
            Box::new(DiffCalculatorTool::new()),
            json!({"format": "unified"}),
        ), // missing old_text/new_text
        (Box::new(DataValidationTool::new()), json!({"input": {}})), // missing rules
        (
            Box::new(TemplateEngineTool::new()),
            json!({"engine": "handlebars"}),
        ), // missing input
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
            "Tool {} should have success=false for missing parameters",
            i
        );

        // Error should be in the response and mention missing parameters
        assert!(
            parsed.get("error").is_some(),
            "Tool {} should have error field when parameters are missing",
            i
        );
    }
}

#[tokio::test]
async fn test_refactored_tools_functionality() {
    // Test actual functionality of refactored tools

    // HashCalculatorTool - verify hash values
    let tool = HashCalculatorTool::new(Default::default());
    let input = create_test_input(
        "test",
        json!({
            "operation": "hash",
            "algorithm": "md5",
            "input": "Hello, World!"
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
    let input = create_test_input(
        "test",
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
    let input = create_test_input("test", json!({}));
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output = extract_result(&result.text);
    let uuid_str = output["uuid"].as_str().unwrap();
    assert_eq!(uuid_str.len(), 36); // Standard UUID length

    // TextManipulatorTool - verify text operations
    let tool = TextManipulatorTool::new(Default::default());
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
    let output = extract_result(&result.text);
    assert_eq!(output["result"], "HELLO WORLD");

    // CalculatorTool - verify calculations
    let tool = CalculatorTool::new();
    let input = create_test_input(
        "test",
        json!({
            "operation": "evaluate",
            "input": "2 + 3 * 4"
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
    let input = create_test_input(
        "test",
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
    let input = create_test_input(
        "test",
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
    let input = create_test_input(
        "test",
        json!({
            "input": {"email": "test@example.com"},
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
    let input = create_test_input(
        "test",
        json!({
            "input": "Hello, {{name}}!",
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
    let uuid_input = create_test_input("generate", json!({}));
    let uuid_result = uuid_tool
        .execute(uuid_input, ExecutionContext::default())
        .await
        .unwrap();
    let uuid_output = extract_result(&uuid_result.text);
    let uuid = uuid_output["uuid"].as_str().unwrap();

    // Step 2: Hash the UUID
    let hash_tool = HashCalculatorTool::new(Default::default());
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
