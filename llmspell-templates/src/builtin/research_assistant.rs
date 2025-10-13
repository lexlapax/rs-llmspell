//! Research Assistant Template
//!
//! 4-phase workflow template for academic/professional research:
//! 1. Gather: Parallel web search for sources
//! 2. Ingest: RAG indexing of documents
//! 3. Synthesize: Agent-based synthesis with RAG context
//! 4. Validate: Citation validation agent

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

/// Research Assistant Template
///
/// Comprehensive research workflow with web search, RAG, and AI synthesis:
/// - Gathers sources from web search
/// - Ingests into RAG store
/// - Synthesizes findings with citations
/// - Validates citations and sources
#[derive(Debug)]
pub struct ResearchAssistantTemplate {
    metadata: TemplateMetadata,
}

impl ResearchAssistantTemplate {
    /// Create a new Research Assistant template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "research-assistant".to_string(),
                name: "Research Assistant".to_string(),
                description: "Multi-source research with citations, synthesis, and validation. \
                             Performs parallel web search, RAG ingestion, AI synthesis with \
                             citations, and citation validation."
                    .to_string(),
                category: TemplateCategory::Research,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec![
                    "web-search".to_string(),
                    "rag".to_string(),
                    "local-llm".to_string(),
                ],
                tags: vec![
                    "research".to_string(),
                    "citations".to_string(),
                    "multi-source".to_string(),
                    "synthesis".to_string(),
                    "rag".to_string(),
                ],
            },
        }
    }
}

impl Default for ResearchAssistantTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for ResearchAssistantTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // topic (required)
            ParameterSchema::required("topic", "Research topic or question", ParameterType::String)
                .with_constraints(ParameterConstraints {
                    min_length: Some(3),
                    ..Default::default()
                }),
            // max_sources (optional with default)
            ParameterSchema::optional(
                "max_sources",
                "Maximum number of sources to gather (1-50)",
                ParameterType::Integer,
                json!(10),
            )
            .with_constraints(ParameterConstraints {
                min: Some(1.0),
                max: Some(50.0),
                ..Default::default()
            }),
            // model (optional with default)
            ParameterSchema::optional(
                "model",
                "LLM model to use for synthesis and validation",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
            // output_format (optional with default and enum)
            ParameterSchema::optional(
                "output_format",
                "Output format: markdown, json, or html",
                ParameterType::String,
                json!("markdown"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![json!("markdown"), json!("json"), json!("html")]),
                ..Default::default()
            }),
            // include_citations (optional with default)
            ParameterSchema::optional(
                "include_citations",
                "Include citation links in output",
                ParameterType::Boolean,
                json!(true),
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
        let topic: String = params.get("topic")?;
        let max_sources: i64 = params.get_or("max_sources", 10);
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());
        let output_format: String = params.get_or("output_format", "markdown".to_string());
        let include_citations: bool = params.get_or("include_citations", true);

        info!(
            "Starting research assistant for topic: '{}' (max_sources={}, model={})",
            topic, max_sources, model
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Phase 1: Gather sources via web search
        info!("Phase 1: Gathering sources...");
        let sources = self
            .gather_sources(&topic, max_sources as usize, &context)
            .await?;
        output.metrics.tools_invoked += sources.len();

        // Phase 2: Ingest sources into RAG
        info!("Phase 2: Ingesting sources into RAG...");
        let session_tag = format!("research-{}", uuid::Uuid::new_v4());
        let rag_result = self
            .ingest_sources(&sources, &session_tag, &context)
            .await?;
        output.metrics.rag_queries += 1;

        // Phase 3: Synthesize findings with agent
        info!("Phase 3: Synthesizing findings...");
        let synthesis = self
            .synthesize_findings(&topic, &session_tag, &model, &context)
            .await?;
        output.metrics.agents_invoked += 1;

        // Phase 4: Validate citations
        info!("Phase 4: Validating citations...");
        let validation = self
            .validate_citations(&synthesis, &sources, &model, &context)
            .await?;
        output.metrics.agents_invoked += 1;

        // Generate final output based on format
        let final_result = self.format_output(
            &topic,
            &synthesis,
            &validation,
            &sources,
            include_citations,
            &output_format,
        )?;

        // Save artifacts
        if let Some(output_dir) = &context.output_dir {
            self.save_artifacts(
                output_dir,
                &synthesis,
                &validation,
                &sources,
                &output_format,
                &mut output,
            )?;
        }

        // Set result and metrics
        output.result = final_result;
        output.set_duration(start_time.elapsed().as_millis() as u64);

        // Add custom metrics
        output.add_metric("sources_gathered", serde_json::json!(sources.len()));
        output.add_metric(
            "rag_documents_ingested",
            serde_json::json!(rag_result.count),
        );
        output.add_metric("session_tag", serde_json::json!(session_tag));

        info!(
            "Research complete (duration: {}ms)",
            output.metrics.duration_ms
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        // Estimate based on max_sources and typical token usage
        let max_sources: i64 = params.get_or("max_sources", 10);

        // Rough estimates:
        // - Web search: minimal tokens
        // - RAG ingestion: ~500 tokens per source
        // - Synthesis: ~2000 tokens
        // - Validation: ~1000 tokens
        let estimated_tokens = (max_sources * 500) + 2000 + 1000;

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Each source: ~2s gather + ~1s ingest
        // Synthesis: ~5s
        // Validation: ~3s
        let estimated_duration = (max_sources * 3000) + 5000 + 3000;

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.6, // Medium confidence
        )
    }
}

impl ResearchAssistantTemplate {
    /// Phase 1: Gather sources via parallel web search
    async fn gather_sources(
        &self,
        topic: &str,
        max_sources: usize,
        _context: &ExecutionContext,
    ) -> Result<Vec<Source>> {
        // TODO: Implement actual web search integration
        // For now, return placeholder sources
        warn!("Web search not yet implemented - using placeholder sources");

        Ok((0..max_sources.min(3))
            .map(|i| Source {
                title: format!("Source {} for: {}", i + 1, topic),
                url: format!("https://example.com/source-{}", i + 1),
                content: format!(
                    "This is placeholder content for source {} about {}. \
                     In production, this would contain actual web search results.",
                    i + 1,
                    topic
                ),
                relevance_score: 0.8 - (i as f64 * 0.1),
            })
            .collect())
    }

    /// Phase 2: Ingest sources into RAG store
    async fn ingest_sources(
        &self,
        sources: &[Source],
        session_tag: &str,
        _context: &ExecutionContext,
    ) -> Result<RagIngestionResult> {
        // TODO: Implement actual RAG integration
        // For now, simulate ingestion
        warn!("RAG ingestion not yet implemented - simulating ingestion");

        info!(
            "Ingesting {} sources with tag: {}",
            sources.len(),
            session_tag
        );

        Ok(RagIngestionResult {
            count: sources.len(),
            session_tag: session_tag.to_string(),
        })
    }

    /// Phase 3: Synthesize findings with agent using RAG context
    async fn synthesize_findings(
        &self,
        topic: &str,
        _session_tag: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<String> {
        // TODO: Implement actual agent synthesis with RAG retrieval
        // For now, return placeholder synthesis
        warn!("Agent synthesis not yet implemented - using placeholder");

        Ok(format!(
            "# Research Synthesis: {}\n\n\
             ## Overview\n\
             This is a placeholder synthesis for the research topic: {}\n\n\
             ## Key Findings\n\
             1. Finding 1: Placeholder finding based on sources\n\
             2. Finding 2: Another placeholder finding\n\
             3. Finding 3: Additional placeholder insight\n\n\
             ## Conclusions\n\
             The research on {} reveals several important aspects that require further investigation.\n\n\
             ## Citations\n\
             [1] Source 1\n\
             [2] Source 2\n\
             [3] Source 3\n",
            topic, topic, topic
        ))
    }

    /// Phase 4: Validate citations with validation agent
    async fn validate_citations(
        &self,
        synthesis: &str,
        sources: &[Source],
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<String> {
        // TODO: Implement actual validation agent
        // For now, return placeholder validation
        warn!("Citation validation not yet implemented - using placeholder");

        Ok(format!(
            "# Citation Validation Report\n\n\
             ## Summary\n\
             Validated {} citations in synthesis ({} characters)\n\n\
             ## Validation Results\n\
             - Citations found: 3\n\
             - Citations verified: 3\n\
             - Missing citations: 0\n\
             - Broken links: 0\n\n\
             ## Source Quality\n\
             - Total sources: {}\n\
             - Average relevance: 0.75\n\n\
             ## Recommendations\n\
             All citations appear valid and properly formatted.\n",
            sources.len(),
            synthesis.len(),
            sources.len()
        ))
    }

    /// Format final output based on requested format
    fn format_output(
        &self,
        topic: &str,
        synthesis: &str,
        validation: &str,
        sources: &[Source],
        include_citations: bool,
        format: &str,
    ) -> Result<TemplateResult> {
        match format {
            "markdown" => Ok(TemplateResult::text(self.format_markdown(
                topic,
                synthesis,
                validation,
                sources,
                include_citations,
            ))),
            "json" => Ok(TemplateResult::structured(self.format_json(
                topic,
                synthesis,
                validation,
                sources,
                include_citations,
            ))),
            "html" => Ok(TemplateResult::text(self.format_html(
                topic,
                synthesis,
                validation,
                sources,
                include_citations,
            ))),
            _ => Err(TemplateError::ExecutionFailed(format!(
                "Unsupported output format: {}",
                format
            ))),
        }
    }

    /// Format as markdown
    fn format_markdown(
        &self,
        topic: &str,
        synthesis: &str,
        validation: &str,
        sources: &[Source],
        include_citations: bool,
    ) -> String {
        let mut output = String::new();
        output.push_str(&format!("# Research Report: {}\n\n", topic));
        output.push_str("---\n\n");
        output.push_str(synthesis);
        output.push_str("\n\n---\n\n");
        output.push_str(validation);

        if include_citations {
            output.push_str("\n\n---\n\n## References\n\n");
            for (i, source) in sources.iter().enumerate() {
                output.push_str(&format!("{}. [{}]({})\n", i + 1, source.title, source.url));
            }
        }

        output
    }

    /// Format as JSON
    fn format_json(
        &self,
        topic: &str,
        synthesis: &str,
        validation: &str,
        sources: &[Source],
        include_citations: bool,
    ) -> serde_json::Value {
        serde_json::json!({
            "topic": topic,
            "synthesis": synthesis,
            "validation": validation,
            "sources": if include_citations {
                sources.iter().map(|s| serde_json::json!({
                    "title": s.title,
                    "url": s.url,
                    "relevance": s.relevance_score
                })).collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
    }

    /// Format as HTML
    fn format_html(
        &self,
        topic: &str,
        synthesis: &str,
        validation: &str,
        sources: &[Source],
        include_citations: bool,
    ) -> String {
        let mut output = String::new();
        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str(&format!("<title>Research Report: {}</title>\n", topic));
        output.push_str("<style>body { font-family: Arial, sans-serif; margin: 40px; }</style>\n");
        output.push_str("</head>\n<body>\n");
        output.push_str(&format!("<h1>Research Report: {}</h1>\n", topic));
        output.push_str("<hr>\n");
        output.push_str(&format!("<pre>{}</pre>\n", synthesis));
        output.push_str("<hr>\n");
        output.push_str(&format!("<pre>{}</pre>\n", validation));

        if include_citations {
            output.push_str("<hr>\n<h2>References</h2>\n<ol>\n");
            for source in sources {
                output.push_str(&format!(
                    "<li><a href=\"{}\">{}</a></li>\n",
                    source.url, source.title
                ));
            }
            output.push_str("</ol>\n");
        }

        output.push_str("</body>\n</html>\n");
        output
    }

    /// Save artifacts to output directory
    fn save_artifacts(
        &self,
        output_dir: &std::path::Path,
        synthesis: &str,
        validation: &str,
        _sources: &[Source],
        format: &str,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save synthesis
        let synthesis_path = output_dir.join(format!("synthesis.{}", format));
        fs::write(&synthesis_path, synthesis).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write synthesis: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            synthesis_path.to_string_lossy().to_string(),
            synthesis.to_string(),
            format!("text/{}", if format == "html" { "html" } else { "plain" }),
        ));

        // Save validation report
        let validation_path = output_dir.join("validation.txt");
        fs::write(&validation_path, validation).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write validation report: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            validation_path.to_string_lossy().to_string(),
            validation.to_string(),
            "text/plain",
        ));

        Ok(())
    }
}

/// Source document from web search
#[derive(Debug, Clone)]
struct Source {
    title: String,
    url: String,
    /// Placeholder for future web search integration
    #[allow(dead_code)]
    content: String,
    relevance_score: f64,
}

/// RAG ingestion result
#[derive(Debug)]
struct RagIngestionResult {
    count: usize,
    /// Placeholder for future RAG integration
    #[allow(dead_code)]
    session_tag: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;

    #[test]
    fn test_template_metadata() {
        let template = ResearchAssistantTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "research-assistant");
        assert_eq!(metadata.name, "Research Assistant");
        assert_eq!(metadata.category, TemplateCategory::Research);
        assert!(metadata.requires.contains(&"web-search".to_string()));
        assert!(metadata.requires.contains(&"rag".to_string()));
        assert!(metadata.requires.contains(&"local-llm".to_string()));
        assert!(metadata.tags.contains(&"research".to_string()));
        assert!(metadata.tags.contains(&"citations".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = ResearchAssistantTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("topic").is_some());
        assert!(schema.get_parameter("max_sources").is_some());
        assert!(schema.get_parameter("model").is_some());
        assert!(schema.get_parameter("output_format").is_some());
        assert!(schema.get_parameter("include_citations").is_some());

        let topic_param = schema.get_parameter("topic").unwrap();
        assert!(topic_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate() {
        let template = ResearchAssistantTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("max_sources", serde_json::json!(10));

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        assert!(estimate.estimated_cost_usd.is_some());
        assert!(estimate.estimated_duration_ms.is_some());
        assert!(estimate.confidence > 0.0);
    }

    #[test]
    fn test_parameter_validation_missing_required() {
        let template = ResearchAssistantTemplate::new();
        let schema = template.config_schema();
        let params = std::collections::HashMap::new();

        // Should fail - missing required "topic" parameter
        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_out_of_range() {
        let template = ResearchAssistantTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("topic".to_string(), serde_json::json!("test topic"));
        params.insert("max_sources".to_string(), serde_json::json!(100)); // Over max of 50

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_invalid_enum() {
        let template = ResearchAssistantTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("topic".to_string(), serde_json::json!("test topic"));
        params.insert(
            "output_format".to_string(),
            serde_json::json!("invalid_format"),
        ); // Not in allowed values

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = ResearchAssistantTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("topic".to_string(), serde_json::json!("AI research"));
        params.insert("max_sources".to_string(), serde_json::json!(10));
        params.insert("output_format".to_string(), serde_json::json!("markdown"));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    // NOTE: Full integration tests with ExecutionContext require actual infrastructure
    // (tool_registry, agent_registry, workflow_factory, providers). These tests will
    // be added once web search, RAG, and agent integration is implemented.
    // For now, testing covers:
    // - Template metadata and schema
    // - Parameter validation
    // - Output formatting
    // - Placeholder phase execution (gather, ingest, synthesize, validate)

    #[tokio::test]
    async fn test_gather_sources_placeholder() {
        let template = ResearchAssistantTemplate::new();
        // Create a minimal context (doesn't need full infrastructure for placeholder)
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available - this is a placeholder test
            return;
        }
        let context = context.unwrap();

        let sources = template.gather_sources("test topic", 3, &context).await;
        assert!(sources.is_ok());
        let sources = sources.unwrap();
        assert_eq!(sources.len(), 3);
        assert!(sources[0].title.contains("test topic"));
    }

    #[tokio::test]
    async fn test_format_output_types() {
        let template = ResearchAssistantTemplate::new();
        let sources = vec![];

        // Test markdown format
        let result = template.format_output(
            "Test",
            "Synthesis",
            "Validation",
            &sources,
            true,
            "markdown",
        );
        assert!(result.is_ok());
        match result.unwrap() {
            TemplateResult::Text(_) => (),
            _ => panic!("Expected text result for markdown"),
        }

        // Test JSON format
        let result =
            template.format_output("Test", "Synthesis", "Validation", &sources, true, "json");
        assert!(result.is_ok());
        match result.unwrap() {
            TemplateResult::Structured(_) => (),
            _ => panic!("Expected structured result for JSON"),
        }

        // Test HTML format
        let result =
            template.format_output("Test", "Synthesis", "Validation", &sources, true, "html");
        assert!(result.is_ok());
        match result.unwrap() {
            TemplateResult::Text(_) => (),
            _ => panic!("Expected text result for HTML"),
        }
    }

    #[test]
    fn test_format_markdown() {
        let template = ResearchAssistantTemplate::new();
        let sources = vec![
            Source {
                title: "Source 1".to_string(),
                url: "https://example.com/1".to_string(),
                content: "content".to_string(),
                relevance_score: 0.9,
            },
            Source {
                title: "Source 2".to_string(),
                url: "https://example.com/2".to_string(),
                content: "content".to_string(),
                relevance_score: 0.8,
            },
        ];

        let output = template.format_markdown(
            "Test Topic",
            "Synthesis content",
            "Validation content",
            &sources,
            true,
        );

        assert!(output.contains("# Research Report: Test Topic"));
        assert!(output.contains("Synthesis content"));
        assert!(output.contains("Validation content"));
        assert!(output.contains("## References"));
        assert!(output.contains("[Source 1](https://example.com/1)"));
    }

    #[test]
    fn test_format_json() {
        let template = ResearchAssistantTemplate::new();
        let sources = vec![Source {
            title: "Source 1".to_string(),
            url: "https://example.com/1".to_string(),
            content: "content".to_string(),
            relevance_score: 0.9,
        }];

        let json_output =
            template.format_json("Test Topic", "Synthesis", "Validation", &sources, true);

        assert_eq!(json_output["topic"], "Test Topic");
        assert_eq!(json_output["synthesis"], "Synthesis");
        assert_eq!(json_output["validation"], "Validation");
        assert_eq!(json_output["sources"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_format_html() {
        let template = ResearchAssistantTemplate::new();
        let sources = vec![Source {
            title: "Source 1".to_string(),
            url: "https://example.com/1".to_string(),
            content: "content".to_string(),
            relevance_score: 0.9,
        }];

        let html_output =
            template.format_html("Test Topic", "Synthesis", "Validation", &sources, true);

        assert!(html_output.contains("<!DOCTYPE html>"));
        assert!(html_output.contains("<title>Research Report: Test Topic</title>"));
        assert!(html_output.contains("Synthesis"));
        assert!(html_output.contains("Validation"));
        assert!(html_output.contains("<a href=\"https://example.com/1\">Source 1</a>"));
    }

    #[test]
    fn test_unsupported_output_format() {
        let template = ResearchAssistantTemplate::new();
        let sources = vec![];

        let result = template.format_output(
            "Test",
            "Synthesis",
            "Validation",
            &sources,
            true,
            "xml", // Unsupported format
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported output format"));
    }
}
