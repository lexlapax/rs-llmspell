// ABOUTME: Integration tests for the Base64 encoder tool
// ABOUTME: Tests encoding/decoding with files, binary data, and error handling

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::util::Base64EncoderTool;
use serde_json::{json, Value};
use std::fs;
use tempfile::TempDir;

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_standard_base64_roundtrip() {
    let tool = Base64EncoderTool::new();

    // Test various input strings
    let test_cases = vec![
        "Hello, World!",
        "The quick brown fox jumps over the lazy dog",
        "1234567890",
        "Special chars: !@#$%^&*()",
        "Unicode: 你好世界 🌍",
        "", // Empty string
    ];

    for test_text in test_cases {
        // Encode
        let input = AgentInput::text("encode text").with_parameter(
            "parameters",
            json!({
                "operation": "encode",
                "input": test_text
            }),
        );
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap());
        let encoded = output["result"]["output"].as_str().unwrap();

        // Decode
        let input = AgentInput::text("decode text").with_parameter(
            "parameters",
            json!({
                "operation": "decode",
                "input": encoded
            }),
        );
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap());
        let decoded = output["result"]["output"].as_str().unwrap();

        assert_eq!(decoded, test_text, "Failed to roundtrip: {}", test_text);
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_url_safe_base64() {
    let tool = Base64EncoderTool::new();

    // Test data that would contain + and / in standard Base64
    let test_data = "Sure? Yes! This > that & more << less";

    // Encode with URL-safe variant
    let input = AgentInput::text("encode url-safe").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "variant": "url-safe",
            "input": test_data
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    let encoded = output["result"]["output"].as_str().unwrap();

    // Verify no + or / characters
    assert!(
        !encoded.contains('+'),
        "URL-safe encoding should not contain +"
    );
    assert!(
        !encoded.contains('/'),
        "URL-safe encoding should not contain /"
    );

    // Decode URL-safe
    let input = AgentInput::text("decode url-safe").with_parameter(
        "parameters",
        json!({
            "operation": "decode",
            "variant": "url-safe",
            "input": encoded
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    let decoded = output["result"]["output"].as_str().unwrap();
    assert_eq!(decoded, test_data);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_binary_data_handling() {
    let tool = Base64EncoderTool::new();

    // Test with binary data as hex
    let test_cases = vec![
        ("deadbeef", "3q2+7w=="),
        (
            "00112233445566778899aabbccddeeff",
            "ABEiM0RVZneImaq7zN3u/w==",
        ),
        ("ff00ff00", "/wD/AA=="),
    ];

    for (hex_input, expected_base64) in test_cases {
        // Encode binary data
        let input = AgentInput::text("encode binary").with_parameter(
            "parameters",
            json!({
                "operation": "encode",
                "input": hex_input,
                "binary_input": true
            }),
        );
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        let encoded = output["result"]["output"].as_str().unwrap();

        // Verify the encoding is correct
        assert_eq!(
            encoded, expected_base64,
            "Encoding mismatch for {}",
            hex_input
        );

        // Decode back
        let input = AgentInput::text("decode binary").with_parameter(
            "parameters",
            json!({
                "operation": "decode",
                "input": encoded
            }),
        );
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        let decoded_hex = output["result"]["output"].as_str().unwrap();

        // The decoded hex might not have leading zeros, so we compare the actual bytes
        let original_bytes = hex::decode(hex_input).unwrap();
        let decoded_hex_padded = if decoded_hex.len() % 2 == 1 {
            format!("0{}", decoded_hex)
        } else {
            decoded_hex.to_string()
        };
        let decoded_bytes = hex::decode(&decoded_hex_padded).unwrap();
        assert_eq!(
            decoded_bytes, original_bytes,
            "Failed for input: {}",
            hex_input
        );
    }
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_file_operations() {
    let tool = Base64EncoderTool::new();
    let temp_dir = TempDir::new().unwrap();

    // Test content
    let test_content = "This is a test file for Base64 encoding!";
    let input_path = temp_dir.path().join("input.txt");
    let encoded_path = temp_dir.path().join("encoded.b64");
    let decoded_path = temp_dir.path().join("decoded.txt");

    // Write test content
    fs::write(&input_path, test_content).unwrap();

    // Encode file to file
    let input = AgentInput::text("encode file").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "input_file": input_path.to_str().unwrap(),
            "output_file": encoded_path.to_str().unwrap()
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap());

    // Verify encoded file exists
    assert!(encoded_path.exists());
    let encoded_content = fs::read_to_string(&encoded_path).unwrap();
    assert!(!encoded_content.is_empty());

    // Decode file to file
    let input = AgentInput::text("decode file").with_parameter(
        "parameters",
        json!({
            "operation": "decode",
            "input_file": encoded_path.to_str().unwrap(),
            "output_file": decoded_path.to_str().unwrap()
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap());

    // Verify decoded content matches original
    let decoded_content = fs::read_to_string(&decoded_path).unwrap();
    assert_eq!(decoded_content, test_content);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_large_file_handling() {
    let tool = Base64EncoderTool::new();
    let temp_dir = TempDir::new().unwrap();

    // Create a larger file (1MB)
    let large_content = "A".repeat(1024 * 1024);
    let input_path = temp_dir.path().join("large.txt");
    let output_path = temp_dir.path().join("large.b64");

    fs::write(&input_path, &large_content).unwrap();

    // Encode large file
    let input = AgentInput::text("encode large file").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "input_file": input_path.to_str().unwrap(),
            "output_file": output_path.to_str().unwrap()
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    assert!(output["success"].as_bool().unwrap());

    // Verify success
    assert!(output["success"].as_bool().unwrap());
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_error_handling() {
    let tool = Base64EncoderTool::new();

    // Test invalid Base64 input
    let input = AgentInput::text("decode invalid").with_parameter(
        "parameters",
        json!({
            "operation": "decode",
            "input": "This is not valid Base64!@#$%"
        }),
    );
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test missing input
    let input = AgentInput::text("encode missing").with_parameter(
        "parameters",
        json!({
            "operation": "encode"
        }),
    );
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test non-existent input file
    let input = AgentInput::text("encode non-existent").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "input_file": "/non/existent/file.txt"
        }),
    );
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());

    // Test invalid hex input for binary
    let input = AgentInput::text("encode invalid hex").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "input": "not-valid-hex",
            "binary_input": true
        }),
    );
    let result = tool.execute(input, ExecutionContext::default()).await;
    assert!(result.is_err());
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_mixed_variants() {
    let tool = Base64EncoderTool::new();
    let test_text = "Test data for variant mixing";

    // Encode with standard
    let input = AgentInput::text("encode standard").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "variant": "standard",
            "input": test_text
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output: Value = serde_json::from_str(&result.text).unwrap();
    let standard_encoded = output["result"]["output"].as_str().unwrap().to_string();

    // Encode with URL-safe
    let input = AgentInput::text("encode url-safe").with_parameter(
        "parameters",
        json!({
            "operation": "encode",
            "variant": "url-safe",
            "input": test_text
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output2: Value = serde_json::from_str(&result.text).unwrap();
    let urlsafe_encoded = output2["result"]["output"].as_str().unwrap().to_string();

    // Try to decode standard with URL-safe decoder (should fail for some inputs)
    // Try to decode URL-safe with standard decoder (should fail for some inputs)

    // But both should decode correctly with their matching variants
    let input = AgentInput::text("decode standard").with_parameter(
        "parameters",
        json!({
            "operation": "decode",
            "variant": "standard",
            "input": standard_encoded
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output_value = serde_json::from_str::<Value>(&result.text).unwrap();
    let decoded = output_value["result"]["output"].as_str().unwrap();
    assert_eq!(decoded, test_text);

    let input = AgentInput::text("decode url-safe").with_parameter(
        "parameters",
        json!({
            "operation": "decode",
            "variant": "url-safe",
            "input": urlsafe_encoded
        }),
    );
    let result = tool
        .execute(input, ExecutionContext::default())
        .await
        .unwrap();
    let output_value = serde_json::from_str::<Value>(&result.text).unwrap();
    let decoded = output_value["result"]["output"].as_str().unwrap();
    assert_eq!(decoded, test_text);
}

#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
#[cfg_attr(test_category = "security")]
#[tokio::test]
async fn test_tool_metadata() {
    use llmspell_core::traits::tool::{SecurityLevel, Tool, ToolCategory};

    let tool = Base64EncoderTool::new();

    assert_eq!(tool.metadata().name, "base64-encoder");
    assert!(tool.metadata().description.contains("Base64"));
    assert_eq!(tool.category(), ToolCategory::Utility);
    assert_eq!(tool.security_level(), SecurityLevel::Safe);

    // Verify schema
    let schema = tool.schema();
    assert_eq!(schema.name, "base64_encoder");
    assert!(schema.description.contains("Base64"));
}
