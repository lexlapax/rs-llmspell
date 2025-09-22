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
use std::collections::HashMap;
use std::fs;
use std::time::Instant;
use tracing::{debug, error, info, instrument};

/// Base64 encoding/decoding tool
#[derive(Debug, Clone)]
pub struct Base64EncoderTool {
    /// Tool metadata
    metadata: ComponentMetadata,
}

impl Default for Base64EncoderTool {
    fn default() -> Self {
        info!(
            tool_name = "base64-encoder",
            supported_operations = 2,  // encode, decode
            supported_variants = 2,    // standard, url-safe
            supported_input_types = 3, // text, binary (hex), file
            max_file_size_mb = 50,
            cpu_limit_seconds = 5,
            security_level = "Safe",
            category = "Utility",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating Base64EncoderTool"
        );
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

    fn validate_operation(params: &Value) -> Result<String> {
        let operation = extract_required_string(params, "operation")?;
        validate_enum(&operation, &["encode", "decode"], "operation")?;
        Ok(operation.to_string())
    }

    fn validate_variant(params: &Value) -> Result<String> {
        let variant = extract_optional_string(params, "variant").unwrap_or("standard");
        validate_enum(&variant, &["standard", "url-safe"], "variant")?;
        Ok(variant.to_string())
    }

    fn load_input_from_file(file_path: &str) -> Result<Vec<u8>> {
        fs::read(file_path).map_err(|e| {
            storage_error(
                format!("Failed to read input file: {e}"),
                Some("read_file".to_string()),
            )
        })
    }

    fn load_input_from_string(input: &str, binary_input: bool) -> Result<Vec<u8>> {
        if binary_input {
            hex::decode(input).map_err(|e| {
                validation_error(
                    format!("Failed to parse hex input: {e}"),
                    Some("input".to_string()),
                )
            })
        } else {
            Ok(input.as_bytes().to_vec())
        }
    }

    fn load_input_data(params: &Value) -> Result<Vec<u8>> {
        let input_file = extract_optional_string(params, "input_file");
        let input_str = extract_optional_string(params, "input");
        let binary_input = extract_optional_bool(params, "binary_input").unwrap_or(false);

        input_file.map_or_else(
            || {
                input_str.map_or_else(
                    || {
                        Err(validation_error(
                            "Either 'input' or 'input_file' must be provided",
                            Some("input".to_string()),
                        ))
                    },
                    |input| Self::load_input_from_string(input, binary_input),
                )
            },
            Self::load_input_from_file,
        )
    }

    fn encode_data(input_data: &[u8], variant: &str) -> Vec<u8> {
        let encoded = if variant == "url-safe" {
            base64_encode_url_safe(input_data)
        } else {
            base64_encode(input_data)
        };
        encoded.into_bytes()
    }

    fn prepare_decode_string(params: &Value, input_data: &[u8]) -> Result<String> {
        extract_optional_string(params, "input").map_or_else(
            || {
                String::from_utf8(input_data.to_vec()).map_err(|e| {
                    validation_error(
                        format!("Input file contains invalid UTF-8 for Base64 decoding: {e}"),
                        Some("input_file".to_string()),
                    )
                })
            },
            |input_str| Ok(input_str.to_string()),
        )
    }

    fn decode_data(input_str: &str, variant: &str) -> Result<Vec<u8>> {
        let result = if variant == "url-safe" {
            base64_decode_url_safe(input_str)
        } else {
            base64_decode(input_str)
        };

        result.map_err(|e| {
            validation_error(
                format!("Base64 decode error: {e}"),
                Some("input".to_string()),
            )
        })
    }

    fn perform_operation(
        operation: &str,
        params: &Value,
        input_data: &[u8],
        variant: &str,
    ) -> Result<Vec<u8>> {
        match operation {
            "encode" => Ok(Self::encode_data(input_data, variant)),
            "decode" => {
                let input_str = Self::prepare_decode_string(params, input_data)?;
                Self::decode_data(&input_str, variant)
            }
            _ => unreachable!("Operation already validated"),
        }
    }

    fn write_output_file(path: &str, data: &[u8]) -> Result<()> {
        fs::write(path, data).map_err(|e| {
            storage_error(
                format!("Failed to write output file: {e}"),
                Some("write_file".to_string()),
            )
        })
    }

    fn format_output_string(operation: &str, result_data: &[u8]) -> String {
        match operation {
            "encode" => String::from_utf8_lossy(result_data).to_string(),
            "decode" => {
                String::from_utf8(result_data.to_vec()).unwrap_or_else(|_| hex::encode(result_data))
            }
            _ => unreachable!(),
        }
    }

    fn build_file_response(operation: &str, variant: &str, path: &str, size: usize) -> Value {
        ResponseBuilder::success(operation)
            .with_message(format!("Base64 {operation} completed successfully"))
            .with_metadata("variant", json!(variant))
            .with_file_info(path, Some(size as u64))
            .build()
    }

    fn build_string_response(operation: &str, variant: &str, output: &str) -> Value {
        let is_binary = operation == "decode" && !output.is_ascii();
        ResponseBuilder::success(operation)
            .with_message(format!("Base64 {operation} completed"))
            .with_result(json!({
                "output": output,
                "variant": variant,
                "binary": is_binary
            }))
            .build()
    }

    /// Process Base64 operation
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn process_operation(&self, params: &Value) -> Result<Value> {
        let operation = Self::validate_operation(params)?;
        let variant = Self::validate_variant(params)?;
        let input_data = Self::load_input_data(params)?;

        let result_data = Self::perform_operation(&operation, params, &input_data, &variant)?;

        // Handle output
        let output_path = extract_optional_string(params, "output_file");

        if let Some(path) = output_path {
            Self::write_output_file(path, &result_data)?;
            Ok(Self::build_file_response(
                &operation,
                &variant,
                path,
                result_data.len(),
            ))
        } else {
            let output = Self::format_output_string(&operation, &result_data);
            Ok(Self::build_string_response(&operation, &variant, &output))
        }
    }
}

#[async_trait]
impl BaseAgent for Base64EncoderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let execute_start = Instant::now();
        info!(
            tool_name = %self.metadata().name,
            input_text_length = input.text.len(),
            has_parameters = !input.parameters.is_empty(),
            "Starting Base64EncoderTool execution"
        );

        // Extract parameters using shared utility
        let params = extract_parameters(&input)?;
        debug!(
            param_count = params.as_object().map_or(0, serde_json::Map::len),
            "Successfully extracted parameters"
        );

        let operation_start = Instant::now();
        let response = self.process_operation(params).await?;

        let total_duration_ms = execute_start.elapsed().as_millis();
        let operation_duration_ms = operation_start.elapsed().as_millis();

        debug!(
            tool_name = %self.metadata().name,
            total_duration_ms,
            operation_duration_ms,
            parameter_extraction_ms = 0, // Simplified timing
            "Base64EncoderTool execution completed successfully"
        );

        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&response).unwrap(),
        ))
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        // Validation is performed in process_operation
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        error!(
            tool_name = %self.metadata().name,
            error = %error,
            "Handling error in Base64EncoderTool"
        );
        let error_response = ResponseBuilder::error("base64", error.to_string()).build();
        Ok(AgentOutput::text(
            serde_json::to_string_pretty(&error_response).unwrap(),
        ))
    }
}

#[async_trait]
impl Tool for Base64EncoderTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: 'encode' or 'decode'".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Input text or Base64 string (optional if input_file provided)"
                .to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "input_file".to_string(),
            param_type: ParameterType::String,
            description: "Path to input file (optional if input provided)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "output_file".to_string(),
            param_type: ParameterType::String,
            description: "Path to output file (optional)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "variant".to_string(),
            param_type: ParameterType::String,
            description: "Encoding variant: 'standard' (default) or 'url-safe'".to_string(),
            required: false,
            default: Some(json!("standard")),
        })
        .with_parameter(ParameterDef {
            name: "binary_input".to_string(),
            param_type: ParameterType::Boolean,
            description: "Treat input string as hex-encoded binary (default: false)".to_string(),
            required: false,
            default: Some(json!(false)),
        })
    }

    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB
            max_cpu_time_ms: Some(5000),               // 5 seconds
            max_network_bps: None,
            max_file_ops_per_sec: Some(10),
            custom_limits: HashMap::default(),
        }
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Safe,
            file_permissions: Vec::default(),
            network_permissions: Vec::default(),
            env_permissions: Vec::default(),
            custom_requirements: HashMap::default(),
        }
    }
}
