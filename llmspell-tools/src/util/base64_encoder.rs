// ABOUTME: Base64 encoding and decoding tool with standard and URL-safe variants
// ABOUTME: Provides encode/decode operations for text and binary data

//! Base64 encoding and decoding tool
//!
//! This tool provides Base64 encoding and decoding functionality with support for:
//! - Standard Base64 encoding/decoding
//! - URL-safe Base64 encoding/decoding
//! - Binary data and file handling
//! - Text and binary input/output

use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits, SecurityLevel, SecurityRequirements, Tool,
            ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    encoding::{base64_decode, base64_decode_url_safe, base64_encode, base64_encode_url_safe},
    error_builders::llmspell::{storage_error, validation_error},
    params::{
        extract_optional_bool, extract_optional_string, extract_parameters, extract_required_string,
    },
    response::ResponseBuilder,
    validators::validate_enum,
};
use serde_json::{json, Value};
use std::fs;
use tracing::info;

/// Base64 encoding/decoding tool
#[derive(Debug, Clone)]
pub struct Base64EncoderTool {
    /// Tool metadata
    metadata: ComponentMetadata,
}

impl Default for Base64EncoderTool {
    fn default() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "base64-encoder".to_string(),
                "Base64 encoding and decoding tool with standard and URL-safe variants".to_string(),
            ),
        }
    }
}

impl Base64EncoderTool {
    /// Create a new Base64 encoder tool
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Process Base64 operation
    async fn process_operation(&self, params: &Value) -> Result<Value> {
        // Extract parameters using utilities
        let operation = extract_required_string(params, "operation")?;
        validate_enum(&operation, &["encode", "decode"], "operation")?;

        let variant = extract_optional_string(params, "variant").unwrap_or("standard");
        validate_enum(&variant, &["standard", "url-safe"], "variant")?;

        // Get input data
        let input_file = extract_optional_string(params, "input_file");
        let input_str = extract_optional_string(params, "input");
        let binary_input = extract_optional_bool(params, "binary_input").unwrap_or(false);

        let input_data = if let Some(file_path) = input_file {
            // Read from file
            fs::read(file_path).map_err(|e| {
                storage_error(
                    format!("Failed to read input file: {}", e),
                    Some("read_file".to_string()),
                )
            })?
        } else if let Some(input) = input_str {
            if binary_input {
                // Parse hex string as binary
                hex::decode(input).map_err(|e| {
                    validation_error(
                        format!("Failed to parse hex input: {}", e),
                        Some("input".to_string()),
                    )
                })?
            } else {
                // Use text input
                input.as_bytes().to_vec()
            }
        } else {
            return Err(validation_error(
                "Either 'input' or 'input_file' must be provided",
                Some("input".to_string()),
            ));
        };

        // Perform operation
        let result_data = match operation {
            "encode" => {
                let encoded = match variant {
                    "url-safe" => base64_encode_url_safe(&input_data),
                    _ => base64_encode(&input_data),
                };
                encoded.into_bytes()
            }
            "decode" => {
                let input_str = if let Some(input) = input_str {
                    input.to_string()
                } else {
                    // Convert file data to string for decoding
                    String::from_utf8(input_data).map_err(|e| {
                        validation_error(
                            format!(
                                "Input file contains invalid UTF-8 for Base64 decoding: {}",
                                e
                            ),
                            Some("input_file".to_string()),
                        )
                    })?
                };

                let decoded = match variant {
                    "url-safe" => base64_decode_url_safe(&input_str),
                    _ => base64_decode(&input_str),
                };

                decoded.map_err(|e| {
                    validation_error(
                        format!("Base64 decode error: {}", e),
                        Some("input".to_string()),
                    )
                })?
            }
            _ => unreachable!(), // Already validated
        };

        // Handle output
        let output_path = extract_optional_string(params, "output_file");

        if let Some(path) = output_path {
            // Write to file
            fs::write(path, &result_data).map_err(|e| {
                storage_error(
                    format!("Failed to write output file: {}", e),
                    Some("write_file".to_string()),
                )
            })?;

            info!(
                "Base64 {} completed: {} -> {}",
                operation,
                input_file.unwrap_or("input"),
                path
            );

            Ok(ResponseBuilder::success(operation)
                .with_message(format!("Base64 {} completed successfully", operation))
                .with_metadata("variant", json!(variant))
                .with_file_info(path, Some(result_data.len() as u64))
                .build())
        } else {
            // Return as string
            let output = match operation {
                "encode" => String::from_utf8_lossy(&result_data).to_string(),
                "decode" => {
                    // Try to convert to string, otherwise return hex
                    match String::from_utf8(result_data.clone()) {
                        Ok(s) => s,
                        Err(_) => hex::encode(&result_data),
                    }
                }
                _ => unreachable!(),
            };

            Ok(ResponseBuilder::success(operation)
                .with_message(format!("Base64 {} completed", operation))
                .with_result(json!({
                    "output": output,
                    "variant": variant,
                    "binary": operation == "decode" && !output.is_ascii()
                }))
                .build())
        }
    }
}

#[async_trait]
impl BaseAgent for Base64EncoderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Extract parameters using shared utility
        let params = extract_parameters(&input)?;

        // Process the operation
        let result = self.process_operation(params).await?;

        // Return the result as JSON formatted text
        Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(validation_error(
                "Input prompt cannot be empty",
                Some("prompt".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Base64 encoding error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for Base64EncoderTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "base64_encoder".to_string(),
            "Base64 encoding and decoding tool with standard and URL-safe variants".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: 'encode' or 'decode'".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "variant".to_string(),
            param_type: ParameterType::String,
            description: "Base64 variant: 'standard' or 'url-safe' (default: 'standard')"
                .to_string(),
            required: false,
            default: Some(json!("standard")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input data (text for encode, base64 for decode)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "input_file".to_string(),
            param_type: ParameterType::String,
            description: "Input file path (alternative to input)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "output_file".to_string(),
            param_type: ParameterType::String,
            description: "Output file path (optional)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "binary_input".to_string(),
            param_type: ParameterType::Boolean,
            description: "Treat input as hex string for binary data".to_string(),
            required: false,
            default: Some(json!(false)),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::safe()
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::strict()
            .with_memory_limit(50 * 1024 * 1024) // 50MB
            .with_cpu_limit(5000) // 5 seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[tokio::test]
    async fn test_encode_decode_text() {
        let tool = Base64EncoderTool::new();
        let test_text = "Hello, Base64!";

        // Test standard encoding
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
        assert!(output["success"].as_bool().unwrap_or(false));
        let encoded = output["result"]["output"].as_str().unwrap();
        assert_eq!(encoded, "SGVsbG8sIEJhc2U2NCE=");

        // Test standard decoding
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
        assert!(output["success"].as_bool().unwrap_or(false));
        let decoded = output["result"]["output"].as_str().unwrap();
        assert_eq!(decoded, test_text);
    }
    #[tokio::test]
    async fn test_url_safe_variant() {
        let tool = Base64EncoderTool::new();
        let test_data = "data with +/ characters";

        // Test URL-safe encoding
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
        assert!(output["success"].as_bool().unwrap_or(false));
        let encoded = output["result"]["output"].as_str().unwrap();
        assert!(!encoded.contains('+'));
        assert!(!encoded.contains('/'));

        // Test URL-safe decoding
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
        assert!(output["success"].as_bool().unwrap_or(false));
        let decoded = output["result"]["output"].as_str().unwrap();
        assert_eq!(decoded, test_data);
    }
    #[tokio::test]
    async fn test_binary_input() {
        let tool = Base64EncoderTool::new();
        let hex_data = "deadbeef";

        // Test encoding binary data
        let input = AgentInput::text("encode binary").with_parameter(
            "parameters",
            json!({
                "operation": "encode",
                "input": hex_data,
                "binary_input": true
            }),
        );

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        let encoded = output["result"]["output"].as_str().unwrap();
        assert_eq!(encoded, "3q2+7w==");
    }
    #[tokio::test]
    async fn test_invalid_operation() {
        let tool = Base64EncoderTool::new();

        let input = AgentInput::text("invalid operation").with_parameter(
            "parameters",
            json!({
                "operation": "invalid",
                "input": "test"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_missing_input() {
        let tool = Base64EncoderTool::new();

        let input = AgentInput::text("missing input").with_parameter(
            "parameters",
            json!({
                "operation": "encode"
            }),
        );

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = Base64EncoderTool::new();

        assert_eq!(tool.metadata().name, "base64-encoder");
        assert!(tool.metadata().description.contains("Base64"));
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }
}
