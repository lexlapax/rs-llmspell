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
use tracing::{debug, info, warn};

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
        context: &ExecutionContext,
    ) -> Result<Vec<Source>> {
        use llmspell_core::types::AgentInput;

        info!(
            "Gathering sources for topic: '{}' (max_sources: {})",
            topic, max_sources
        );

        // Get tool registry from context
        let tool_registry = context.tool_registry();

        // Create input for web-searcher tool with properly nested parameters
        // Note: extract_parameters() expects { "parameters": { ... } } structure
        let nested_params = serde_json::json!({
            "input": topic,
            "max_results": max_sources,
            "search_type": "web"
        });

        let input = AgentInput::builder()
            .text("") // Empty text, using parameters instead
            .parameter("parameters", nested_params)
            .build();

        // Execute web search
        let output = tool_registry
            .execute_tool(
                "web-searcher",
                input,
                llmspell_core::ExecutionContext::default(),
            )
            .await
            .map_err(|e| {
                warn!("Web search failed: {}", e);
                TemplateError::ExecutionFailed(format!("Web search failed: {}", e))
            })?;

        // Parse JSON response
        let response: serde_json::Value = serde_json::from_str(&output.text).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to parse search response: {}", e))
        })?;

        // Extract results array from response
        let results = response
            .get("result")
            .and_then(|r| r.get("results"))
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| {
                TemplateError::ExecutionFailed(
                    "Search response missing 'result.results' array".to_string(),
                )
            })?;

        // Convert SearchResult JSON to Source structs
        let sources: Vec<Source> = results
            .iter()
            .filter_map(|r| {
                Some(Source {
                    title: r.get("title")?.as_str()?.to_string(),
                    url: r.get("url")?.as_str()?.to_string(),
                    content: r.get("snippet")?.as_str()?.to_string(),
                    relevance_score: 1.0 - (r.get("rank")?.as_u64()? as f64 * 0.1),
                })
            })
            .take(max_sources)
            .collect();

        if sources.is_empty() {
            warn!("No sources found for topic: '{}'", topic);
            return Err(TemplateError::ExecutionFailed(format!(
                "No search results found for topic: '{}'",
                topic
            )));
        }

        info!(
            "Successfully gathered {} sources for topic: '{}'",
            sources.len(),
            topic
        );

        Ok(sources)
    }

    /// Phase 2: Ingest sources into RAG store
    async fn ingest_sources(
        &self,
        sources: &[Source],
        session_tag: &str,
        context: &ExecutionContext,
    ) -> Result<RagIngestionResult> {
        use llmspell_core::state::StateScope;
        use std::collections::HashMap;
        use std::time::SystemTime;

        info!(
            "Ingesting {} sources into RAG with tag: '{}'",
            sources.len(),
            session_tag
        );

        // Check if RAG is available
        let rag = context.rag().ok_or_else(|| {
            TemplateError::InfrastructureUnavailable("RAG not available".to_string())
        })?;

        // Create tenant if it doesn't exist (auto-provisioning for research sessions)
        // Uses session_tag as tenant ID for isolation
        let tenant_config = llmspell_tenancy::TenantConfig {
            tenant_id: session_tag.to_string(),
            name: format!("Research Session: {}", session_tag),
            limits: llmspell_tenancy::TenantLimits::default(),
            active: true,
            metadata: std::collections::HashMap::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            custom_config: None,
        };

        // Create tenant (ignore error if already exists)
        if let Err(e) = rag.tenant_manager().create_tenant(tenant_config).await {
            debug!("Tenant creation skipped (may already exist): {}", e);
        }

        // Create session scope for isolation
        let scope = StateScope::Custom(format!("research_session:{}", session_tag));

        // Prepare texts for embedding (combine title, URL, and content)
        let texts: Vec<String> = sources
            .iter()
            .map(|s| format!("{}\n{}\n{}", s.title, s.url, s.content))
            .collect();

        // Clone sources data for closure to avoid borrow checker issues
        let sources_clone: Vec<Source> = sources.to_vec();
        let session_tag_clone = session_tag.to_string();

        // Ingest documents into RAG storage with metadata
        let vector_ids = rag
            .ingest_documents(
                session_tag, // tenant_id
                &texts,
                scope,
                Some(
                    move |i: usize, _text: &str| -> HashMap<String, serde_json::Value> {
                        // Build metadata for each source
                        let mut metadata = HashMap::new();
                        let source = &sources_clone[i];

                        metadata.insert(
                            "title".to_string(),
                            serde_json::Value::String(source.title.clone()),
                        );
                        metadata.insert(
                            "url".to_string(),
                            serde_json::Value::String(source.url.clone()),
                        );
                        metadata.insert(
                            "content".to_string(),
                            serde_json::Value::String(source.content.clone()),
                        );
                        metadata.insert(
                            "relevance_score".to_string(),
                            serde_json::Value::Number(
                                serde_json::Number::from_f64(source.relevance_score)
                                    .unwrap_or_else(|| serde_json::Number::from(0)),
                            ),
                        );
                        metadata.insert(
                            "session_tag".to_string(),
                            serde_json::Value::String(session_tag_clone.clone()),
                        );
                        metadata
                    },
                ),
            )
            .await
            .map_err(|e| {
                warn!("Failed to ingest documents into RAG: {}", e);
                TemplateError::ExecutionFailed(format!("RAG ingestion failed: {}", e))
            })?;

        info!(
            "Successfully ingested {} sources into RAG storage (session: {}, vector_ids: {:?})",
            sources.len(),
            session_tag,
            vector_ids
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
        session_tag: &str,
        model: &str,
        context: &ExecutionContext,
    ) -> Result<String> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::state::StateScope;
        use llmspell_core::types::AgentInput;

        info!(
            "Synthesizing findings for topic: '{}' (session: {}, model: {})",
            topic, session_tag, model
        );

        // Retrieve RAG context from ingested sources
        let rag_context = if let Some(rag) = context.rag() {
            let scope = StateScope::Custom(format!("research_session:{}", session_tag));

            // Retrieve top 5 most relevant sources
            match rag.retrieve_context(session_tag, topic, scope, 5).await {
                Ok(results) => {
                    info!("Retrieved {} relevant sources from RAG", results.len());

                    // Format retrieved sources for inclusion in prompt
                    if results.is_empty() {
                        String::new()
                    } else {
                        let formatted_sources = results
                            .iter()
                            .enumerate()
                            .map(|(i, result)| {
                                let title = result
                                    .metadata
                                    .get("title")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown");
                                let url = result
                                    .metadata
                                    .get("url")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");

                                format!(
                                    "SOURCE {}: {} (relevance: {:.2})\nURL: {}\nContent:\n{}\n",
                                    i + 1,
                                    title,
                                    result.score,
                                    url,
                                    result.text
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n---\n\n");

                        format!("\n\nRELEVANT SOURCES:\n{}\n", formatted_sources)
                    }
                }
                Err(e) => {
                    warn!("Failed to retrieve RAG context: {}", e);
                    String::new()
                }
            }
        } else {
            String::new()
        };

        // Parse model specification (format: "provider/model-id" or just "model-id")
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        // Get agent registry
        let agent_registry = context.agent_registry();

        // Create synthesis agent configuration
        let agent_config = AgentConfig {
            name: format!("research-synthesizer-{}", session_tag),
            description: format!("Research synthesis agent for topic: {}", topic),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: Some(0.7), // Balanced creativity for synthesis
                max_tokens: Some(2000),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 120, // 2 minutes for synthesis
                max_memory_mb: 512,
                max_tool_calls: 0, // No tools needed for synthesis
                max_recursion_depth: 1,
            },
        };

        // Create agent
        let agent = agent_registry
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create synthesis agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build synthesis prompt with RAG context
        let synthesis_prompt = format!(
            "You are a research synthesis assistant. Your task is to create a comprehensive research report based on the provided sources.\n\n\
             RESEARCH TOPIC: {}{}\n\n\
             INSTRUCTIONS:\n\
             1. Provide an executive summary of the research topic based on the sources\n\
             2. Identify 3-5 key findings or insights about the topic from the sources\n\
             3. Discuss implications and practical applications\n\
             4. Suggest 2-3 areas for further investigation\n\
             5. Use clear section headers and bullet points\n\
             6. Keep the tone professional and academic\n\
             7. Base your synthesis on the provided sources when available\n\n\
             FORMAT:\n\
             # Research Synthesis: [Topic]\n\n\
             ## Executive Summary\n\
             [1-2 paragraphs]\n\n\
             ## Key Findings\n\
             1. [Finding with explanation]\n\
             2. [Finding with explanation]\n\
             3. [Finding with explanation]\n\n\
             ## Implications\n\
             [Discussion of practical applications and significance]\n\n\
             ## Further Research\n\
             [Suggested areas for deeper investigation]\n\n\
             Please provide a well-structured synthesis now.",
            topic,
            rag_context
        );

        // Create input for agent
        let agent_input = AgentInput::builder().text(synthesis_prompt).build();

        // Execute synthesis agent
        let output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Synthesis agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Synthesis failed: {}", e))
            })?;

        info!(
            "Successfully synthesized findings for topic: '{}' ({} characters)",
            topic,
            output.text.len()
        );

        Ok(output.text)
    }

    /// Phase 4: Validate citations with validation agent
    async fn validate_citations(
        &self,
        synthesis: &str,
        sources: &[Source],
        model: &str,
        context: &ExecutionContext,
    ) -> Result<String> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!(
            "Validating citations in synthesis ({} chars, {} sources, model: {})",
            synthesis.len(),
            sources.len(),
            model
        );

        // Parse model specification
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        };

        // Get agent registry
        let agent_registry = context.agent_registry();

        // Create validation agent configuration
        let agent_config = AgentConfig {
            name: "citation-validator".to_string(),
            description: "Citation validation agent for research reports".to_string(),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: Some(0.3), // Lower temperature for factual validation
                max_tokens: Some(1500),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 90, // 1.5 minutes for validation
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create validation agent
        let agent = agent_registry
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create validation agent: {}", e);
                TemplateError::ExecutionFailed(format!("Validation agent creation failed: {}", e))
            })?;

        // Build validation prompt with source information
        let sources_list: String = sources
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{}. {} - {}", i + 1, s.title, s.url))
            .collect::<Vec<_>>()
            .join("\n");

        let validation_prompt = format!(
            "You are a citation validation assistant. Analyze the research synthesis and verify citation quality.\n\n\
             SYNTHESIS TO VALIDATE:\n{}\n\n\
             AVAILABLE SOURCES:\n{}\n\n\
             VALIDATION TASKS:\n\
             1. Check if the synthesis maintains academic rigor\n\
             2. Verify claims are reasonable for the given topic\n\
             3. Assess if key points are properly supported\n\
             4. Identify any unsubstantiated assertions\n\
             5. Evaluate overall quality and coherence\n\n\
             Provide a validation report in this format:\n\n\
             # Citation Validation Report\n\n\
             ## Summary\n\
             [Brief overview of validation findings]\n\n\
             ## Validation Results\n\
             - Quality Score: [X/10]\n\
             - Rigor Assessment: [Pass/Needs Improvement]\n\
             - Claims Support: [Adequate/Weak]\n\n\
             ## Source Utilization\n\
             [Assessment of how well the synthesis uses available sources]\n\n\
             ## Recommendations\n\
             [Specific suggestions for improving the synthesis]\n\n\
             Please provide your validation report now.",
            synthesis, sources_list
        );

        // Create input for validation agent
        let agent_input = AgentInput::builder().text(validation_prompt).build();

        // Execute validation agent
        let output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Validation agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Validation failed: {}", e))
            })?;

        info!(
            "Successfully validated synthesis ({} characters validation report)",
            output.text.len()
        );

        Ok(output.text)
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
