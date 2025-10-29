//! Knowledge Management Template - RAG-Centric Workflow
//!
//! Provides personal knowledge management with ingest-query-synthesize pipeline.
//! Implements CRUD operations for knowledge bases with multi-collection support
//! and citation tracking.

use crate::context::ExecutionContext;
use crate::core::{
    memory_parameters, provider_parameters, CostEstimate, TemplateCategory, TemplateMetadata,
    TemplateOutput, TemplateParams, TemplateResult,
};
use crate::error::{TemplateError, ValidationError};
use crate::validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType};
use async_trait::async_trait;
use llmspell_core::state::StateScope;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};

/// Knowledge document with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeDocument {
    pub id: String,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub chunks: Vec<String>,
}

/// Document metadata for citation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub title: Option<String>,
    pub source_type: String,
    pub source_path: Option<String>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub timestamp: String,
}

/// Query result with citation
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResult {
    document_id: String,
    chunk: String,
    relevance_score: f32,
    metadata: DocumentMetadata,
}

/// Knowledge Management Template
#[derive(Debug)]
pub struct KnowledgeManagementTemplate {
    metadata: TemplateMetadata,
}

impl Default for KnowledgeManagementTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl KnowledgeManagementTemplate {
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "knowledge-management".to_string(),
                name: "Knowledge Management".to_string(),
                description: "RAG-powered knowledge management with ingest-query-synthesize pipeline. Supports multi-collection storage, CRUD operations, and citation tracking for personal knowledge bases.".to_string(),
                category: TemplateCategory::Research,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec![],
                tags: vec![
                    "rag".to_string(),
                    "knowledge-base".to_string(),
                    "semantic-search".to_string(),
                    "research".to_string(),
                    "learning".to_string(),
                ],
            },
        }
    }

    /// Chunk document content into manageable pieces
    fn chunk_document(&self, content: &str, chunk_size: usize, overlap: usize) -> Vec<String> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();

        let mut i = 0;
        while i < words.len() {
            let end = (i + chunk_size).min(words.len());
            let chunk = words[i..end].join(" ");
            chunks.push(chunk);

            if end >= words.len() {
                break;
            }

            i += chunk_size - overlap;
        }

        if chunks.is_empty() {
            chunks.push(content.to_string());
        }

        chunks
    }

    /// Parse source content based on type
    fn parse_content(&self, content: &str, source_type: &str) -> Result<String, TemplateError> {
        match source_type {
            "text" => Ok(content.to_string()),
            "markdown" => {
                // Simple markdown parsing - extract text
                let mut text = content.to_string();
                // Remove markdown headers
                text = text.replace('#', "");
                // Remove code blocks
                text = text.replace("```", "");
                Ok(text)
            }
            "file" => {
                // Read from file path
                std::fs::read_to_string(content).map_err(|e| {
                    TemplateError::ExecutionFailed(format!("Failed to read file: {}", e))
                })
            }
            _ => Ok(content.to_string()),
        }
    }

    /// Generate document ID from content
    fn generate_doc_id(&self, content: &str, collection: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        collection.hash(&mut hasher);
        format!("doc-{:x}", hasher.finish())
    }

    /// Extract tags from content (simple keyword extraction)
    fn extract_tags(&self, content: &str) -> Vec<String> {
        let content_lower = content.to_lowercase();
        let mut tags = Vec::new();

        // Common technical tags
        let tag_keywords = [
            "rust",
            "python",
            "javascript",
            "ai",
            "machine-learning",
            "llm",
            "rag",
            "documentation",
            "api",
            "database",
            "architecture",
            "design",
            "testing",
            "security",
            "performance",
            "optimization",
        ];

        for keyword in &tag_keywords {
            if content_lower.contains(keyword) {
                tags.push(keyword.to_string());
            }
        }

        // Limit to top 5 tags
        tags.truncate(5);
        tags
    }

    /// Perform simple similarity search (mock RAG)
    fn simple_search(
        &self,
        query: &str,
        documents: &[KnowledgeDocument],
        max_results: usize,
    ) -> Vec<QueryResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for doc in documents {
            for (chunk_idx, chunk) in doc.chunks.iter().enumerate() {
                let chunk_lower = chunk.to_lowercase();

                // Simple word overlap scoring
                let query_words: Vec<&str> = query_lower.split_whitespace().collect();
                let chunk_words: Vec<&str> = chunk_lower.split_whitespace().collect();

                let mut matches = 0;
                for word in &query_words {
                    if chunk_words.contains(word) {
                        matches += 1;
                    }
                }

                if matches > 0 {
                    let score = matches as f32 / query_words.len() as f32;
                    results.push(QueryResult {
                        document_id: format!("{}-chunk-{}", doc.id, chunk_idx),
                        chunk: chunk.clone(),
                        relevance_score: score,
                        metadata: doc.metadata.clone(),
                    });
                }
            }
        }

        // Sort by relevance score
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        // Return top-k results
        results.truncate(max_results);
        results
    }

    /// Format query results as text
    fn format_query_results(
        &self,
        query: &str,
        results: &[QueryResult],
        include_citations: bool,
    ) -> String {
        let mut output = String::from("=== KNOWLEDGE QUERY RESULTS ===\n\n");
        output.push_str(&format!("Query: \"{}\"\n", query));
        output.push_str(&format!("Results: {} matches found\n\n", results.len()));

        if results.is_empty() {
            output.push_str("No relevant knowledge found.\n");
            return output;
        }

        for (idx, result) in results.iter().enumerate() {
            output.push_str(&format!(
                "Result {}: (relevance: {:.2})\n",
                idx + 1,
                result.relevance_score
            ));
            output.push_str(&format!("  {}\n\n", result.chunk));

            if include_citations {
                output.push_str("  Citations:\n");
                output.push_str(&format!("    Document ID: {}\n", result.document_id));
                if let Some(title) = &result.metadata.title {
                    output.push_str(&format!("    Title: {}\n", title));
                }
                output.push_str(&format!(
                    "    Source Type: {}\n",
                    result.metadata.source_type
                ));
                if !result.metadata.tags.is_empty() {
                    output.push_str(&format!("    Tags: {}\n", result.metadata.tags.join(", ")));
                }
                output.push_str(&format!("    Timestamp: {}\n", result.metadata.timestamp));
                output.push('\n');
            }
        }

        output
    }

    /// Format as JSON
    fn format_json_output(&self, data: &serde_json::Value) -> Result<String, TemplateError> {
        serde_json::to_string_pretty(data).map_err(|e| {
            TemplateError::ExecutionFailed(format!("JSON serialization failed: {}", e))
        })
    }
}

#[async_trait]
impl crate::core::Template for KnowledgeManagementTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        let mut params = vec![
            // operation (required enum)
            ParameterSchema::required(
                "operation",
                "Knowledge base operation",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("ingest"),
                    json!("query"),
                    json!("update"),
                    json!("delete"),
                    json!("list"),
                ]),
                ..Default::default()
            }),
            // collection (required)
            ParameterSchema::required(
                "collection",
                "Collection name for knowledge storage",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                max_length: Some(100),
                ..Default::default()
            }),
            // content (optional - for ingest/update)
            ParameterSchema::optional(
                "content",
                "Content to ingest or file path",
                ParameterType::String,
                json!(null),
            ),
            // query (optional - for query operation)
            ParameterSchema::optional(
                "query",
                "Search query for knowledge retrieval",
                ParameterType::String,
                json!(null),
            ),
            // document_id (optional - for update/delete)
            ParameterSchema::optional(
                "document_id",
                "Document identifier for update/delete",
                ParameterType::String,
                json!(null),
            ),
            // source_type (optional enum)
            ParameterSchema::optional(
                "source_type",
                "Content source type",
                ParameterType::String,
                json!("text"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("text"), json!("markdown"), json!("file")]),
                ..Default::default()
            }),
            // max_results (optional)
            ParameterSchema::optional(
                "max_results",
                "Maximum query results",
                ParameterType::Integer,
                json!(5),
            )
            .with_constraints(ParameterConstraints {
                min: Some(1.0),
                max: Some(50.0),
                ..Default::default()
            }),
            // include_citations (optional bool)
            ParameterSchema::optional(
                "include_citations",
                "Include source citations in results",
                ParameterType::Boolean,
                json!(true),
            ),
            // chunk_size (optional)
            ParameterSchema::optional(
                "chunk_size",
                "Words per chunk for document splitting",
                ParameterType::Integer,
                json!(200),
            )
            .with_constraints(ParameterConstraints {
                min: Some(50.0),
                max: Some(1000.0),
                ..Default::default()
            }),
            // chunk_overlap (optional)
            ParameterSchema::optional(
                "chunk_overlap",
                "Word overlap between chunks",
                ParameterType::Integer,
                json!(50),
            ),
            // output_format (optional)
            ParameterSchema::optional(
                "output_format",
                "Output format",
                ParameterType::String,
                json!("text"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("text"), json!("json")]),
                ..Default::default()
            }),
        ];

        // Add memory parameters (Task 13.11.1)
        params.extend(memory_parameters());

        // Add provider parameters (Task 13.5.7d)
        params.extend(provider_parameters());

        tracing::debug!(
            "KnowledgeManagement: Generated config schema with {} parameters",
            params.len()
        );
        ConfigSchema::new(params)
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput, TemplateError> {
        let start_time = std::time::Instant::now();
        let mut output = TemplateOutput::new(
            TemplateResult::text(""),
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params.clone(),
        );

        // Extract parameters
        let operation: String = params.get("operation")?;
        let collection: String = params.get("collection")?;
        let content: Option<String> = params.get_optional("content");
        let query: Option<String> = params.get_optional("query");
        let document_id: Option<String> = params.get_optional("document_id");
        let source_type: String = params.get_or("source_type", "text".to_string());
        let max_results: usize = params.get_or("max_results", 5);
        let include_citations: bool = params.get_or("include_citations", true);
        let chunk_size: usize = params.get_or("chunk_size", 200);
        let chunk_overlap: usize = params.get_or("chunk_overlap", 50);
        let output_format: String = params.get_or("output_format", "text".to_string());

        info!(
            "Executing knowledge management operation: {} on collection: {}",
            operation, collection
        );

        // Get state manager for persistent storage
        let state_manager = context.state_manager().ok_or_else(|| {
            TemplateError::InfrastructureUnavailable(
                "StateManager required for knowledge management".to_string(),
            )
        })?;

        // Collection key in state
        let collection_key = format!("knowledge:collections:{}", collection);

        // Execute operation
        let result_text = match operation.as_str() {
            "ingest" => {
                // Validate content parameter
                let content = content.ok_or_else(|| ValidationError::missing("content"))?;

                info!("Ingesting content into collection: {}", collection);

                // Parse content based on source type
                let parsed_content = self.parse_content(&content, &source_type)?;

                // Generate document ID
                let doc_id = self.generate_doc_id(&parsed_content, &collection);

                // Chunk the document
                let chunks = self.chunk_document(&parsed_content, chunk_size, chunk_overlap);

                // Extract metadata
                let tags = self.extract_tags(&parsed_content);
                let metadata = DocumentMetadata {
                    title: None, // Could extract from first line
                    source_type: source_type.clone(),
                    source_path: if source_type == "file" {
                        Some(content.clone())
                    } else {
                        None
                    },
                    category: None,
                    tags,
                    timestamp: chrono::Utc::now().to_rfc3339(),
                };

                // Create document
                let document = KnowledgeDocument {
                    id: doc_id.clone(),
                    content: parsed_content,
                    metadata,
                    chunks,
                };

                // Store in state manager
                let mut documents: Vec<KnowledgeDocument> = state_manager
                    .get(StateScope::Global, &collection_key)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                documents.push(document.clone());

                state_manager
                    .set(
                        StateScope::Global,
                        &collection_key,
                        serde_json::to_value(&documents).unwrap(),
                    )
                    .await?;

                info!(
                    "Document ingested: {} ({} chunks)",
                    doc_id,
                    document.chunks.len()
                );

                output.add_metric("document_id", json!(doc_id));
                output.add_metric("chunks_created", json!(document.chunks.len()));
                output.add_metric("tags_extracted", json!(document.metadata.tags.len()));

                format!(
                    "✅ Document ingested successfully\n\nDocument ID: {}\nChunks: {}\nTags: {}\nCollection: {}",
                    doc_id,
                    document.chunks.len(),
                    document.metadata.tags.join(", "),
                    collection
                )
            }

            "query" => {
                // Validate query parameter
                let query = query.ok_or_else(|| ValidationError::missing("query"))?;

                info!("Querying collection: {} with query: {}", collection, query);

                // Retrieve documents from state
                let documents: Vec<KnowledgeDocument> = state_manager
                    .get(StateScope::Global, &collection_key)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                if documents.is_empty() {
                    warn!("Collection {} is empty", collection);
                    return Err(ValidationError::invalid_value(
                        "collection",
                        format!("Collection '{}' is empty or does not exist", collection),
                    )
                    .into());
                }

                // Perform search
                let results = self.simple_search(&query, &documents, max_results);

                info!("Found {} results for query", results.len());

                output.add_metric("results_found", json!(results.len()));
                output.add_metric("documents_searched", json!(documents.len()));

                if output_format == "json" {
                    let json_output = json!({
                        "query": query,
                        "collection": collection,
                        "results": results,
                        "total_documents": documents.len(),
                        "max_results": max_results,
                    });
                    self.format_json_output(&json_output)?
                } else {
                    self.format_query_results(&query, &results, include_citations)
                }
            }

            "update" => {
                // Validate parameters
                let document_id =
                    document_id.ok_or_else(|| ValidationError::missing("document_id"))?;
                let content = content.ok_or_else(|| ValidationError::missing("content"))?;

                info!(
                    "Updating document: {} in collection: {}",
                    document_id, collection
                );

                // Retrieve documents
                let mut documents: Vec<KnowledgeDocument> = state_manager
                    .get(StateScope::Global, &collection_key)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                // Find and update document
                let doc_index = documents
                    .iter()
                    .position(|d| d.id == document_id)
                    .ok_or_else(|| {
                        ValidationError::invalid_value(
                            "document_id",
                            format!("Document '{}' not found in collection", document_id),
                        )
                    })?;

                // Parse and update content
                let parsed_content = self.parse_content(&content, &source_type)?;
                let chunks = self.chunk_document(&parsed_content, chunk_size, chunk_overlap);
                let tags = self.extract_tags(&parsed_content);

                documents[doc_index].content = parsed_content;
                documents[doc_index].chunks = chunks;
                documents[doc_index].metadata.tags = tags;
                documents[doc_index].metadata.timestamp = chrono::Utc::now().to_rfc3339();

                // Save updated documents
                state_manager
                    .set(
                        StateScope::Global,
                        &collection_key,
                        serde_json::to_value(&documents).unwrap(),
                    )
                    .await?;

                info!("Document updated: {}", document_id);

                output.add_metric("document_id", json!(document_id));
                output.add_metric("chunks_updated", json!(documents[doc_index].chunks.len()));

                format!(
                    "✅ Document updated successfully\n\nDocument ID: {}\nNew chunks: {}\nCollection: {}",
                    document_id,
                    documents[doc_index].chunks.len(),
                    collection
                )
            }

            "delete" => {
                // Validate document_id parameter
                let document_id =
                    document_id.ok_or_else(|| ValidationError::missing("document_id"))?;

                info!(
                    "Deleting document: {} from collection: {}",
                    document_id, collection
                );

                // Retrieve documents
                let mut documents: Vec<KnowledgeDocument> = state_manager
                    .get(StateScope::Global, &collection_key)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                // Find and remove document
                let doc_index = documents
                    .iter()
                    .position(|d| d.id == document_id)
                    .ok_or_else(|| {
                        ValidationError::invalid_value(
                            "document_id",
                            format!("Document '{}' not found in collection", document_id),
                        )
                    })?;

                documents.remove(doc_index);

                // Save updated documents
                state_manager
                    .set(
                        StateScope::Global,
                        &collection_key,
                        serde_json::to_value(&documents).unwrap(),
                    )
                    .await?;

                info!("Document deleted: {}", document_id);

                output.add_metric("document_id", json!(document_id));
                output.add_metric("remaining_documents", json!(documents.len()));

                format!(
                    "✅ Document deleted successfully\n\nDocument ID: {}\nRemaining documents: {}\nCollection: {}",
                    document_id,
                    documents.len(),
                    collection
                )
            }

            "list" => {
                info!("Listing documents in collection: {}", collection);

                // Retrieve documents
                let documents: Vec<KnowledgeDocument> = state_manager
                    .get(StateScope::Global, &collection_key)
                    .await
                    .ok()
                    .flatten()
                    .and_then(|v: serde_json::Value| serde_json::from_value(v).ok())
                    .unwrap_or_default();

                output.add_metric("total_documents", json!(documents.len()));

                if output_format == "json" {
                    let json_output = json!({
                        "collection": collection,
                        "total_documents": documents.len(),
                        "documents": documents.iter().map(|d| {
                            json!({
                                "id": d.id,
                                "chunks": d.chunks.len(),
                                "tags": d.metadata.tags,
                                "source_type": d.metadata.source_type,
                                "timestamp": d.metadata.timestamp,
                                "preview": d.content.chars().take(100).collect::<String>() + "...",
                            })
                        }).collect::<Vec<_>>(),
                    });
                    self.format_json_output(&json_output)?
                } else {
                    let mut output_text = format!("=== KNOWLEDGE BASE: {} ===\n\n", collection);
                    output_text.push_str(&format!("Total Documents: {}\n\n", documents.len()));

                    if documents.is_empty() {
                        output_text.push_str("Collection is empty.\n");
                    } else {
                        for (idx, doc) in documents.iter().enumerate() {
                            output_text.push_str(&format!("Document {}:\n", idx + 1));
                            output_text.push_str(&format!("  ID: {}\n", doc.id));
                            output_text.push_str(&format!("  Chunks: {}\n", doc.chunks.len()));
                            output_text
                                .push_str(&format!("  Tags: {}\n", doc.metadata.tags.join(", ")));
                            output_text
                                .push_str(&format!("  Source: {}\n", doc.metadata.source_type));
                            output_text
                                .push_str(&format!("  Timestamp: {}\n", doc.metadata.timestamp));
                            let preview: String = doc.content.chars().take(100).collect();
                            output_text.push_str(&format!("  Preview: {}...\n\n", preview));
                        }
                    }

                    output_text
                }
            }

            _ => {
                return Err(ValidationError::invalid_value(
                    "operation",
                    format!("Unknown operation: {}", operation),
                )
                .into());
            }
        };

        // Set output
        output.result = TemplateResult::text(result_text);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("operation", json!(operation));
        output.add_metric("collection", json!(collection));

        info!(
            "Knowledge management operation complete (duration: {}ms)",
            output.metrics.duration_ms
        );

        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let operation: String = params.get_or("operation", "query".to_string());

        // Token estimation based on operation
        let estimated_tokens = match operation.as_str() {
            "ingest" => 500, // Parsing and chunking
            "query" => 1000, // Search and synthesis
            "update" => 400,
            "delete" => 100,
            "list" => 200,
            _ => 300,
        };

        // Duration estimates (in milliseconds)
        let estimated_duration = match operation.as_str() {
            "ingest" => 500, // Document processing
            "query" => 300,  // Search operation
            "update" => 400,
            "delete" => 100,
            "list" => 150,
            _ => 250,
        };

        CostEstimate::new(estimated_tokens, 0.0, estimated_duration, 0.7)
    }
}
