//! Content Generation Template
//!
//! Quality-driven content creation with 4-agent pipeline:
//! 1. Content Planner: Creates outline, structure, and key points
//! 2. Content Writer: Generates initial draft based on plan
//! 3. Content Editor: Reviews quality, scores content, suggests improvements
//! 4. Content Formatter: Finalizes formatting for publication
//!
//! Implements iterative quality improvement: if quality score < threshold, iterate with editor feedback.

use crate::{
    artifacts::Artifact,
    context::ExecutionContext,
    core::{
        memory_parameters, provider_parameters, CostEstimate, TemplateCategory, TemplateMetadata,
        TemplateOutput, TemplateParams, TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

/// Content Generation Template
///
/// Automated content creation with quality-driven iteration:
/// - 4-agent pipeline: planner → writer → editor → formatter
/// - Quality scoring (0.0-1.0) with threshold-based iteration
/// - Content type presets (blog, documentation, marketing, technical, creative)
/// - Tone/style configuration
/// - Multiple output formats (markdown, HTML, text, JSON)
#[derive(Debug)]
pub struct ContentGenerationTemplate {
    metadata: TemplateMetadata,
}

impl ContentGenerationTemplate {
    /// Create a new Content Generation template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "content-generation".to_string(),
                name: "Content Generation".to_string(),
                description: "Quality-driven content creation with iterative improvement. \
                             Creates content through a 4-stage pipeline: planning, writing, \
                             editing, and formatting. Automatically iterates based on quality \
                             scores until threshold is met or max iterations reached."
                    .to_string(),
                category: TemplateCategory::Custom("Content".to_string()),
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["agents".to_string()],
                tags: vec![
                    "content".to_string(),
                    "writing".to_string(),
                    "documentation".to_string(),
                    "marketing".to_string(),
                    "quality-control".to_string(),
                ],
            },
        }
    }
}

impl Default for ContentGenerationTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for ContentGenerationTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        let mut params = vec![
            // topic (required)
            ParameterSchema::required("topic", "Content topic or title", ParameterType::String)
                .with_constraints(ParameterConstraints {
                    min_length: Some(3),
                    ..Default::default()
                }),
            // content_type (optional enum with default)
            ParameterSchema::optional(
                "content_type",
                "Type of content to generate",
                ParameterType::String,
                json!("general"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("blog"),
                    json!("documentation"),
                    json!("marketing"),
                    json!("technical"),
                    json!("creative"),
                    json!("general"),
                ]),
                ..Default::default()
            }),
            // target_length (optional integer)
            ParameterSchema::optional(
                "target_length",
                "Target word count for content",
                ParameterType::Integer,
                json!(null),
            )
            .with_constraints(ParameterConstraints {
                min: Some(50.0),
                max: Some(10000.0),
                ..Default::default()
            }),
            // tone (optional enum)
            ParameterSchema::optional(
                "tone",
                "Tone of the content",
                ParameterType::String,
                json!("professional"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("professional"),
                    json!("casual"),
                    json!("technical"),
                    json!("persuasive"),
                    json!("friendly"),
                ]),
                ..Default::default()
            }),
            // style_guide (optional string)
            ParameterSchema::optional(
                "style_guide",
                "Custom style guidelines for content",
                ParameterType::String,
                json!(null),
            ),
            // quality_threshold (optional float with range)
            ParameterSchema::optional(
                "quality_threshold",
                "Quality score threshold (0.0-1.0) for iteration",
                ParameterType::Number,
                json!(0.8),
            )
            .with_constraints(ParameterConstraints {
                min: Some(0.0),
                max: Some(1.0),
                ..Default::default()
            }),
            // max_iterations (optional integer with range)
            ParameterSchema::optional(
                "max_iterations",
                "Maximum editing iterations (1-10)",
                ParameterType::Integer,
                json!(3),
            )
            .with_constraints(ParameterConstraints {
                min: Some(1.0),
                max: Some(10.0),
                ..Default::default()
            }),
            // output_format (optional enum)
            ParameterSchema::optional(
                "output_format",
                "Output format for generated content",
                ParameterType::String,
                json!("markdown"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("markdown"),
                    json!("html"),
                    json!("text"),
                    json!("json"),
                ]),
                ..Default::default()
            }),
            // include_outline (optional boolean)
            ParameterSchema::optional(
                "include_outline",
                "Include planning outline in output",
                ParameterType::Boolean,
                json!(false),
            ),
            // model (optional)
            ParameterSchema::optional(
                "model",
                "LLM model to use for content generation agents",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
        ];

        // Add memory parameters (Task 13.11.1)
        params.extend(memory_parameters());

        // Add provider parameters (Task 13.5.7d)
        params.extend(provider_parameters());

        tracing::debug!(
            "ContentGeneration: Generated config schema with {} parameters",
            params.len()
        );
        ConfigSchema::new(params)
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let topic: String = params.get("topic")?;
        let content_type: String = params.get_or("content_type", "general".to_string());
        let target_length: Option<i64> = params.get_optional("target_length");
        let tone: String = params.get_or("tone", "professional".to_string());
        let style_guide: Option<String> = params.get_optional("style_guide");
        let quality_threshold: f32 = params.get_or("quality_threshold", 0.8);
        let max_iterations: i64 = params.get_or("max_iterations", 3);
        let output_format: String = params.get_or("output_format", "markdown".to_string());
        let include_outline: bool = params.get_or("include_outline", false);

        // Smart dual-path provider resolution (Task 13.5.7d)
        let provider_config = context.resolve_llm_config(&params)?;
        let model_str = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        info!(
            "Starting content generation (topic='{}', type={}, quality_threshold={}, max_iterations={}, model={})",
            topic, content_type, quality_threshold, max_iterations, model_str
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params.clone(),
        );

        // Phase 1: Plan content
        info!("Phase 1: Planning content...");
        let plan = self
            .create_content_plan(
                &topic,
                &content_type,
                target_length,
                &tone,
                &provider_config,
                &context,
            )
            .await?;
        output.metrics.agents_invoked += 1;

        // Phase 2: Write initial draft
        info!("Phase 2: Writing initial draft...");
        let mut draft = self
            .write_content(
                &plan,
                target_length,
                &tone,
                style_guide.as_deref(),
                &provider_config,
                &context,
            )
            .await?;
        output.metrics.agents_invoked += 1;

        // Phase 3: Iterative editing with quality control
        info!("Phase 3: Editing with quality control...");
        let mut current_quality = 0.0;
        let mut iteration_count = 0;
        let mut editor_feedback = Vec::new();

        while iteration_count < max_iterations {
            // Get quality score and feedback
            let review = self
                .review_content(&draft, &content_type, &tone, &provider_config, &context)
                .await?;
            output.metrics.agents_invoked += 1;

            current_quality = review.quality_score;
            editor_feedback.push(review.clone());

            info!(
                "Iteration {}: Quality score = {:.2} (threshold = {:.2})",
                iteration_count + 1,
                current_quality,
                quality_threshold
            );

            // Check if quality threshold met
            if current_quality >= quality_threshold {
                info!("Quality threshold met, stopping iterations");
                break;
            }

            // If not last iteration, improve content based on feedback
            if iteration_count < max_iterations - 1 {
                info!("Improving content based on feedback...");
                draft = self
                    .improve_content(&draft, &review.feedback, &provider_config, &context)
                    .await?;
                output.metrics.agents_invoked += 1;
            }

            iteration_count += 1;
        }

        // Phase 4: Format final content
        info!("Phase 4: Formatting final content...");
        let formatted = self
            .format_content(&draft, &output_format, &provider_config, &context)
            .await?;
        output.metrics.agents_invoked += 1;

        // Generate final output
        let report = match output_format.as_str() {
            "json" => self.format_json_output(
                &topic,
                &plan,
                &formatted,
                current_quality,
                iteration_count,
                &editor_feedback,
            )?,
            "html" => self.format_html_output(&topic, &plan, &formatted, include_outline),
            "text" => self.format_text_output(&formatted),
            _ => self.format_markdown_output(
                &topic,
                &plan,
                &formatted,
                include_outline,
                current_quality,
                iteration_count,
            ),
        };

        // Save artifacts if output directory provided
        if let Some(output_dir) = &context.output_dir {
            self.save_content_artifacts(output_dir, &topic, &plan, &formatted, &mut output)?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(report);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("topic", json!(topic));
        output.add_metric("content_type", json!(content_type));
        output.add_metric("quality_score", json!(current_quality));
        output.add_metric("iterations", json!(iteration_count));
        output.add_metric("quality_threshold", json!(quality_threshold));
        output.add_metric("threshold_met", json!(current_quality >= quality_threshold));
        if let Some(length) = target_length {
            output.add_metric("target_length", json!(length));
        }
        output.add_metric(
            "final_word_count",
            json!(formatted.split_whitespace().count()),
        );

        info!(
            "Content generation complete (duration: {}ms, quality: {:.2}, iterations: {}, agents: {})",
            output.metrics.duration_ms,
            current_quality,
            iteration_count,
            output.metrics.agents_invoked
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let max_iterations: i64 = params.get_or("max_iterations", 3);
        let target_length: Option<i64> = params.get_optional("target_length");

        // Base tokens for 4-agent pipeline:
        // - Plan: ~600 tokens
        // - Write: ~1500 tokens (varies with target_length)
        // - Review: ~800 tokens per iteration
        // - Improve: ~1200 tokens per iteration
        // - Format: ~500 tokens

        let base_tokens = 600 + 1500 + 500; // plan + write + format
        let review_tokens = 800 * max_iterations as u64; // review per iteration
        let improve_tokens = 1200 * (max_iterations - 1).max(0) as u64; // improve (iterations - 1)

        let mut estimated_tokens = base_tokens + review_tokens + improve_tokens;

        // Adjust for target length
        if let Some(length) = target_length {
            // Rough estimate: 1 token per word + overhead
            let length_adjustment = (length as f64 * 1.3) as u64;
            estimated_tokens = estimated_tokens.max(length_adjustment);
        }

        // Base duration:
        // - Plan: ~5s
        // - Write: ~8s
        // - Review: ~4s per iteration
        // - Improve: ~6s per iteration (iterations - 1)
        // - Format: ~3s
        let base_duration = 5000 + 8000 + 3000; // plan + write + format
        let review_duration = 4000 * max_iterations as u64;
        let improve_duration = 6000 * (max_iterations - 1).max(0) as u64;
        let estimated_duration = base_duration + review_duration + improve_duration;

        // Assuming $0.10 per 1M tokens
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        CostEstimate::new(
            estimated_tokens,
            estimated_cost,
            estimated_duration,
            0.7, // Medium-high confidence
        )
    }
}

// Internal helper types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentPlan {
    outline: String,
    key_points: Vec<String>,
    target_audience: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ContentReview {
    quality_score: f32,
    feedback: String,
    strengths: Vec<String>,
    improvements: Vec<String>,
}

impl ContentGenerationTemplate {
    /// Phase 1: Create content plan with outline and structure
    async fn create_content_plan(
        &self,
        topic: &str,
        content_type: &str,
        target_length: Option<i64>,
        tone: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<ContentPlan> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        let (provider, model_id) = Self::parse_model(model);

        // Get content type-specific planning guidance
        let type_guidance = self.get_content_type_guidance(content_type);

        let agent_config = AgentConfig {
            name: "content-planner".to_string(),
            description: format!("Content planning agent for {} content", content_type),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.3)), // Lower temperature for structured planning
                max_tokens: provider_config.max_tokens.or(Some(1500)),
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

        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create content planner: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        let length_guidance = target_length
            .map(|l| format!("\n- Target length: {} words", l))
            .unwrap_or_default();

        let planning_prompt = format!(
            "You are a content planning expert. Create a detailed plan for {} content.\n\n\
             **Topic**: {}\n\
             **Tone**: {}{}\n\n\
             {}\n\n\
             Provide your response as JSON with this structure:\n\
             {{\n\
               \"outline\": \"detailed outline with sections and subsections\",\n\
               \"key_points\": [\"point 1\", \"point 2\", ...],\n\
               \"target_audience\": \"description of target audience\"\n\
             }}",
            content_type, topic, tone, length_guidance, type_guidance
        );

        let agent_input = AgentInput::builder().text(planning_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Content planning failed: {}", e);
                TemplateError::ExecutionFailed(format!("Planning failed: {}", e))
            })?;

        // Parse JSON response
        let plan_data: serde_json::Value =
            serde_json::from_str(&agent_output.text).unwrap_or_else(|_| {
                json!({
                    "outline": agent_output.text,
                    "key_points": [],
                    "target_audience": "general audience"
                })
            });

        let plan = ContentPlan {
            outline: plan_data["outline"]
                .as_str()
                .unwrap_or("No outline provided")
                .to_string(),
            key_points: plan_data["key_points"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            target_audience: plan_data["target_audience"]
                .as_str()
                .unwrap_or("general audience")
                .to_string(),
        };

        info!(
            "Content plan created with {} key points",
            plan.key_points.len()
        );
        Ok(plan)
    }

    /// Phase 2: Write content based on plan
    async fn write_content(
        &self,
        plan: &ContentPlan,
        target_length: Option<i64>,
        tone: &str,
        style_guide: Option<&str>,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<String> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        let (provider, model_id) = Self::parse_model(model);

        let agent_config = AgentConfig {
            name: "content-writer".to_string(),
            description: "Content writing agent".to_string(),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.5)), // Higher temperature for creative writing
                max_tokens: provider_config.max_tokens.or(Some(3000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 180,
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create content writer: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        let length_instruction = target_length
            .map(|l| format!("\n- Aim for approximately {} words", l))
            .unwrap_or_default();

        let style_instruction = style_guide
            .map(|s| format!("\n- Follow these style guidelines: {}", s))
            .unwrap_or_default();

        let writing_prompt = format!(
            "You are a professional content writer. Write engaging, high-quality content based on the plan below.\n\n\
             **Content Plan**:\n{}\n\n\
             **Key Points to Cover**:\n{}\n\n\
             **Target Audience**: {}\n\
             **Tone**: {}{}{}\n\n\
             Write complete, polished content that fully addresses the plan. Make it engaging and valuable.",
            plan.outline,
            plan.key_points.join("\n- "),
            plan.target_audience,
            tone,
            length_instruction,
            style_instruction
        );

        let agent_input = AgentInput::builder().text(writing_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Content writing failed: {}", e);
                TemplateError::ExecutionFailed(format!("Writing failed: {}", e))
            })?;

        let word_count = agent_output.text.split_whitespace().count();
        info!("Content draft written ({} words)", word_count);
        Ok(agent_output.text)
    }

    /// Phase 3: Review content and provide quality score + feedback
    async fn review_content(
        &self,
        content: &str,
        content_type: &str,
        tone: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<ContentReview> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        let (provider, model_id) = Self::parse_model(model);

        let agent_config = AgentConfig {
            name: "content-editor".to_string(),
            description: "Content editor and quality reviewer".to_string(),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.3)), // Lower temperature for consistent evaluation
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

        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create content editor: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        let review_prompt = format!(
            "You are a content editor. Review this {} content with {} tone for quality.\n\n\
             **Content to Review**:\n{}\n\n\
             Evaluate the content on:\n\
             - Clarity and coherence\n\
             - Engagement and value\n\
             - Grammar and style\n\
             - Structure and flow\n\
             - Completeness\n\n\
             Provide your review as JSON:\n\
             {{\n\
               \"quality_score\": 0.0-1.0,\n\
               \"feedback\": \"overall assessment\",\n\
               \"strengths\": [\"strength 1\", \"strength 2\"],\n\
               \"improvements\": [\"improvement 1\", \"improvement 2\"]\n\
             }}",
            content_type, tone, content
        );

        let agent_input = AgentInput::builder().text(review_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Content review failed: {}", e);
                TemplateError::ExecutionFailed(format!("Review failed: {}", e))
            })?;

        // Parse JSON response
        let review_data: serde_json::Value = serde_json::from_str(&agent_output.text)
            .unwrap_or_else(|_| {
                json!({
                    "quality_score": 0.7,
                    "feedback": agent_output.text,
                    "strengths": [],
                    "improvements": []
                })
            });

        let review = ContentReview {
            quality_score: review_data["quality_score"].as_f64().unwrap_or(0.7) as f32,
            feedback: review_data["feedback"]
                .as_str()
                .unwrap_or("Review completed")
                .to_string(),
            strengths: review_data["strengths"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
            improvements: review_data["improvements"]
                .as_array()
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect()
                })
                .unwrap_or_default(),
        };

        Ok(review)
    }

    /// Improve content based on editor feedback
    async fn improve_content(
        &self,
        content: &str,
        feedback: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<String> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        let (provider, model_id) = Self::parse_model(model);

        let agent_config = AgentConfig {
            name: "content-improver".to_string(),
            description: "Content improvement agent".to_string(),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.4)),
                max_tokens: provider_config.max_tokens.or(Some(3000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 180,
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create content improver: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        let improvement_prompt = format!(
            "Improve this content based on the editor's feedback. Keep the core structure but enhance quality.\n\n\
             **Current Content**:\n{}\n\n\
             **Editor Feedback**:\n{}\n\n\
             Provide the improved version of the content.",
            content, feedback
        );

        let agent_input = AgentInput::builder().text(improvement_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Content improvement failed: {}", e);
                TemplateError::ExecutionFailed(format!("Improvement failed: {}", e))
            })?;

        Ok(agent_output.text)
    }

    /// Phase 4: Format final content
    async fn format_content(
        &self,
        content: &str,
        output_format: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<String> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        let (provider, model_id) = Self::parse_model(model);

        let agent_config = AgentConfig {
            name: "content-formatter".to_string(),
            description: "Content formatting agent".to_string(),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.2)), // Low temperature for consistent formatting
                max_tokens: provider_config.max_tokens.or(Some(3000)),
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

        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create content formatter: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        let format_instructions = match output_format {
            "html" => "Format as clean, semantic HTML with proper tags, headings, and structure.",
            "markdown" => {
                "Format as well-structured Markdown with proper headings, lists, and emphasis."
            }
            "text" => "Format as plain text with clear sections and readable structure.",
            _ => "Format for publication with professional structure.",
        };

        let formatting_prompt = format!(
            "Format this content for publication. {}\n\n\
             **Content**:\n{}\n\n\
             Provide the formatted version.",
            format_instructions, content
        );

        let agent_input = AgentInput::builder().text(formatting_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Content formatting failed: {}", e);
                TemplateError::ExecutionFailed(format!("Formatting failed: {}", e))
            })?;

        Ok(agent_output.text)
    }

    /// Get content type-specific guidance for planning
    fn get_content_type_guidance(&self, content_type: &str) -> &'static str {
        match content_type {
            "blog" => {
                "Create a blog post plan with:\n\
                 - Engaging introduction with hook\n\
                 - Clear section structure with subheadings\n\
                 - Practical examples and takeaways\n\
                 - Strong call-to-action conclusion"
            }
            "documentation" => {
                "Create technical documentation plan with:\n\
                 - Clear overview and prerequisites\n\
                 - Step-by-step instructions\n\
                 - Code examples and usage scenarios\n\
                 - Troubleshooting section\n\
                 - References and related resources"
            }
            "marketing" => {
                "Create marketing content plan with:\n\
                 - Attention-grabbing headline\n\
                 - Benefit-focused structure\n\
                 - Social proof and testimonials\n\
                 - Clear value proposition\n\
                 - Strong call-to-action"
            }
            "technical" => {
                "Create technical content plan with:\n\
                 - Precise technical introduction\n\
                 - Detailed methodology or approach\n\
                 - Technical specifications and data\n\
                 - Analysis and findings\n\
                 - Conclusions and recommendations"
            }
            "creative" => {
                "Create creative content plan with:\n\
                 - Compelling narrative structure\n\
                 - Character or theme development\n\
                 - Descriptive and engaging language\n\
                 - Emotional resonance\n\
                 - Satisfying resolution"
            }
            _ => {
                "Create a general content plan with:\n\
                 - Clear introduction\n\
                 - Well-organized main points\n\
                 - Supporting details and examples\n\
                 - Logical flow and transitions\n\
                 - Effective conclusion"
            }
        }
    }

    /// Parse model string into provider and model_id
    fn parse_model(model: &str) -> (String, String) {
        if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            ("ollama".to_string(), model.to_string())
        }
    }

    /// Format output as JSON
    fn format_json_output(
        &self,
        topic: &str,
        plan: &ContentPlan,
        content: &str,
        quality_score: f32,
        iterations: i64,
        feedback: &[ContentReview],
    ) -> Result<String> {
        let output_json = json!({
            "topic": topic,
            "plan": {
                "outline": plan.outline,
                "key_points": plan.key_points,
                "target_audience": plan.target_audience
            },
            "content": content,
            "quality_metrics": {
                "final_quality_score": quality_score,
                "iterations": iterations,
                "word_count": content.split_whitespace().count()
            },
            "reviews": feedback.iter().map(|r| json!({
                "quality_score": r.quality_score,
                "feedback": r.feedback,
                "strengths": r.strengths,
                "improvements": r.improvements
            })).collect::<Vec<_>>()
        });

        serde_json::to_string_pretty(&output_json).map_err(|e| {
            TemplateError::ExecutionFailed(format!("JSON serialization failed: {}", e))
        })
    }

    /// Format output as HTML
    fn format_html_output(
        &self,
        topic: &str,
        plan: &ContentPlan,
        content: &str,
        include_outline: bool,
    ) -> String {
        let mut html = format!(
            "<!DOCTYPE html>\n\
             <html>\n\
             <head>\n\
               <meta charset=\"utf-8\">\n\
               <title>{}</title>\n\
             </head>\n\
             <body>\n",
            topic
        );

        if include_outline {
            html.push_str(&format!(
                "  <details>\n\
                 <summary>Content Plan</summary>\n\
                 <pre>{}</pre>\n\
                 </details>\n\n",
                plan.outline
            ));
        }

        html.push_str(&format!("  {}\n", content));
        html.push_str("</body>\n</html>");

        html
    }

    /// Format output as plain text
    fn format_text_output(&self, content: &str) -> String {
        content.to_string()
    }

    /// Format output as markdown
    fn format_markdown_output(
        &self,
        topic: &str,
        plan: &ContentPlan,
        content: &str,
        include_outline: bool,
        quality_score: f32,
        iterations: i64,
    ) -> String {
        let mut output = format!("# {}\n\n", topic);

        if include_outline {
            output.push_str("## Content Plan\n\n");
            output.push_str(&format!("{}\n\n", plan.outline));
            output.push_str("---\n\n");
        }

        output.push_str(content);

        output.push_str(&format!(
            "\n\n---\n\n\
             *Generated by LLMSpell Content Generation Template*\n\
             *Quality Score: {:.2} | Iterations: {}*\n",
            quality_score, iterations
        ));

        output
    }

    /// Save content artifacts
    fn save_content_artifacts(
        &self,
        output_dir: &std::path::Path,
        topic: &str,
        plan: &ContentPlan,
        content: &str,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save plan
        let plan_path = output_dir.join("content-plan.md");
        let plan_content = format!(
            "# Content Plan: {}\n\n{}\n\n## Key Points\n\n{}\n\n## Target Audience\n\n{}\n",
            topic,
            plan.outline,
            plan.key_points.join("\n- "),
            plan.target_audience
        );
        fs::write(&plan_path, &plan_content)
            .map_err(|e| TemplateError::ExecutionFailed(format!("Failed to write plan: {}", e)))?;
        output.add_artifact(Artifact::new(
            plan_path.to_string_lossy().to_string(),
            plan_content,
            "text/markdown".to_string(),
        ));

        // Save content
        let content_path = output_dir.join("generated-content.md");
        fs::write(&content_path, content).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write content: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            content_path.to_string_lossy().to_string(),
            content.to_string(),
            "text/markdown".to_string(),
        ));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;
    use serde_json::json;
    use std::collections::HashMap;

    // Helper to convert json! to HashMap
    fn json_to_hashmap(value: serde_json::Value) -> HashMap<String, serde_json::Value> {
        value
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    #[test]
    fn test_template_metadata() {
        let template = ContentGenerationTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "content-generation");
        assert_eq!(metadata.name, "Content Generation");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(
            metadata.category,
            TemplateCategory::Custom("Content".to_string())
        );
        assert!(metadata.tags.contains(&"content".to_string()));
        assert!(metadata.tags.contains(&"writing".to_string()));
        assert!(metadata.tags.contains(&"quality-control".to_string()));
    }

    #[test]
    fn test_config_schema_required_parameters() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // topic is required
        let params = json_to_hashmap(json!({
            "content_type": "blog",
        }));
        let result = schema.validate(&params);
        assert!(
            result.is_err(),
            "Should fail without required topic parameter"
        );

        // Valid with just topic
        let params = json_to_hashmap(json!({
            "topic": "Rust async programming",
        }));
        let result = schema.validate(&params);
        assert!(result.is_ok(), "Should succeed with just topic parameter");
    }

    #[test]
    fn test_config_schema_content_type_validation() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid content types
        for content_type in &[
            "blog",
            "documentation",
            "marketing",
            "technical",
            "creative",
            "general",
        ] {
            let params = json_to_hashmap(json!({
                "topic": "Test topic",
                "content_type": content_type,
            }));
            assert!(
                schema.validate(&params).is_ok(),
                "Should accept valid content_type '{}'",
                content_type
            );
        }

        // Invalid content type
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "content_type": "poetry", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid content_type 'poetry'"
        );
    }

    #[test]
    fn test_config_schema_tone_validation() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid tones
        for tone in &[
            "professional",
            "casual",
            "technical",
            "persuasive",
            "friendly",
        ] {
            let params = json_to_hashmap(json!({
                "topic": "Test topic",
                "tone": tone,
            }));
            assert!(
                schema.validate(&params).is_ok(),
                "Should accept valid tone '{}'",
                tone
            );
        }

        // Invalid tone
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "tone": "sarcastic", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid tone 'sarcastic'"
        );
    }

    #[test]
    fn test_config_schema_output_format_validation() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid output formats
        for format in &["markdown", "html", "text", "json"] {
            let params = json_to_hashmap(json!({
                "topic": "Test topic",
                "output_format": format,
            }));
            assert!(
                schema.validate(&params).is_ok(),
                "Should accept valid output_format '{}'",
                format
            );
        }

        // Invalid output format
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "output_format": "pdf", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid output_format 'pdf'"
        );
    }

    #[test]
    fn test_config_schema_target_length_range() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid target_length
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "target_length": 500,
        }));
        assert!(
            schema.validate(&params).is_ok(),
            "Should accept target_length 500"
        );

        // target_length too low
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "target_length": 10, // Below minimum 50
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject target_length < 50"
        );

        // target_length too high
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "target_length": 20000, // Above maximum 10000
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject target_length > 10000"
        );
    }

    #[test]
    fn test_config_schema_quality_threshold_range() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid quality_threshold
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "quality_threshold": 0.75,
        }));
        assert!(
            schema.validate(&params).is_ok(),
            "Should accept quality_threshold 0.75"
        );

        // quality_threshold too low
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "quality_threshold": -0.1,
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject quality_threshold < 0.0"
        );

        // quality_threshold too high
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "quality_threshold": 1.5,
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject quality_threshold > 1.0"
        );
    }

    #[test]
    fn test_config_schema_max_iterations_range() {
        let template = ContentGenerationTemplate::new();
        let schema = template.config_schema();

        // Valid max_iterations
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "max_iterations": 5,
        }));
        assert!(
            schema.validate(&params).is_ok(),
            "Should accept max_iterations 5"
        );

        // max_iterations too low
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "max_iterations": 0,
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject max_iterations < 1"
        );

        // max_iterations too high
        let params = json_to_hashmap(json!({
            "topic": "Test topic",
            "max_iterations": 15, // Above maximum 10
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject max_iterations > 10"
        );
    }

    #[test]
    fn test_content_type_guidance_blog() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("blog");

        assert!(guidance.contains("blog post"));
        assert!(guidance.contains("hook"));
        assert!(guidance.contains("call-to-action"));
    }

    #[test]
    fn test_content_type_guidance_documentation() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("documentation");

        assert!(guidance.contains("technical documentation"));
        assert!(guidance.contains("prerequisites"));
        assert!(guidance.contains("Code examples"));
    }

    #[test]
    fn test_content_type_guidance_marketing() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("marketing");

        assert!(guidance.contains("marketing"));
        assert!(guidance.contains("headline"));
        assert!(guidance.contains("call-to-action"));
    }

    #[test]
    fn test_content_type_guidance_technical() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("technical");

        assert!(guidance.contains("technical content"));
        assert!(guidance.contains("specification"));
    }

    #[test]
    fn test_content_type_guidance_creative() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("creative");

        assert!(guidance.contains("creative content"));
        assert!(guidance.contains("narrative"));
    }

    #[test]
    fn test_content_type_guidance_general() {
        let template = ContentGenerationTemplate::new();
        let guidance = template.get_content_type_guidance("general");

        assert!(guidance.contains("general content"));
        assert!(guidance.contains("introduction"));
    }

    #[test]
    fn test_parse_model_with_provider() {
        let (provider, model) = ContentGenerationTemplate::parse_model("openai/gpt-4");

        assert_eq!(provider, "openai");
        assert_eq!(model, "gpt-4");
    }

    #[test]
    fn test_parse_model_without_provider() {
        let (provider, model) = ContentGenerationTemplate::parse_model("llama3.2:3b");

        assert_eq!(provider, "ollama"); // Default provider
        assert_eq!(model, "llama3.2:3b");
    }

    #[test]
    fn test_format_json_output() {
        let template = ContentGenerationTemplate::new();
        let plan = ContentPlan {
            outline: "1. Introduction\n2. Body\n3. Conclusion".to_string(),
            key_points: vec!["Point 1".to_string(), "Point 2".to_string()],
            target_audience: "Developers".to_string(),
        };
        let content = "Test content body".to_string();
        let quality = 0.85;
        let iterations = 2;
        let feedback = vec![];

        let json_output = template
            .format_json_output(
                "Test Topic",
                &plan,
                &content,
                quality,
                iterations,
                &feedback,
            )
            .unwrap();

        assert!(json_output.contains("\"content\":"));
        assert!(json_output.contains("Test content body"));
        assert!(json_output.contains("\"final_quality_score\":"));
        assert!(json_output.contains("\"iterations\":"));
        assert!(json_output.contains("\"outline\":"));
    }

    #[test]
    fn test_format_html_output() {
        let template = ContentGenerationTemplate::new();
        let plan = ContentPlan {
            outline: "1. Intro\n2. Body".to_string(),
            key_points: vec!["Key point".to_string()],
            target_audience: "General".to_string(),
        };
        let content = "Test HTML content".to_string();

        let html_output = template.format_html_output("Test Topic", &plan, &content, true);

        assert!(html_output.contains("<!DOCTYPE html>"));
        assert!(html_output.contains("<html"));
        assert!(html_output.contains("<body>"));
        assert!(html_output.contains("Test HTML content"));
        assert!(html_output.contains("<details>"));
        assert!(html_output.contains("Content Plan"));
    }

    #[test]
    fn test_format_html_output_without_outline() {
        let template = ContentGenerationTemplate::new();
        let plan = ContentPlan {
            outline: "Outline".to_string(),
            key_points: vec![],
            target_audience: "Readers".to_string(),
        };
        let content = "Just content".to_string();

        let html_output = template.format_html_output("Topic", &plan, &content, false);

        assert!(!html_output.contains("<details>"));
        assert!(html_output.contains("Just content"));
    }

    #[test]
    fn test_format_markdown_output() {
        let template = ContentGenerationTemplate::new();
        let plan = ContentPlan {
            outline: "1. Section A\n2. Section B".to_string(),
            key_points: vec!["Point A".to_string(), "Point B".to_string()],
            target_audience: "Readers".to_string(),
        };
        let content = "# Heading\n\nTest markdown content".to_string();

        let md_output =
            template.format_markdown_output("Test Topic", &plan, &content, true, 0.9, 2);

        assert!(md_output.contains("# Test Topic"));
        assert!(md_output.contains("## Content Plan"));
        assert!(md_output.contains("Section A"));
        assert!(md_output.contains("Test markdown content"));
        assert!(md_output.contains("Quality Score: 0.90"));
        assert!(md_output.contains("Iterations: 2"));
    }

    #[test]
    fn test_format_markdown_output_without_outline() {
        let template = ContentGenerationTemplate::new();
        let plan = ContentPlan {
            outline: "Outline".to_string(),
            key_points: vec![],
            target_audience: "Readers".to_string(),
        };
        let content = "Just content".to_string();

        let md_output = template.format_markdown_output("Topic", &plan, &content, false, 0.8, 1);

        assert!(!md_output.contains("## Content Plan"));
        assert!(md_output.contains("Just content"));
        assert!(md_output.contains("Quality Score: 0.80"));
    }

    #[test]
    fn test_format_text_output() {
        let template = ContentGenerationTemplate::new();
        let content = "Plain text content for testing".to_string();

        let text_output = template.format_text_output(&content);

        assert_eq!(text_output, "Plain text content for testing");
    }

    #[tokio::test]
    async fn test_estimate_cost_default_params() {
        let template = ContentGenerationTemplate::new();
        let params = TemplateParams::new(); // Defaults: max_iterations=3, no target_length

        let cost = template.estimate_cost(&params).await;

        // base_tokens = 600 + 1500 + 500 = 2600
        // review_tokens = 800 * 3 = 2400
        // improve_tokens = 1200 * 2 = 2400
        // Total = 2600 + 2400 + 2400 = 7400
        assert_eq!(cost.estimated_tokens, Some(7400));

        // base_duration = 5000 + 8000 + 3000 = 16000
        // review_duration = 4000 * 3 = 12000
        // improve_duration = 6000 * 2 = 12000
        // Total = 16000 + 12000 + 12000 = 40000
        assert_eq!(cost.estimated_duration_ms, Some(40000));
        assert_eq!(cost.confidence, 0.7);
    }

    #[tokio::test]
    async fn test_estimate_cost_with_target_length() {
        let template = ContentGenerationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("target_length", json!(2000));
        params.insert("max_iterations", json!(2));

        let cost = template.estimate_cost(&params).await;

        // base_tokens = 600 + 1500 + 500 = 2600
        // review_tokens = 800 * 2 = 1600
        // improve_tokens = 1200 * 1 = 1200
        // Total = 2600 + 1600 + 1200 = 5400
        // length_adjustment = 2000 * 1.3 = 2600
        // Final = max(5400, 2600) = 5400
        assert_eq!(cost.estimated_tokens, Some(5400));
    }

    #[tokio::test]
    async fn test_estimate_cost_high_target_length() {
        let template = ContentGenerationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("target_length", json!(8000));
        params.insert("max_iterations", json!(1));

        let cost = template.estimate_cost(&params).await;

        // base_tokens = 600 + 1500 + 500 = 2600
        // review_tokens = 800 * 1 = 800
        // improve_tokens = 1200 * 0 = 0
        // Total = 2600 + 800 + 0 = 3400
        // length_adjustment = 8000 * 1.3 = 10400
        // Final = max(3400, 10400) = 10400
        assert_eq!(cost.estimated_tokens, Some(10400));
    }

    #[tokio::test]
    async fn test_estimate_cost_max_iterations() {
        let template = ContentGenerationTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("max_iterations", json!(10)); // Maximum allowed

        let cost = template.estimate_cost(&params).await;

        // base_tokens = 600 + 1500 + 500 = 2600
        // review_tokens = 800 * 10 = 8000
        // improve_tokens = 1200 * 9 = 10800
        // Total = 2600 + 8000 + 10800 = 21400
        assert_eq!(cost.estimated_tokens, Some(21400));

        // base_duration = 5000 + 8000 + 3000 = 16000
        // review_duration = 4000 * 10 = 40000
        // improve_duration = 6000 * 9 = 54000
        // Total = 16000 + 40000 + 54000 = 110000
        assert_eq!(cost.estimated_duration_ms, Some(110000));
    }
}
