// ABOUTME: PDF document processing tool for text extraction and metadata analysis
// ABOUTME: Provides secure PDF text extraction with file size limits and content validation

//! PDF Processor tool
//!
//! This tool provides PDF document processing including:
//! - Text extraction from PDF documents
//! - Metadata extraction (title, author, creation date, etc.)
//! - Page-specific text extraction
//! - Security features and resource limits

use crate::resource_limited::ResourceLimited;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits as ToolResourceLimits, SecurityLevel,
            SecurityRequirements, Tool, ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_optional_u64, extract_parameters, extract_required_string,
        extract_string_with_default,
    },
    resource_limits::{ResourceLimits, ResourceTracker},
    response::ResponseBuilder,
    timeout::with_timeout,
};
use pdf_extract::{extract_text_from_mem, OutputError as PdfError};
use serde_json::{json, Value as JsonValue};
use std::fs;
use std::path::Path;

/// PDF Processor tool for document analysis and text extraction
#[derive(Debug, Clone)]
pub struct PdfProcessorTool {
    /// Tool metadata
    metadata: ComponentMetadata,
    /// Maximum file size in bytes (10MB default)
    max_file_size: usize,
    /// Maximum text length to extract (1MB default)
    max_text_length: usize,
}

impl PdfProcessorTool {
    /// Create a new PDF processor tool
    #[must_use]
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "pdf-processor".to_string(),
                "Extract text and metadata from PDF documents with security controls".to_string(),
            ),
            max_file_size: 10 * 1024 * 1024, // 10MB
            max_text_length: 1024 * 1024,    // 1MB text
        }
    }

    /// Extract text from PDF file
    async fn extract_pdf_text(&self, file_path: &str) -> Result<String> {
        // Validate file path for security
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(validation_error(
                format!("PDF file not found: {file_path}"),
                Some("input".to_string()),
            ));
        }

        // Check file size
        let metadata = fs::metadata(path).map_err(|e| {
            tool_error(
                format!("Failed to read file metadata: {e}"),
                Some("file_access".to_string()),
            )
        })?;

        if metadata.len() > self.max_file_size as u64 {
            return Err(validation_error(
                format!(
                    "PDF file too large: {} bytes (max: {} bytes)",
                    metadata.len(),
                    self.max_file_size
                ),
                Some("input".to_string()),
            ));
        }

        // Read file content
        let file_content = fs::read(path).map_err(|e| {
            tool_error(
                format!("Failed to read PDF file: {e}"),
                Some("file_read".to_string()),
            )
        })?;

        // Extract text with timeout protection
        let text = with_timeout(std::time::Duration::from_secs(30), async move {
            extract_text_from_mem(&file_content).map_err(|e: PdfError| {
                tool_error(
                    format!("PDF text extraction failed: {e}"),
                    Some("pdf_extraction".to_string()),
                )
            })
        })
        .await
        .map_err(|_| {
            tool_error(
                "PDF extraction timed out after 30 seconds".to_string(),
                Some("timeout".to_string()),
            )
        })??;

        // Limit text length for security
        if text.len() > self.max_text_length {
            Ok(text.chars().take(self.max_text_length).collect::<String>()
                + "\n... [text truncated due to length limit]")
        } else {
            Ok(text)
        }
    }

    /// Get basic PDF metadata
    #[allow(clippy::unused_async)]
    async fn extract_pdf_metadata(&self, file_path: &str) -> Result<JsonValue> {
        let path = Path::new(file_path);
        if !path.exists() {
            return Err(validation_error(
                format!("PDF file not found: {file_path}"),
                Some("input".to_string()),
            ));
        }

        let file_metadata = fs::metadata(path).map_err(|e| {
            tool_error(
                format!("Failed to read file metadata: {e}"),
                Some("file_access".to_string()),
            )
        })?;

        // Extract basic file system metadata
        Ok(json!({
            "file_size": file_metadata.len(),
            "file_name": path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown"),
            "file_extension": path.extension().and_then(|e| e.to_str()).unwrap_or("pdf"),
            "modified": file_metadata.modified().ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
            "note": "Advanced PDF metadata extraction requires additional libraries"
        }))
    }

    /// Extract text from specific pages
    async fn extract_pages(&self, file_path: &str, start_page: Option<u32>) -> Result<String> {
        // For this Phase 7 implementation, we extract all text and note the limitation
        let full_text = self.extract_pdf_text(file_path).await?;

        if let Some(page) = start_page {
            Ok(format!(
                "Page {page} and beyond:\n{full_text}\n\n[Note: Page-specific extraction requires additional PDF parsing library]"
            ))
        } else {
            Ok(full_text)
        }
    }
}

impl Default for PdfProcessorTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseAgent for PdfProcessorTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Create resource tracker for this execution
        let limits = ResourceLimits {
            max_memory_bytes: Some(50 * 1024 * 1024), // 50MB for PDF processing
            max_cpu_time_ms: Some(60_000),            // 60 seconds
            max_operations: Some(1_000),              // 1K operations
            operation_timeout_ms: Some(60_000),       // 60 seconds
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Track the operation
        tracker.track_operation()?;

        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Extract operation type
        let operation = extract_string_with_default(params, "operation", "extract_text");

        // Extract file path
        let file_path = extract_required_string(params, "input")?;

        // Execute based on operation
        let result = match operation {
            "extract_text" => {
                let text = self.extract_pdf_text(file_path).await?;
                json!({
                    "operation": "extract_text",
                    "success": true,
                    "result": {
                        "text": text,
                        "file_path": file_path,
                        "length": text.len()
                    },
                    "metadata": {
                        "tool": "pdf-processor",
                        "operation": "extract_text"
                    }
                })
            }
            "extract_metadata" => {
                let metadata = self.extract_pdf_metadata(file_path).await?;
                json!({
                    "operation": "extract_metadata",
                    "success": true,
                    "result": metadata,
                    "metadata": {
                        "tool": "pdf-processor",
                        "operation": "extract_metadata"
                    }
                })
            }
            "extract_pages" => {
                let start_page =
                    extract_optional_u64(params, "start_page").and_then(|n| u32::try_from(n).ok());
                let text = self.extract_pages(file_path, start_page).await?;
                json!({
                    "operation": "extract_pages",
                    "success": true,
                    "result": {
                        "text": text,
                        "start_page": start_page,
                        "file_path": file_path
                    },
                    "metadata": {
                        "tool": "pdf-processor",
                        "operation": "extract_pages"
                    }
                })
            }
            _ => {
                return Err(validation_error(
                    format!("Unsupported operation: {operation}. Supported: extract_text, extract_metadata, extract_pages"),
                    Some("operation".to_string()),
                ));
            }
        };

        let response = ResponseBuilder::success("pdf_processor_execute")
            .with_result(result)
            .build();
        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        let params = extract_parameters(input)?;

        // Validate required parameters
        extract_required_string(params, "input")?;

        // Validate operation if provided
        let operation = extract_string_with_default(params, "operation", "extract_text");
        match operation {
            "extract_text" | "extract_metadata" | "extract_pages" => {}
            _ => {
                return Err(validation_error(
                    format!("Invalid operation: {operation}"),
                    Some("operation".to_string()),
                ));
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        let error_response = json!({
            "operation": "error",
            "success": false,
            "error": error.to_string(),
            "metadata": {
                "tool": "pdf-processor"
            }
        });

        let response = ResponseBuilder::error("pdf_processor_error", error.to_string())
            .with_result(error_response)
            .build();
        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }
}

#[async_trait]
impl Tool for PdfProcessorTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted // File system access
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "pdf-processor".to_string(),
            "Extract text and metadata from PDF documents with security controls".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform: extract_text, extract_metadata, extract_pages"
                .to_string(),
            required: false,
            default: Some(json!("extract_text")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Path to PDF file to process".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "start_page".to_string(),
            param_type: ParameterType::Number,
            description: "Starting page number for extract_pages operation (1-based)".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Restricted,
            file_permissions: vec!["file_read".to_string()],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: std::collections::HashMap::new(),
        }
    }

    fn resource_limits(&self) -> ToolResourceLimits {
        ToolResourceLimits {
            max_memory_bytes: Some(50 * 1024 * 1024), // 50MB for PDF processing
            max_cpu_time_ms: Some(60_000),            // 60 seconds
            max_network_bps: Some(0),                 // No network needed
            max_file_ops_per_sec: Some(1),            // One file read per second
            custom_limits: std::collections::HashMap::new(),
        }
    }
}

impl ResourceLimited for PdfProcessorTool {}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::create_test_tool_input;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_pdf_processor_creation() {
        let tool = PdfProcessorTool::new();
        assert_eq!(tool.metadata().name, "pdf-processor");
        assert_eq!(tool.category(), ToolCategory::Data);
        assert_eq!(tool.security_level(), SecurityLevel::Restricted);
    }

    #[tokio::test]
    async fn test_pdf_processor_validation() {
        let tool = PdfProcessorTool::new();

        // Test missing input parameter
        let invalid_input = create_test_tool_input(vec![("operation", "extract_text")]);
        let result = tool.validate_input(&invalid_input).await;
        assert!(result.is_err());

        // Test invalid operation
        let invalid_op_input = create_test_tool_input(vec![
            ("input", "/tmp/test.pdf"),
            ("operation", "invalid_op"),
        ]);
        let result = tool.validate_input(&invalid_op_input).await;
        assert!(result.is_err());

        // Test valid input
        let valid_input = create_test_tool_input(vec![
            ("input", "/tmp/test.pdf"),
            ("operation", "extract_text"),
        ]);
        let result = tool.validate_input(&valid_input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_pdf_processor_nonexistent_file() {
        let tool = PdfProcessorTool::new();

        let input = create_test_tool_input(vec![
            ("input", "/nonexistent/file.pdf"),
            ("operation", "extract_text"),
        ]);

        let context = ExecutionContext::default();
        let result = tool.execute(input, context).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_pdf_processor_empty_file() {
        let tool = PdfProcessorTool::new();

        // Create a temporary "PDF" file (will fail extraction but test file handling)
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Not a real PDF").unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        let input =
            create_test_tool_input(vec![("input", temp_path), ("operation", "extract_text")]);

        let context = ExecutionContext::default();
        let result = tool.execute(input, context).await;

        // Should fail because it's not a real PDF
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pdf_processor_schema() {
        let tool = PdfProcessorTool::new();
        let schema = tool.schema();

        assert_eq!(schema.name, "pdf-processor");
        assert!(schema.description.contains("PDF documents"));
        assert!(schema.parameters.len() >= 2); // Should have operation and input parameters

        // Check for required input parameter
        let input_param = schema.parameters.iter().find(|p| p.name == "input");
        assert!(input_param.is_some());
        assert!(input_param.unwrap().required);
    }
}
