//! Core template trait and types

use crate::{
    artifacts::Artifact, context::ExecutionContext, error::Result, validation::ConfigSchema,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Template trait - core abstraction for workflow templates
///
/// Templates combine agents, tools, RAG, and workflows into executable patterns that solve
/// common use cases. Each template implements its own parameter schema, validation, and execution logic.
#[async_trait]
pub trait Template: Send + Sync + std::fmt::Debug {
    /// Template metadata (id, name, description, category, version, tags)
    fn metadata(&self) -> &TemplateMetadata;

    /// Configuration schema with parameter types and defaults
    fn config_schema(&self) -> ConfigSchema;

    /// Execute template with parameters and context
    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput>;

    /// Optional: Validate parameters before execution
    fn validate(&self, params: &TemplateParams) -> Result<()> {
        // Default: validate against config_schema
        self.config_schema().validate(&params.values)
    }

    /// Optional: Estimate execution cost (tokens, time)
    async fn estimate_cost(&self, _params: &TemplateParams) -> CostEstimate {
        CostEstimate::unknown()
    }
}

/// Template metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    /// Template ID (e.g., "research-assistant")
    pub id: String,

    /// Human-readable name (e.g., "Research Assistant")
    pub name: String,

    /// Description
    pub description: String,

    /// Category
    pub category: TemplateCategory,

    /// Version (semver)
    pub version: String,

    /// Author (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// Required infrastructure (e.g., ["rag", "local-llm", "web-search"])
    #[serde(default)]
    pub requires: Vec<String>,

    /// Tags for discovery (e.g., ["research", "citations", "multi-source"])
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Template category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TemplateCategory {
    /// Research templates (research assistant, literature review)
    Research,

    /// Chat templates (interactive chat, conversational agents)
    Chat,

    /// Analysis templates (data analysis, visualization)
    Analysis,

    /// Code generation templates (code generator, refactoring)
    CodeGen,

    /// Document processing templates (PDF processing, OCR)
    Document,

    /// Workflow orchestration templates (custom patterns)
    Workflow,

    /// Custom category
    Custom(String),
}

impl std::fmt::Display for TemplateCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Research => write!(f, "research"),
            Self::Chat => write!(f, "chat"),
            Self::Analysis => write!(f, "analysis"),
            Self::CodeGen => write!(f, "codegen"),
            Self::Document => write!(f, "document"),
            Self::Workflow => write!(f, "workflow"),
            Self::Custom(s) => write!(f, "{}", s),
        }
    }
}

/// Template parameters (input values)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateParams {
    /// Parameter values
    pub values: HashMap<String, serde_json::Value>,
}

impl TemplateParams {
    /// Create new empty parameters
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
        }
    }

    /// Insert a parameter value
    pub fn insert(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.values.insert(key.into(), value);
    }

    /// Get parameter value with deserialization
    ///
    /// # Errors
    ///
    /// Returns error if parameter is missing or cannot be deserialized
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self
            .values
            .get(key)
            .ok_or_else(|| crate::error::ValidationError::missing(key))?;

        serde_json::from_value(value.clone()).map_err(|_e| {
            crate::error::ValidationError::type_mismatch(
                key,
                std::any::type_name::<T>(),
                format!("{:?}", value),
            )
            .into()
        })
    }

    /// Get parameter value or return default
    pub fn get_or<T: serde::de::DeserializeOwned>(&self, key: &str, default: T) -> T {
        self.get(key).unwrap_or(default)
    }

    /// Get optional parameter value
    pub fn get_optional<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.get(key).ok()
    }

    /// Check if parameter exists
    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// Get all parameter keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }

    /// Get number of parameters
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if parameters are empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl From<HashMap<String, serde_json::Value>> for TemplateParams {
    fn from(values: HashMap<String, serde_json::Value>) -> Self {
        Self { values }
    }
}

impl From<serde_json::Value> for TemplateParams {
    fn from(value: serde_json::Value) -> Self {
        if let serde_json::Value::Object(map) = value {
            Self {
                values: map.into_iter().collect(),
            }
        } else {
            Self::new()
        }
    }
}

/// Template execution output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateOutput {
    /// Execution result
    pub result: TemplateResult,

    /// Generated artifacts (files, reports, etc.)
    #[serde(default)]
    pub artifacts: Vec<Artifact>,

    /// Output metadata
    pub metadata: OutputMetadata,

    /// Execution metrics
    pub metrics: ExecutionMetrics,
}

impl TemplateOutput {
    /// Create a new template output
    pub fn new(
        result: TemplateResult,
        template_id: String,
        template_version: String,
        params: TemplateParams,
    ) -> Self {
        Self {
            result,
            artifacts: Vec::new(),
            metadata: OutputMetadata {
                template_id,
                template_version,
                executed_at: chrono::Utc::now(),
                parameters: params,
            },
            metrics: ExecutionMetrics::default(),
        }
    }

    /// Add an artifact to the output
    pub fn add_artifact(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Set execution duration
    pub fn set_duration(&mut self, duration_ms: u64) {
        self.metrics.duration_ms = duration_ms;
    }

    /// Set token usage
    pub fn set_tokens(&mut self, tokens: u64) {
        self.metrics.tokens_used = Some(tokens);
    }

    /// Set cost
    pub fn set_cost(&mut self, cost_usd: f64) {
        self.metrics.cost_usd = Some(cost_usd);
    }

    /// Add custom metric
    pub fn add_metric(&mut self, key: impl Into<String>, value: serde_json::Value) {
        self.metrics.custom_metrics.insert(key.into(), value);
    }
}

/// Template execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TemplateResult {
    /// Plain text result
    Text(String),

    /// Structured JSON result
    Structured(serde_json::Value),

    /// File path result
    File(PathBuf),

    /// Multiple results
    Multiple(Vec<TemplateResult>),
}

impl TemplateResult {
    /// Create a text result
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    /// Create a structured result
    pub fn structured(value: serde_json::Value) -> Self {
        Self::Structured(value)
    }

    /// Create a file result
    pub fn file(path: PathBuf) -> Self {
        Self::File(path)
    }

    /// Create a multiple result
    pub fn multiple(results: Vec<TemplateResult>) -> Self {
        Self::Multiple(results)
    }
}

/// Output metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputMetadata {
    /// Template ID that generated this output
    pub template_id: String,

    /// Template version
    pub template_version: String,

    /// Execution timestamp
    pub executed_at: chrono::DateTime<chrono::Utc>,

    /// Parameters used
    pub parameters: TemplateParams,
}

/// Execution metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExecutionMetrics {
    /// Execution duration in milliseconds
    #[serde(default)]
    pub duration_ms: u64,

    /// Tokens used (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_used: Option<u64>,

    /// Estimated cost in USD (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_usd: Option<f64>,

    /// Number of agents invoked
    #[serde(default)]
    pub agents_invoked: usize,

    /// Number of tools invoked
    #[serde(default)]
    pub tools_invoked: usize,

    /// Number of RAG queries
    #[serde(default)]
    pub rag_queries: usize,

    /// Custom metrics
    #[serde(default)]
    pub custom_metrics: HashMap<String, serde_json::Value>,
}

/// Cost estimate for template execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    /// Estimated tokens
    pub estimated_tokens: Option<u64>,

    /// Estimated cost in USD
    pub estimated_cost_usd: Option<f64>,

    /// Estimated duration in milliseconds
    pub estimated_duration_ms: Option<u64>,

    /// Confidence level (0.0-1.0)
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}

fn default_confidence() -> f64 {
    0.5
}

impl CostEstimate {
    /// Create an unknown cost estimate
    pub fn unknown() -> Self {
        Self {
            estimated_tokens: None,
            estimated_cost_usd: None,
            estimated_duration_ms: None,
            confidence: 0.0,
        }
    }

    /// Create a cost estimate with all values
    pub fn new(tokens: u64, cost_usd: f64, duration_ms: u64, confidence: f64) -> Self {
        Self {
            estimated_tokens: Some(tokens),
            estimated_cost_usd: Some(cost_usd),
            estimated_duration_ms: Some(duration_ms),
            confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_params_basic() {
        let mut params = TemplateParams::new();
        assert!(params.is_empty());

        params.insert("topic", json!("Rust"));
        assert_eq!(params.len(), 1);
        assert!(params.contains("topic"));

        let topic: String = params.get("topic").unwrap();
        assert_eq!(topic, "Rust");
    }

    #[test]
    fn test_template_params_get_or() {
        let params = TemplateParams::new();
        let default_value: usize = params.get_or("max_sources", 10);
        assert_eq!(default_value, 10);
    }

    #[test]
    fn test_template_params_type_conversion() {
        let mut params = TemplateParams::new();
        params.insert("count", json!(42));

        let count: i64 = params.get("count").unwrap();
        assert_eq!(count, 42);

        // Wrong type should fail
        let result: Result<String> = params.get("count");
        assert!(result.is_err());
    }

    #[test]
    fn test_template_params_from_json() {
        let json_value = json!({
            "topic": "Rust",
            "max_sources": 10,
            "include_citations": true
        });

        let params: TemplateParams = json_value.into();
        assert_eq!(params.len(), 3);

        let topic: String = params.get("topic").unwrap();
        assert_eq!(topic, "Rust");
    }

    #[test]
    fn test_template_category_display() {
        assert_eq!(TemplateCategory::Research.to_string(), "research");
        assert_eq!(TemplateCategory::Chat.to_string(), "chat");
        assert_eq!(TemplateCategory::Analysis.to_string(), "analysis");
        assert_eq!(
            TemplateCategory::Custom("test".to_string()).to_string(),
            "test"
        );
    }

    #[test]
    fn test_template_result_creation() {
        let text_result = TemplateResult::text("Hello");
        assert!(matches!(text_result, TemplateResult::Text(_)));

        let structured_result = TemplateResult::structured(json!({"key": "value"}));
        assert!(matches!(structured_result, TemplateResult::Structured(_)));

        let file_result = TemplateResult::file(PathBuf::from("/tmp/test.txt"));
        assert!(matches!(file_result, TemplateResult::File(_)));
    }

    #[test]
    fn test_template_output_builder() {
        let params = TemplateParams::new();
        let mut output = TemplateOutput::new(
            TemplateResult::text("Test result"),
            "test-template".to_string(),
            "0.1.0".to_string(),
            params,
        );

        output.set_duration(1000);
        output.set_tokens(500);
        output.set_cost(0.01);
        output.add_metric("custom", json!("value"));

        assert_eq!(output.metrics.duration_ms, 1000);
        assert_eq!(output.metrics.tokens_used, Some(500));
        assert_eq!(output.metrics.cost_usd, Some(0.01));
        assert_eq!(
            output.metrics.custom_metrics.get("custom"),
            Some(&json!("value"))
        );
    }

    #[test]
    fn test_cost_estimate() {
        let unknown = CostEstimate::unknown();
        assert_eq!(unknown.confidence, 0.0);
        assert!(unknown.estimated_tokens.is_none());

        let known = CostEstimate::new(1000, 0.05, 5000, 0.8);
        assert_eq!(known.estimated_tokens, Some(1000));
        assert_eq!(known.estimated_cost_usd, Some(0.05));
        assert_eq!(known.estimated_duration_ms, Some(5000));
        assert_eq!(known.confidence, 0.8);
    }
}
