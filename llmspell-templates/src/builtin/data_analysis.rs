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
        ConfigSchema::new(vec![
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
        ])
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
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());

        info!(
            "Starting data analysis (file={}, analysis={}, chart={}, model={})",
            data_file, analysis_type, chart_type, model
        );

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
            .run_analysis(&dataset, &analysis_type, &model, &context)
            .await?;
        output.metrics.agents_invoked += 1; // analyzer agent

        // Phase 3: Visualization with visualizer agent
        info!("Phase 3: Generating visualizations...");
        let chart_result = self
            .generate_chart(&dataset, &analysis_result, &chart_type, &model, &context)
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
        // TODO: Implement actual data loading using data-loader tool
        // For now, return placeholder dataset
        warn!(
            "Data loading not yet implemented - using placeholder for file: {}",
            file_path
        );

        // Mock dataset based on file extension
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("csv");

        Ok(DataSet {
            rows: 100,
            columns: 5,
            format: extension.to_string(),
            preview: format!(
                "# Data Preview ({})\n\n\
                 [Placeholder data from {}]\n\n\
                 Columns: column_1, column_2, column_3, column_4, column_5\n\
                 Sample rows:\n\
                   Row 1: value_1_1, value_1_2, value_1_3, value_1_4, value_1_5\n\
                   Row 2: value_2_1, value_2_2, value_2_3, value_2_4, value_2_5\n\
                   ...\n",
                extension, file_path
            ),
        })
    }

    /// Phase 2: Run statistical analysis with analyzer agent
    async fn run_analysis(
        &self,
        dataset: &DataSet,
        analysis_type: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<AnalysisResult> {
        // TODO: Implement actual agent-based analysis
        // For now, return placeholder analysis
        warn!(
            "Statistical analysis not yet implemented - using placeholder for type: {}",
            analysis_type
        );

        let analysis_text = match analysis_type {
            "descriptive" => format!(
                "# Descriptive Statistics Analysis\n\n\
                 ## Dataset Summary\n\
                 - Total Rows: {}\n\
                 - Total Columns: {}\n\
                 - Format: {}\n\n\
                 ## Statistical Measures (Placeholder)\n\
                 - Mean: 45.2\n\
                 - Median: 42.0\n\
                 - Std Dev: 12.3\n\
                 - Min: 10.0\n\
                 - Max: 95.0\n\n\
                 ## Key Insights\n\
                 - Data distribution appears roughly normal\n\
                 - No significant outliers detected\n\
                 - Correlation between column_1 and column_2: 0.75\n",
                dataset.rows, dataset.columns, dataset.format
            ),
            "correlation" => format!(
                "# Correlation Analysis\n\n\
                 ## Correlation Matrix (Placeholder)\n\
                 - column_1 ↔ column_2: 0.75 (strong positive)\n\
                 - column_1 ↔ column_3: -0.32 (weak negative)\n\
                 - column_2 ↔ column_3: 0.15 (very weak positive)\n\n\
                 Dataset: {} rows, {} columns\n",
                dataset.rows, dataset.columns
            ),
            "regression" => format!(
                "# Regression Analysis\n\n\
                 ## Linear Regression Model (Placeholder)\n\
                 - R²: 0.82\n\
                 - Adjusted R²: 0.80\n\
                 - P-value: < 0.001 (significant)\n\
                 - Coefficients: β₀=12.5, β₁=3.2\n\n\
                 Dataset: {} rows, {} columns\n",
                dataset.rows, dataset.columns
            ),
            "timeseries" => format!(
                "# Time Series Analysis\n\n\
                 ## Trend Analysis (Placeholder)\n\
                 - Overall trend: Upward\n\
                 - Seasonality: Detected (period=7)\n\
                 - Autocorrelation: 0.68\n\n\
                 Dataset: {} rows, {} columns\n",
                dataset.rows, dataset.columns
            ),
            "clustering" => format!(
                "# Clustering Analysis\n\n\
                 ## K-Means Clustering (Placeholder)\n\
                 - Optimal clusters: 3\n\
                 - Silhouette score: 0.72\n\
                 - Cluster sizes: 35, 42, 23\n\n\
                 Dataset: {} rows, {} columns\n",
                dataset.rows, dataset.columns
            ),
            _ => format!(
                "# Custom Analysis: {}\n\n\
                 Dataset: {} rows, {} columns\n\
                 [Placeholder analysis results]\n",
                analysis_type, dataset.rows, dataset.columns
            ),
        };

        Ok(AnalysisResult {
            text: analysis_text,
            metrics: vec![("rows".to_string(), dataset.rows as f64)],
        })
    }

    /// Phase 3: Generate chart with visualizer agent
    async fn generate_chart(
        &self,
        dataset: &DataSet,
        _analysis: &AnalysisResult,
        chart_type: &str,
        _model: &str,
        _context: &ExecutionContext,
    ) -> Result<ChartResult> {
        // TODO: Implement actual chart generation with visualizer agent
        // For now, return placeholder chart description
        warn!(
            "Chart generation not yet implemented - using placeholder for type: {}",
            chart_type
        );

        let chart_description = match chart_type {
            "bar" => format!(
                "Bar Chart ({} x {})\n\
                 - X-axis: Categories (columns)\n\
                 - Y-axis: Values\n\
                 - Shows distribution across {} columns\n",
                dataset.columns, dataset.rows, dataset.columns
            ),
            "line" => format!(
                "Line Chart ({} points)\n\
                 - X-axis: Sequential index\n\
                 - Y-axis: Value\n\
                 - Shows trend over {} data points\n",
                dataset.rows, dataset.rows
            ),
            "scatter" => format!(
                "Scatter Plot ({} points)\n\
                 - X-axis: column_1\n\
                 - Y-axis: column_2\n\
                 - Shows relationship between two variables\n",
                dataset.rows
            ),
            "histogram" => format!(
                "Histogram ({} bins)\n\
                 - X-axis: Value ranges\n\
                 - Y-axis: Frequency\n\
                 - Shows distribution of values\n",
                (dataset.rows as f64).sqrt().ceil() as usize
            ),
            "heatmap" => format!(
                "Heatmap ({} x {})\n\
                 - Rows: {} data points\n\
                 - Columns: {} variables\n\
                 - Color intensity represents values\n",
                dataset.rows, dataset.columns, dataset.rows, dataset.columns
            ),
            "box" => format!(
                "Box Plot ({} variables)\n\
                 - Shows quartiles, median, and outliers\n\
                 - Useful for distribution comparison\n",
                dataset.columns
            ),
            _ => format!(
                "Custom Chart: {}\n\
                 Dataset: {} rows x {} columns\n",
                chart_type, dataset.rows, dataset.columns
            ),
        };

        Ok(ChartResult {
            chart_type: chart_type.to_string(),
            description: chart_description,
            chart_data: format!(
                "[Placeholder {} chart data for visualization]\n\
                 In production, this would be actual chart image or data",
                chart_type
            ),
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
             ---\n\n\
             Generated by LLMSpell Data Analysis Template\n",
            data_file, analysis_type, chart_type, analysis.text, chart.description
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
            .run_analysis(&dataset, "descriptive", "ollama/llama3.2:3b", &context)
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
            .generate_chart(&dataset, &analysis, "bar", "ollama/llama3.2:3b", &context)
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
