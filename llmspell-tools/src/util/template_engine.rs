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
use std::time::Instant;
use tera::{Context as TeraContext, Tera};
use tracing::{debug, info, instrument, trace};

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
            Self::Tera => write!(f, "tera"),
            Self::Handlebars => write!(f, "handlebars"),
        }
    }
}

impl std::str::FromStr for TemplateEngine {
    type Err = LLMSpellError;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "tera" => Ok(Self::Tera),
            "handlebars" | "hbs" => Ok(Self::Handlebars),
            _ => Err(validation_error(
                format!("Unknown template engine: {s}"),
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

const fn default_engine() -> TemplateEngine {
    TemplateEngine::Tera
}
const fn default_auto_escape() -> bool {
    true
}
const fn default_max_template_size() -> usize {
    1024 * 1024
} // 1MB
const fn default_max_context_size() -> usize {
    10 * 1024 * 1024
} // 10MB
const fn default_max_render_time_ms() -> u64 {
    5000
} // 5 seconds
const fn default_allow_custom_filters() -> bool {
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
    #[must_use]
    pub fn new() -> Self {
        info!(
            tool_name = "template-engine-tool",
            default_engine = "tera",
            supported_engines = 2,    // Tera, Handlebars
            max_template_size_mb = 1, // 1MB default
            max_context_size_mb = 10, // 10MB default
            max_render_time_seconds = 5,
            auto_escape = true,
            allow_custom_filters = true,
            security_level = "Restricted",
            category = "Utility",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating TemplateEngineTool with default configuration"
        );
        Self::with_config(TemplateEngineConfig::default())
    }

    /// Create with custom configuration
    #[must_use]
    pub fn with_config(config: TemplateEngineConfig) -> Self {
        debug!(
            default_engine = %config.default_engine,
            auto_escape = config.auto_escape,
            max_template_size_bytes = config.max_template_size,
            max_context_size_bytes = config.max_context_size,
            max_render_time_ms = config.max_render_time_ms,
            allow_custom_filters = config.allow_custom_filters,
            "Creating TemplateEngineTool with custom configuration"
        );
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
                    format!("Invalid Tera template: {e}"),
                    Some("template".to_string()),
                )
            })?;

        // Convert JSON context to Tera context
        let tera_context = TeraContext::from_value(context.clone()).map_err(|e| {
            validation_error(
                format!("Invalid context for Tera: {e}"),
                Some("context".to_string()),
            )
        })?;

        // Render the template
        tera.render(template_name, &tera_context).map_err(|e| {
            tool_error(
                format!("Template rendering failed: {e}"),
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
                format!("Template rendering failed: {e}"),
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
    #[allow(clippy::unused_self)]
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
                    message: format!("Potentially dangerous pattern detected: {pattern}"),
                    violation_type: Some("template_injection".to_string()),
                });
            }
        }

        Ok(template.to_string())
    }
}

impl Default for TemplateEngineTool {
    fn default() -> Self {
        info!(
            tool_name = "template-engine",
            category = "Tool",
            phase = "Phase 3 (comprehensive instrumentation)",
            "Creating TemplateEngineTool"
        );

        Self::new()
    }
}

impl TemplateEngineTool {
    fn extract_template_params(params: &Value) -> Result<(String, Value)> {
        let template = extract_required_string(params, "input")?;
        trace!(
            template_length = template.len(),
            template_preview = %&template[..template.len().min(100)],
            "Extracted template parameter"
        );

        let context = extract_optional_object(params, "context").map_or_else(
            || {
                debug!("No context provided, using empty object");
                Value::Object(serde_json::Map::new())
            },
            |obj| {
                debug!(
                    context_keys = obj.len(),
                    context_size_estimate = serde_json::to_string(obj).unwrap_or_default().len(),
                    "Extracted context parameter"
                );
                Value::Object(obj.clone())
            },
        );

        Ok((template.to_string(), context))
    }

    fn determine_template_engine(
        &self,
        params: &Value,
        template: &str,
        auto_detect: bool,
    ) -> Result<TemplateEngine> {
        let engine_detection_start = Instant::now();

        let engine = self.get_template_engine(params, template, auto_detect)?;

        let engine_detection_duration_ms = engine_detection_start.elapsed().as_millis();
        debug!(
            engine = %engine,
            auto_detect,
            detection_duration_ms = engine_detection_duration_ms,
            "Template engine determined"
        );

        Ok(engine)
    }

    fn get_template_engine(
        &self,
        params: &Value,
        template: &str,
        auto_detect: bool,
    ) -> Result<TemplateEngine> {
        extract_optional_string(params, "engine").map_or_else(
            || {
                if auto_detect {
                    Ok(Self::detect_engine_from_template(template))
                } else {
                    Ok(self.get_default_engine())
                }
            },
            Self::parse_engine_from_params,
        )
    }

    fn parse_engine_from_params(engine_str: &str) -> Result<TemplateEngine> {
        let e = engine_str.parse::<TemplateEngine>()?;
        debug!(
            engine = %e,
            source = "parameter",
            "Template engine specified in parameters"
        );
        Ok(e)
    }

    fn detect_engine_from_template(template: &str) -> TemplateEngine {
        let detected = Self::detect_engine(template);
        debug!(
            detected_engine = %detected,
            template_preview = %&template[..template.len().min(50)],
            source = "auto_detection",
            "Auto-detected template engine from syntax"
        );
        detected
    }

    fn get_default_engine(&self) -> TemplateEngine {
        debug!(
            default_engine = %self.config.default_engine,
            source = "configuration",
            "Using configured default engine"
        );
        self.config.default_engine
    }

    fn validate_and_sanitize(&self, template: &str, context: &Value) -> Result<String> {
        self.perform_size_validation(template, context)?;
        self.perform_template_sanitization(template)
    }

    fn perform_size_validation(&self, template: &str, context: &Value) -> Result<()> {
        let validation_start = Instant::now();
        debug!(
            template_size_bytes = template.len(),
            context_size_estimate = serde_json::to_string(context).unwrap_or_default().len(),
            max_template_size = self.config.max_template_size,
            max_context_size = self.config.max_context_size,
            "Starting size validation"
        );

        self.validate_sizes(template, context)?;
        let validation_duration_ms = validation_start.elapsed().as_millis();
        debug!(validation_duration_ms, "Size validation passed");
        Ok(())
    }

    fn perform_template_sanitization(&self, template: &str) -> Result<String> {
        let sanitization_start = Instant::now();
        debug!("Starting template sanitization");

        let safe_template = self.sanitize_template(template)?;
        let sanitization_duration_ms = sanitization_start.elapsed().as_millis();
        debug!(
            original_length = template.len(),
            sanitized_length = safe_template.len(),
            sanitization_duration_ms,
            "Template sanitization completed"
        );

        Ok(safe_template)
    }

    fn render_with_engine(
        &self,
        engine: TemplateEngine,
        template: &str,
        context: &Value,
    ) -> Result<String> {
        let rendering_start = Instant::now();
        debug!(
            engine = %engine,
            template_size = template.len(),
            "Starting template rendering"
        );

        let result = self.execute_engine_rendering(engine, template, context)?;

        let rendering_duration_ms = rendering_start.elapsed().as_millis();
        debug!(
            engine = %engine,
            template_size = template.len(),
            output_size = result.len(),
            rendering_duration_ms,
            "Template rendering completed successfully"
        );

        Ok(result)
    }

    fn execute_engine_rendering(
        &self,
        engine: TemplateEngine,
        template: &str,
        context: &Value,
    ) -> Result<String> {
        match engine {
            TemplateEngine::Tera => self.render_with_tera(template, context),
            TemplateEngine::Handlebars => self.render_with_handlebars(template, context),
        }
    }

    fn render_with_tera(&self, template: &str, context: &Value) -> Result<String> {
        trace!("Using Tera rendering engine");
        self.render_tera(template, context)
    }

    fn render_with_handlebars(&self, template: &str, context: &Value) -> Result<String> {
        trace!("Using Handlebars rendering engine");
        self.render_handlebars(template, context)
    }

    fn build_template_response(
        engine: TemplateEngine,
        template_len: usize,
        context: &Value,
        rendered: &str,
    ) -> Result<Value> {
        let response_building_start = Instant::now();
        let context_size = serde_json::to_string(context)?.len();

        debug!(
            rendered_size = rendered.len(),
            context_size, "Building response"
        );

        let response = ResponseBuilder::success("render_template")
            .with_message(format!(
                "Template rendered successfully using {engine} engine"
            ))
            .with_result(json!({
                "rendered": rendered,
                "engine": engine.to_string(),
                "template_length": template_len,
                "context_size": context_size
            }))
            .with_metadata("engine", json!(engine.to_string()))
            .with_metadata("template_length", json!(template_len))
            .build();

        let response_building_duration_ms = response_building_start.elapsed().as_millis();
        debug!(response_building_duration_ms, "Response building completed");

        Ok(response)
    }
}

#[async_trait]
impl BaseAgent for TemplateEngineTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[instrument(skip(_context, input, self), fields(tool = %self.metadata().name))]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let execute_start = Instant::now();
        info!(
            tool_name = %self.metadata().name,
            input_text_length = input.text.len(),
            has_parameters = !input.parameters.is_empty(),
            "Starting TemplateEngineTool execution"
        );

        // Get parameters using shared utility
        let params = extract_parameters(&input)?;
        debug!(
            param_count = params.as_object().map_or(0, serde_json::Map::len),
            "Successfully extracted parameters"
        );

        // Extract template and context
        let (template, context) = Self::extract_template_params(params)?;
        let auto_detect = extract_bool_with_default(params, "auto_detect", true);

        // Determine which engine to use
        let engine = self.determine_template_engine(params, &template, auto_detect)?;

        info!(
            engine = %engine,
            template_length = template.len(),
            context_size = serde_json::to_string(&context).unwrap_or_default().len(),
            auto_detect,
            "Rendering template with selected engine"
        );

        // Validate and sanitize
        let safe_template = self.validate_and_sanitize(&template, &context)?;

        // Render template
        let rendered = self.render_with_engine(engine, &safe_template, &context)?;

        // Build response
        let response = Self::build_template_response(engine, template.len(), &context, &rendered)?;

        let total_execution_duration_ms = execute_start.elapsed().as_millis();
        info!(
            engine = %engine,
            template_length = template.len(),
            rendered_size = rendered.len(),
            total_duration_ms = total_execution_duration_ms,
            success = true,
            "TemplateEngineTool execution completed successfully"
        );

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    #[instrument(skip(self))]
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.parameters.is_empty() {
            return Err(validation_error(
                "No parameters provided",
                Some("parameters".to_string()),
            ));
        }
        Ok(())
    }

    #[instrument(skip(self))]
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!(
            "Template rendering error: {error}"
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
