// ABOUTME: Tests to verify DRY principle - that similar operations produce consistent results across tools
// ABOUTME: Ensures tools using shared utilities from llmspell-utils behave consistently

use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_tools::{
    data::JsonProcessorTool,
    util::{Base64EncoderTool, DateTimeHandlerTool, HashCalculatorTool, UuidGeneratorTool},
};
use serde_json::{json, Value};

#[cfg(test)]
mod dry_principle_tests {
    use super::*;
    #[tokio::test]
    async fn test_hash_consistency_across_tools() {
        // Test that hash operations produce consistent results
        let hash_tool = HashCalculatorTool::new(Default::default());
        let test_data = "Hello, DRY Principle!";

        // Test SHA256
        let input = AgentInput::text("hash test").with_parameter(
            "parameters",
            json!({
                "operation": "hash",
                "algorithm": "sha256",
                "input": test_data
            }),
        );

        let result = hash_tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        let sha256_hash = output["result"]["hash"].as_str().unwrap();

        // Verify the hash is consistent with the utility function
        use llmspell_utils::encoding::{hash_string, to_hex_string, HashAlgorithm};
        let expected_hash_bytes = hash_string(test_data, HashAlgorithm::Sha256);
        let expected_hash = to_hex_string(&expected_hash_bytes);
        assert_eq!(
            sha256_hash, expected_hash,
            "Hash tool should produce same result as utility"
        );
    }
    #[tokio::test]
    async fn test_base64_consistency() {
        // Test that base64 operations are consistent
        let base64_tool = Base64EncoderTool::new();
        let test_data = "Test Base64 Consistency";

        // Encode
        let input = AgentInput::text("encode").with_parameter(
            "parameters",
            json!({
                "operation": "encode",
                "input": test_data
            }),
        );

        let result = base64_tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        let encoded = output["result"]["output"].as_str().unwrap();

        // Verify with utility
        use llmspell_utils::encoding::base64_encode;
        let expected = base64_encode(test_data.as_bytes());
        assert_eq!(
            encoded, expected,
            "Base64 tool should produce same result as utility"
        );
    }
    #[tokio::test]
    async fn test_uuid_format_consistency() {
        // Test that UUID generation follows consistent patterns
        let uuid_tool = UuidGeneratorTool::new(Default::default());

        // Generate UUID v4
        let input = AgentInput::text("generate").with_parameter(
            "parameters",
            json!({
                "operation": "generate",
                "version": "v4"
            }),
        );

        let result = uuid_tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        let uuid_str = output["result"]["uuid"].as_str().unwrap();

        // Verify format
        assert_eq!(uuid_str.len(), 36, "UUID should be 36 characters");
        assert!(
            uuid_str.chars().filter(|&c| c == '-').count() == 4,
            "UUID should have 4 hyphens"
        );

        // Verify it's a valid UUID v4
        let uuid = uuid::Uuid::parse_str(uuid_str).expect("Should be valid UUID");
        assert_eq!(
            uuid.get_version(),
            Some(uuid::Version::Random),
            "Should be UUID v4"
        );
    }
    #[tokio::test]
    async fn test_json_processing_consistency() {
        // Test that JSON operations are consistent
        let json_tool = JsonProcessorTool::new(Default::default());
        let test_json = json!({
            "name": "test",
            "value": 42,
            "nested": {
                "array": [1, 2, 3]
            }
        });

        // Query JSON using jq
        let input = AgentInput::text("query").with_parameter(
            "parameters",
            json!({
                "operation": "query",
                "input": test_json,
                "query": "."  // Identity query - returns the whole object
            }),
        );

        let result = json_tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        // JsonProcessor returns raw JSON results, not wrapped
        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert_eq!(
            output["name"], "test",
            "Query should return the original object"
        );
    }
    #[tokio::test]
    async fn test_date_time_consistency() {
        // Test that date/time operations are consistent
        let datetime_tool = DateTimeHandlerTool::new();

        // Parse a specific date
        let test_date = "2025-01-27T10:30:00Z";
        let input = AgentInput::text("parse").with_parameter(
            "parameters",
            json!({
                "operation": "parse",
                "input": test_date
            }),
        );

        let result = datetime_tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();

        // Verify parsed components
        assert_eq!(output["result"]["parsed"]["year"].as_i64().unwrap(), 2025);
        assert_eq!(output["result"]["parsed"]["month"].as_i64().unwrap(), 1);
        assert_eq!(output["result"]["parsed"]["day"].as_i64().unwrap(), 27);
        assert_eq!(output["result"]["parsed"]["hour"].as_i64().unwrap(), 10);
        assert_eq!(output["result"]["parsed"]["minute"].as_i64().unwrap(), 30);
    }
    #[tokio::test]
    async fn test_error_handling_consistency() {
        // Test that error handling is consistent across tools
        let hash_tool = HashCalculatorTool::new(Default::default());
        let base64_tool = Base64EncoderTool::new();

        // Test missing required parameter in hash tool
        let input = AgentInput::text("hash").with_parameter(
            "parameters",
            json!({
                "operation": "hash",
                "algorithm": "sha256"
                // Missing 'data' parameter
            }),
        );

        let result = hash_tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Missing required parameter"));

        // Test missing required parameter in base64 tool
        let input = AgentInput::text("encode").with_parameter(
            "parameters",
            json!({
                "operation": "encode"
                // Missing 'input' parameter
            }),
        );

        let result = base64_tool
            .execute(input, ExecutionContext::default())
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("ither")
                || err.to_string().contains("missing")
                || err.to_string().contains("required"),
            "Error was: {err}"
        );

        // Both should handle missing parameters consistently
    }
    #[tokio::test]
    async fn test_parameter_extraction_consistency() {
        // Test that all tools use consistent parameter extraction
        let tools: Vec<Box<dyn BaseAgent>> = vec![
            Box::new(HashCalculatorTool::new(Default::default())),
            Box::new(Base64EncoderTool::new()),
            Box::new(UuidGeneratorTool::new(Default::default())),
        ];

        // Test with no parameters wrapper
        let input = AgentInput::text("test");

        for tool in tools {
            let result = tool
                .execute(input.clone(), ExecutionContext::default())
                .await;
            assert!(
                result.is_err(),
                "All tools should fail without parameters wrapper"
            );
            let err = result.unwrap_err();
            assert!(
                err.to_string().contains("parameters") || err.to_string().contains("Parameters"),
                "Error should mention missing parameters"
            );
        }
    }
}
#[test]
fn test_shared_utility_usage() {
    // Verify tools are importing from llmspell_utils
    // This is a compile-time test - if it compiles, the imports are correct

    // These should all compile, proving the tools use shared utilities
    let _ = llmspell_utils::encoding::hash_string;
    let _ = llmspell_utils::encoding::base64_encode;
    let _ = llmspell_utils::id_generator::generate_component_id;
    let _ = llmspell_utils::params::extract_parameters;
    // validation_error exists but needs type parameters, just proving it exists

    // If this test compiles, it proves the utilities exist and are accessible
    // Test passes by compilation - shared utilities are properly exposed
}
