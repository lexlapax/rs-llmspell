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
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());

        info!(
            "Starting document processing ({} docs, transformation={}, format={}, parallel={}, model={})",
            document_paths.len(),
            transformation_type,
            output_format,
            parallel_processing,
            model
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
            .transform_content(&extracted_docs, &transformation_type, &model, &context)
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
    /// Phase 1: Extract text from documents in parallel
    async fn extract_parallel(
        &self,
        document_paths: &[String],
        _context: &ExecutionContext,
    ) -> Result<Vec<ExtractedDocument>> {
        // TODO: Implement actual parallel extraction with pdf-reader and ocr tools
        // For now, return placeholder extracted documents
        warn!(
            "Parallel document extraction not yet implemented - using placeholders for {} docs",
            document_paths.len()
        );

        let mut extracted = Vec::new();
        for (idx, path) in document_paths.iter().enumerate() {
            extracted.push(ExtractedDocument {
                source_path: path.clone(),
                extracted_text: format!(
                    "# Extracted Text from Document {}\n\n\
                     Source: {}\n\n\
                     ## Page 1\n\
                     [Placeholder extracted text from page 1]\n\
                     Lorem ipsum dolor sit amet, consectetur adipiscing elit...\n\n\
                     ## Page 2\n\
                     [Placeholder extracted text from page 2]\n\
                     Sed do eiusmod tempor incididunt ut labore...\n\n\
                     Total pages: 2\n",
                    idx + 1,
                    path
                ),
                page_count: 2,
                word_count: 150,
            });
        }

        Ok(extracted)
    }

    /// Phase 1 (alternative): Extract text from documents sequentially
    async fn extract_sequential(
        &self,
        document_paths: &[String],
        context: &ExecutionContext,
    ) -> Result<Vec<ExtractedDocument>> {
        // Sequential extraction is same as parallel in placeholder
        self.extract_parallel(document_paths, context).await
    }

    /// Phase 2: Transform content with transformer agent
    async fn transform_content(
        &self,
        documents: &[ExtractedDocument],
        transformation_type: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<Vec<TransformedDocument>> {
        // TODO: Implement actual agent-based transformation
        // For now, return placeholder transformations
        warn!(
            "Content transformation not yet implemented - using placeholder for type: {}",
            transformation_type
        );

        let mut transformed = Vec::new();
        for doc in documents {
            let content = match transformation_type {
                "summarize" => format!(
                    "# Summary: {}\n\n\
                     ## Executive Summary\n\
                     This document contains {} words across {} pages. \
                     Key points have been extracted and summarized below.\n\n\
                     ## Extracted Content Preview\n\
                     {}\n\n\
                     ## Key Points\n\
                     1. Main topic discussed in section 1\n\
                     2. Important findings from section 2\n\
                     3. Conclusions and recommendations\n\n\
                     ## Source\n\
                     Original document: {}\n",
                    doc.source_path,
                    doc.word_count,
                    doc.page_count,
                    doc.extracted_text
                        .lines()
                        .take(5)
                        .collect::<Vec<_>>()
                        .join("\n"),
                    doc.source_path
                ),
                "extract_key_points" => format!(
                    "# Key Points: {}\n\n\
                     ## Original Content\n\
                     {}\n\n\
                     ## Extracted Key Points\n\
                     - Point 1: [Extracted from page 1]\n\
                     - Point 2: [Extracted from page 1]\n\
                     - Point 3: [Extracted from page 2]\n\
                     - Point 4: [Extracted from page 2]\n\n\
                     Total pages analyzed: {}\n",
                    doc.source_path,
                    doc.extracted_text
                        .lines()
                        .take(3)
                        .collect::<Vec<_>>()
                        .join("\n"),
                    doc.page_count
                ),
                "translate" => format!(
                    "# Translated Content: {}\n\n\
                     [Placeholder translation of extracted text]\n\
                     This would contain the translated version of the {} words \
                     from the original document.\n\n\
                     Source: {}\n",
                    doc.source_path, doc.word_count, doc.source_path
                ),
                "reformat" => format!(
                    "# Reformatted Document: {}\n\n\
                     [Placeholder reformatted version]\n\
                     Content has been restructured and formatted for better readability.\n\n\
                     Original pages: {}, Words: {}\n",
                    doc.source_path, doc.page_count, doc.word_count
                ),
                "classify" => format!(
                    "# Document Classification: {}\n\n\
                     ## Classification Results\n\
                     - Category: Technical Documentation\n\
                     - Confidence: 0.85\n\
                     - Language: English\n\
                     - Content Type: Informational\n\n\
                     Pages: {}, Words: {}\n",
                    doc.source_path, doc.page_count, doc.word_count
                ),
                _ => format!(
                    "# Processed Document: {}\n\n\
                     Transformation type: {}\n\
                     [Placeholder transformed content]\n",
                    doc.source_path, transformation_type
                ),
            };

            transformed.push(TransformedDocument {
                source_path: doc.source_path.clone(),
                transformed_content: content,
            });
        }

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
            .transform_content(&extracted, "summarize", "ollama/llama3.2:3b", &context)
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
}
