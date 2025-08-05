// ABOUTME: Hash calculation and verification tool supporting MD5, SHA-1, SHA-256, SHA-512
// ABOUTME: Provides hash computation for strings and files with verification capabilities

//! Hash calculation and verification tool
//!
//! This tool provides hash calculation capabilities including:
//! - Multiple hash algorithms (MD5, SHA-1, SHA-256, SHA-512)
//! - String and file hashing
//! - Hash verification
//! - Hex and Base64 output formats

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
    ComponentMetadata, ExecutionContext, Result,
};
use llmspell_utils::{
    encoding::{from_hex_string, hash_file, hash_string, to_hex_string, HashAlgorithm},
    error_builders::llmspell::{storage_error, validation_error},
    params::{
        extract_optional_string, extract_parameters, extract_required_string,
        extract_string_with_default,
    },
    response::ResponseBuilder,
    validators::validate_enum,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashCalculatorConfig {
    /// Default hash algorithm to use
    pub default_algorithm: HashAlgorithm,
    /// Default output format
    pub default_format: OutputFormat,
    /// Maximum file size for hashing (in bytes)
    pub max_file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Hex,
    Base64,
}

impl Default for HashCalculatorConfig {
    fn default() -> Self {
        Self {
            default_algorithm: HashAlgorithm::Sha256,
            default_format: OutputFormat::Hex,
            max_file_size: 100 * 1024 * 1024, // 100MB
        }
    }
}

/// Hash calculation tool
pub struct HashCalculatorTool {
    metadata: ComponentMetadata,
    config: HashCalculatorConfig,
}

impl HashCalculatorTool {
    /// Create a new hash calculator tool
    #[must_use]
    pub fn new(config: HashCalculatorConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "hash-calculator".to_string(),
                "Calculate and verify hashes using multiple algorithms".to_string(),
            ),
            config,
        }
    }

    fn parse_algorithm(&self, algorithm: Option<&str>) -> HashAlgorithm {
        match algorithm {
            Some("md5") => HashAlgorithm::Md5,
            Some("sha1" | "sha-1") => HashAlgorithm::Sha1,
            Some("sha256" | "sha-256") => HashAlgorithm::Sha256,
            Some("sha512" | "sha-512") => HashAlgorithm::Sha512,
            _ => self.config.default_algorithm,
        }
    }

    fn parse_format(&self, format: Option<&str>) -> OutputFormat {
        match format {
            Some("hex") => OutputFormat::Hex,
            Some("base64") => OutputFormat::Base64,
            _ => self.config.default_format.clone(),
        }
    }

    #[allow(clippy::unused_self)]
    fn format_hash(&self, hash: &[u8], format: &OutputFormat) -> String {
        match format {
            OutputFormat::Hex => to_hex_string(hash),
            OutputFormat::Base64 => llmspell_utils::encoding::base64_encode(hash),
        }
    }

    async fn check_file_size(&self, path: &Path) -> Result<u64> {
        let metadata = tokio::fs::metadata(path).await.map_err(|e| {
            storage_error(
                format!("Failed to read file metadata: {e}"),
                Some("read_metadata".to_string()),
            )
        })?;

        let file_size = metadata.len();
        llmspell_utils::validators::validate_file_size(file_size, self.config.max_file_size)?;

        Ok(file_size)
    }

    async fn execute_hash_operation(&self, params: &serde_json::Value) -> Result<AgentOutput> {
        let input_type = extract_string_with_default(params, "input_type", "string");
        let algorithm = self.parse_algorithm(extract_optional_string(params, "algorithm"));
        let format = self.parse_format(extract_optional_string(params, "format"));

        // Validate input type
        validate_enum(&input_type, &["string", "file"], "input_type")?;

        let hash = self.compute_hash(params, &input_type, algorithm).await?;
        let formatted = self.format_hash(&hash, &format);

        let response = ResponseBuilder::success("hash")
            .with_message(format!(
                "Calculated {} hash",
                algorithm.to_string().to_uppercase()
            ))
            .with_result(json!({
                "algorithm": algorithm.to_string(),
                "hash": formatted,
                "format": match format {
                    OutputFormat::Hex => "hex",
                    OutputFormat::Base64 => "base64",
                }
            }))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn execute_verify_operation(&self, params: &serde_json::Value) -> Result<AgentOutput> {
        let input_type = extract_string_with_default(params, "input_type", "string");
        let algorithm = self.parse_algorithm(extract_optional_string(params, "algorithm"));
        let expected_hash_str = extract_required_string(params, "expected_hash")?;
        let expected_format = extract_string_with_default(params, "expected_format", "hex");

        // Validate enums
        validate_enum(&input_type, &["string", "file"], "input_type")?;
        validate_enum(&expected_format, &["hex", "base64"], "expected_format")?;

        let expected_hash = self.decode_expected_hash(expected_hash_str, &expected_format)?;
        let actual_hash = self.compute_hash(params, &input_type, algorithm).await?;
        let matches = actual_hash == expected_hash;

        let response = if matches {
            ResponseBuilder::success("verify")
                .with_message("Hash verification successful")
                .with_result(json!({
                    "verified": true,
                    "algorithm": algorithm.to_string(),
                }))
        } else {
            ResponseBuilder::success("verify")
                .with_message("Hash verification failed")
                .with_result(json!({
                    "verified": false,
                    "algorithm": algorithm.to_string(),
                    "expected": self.format_hash(&expected_hash, &self.parse_format(Some(&expected_format))),
                    "actual": self.format_hash(&actual_hash, &self.parse_format(Some(&expected_format))),
                }))
        }
        .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn compute_hash(
        &self,
        params: &serde_json::Value,
        input_type: &str,
        algorithm: HashAlgorithm,
    ) -> Result<Vec<u8>> {
        match input_type {
            "string" => {
                let text = extract_required_string(params, "input")?;
                Ok(hash_string(text, algorithm))
            }
            "file" => {
                let file_path = extract_required_string(params, "file")?;
                let path = Path::new(file_path);
                self.check_file_size(path).await?;

                hash_file(path, algorithm).map_err(|e| {
                    storage_error(
                        format!("Failed to hash file: {e}"),
                        Some(format!("hash file {file_path}")),
                    )
                })
            }
            _ => unreachable!(), // Already validated
        }
    }

    fn decode_expected_hash(
        &self,
        expected_hash_str: &str,
        expected_format: &str,
    ) -> Result<Vec<u8>> {
        match expected_format {
            "hex" => from_hex_string(expected_hash_str).map_err(|_| {
                validation_error(
                    "Invalid hex string in expected_hash",
                    Some("expected_hash".to_string()),
                )
            }),
            "base64" => llmspell_utils::encoding::base64_decode(expected_hash_str).map_err(|_| {
                validation_error(
                    "Invalid base64 string in expected_hash",
                    Some("expected_hash".to_string()),
                )
            }),
            _ => unreachable!(), // Already validated
        }
    }
}

#[async_trait]
impl BaseAgent for HashCalculatorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        let params = extract_parameters(&input)?;
        let operation = extract_required_string(params, "operation")?;

        match operation {
            "hash" => self.execute_hash_operation(params).await,
            "verify" => self.execute_verify_operation(params).await,
            _ => Err(validation_error(
                format!("Invalid operation: {operation}"),
                Some("operation".to_string()),
            )),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(validation_error(
                "Input text cannot be empty",
                Some("text".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Hash calculator error: {error}")))
    }
}

#[async_trait]
impl Tool for HashCalculatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "hash_calculator".to_string(),
            "Calculate and verify hashes using various algorithms".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: 'hash' or 'verify'".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "algorithm".to_string(),
            param_type: ParameterType::String,
            description: "Hash algorithm: 'md5', 'sha1', 'sha256', or 'sha512'".to_string(),
            required: false,
            default: Some(json!("sha256")),
        })
        .with_parameter(ParameterDef {
            name: "input_type".to_string(),
            param_type: ParameterType::String,
            description: "Type of input: 'string' or 'file'".to_string(),
            required: false,
            default: Some(json!("string")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "String data to hash (for input_type=string)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "file".to_string(),
            param_type: ParameterType::String,
            description: "File path to hash (for input_type=file)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "format".to_string(),
            param_type: ParameterType::String,
            description: "Output format: 'hex' or 'base64'".to_string(),
            required: false,
            default: Some(json!("hex")),
        })
        .with_parameter(ParameterDef {
            name: "expected_hash".to_string(),
            param_type: ParameterType::String,
            description: "Expected hash for verification".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "expected_format".to_string(),
            param_type: ParameterType::String,
            description: "Format of expected hash: 'hex' or 'base64'".to_string(),
            required: false,
            default: Some(json!("hex")),
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted().with_file_access("*") // Needs file access for hashing files
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_cpu_limit(30000) // 30 seconds for large files
            .with_memory_limit(100 * 1024 * 1024) // 100MB for large files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::LLMSpellError;
    use llmspell_testing::tool_helpers::{create_test_tool, create_test_tool_input};
    use llmspell_utils::file_utils::write_file;
    use tempfile::TempDir;

    fn create_test_hash_calculator() -> HashCalculatorTool {
        HashCalculatorTool::new(HashCalculatorConfig::default())
    }

    #[tokio::test]
    async fn test_hash_string() {
        let tool = create_test_hash_calculator();
        let input = create_test_tool_input(vec![
            ("operation", "hash"),
            ("input", "hello world"),
            ("algorithm", "sha256"),
            ("format", "hex"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));
        assert_eq!(response["result"]["algorithm"], "SHA-256");
        assert!(response["result"]["hash"].is_string());
    }
    #[tokio::test]
    async fn test_hash_file() {
        let tool = create_test_hash_calculator();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        write_file(&file_path, b"file content").unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "hash"),
            ("input_type", "file"),
            ("file", &file_path.to_str().unwrap()),
            ("algorithm", "md5"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));
        assert_eq!(response["result"]["algorithm"], "MD5");
    }
    #[tokio::test]
    async fn test_verify_hash_success() {
        let tool = create_test_hash_calculator();
        let input = create_test_tool_input(vec![
            ("operation", "verify"),
            ("input", "test"),
            ("algorithm", "sha256"),
            (
                "expected_hash",
                "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
            ),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(response["result"]["verified"].as_bool().unwrap_or(false));
    }
    #[tokio::test]
    async fn test_verify_hash_failure() {
        let tool = create_test_hash_calculator();
        let input = create_test_tool_input(vec![
            ("operation", "verify"),
            ("input", "test"),
            ("algorithm", "sha256"),
            (
                "expected_hash",
                "0000000000000000000000000000000000000000000000000000000000000000",
            ),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert!(response["success"].as_bool().unwrap_or(false));
        assert!(!response["result"]["verified"].as_bool().unwrap_or(true));
    }
    #[tokio::test]
    async fn test_missing_required_parameter() {
        let tool = create_test_hash_calculator();
        let input = create_test_tool_input(vec![("data", "test")]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));
    }
    #[tokio::test]
    async fn test_invalid_algorithm() {
        let tool = create_test_hash_calculator();
        let input = create_test_tool_input(vec![
            ("operation", "hash"),
            ("input", "test"),
            ("algorithm", "invalid"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        let response: serde_json::Value = serde_json::from_str(&output.text).unwrap();

        assert_eq!(response["result"]["algorithm"], "SHA-256");
    }
    #[tokio::test]
    async fn test_file_size_limit() {
        let tool = HashCalculatorTool::new(HashCalculatorConfig {
            max_file_size: 10, // Very small limit
            ..Default::default()
        });

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");
        write_file(&file_path, b"This content is larger than 10 bytes").unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "hash"),
            ("input_type", "file"),
            ("file", &file_path.to_str().unwrap()),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LLMSpellError::Validation { .. }
        ));
    }
}
