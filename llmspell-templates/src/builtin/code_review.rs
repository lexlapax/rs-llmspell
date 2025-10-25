//! Code Review Template
//!
//! Multi-aspect code analysis with 7 specialized reviewers:
//! 1. Security reviewer: vulnerabilities, CVEs, sensitive data exposure
//! 2. Quality reviewer: code smells, complexity, maintainability
//! 3. Performance reviewer: algorithm efficiency, memory usage
//! 4. Practices reviewer: SOLID principles, design patterns
//! 5. Dependency reviewer: outdated deps, circular dependencies
//! 6. Architecture reviewer: modularity, coupling, abstractions
//! 7. Documentation reviewer: API docs, comments, README completeness

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
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

/// Code Review Template
///
/// Automated multi-aspect code review with severity filtering and fix generation:
/// - 7 specialized review agents (security, quality, performance, practices, dependencies, architecture, docs)
/// - Language-specific analysis (Rust, Python, JavaScript, Go, TypeScript, Java)
/// - Severity-based filtering (critical, high, medium, low)
/// - Optional fix generation with before/after code snippets
/// - Multiple output formats (markdown, JSON, diff patches)
#[derive(Debug)]
pub struct CodeReviewTemplate {
    metadata: TemplateMetadata,
}

impl CodeReviewTemplate {
    /// Create a new Code Review template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "code-review".to_string(),
                name: "Code Review".to_string(),
                description: "Multi-aspect code review with 7 specialized AI reviewers. \
                             Analyzes security, quality, performance, best practices, dependencies, \
                             architecture, and documentation. Supports severity filtering and \
                             automatic fix generation for common issues."
                    .to_string(),
                category: TemplateCategory::Custom("Development".to_string()),
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["agents".to_string()],
                tags: vec![
                    "code-review".to_string(),
                    "quality".to_string(),
                    "security".to_string(),
                    "performance".to_string(),
                    "static-analysis".to_string(),
                ],
            },
        }
    }
}

impl Default for CodeReviewTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for CodeReviewTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // code_path (required)
            ParameterSchema::required(
                "code_path",
                "File or directory path to review",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                ..Default::default()
            }),
            // language (required enum)
            ParameterSchema::required(
                "language",
                "Programming language of the code being reviewed",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("rust"),
                    json!("python"),
                    json!("javascript"),
                    json!("typescript"),
                    json!("go"),
                    json!("java"),
                ]),
                ..Default::default()
            }),
            // aspects (optional array with default all)
            ParameterSchema::optional(
                "aspects",
                "Review aspects to analyze (security, quality, performance, practices, dependencies, architecture, docs)",
                ParameterType::Array,
                json!(["security", "quality", "performance", "practices", "dependencies", "architecture", "docs"]),
            ),
            // severity_filter (optional enum with default all)
            ParameterSchema::optional(
                "severity_filter",
                "Filter issues by severity level",
                ParameterType::String,
                json!("all"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("critical"),
                    json!("high"),
                    json!("medium"),
                    json!("low"),
                    json!("all"),
                ]),
                ..Default::default()
            }),
            // generate_fixes (optional boolean with default false)
            ParameterSchema::optional(
                "generate_fixes",
                "Generate fix suggestions for identified issues",
                ParameterType::Boolean,
                json!(false),
            ),
            // output_format (optional enum with default markdown)
            ParameterSchema::optional(
                "output_format",
                "Output format for review results",
                ParameterType::String,
                json!("markdown"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("markdown"),
                    json!("json"),
                    json!("text"),
                ]),
                ..Default::default()
            }),
            // model (optional)
            ParameterSchema::optional(
                "model",
                "LLM model to use for review agents (can be overridden per aspect via aspect_models)",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
            // temperature (optional with range)
            ParameterSchema::optional(
                "temperature",
                "Temperature for LLM agents (0.0-1.0, default 0.2 for consistency)",
                ParameterType::Number,
                json!(0.2),
            )
            .with_constraints(ParameterConstraints {
                min: Some(0.0),
                max: Some(1.0),
                ..Default::default()
            }),
        ])
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let code_path: String = params.get("code_path")?;
        let language: String = params.get("language")?;
        let aspects: Vec<String> = params.get_or(
            "aspects",
            vec![
                "security".to_string(),
                "quality".to_string(),
                "performance".to_string(),
                "practices".to_string(),
                "dependencies".to_string(),
                "architecture".to_string(),
                "docs".to_string(),
            ],
        );
        let severity_filter: String = params.get_or("severity_filter", "all".to_string());
        let generate_fixes: bool = params.get_or("generate_fixes", false);
        let output_format: String = params.get_or("output_format", "markdown".to_string());

        // Smart dual-path provider resolution (Task 13.5.7d)
        let provider_config = context.resolve_llm_config(&params)?;
        let model_str = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        info!(
            "Starting code review (path={}, language={}, aspects={}, severity_filter={}, model={})",
            code_path,
            language,
            aspects.len(),
            severity_filter,
            model_str
        );

        // Read code from path
        let code_content = self.read_code_file(&code_path)?;

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params.clone(),
        );

        // Execute reviews for each requested aspect
        let mut all_reviews = Vec::new();
        for aspect in &aspects {
            info!("Executing {} review...", aspect);
            let review_config = ReviewConfig {
                aspect,
                code: &code_content,
                language: &language,
                code_path: &code_path,
                provider_config: &provider_config,
            };
            let review = self.execute_aspect_review(&review_config, &context).await?;
            all_reviews.push(review);
            output.metrics.agents_invoked += 1;
        }

        // Aggregate and filter results by severity
        let aggregated = self.aggregate_reviews(&all_reviews, &severity_filter)?;

        // Generate fixes if requested
        let fixes = if generate_fixes && !aggregated.issues.is_empty() {
            info!("Generating fixes for {} issues...", aggregated.issues.len());
            let fix_result = self
                .generate_fixes(
                    &aggregated,
                    &code_content,
                    &language,
                    &provider_config,
                    &context,
                )
                .await?;
            output.metrics.agents_invoked += 1; // Fix generator agent
            Some(fix_result)
        } else {
            None
        };

        // Format output based on output_format
        let report = match output_format.as_str() {
            "json" => self.format_json_output(&aggregated, &fixes)?,
            "text" => self.format_text_output(&aggregated, &fixes),
            _ => self.format_markdown_output(&aggregated, &fixes, &code_path, &language),
        };

        // Save artifacts if output directory is provided
        if let Some(output_dir) = &context.output_dir {
            self.save_review_artifacts(output_dir, &aggregated, &fixes, &mut output)?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(report);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("code_path", json!(code_path));
        output.add_metric("language", json!(language));
        output.add_metric("aspects_analyzed", json!(aspects.len()));
        output.add_metric("total_issues", json!(aggregated.issues.len()));
        output.add_metric(
            "critical_issues",
            json!(aggregated.severity_counts.critical),
        );
        output.add_metric("high_issues", json!(aggregated.severity_counts.high));
        output.add_metric("medium_issues", json!(aggregated.severity_counts.medium));
        output.add_metric("low_issues", json!(aggregated.severity_counts.low));
        output.add_metric(
            "fixes_generated",
            json!(fixes.as_ref().map(|f| f.fixes.len()).unwrap_or(0)),
        );

        info!(
            "Code review complete (duration: {}ms, issues: {}, agents: {})",
            output.metrics.duration_ms,
            aggregated.issues.len(),
            output.metrics.agents_invoked
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        let aspects: Vec<String> = params.get_or(
            "aspects",
            vec![
                "security".to_string(),
                "quality".to_string(),
                "performance".to_string(),
                "practices".to_string(),
                "dependencies".to_string(),
                "architecture".to_string(),
                "docs".to_string(),
            ],
        );
        let generate_fixes: bool = params.get_or("generate_fixes", false);

        // Each review aspect: ~800 tokens (code + review output)
        let review_tokens = aspects.len() as u64 * 800;
        // Fix generation: ~1200 tokens (if enabled)
        let fix_tokens = if generate_fixes { 1200 } else { 0 };
        let estimated_tokens = review_tokens + fix_tokens;

        // Each review: ~4s
        // Fix generation: ~5s
        let review_duration = aspects.len() as u64 * 4000;
        let fix_duration = if generate_fixes { 5000 } else { 0 };
        let estimated_duration = review_duration + fix_duration;

        // Assuming $0.10 per 1M tokens (local LLM)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        CostEstimate::new(
            estimated_tokens,
            estimated_cost,
            estimated_duration,
            0.75, // High confidence
        )
    }
}

// Internal helper types
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReviewIssue {
    severity: String,
    aspect: String,
    line: Option<usize>,
    description: String,
    recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AspectReview {
    aspect: String,
    issues: Vec<ReviewIssue>,
    summary: String,
    metrics: Option<serde_json::Value>,
}

#[derive(Debug, Clone)]
struct AggregatedReview {
    issues: Vec<ReviewIssue>,
    summaries: Vec<(String, String)>, // (aspect, summary)
    severity_counts: SeverityCounts,
}

#[derive(Debug, Clone, Default)]
struct SeverityCounts {
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Fix {
    issue: String,
    original_code: Option<String>,
    fixed_code: Option<String>,
    explanation: String,
}

#[derive(Debug, Clone)]
struct FixResult {
    fixes: Vec<Fix>,
}

// Helper struct to reduce parameter count
struct ReviewConfig<'a> {
    aspect: &'a str,
    code: &'a str,
    language: &'a str,
    code_path: &'a str,
    provider_config: &'a llmspell_config::ProviderConfig,
}

impl CodeReviewTemplate {
    /// Read code from file path
    fn read_code_file(&self, path: &str) -> Result<String> {
        use std::fs;

        // For now, just read the file directly
        // TODO: Support directory scanning and multiple files
        fs::read_to_string(path).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to read code file '{}': {}", path, e))
        })
    }

    /// Execute review for a specific aspect
    async fn execute_aspect_review(
        &self,
        config: &ReviewConfig<'_>,
        context: &ExecutionContext,
    ) -> Result<AspectReview> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Get aspect-specific configuration
        let (agent_name, system_prompt) = self.get_aspect_config(config.aspect, config.language);

        // Extract model from provider config
        let model = config
            .provider_config
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
            (
                config.provider_config.provider_type.clone(),
                model.to_string(),
            )
        };

        // Create agent config
        let agent_config = AgentConfig {
            name: agent_name.clone(),
            description: format!("{} reviewer for {} code", config.aspect, config.language),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: config.provider_config.temperature.or(Some(0.2)),
                max_tokens: config.provider_config.max_tokens.or(Some(2000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 180, // 3 minutes per review
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create {} reviewer: {}", config.aspect, e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build review prompt
        let review_prompt = format!(
            "{}\n\nReview the following {} code:\n\n**File**: {}\n\n```{}\n{}\n```",
            system_prompt, config.language, config.code_path, config.language, config.code
        );

        // Execute the agent
        let agent_input = AgentInput::builder().text(review_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("{} review agent execution failed: {}", config.aspect, e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Parse JSON output
        let review_data: serde_json::Value = serde_json::from_str(&agent_output.text)
            .unwrap_or_else(|_| {
                // Fallback: treat as plain text summary
                json!({
                    "issues": [],
                    "summary": agent_output.text
                })
            });

        // Extract issues and summary
        let issues = review_data["issues"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|issue| ReviewIssue {
                        severity: issue["severity"].as_str().unwrap_or("medium").to_string(),
                        aspect: config.aspect.to_string(),
                        line: issue["line"].as_u64().map(|l| l as usize),
                        description: issue["description"]
                            .as_str()
                            .unwrap_or("No description")
                            .to_string(),
                        recommendation: issue["recommendation"]
                            .as_str()
                            .or_else(|| issue["suggestion"].as_str())
                            .unwrap_or("No recommendation")
                            .to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        let summary = review_data["summary"]
            .as_str()
            .unwrap_or("Review completed")
            .to_string();

        let metrics = review_data.get("metrics").cloned();

        Ok(AspectReview {
            aspect: config.aspect.to_string(),
            issues,
            summary,
            metrics,
        })
    }

    /// Get aspect-specific configuration (agent name and system prompt)
    fn get_aspect_config(&self, aspect: &str, language: &str) -> (String, String) {
        match aspect {
            "security" => (
                "security-reviewer".to_string(),
                format!(
                    "You are a security expert specializing in {} code vulnerability analysis.\n\
                     Review the provided code for security issues including:\n\
                     - Authentication and authorization flaws\n\
                     - Injection vulnerabilities (SQL, command, XSS)\n\
                     - Sensitive data exposure\n\
                     - Cryptographic weaknesses\n\
                     - Insecure configurations\n\
                     - Race conditions and concurrency issues\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"critical|high|medium|low\",\n\
                           \"line\": <line_number_if_identifiable>,\n\
                           \"description\": \"detailed description of the issue\",\n\
                           \"recommendation\": \"specific fix recommendation\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief overall security assessment\"\n\
                     }}",
                    language
                ),
            ),
            "quality" => (
                "quality-reviewer".to_string(),
                format!(
                    "You are a code quality expert focusing on maintainability and readability of {} code.\n\
                     Review the provided code for quality issues including:\n\
                     - Code complexity and readability\n\
                     - Error handling\n\
                     - Magic numbers and hardcoded values\n\
                     - Code duplication\n\
                     - Naming conventions\n\
                     - Documentation and comments\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"high|medium|low\",\n\
                           \"description\": \"detailed description\",\n\
                           \"recommendation\": \"improvement suggestion\"\n\
                         }}\n\
                       ],\n\
                       \"metrics\": {{\n\
                         \"complexity\": \"high|medium|low\",\n\
                         \"readability\": 7,\n\
                         \"maintainability\": 6\n\
                       }},\n\
                       \"summary\": \"brief quality assessment\"\n\
                     }}",
                    language
                ),
            ),
            "performance" => (
                "performance-reviewer".to_string(),
                format!(
                    "You are a performance optimization expert for {} code.\n\
                     Review the provided code for performance issues including:\n\
                     - Inefficient algorithms (O(n²) or worse)\n\
                     - Memory leaks and excessive allocations\n\
                     - Unnecessary loops and iterations\n\
                     - Database query optimization\n\
                     - Caching opportunities\n\
                     - Resource management\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"high|medium|low\",\n\
                           \"description\": \"detailed description\",\n\
                           \"recommendation\": \"suggested optimization\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief performance assessment\"\n\
                     }}",
                    language
                ),
            ),
            "practices" => (
                "practices-reviewer".to_string(),
                format!(
                    "You are a software engineering best practices expert for {}.\n\
                     Review the provided code for violations of best practices including:\n\
                     - SOLID principles violations\n\
                     - Design pattern misuse\n\
                     - Anti-patterns\n\
                     - Code organization issues\n\
                     - Testing considerations\n\
                     - Documentation standards\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"medium|low\",\n\
                           \"description\": \"principle or pattern violated\",\n\
                           \"recommendation\": \"how to improve\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief best practices assessment\"\n\
                     }}",
                    language
                ),
            ),
            "dependencies" => (
                "dependency-reviewer".to_string(),
                format!(
                    "You are a dependency and architecture expert for {} projects.\n\
                     Review the provided code for dependency issues including:\n\
                     - Outdated or vulnerable dependencies (if imports/uses are visible)\n\
                     - Unnecessary dependencies\n\
                     - Circular dependencies\n\
                     - Tight coupling\n\
                     - Missing abstractions\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"medium|low\",\n\
                           \"description\": \"detailed description\",\n\
                           \"recommendation\": \"suggested improvement\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief dependency assessment\"\n\
                     }}",
                    language
                ),
            ),
            "architecture" => (
                "architecture-reviewer".to_string(),
                format!(
                    "You are a software architecture expert for {} systems.\n\
                     Review the provided code for architectural issues including:\n\
                     - Modularity and separation of concerns\n\
                     - Abstraction levels\n\
                     - Component coupling and cohesion\n\
                     - Scalability considerations\n\
                     - Design pattern application\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"high|medium|low\",\n\
                           \"description\": \"detailed description\",\n\
                           \"recommendation\": \"architectural improvement\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief architectural assessment\"\n\
                     }}",
                    language
                ),
            ),
            "docs" => (
                "documentation-reviewer".to_string(),
                format!(
                    "You are a technical documentation expert for {} code.\n\
                     Review the provided code for documentation issues including:\n\
                     - API documentation completeness\n\
                     - Code comment quality and clarity\n\
                     - Function/method documentation\n\
                     - Complex logic explanations\n\
                     - README and usage examples (if present)\n\n\
                     Output a JSON object with this exact format:\n\
                     {{\n\
                       \"issues\": [\n\
                         {{\n\
                           \"severity\": \"medium|low\",\n\
                           \"description\": \"detailed description\",\n\
                           \"recommendation\": \"documentation improvement\"\n\
                         }}\n\
                       ],\n\
                       \"summary\": \"brief documentation assessment\"\n\
                     }}",
                    language
                ),
            ),
            _ => (
                format!("{}-reviewer", aspect),
                format!(
                    "You are a code review expert for {}. Review the code and provide findings in JSON format.",
                    language
                ),
            ),
        }
    }

    /// Aggregate reviews and filter by severity
    fn aggregate_reviews(
        &self,
        reviews: &[AspectReview],
        severity_filter: &str,
    ) -> Result<AggregatedReview> {
        let mut all_issues = Vec::new();
        let mut summaries = Vec::new();

        // Collect all issues from all reviews
        for review in reviews {
            all_issues.extend(review.issues.clone());
            summaries.push((review.aspect.clone(), review.summary.clone()));
        }

        // Filter by severity if not "all"
        let filtered_issues: Vec<ReviewIssue> = if severity_filter == "all" {
            all_issues
        } else {
            all_issues
                .into_iter()
                .filter(|issue| {
                    // Filter logic: include issue if it matches severity level or is more severe
                    match severity_filter {
                        "critical" => issue.severity == "critical",
                        "high" => issue.severity == "critical" || issue.severity == "high",
                        "medium" => {
                            issue.severity == "critical"
                                || issue.severity == "high"
                                || issue.severity == "medium"
                        }
                        "low" => true, // All severities
                        _ => true,
                    }
                })
                .collect()
        };

        // Count severities
        let mut severity_counts = SeverityCounts::default();
        for issue in &filtered_issues {
            match issue.severity.as_str() {
                "critical" => severity_counts.critical += 1,
                "high" => severity_counts.high += 1,
                "medium" => severity_counts.medium += 1,
                "low" => severity_counts.low += 1,
                _ => {}
            }
        }

        Ok(AggregatedReview {
            issues: filtered_issues,
            summaries,
            severity_counts,
        })
    }

    /// Generate fixes for identified issues
    async fn generate_fixes(
        &self,
        aggregated: &AggregatedReview,
        code: &str,
        language: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
    ) -> Result<FixResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model string
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create fix generator agent config
        let agent_config = AgentConfig {
            name: "fix-generator".to_string(),
            description: format!("Code fixing expert for {}", language),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.4)), // Balanced creativity
                max_tokens: provider_config.max_tokens.or(Some(3000)),
                settings: serde_json::Map::new(),
            }),
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits {
                max_execution_time_secs: 240, // 4 minutes for fix generation
                max_memory_mb: 512,
                max_tool_calls: 0,
                max_recursion_depth: 1,
            },
        };

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create fix generator: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build fix generation prompt
        let issues_summary = aggregated
            .issues
            .iter()
            .take(10) // Limit to top 10 issues to avoid token limits
            .map(|issue| {
                format!(
                    "- [{}] {}: {}",
                    issue.severity, issue.aspect, issue.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        let fix_prompt = format!(
            "You are a code fixing expert for {}. Generate specific fixes for the following issues:\n\n\
             **Issues Found**:\n{}\n\n\
             **Original Code**:\n```{}\n{}\n```\n\n\
             Output a JSON object with this exact format:\n\
             {{\n\
               \"fixes\": [\n\
                 {{\n\
                   \"issue\": \"issue being fixed\",\n\
                   \"original_code\": \"problematic snippet (optional)\",\n\
                   \"fixed_code\": \"corrected snippet (optional)\",\n\
                   \"explanation\": \"what was changed and why\"\n\
                 }}\n\
               ]\n\
             }}",
            language, issues_summary, language, code
        );

        // Execute the agent
        let agent_input = AgentInput::builder().text(fix_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Fix generation failed: {}", e);
                TemplateError::ExecutionFailed(format!("Fix generation failed: {}", e))
            })?;

        // Parse JSON output
        let fix_data: serde_json::Value =
            serde_json::from_str(&agent_output.text).unwrap_or_else(|_| {
                json!({
                    "fixes": []
                })
            });

        let fixes = fix_data["fixes"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|fix| Fix {
                        issue: fix["issue"].as_str().unwrap_or("Unknown").to_string(),
                        original_code: fix["original_code"].as_str().map(|s| s.to_string()),
                        fixed_code: fix["fixed_code"].as_str().map(|s| s.to_string()),
                        explanation: fix["explanation"]
                            .as_str()
                            .unwrap_or("No explanation")
                            .to_string(),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(FixResult { fixes })
    }

    /// Format output as JSON
    fn format_json_output(
        &self,
        aggregated: &AggregatedReview,
        fixes: &Option<FixResult>,
    ) -> Result<String> {
        let output_json = json!({
            "total_issues": aggregated.issues.len(),
            "severity_counts": {
                "critical": aggregated.severity_counts.critical,
                "high": aggregated.severity_counts.high,
                "medium": aggregated.severity_counts.medium,
                "low": aggregated.severity_counts.low,
            },
            "issues": aggregated.issues,
            "summaries": aggregated.summaries.iter().map(|(aspect, summary)| {
                json!({
                    "aspect": aspect,
                    "summary": summary
                })
            }).collect::<Vec<_>>(),
            "fixes": fixes.as_ref().map(|f| &f.fixes).unwrap_or(&vec![]),
        });

        serde_json::to_string_pretty(&output_json).map_err(|e| {
            TemplateError::ExecutionFailed(format!("JSON serialization failed: {}", e))
        })
    }

    /// Format output as plain text
    fn format_text_output(
        &self,
        aggregated: &AggregatedReview,
        fixes: &Option<FixResult>,
    ) -> String {
        let mut output = String::new();

        output.push_str("=== CODE REVIEW RESULTS ===\n\n");
        output.push_str(&format!("Total Issues: {}\n", aggregated.issues.len()));
        output.push_str(&format!(
            "  Critical: {}\n",
            aggregated.severity_counts.critical
        ));
        output.push_str(&format!("  High: {}\n", aggregated.severity_counts.high));
        output.push_str(&format!(
            "  Medium: {}\n",
            aggregated.severity_counts.medium
        ));
        output.push_str(&format!("  Low: {}\n\n", aggregated.severity_counts.low));

        output.push_str("=== ISSUES BY ASPECT ===\n\n");
        for issue in &aggregated.issues {
            output.push_str(&format!(
                "[{}] {} ({})\n",
                issue.severity.to_uppercase(),
                issue.aspect,
                issue
                    .line
                    .map(|l| format!("line {}", l))
                    .unwrap_or_else(|| "location unknown".to_string())
            ));
            output.push_str(&format!("  Description: {}\n", issue.description));
            output.push_str(&format!("  Recommendation: {}\n\n", issue.recommendation));
        }

        if let Some(fix_result) = fixes {
            if !fix_result.fixes.is_empty() {
                output.push_str("=== SUGGESTED FIXES ===\n\n");
                for fix in &fix_result.fixes {
                    output.push_str(&format!("Issue: {}\n", fix.issue));
                    output.push_str(&format!("Explanation: {}\n\n", fix.explanation));
                }
            }
        }

        output
    }

    /// Format output as markdown
    fn format_markdown_output(
        &self,
        aggregated: &AggregatedReview,
        fixes: &Option<FixResult>,
        code_path: &str,
        language: &str,
    ) -> String {
        let mut output = String::new();

        output.push_str("# Code Review Report\n\n");
        output.push_str(&format!("**File**: {}\n", code_path));
        output.push_str(&format!("**Language**: {}\n\n", language));
        output.push_str("---\n\n");

        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "- **Total Issues**: {}\n",
            aggregated.issues.len()
        ));
        output.push_str(&format!(
            "  - Critical: {}\n",
            aggregated.severity_counts.critical
        ));
        output.push_str(&format!("  - High: {}\n", aggregated.severity_counts.high));
        output.push_str(&format!(
            "  - Medium: {}\n",
            aggregated.severity_counts.medium
        ));
        output.push_str(&format!("  - Low: {}\n\n", aggregated.severity_counts.low));

        // Aspect summaries
        output.push_str("## Review Summaries by Aspect\n\n");
        for (aspect, summary) in &aggregated.summaries {
            output.push_str(&format!("### {}\n\n{}\n\n", aspect, summary));
        }

        // Detailed findings
        output.push_str("## Detailed Findings\n\n");
        for issue in &aggregated.issues {
            output.push_str(&format!(
                "### [{} - {}] {}\n\n",
                issue.severity.to_uppercase(),
                issue.aspect,
                if let Some(line) = issue.line {
                    format!("Line {}", line)
                } else {
                    "Location unknown".to_string()
                }
            ));
            output.push_str(&format!("**Description**: {}\n\n", issue.description));
            output.push_str(&format!("**Recommendation**: {}\n\n", issue.recommendation));
            output.push_str("---\n\n");
        }

        // Suggested fixes
        if let Some(fix_result) = fixes {
            if !fix_result.fixes.is_empty() {
                output.push_str("## Suggested Fixes\n\n");
                for (idx, fix) in fix_result.fixes.iter().enumerate() {
                    output.push_str(&format!("### Fix {}: {}\n\n", idx + 1, fix.issue));
                    output.push_str(&format!("**Explanation**: {}\n\n", fix.explanation));
                    if let Some(original) = &fix.original_code {
                        output.push_str(&format!(
                            "**Original**:\n```{}\n{}\n```\n\n",
                            language, original
                        ));
                    }
                    if let Some(fixed) = &fix.fixed_code {
                        output
                            .push_str(&format!("**Fixed**:\n```{}\n{}\n```\n\n", language, fixed));
                    }
                    output.push_str("---\n\n");
                }
            }
        }

        output.push_str("---\n\n");
        output.push_str("*Generated by LLMSpell Code Review Template*\n");

        output
    }

    /// Save review artifacts to output directory
    fn save_review_artifacts(
        &self,
        output_dir: &std::path::Path,
        aggregated: &AggregatedReview,
        fixes: &Option<FixResult>,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save findings as JSON
        let findings_path = output_dir.join("review-findings.json");
        let findings_json = self.format_json_output(aggregated, fixes)?;
        fs::write(&findings_path, &findings_json).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write findings JSON: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            findings_path.to_string_lossy().to_string(),
            findings_json,
            "application/json".to_string(),
        ));

        // Save report as markdown
        let report_path = output_dir.join("review-report.md");
        let report_md = self.format_markdown_output(aggregated, fixes, "code", "unknown");
        fs::write(&report_path, &report_md).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write markdown report: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            report_path.to_string_lossy().to_string(),
            report_md,
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
        let template = CodeReviewTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "code-review");
        assert_eq!(metadata.name, "Code Review");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(
            metadata.category,
            TemplateCategory::Custom("Development".to_string())
        );
        assert!(metadata.tags.contains(&"code-review".to_string()));
        assert!(metadata.tags.contains(&"security".to_string()));
        assert!(metadata.tags.contains(&"quality".to_string()));
    }

    #[test]
    fn test_config_schema_required_parameters() {
        let template = CodeReviewTemplate::new();
        let schema = template.config_schema();

        // code_path is required
        let params = json_to_hashmap(json!({
            "language": "rust",
        }));
        let result = schema.validate(&params);
        assert!(
            result.is_err(),
            "Should fail without required code_path parameter"
        );

        // language is required
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
        }));
        let result = schema.validate(&params);
        assert!(
            result.is_err(),
            "Should fail without required language parameter"
        );
    }

    #[test]
    fn test_config_schema_language_validation() {
        let template = CodeReviewTemplate::new();
        let schema = template.config_schema();

        // Valid language
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
            "language": "rust",
        }));
        assert!(
            schema.validate(&params).is_ok(),
            "Should accept valid language 'rust'"
        );

        // Invalid language
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.cpp",
            "language": "cobol", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid language 'cobol'"
        );
    }

    #[test]
    fn test_config_schema_severity_filter_validation() {
        let template = CodeReviewTemplate::new();
        let schema = template.config_schema();

        // Valid severity filters
        for severity in &["critical", "high", "medium", "low", "all"] {
            let params = json_to_hashmap(json!({
                "code_path": "/tmp/test.rs",
                "language": "rust",
                "severity_filter": severity,
            }));
            assert!(
                schema.validate(&params).is_ok(),
                "Should accept valid severity_filter '{}'",
                severity
            );
        }

        // Invalid severity filter
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
            "language": "rust",
            "severity_filter": "extreme", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid severity_filter 'extreme'"
        );
    }

    #[test]
    fn test_config_schema_output_format_validation() {
        let template = CodeReviewTemplate::new();
        let schema = template.config_schema();

        // Valid output formats
        for format in &["markdown", "json", "text"] {
            let params = json_to_hashmap(json!({
                "code_path": "/tmp/test.rs",
                "language": "rust",
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
            "code_path": "/tmp/test.rs",
            "language": "rust",
            "output_format": "xml", // Not in allowed list
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject invalid output_format 'xml'"
        );
    }

    #[test]
    fn test_config_schema_temperature_range() {
        let template = CodeReviewTemplate::new();
        let schema = template.config_schema();

        // Valid temperature
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
            "language": "rust",
            "temperature": 0.5,
        }));
        assert!(
            schema.validate(&params).is_ok(),
            "Should accept temperature 0.5"
        );

        // Temperature too low
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
            "language": "rust",
            "temperature": -0.1,
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject temperature < 0.0"
        );

        // Temperature too high
        let params = json_to_hashmap(json!({
            "code_path": "/tmp/test.rs",
            "language": "rust",
            "temperature": 1.5,
        }));
        assert!(
            schema.validate(&params).is_err(),
            "Should reject temperature > 1.0"
        );
    }

    #[test]
    fn test_aspect_config_security() {
        let template = CodeReviewTemplate::new();
        let (name, prompt) = template.get_aspect_config("security", "rust");

        assert_eq!(name, "security-reviewer");
        assert!(prompt.contains("security expert"));
        assert!(prompt.contains("vulnerability"));
        assert!(prompt.contains("Authentication"));
        assert!(prompt.contains("Injection"));
    }

    #[test]
    fn test_aspect_config_quality() {
        let template = CodeReviewTemplate::new();
        let (name, prompt) = template.get_aspect_config("quality", "python");

        assert_eq!(name, "quality-reviewer");
        assert!(prompt.contains("quality expert"));
        assert!(prompt.contains("maintainability"));
        assert!(prompt.contains("readability"));
    }

    #[test]
    fn test_aspect_config_performance() {
        let template = CodeReviewTemplate::new();
        let (name, prompt) = template.get_aspect_config("performance", "javascript");

        assert_eq!(name, "performance-reviewer");
        assert!(prompt.contains("performance optimization"));
        assert!(prompt.contains("algorithm"));
        assert!(prompt.contains("Memory")); // Capital M in the prompt
    }

    #[test]
    fn test_severity_filtering_all() {
        let template = CodeReviewTemplate::new();
        let reviews = vec![AspectReview {
            aspect: "security".to_string(),
            issues: vec![
                ReviewIssue {
                    severity: "critical".to_string(),
                    aspect: "security".to_string(),
                    line: Some(10),
                    description: "SQL injection".to_string(),
                    recommendation: "Use prepared statements".to_string(),
                },
                ReviewIssue {
                    severity: "low".to_string(),
                    aspect: "quality".to_string(),
                    line: Some(20),
                    description: "Magic number".to_string(),
                    recommendation: "Use named constant".to_string(),
                },
            ],
            summary: "Test summary".to_string(),
            metrics: None,
        }];

        let aggregated = template.aggregate_reviews(&reviews, "all").unwrap();
        assert_eq!(aggregated.issues.len(), 2);
        assert_eq!(aggregated.severity_counts.critical, 1);
        assert_eq!(aggregated.severity_counts.low, 1);
    }

    #[test]
    fn test_severity_filtering_critical_only() {
        let template = CodeReviewTemplate::new();
        let reviews = vec![AspectReview {
            aspect: "security".to_string(),
            issues: vec![
                ReviewIssue {
                    severity: "critical".to_string(),
                    aspect: "security".to_string(),
                    line: Some(10),
                    description: "SQL injection".to_string(),
                    recommendation: "Use prepared statements".to_string(),
                },
                ReviewIssue {
                    severity: "high".to_string(),
                    aspect: "security".to_string(),
                    line: Some(15),
                    description: "XSS vulnerability".to_string(),
                    recommendation: "Sanitize input".to_string(),
                },
                ReviewIssue {
                    severity: "low".to_string(),
                    aspect: "quality".to_string(),
                    line: Some(20),
                    description: "Magic number".to_string(),
                    recommendation: "Use named constant".to_string(),
                },
            ],
            summary: "Test summary".to_string(),
            metrics: None,
        }];

        let aggregated = template.aggregate_reviews(&reviews, "critical").unwrap();
        assert_eq!(aggregated.issues.len(), 1);
        assert_eq!(aggregated.severity_counts.critical, 1);
        assert_eq!(aggregated.severity_counts.high, 0);
        assert_eq!(aggregated.severity_counts.low, 0);
    }

    #[test]
    fn test_severity_filtering_high() {
        let template = CodeReviewTemplate::new();
        let reviews = vec![AspectReview {
            aspect: "security".to_string(),
            issues: vec![
                ReviewIssue {
                    severity: "critical".to_string(),
                    aspect: "security".to_string(),
                    line: Some(10),
                    description: "SQL injection".to_string(),
                    recommendation: "Use prepared statements".to_string(),
                },
                ReviewIssue {
                    severity: "high".to_string(),
                    aspect: "security".to_string(),
                    line: Some(15),
                    description: "XSS vulnerability".to_string(),
                    recommendation: "Sanitize input".to_string(),
                },
                ReviewIssue {
                    severity: "medium".to_string(),
                    aspect: "quality".to_string(),
                    line: Some(20),
                    description: "Code duplication".to_string(),
                    recommendation: "Extract function".to_string(),
                },
            ],
            summary: "Test summary".to_string(),
            metrics: None,
        }];

        let aggregated = template.aggregate_reviews(&reviews, "high").unwrap();
        assert_eq!(aggregated.issues.len(), 2); // critical + high
        assert_eq!(aggregated.severity_counts.critical, 1);
        assert_eq!(aggregated.severity_counts.high, 1);
        assert_eq!(aggregated.severity_counts.medium, 0);
    }

    #[test]
    fn test_format_json_output() {
        let template = CodeReviewTemplate::new();
        let aggregated = AggregatedReview {
            issues: vec![ReviewIssue {
                severity: "high".to_string(),
                aspect: "security".to_string(),
                line: Some(42),
                description: "Potential SQL injection".to_string(),
                recommendation: "Use parameterized queries".to_string(),
            }],
            summaries: vec![("security".to_string(), "Found 1 security issue".to_string())],
            severity_counts: SeverityCounts {
                critical: 0,
                high: 1,
                medium: 0,
                low: 0,
            },
        };

        let json_output = template.format_json_output(&aggregated, &None).unwrap();
        assert!(json_output.contains("\"total_issues\": 1"));
        assert!(json_output.contains("\"high\": 1"));
        assert!(json_output.contains("SQL injection"));
    }

    #[test]
    fn test_format_text_output() {
        let template = CodeReviewTemplate::new();
        let aggregated = AggregatedReview {
            issues: vec![ReviewIssue {
                severity: "medium".to_string(),
                aspect: "quality".to_string(),
                line: Some(15),
                description: "Code complexity too high".to_string(),
                recommendation: "Refactor into smaller functions".to_string(),
            }],
            summaries: vec![(
                "quality".to_string(),
                "Code quality needs improvement".to_string(),
            )],
            severity_counts: SeverityCounts {
                critical: 0,
                high: 0,
                medium: 1,
                low: 0,
            },
        };

        let text_output = template.format_text_output(&aggregated, &None);
        assert!(text_output.contains("CODE REVIEW RESULTS"));
        assert!(text_output.contains("Total Issues: 1"));
        assert!(text_output.contains("Medium: 1"));
        assert!(text_output.contains("[MEDIUM]"));
        assert!(text_output.contains("Code complexity"));
    }

    #[test]
    fn test_format_markdown_output() {
        let template = CodeReviewTemplate::new();
        let aggregated = AggregatedReview {
            issues: vec![ReviewIssue {
                severity: "low".to_string(),
                aspect: "docs".to_string(),
                line: None,
                description: "Missing function documentation".to_string(),
                recommendation: "Add docstring".to_string(),
            }],
            summaries: vec![(
                "docs".to_string(),
                "Documentation coverage is low".to_string(),
            )],
            severity_counts: SeverityCounts {
                critical: 0,
                high: 0,
                medium: 0,
                low: 1,
            },
        };

        let md_output = template.format_markdown_output(&aggregated, &None, "test.py", "python");
        assert!(md_output.contains("# Code Review Report"));
        assert!(md_output.contains("**File**: test.py"));
        assert!(md_output.contains("**Language**: python"));
        assert!(md_output.contains("Low: 1"));
        assert!(md_output.contains("### [LOW - docs]"));
        assert!(md_output.contains("Missing function documentation"));
    }

    #[tokio::test]
    async fn test_estimate_cost_all_aspects() {
        let template = CodeReviewTemplate::new();
        let params = TemplateParams::new(); // Use defaults (all 7 aspects, no fixes)

        let cost = template.estimate_cost(&params).await;
        assert_eq!(cost.estimated_tokens, Some(7 * 800)); // 7 aspects * 800 tokens
        assert_eq!(cost.estimated_duration_ms, Some(7 * 4000)); // ~4s per aspect
        assert_eq!(cost.confidence, 0.75);
    }

    #[tokio::test]
    async fn test_estimate_cost_with_fixes() {
        let template = CodeReviewTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("generate_fixes", json!(true));

        let cost = template.estimate_cost(&params).await;
        // 7 aspects * 800 + 1200 fix tokens
        assert_eq!(cost.estimated_tokens, Some(7 * 800 + 1200));
        assert_eq!(cost.estimated_duration_ms, Some(7 * 4000 + 5000));
    }

    #[tokio::test]
    async fn test_estimate_cost_subset_aspects() {
        let template = CodeReviewTemplate::new();
        let mut params = TemplateParams::new();
        params.insert("aspects", json!(["security", "quality"])); // Only 2 aspects

        let cost = template.estimate_cost(&params).await;
        assert_eq!(cost.estimated_tokens, Some(2 * 800)); // 2 aspects * 800 tokens
        assert_eq!(cost.estimated_duration_ms, Some(2 * 4000));
    }
}
