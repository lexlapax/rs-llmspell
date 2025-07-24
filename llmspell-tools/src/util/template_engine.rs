//! ABOUTME: Template rendering tool with multiple engine support
//! ABOUTME: Provides safe template rendering with Tera and Handlebars engines

use async_trait::async_trait;
use handlebars::Handlebars;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_bool_with_default, extract_optional_object, extract_optional_string,
        extract_parameters, extract_required_string,
    },
    response::ResponseBuilder,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tera::{Context as TeraContext, Tera};
use tracing::info;

/// Supported template engines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateEngine {
    Tera,
    Handlebars,
}

impl std::fmt::Display for TemplateEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateEngine::Tera => write!(f, "tera"),
            TemplateEngine::Handlebars => write!(f, "handlebars"),
        }
    }
}

impl std::str::FromStr for TemplateEngine {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tera" => Ok(TemplateEngine::Tera),
            "handlebars" | "hbs" => Ok(TemplateEngine::Handlebars),
            _ => Err(validation_error(
                format!("Unknown template engine: {}", s),
                Some("engine".to_string()),
            )),
        }
    }
}

/// Template engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateEngineConfig {
    /// Default template engine to use
    #[serde(default = "default_engine")]
    pub default_engine: TemplateEngine,

    /// Whether to auto-escape HTML
    #[serde(default = "default_auto_escape")]
    pub auto_escape: bool,

    /// Maximum template size in bytes
    #[serde(default = "default_max_template_size")]
    pub max_template_size: usize,

    /// Maximum context size in bytes
    #[serde(default = "default_max_context_size")]
    pub max_context_size: usize,

    /// Maximum rendering time in milliseconds
    #[serde(default = "default_max_render_time_ms")]
    pub max_render_time_ms: u64,

    /// Whether to allow custom filters
    #[serde(default = "default_allow_custom_filters")]
    pub allow_custom_filters: bool,
}

fn default_engine() -> TemplateEngine {
    TemplateEngine::Tera
}
fn default_auto_escape() -> bool {
    true
}
fn default_max_template_size() -> usize {
    1024 * 1024
} // 1MB
fn default_max_context_size() -> usize {
    10 * 1024 * 1024
} // 10MB
fn default_max_render_time_ms() -> u64 {
    5000
} // 5 seconds
fn default_allow_custom_filters() -> bool {
    true
}

impl Default for TemplateEngineConfig {
    fn default() -> Self {
        Self {
            default_engine: default_engine(),
            auto_escape: default_auto_escape(),
            max_template_size: default_max_template_size(),
            max_context_size: default_max_context_size(),
            max_render_time_ms: default_max_render_time_ms(),
            allow_custom_filters: default_allow_custom_filters(),
        }
    }
}

/// Template engine tool for rendering templates
pub struct TemplateEngineTool {
    metadata: ComponentMetadata,
    config: TemplateEngineConfig,
}

impl TemplateEngineTool {
    /// Create a new template engine tool
    pub fn new() -> Self {
        Self::with_config(TemplateEngineConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: TemplateEngineConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "template-engine-tool".to_string(),
                "Render templates using Tera or Handlebars engines".to_string(),
            ),
            config,
        }
    }

    /// Register built-in filters and helpers
    fn register_builtin_filters(_tera: &mut Tera, handlebars: &mut Handlebars) {
        // Tera filters are registered by default

        // Register some useful Handlebars helpers
        handlebars.register_helper(
            "uppercase",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    if let Some(param) = h.param(0) {
                        out.write(&param.value().as_str().unwrap_or("").to_uppercase())?;
                    }
                    Ok(())
                },
            ),
        );

        handlebars.register_helper(
            "lowercase",
            Box::new(
                |h: &handlebars::Helper,
                 _: &Handlebars,
                 _: &handlebars::Context,
                 _: &mut handlebars::RenderContext,
                 out: &mut dyn handlebars::Output|
                 -> handlebars::HelperResult {
                    if let Some(param) = h.param(0) {
                        out.write(&param.value().as_str().unwrap_or("").to_lowercase())?;
                    }
                    Ok(())
                },
            ),
        );
    }

    /// Validate template and context sizes
    fn validate_sizes(&self, template: &str, context: &Value) -> Result<()> {
        if template.len() > self.config.max_template_size {
            return Err(validation_error(
                format!(
                    "Template size {} exceeds maximum {}",
                    template.len(),
                    self.config.max_template_size
                ),
                Some("template".to_string()),
            ));
        }

        let context_size = serde_json::to_string(context)?.len();
        if context_size > self.config.max_context_size {
            return Err(validation_error(
                format!(
                    "Context size {} exceeds maximum {}",
                    context_size, self.config.max_context_size
                ),
                Some("context".to_string()),
            ));
        }

        Ok(())
    }

    /// Render template using Tera engine
    fn render_tera(&self, template: &str, context: &Value) -> Result<String> {
        // Create a temporary Tera instance for this template
        let mut tera = Tera::default();

        // Add the template with proper extension for auto-escaping
        let template_name = if self.config.auto_escape {
            "template.html" // Use .html extension to trigger auto-escape
        } else {
            "template"
        };

        tera.add_raw_template(template_name, template)
            .map_err(|e| {
                validation_error(
                    format!("Invalid Tera template: {}", e),
                    Some("template".to_string()),
                )
            })?;

        // Convert JSON context to Tera context
        let tera_context = TeraContext::from_value(context.clone()).map_err(|e| {
            validation_error(
                format!("Invalid context for Tera: {}", e),
                Some("context".to_string()),
            )
        })?;

        // Render the template
        tera.render(template_name, &tera_context).map_err(|e| {
            tool_error(
                format!("Template rendering failed: {}", e),
                Some("template_engine".to_string()),
            )
        })
    }

    /// Render template using Handlebars engine
    fn render_handlebars(&self, template: &str, context: &Value) -> Result<String> {
        // Create a temporary Handlebars instance for this template
        let mut handlebars = Handlebars::new();
        handlebars.set_strict_mode(true);
        if self.config.auto_escape {
            handlebars.register_escape_fn(handlebars::html_escape);
        }

        // Copy helpers from the main instance
        // Note: In a real implementation, we'd need a way to share helpers
        Self::register_builtin_filters(&mut Tera::default(), &mut handlebars);

        // Render the template directly
        handlebars.render_template(template, context).map_err(|e| {
            tool_error(
                format!("Template rendering failed: {}", e),
                Some("template_engine".to_string()),
            )
        })
    }

    /// Detect template engine from syntax hints
    fn detect_engine(template: &str) -> TemplateEngine {
        // Simple heuristics to detect template engine
        if template.contains("{{#") || template.contains("{{/") || template.contains("{{>") {
            // Handlebars block helpers or partials
            TemplateEngine::Handlebars
        } else if template.contains("{%") || template.contains("{{-") || template.contains("-}}") {
            // Tera/Jinja2 style blocks or whitespace control
            TemplateEngine::Tera
        } else {
            // Default to Tera for simple variable substitution
            TemplateEngine::Tera
        }
    }

    /// Sanitize template to prevent injection attacks
    fn sanitize_template(&self, template: &str) -> Result<String> {
        // Basic sanitization - in production, this would be more comprehensive
        let dangerous_patterns = [
            "system(",
            "exec(",
            "eval(",
            "__import__",
            "subprocess",
            "os.system",
        ];

        for pattern in &dangerous_patterns {
            if template.contains(pattern) {
                return Err(LLMSpellError::Security {
                    message: format!("Potentially dangerous pattern detected: {}", pattern),
                    violation_type: Some("template_injection".to_string()),
                });
            }
        }

        Ok(template.to_string())
    }
}

impl Default for TemplateEngineTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseAgent for TemplateEngineTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Extract parameters
        let template = extract_required_string(params, "input")?;
        let context = extract_optional_object(params, "context")
            .map(|obj| Value::Object(obj.clone()))
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));
        let auto_detect = extract_bool_with_default(params, "auto_detect", true);

        let engine = if let Some(engine_str) = extract_optional_string(params, "engine") {
            engine_str.parse::<TemplateEngine>()?
        } else if auto_detect {
            Self::detect_engine(template)
        } else {
            self.config.default_engine
        };

        info!("Rendering template with {} engine", engine);

        // Validate sizes
        self.validate_sizes(template, &context)?;

        // Sanitize template
        let safe_template = self.sanitize_template(template)?;

        // Render template
        let rendered = match engine {
            TemplateEngine::Tera => self.render_tera(&safe_template, &context)?,
            TemplateEngine::Handlebars => self.render_handlebars(&safe_template, &context)?,
        };

        // Create response using ResponseBuilder
        let response = ResponseBuilder::success("render_template")
            .with_message(format!(
                "Template rendered successfully using {} engine",
                engine
            ))
            .with_result(json!({
                "rendered": rendered,
                "engine": engine.to_string(),
                "template_length": template.len(),
                "context_size": serde_json::to_string(&context)?.len()
            }))
            .with_metadata("engine", json!(engine.to_string()))
            .with_metadata("template_length", json!(template.len()))
            .build();

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(validation_error(
                "No parameters provided",
                Some("parameters".to_string()),
            ));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Template rendering error: {}",
            error
        )))
    }
}

#[async_trait]
impl Tool for TemplateEngineTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Restricted
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "template_engine".to_string(),
            description: "Render templates using Tera or Handlebars engines".to_string(),
            parameters: vec![
                ParameterDef {
                    name: "input".to_string(),
                    description: "Template string to render".to_string(),
                    param_type: ParameterType::String,
                    required: true,
                    default: None,
                },
                ParameterDef {
                    name: "context".to_string(),
                    description: "Context data for template rendering".to_string(),
                    param_type: ParameterType::Object,
                    required: false,
                    default: Some(Value::Object(serde_json::Map::new())),
                },
                ParameterDef {
                    name: "engine".to_string(),
                    description: "Template engine to use (tera or handlebars)".to_string(),
                    param_type: ParameterType::String,
                    required: false,
                    default: Some(Value::String("tera".to_string())),
                },
                ParameterDef {
                    name: "auto_detect".to_string(),
                    description: "Auto-detect template engine from syntax".to_string(),
                    param_type: ParameterType::Boolean,
                    required: false,
                    default: Some(Value::Bool(true)),
                },
            ],
            returns: Some(ParameterType::String),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_engine() {
        // Handlebars patterns
        assert_eq!(
            TemplateEngineTool::detect_engine("{{#if user}}Hello{{/if}}"),
            TemplateEngine::Handlebars
        );
        assert_eq!(
            TemplateEngineTool::detect_engine("{{> partial}}"),
            TemplateEngine::Handlebars
        );

        // Tera patterns
        assert_eq!(
            TemplateEngineTool::detect_engine("{% if user %}Hello{% endif %}"),
            TemplateEngine::Tera
        );
        assert_eq!(
            TemplateEngineTool::detect_engine("{{- name -}}"),
            TemplateEngine::Tera
        );

        // Simple variable - defaults to Tera
        assert_eq!(
            TemplateEngineTool::detect_engine("Hello {{ name }}"),
            TemplateEngine::Tera
        );
    }

    #[tokio::test]
    async fn test_tera_rendering() {
        let tool = TemplateEngineTool::new();

        let params = serde_json::json!({
            "input": "Hello {{ name }}!",
            "context": {
                "name": "World"
            },
            "engine": "tera"
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["rendered"], "Hello World!");
    }

    #[tokio::test]
    async fn test_handlebars_rendering() {
        let tool = TemplateEngineTool::new();

        let params = serde_json::json!({
            "input": "{{#if show}}Hello {{name}}!{{/if}}",
            "context": {
                "name": "Alice",
                "show": true
            },
            "engine": "handlebars"
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        let output: Value = serde_json::from_str(&result.text).unwrap();
        assert!(output["success"].as_bool().unwrap_or(false));
        assert_eq!(output["result"]["rendered"], "Hello Alice!");
    }

    #[tokio::test]
    async fn test_template_sanitization() {
        let tool = TemplateEngineTool::new();

        let params = serde_json::json!({
            "input": "{{ system('rm -rf /') }}",
            "context": {},
            "engine": "tera"
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool.execute(input, ExecutionContext::default()).await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("dangerous pattern"));
    }

    #[tokio::test]
    async fn test_size_limits() {
        let config = TemplateEngineConfig {
            max_template_size: 100, // Very small limit for testing
            ..Default::default()
        };
        let tool = TemplateEngineTool::with_config(config);

        let params = serde_json::json!({
            "input": "a".repeat(200), // Exceeds limit
            "context": {},
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool.execute(input, ExecutionContext::default()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }
}
