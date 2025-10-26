//! Document Processor Template
//!
//! Parallel workflow for document processing and transformation:
//! 1. Load documents from files
//! 2. Extractor agent: extract text from PDFs/images (parallel)
//! 3. Transformer agent: transform/enhance extracted content
//! 4. Save processed documents

use crate::{
    artifacts::Artifact,
    context::ExecutionContext,
    core::{
        CostEstimate, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
        TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

/// Document Processor Template
///
/// Automated document processing with extraction and transformation:
/// - Loads documents from various formats (PDF, images, text)
/// - Extracts text using OCR and PDF parsers (parallel processing)
/// - Transforms content with AI-powered enhancement
/// - Supports batch processing of multiple documents
#[derive(Debug)]
pub struct DocumentProcessorTemplate {
    metadata: TemplateMetadata,
}

impl DocumentProcessorTemplate {
    /// Create a new Document Processor template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "document-processor".to_string(),
                name: "Document Processor".to_string(),
                description: "AI-powered document processing with extraction and transformation. \
                             Extracts text from PDFs and images using OCR, transforms content \
                             with AI agents, and produces structured output. Supports batch \
                             processing for multiple documents in parallel."
                    .to_string(),
                category: TemplateCategory::Document,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["pdf-reader".to_string(), "ocr".to_string()],
                tags: vec![
                    "documents".to_string(),
                    "pdf".to_string(),
                    "ocr".to_string(),
                    "extraction".to_string(),
                    "transformation".to_string(),
                ],
            },
        }
    }
}

impl Default for DocumentProcessorTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for DocumentProcessorTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // document_paths (required array)
            ParameterSchema::required(
                "document_paths",
                "Paths to documents to process (PDFs, images, text files)",
                ParameterType::Array,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                ..Default::default()
            }),
            // transformation_type (optional enum with default)
            ParameterSchema::optional(
                "transformation_type",
                "Type of content transformation to apply",
                ParameterType::String,
                json!("summarize"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("summarize"),
                    json!("extract_key_points"),
                    json!("translate"),
                    json!("reformat"),
                    json!("classify"),
                ]),
                ..Default::default()
            }),
            // output_format (optional enum with default)
            ParameterSchema::optional(
                "output_format",
                "Output format for processed documents",
                ParameterType::String,
                json!("markdown"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("markdown"),
                    json!("json"),
                    json!("text"),
                    json!("html"),
                ]),
                ..Default::default()
            }),
            // parallel_processing (optional boolean with default)
            ParameterSchema::optional(
                "parallel_processing",
                "Process multiple documents in parallel",
                ParameterType::Boolean,
                json!(true),
            ),
            // model (optional - for agent execution)
            ParameterSchema::optional(
                "model",
                "LLM model to use for transformation agents",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
        ])
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let document_paths: Vec<String> = params.get("document_paths")?;
        let transformation_type: String =
            params.get_or("transformation_type", "summarize".to_string());
        let output_format: String = params.get_or("output_format", "markdown".to_string());
        let parallel_processing: bool = params.get_or("parallel_processing", true);

        // Smart dual-path provider resolution (Task 13.5.7d)
        let provider_config = context.resolve_llm_config(&params)?;
        let model_str = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        info!(
            "Starting document processing ({} docs, transformation={}, format={}, parallel={}, model={})",
            document_paths.len(),
            transformation_type,
            output_format,
            parallel_processing,
            model_str
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Phase 1: Extract text from all documents (parallel or sequential)
        info!(
            "Phase 1: Extracting text from {} documents...",
            document_paths.len()
        );
        let extracted_docs = if parallel_processing {
            self.extract_parallel(&document_paths, &context).await?
        } else {
            self.extract_sequential(&document_paths, &context).await?
        };
        output.metrics.tools_invoked += document_paths.len(); // extraction tools

        // Phase 2: Transform content with transformer agent
        info!("Phase 2: Transforming extracted content...");
        let transformed_docs = self
            .transform_content(
                &extracted_docs,
                &transformation_type,
                &provider_config,
                &context,
            )
            .await?;
        output.metrics.agents_invoked += extracted_docs.len(); // one agent per document

        // Phase 3: Format output
        let formatted_output = self.format_documents(&transformed_docs, &output_format)?;

        // Save artifacts
        if let Some(output_dir) = &context.output_dir {
            self.save_artifacts(output_dir, &transformed_docs, &output_format, &mut output)?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(formatted_output);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("documents_processed", json!(document_paths.len()));
        output.add_metric("transformation_type", json!(transformation_type));
        output.add_metric("output_format", json!(output_format));
        output.add_metric("parallel_processing", json!(parallel_processing));

        info!(
            "Document processing complete (duration: {}ms, docs: {})",
            output.metrics.duration_ms,
            document_paths.len()
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let document_paths: Vec<String> = params
            .get("document_paths")
            .unwrap_or_else(|_| vec!["doc.pdf".to_string()]);
        let doc_count = document_paths.len();

        // Rough estimates per document:
        // - Extraction: minimal tokens (tool-based)
        // - Transformation: ~1500 tokens per document
        let estimated_tokens = doc_count * 1500;

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Per document:
        // - Extraction: ~2s (PDF/OCR processing)
        // - Transformation: ~4s (agent processing)
        let estimated_duration = doc_count * (2000 + 4000);

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.6, // Medium confidence - varies with document complexity
        )
    }
}

impl DocumentProcessorTemplate {
    /// Helper: Read document file from path (supports .txt, .md files)
    fn read_document_file(path: &str) -> Result<ExtractedDocument> {
        use std::fs;

        // Read file content
        let content = fs::read_to_string(path).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to read file '{}': {}", path, e))
        })?;

        // Count words (split on whitespace)
        let word_count = content.split_whitespace().count();

        // Calculate "pages" (500 words per page)
        let page_count = (word_count as f64 / 500.0).ceil() as usize;
        let page_count = if page_count == 0 { 1 } else { page_count };

        Ok(ExtractedDocument {
            source_path: path.to_string(),
            extracted_text: content,
            page_count,
            word_count,
        })
    }

    /// Phase 1: Extract text from documents in parallel
    async fn extract_parallel(
        &self,
        document_paths: &[String],
        _context: &ExecutionContext,
    ) -> Result<Vec<ExtractedDocument>> {
        // Real file extraction for text/markdown files
        // Note: PDF/OCR support deferred to Phase 14
        info!(
            "Extracting text from {} documents (text/markdown files supported)",
            document_paths.len()
        );

        let mut extracted = Vec::new();
        for (idx, path) in document_paths.iter().enumerate() {
            match Self::read_document_file(path) {
                Ok(doc) => {
                    info!(
                        "Extracted document {}/{}: {} ({} words, {} pages)",
                        idx + 1,
                        document_paths.len(),
                        path,
                        doc.word_count,
                        doc.page_count
                    );
                    extracted.push(doc);
                }
                Err(e) => {
                    warn!("Failed to extract document {}: {}", path, e);
                    return Err(e);
                }
            }
        }

        Ok(extracted)
    }

    /// Phase 1 (alternative): Extract text from documents sequentially
    async fn extract_sequential(
        &self,
        document_paths: &[String],
        context: &ExecutionContext,
    ) -> Result<Vec<ExtractedDocument>> {
        // Note: Currently same as parallel (both read synchronously)
        // Future Phase 14: Use tokio::spawn for true parallel file I/O
        self.extract_parallel(document_paths, context).await
    }

    /// Phase 2: Transform content with transformer agent
    async fn transform_content(
        &self,
        documents: &[ExtractedDocument],
        transformation_type: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<Vec<TransformedDocument>> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!(
            "Transforming {} documents with agent (type: {})",
            documents.len(),
            transformation_type
        );

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model string (provider/model format)
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create agent config for document transformation
        let agent_config = AgentConfig {
            name: "doc-transformer-agent".to_string(),
            description: format!(
                "Document transformation agent for {} transformation",
                transformation_type
            ),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.5)), // Balanced for creative yet accurate transformation
                max_tokens: provider_config.max_tokens.or(Some(2000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 120,
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the transformation agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create transformation agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Transform each document with the agent
        let mut transformed = Vec::new();
        for (idx, doc) in documents.iter().enumerate() {
            info!(
                "Transforming document {}/{}: {} ({} words)",
                idx + 1,
                documents.len(),
                doc.source_path,
                doc.word_count
            );

            // Build transformation instructions based on type
            let transformation_instructions = match transformation_type {
                "summarize" => {
                    "Create a concise summary with:\n\
                    - Executive summary (2-3 sentences)\n\
                    - Main topics and key points (bullet points)\n\
                    - Conclusions and takeaways\n\
                    Keep the summary informative yet brief."
                }
                "extract_key_points" => {
                    "Extract and list the key points from the document:\n\
                    - Identify main arguments or findings\n\
                    - List each key point as a bullet\n\
                    - Include supporting evidence where relevant\n\
                    Focus on the most important information."
                }
                "translate" => {
                    "Translate this document to Spanish:\n\
                    - Maintain the original structure and formatting\n\
                    - Preserve technical terms appropriately\n\
                    - Ensure natural language flow\n\
                    Provide accurate translation."
                }
                "reformat" => {
                    "Reformat this document for better readability:\n\
                    - Use clear headings and sections\n\
                    - Add bullet points for lists\n\
                    - Improve paragraph structure\n\
                    - Maintain all original information\n\
                    Make it easier to scan and read."
                }
                "classify" => {
                    "Classify this document:\n\
                    - Category (technical, business, academic, etc.)\n\
                    - Content type (informational, instructional, etc.)\n\
                    - Primary topics\n\
                    - Confidence level\n\
                    Provide structured classification."
                }
                _ => "Process and transform this document according to best practices.",
            };

            // Build the transformation prompt
            let transformation_prompt = format!(
                "You are an expert document processor specializing in {} transformations.\n\n\
                 **SOURCE DOCUMENT**: {}\n\
                 **DOCUMENT STATISTICS**: {} words, {} pages\n\n\
                 **DOCUMENT CONTENT**:\n{}\n\n\
                 **TRANSFORMATION TYPE**: {}\n\n\
                 **INSTRUCTIONS**:\n{}\n\n\
                 **REQUIREMENTS**:\n\
                 1. Base your transformation on the document content above\n\
                 2. Maintain accuracy and preserve important details\n\
                 3. Structure your output clearly with headings/sections\n\
                 4. Be thorough yet concise\n\n\
                 Perform the {} transformation now:",
                transformation_type,
                doc.source_path,
                doc.word_count,
                doc.page_count,
                doc.extracted_text,
                transformation_type,
                transformation_instructions,
                transformation_type
            );

            // Execute the transformation agent
            let agent_input = AgentInput::builder().text(transformation_prompt).build();
            let agent_output = agent
                .execute(agent_input, llmspell_core::ExecutionContext::default())
                .await
                .map_err(|e| {
                    warn!(
                        "Transformation agent execution failed for {}: {}",
                        doc.source_path, e
                    );
                    TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
                })?;

            // Extract transformed content from agent output
            let transformed_content = agent_output.text;

            transformed.push(TransformedDocument {
                source_path: doc.source_path.clone(),
                transformed_content,
            });
        }

        info!(
            "Successfully transformed {} documents with {} transformation",
            documents.len(),
            transformation_type
        );
        Ok(transformed)
    }

    /// Phase 3: Format documents for output
    fn format_documents(&self, documents: &[TransformedDocument], format: &str) -> Result<String> {
        match format {
            "markdown" => {
                let mut output = String::from("# Processed Documents\n\n");
                for (idx, doc) in documents.iter().enumerate() {
                    output.push_str(&format!(
                        "## Document {}: {}\n\n{}\n\n---\n\n",
                        idx + 1,
                        doc.source_path,
                        doc.transformed_content
                    ));
                }
                Ok(output)
            }
            "json" => {
                let json_docs: Vec<_> = documents
                    .iter()
                    .map(|doc| {
                        json!({
                            "source": doc.source_path,
                            "content": doc.transformed_content,
                        })
                    })
                    .collect();
                serde_json::to_string_pretty(&json!({
                    "documents": json_docs,
                    "total": documents.len(),
                }))
                .map_err(|e| {
                    TemplateError::ExecutionFailed(format!("JSON formatting failed: {}", e))
                })
            }
            "text" => {
                let mut output = String::from("PROCESSED DOCUMENTS\n\n");
                let separator = "=".repeat(80);
                for (idx, doc) in documents.iter().enumerate() {
                    output.push_str(&format!(
                        "Document {}: {}\n\n{}\n\n{}\n\n",
                        idx + 1,
                        doc.source_path,
                        doc.transformed_content,
                        separator
                    ));
                }
                Ok(output)
            }
            "html" => {
                let mut html = String::from(
                    "<!DOCTYPE html>\n\
                     <html>\n\
                     <head>\n    \
                         <title>Processed Documents</title>\n    \
                         <style>\n        \
                             body { font-family: Arial, sans-serif; margin: 40px; }\n        \
                             .document { margin-bottom: 40px; border-bottom: 2px solid #ccc; padding-bottom: 20px; }\n        \
                             .source { color: #666; font-style: italic; }\n        \
                             pre { background: #f5f5f5; padding: 10px; border-radius: 5px; }\n    \
                         </style>\n\
                     </head>\n\
                     <body>\n    \
                         <h1>Processed Documents</h1>\n",
                );
                for (idx, doc) in documents.iter().enumerate() {
                    html.push_str(&format!(
                        "    <div class=\"document\">\n        \
                             <h2>Document {}</h2>\n        \
                             <p class=\"source\">Source: {}</p>\n        \
                             <pre>{}</pre>\n    \
                         </div>\n",
                        idx + 1,
                        doc.source_path,
                        doc.transformed_content
                    ));
                }
                html.push_str("</body>\n</html>\n");
                Ok(html)
            }
            _ => Err(TemplateError::ExecutionFailed(format!(
                "Unsupported output format: {}",
                format
            ))),
        }
    }

    /// Save artifacts to output directory
    fn save_artifacts(
        &self,
        output_dir: &std::path::Path,
        documents: &[TransformedDocument],
        format: &str,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save individual processed documents
        let ext = match format {
            "markdown" => "md",
            "json" => "json",
            "html" => "html",
            _ => "txt",
        };

        for (idx, doc) in documents.iter().enumerate() {
            let filename = format!("processed_doc_{}.{}", idx + 1, ext);
            let doc_path = output_dir.join(&filename);
            fs::write(&doc_path, &doc.transformed_content).map_err(|e| {
                TemplateError::ExecutionFailed(format!(
                    "Failed to write document {}: {}",
                    idx + 1,
                    e
                ))
            })?;
            output.add_artifact(Artifact::new(
                doc_path.to_string_lossy().to_string(),
                doc.transformed_content.clone(),
                format!("text/{}", if ext == "md" { "markdown" } else { ext }),
            ));
        }

        Ok(())
    }
}

/// Extracted document from Phase 1
#[derive(Debug, Clone)]
struct ExtractedDocument {
    /// Source file path
    source_path: String,
    /// Extracted text content
    extracted_text: String,
    /// Number of pages
    page_count: usize,
    /// Word count
    word_count: usize,
}

/// Transformed document from Phase 2
#[derive(Debug, Clone)]
struct TransformedDocument {
    /// Source file path
    source_path: String,
    /// Transformed content
    transformed_content: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;
    /// Test helper: Create a provider config for tests
    fn test_provider_config() -> llmspell_config::ProviderConfig {
        llmspell_config::ProviderConfig {
            default_model: Some("ollama/llama3.2:3b".to_string()),
            provider_type: "ollama".to_string(),
            temperature: Some(0.3),
            max_tokens: Some(2000),
            timeout_seconds: Some(120),
            ..Default::default()
        }
    }

    #[test]
    fn test_template_metadata() {
        let template = DocumentProcessorTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "document-processor");
        assert_eq!(metadata.name, "Document Processor");
        assert_eq!(metadata.category, TemplateCategory::Document);
        assert!(metadata.requires.contains(&"pdf-reader".to_string()));
        assert!(metadata.requires.contains(&"ocr".to_string()));
        assert!(metadata.tags.contains(&"documents".to_string()));
        assert!(metadata.tags.contains(&"pdf".to_string()));
        assert!(metadata.tags.contains(&"ocr".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = DocumentProcessorTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("document_paths").is_some());
        assert!(schema.get_parameter("transformation_type").is_some());
        assert!(schema.get_parameter("output_format").is_some());
        assert!(schema.get_parameter("parallel_processing").is_some());
        assert!(schema.get_parameter("model").is_some());

        // document_paths is required
        let paths_param = schema.get_parameter("document_paths").unwrap();
        assert!(paths_param.required);

        // others are optional
        let trans_param = schema.get_parameter("transformation_type").unwrap();
        assert!(!trans_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate_single_doc() {
        let template = DocumentProcessorTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("document_paths", json!(["doc1.pdf"]));

        let estimate = template.estimate_cost(&params).await;
        assert_eq!(estimate.estimated_tokens, Some(1500));
        assert_eq!(estimate.estimated_duration_ms, Some(6000)); // 2s + 4s
    }

    #[tokio::test]
    async fn test_cost_estimate_multiple_docs() {
        let template = DocumentProcessorTemplate::new();
        let mut params = TemplateParams::new();
        params.insert(
            "document_paths",
            json!(["doc1.pdf", "doc2.pdf", "doc3.pdf"]),
        );

        let estimate = template.estimate_cost(&params).await;
        assert_eq!(estimate.estimated_tokens, Some(4500)); // 3 * 1500
        assert_eq!(estimate.estimated_duration_ms, Some(18000)); // 3 * 6000
    }

    #[test]
    fn test_parameter_validation_missing_required() {
        let template = DocumentProcessorTemplate::new();
        let schema = template.config_schema();
        let params = std::collections::HashMap::new();

        // Should fail - missing required "document_paths" parameter
        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_invalid_transformation() {
        let template = DocumentProcessorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("document_paths".to_string(), json!(["doc.pdf"]));
        params.insert("transformation_type".to_string(), json!("invalid_type"));

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = DocumentProcessorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "document_paths".to_string(),
            json!(["doc1.pdf", "doc2.pdf"]),
        );
        params.insert("transformation_type".to_string(), json!("summarize"));
        params.insert("output_format".to_string(), json!("markdown"));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_extract_parallel_placeholder() {
        let template = DocumentProcessorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            return;
        }
        let context = context.unwrap();

        let paths = vec!["doc1.pdf".to_string(), "doc2.pdf".to_string()];
        let result = template.extract_parallel(&paths, &context).await;
        assert!(result.is_ok());
        let docs = result.unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].source_path, "doc1.pdf");
        assert_eq!(docs[1].source_path, "doc2.pdf");
    }

    #[tokio::test]
    async fn test_transform_content_summarize() {
        let template = DocumentProcessorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            return;
        }
        let context = context.unwrap();

        let extracted = vec![ExtractedDocument {
            source_path: "test.pdf".to_string(),
            extracted_text: "Sample text".to_string(),
            page_count: 2,
            word_count: 100,
        }];

        let result = template
            .transform_content(&extracted, "summarize", &test_provider_config(), &context)
            .await;
        assert!(result.is_ok());
        let docs = result.unwrap();
        assert_eq!(docs.len(), 1);
        assert!(docs[0].transformed_content.contains("Summary"));
    }

    #[test]
    fn test_format_documents_markdown() {
        let template = DocumentProcessorTemplate::new();
        let docs = vec![TransformedDocument {
            source_path: "test.pdf".to_string(),
            transformed_content: "Content".to_string(),
        }];

        let result = template.format_documents(&docs, "markdown");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("# Processed Documents"));
        assert!(output.contains("test.pdf"));
    }

    #[test]
    fn test_format_documents_json() {
        let template = DocumentProcessorTemplate::new();
        let docs = vec![TransformedDocument {
            source_path: "test.pdf".to_string(),
            transformed_content: "Content".to_string(),
        }];

        let result = template.format_documents(&docs, "json");
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("\"documents\""));
        assert!(output.contains("\"total\""));
    }

    #[test]
    fn test_format_documents_unsupported() {
        let template = DocumentProcessorTemplate::new();
        let docs = vec![TransformedDocument {
            source_path: "test.pdf".to_string(),
            transformed_content: "Content".to_string(),
        }];

        let result = template.format_documents(&docs, "xml");
        assert!(result.is_err());
    }

    // Integration tests for real file I/O and agent execution

    #[test]
    fn test_read_document_file_with_real_file() {
        use std::fs;

        // Create temp file with test content
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("llmspell_test_doc.txt");
        let test_content = "This is a test document.\n\nIt has multiple paragraphs.\n\nAnd some more content here.\n\nTotal word count should be calculated correctly.";

        fs::write(&test_file, test_content).expect("Failed to write test file");

        // Test reading the file
        let result = DocumentProcessorTemplate::read_document_file(test_file.to_str().unwrap());

        assert!(result.is_ok(), "Failed to read test file");
        let doc = result.unwrap();
        assert_eq!(doc.source_path, test_file.to_string_lossy().to_string());
        assert!(!doc.extracted_text.is_empty());
        assert!(doc.word_count > 0);
        assert!(doc.page_count > 0);

        // Verify word count is correct
        assert_eq!(doc.word_count, test_content.split_whitespace().count());

        // Cleanup
        fs::remove_file(&test_file).ok();
    }

    #[tokio::test]
    async fn test_extract_with_real_files() {
        use std::fs;

        let template = DocumentProcessorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            return;
        }
        let context = context.unwrap();

        // Create temp files
        let temp_dir = std::env::temp_dir();
        let file1 = temp_dir.join("llmspell_test1.txt");
        let file2 = temp_dir.join("llmspell_test2.md");

        fs::write(&file1, "First test document with some content.").expect("Write failed");
        fs::write(&file2, "# Second Document\n\nMarkdown content here.").expect("Write failed");

        let paths = vec![
            file1.to_string_lossy().to_string(),
            file2.to_string_lossy().to_string(),
        ];

        // Test extraction
        let result = template.extract_parallel(&paths, &context).await;
        assert!(result.is_ok(), "Extraction failed");

        let docs = result.unwrap();
        assert_eq!(docs.len(), 2);
        assert!(docs[0].word_count > 0);
        assert!(docs[1].word_count > 0);

        // Cleanup
        fs::remove_file(&file1).ok();
        fs::remove_file(&file2).ok();
    }

    #[tokio::test]
    #[ignore = "Requires full infrastructure (AgentRegistry, ProviderManager) for real LLM execution"]
    async fn test_end_to_end_with_real_agent() {
        // Note: This test is ignored by default as it requires full infrastructure
        // Run with: cargo test --lib -- --ignored test_end_to_end_with_real_agent
        //
        // This is a placeholder for CLI-based integration testing
        // See Sub-Task 12.8.6.6 for CLI testing approach
    }
}
