//! ABOUTME: Template rendering tool with multiple engine support
//! ABOUTME: Provides safe template rendering with Tera and Handlebars engines

use async_trait::async_trait;
use handlebars::Handlebars;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
    },
    types::{AgentInput, AgentOutput, ExecutionContext},
    ComponentMetadata, LLMSpellError, Result,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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
            _ => Err(LLMSpellError::Validation {
                message: format!("Unknown template engine: {}", s),
                field: Some("engine".to_string()),
            }),
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
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Template size {} exceeds maximum {}",
                    template.len(),
                    self.config.max_template_size
                ),
                field: Some("template".to_string()),
            });
        }

        let context_size = serde_json::to_string(context)?.len();
        if context_size > self.config.max_context_size {
            return Err(LLMSpellError::Validation {
                message: format!(
                    "Context size {} exceeds maximum {}",
                    context_size, self.config.max_context_size
                ),
                field: Some("context".to_string()),
            });
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
            .map_err(|e| LLMSpellError::Validation {
                message: format!("Invalid Tera template: {}", e),
                field: Some("template".to_string()),
            })?;

        // Convert JSON context to Tera context
        let tera_context =
            TeraContext::from_value(context.clone()).map_err(|e| LLMSpellError::Validation {
                message: format!("Invalid context for Tera: {}", e),
                field: Some("context".to_string()),
            })?;

        // Render the template
        tera.render(template_name, &tera_context)
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Template rendering failed: {}", e),
                tool_name: Some("template_engine".to_string()),
                source: Some(Box::new(e)),
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
        handlebars
            .render_template(template, context)
            .map_err(|e| LLMSpellError::Tool {
                message: format!("Template rendering failed: {}", e),
                tool_name: Some("template_engine".to_string()),
                source: Some(Box::new(e)),
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
        let params =
            input
                .parameters
                .get("parameters")
                .ok_or_else(|| LLMSpellError::Validation {
                    message: "Missing parameters".to_string(),
                    field: Some("parameters".to_string()),
                })?;

        // Extract parameters
        let template = params
            .get("template")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'template' parameter".to_string(),
                field: Some("template".to_string()),
            })?;

        let context = params
            .get("context")
            .cloned()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new()));

        let engine = params
            .get("engine")
            .and_then(|v| v.as_str())
            .map(|s| s.parse::<TemplateEngine>())
            .transpose()?
            .unwrap_or_else(|| {
                if params
                    .get("auto_detect")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true)
                {
                    Self::detect_engine(template)
                } else {
                    self.config.default_engine
                }
            });

        info!("Rendering template with {} engine", engine);

        // Validate sizes
        self.validate_sizes(template, &context)?;

        // Sanitize template
        let safe_template = self.sanitize_template(template)?;

        // Render template
        let result = match engine {
            TemplateEngine::Tera => self.render_tera(&safe_template, &context)?,
            TemplateEngine::Handlebars => self.render_handlebars(&safe_template, &context)?,
        };

        // Create output with metadata
        let mut metadata = llmspell_core::types::OutputMetadata::default();
        metadata
            .extra
            .insert("engine".to_string(), Value::String(engine.to_string()));
        metadata.extra.insert(
            "template_length".to_string(),
            Value::Number(serde_json::Number::from(template.len())),
        );

        Ok(AgentOutput::text(result).with_metadata(metadata))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "No parameters provided".to_string(),
                field: Some("parameters".to_string()),
            });
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
                    name: "template".to_string(),
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
            "template": "Hello {{ name }}!",
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

        assert_eq!(result.text, "Hello World!");
    }

    #[tokio::test]
    async fn test_handlebars_rendering() {
        let tool = TemplateEngineTool::new();

        let params = serde_json::json!({
            "template": "{{#if show}}Hello {{name}}!{{/if}}",
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

        assert_eq!(result.text, "Hello Alice!");
    }

    #[tokio::test]
    async fn test_template_sanitization() {
        let tool = TemplateEngineTool::new();

        let params = serde_json::json!({
            "template": "{{ system('rm -rf /') }}",
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
        let mut config = TemplateEngineConfig::default();
        config.max_template_size = 100; // Very small limit for testing
        let tool = TemplateEngineTool::with_config(config);

        let params = serde_json::json!({
            "template": "a".repeat(200), // Exceeds limit
            "context": {},
        });

        let input = AgentInput::text("").with_parameter("parameters", params);
        let result = tool.execute(input, ExecutionContext::default()).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds maximum"));
    }
}
