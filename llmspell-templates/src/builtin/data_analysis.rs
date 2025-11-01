//! Data Analysis Template
//!
//! Sequential agent workflow for data analysis and visualization:
//! 1. Load data from file
//! 2. Analyzer agent: statistical analysis
//! 3. Visualizer agent: chart generation
//! 4. Save report + chart artifacts

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
use serde_json::json;
use std::time::Instant;
use tracing::{debug, info, warn};

/// Data Analysis Template
///
/// Statistical analysis and visualization workflow:
/// - Loads data from CSV/JSON files
/// - Runs statistical analysis with analyzer agent
/// - Generates charts with visualizer agent
/// - Produces comprehensive analysis report
#[derive(Debug)]
pub struct DataAnalysisTemplate {
    metadata: TemplateMetadata,
}

impl DataAnalysisTemplate {
    /// Create a new Data Analysis template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "data-analysis".to_string(),
                name: "Data Analysis".to_string(),
                description: "Automated data analysis with statistical insights and visualization. \
                             Loads data files, performs analysis with AI agents, and generates \
                             charts and reports. Supports multiple analysis types and chart formats."
                    .to_string(),
                category: TemplateCategory::Analysis,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec!["data-loader".to_string(), "stats".to_string()],
                tags: vec![
                    "analysis".to_string(),
                    "statistics".to_string(),
                    "visualization".to_string(),
                    "data".to_string(),
                    "charts".to_string(),
                ],
            },
        }
    }
}

impl Default for DataAnalysisTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for DataAnalysisTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        let mut params = vec![
            // data_file (required)
            ParameterSchema::required(
                "data_file",
                "Path to data file (CSV, JSON, or Excel format)",
                ParameterType::String,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                ..Default::default()
            }),
            // analysis_type (optional enum with default)
            ParameterSchema::optional(
                "analysis_type",
                "Type of statistical analysis to perform",
                ParameterType::String,
                json!("descriptive"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("descriptive"),
                    json!("correlation"),
                    json!("regression"),
                    json!("timeseries"),
                    json!("clustering"),
                ]),
                ..Default::default()
            }),
            // chart_type (optional enum with default)
            ParameterSchema::optional(
                "chart_type",
                "Type of chart to generate for visualization",
                ParameterType::String,
                json!("bar"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("bar"),
                    json!("line"),
                    json!("scatter"),
                    json!("histogram"),
                    json!("heatmap"),
                    json!("box"),
                ]),
                ..Default::default()
            }),
            // model (optional - for agent execution)
            ParameterSchema::optional(
                "model",
                "LLM model to use for analysis agents",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
        ];

        // Add memory parameters (Task 13.11.1)
        params.extend(memory_parameters());

        // Add provider parameters (Task 13.5.7d)
        params.extend(provider_parameters());

        debug!(
            "DataAnalysis: Generated config schema with {} parameters",
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
        let data_file: String = params.get("data_file")?;
        let analysis_type: String = params.get_or("analysis_type", "descriptive".to_string());
        let chart_type: String = params.get_or("chart_type", "bar".to_string());

        // Smart dual-path provider resolution (Task 13.5.7d)
        let provider_config = context.resolve_llm_config(&params)?;
        let model_str = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        info!(
            "Starting data analysis (file={}, analysis={}, chart={}, model={})",
            data_file, analysis_type, chart_type, model_str
        );

        // Extract memory parameters (Task 13.11.2)
        let session_id: Option<String> = params.get_optional("session_id").unwrap_or(None);
        let memory_enabled: bool = params.get_or("memory_enabled", true);
        let context_budget: i64 = params.get_or("context_budget", 2000);

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Phase 1: Load data
        info!("Phase 1: Loading data from file...");
        let dataset = self.load_data(&data_file, &context).await?;
        output.metrics.tools_invoked += 1; // data-loader tool

        // Phase 2: Statistical analysis with analyzer agent
        info!("Phase 2: Running statistical analysis...");
        let analysis_result = self
            .run_analysis(
                &dataset,
                &analysis_type,
                &provider_config,
                &context,
                session_id.as_deref(),
                memory_enabled,
                context_budget,
            )
            .await?;
        output.metrics.agents_invoked += 1; // analyzer agent

        // Phase 3: Visualization with visualizer agent
        info!("Phase 3: Generating visualizations...");
        let chart_result = self
            .generate_chart(
                &dataset,
                &analysis_result,
                &chart_type,
                &provider_config,
                &context,
                session_id.as_deref(),
                memory_enabled,
                context_budget,
            )
            .await?;
        output.metrics.agents_invoked += 1; // visualizer agent

        // Generate final report
        let report = self.format_report(
            &data_file,
            &analysis_type,
            &chart_type,
            &analysis_result,
            &chart_result,
        );

        // Save artifacts
        if let Some(output_dir) = &context.output_dir {
            self.save_artifacts(output_dir, &report, &chart_result.chart_data, &mut output)?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(report);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("dataset_rows", json!(dataset.rows));
        output.add_metric("dataset_columns", json!(dataset.columns));
        output.add_metric("analysis_type", json!(analysis_type));
        output.add_metric("chart_type", json!(chart_type));

        info!(
            "Data analysis complete (duration: {}ms, rows: {})",
            output.metrics.duration_ms, dataset.rows
        );

        // Store in memory if enabled (Task 13.11.3)
        if memory_enabled && session_id.is_some() && context.memory_manager().is_some() {
            let memory_mgr = context.memory_manager().unwrap();
            let input_summary = format!(
                "Analyze dataset {} with {} analysis",
                data_file, analysis_type
            );
            let output_summary = format!(
                "Analyzed {} rows × {} columns, generated {} chart",
                dataset.rows, dataset.columns, chart_type
            );

            crate::context::store_template_execution(
                &memory_mgr,
                session_id.as_ref().unwrap(),
                &self.metadata.id,
                &input_summary,
                &output_summary,
                json!({
                    "analysis_type": analysis_type,
                    "chart_type": chart_type,
                    "dataset_rows": dataset.rows,
                    "dataset_columns": dataset.columns,
                }),
            )
            .await
            .ok(); // Don't fail execution if storage fails
        }

        Ok(output)
    }

    async fn estimate_cost(&self, _params: &TemplateParams) -> CostEstimate {
        // Estimate based on typical data analysis workflow
        // Rough estimates:
        // - Data loading: minimal tokens
        // - Statistical analysis: ~1500 tokens (depends on dataset size)
        // - Visualization: ~1000 tokens
        let estimated_tokens = 1500 + 1000;

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Data loading: ~1s
        // Analysis: ~5s (agent + computation)
        // Visualization: ~3s (agent + rendering)
        let estimated_duration = 1000 + 5000 + 3000;

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.6, // Medium confidence - varies with dataset size
        )
    }
}

impl DataAnalysisTemplate {
    /// Phase 1: Load data from file
    async fn load_data(&self, file_path: &str, _context: &ExecutionContext) -> Result<DataSet> {
        use std::fs;
        use std::path::Path;

        info!("Loading data from file: {}", file_path);

        // Check if file exists
        if !Path::new(file_path).exists() {
            return Err(TemplateError::ExecutionFailed(format!(
                "Data file not found: {}",
                file_path
            )));
        }

        // Determine file format from extension
        let extension = Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("txt");

        // Read file contents
        let contents = fs::read_to_string(file_path).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to read file {}: {}", file_path, e))
        })?;

        // Parse based on format
        let (rows, columns, preview) = match extension {
            "csv" | "tsv" => self.parse_csv_data(&contents, extension)?,
            "json" => self.parse_json_data(&contents)?,
            _ => {
                // Fallback: treat as plain text data
                warn!("Unknown format '{}', treating as plain text", extension);
                let lines: Vec<&str> = contents.lines().collect();
                let preview = if lines.len() > 10 {
                    format!(
                        "# Data Preview (text)\n\n{}\n...\n({} more lines)",
                        lines[..10].join("\n"),
                        lines.len() - 10
                    )
                } else {
                    format!("# Data Preview (text)\n\n{}", contents)
                };
                (lines.len(), 1, preview)
            }
        };

        info!(
            "Data loaded successfully: {} rows x {} columns ({})",
            rows, columns, extension
        );

        Ok(DataSet {
            rows,
            columns,
            format: extension.to_string(),
            preview,
        })
    }

    /// Parse CSV/TSV data (simple implementation without external crates)
    fn parse_csv_data(&self, contents: &str, format: &str) -> Result<(usize, usize, String)> {
        let delimiter = if format == "tsv" { '\t' } else { ',' };

        let lines: Vec<&str> = contents.lines().filter(|l| !l.trim().is_empty()).collect();

        if lines.is_empty() {
            return Err(TemplateError::ExecutionFailed("Empty CSV file".to_string()));
        }

        // Parse header (first line)
        let header: Vec<&str> = lines[0].split(delimiter).map(|s| s.trim()).collect();
        let columns = header.len();

        // Count data rows (excluding header)
        let rows = lines.len() - 1;

        // Create preview with header + first 5 data rows
        let preview_lines = if lines.len() > 6 {
            &lines[..6]
        } else {
            &lines[..]
        };

        let preview = format!(
            "# Data Preview ({})\n\n\
             Columns: {}\n\
             Rows: {}\n\n\
             {}\n\
             {}",
            format,
            header.join(", "),
            rows,
            preview_lines.join("\n"),
            if lines.len() > 6 {
                format!("... ({} more rows)", rows - 5)
            } else {
                String::new()
            }
        );

        Ok((rows, columns, preview))
    }

    /// Parse JSON data (simple implementation)
    fn parse_json_data(&self, contents: &str) -> Result<(usize, usize, String)> {
        // Try to parse as JSON value
        let json_value: serde_json::Value = serde_json::from_str(contents)
            .map_err(|e| TemplateError::ExecutionFailed(format!("Failed to parse JSON: {}", e)))?;

        let (rows, columns) = match &json_value {
            serde_json::Value::Array(arr) => {
                // Array of objects
                let row_count = arr.len();
                let col_count = if let Some(serde_json::Value::Object(obj)) = arr.first() {
                    obj.len()
                } else {
                    1
                };
                (row_count, col_count)
            }
            serde_json::Value::Object(obj) => {
                // Single object
                (1, obj.len())
            }
            _ => (1, 1),
        };

        // Create preview
        let preview_text =
            serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| contents.to_string());

        let preview = if preview_text.len() > 500 {
            format!(
                "# Data Preview (json)\n\n{}...\n(truncated)",
                &preview_text[..500]
            )
        } else {
            format!("# Data Preview (json)\n\n{}", preview_text)
        };

        Ok((rows, columns, preview))
    }

    /// Phase 2: Run statistical analysis with analyzer agent
    #[allow(clippy::too_many_arguments)]
    async fn run_analysis(
        &self,
        dataset: &DataSet,
        analysis_type: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
        session_id: Option<&str>,
        memory_enabled: bool,
        context_budget: i64,
    ) -> Result<AnalysisResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!(
            "Creating statistical analysis agent (type: {})",
            analysis_type
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

        // Create agent config for statistical analysis
        let agent_config = AgentConfig {
            name: "data-analyst-agent".to_string(),
            description: format!("Statistical analysis agent for {} analysis", analysis_type),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.4)), // Balanced for analytical reasoning
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

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create analysis agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build analysis instructions based on type
        let analysis_instructions = match analysis_type {
            "descriptive" => {
                "Provide descriptive statistics including:\n\
                - Central tendency measures (mean, median, mode)\n\
                - Dispersion measures (std dev, variance, range)\n\
                - Distribution characteristics\n\
                - Data quality observations (outliers, missing values)"
            }
            "correlation" => {
                "Analyze correlations between variables:\n\
                - Correlation coefficients between all variable pairs\n\
                - Strength and direction of relationships\n\
                - Statistical significance\n\
                - Interpretation of findings"
            }
            "regression" => {
                "Perform regression analysis:\n\
                - Model fit statistics (R², adjusted R²)\n\
                - Coefficients and their significance\n\
                - Residual analysis\n\
                - Predictive insights"
            }
            "timeseries" => {
                "Analyze time series patterns:\n\
                - Trend identification\n\
                - Seasonality detection\n\
                - Autocorrelation patterns\n\
                - Forecasting insights"
            }
            "clustering" => {
                "Perform clustering analysis:\n\
                - Optimal number of clusters\n\
                - Cluster characteristics\n\
                - Quality metrics (silhouette score, inertia)\n\
                - Pattern interpretation"
            }
            _ => "Perform comprehensive data analysis with appropriate statistical methods",
        };

        // Build the analysis prompt with data context
        let analysis_prompt = format!(
            "You are an expert data analyst. Analyze the following dataset using {} analysis methods.\n\n\
             **DATASET INFORMATION**:\n\
             - Rows: {}\n\
             - Columns: {}\n\
             - Format: {}\n\n\
             **DATA PREVIEW**:\n{}\n\n\
             **ANALYSIS TYPE**: {}\n\n\
             **INSTRUCTIONS**:\n{}\n\n\
             **REQUIREMENTS**:\n\
             1. Base your analysis on the data preview shown above\n\
             2. Provide specific numerical insights where possible\n\
             3. Identify patterns, trends, and anomalies\n\
             4. Explain statistical significance of findings\n\
             5. Offer actionable insights for decision-making\n\
             6. Structure your report with clear sections and bullet points\n\n\
             Generate a comprehensive statistical analysis report now:",
            analysis_type,
            dataset.rows,
            dataset.columns,
            dataset.format,
            dataset.preview,
            analysis_type,
            analysis_instructions
        );

        // Assemble memory context (Task 13.11.2)
        let memory_context = if let (true, Some(sid)) = (memory_enabled, session_id) {
            if let Some(bridge) = context.context_bridge() {
                debug!(
                    "Assembling memory context: session={}, budget={}",
                    sid, context_budget
                );
                crate::assemble_template_context(&bridge, &analysis_prompt, sid, context_budget)
                    .await
            } else {
                warn!("Memory enabled but ContextBridge unavailable");
                vec![]
            }
        } else {
            vec![]
        };

        let memory_context_str = if !memory_context.is_empty() {
            memory_context
                .iter()
                .map(|msg| format!("{}: {}", msg.role, msg.content))
                .collect::<Vec<_>>()
                .join("\n\n")
        } else {
            String::new()
        };

        // Prepend memory context to prompt
        let analysis_prompt = if !memory_context_str.is_empty() {
            format!("{}\n\n{}", memory_context_str, analysis_prompt)
        } else {
            analysis_prompt
        };

        // Execute the agent
        info!("Executing statistical analysis agent...");
        let agent_input = AgentInput::builder().text(analysis_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Analysis agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Extract analysis text
        let analysis_text = agent_output.text;

        info!(
            "Statistical analysis complete ({} characters)",
            analysis_text.len()
        );

        Ok(AnalysisResult {
            text: analysis_text,
            metrics: vec![
                ("rows".to_string(), dataset.rows as f64),
                ("columns".to_string(), dataset.columns as f64),
            ],
        })
    }

    /// Phase 3: Generate chart with visualizer agent
    #[allow(clippy::too_many_arguments)]
    async fn generate_chart(
        &self,
        dataset: &DataSet,
        analysis: &AnalysisResult,
        chart_type: &str,
        provider_config: &llmspell_config::ProviderConfig,
        context: &ExecutionContext,
        session_id: Option<&str>,
        memory_enabled: bool,
        context_budget: i64,
    ) -> Result<ChartResult> {
        use llmspell_agents::factory::{AgentConfig, ModelConfig, ResourceLimits};
        use llmspell_core::types::AgentInput;

        info!("Creating visualization agent (type: {})", chart_type);

        // Extract model from provider config
        let model = provider_config
            .default_model
            .as_ref()
            .ok_or_else(|| TemplateError::Config("provider missing model".into()))?;

        // Parse model parameter
        let (provider, model_id) = if let Some(slash_pos) = model.find('/') {
            (
                model[..slash_pos].to_string(),
                model[slash_pos + 1..].to_string(),
            )
        } else {
            (provider_config.provider_type.clone(), model.to_string())
        };

        // Create visualizer agent config
        let agent_config = AgentConfig {
            name: "data-visualizer-agent".to_string(),
            description: format!("Visualization agent for {} chart generation", chart_type),
            agent_type: "llm".to_string(),
            model: Some(ModelConfig {
                provider,
                model_id,
                temperature: provider_config.temperature.or(Some(0.5)), // Creative for chart design
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

        // Create the agent
        let agent = context
            .agent_registry()
            .create_agent(agent_config)
            .await
            .map_err(|e| {
                warn!("Failed to create visualizer agent: {}", e);
                TemplateError::ExecutionFailed(format!("Agent creation failed: {}", e))
            })?;

        // Build visualization prompt with chart type-specific instructions
        let viz_instructions = match chart_type {
            "bar" => {
                "Create a text-based BAR CHART showing:\n\
                - Horizontal bars with ASCII characters (█ for full blocks)\n\
                - Labels for each category\n\
                - Scale/legend showing value ranges\n\
                - Title and axis labels"
            }
            "line" => {
                "Create a text-based LINE CHART showing:\n\
                - Trend line using ASCII characters (*, -, |)\n\
                - X-axis (time/sequence) and Y-axis (values)\n\
                - Data points marked clearly\n\
                - Title and axis labels"
            }
            "scatter" => {
                "Create a text-based SCATTER PLOT showing:\n\
                - Points plotted using * or • characters\n\
                - X and Y axes with scale markers\n\
                - Relationship between two variables\n\
                - Title and axis labels"
            }
            "histogram" => {
                "Create a text-based HISTOGRAM showing:\n\
                - Vertical bars representing frequency bins\n\
                - ASCII characters (█, ▓, ▒, ░) for bar heights\n\
                - Bin ranges on X-axis\n\
                - Frequency counts on Y-axis"
            }
            "heatmap" => {
                "Create a text-based HEATMAP showing:\n\
                - Grid of values with color/shade indicators\n\
                - Characters representing intensity (█ ▓ ▒ ░ ·)\n\
                - Row and column labels\n\
                - Legend explaining intensity mapping"
            }
            "box" => {
                "Create a text-based BOX PLOT showing:\n\
                - Box-and-whisker diagram using ASCII (├─┼─┤)\n\
                - Median line, quartile boxes, whiskers\n\
                - Outliers marked with ○\n\
                - Labels for each variable"
            }
            _ => {
                "Create a text-based CHART showing:\n\
                - Clear visual representation of the data\n\
                - ASCII characters for visualization\n\
                - Title, labels, and legend"
            }
        };

        let viz_prompt = format!(
            "You are an expert data visualizer. Generate a text-based visualization for the following dataset.\n\n\
             **DATASET INFORMATION**:\n\
             - Rows: {}\n\
             - Columns: {}\n\
             - Format: {}\n\n\
             **DATA PREVIEW**:\n{}\n\n\
             **STATISTICAL ANALYSIS RESULTS**:\n{}\n\n\
             **CHART TYPE**: {}\n\n\
             **INSTRUCTIONS**:\n{}\n\n\
             **OUTPUT REQUIREMENTS**:\n\
             - Generate actual ASCII-based chart (not just a description)\n\
             - Use box-drawing characters and block elements for visual appeal\n\
             - Include title, axis labels, and legend\n\
             - Make the chart readable and informative\n\
             - Width should be 60-80 characters max for terminal display\n\
             - Height should be 15-25 lines for readability\n",
            dataset.rows,
            dataset.columns,
            dataset.format,
            dataset.preview,
            analysis.text,
            chart_type,
            viz_instructions
        );

        // Assemble memory context (Task 13.11.2)
        let memory_context = if let (true, Some(sid)) = (memory_enabled, session_id) {
            if let Some(bridge) = context.context_bridge() {
                debug!(
                    "Assembling memory context: session={}, budget={}",
                    sid, context_budget
                );
                crate::assemble_template_context(&bridge, &viz_prompt, sid, context_budget).await
            } else {
                warn!("Memory enabled but ContextBridge unavailable");
                vec![]
            }
        } else {
            vec![]
        };

        let memory_context_str = if !memory_context.is_empty() {
            memory_context
                .iter()
                .map(|msg| format!("{}: {}", msg.role, msg.content))
                .collect::<Vec<_>>()
                .join("\n\n")
        } else {
            String::new()
        };

        // Prepend memory context to prompt
        let viz_prompt = if !memory_context_str.is_empty() {
            format!("{}\n\n{}", memory_context_str, viz_prompt)
        } else {
            viz_prompt
        };

        // Execute the agent
        info!("Executing visualization agent...");
        let agent_input = AgentInput::builder().text(viz_prompt).build();
        let agent_output = agent
            .execute(agent_input, llmspell_core::ExecutionContext::default())
            .await
            .map_err(|e| {
                warn!("Visualizer agent execution failed: {}", e);
                TemplateError::ExecutionFailed(format!("Agent execution failed: {}", e))
            })?;

        // Extract chart from agent output
        let chart_content = agent_output.text;

        info!(
            "Visualization complete ({} characters)",
            chart_content.len()
        );

        // Build chart description from visualization
        let description = format!(
            "{} Chart\nDataset: {} rows x {} columns\nFormat: {}",
            chart_type, dataset.rows, dataset.columns, dataset.format
        );

        Ok(ChartResult {
            chart_type: chart_type.to_string(),
            description,
            chart_data: chart_content,
        })
    }

    /// Format final report
    fn format_report(
        &self,
        data_file: &str,
        analysis_type: &str,
        chart_type: &str,
        analysis: &AnalysisResult,
        chart: &ChartResult,
    ) -> String {
        format!(
            "# Data Analysis Report\n\n\
             **Data Source**: {}\n\
             **Analysis Type**: {}\n\
             **Visualization**: {}\n\n\
             ---\n\n\
             ## Statistical Analysis\n\n\
             {}\n\n\
             ---\n\n\
             ## Visualization\n\n\
             {}\n\n\
             {}\n\n\
             ---\n\n\
             Generated by LLMSpell Data Analysis Template\n",
            data_file,
            analysis_type,
            chart_type,
            analysis.text,
            chart.description,
            chart.chart_data
        )
    }

    /// Save artifacts to output directory
    fn save_artifacts(
        &self,
        output_dir: &std::path::Path,
        report: &str,
        chart_data: &str,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save analysis report
        let report_path = output_dir.join("analysis_report.md");
        fs::write(&report_path, report).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write analysis report: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            report_path.to_string_lossy().to_string(),
            report.to_string(),
            "text/markdown".to_string(),
        ));

        // Save chart data
        let chart_path = output_dir.join("visualization.txt");
        fs::write(&chart_path, chart_data).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write chart data: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            chart_path.to_string_lossy().to_string(),
            chart_data.to_string(),
            "text/plain".to_string(),
        ));

        Ok(())
    }
}

/// Dataset loaded from file
#[derive(Debug, Clone)]
struct DataSet {
    /// Number of rows
    rows: usize,
    /// Number of columns
    columns: usize,
    /// File format (csv, json, excel)
    format: String,
    /// Data preview
    /// Placeholder for future data loading
    #[allow(dead_code)]
    preview: String,
}

/// Analysis result from analyzer agent
#[derive(Debug, Clone)]
struct AnalysisResult {
    /// Analysis text/report
    text: String,
    /// Statistical metrics
    /// Placeholder for future agent integration
    #[allow(dead_code)]
    metrics: Vec<(String, f64)>,
}

/// Chart result from visualizer agent
#[derive(Debug, Clone)]
struct ChartResult {
    /// Chart type
    /// Placeholder for future visualizer agent integration
    #[allow(dead_code)]
    chart_type: String,
    /// Chart description
    description: String,
    /// Chart data (image or data structure)
    chart_data: String,
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
        let template = DataAnalysisTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "data-analysis");
        assert_eq!(metadata.name, "Data Analysis");
        assert_eq!(metadata.category, TemplateCategory::Analysis);
        assert!(metadata.requires.contains(&"data-loader".to_string()));
        assert!(metadata.requires.contains(&"stats".to_string()));
        assert!(metadata.tags.contains(&"analysis".to_string()));
        assert!(metadata.tags.contains(&"statistics".to_string()));
        assert!(metadata.tags.contains(&"visualization".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = DataAnalysisTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("data_file").is_some());
        assert!(schema.get_parameter("analysis_type").is_some());
        assert!(schema.get_parameter("chart_type").is_some());
        assert!(schema.get_parameter("model").is_some());

        // data_file is required
        let data_file_param = schema.get_parameter("data_file").unwrap();
        assert!(data_file_param.required);

        // others are optional
        let analysis_param = schema.get_parameter("analysis_type").unwrap();
        assert!(!analysis_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate() {
        let template = DataAnalysisTemplate::new();
        let params = TemplateParams::new();

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        assert!(estimate.estimated_cost_usd.is_some());
        assert!(estimate.estimated_duration_ms.is_some());
        assert!(estimate.confidence > 0.0);

        // Verify calculation (1500 + 1000 = 2500 tokens)
        assert_eq!(estimate.estimated_tokens, Some(2500));
    }

    #[test]
    fn test_parameter_validation_missing_required() {
        let template = DataAnalysisTemplate::new();
        let schema = template.config_schema();
        let params = std::collections::HashMap::new();

        // Should fail - missing required "data_file" parameter
        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_invalid_enum() {
        let template = DataAnalysisTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("data_file".to_string(), serde_json::json!("data.csv"));
        params.insert(
            "analysis_type".to_string(),
            serde_json::json!("invalid_type"),
        ); // Not in allowed values

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = DataAnalysisTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("data_file".to_string(), serde_json::json!("data.csv"));
        params.insert(
            "analysis_type".to_string(),
            serde_json::json!("descriptive"),
        );
        params.insert("chart_type".to_string(), serde_json::json!("bar"));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_load_data_placeholder() {
        let template = DataAnalysisTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let dataset = template.load_data("data.csv", &context).await;
        assert!(dataset.is_ok());
        let dataset = dataset.unwrap();
        assert_eq!(dataset.rows, 100);
        assert_eq!(dataset.columns, 5);
        assert_eq!(dataset.format, "csv");
    }

    #[tokio::test]
    async fn test_run_analysis_placeholder() {
        let template = DataAnalysisTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let dataset = DataSet {
            rows: 100,
            columns: 5,
            format: "csv".to_string(),
            preview: "".to_string(),
        };

        let result = template
            .run_analysis(
                &dataset,
                "descriptive",
                &test_provider_config(),
                &context,
                None,
                false,
                2000,
            )
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.text.contains("Descriptive Statistics"));
    }

    #[tokio::test]
    async fn test_generate_chart_placeholder() {
        let template = DataAnalysisTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            // Skip if infrastructure not available
            return;
        }
        let context = context.unwrap();

        let dataset = DataSet {
            rows: 100,
            columns: 5,
            format: "csv".to_string(),
            preview: "".to_string(),
        };

        let analysis = AnalysisResult {
            text: "Test analysis".to_string(),
            metrics: vec![],
        };

        let result = template
            .generate_chart(
                &dataset,
                &analysis,
                "bar",
                &test_provider_config(),
                &context,
                None,
                false,
                2000,
            )
            .await;
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.chart_type, "bar");
        assert!(result.description.contains("Bar Chart"));
    }

    #[test]
    fn test_format_report() {
        let template = DataAnalysisTemplate::new();
        let analysis = AnalysisResult {
            text: "Test analysis text".to_string(),
            metrics: vec![],
        };
        let chart = ChartResult {
            chart_type: "bar".to_string(),
            description: "Bar chart description".to_string(),
            chart_data: "chart data".to_string(),
        };

        let report = template.format_report("data.csv", "descriptive", "bar", &analysis, &chart);

        assert!(report.contains("# Data Analysis Report"));
        assert!(report.contains("data.csv"));
        assert!(report.contains("descriptive"));
        assert!(report.contains("bar"));
        assert!(report.contains("Test analysis text"));
        assert!(report.contains("Bar chart description"));
    }
}
