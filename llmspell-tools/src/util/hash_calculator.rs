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
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use llmspell_utils::encoding::{
    from_hex_string, hash_file, hash_string, to_hex_string, verify_hash, HashAlgorithm,
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
            Some("sha1") | Some("sha-1") => HashAlgorithm::Sha1,
            Some("sha256") | Some("sha-256") => HashAlgorithm::Sha256,
            Some("sha512") | Some("sha-512") => HashAlgorithm::Sha512,
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

    fn format_hash(&self, hash: &[u8], format: &OutputFormat) -> String {
        match format {
            OutputFormat::Hex => to_hex_string(hash),
            OutputFormat::Base64 => llmspell_utils::encoding::base64_encode(hash),
        }
    }

    async fn check_file_size(&self, path: &Path) -> Result<()> {
        let metadata = tokio::fs::metadata(path)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to get file metadata: {}", e),
                operation: Some(format!("read metadata for {}", path.to_string_lossy())),
                source: None,
            })?;

        if metadata.len() > self.config.max_file_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "File size ({} bytes) exceeds maximum allowed size ({} bytes)",
                    metadata.len(),
                    self.config.max_file_size
                ),
                field: Some("file".to_string()),
            });
        }

        Ok(())
    }
}

impl Default for HashCalculatorTool {
    fn default() -> Self {
        Self::new(HashCalculatorConfig::default())
    }
}

#[async_trait]
impl BaseAgent for HashCalculatorTool {
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

        // Validate parameters
        self.validate_parameters(params).await?;

        // Extract operation type
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("hash");

        match operation {
            "hash" => {
                // Extract input type
                let input_type = params
                    .get("input_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("string");

                let algorithm =
                    self.parse_algorithm(params.get("algorithm").and_then(|v| v.as_str()));
                let format = self.parse_format(params.get("format").and_then(|v| v.as_str()));

                let hash = match input_type {
                    "string" => {
                        let text =
                            params.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
                                LLMSpellError::Validation {
                                    message: "Missing 'data' parameter for string hashing"
                                        .to_string(),
                                    field: Some("data".to_string()),
                                }
                            })?;
                        hash_string(text, algorithm)
                    }
                    "file" => {
                        let file_path =
                            params.get("file").and_then(|v| v.as_str()).ok_or_else(|| {
                                LLMSpellError::Validation {
                                    message: "Missing 'file' parameter for file hashing"
                                        .to_string(),
                                    field: Some("file".to_string()),
                                }
                            })?;

                        let path = Path::new(file_path);
                        self.check_file_size(path).await?;

                        hash_file(path, algorithm).map_err(|e| LLMSpellError::Storage {
                            message: format!("Failed to hash file: {}", e),
                            operation: Some(format!("hash file {}", file_path)),
                            source: None,
                        })?
                    }
                    _ => {
                        return Err(LLMSpellError::Validation {
                            message: format!("Invalid input_type: {}", input_type),
                            field: Some("input_type".to_string()),
                        });
                    }
                };

                let formatted = self.format_hash(&hash, &format);
                let result = json!({
                    "algorithm": algorithm.to_string(),
                    "hash": formatted,
                    "format": match format {
                        OutputFormat::Hex => "hex",
                        OutputFormat::Base64 => "base64",
                    }
                });

                Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
            }
            "verify" => {
                let input_type = params
                    .get("input_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("string");

                let algorithm =
                    self.parse_algorithm(params.get("algorithm").and_then(|v| v.as_str()));

                let expected_hash_str = params
                    .get("expected_hash")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| LLMSpellError::Validation {
                        message: "Missing 'expected_hash' parameter for verification".to_string(),
                        field: Some("expected_hash".to_string()),
                    })?;

                // Determine format of expected hash
                let expected_format = params
                    .get("expected_format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("hex");

                let expected_hash = match expected_format {
                    "hex" => from_hex_string(expected_hash_str).map_err(|_| {
                        LLMSpellError::Validation {
                            message: "Invalid hex string in expected_hash".to_string(),
                            field: Some("expected_hash".to_string()),
                        }
                    })?,
                    "base64" => llmspell_utils::encoding::base64_decode(expected_hash_str)
                        .map_err(|_| LLMSpellError::Validation {
                            message: "Invalid base64 string in expected_hash".to_string(),
                            field: Some("expected_hash".to_string()),
                        })?,
                    _ => {
                        return Err(LLMSpellError::Validation {
                            message: format!("Invalid expected_format: {}", expected_format),
                            field: Some("expected_format".to_string()),
                        });
                    }
                };

                let data = match input_type {
                    "string" => {
                        let text =
                            params.get("data").and_then(|v| v.as_str()).ok_or_else(|| {
                                LLMSpellError::Validation {
                                    message: "Missing 'data' parameter for verification"
                                        .to_string(),
                                    field: Some("data".to_string()),
                                }
                            })?;
                        text.as_bytes().to_vec()
                    }
                    "file" => {
                        let file_path =
                            params.get("file").and_then(|v| v.as_str()).ok_or_else(|| {
                                LLMSpellError::Validation {
                                    message: "Missing 'file' parameter for verification"
                                        .to_string(),
                                    field: Some("file".to_string()),
                                }
                            })?;

                        let path = Path::new(file_path);
                        self.check_file_size(path).await?;

                        tokio::fs::read(path)
                            .await
                            .map_err(|e| LLMSpellError::Storage {
                                message: format!("Failed to read file: {}", e),
                                operation: Some(format!("read file {}", file_path)),
                                source: None,
                            })?
                    }
                    _ => {
                        return Err(LLMSpellError::Validation {
                            message: format!("Invalid input_type: {}", input_type),
                            field: Some("input_type".to_string()),
                        });
                    }
                };

                let is_valid = verify_hash(&data, &expected_hash, algorithm);
                let result = json!({
                    "valid": is_valid,
                    "algorithm": algorithm.to_string(),
                    "message": if is_valid {
                        "Hash verification successful"
                    } else {
                        "Hash verification failed - data does not match expected hash"
                    }
                });

                Ok(AgentOutput::text(serde_json::to_string_pretty(&result)?))
            }
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown operation: {}", operation),
                field: Some("operation".to_string()),
            }),
        }
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
            "Hash calculation error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for HashCalculatorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // File access requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "hash_calculator".to_string(),
            "Calculate and verify hashes using multiple algorithms".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: hash or verify".to_string(),
            required: false,
            default: Some(json!("hash")),
        })
        .with_parameter(ParameterDef {
            name: "input_type".to_string(),
            param_type: ParameterType::String,
            description: "Type of input: string or file".to_string(),
            required: false,
            default: Some(json!("string")),
        })
        .with_parameter(ParameterDef {
            name: "algorithm".to_string(),
            param_type: ParameterType::String,
            description: "Hash algorithm: md5, sha1, sha256, sha512".to_string(),
            required: false,
            default: Some(json!("sha256")),
        })
        .with_parameter(ParameterDef {
            name: "data".to_string(),
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
            description: "Output format: hex or base64".to_string(),
            required: false,
            default: Some(json!("hex")),
        })
        .with_parameter(ParameterDef {
            name: "expected_hash".to_string(),
            param_type: ParameterType::String,
            description: "Expected hash value for verification".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "expected_format".to_string(),
            param_type: ParameterType::String,
            description: "Format of expected_hash: hex or base64".to_string(),
            required: false,
            default: Some(json!("hex")),
        })
        .with_returns(ParameterType::String)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted().with_file_access("*") // Allow read access to all files
    }

    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(50 * 1024 * 1024) // 50MB for file operations
            .with_cpu_limit(5000) // 5 seconds for large files
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_hash_string_md5() {
        let tool = HashCalculatorTool::default();
        let input = AgentInput::text("hash string".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "hash",
                "input_type": "string",
                "data": "Hello, World!",
                "algorithm": "md5",
                "format": "hex"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["algorithm"], "MD5");
        assert_eq!(parsed["hash"], "65a8e27d8879283831b664bd8b7f0ad4");
        assert_eq!(parsed["format"], "hex");
    }

    #[tokio::test]
    async fn test_hash_string_sha256() {
        let tool = HashCalculatorTool::default();
        let input = AgentInput::text("hash string".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "hash",
                "input_type": "string",
                "data": "test data",
                "algorithm": "sha256"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["algorithm"], "SHA-256");
        assert_eq!(
            parsed["hash"],
            "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9"
        );
    }

    #[tokio::test]
    async fn test_hash_file() {
        let tool = HashCalculatorTool::default();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");

        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Test file content").unwrap();

        let input = AgentInput::text("hash file".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "hash",
                "input_type": "file",
                "file": file_path.to_str().unwrap(),
                "algorithm": "sha256"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["algorithm"], "SHA-256");
        assert!(parsed["hash"].is_string());
    }

    #[tokio::test]
    async fn test_verify_hash_success() {
        let tool = HashCalculatorTool::default();
        let input = AgentInput::text("verify hash".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "verify",
                "input_type": "string",
                "data": "test data",
                "algorithm": "sha256",
                "expected_hash": "916f0027a575074ce72a331777c3478d6513f786a591bd892da1a577bf2335f9",
                "expected_format": "hex"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["valid"], true);
        assert_eq!(parsed["algorithm"], "SHA-256");
    }

    #[tokio::test]
    async fn test_verify_hash_failure() {
        let tool = HashCalculatorTool::default();
        let input = AgentInput::text("verify hash".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "verify",
                "input_type": "string",
                "data": "test data",
                "algorithm": "sha256",
                "expected_hash": "0000000000000000000000000000000000000000000000000000000000000000",
                "expected_format": "hex"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["valid"], false);
    }

    #[tokio::test]
    async fn test_base64_format() {
        let tool = HashCalculatorTool::default();
        let input = AgentInput::text("hash with base64".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "hash",
                "input_type": "string",
                "data": "test",
                "algorithm": "sha256",
                "format": "base64"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_ok());

        let output = result.unwrap().text;
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["format"], "base64");
        // SHA-256 hash of "test" in base64
        assert_eq!(
            parsed["hash"],
            "n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="
        );
    }

    #[tokio::test]
    async fn test_all_algorithms() {
        let tool = HashCalculatorTool::default();
        let algorithms = vec!["md5", "sha1", "sha256", "sha512"];

        for algo in algorithms {
            let input = AgentInput::text("hash test".to_string()).with_parameter(
                "parameters".to_string(),
                json!({
                    "operation": "hash",
                    "input_type": "string",
                    "data": "test",
                    "algorithm": algo
                }),
            );
            let context = ExecutionContext::with_conversation("test".to_string());

            let result = tool.execute(input, context).await;
            assert!(result.is_ok(), "Algorithm {} failed", algo);
        }
    }

    #[tokio::test]
    async fn test_file_size_limit() {
        let mut config = HashCalculatorConfig::default();
        config.max_file_size = 100; // 100 bytes limit
        let tool = HashCalculatorTool::new(config);

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("large.txt");

        let mut file = File::create(&file_path).unwrap();
        // Write more than 100 bytes
        for _ in 0..20 {
            writeln!(file, "This is a line of text").unwrap();
        }

        let input = AgentInput::text("hash large file".to_string()).with_parameter(
            "parameters".to_string(),
            json!({
                "operation": "hash",
                "input_type": "file",
                "file": file_path.to_str().unwrap(),
                "algorithm": "sha256"
            }),
        );
        let context = ExecutionContext::with_conversation("test".to_string());

        let result = tool.execute(input, context).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("exceeds maximum allowed size"));
    }

    #[tokio::test]
    async fn test_tool_metadata() {
        let tool = HashCalculatorTool::default();
        assert_eq!(tool.category(), ToolCategory::Utility);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);
        assert_eq!(tool.metadata().name, "hash-calculator");

        let schema = tool.schema();
        assert_eq!(schema.name, "hash_calculator");
        assert!(schema.parameters.len() >= 8);
    }
}
