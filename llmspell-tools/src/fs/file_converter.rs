// ABOUTME: File format and encoding conversion tool
// ABOUTME: Handles text encoding, line endings, and format conversions

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result as LLMResult,
};
use llmspell_security::sandbox::FileSandbox;
use llmspell_utils::{
    encoding::{
        convert_line_endings, convert_text_encoding, detect_line_ending, detect_text_encoding,
        remove_bom, spaces_to_tabs, tabs_to_spaces, LineEnding, TextEncoding,
    },
    extract_optional_bool, extract_optional_string, extract_optional_u64, extract_parameters,
    extract_required_string,
    response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs;
use tracing::{debug, info, instrument};

/// File converter tool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConverterConfig {
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Create backup files before conversion
    pub create_backups: bool,
    /// Output directory for converted files
    pub output_dir: Option<PathBuf>,
    /// Preserve original file timestamps
    pub preserve_timestamps: bool,
}

impl Default for FileConverterConfig {
    fn default() -> Self {
        Self {
            max_file_size: 50 * 1024 * 1024, // 50MB
            create_backups: true,
            output_dir: None,
            preserve_timestamps: true,
        }
    }
}

/// File converter tool for handling various file format conversions
#[derive(Clone)]
pub struct FileConverterTool {
    metadata: ComponentMetadata,
    config: FileConverterConfig,
    sandbox: Arc<FileSandbox>,
}

impl FileConverterTool {
    /// Create a new file converter tool
    #[must_use]
    pub fn new(config: FileConverterConfig, sandbox: Arc<FileSandbox>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "file-converter".to_string(),
                "File format and encoding conversion tool".to_string(),
            ),
            config,
            sandbox,
        }
    }

    /// Convert file encoding
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to read input file
    /// - Text encoding conversion fails
    /// - Failed to write output file
    #[instrument(skip(self))]
    async fn convert_encoding(
        &self,
        input_path: &Path,
        output_path: &Path,
        from_encoding: Option<TextEncoding>,
        to_encoding: TextEncoding,
    ) -> Result<()> {
        let content = fs::read(input_path).await?;
        let content = remove_bom(&content);
        let from_encoding = from_encoding.unwrap_or_else(|| detect_text_encoding(content));

        debug!(
            "Converting {} from {} to {}",
            input_path.display(),
            from_encoding,
            to_encoding
        );

        let converted = convert_text_encoding(content, from_encoding, to_encoding)
            .map_err(|e| anyhow::anyhow!("Text encoding conversion failed: {}", e))?;
        fs::write(output_path, converted).await?;

        info!(
            "Successfully converted {} from {} to {}",
            input_path.display(),
            from_encoding,
            to_encoding
        );

        Ok(())
    }

    /// Convert line endings
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to read input file
    /// - Failed to write output file
    #[instrument(skip(self))]
    async fn convert_line_endings(
        &self,
        input_path: &Path,
        output_path: &Path,
        target_ending: LineEnding,
    ) -> Result<()> {
        let content = fs::read_to_string(input_path).await?;
        let current_ending = detect_line_ending(&content);

        debug!(
            "Converting line endings in {} from {} to {}",
            input_path.display(),
            current_ending,
            target_ending
        );

        let converted = convert_line_endings(&content, target_ending);
        fs::write(output_path, converted).await?;

        info!(
            "Successfully converted line endings in {} from {} to {}",
            input_path.display(),
            current_ending,
            target_ending
        );

        Ok(())
    }

    /// Convert tabs to spaces or vice versa
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to read input file
    /// - Failed to write output file
    #[instrument(skip(self))]
    async fn convert_indentation(
        &self,
        input_path: &Path,
        output_path: &Path,
        convert_to_spaces: bool,
        tab_size: usize,
    ) -> Result<()> {
        let content = fs::read_to_string(input_path).await?;

        let converted = if convert_to_spaces {
            debug!(
                "Converting tabs to spaces in {} (tab size: {})",
                input_path.display(),
                tab_size
            );
            tabs_to_spaces(&content, tab_size)
        } else {
            debug!(
                "Converting spaces to tabs in {} (tab size: {})",
                input_path.display(),
                tab_size
            );
            spaces_to_tabs(&content, tab_size)
        };

        fs::write(output_path, converted).await?;

        info!(
            "Successfully converted indentation in {}",
            input_path.display()
        );

        Ok(())
    }

    /// Create backup file
    ///
    /// # Errors
    ///
    /// Returns an error if file copy operation fails
    #[instrument(skip(self))]
    async fn create_backup(&self, file_path: &Path) -> Result<PathBuf> {
        let backup_path = file_path.with_extension(format!(
            "{}.backup",
            file_path.extension().unwrap_or_default().to_string_lossy()
        ));

        fs::copy(file_path, &backup_path).await?;
        debug!("Created backup: {}", backup_path.display());

        Ok(backup_path)
    }

    /// Determine output path
    fn get_output_path(&self, input_path: &Path, operation: &str) -> PathBuf {
        self.config.output_dir.as_ref().map_or_else(
            || input_path.to_path_buf(),
            |output_dir| {
                let stem = input_path.file_stem().unwrap_or_default();
                let extension = input_path.extension().unwrap_or_default();

                let new_filename = format!(
                    "{}_{}{}",
                    stem.to_string_lossy(),
                    operation,
                    if extension.is_empty() {
                        String::new()
                    } else {
                        format!(".{}", extension.to_string_lossy())
                    }
                );

                output_dir.join(new_filename)
            },
        )
    }

    /// Validate parameters for file conversion operations
    ///
    /// # Errors
    ///
    /// Returns an error if an invalid operation is specified
    #[allow(clippy::unused_async)]
    #[instrument(skip(self))]
    async fn validate_parameters(&self, params: &serde_json::Value) -> LLMResult<()> {
        // Validate operation
        if let Some(operation) = params.get("operation").and_then(|v| v.as_str()) {
            if !matches!(operation, "encoding" | "line_endings" | "indentation") {
                return Err(LLMSpellError::Validation {
                    message: format!("Invalid operation: {operation}"),
                    field: Some("operation".to_string()),
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl BaseAgent for FileConverterTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[allow(clippy::too_many_lines)]
    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> LLMResult<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        self.validate_parameters(params).await?;

        // Extract operation
        let operation = extract_required_string(params, "operation")?;

        // Extract input path
        let path = extract_required_string(params, "path")?;
        let path = PathBuf::from(path);

        // Validate input path
        self.sandbox
            .validate_path(&path)
            .map_err(|e| LLMSpellError::Security {
                message: format!("Path validation failed: {e}"),
                violation_type: Some("path_validation".to_string()),
            })?;

        if !path.exists() {
            return Err(LLMSpellError::Validation {
                message: format!("Input file does not exist: {}", path.display()),
                field: Some("path".to_string()),
            });
        }

        // Check file size
        let metadata = fs::metadata(&path)
            .await
            .map_err(|e| LLMSpellError::Storage {
                message: format!("Failed to read file metadata: {e}"),
                operation: Some("metadata".to_string()),
                source: Some(Box::new(e)),
            })?;

        if metadata.len() > self.config.max_file_size as u64 {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "File too large: {} bytes (max: {})",
                    metadata.len(),
                    self.config.max_file_size
                ),
                field: Some("input_path".to_string()),
            });
        }

        // Determine output path
        let target_path = extract_optional_string(params, "target_path")
            .map_or_else(|| self.get_output_path(&path, operation), PathBuf::from);

        // Validate output path
        self.sandbox
            .validate_path(&target_path)
            .map_err(|e| LLMSpellError::Security {
                message: format!("Output path validation failed: {e}"),
                violation_type: Some("path_validation".to_string()),
            })?;

        // Create backup if enabled
        if self.config.create_backups && path == target_path {
            self.create_backup(&path)
                .await
                .map_err(|e| LLMSpellError::Storage {
                    message: format!("Failed to create backup: {e}"),
                    operation: Some("backup".to_string()),
                    source: None,
                })?;
        }

        // Execute conversion based on operation
        match operation {
            "encoding" => {
                let from_encoding =
                    extract_optional_string(params, "from_encoding").and_then(|s| {
                        match s.to_lowercase().as_str() {
                            "utf8" | "utf-8" => Some(TextEncoding::Utf8),
                            "utf16le" | "utf-16le" => Some(TextEncoding::Utf16Le),
                            "utf16be" | "utf-16be" => Some(TextEncoding::Utf16Be),
                            "windows1252" | "windows-1252" => Some(TextEncoding::Windows1252),
                            "iso88591" | "iso-8859-1" => Some(TextEncoding::Iso88591),
                            "ascii" => Some(TextEncoding::Ascii),
                            _ => None,
                        }
                    });

                let to_encoding = extract_required_string(params, "to_encoding").and_then(|s| {
                    match s.to_lowercase().as_str() {
                        "utf8" | "utf-8" => Ok(TextEncoding::Utf8),
                        "utf16le" | "utf-16le" => Ok(TextEncoding::Utf16Le),
                        "utf16be" | "utf-16be" => Ok(TextEncoding::Utf16Be),
                        "windows1252" | "windows-1252" => Ok(TextEncoding::Windows1252),
                        "iso88591" | "iso-8859-1" => Ok(TextEncoding::Iso88591),
                        "ascii" => Ok(TextEncoding::Ascii),
                        _ => Err(LLMSpellError::Validation {
                            message: format!("Invalid encoding: {s}"),
                            field: Some("to_encoding".to_string()),
                        }),
                    }
                })?;

                self.convert_encoding(&path, &target_path, from_encoding, to_encoding)
                    .await
                    .map_err(|e| LLMSpellError::Tool {
                        message: format!("Encoding conversion failed: {e}"),
                        tool_name: Some("file-converter".to_string()),
                        source: None,
                    })?;
            }

            "line_endings" => {
                let line_ending = extract_required_string(params, "line_ending").and_then(|s| {
                    match s.to_lowercase().as_str() {
                        "lf" => Ok(LineEnding::Lf),
                        "crlf" => Ok(LineEnding::Crlf),
                        "cr" => Ok(LineEnding::Cr),
                        _ => Err(LLMSpellError::Validation {
                            message: format!("Invalid line ending: {s}"),
                            field: Some("line_ending".to_string()),
                        }),
                    }
                })?;

                self.convert_line_endings(&path, &target_path, line_ending)
                    .await
                    .map_err(|e| LLMSpellError::Tool {
                        message: format!("Line ending conversion failed: {e}"),
                        tool_name: Some("file-converter".to_string()),
                        source: None,
                    })?;
            }

            "indentation" => {
                let convert_to_spaces =
                    extract_optional_bool(params, "convert_to_spaces").unwrap_or(true);

                let tab_size =
                    usize::try_from(extract_optional_u64(params, "tab_size").unwrap_or(4))
                        .map_err(|_| LLMSpellError::Validation {
                            message: "Tab size value too large for platform".to_string(),
                            field: Some("tab_size".to_string()),
                        })?;

                self.convert_indentation(&path, &target_path, convert_to_spaces, tab_size)
                    .await
                    .map_err(|e| LLMSpellError::Tool {
                        message: format!("Indentation conversion failed: {e}"),
                        tool_name: Some("file-converter".to_string()),
                        source: None,
                    })?;
            }

            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!("Unknown operation: {operation}"),
                    field: Some("operation".to_string()),
                });
            }
        }

        // Preserve timestamps if enabled
        if self.config.preserve_timestamps {
            let original_metadata =
                fs::metadata(&path)
                    .await
                    .map_err(|e| LLMSpellError::Storage {
                        message: format!("Failed to read original metadata: {e}"),
                        operation: Some("metadata".to_string()),
                        source: None,
                    })?;
            if let (Ok(_accessed), Ok(_modified)) =
                (original_metadata.accessed(), original_metadata.modified())
            {
                debug!("Would preserve timestamps for {}", target_path.display());
            }
        }

        // Return results using ResponseBuilder
        let (output_text, response) = ResponseBuilder::success(operation)
            .with_message("File conversion completed successfully".to_string())
            .with_result(json!({
                "input_path": path.to_string_lossy(),
                "output_path": target_path.to_string_lossy(),
                "operation": operation
            }))
            .build_for_output();

        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("operation".to_string(), operation.into());
        metadata.extra.insert("response".to_string(), response);

        Ok(AgentOutput::text(output_text).with_metadata(metadata))
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, input: &AgentInput) -> LLMResult<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> LLMResult<AgentOutput> {
        Ok(AgentOutput::text(format!("File converter error: {error}")))
    }
}

#[async_trait]
impl Tool for FileConverterTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Filesystem
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // File conversion requires restricted security
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "file-converter".to_string(),
            "Convert file formats, encodings, and line endings".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Conversion operation: encoding, line_endings, indentation".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "path".to_string(),
            param_type: ParameterType::String,
            description: "Path to input file".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "target_path".to_string(),
            param_type: ParameterType::String,
            description: "Path to output file (optional)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "from_encoding".to_string(),
            param_type: ParameterType::String,
            description: "Source encoding (optional, auto-detected if not specified)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "to_encoding".to_string(),
            param_type: ParameterType::String,
            description: "Target encoding (for encoding conversion)".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "line_ending".to_string(),
            param_type: ParameterType::String,
            description: "Target line ending: lf, crlf, cr (for line ending conversion)"
                .to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "convert_to_spaces".to_string(),
            param_type: ParameterType::Boolean,
            description: "Convert tabs to spaces (true) or spaces to tabs (false)".to_string(),
            required: false,
            default: Some(json!(true)),
        })
        .with_parameter(ParameterDef {
            name: "tab_size".to_string(),
            param_type: ParameterType::Number,
            description: "Tab size for indentation conversion (default: 4)".to_string(),
            required: false,
            default: Some(json!(4)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
    use llmspell_security::sandbox::SandboxContext;
    use llmspell_testing::tool_helpers::create_test_tool_input;
    use std::collections::HashMap;
    use tempfile::TempDir;
    use tokio::fs;

    fn create_test_file_converter() -> (FileConverterTool, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        // Create sandbox context
        let security_requirements = SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec!["*".to_string()],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: HashMap::new(),
        };
        let resource_limits = ResourceLimits::default();
        let context = SandboxContext::new(
            "test_file_converter".to_string(),
            security_requirements,
            resource_limits,
        );
        let sandbox = Arc::new(FileSandbox::new(context).unwrap());

        let config = FileConverterConfig::default();
        let tool = FileConverterTool::new(config, sandbox);

        (tool, temp_dir)
    }

    #[tokio::test]
    async fn test_encoding_conversion() {
        let (tool, temp_dir) = create_test_file_converter();

        // Create test file
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, World!").await.unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "encoding"),
            ("path", &test_file.to_string_lossy()),
            ("to_encoding", "utf8"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result
            .text
            .contains("File conversion completed successfully"));
    }
    #[tokio::test]
    async fn test_line_ending_conversion() {
        let (tool, temp_dir) = create_test_file_converter();

        // Create test file with mixed line endings
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "line1\r\nline2\nline3\r")
            .await
            .unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "line_endings"),
            ("path", &test_file.to_string_lossy()),
            ("line_ending", "lf"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result
            .text
            .contains("File conversion completed successfully"));

        // Verify conversion
        let content = fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
    }
    #[tokio::test]
    async fn test_indentation_conversion() {
        let (tool, temp_dir) = create_test_file_converter();

        // Create test file with tabs
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "line1\tindented\ttext")
            .await
            .unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "indentation"),
            ("path", &test_file.to_string_lossy()),
            ("convert_to_spaces", "true"),
            ("tab_size", "4"),
        ]);

        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(result
            .text
            .contains("File conversion completed successfully"));

        // Verify conversion
        let content = fs::read_to_string(&test_file).await.unwrap();
        assert_eq!(content, "line1    indented    text");
    }
    #[tokio::test]
    async fn test_invalid_operation() {
        let (tool, temp_dir) = create_test_file_converter();

        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test").await.unwrap();

        let input = create_test_tool_input(vec![
            ("operation", "invalid"),
            ("path", &test_file.to_string_lossy()),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_missing_parameters() {
        let (tool, _temp_dir) = create_test_file_converter();

        let input = AgentInput {
            text: "Missing params".to_string(),
            media: vec![],
            context: None,
            parameters: HashMap::new(),
            output_modalities: vec![],
        };

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_nonexistent_file() {
        let (tool, temp_dir) = create_test_file_converter();

        let nonexistent_file = temp_dir.path().join("nonexistent.txt");

        let input = create_test_tool_input(vec![
            ("operation", "encoding"),
            ("path", &nonexistent_file.to_string_lossy()),
            ("to_encoding", "utf8"),
        ]);

        let result = tool.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_tool_metadata() {
        let (tool, _temp_dir) = create_test_file_converter();

        let metadata = tool.metadata();
        assert_eq!(metadata.name, "file-converter");
        assert_eq!(metadata.version, llmspell_core::Version::new(0, 1, 0));
        assert!(metadata
            .description
            .contains("File format and encoding conversion"));

        let schema = tool.schema();
        assert_eq!(schema.name, "file-converter");
        assert_eq!(tool.category(), ToolCategory::Filesystem);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);
    }
}
