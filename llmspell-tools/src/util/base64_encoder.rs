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
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::encoding::{
    base64_decode, base64_decode_url_safe, base64_encode, base64_encode_url_safe,
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
        // Extract operation
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'operation' parameter".to_string(),
                field: Some("operation".to_string()),
            })?;

        // Extract variant (default to standard)
        let variant = params
            .get("variant")
            .and_then(|v| v.as_str())
            .unwrap_or("standard");

        // Get input data
        let input_data = if let Some(file_path) = params.get("input_file").and_then(|v| v.as_str())
        {
            // Read from file
            fs::read(file_path).map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to read input file: {}", e),
                tool_name: Some(self.metadata.name.clone()),
                source: None,
            })?
        } else if let Some(input) = params.get("input").and_then(|v| v.as_str()) {
            let binary_input = params
                .get("binary_input")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if binary_input {
                // Parse hex string as binary
                hex::decode(input).map_err(|e| LLMSpellError::Tool {
                    message: format!("Failed to parse hex input: {}", e),
                    tool_name: Some(self.metadata.name.clone()),
                    source: None,
                })?
            } else {
                // Use text input
                input.as_bytes().to_vec()
            }
        } else {
            return Err(LLMSpellError::Validation {
                message: "Either 'input' or 'input_file' must be provided".to_string(),
                field: Some("input".to_string()),
            });
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
                let input_str = if let Some(input) = params.get("input").and_then(|v| v.as_str()) {
                    input.to_string()
                } else {
                    // Convert file data to string for decoding
                    String::from_utf8(input_data).map_err(|e| LLMSpellError::Tool {
                        message: format!(
                            "Input file contains invalid UTF-8 for Base64 decoding: {}",
                            e
                        ),
                        tool_name: Some(self.metadata.name.clone()),
                        source: None,
                    })?
                };

                let decoded = match variant {
                    "url-safe" => base64_decode_url_safe(&input_str),
                    _ => base64_decode(&input_str),
                };

                decoded.map_err(|e| LLMSpellError::Tool {
                    message: format!("Base64 decode error: {}", e),
                    tool_name: Some(self.metadata.name.clone()),
                    source: None,
                })?
            }
            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!("Invalid operation: {}", operation),
                    field: Some("operation".to_string()),
                });
            }
        };

        // Handle output
        if let Some(output_path) = params.get("output_file").and_then(|v| v.as_str()) {
            // Write to file
            fs::write(output_path, &result_data).map_err(|e| LLMSpellError::Tool {
                message: format!("Failed to write output file: {}", e),
                tool_name: Some(self.metadata.name.clone()),
                source: None,
            })?;

            info!(
                "Base64 {} completed: {} -> {}",
                operation,
                params
                    .get("input_file")
                    .and_then(|v| v.as_str())
                    .unwrap_or("input"),
                output_path
            );

            Ok(json!({
                "success": true,
                "operation": operation,
                "variant": variant,
                "output_file": output_path,
                "size": result_data.len()
            }))
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

            Ok(json!({
                "success": true,
                "operation": operation,
                "variant": variant,
                "output": output,
                "binary": operation == "decode" && !output.is_ascii()
            }))
        }
    }
}

#[async_trait]
impl BaseAgent for Base64EncoderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters from input
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters in input".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        // Process the operation
        let result = self.process_operation(params).await?;

        // Return the result as JSON formatted text
        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&result).unwrap(),
        ))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
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
        let encoded = output["output"].as_str().unwrap();
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
        let decoded = output["output"].as_str().unwrap();
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
        let encoded = output["output"].as_str().unwrap();
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
        let decoded = output["output"].as_str().unwrap();
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
        let encoded = output["output"].as_str().unwrap();
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
